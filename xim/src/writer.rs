use crate::types::*;

pub trait Writable {
    fn write(&self, out: &mut Vec<u8>);
    fn size(&self) -> usize;
}
