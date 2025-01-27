use bytes::Bytes;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Blob(Bytes);

impl From<Bytes> for Blob {
    fn from(value: Bytes) -> Self {
        Self(value)
    }
}

impl fmt::Display for Blob {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let v = unsafe { std::str::from_utf8_unchecked(self.0.as_ref()) };
        write!(f, "{v}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_implements_to_string() {
        let blob = Blob(Bytes::from_static(b"hello"));
        assert_eq!(blob.to_string(), "hello");
    }
}
