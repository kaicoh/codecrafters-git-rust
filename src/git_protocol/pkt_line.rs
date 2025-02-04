use bytes::Bytes;
use std::fmt;
use std::io::{Cursor, Read, Seek};

#[derive(Debug, Clone, PartialEq)]
pub struct PktLine(Option<Vec<u8>>);

impl PktLine {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(Some(bytes))
    }

    pub fn size(&self) -> usize {
        if let Some(bytes) = self.0.as_ref() {
            bytes.len() + 4
        } else {
            0
        }
    }

    pub fn flush() -> Self {
        Self(None)
    }

    pub fn serialize(&self) -> Vec<u8> {
        if let Some(bytes) = self.0.as_ref() {
            bytes.to_vec()
        } else {
            vec![]
        }
    }

    pub fn split_first(&self) -> Option<(&u8, &[u8])> {
        self.0.as_ref().and_then(|bytes| bytes.split_first())
    }
}

impl From<&[u8]> for PktLine {
    fn from(bytes: &[u8]) -> Self {
        Self::new(bytes.into())
    }
}

impl fmt::Display for PktLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:04x}{}",
            self.size(),
            String::from_utf8_lossy(&self.serialize())
        )
    }
}

#[derive(Debug, Clone)]
pub struct PktLines {
    cursor: Cursor<Vec<u8>>,
}

impl From<Bytes> for PktLines {
    fn from(value: Bytes) -> Self {
        Self::new(value.to_vec())
    }
}

impl PktLines {
    pub fn new(buf: Vec<u8>) -> Self {
        Self {
            cursor: Cursor::new(buf),
        }
    }

    pub fn append(self, buf: Vec<u8>) -> Self {
        let bytes = [self.remaining().to_vec(), buf].concat();
        Self::new(bytes)
    }

    fn remaining(&self) -> &[u8] {
        let curr = self.current();
        &self.cursor.get_ref()[curr..]
    }

    fn current(&self) -> usize {
        self.cursor.position() as usize
    }
}

impl Iterator for PktLines {
    type Item = PktLine;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining().len() < 4 {
            return None;
        }

        let mut buf = [0u8; 4];
        self.cursor
            .read_exact(&mut buf)
            .expect("Cannot read pkt line size");

        let line_len = line_size(&buf[..]);
        if line_len == 0 {
            return Some(PktLine::flush());
        }

        let value_len = line_len - 4;
        if self.remaining().len() < value_len {
            self.cursor
                .seek_relative(-4)
                .expect("Cannot seek back cursor");
            return None;
        }

        let mut buf = vec![0u8; value_len];
        self.cursor
            .read_exact(&mut buf)
            .expect("Cannot read pkt line value");

        Some(PktLine::new(buf))
    }
}

fn line_size(buf: &[u8]) -> usize {
    let len_str = unsafe { std::str::from_utf8_unchecked(buf) };
    usize::from_str_radix(len_str, 16).expect("Cannot parse pkt line size.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_prints_to_pkt_line() {
        let line = PktLine::new(b"foobar\n".into());
        let print = format!("{line}");
        assert_eq!(print, "000bfoobar\n");
    }

    #[test]
    fn it_creats_flush_line() {
        let line = PktLine::flush();
        let print = format!("{line}");
        assert_eq!(print, "0000");
    }

    #[test]
    fn it_retrieves_pkt_lines() {
        let bytes = b"00ab3b1031798a00fdf9b574b5857b1721bc4b0e6bac HEAD\x00multi_ack thin-pack side-band side-band-64k ofs-delta shallow no-progress include-tag multi_ack_detailed agent=git/1.8.1\n003f3b1031798a00fdf9b574b5857b1721bc4b0e6bac refs/heads/master\n0048c4bf7555e2eb4a2b55c7404c742e7e95017ec850 refs/remotes/origin/master\n0000".to_vec();
        let mut lines = PktLines::new(bytes);

        let line = lines.next().unwrap();
        assert_eq!(line, PktLine::new(b"3b1031798a00fdf9b574b5857b1721bc4b0e6bac HEAD\x00multi_ack thin-pack side-band side-band-64k ofs-delta shallow no-progress include-tag multi_ack_detailed agent=git/1.8.1\n".into()));

        let line = lines.next().unwrap();
        assert_eq!(
            line,
            PktLine::new(b"3b1031798a00fdf9b574b5857b1721bc4b0e6bac refs/heads/master\n".into())
        );

        let line = lines.next().unwrap();
        assert_eq!(
            line,
            PktLine::new(
                b"c4bf7555e2eb4a2b55c7404c742e7e95017ec850 refs/remotes/origin/master\n".into()
            )
        );

        let line = lines.next().unwrap();
        assert_eq!(line, PktLine::flush());

        assert!(lines.next().is_none());
    }
}
