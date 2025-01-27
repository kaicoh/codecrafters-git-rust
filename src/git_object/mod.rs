mod blob;

use super::{Error, Result, GIT_OBJ_DIR};
use blob::Blob;
use bytes::Bytes;
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use sha1::{Digest, Sha1};
use std::fmt;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub enum GitObject {
    Blob(Blob),
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

    pub fn hash(&self) -> String {
        match self {
            Self::Blob(blob) => {
                let mut hasher = Sha1::new();
                let size = blob.len();
                hasher.update(format!("blob {size}\0"));
                let hasher = hasher.chain_update(blob);
                hex::encode(hasher.finalize())
            }
        }
    }

    pub fn write(self) -> Result<()> {
        let hash = self.hash();
        let path = Self::path(&hash)?;

        if let Some(dir) = path.as_path().parent() {
            if !dir.try_exists()? {
                fs::create_dir(dir)?;
            }
        }

        let f = File::create(path)?;
        let mut e = ZlibEncoder::new(f, Compression::default());

        match self {
            Self::Blob(blob) => {
                let size = blob.len();
                e.write_all(format!("blob {size}\0").as_bytes())?;
                e.write_all(blob.as_ref())?;
                e.finish()?;
                Ok(())
            }
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
        } else {
            unimplemented!()
        }
    }
}

impl fmt::Display for GitObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Blob(blob) => blob.fmt(f),
        }
    }
}

fn zero_position(bytes: &[u8]) -> Option<usize> {
    bytes.iter().position(|&b| b == b'\0')
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
        assert_eq!(obj.hash(), expected);
    }
}
