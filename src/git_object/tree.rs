use super::{space_position, zero_position, Error, GitObject};
use sha1::{Digest, Sha1};
use std::{
    cmp::Ordering,
    fmt,
    fs::{DirEntry, File},
    io::{Cursor, Read},
};

const SHA_SIZE: usize = 20;
const MODE_DIR: isize = 40000;
const MODE_FILE: isize = 100644;

type Sha1Hash = [u8; SHA_SIZE];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tree {
    mode: Mode,
    name: String,
    hash: Sha1Hash,
}

impl Tree {
    fn new(buf: &[u8]) -> Self {
        let sp_pos = space_position(buf).expect("buf must have space");
        let zero_pos = zero_position(buf).expect("buf must have \0");

        let mode_str = String::from_utf8_lossy(&buf[..sp_pos]).to_string();
        let mode = match mode_str.parse::<isize>() {
            Ok(MODE_FILE) => Mode::File,
            Ok(MODE_DIR) => Mode::Directory,
            _ => panic!("Unknown mode: {mode_str}"),
        };

        let name = String::from_utf8_lossy(&buf[(sp_pos + 1)..zero_pos]).to_string();

        Self {
            mode,
            name,
            hash: buf[(zero_pos + 1)..]
                .try_into()
                .expect("SHA1 hash must be 20-bytes long"),
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn serialize(&self) -> Vec<u8> {
        let header = format!("{} {}\0", self.mode as isize, self.name);
        [header.as_bytes().to_vec(), self.hash.to_vec()].concat()
    }

    pub fn len(&self) -> usize {
        self.serialize().len()
    }
}

impl TryFrom<DirEntry> for Tree {
    type Error = Error;

    fn try_from(entry: DirEntry) -> Result<Self, Self::Error> {
        let path = entry.path();
        let name = format!("{}", entry.file_name().to_string_lossy());
        let (mode, hash) = if path.is_dir() {
            let obj = GitObject::new_tree(path)?;
            (Mode::Directory, obj.hash())
        } else if path.is_file() {
            let f = File::open(path)?;
            let obj = GitObject::new_blob(f)?;
            (Mode::File, obj.hash())
        } else {
            return Err(Error::from(anyhow::anyhow!(
                "DirEntry is neither directory nor file."
            )));
        };

        Ok(Self {
            mode,
            name,
            hash: hash.try_into().map_err(|err| {
                Error::from(anyhow::anyhow!(
                    "Git object hash must be 20-bytes long. {err:?}"
                ))
            })?,
        })
    }
}

impl PartialOrd for Tree {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.name().cmp(other.name()))
    }
}

impl Ord for Tree {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name().cmp(other.name())
    }
}

impl fmt::Display for Tree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut hasher = Sha1::new();
        hasher.update(self.hash);
        let hash = hex::encode(hasher.finalize());
        write!(
            f,
            "{:06} {} {}    {}",
            self.mode as isize,
            if self.mode == Mode::Directory {
                "tree"
            } else {
                "blob"
            },
            hash,
            self.name,
        )
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Mode {
    File = MODE_FILE,
    Directory = MODE_DIR,
}

#[derive(Debug)]
pub struct TreeRecords<'a> {
    cursor: Cursor<&'a [u8]>,
}

impl<'a> TreeRecords<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            cursor: Cursor::new(bytes),
        }
    }
}

impl Iterator for TreeRecords<'_> {
    type Item = Tree;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.cursor.position() as usize;
        let bytes = *(self.cursor.get_ref());

        // \0 position from the current position
        let zero_pos = zero_position(&bytes[current..])?;
        let tree_size = zero_pos + SHA_SIZE + 1;
        let mut buf: Vec<u8> = vec![0; tree_size];
        self.cursor
            .read_exact(&mut buf)
            .inspect_err(|err| {
                eprintln!("Err reading tree object. {err}");
            })
            .ok()?;
        Some(Tree::new(&buf))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_creates_file_tree() {
        let bytes = b"100644 file1\x0011111111111111111111";
        let tree = Tree::new(bytes);
        let expected = Tree {
            mode: Mode::File,
            name: "file1".into(),
            hash: [b'1'; 20],
        };
        assert_eq!(tree, expected);
    }

    #[test]
    fn it_creates_dir_tree() {
        let bytes = b"40000 dir1\x0099999999999999999999";
        let tree = Tree::new(bytes);
        let expected = Tree {
            mode: Mode::Directory,
            name: "dir1".into(),
            hash: [b'9'; 20],
        };
        assert_eq!(tree, expected);
    }

    #[test]
    fn it_generates_trees_from_tree_records() {
        let bytes = b"100644 file1\x001111111111111111111140000 dir1\x0099999999999999999999";
        let mut trees = TreeRecords::new(bytes);

        let tree = trees.next().unwrap();
        let expected = Tree {
            mode: Mode::File,
            name: "file1".into(),
            hash: [b'1'; 20],
        };
        assert_eq!(tree, expected);

        let tree = trees.next().unwrap();
        let expected = Tree {
            mode: Mode::Directory,
            name: "dir1".into(),
            hash: [b'9'; 20],
        };
        assert_eq!(tree, expected);

        assert!(trees.next().is_none());
    }
}
