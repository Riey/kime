#![allow(unused_variables)]

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReadError {
    #[error("End of Stream")]
    EndOfStream,
    #[error("Invalid Data")]
    InvalidData,
}

pub type Result<T> = std::result::Result<T, ReadError>;

pub struct Reader<'a> {
    pub b: &'a [u8],
    start: usize,
}

impl<'a> Reader<'a> {
    pub fn new(b: &'a [u8]) -> Self {
        Self {
            b,
            start: b.as_ptr() as usize,
        }
    }

    pub fn eos(&self) -> ReadError {
        ReadError::EndOfStream
    }

    pub fn u8(&mut self) -> Result<u8> {
        match self.b {
            [b, other @ ..] => {
                self.b = other;
                Ok(*b)
            }
            [] => Err(self.eos()),
        }
    }

    pub fn u16(&mut self) -> Result<u16> {
        Readable::read(self)
    }

    fn ptr_offset(&self) -> usize {
        self.b.as_ptr() as usize - self.start
    }

    pub fn pad(&mut self) {
        let p = (4 - (self.ptr_offset() % 4)) % 4;
        self.b = &self.b[p..];
    }

    pub fn string(&mut self, len: usize) -> Result<&'a [u8]> {
        if self.b.len() < len {
            Err(self.eos())
        } else {
            let (bytes, left) = self.b.split_at(len);
            self.b = left;
            Ok(bytes)
        }
    }
}

pub trait Readable<'a>: Sized {
    fn read(reader: &mut Reader<'a>) -> Result<Self>;
}
