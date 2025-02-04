mod delta;
mod pack_file;
mod pkt_line;

use std::io::Read;

pub use delta::Delta;
pub use pack_file::PackFile;
pub use pkt_line::{PktLine, PktLines};

use super::{git_object, GitObject, Sha1Hash, SHA1_HASH_SIZE};

fn read_one<R: Read>(r: &mut R) -> u8 {
    let mut buf = [0u8; 1];
    r.read_exact(&mut buf).expect("Cannot read 1 byte");
    let [byte] = buf;
    byte
}

fn msb_is_1(byte: u8) -> bool {
    byte & 0b10000000 == 0b10000000
}
