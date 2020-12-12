mod impls;
mod reader;
mod types;
mod writer;

pub use self::reader::ReadError;
pub use self::types::*;
use crate::reader::Readable;
use crate::reader::Reader;
use crate::writer::Writable;

pub fn read(bytes: &[u8]) -> Result<Request, ReadError> {
    let mut reader = Reader::new(bytes);
    Request::read(&mut reader)
}

pub fn write(request: Request, out: &mut Vec<u8>) {
    request.write(out);
}

#[cfg(test)]
mod tests {
    use crate::{read, write, Connect, ConnectReply, Request};

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

    #[test]
    fn write_connect_reply() {
        let reply = ConnectReply {
            server_minor_protocol_version: 0,
            server_major_protocol_version: 1,
        };
        let mut out = Vec::new();
        write(Request::ConnectReply(reply), &mut out);

        assert_eq!(out, b"\x02\x00\x04\x00\x01\x00\x00\x00");
    }
}
