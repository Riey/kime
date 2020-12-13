use crate::types::*;

pub struct Writer<'a> {
    out: &'a mut Vec<u8>,
    len: usize,
}

impl<'a> Writer<'a> {
    pub fn new(out: &'a mut Vec<u8>) -> Self {
        Self { out, len: 0 }
    }

    pub fn u8(&mut self, n: u8) {
        self.out.push(n);
        self.len += 1;
    }

    pub fn write(&mut self, bytes: &[u8]) {
        self.out.extend_from_slice(bytes);
        self.len += bytes.len();
    }

    pub fn pad(&mut self) {
        let p = (4 - (self.len % 4)) % 4;
        self.out.extend(std::iter::repeat(0).take(p));
        self.len = 0;
    }
}

pub trait Writable {
    fn write(&self, out: &mut Writer);
    fn size(&self) -> usize;
}
