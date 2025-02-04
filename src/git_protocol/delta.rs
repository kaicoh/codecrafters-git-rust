use std::io::Read;

const MASK_LAST_7: u8 = 0b01111111;

fn get_length<R: Read>(r: &mut R) -> usize {
    let mut byte = super::read_one(r);
    let mut len: usize = (byte & MASK_LAST_7) as usize;

    while super::msb_is_1(byte) {
        byte = super::read_one(r);
        let additional_len: usize = (byte & MASK_LAST_7) as usize;
        len += additional_len << 7;
    }

    len
}

#[derive(Debug)]
pub struct Delta {
    #[allow(unused)]
    base_size: usize,
    #[allow(unused)]
    target_size: usize,
    instructions: Vec<Instruction>,
}

impl Delta {
    pub fn new<R: Read>(r: &mut R) -> Self {
        let base_size = get_length(r);
        let target_size = get_length(r);

        let mut instructions: Vec<Instruction> = vec![];

        let mut buf = [0u8; 1];
        while let Ok(n) = r.read(&mut buf) {
            if n == 0 {
                break;
            }
            let [byte] = buf;
            instructions.push(Instruction::new(byte, r));
        }

        Self {
            base_size,
            target_size,
            instructions,
        }
    }

    pub fn restore(self, buf: &[u8]) -> Vec<u8> {
        let mut result: Vec<u8> = vec![];

        for inst in self.instructions {
            match inst {
                Instruction::Copy { offset, size } => {
                    let mut bytes = buf[offset..(offset + size)].to_vec();
                    result.append(&mut bytes);
                }
                Instruction::Insert(mut bytes) => {
                    result.append(&mut bytes);
                }
            }
        }

        result
    }
}

#[derive(Debug)]
pub enum Instruction {
    Copy { offset: usize, size: usize },
    Insert(Vec<u8>),
}

impl Instruction {
    fn new<R: Read>(byte: u8, r: &mut R) -> Self {
        if super::msb_is_1(byte) {
            let offset = get_delta_offset(byte, r);
            let size = get_delta_size(byte, r);
            Self::Copy { offset, size }
        } else {
            let len = (byte & MASK_LAST_7) as usize;
            let mut buf = vec![0u8; len];
            r.read_exact(&mut buf)
                .expect("Cannot get insert instruction from delta");
            Self::Insert(buf)
        }
    }
}

fn get_delta_offset<R: Read>(byte: u8, r: &mut R) -> usize {
    offset1(byte, r) + offset2(byte, r) + offset3(byte, r) + offset4(byte, r)
}

fn get_delta_size<R: Read>(byte: u8, r: &mut R) -> usize {
    size1(byte, r) + size2(byte, r) + size3(byte, r)
}

fn read_size<R: Read>(mask: u8, shift: usize) -> Box<dyn FnMut(u8, &mut R) -> usize> {
    Box::new(move |byte: u8, r: &mut R| {
        if byte & mask == mask {
            let val: usize = super::read_one(r) as usize;
            val << shift
        } else {
            0
        }
    })
}

fn offset1<R: Read>(byte: u8, r: &mut R) -> usize {
    read_size(0x01, 0)(byte, r)
}

fn offset2<R: Read>(byte: u8, r: &mut R) -> usize {
    read_size(0x02, 8)(byte, r)
}

fn offset3<R: Read>(byte: u8, r: &mut R) -> usize {
    read_size(0x04, 16)(byte, r)
}

fn offset4<R: Read>(byte: u8, r: &mut R) -> usize {
    read_size(0x08, 24)(byte, r)
}

fn size1<R: Read>(byte: u8, r: &mut R) -> usize {
    read_size(0x10, 0)(byte, r)
}

fn size2<R: Read>(byte: u8, r: &mut R) -> usize {
    read_size(0x20, 8)(byte, r)
}

fn size3<R: Read>(byte: u8, r: &mut R) -> usize {
    read_size(0x40, 16)(byte, r)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn it_gets_length() {
        let bytes = [0b10010001, 0b00101110];
        let mut cursor = Cursor::new(&bytes[..]);
        assert_eq!(get_length(&mut cursor), 5905);

        let bytes = [0b10101100, 0b00101110];
        let mut cursor = Cursor::new(&bytes[..]);
        assert_eq!(get_length(&mut cursor), 5932);
    }

    #[test]
    fn it_gets_delta_offset() {
        let byte = 0b10000101;
        let bytes = [0b00000001, 0b00000001];
        let mut cursor = Cursor::new(&bytes[..]);
        assert_eq!(get_delta_offset(byte, &mut cursor), 65537);
    }

    #[test]
    fn it_gets_delta_size() {
        let byte = 0b10110000;
        let bytes = [0b11010001, 0b00000001];
        let mut cursor = Cursor::new(&bytes[..]);
        assert_eq!(get_delta_size(byte, &mut cursor), 465);
    }
}
