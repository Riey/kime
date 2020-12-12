mod reader;
mod types;

pub use self::reader::ReadError;
pub use self::types::*;
use crate::reader::Readable;
use crate::reader::Reader;

pub fn read(bytes: &[u8]) -> Result<Request, ReadError> {
    let mut reader = Reader::new(bytes);
    Request::read(&mut reader)
}

#[cfg(test)]
mod tests {
    use crate::{read, Connect, PreeditDone, Request};

    #[test]
    fn read_connect_req() {
        let req: Request = read(b"\x01\x00\x00\x00\x6c\x00\x00\x00\x00\x00\x00").unwrap();
        assert_eq!(
            req,
            Request::Connect(Connect {
                client_auth_protocol_names: vec![],
                client_minor_protocol_version: 0,
                client_major_protocol_version: 0,
            })
        );
    }
}
