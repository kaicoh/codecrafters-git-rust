use super::{space_position, zero_position, Error, GitObject, Sha1Hash, SHA1_HASH_SIZE};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::{
    cmp::Ordering,
    fmt,
    fs::{DirEntry, File},
    io::{Cursor, Read},
};

const MODE_DIR: isize = 40000;
const MODE_FILE: isize = 100644;
const MODE_SYML: isize = 120000;
#[cfg(unix)]
const MODE_EXEC: isize = 100755;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeNode {
    mode: Mode,
    name: String,
    hash: Sha1Hash,
}

impl TreeNode {
    fn new(buf: &[u8]) -> Self {
        let sp_pos = space_position(buf).expect("buf must have space");
        let zero_pos = zero_position(buf).expect("buf must have \0");

        let mode_str = String::from_utf8_lossy(&buf[..sp_pos]).to_string();
        let mode = match mode_str.parse::<isize>() {
            Ok(MODE_FILE) => Mode::File,
            Ok(MODE_DIR) => Mode::Directory,
            Ok(MODE_SYML) => Mode::Symlink,
            #[cfg(unix)]
            Ok(MODE_EXEC) => Mode::Executable,
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

    pub fn hash(&self) -> Sha1Hash {
        self.hash
    }

    pub fn serialize(&self) -> Vec<u8> {
        let header = format!("{} {}\0", self.mode as isize, self.name);
        [header.as_bytes(), self.hash.as_bytes()].concat()
    }

    pub fn len(&self) -> usize {
        self.serialize().len()
    }
}

impl TryFrom<DirEntry> for TreeNode {
    type Error = Error;

    fn try_from(entry: DirEntry) -> Result<Self, Self::Error> {
        let path = entry.path();
        let name = format!("{}", entry.file_name().to_string_lossy());
        let (mode, hash) = if path.is_dir() {
            let obj = GitObject::new_tree(path)?;
            (Mode::Directory, obj.hash())
        } else if path.is_file() {
            let f = File::open(path)?;
            let mode = if cfg!(unix) {
                Mode::from_file(&f)
            } else {
                Mode::File
            };
            let obj = GitObject::new_blob(f)?;
            (mode, obj.hash())
        } else if path.is_symlink() {
            let f = File::open(path)?;
            let obj = GitObject::new_blob(f)?;
            (Mode::Symlink, obj.hash())
        } else {
            return Err(Error::from(anyhow::anyhow!(
                "DirEntry is not directory, file or symlink."
            )));
        };

        Ok(Self { mode, name, hash })
    }
}

impl PartialOrd for TreeNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.name().cmp(other.name()))
    }
}

impl Ord for TreeNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name().cmp(other.name())
    }
}

impl fmt::Display for TreeNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:06} {} {}    {}",
            self.mode as isize,
            if self.mode == Mode::Directory {
                "tree"
            } else {
                "blob"
            },
            self.hash.hex(),
            self.name,
        )
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Mode {
    File = MODE_FILE,
    Directory = MODE_DIR,
    Symlink = MODE_SYML,
    #[cfg(unix)]
    Executable = MODE_EXEC,
}

impl Mode {
    #[cfg(unix)]
    fn from_file(file: &File) -> Self {
        if file
            .metadata()
            .is_ok_and(|meta| meta.permissions().mode() == 0o755)
        {
            Self::Executable
        } else {
            Self::File
        }
    }
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
    type Item = TreeNode;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.cursor.position() as usize;
        let bytes = *(self.cursor.get_ref());

        // \0 position from the current position
        let zero_pos = zero_position(&bytes[current..])?;
        let tree_size = zero_pos + SHA1_HASH_SIZE + 1;
        let mut buf: Vec<u8> = vec![0; tree_size];
        self.cursor
            .read_exact(&mut buf)
            .inspect_err(|err| {
                eprintln!("Err reading tree object. {err}");
            })
            .ok()?;
        Some(TreeNode::new(&buf))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_creates_file_tree() {
        let bytes = b"100644 file1\x0011111111111111111111";
        let tree = TreeNode::new(bytes);
        let expected = TreeNode {
            mode: Mode::File,
            name: "file1".into(),
            hash: [b'1'; 20].into(),
        };
        assert_eq!(tree, expected);
    }

    #[test]
    fn it_creates_dir_tree() {
        let bytes = b"40000 dir1\x0099999999999999999999";
        let tree = TreeNode::new(bytes);
        let expected = TreeNode {
            mode: Mode::Directory,
            name: "dir1".into(),
            hash: [b'9'; 20].into(),
        };
        assert_eq!(tree, expected);
    }

    #[test]
    fn it_generates_trees_from_tree_records() {
        let bytes = b"100644 file1\x001111111111111111111140000 dir1\x0099999999999999999999";
        let mut trees = TreeRecords::new(bytes);

        let tree = trees.next().unwrap();
        let expected = TreeNode {
            mode: Mode::File,
            name: "file1".into(),
            hash: [b'1'; 20].into(),
        };
        assert_eq!(tree, expected);

        let tree = trees.next().unwrap();
        let expected = TreeNode {
            mode: Mode::Directory,
            name: "dir1".into(),
            hash: [b'9'; 20].into(),
        };
        assert_eq!(tree, expected);

        assert!(trees.next().is_none());
    }
}
