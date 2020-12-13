pub struct Writer<'a> {
    out: &'a mut Vec<u8>,
    len: usize,
}

impl<'a> Writer<'a> {
    pub fn new(out: &'a mut Vec<u8>) -> Self {
        Self { out, len: 0 }
    }

    pub fn mark_len(&self) -> usize {
        self.out.len()
    }

    pub fn write_u16_len_sub(&mut self, mark: usize, sub: usize) {
        let writed = self.out.len() - mark - sub;
        self.out[mark - 2..mark].copy_from_slice(&(writed as u16).to_ne_bytes());
    }

    pub fn write_u16_len(&mut self, mark: usize) {
        let writed = self.out.len() - mark;
        self.out[mark - 2..mark].copy_from_slice(&(writed as u16).to_ne_bytes());
    }

    pub fn write_u16_len_div4(&mut self, mark: usize) {
        let writed = self.out.len() - mark;
        let writed = writed / 4;
        self.out[mark - 2..mark].copy_from_slice(&(writed as u16).to_ne_bytes());
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
}
