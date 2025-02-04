use super::{
    git_object::{blob::Blob, commit::Commit, tree::TreeRecords},
    Delta, GitObject, Sha1Hash, SHA1_HASH_SIZE,
};
use bytes::Bytes;
use flate2::read::ZlibDecoder;
use std::fmt;
use std::io::{Cursor, Read, Seek, SeekFrom};

const MASK_LAST_4: u8 = 0b00001111;
const MASK_LAST_7: u8 = 0b01111111;

#[derive(Debug)]
pub struct PackFile {
    num_objects: u32,
    cursor: Cursor<Vec<u8>>,
}

impl PackFile {
    pub fn new(bytes: Vec<u8>) -> Self {
        let num_bytes: [u8; 4] = bytes[8..12]
            .try_into()
            .expect("Cannot find bytes for number of objects");
        let num_objects = u32::from_be_bytes(num_bytes);

        Self {
            num_objects,
            cursor: Cursor::new(bytes[12..].to_vec()),
        }
    }

    pub fn get_objects(bytes: Vec<u8>) -> Vec<GitObject> {
        let objects: Vec<PackFileObject> = Self::new(bytes).collect();
        expand_deltas(objects)
    }

    fn read_object_header(&mut self) -> Option<(usize, ObjectType)> {
        if self.num_objects == 0 {
            return None;
        }

        let mut byte = super::read_one(&mut self.cursor);
        let obj_type = ObjectType::new(byte);
        let mut len: usize = (byte & MASK_LAST_4) as usize;

        let mut shift = 4;

        while super::msb_is_1(byte) {
            byte = super::read_one(&mut self.cursor);

            let size = (byte & MASK_LAST_7) as usize;
            len += size << shift;
            shift += 7;
        }

        Some((len, obj_type))
    }

    fn read_zlib(&mut self, buf: &mut [u8]) {
        let current = self.cursor.position();

        let mut decoder = ZlibDecoder::new(&mut self.cursor);
        match decoder.read(buf) {
            Ok(n) => {
                if n < buf.len() {
                    eprintln!("Need {} bytes but read {} byte", buf.len(), n);
                    panic!();
                }
            }
            Err(err) => {
                eprintln!("Cannot decompress data from the pack file. {err}");
                panic!();
            }
        }

        let consumed = decoder.total_in();

        self.cursor
            .seek(SeekFrom::Start(current + consumed))
            .expect("Cannot reset cursor position");
    }
}

impl Iterator for PackFile {
    type Item = PackFileObject;

    fn next(&mut self) -> Option<Self::Item> {
        let (len, obj_type) = self.read_object_header()?;

        match obj_type {
            ObjectType::Commit | ObjectType::Tree | ObjectType::Blob => {
                let mut buf = vec![0u8; len];
                self.read_zlib(&mut buf);

                self.num_objects -= 1;

                Some(PackFileObject::undeltified(obj_type, &buf))
            }
            ObjectType::RefDelta => {
                let mut buf = [0u8; SHA1_HASH_SIZE];
                self.cursor
                    .read_exact(&mut buf)
                    .expect("Cannot read refdelta's basename");
                let basename = Sha1Hash::from(buf);

                let mut buf = vec![0u8; len];
                self.read_zlib(&mut buf);

                let mut tmp_cursor = Cursor::new(buf);
                let delta = Delta::new(&mut tmp_cursor);

                self.num_objects -= 1;

                Some(PackFileObject::RefDelta { basename, delta })
            }
            _ => {
                eprintln!("Unexpected object type: {obj_type:?}");
                None
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ObjectType {
    Commit,
    Tree,
    Blob,
    Tag,
    OfsDelta,
    RefDelta,
    Unknown,
}

const MASK_OBJECT_TYPE: u8 = 0b01110000;

impl ObjectType {
    fn new(byte: u8) -> Self {
        match (byte & MASK_OBJECT_TYPE) >> 4 {
            1 => Self::Commit,
            2 => Self::Tree,
            3 => Self::Blob,
            4 => Self::Tag,
            6 => Self::OfsDelta,
            7 => Self::RefDelta,
            _ => Self::Unknown,
        }
    }
}

impl fmt::Display for ObjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Commit => "commit",
            Self::Tree => "tree",
            Self::Blob => "blob",
            Self::Tag => "tag",
            Self::OfsDelta => "ofsdelta",
            Self::RefDelta => "refdelta",
            Self::Unknown => "unknown",
        };
        write!(f, "{value}")
    }
}

#[derive(Debug)]
pub enum PackFileObject {
    GitObject(GitObject),
    RefDelta { basename: Sha1Hash, delta: Delta },
}

impl PackFileObject {
    fn undeltified(obj_type: ObjectType, buf: &[u8]) -> Self {
        match obj_type {
            ObjectType::Commit => GitObject::Commit(Box::new(Commit::from_bytes(buf))).into(),
            ObjectType::Tree => {
                let trees = TreeRecords::new(buf).collect();
                GitObject::Tree(trees).into()
            }
            ObjectType::Blob => {
                let bytes = Bytes::copy_from_slice(buf);
                GitObject::Blob(Blob::from(bytes)).into()
            }
            _ => panic!("Cannot create undeltified PackFileObject from object type: {obj_type}"),
        }
    }
}

impl From<GitObject> for PackFileObject {
    fn from(object: GitObject) -> Self {
        Self::GitObject(object)
    }
}

type DeltaPair = (Sha1Hash, Delta);

fn group_objects(objects: Vec<PackFileObject>) -> (Vec<GitObject>, Vec<DeltaPair>) {
    let mut git_objects: Vec<GitObject> = vec![];
    let mut delta_pairs: Vec<DeltaPair> = vec![];

    for object in objects {
        match object {
            PackFileObject::GitObject(o) => {
                git_objects.push(o);
            }
            PackFileObject::RefDelta { basename, delta } => {
                delta_pairs.push((basename, delta));
            }
        }
    }

    (git_objects, delta_pairs)
}

fn expand_deltas(objects: Vec<PackFileObject>) -> Vec<GitObject> {
    let (mut git_objects, deltas) = group_objects(objects);
    let mut deltas = if deltas.is_empty() {
        None
    } else {
        Some(deltas)
    };

    while deltas.is_some() {
        let mut next_pairs: Vec<DeltaPair> = vec![];
        let pairs = deltas.take().unwrap();
        let pairs_len = pairs.len();
        eprintln!("There are {} deltas left!", pairs_len);

        for (hash, delta) in pairs {
            if let Some(obj) = git_objects.iter().find(|o| o.hash() == hash) {
                let restored = obj.restore(delta).expect("Cannot restore GitObject");
                git_objects.push(restored);
            } else {
                next_pairs.push((hash, delta));
            }
        }

        if next_pairs.len() == pairs_len {
            for o in git_objects.iter() {
                println!("{}", o.hash().hex());
            }
            panic!("Not Found base object for delta");
        }

        deltas = if next_pairs.len() <= 1 {
            None
        } else {
            Some(next_pairs)
        };
    }

    git_objects
}
