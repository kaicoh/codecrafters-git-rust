mod blob;
mod tree;

use super::{Error, Result, GIT_OBJ_DIR};
use blob::Blob;
use bytes::Bytes;
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use sha1::{Digest, Sha1};
use std::ffi::OsStr;
use std::fmt;
use std::fs::{self, DirEntry, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tree::{Tree, TreeRecords};

#[derive(Debug, Clone, PartialEq)]
pub enum GitObject {
    Blob(Blob),
    Tree(Vec<Tree>),
}

impl GitObject {
    pub fn open_from_hash(hash: &str) -> Result<Self> {
        Self::path(hash).and_then(Self::open)
    }

    pub fn new_blob<R: Read>(mut content: R) -> Result<Self> {
        let mut buf = vec![];
        content.read_to_end(&mut buf)?;
        Ok(Self::Blob(Blob::from(Bytes::from_iter(buf))))
    }

    pub fn new_tree<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut trees: Vec<Tree> = vec![];

        for entry in fs::read_dir(path)? {
            let entry = entry?;

            if !is_git_file(&entry) {
                let tree = Tree::try_from(entry)?;
                trees.push(tree);
            }
        }

        trees.sort();

        Ok(Self::Tree(trees))
    }

    pub fn hash(&self) -> Vec<u8> {
        let mut hasher = Sha1::new();
        hasher.update(self.header());

        match self {
            Self::Blob(blob) => {
                hasher = hasher.chain_update(blob);
            }
            Self::Tree(trees) => {
                for tree in trees {
                    hasher = hasher.chain_update(tree.serialize());
                }
            }
        }

        hasher.finalize().to_vec()
    }

    pub fn write(self) -> Result<()> {
        let hash = self.hash();
        let path = Self::path(&hex::encode(hash))?;

        if let Some(dir) = path.as_path().parent() {
            if !dir.try_exists()? {
                fs::create_dir(dir)?;
            }
        }

        let f = File::create(path)?;
        let mut e = ZlibEncoder::new(f, Compression::default());
        e.write_all(self.header().as_bytes())?;

        match self {
            Self::Blob(blob) => {
                e.write_all(blob.as_ref())?;
            }
            Self::Tree(trees) => {
                for tree in trees {
                    e.write_all(&tree.serialize())?;
                }
            }
        }

        e.finish()?;
        Ok(())
    }

    pub fn print_trees(&self, name_only: bool) -> Vec<String> {
        if let Self::Tree(ref trees) = self {
            let mut trees = trees.to_vec();
            trees.sort();
            trees
                .into_iter()
                .map(|tree| {
                    if name_only {
                        tree.name().into()
                    } else {
                        format!("{tree}")
                    }
                })
                .collect()
        } else {
            vec![]
        }
    }

    fn size(&self) -> usize {
        match self {
            Self::Blob(blob) => blob.len(),
            Self::Tree(trees) => trees.iter().map(Tree::len).sum(),
        }
    }

    fn header(&self) -> String {
        let size = self.size();
        match self {
            Self::Blob(_) => format!("blob {size}\0"),
            Self::Tree(_) => format!("tree {size}\0"),
        }
    }

    fn path(hash: &str) -> Result<PathBuf> {
        if hash.len() != 40 {
            return Err(anyhow::anyhow!("SHA-1 hash must be 40-characters long").into());
        }

        Ok(format!("{}/{}/{}", GIT_OBJ_DIR, &hash[..2], &hash[2..]).into())
    }

    fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let f = File::open(path)?;
        let mut decoder = ZlibDecoder::new(f);
        let mut data = vec![];
        decoder.read_to_end(&mut data)?;

        Self::new(data)
    }

    fn new(data: Vec<u8>) -> Result<Self> {
        // NOTE:
        // The blob file is like "blob <size>\0<contents>"
        if data.starts_with(b"blob") {
            let zero_pos =
                zero_position(&data[..]).ok_or(Error::from("Not found \0 in git object file"))?;
            let size = std::str::from_utf8(&data[5..zero_pos])
                .map_err(|err| {
                    let msg = format!("Cannot stringify blob size in git object file. {err}");
                    Error::from(msg.as_str())
                })?
                .parse::<usize>()
                .map_err(|err| {
                    let msg = format!("Parsing error! blob size in git object file. {err}");
                    Error::from(msg.as_str())
                })?;
            let bytes = Bytes::copy_from_slice(&data[(zero_pos + 1)..(zero_pos + 1 + size)]);
            Ok(Self::Blob(Blob::from(bytes)))
        // NOTE:
        // The tree file is like "tree <size>\0...."
        } else if data.starts_with(b"tree") {
            let zero_pos =
                zero_position(&data[..]).ok_or(Error::from("Not found \0 in git object file"))?;
            let trees = TreeRecords::new(&data[(zero_pos + 1)..]).collect::<Vec<Tree>>();
            Ok(Self::Tree(trees))
        } else {
            unimplemented!()
        }
    }
}

impl fmt::Display for GitObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Blob(blob) => blob.fmt(f),
            Self::Tree(_trees) => unimplemented!(),
        }
    }
}

type GetPosition = Box<dyn Fn(&[u8]) -> Option<usize>>;
fn position(byte: u8) -> GetPosition {
    Box::new(move |bytes: &[u8]| bytes.iter().position(|&b| b == byte))
}

fn zero_position(bytes: &[u8]) -> Option<usize> {
    position(b'\0')(bytes)
}

fn space_position(bytes: &[u8]) -> Option<usize> {
    position(b' ')(bytes)
}

fn is_git_file(entry: &DirEntry) -> bool {
    let path = entry.path();
    path.ancestors().any(is_git_root)
}

fn is_git_root<P: AsRef<Path>>(path: P) -> bool {
    let git_root = OsStr::new(".git");
    path.as_ref().file_name().is_some_and(|v| v == git_root)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_creates_filepath_from_sha1_hash() {
        let hash = "e88f7a929cd70b0274c4ea33b209c97fa845fdbc";
        assert_eq!(
            GitObject::path(hash).unwrap(),
            PathBuf::from(".git/objects/e8/8f7a929cd70b0274c4ea33b209c97fa845fdbc")
        );
    }

    #[test]
    fn it_creates_blob_git_object() {
        let bytes = b"blob 11\0hello world";
        let obj = GitObject::new(bytes.to_vec());
        assert_eq!(
            obj.unwrap(),
            GitObject::Blob(Blob::from(Bytes::from_static(b"hello world")))
        );
    }

    #[test]
    fn it_calculates_sha1_hash() {
        let bytes = b"blob 11\0hello world";
        let mut hasher = Sha1::new();
        hasher.update(bytes);
        let expected = hex::encode(hasher.finalize());

        let obj = GitObject::new(bytes.to_vec()).unwrap();
        let hash = hex::encode(obj.hash());
        assert_eq!(hash, expected);
    }
}
