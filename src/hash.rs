use super::Error;
use sha1::{Digest, Sha1};

pub const SHA1_HASH_SIZE: usize = 20;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Sha1Hash([u8; SHA1_HASH_SIZE]);

impl Sha1Hash {
    pub fn hasher() -> Sha1 {
        Sha1::new()
    }

    pub fn new(hasher: Sha1) -> Self {
        Self(hasher.finalize().into())
    }

    pub fn hex(&self) -> String {
        hex::encode(self.0)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl TryFrom<&[u8]> for Sha1Hash {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self(bytes.try_into()?))
    }
}

impl From<[u8; SHA1_HASH_SIZE]> for Sha1Hash {
    fn from(value: [u8; SHA1_HASH_SIZE]) -> Self {
        Self(value)
    }
}
