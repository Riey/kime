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
    use crate::{
        read, write, Connect, ConnectReply, Open, QueryExtension, Request, XimStr, XimString,
    };
    use pretty_assertions::assert_eq;
    use std::marker::PhantomData;

    #[test]
    fn read_connect_req() {
        let req: Request = read(b"\x01\x00\x00\x00\x6c\x00\x00\x00\x00\x00\x00\x00").unwrap();
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
    fn read_open() {
        let req = read(&[
            30, 0, 2, 0, 5, 101, 110, 95, 85, 83, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ])
        .unwrap();
        assert_eq!(
            req,
            Request::Open(Open {
                name: XimStr(b"en_US"),
            })
        );
    }

    #[test]
    fn read_query() {
        let req = read(&[
            40, 0, 5, 0, 0, 0, 13, 0, 12, 88, 73, 77, 95, 69, 88, 84, 95, 77, 79, 86, 69, 0, 0, 0,
        ])
        .unwrap();
        assert_eq!(
            req,
            Request::QueryExtension(QueryExtension {
                input_method_id: 0,
                extensions: vec![XimStr(b"XIM_EXT_MOVE"),],
            })
        );
    }

    #[test]
    fn write_connect_reply() {
        let reply = ConnectReply {
            server_minor_protocol_version: 0,
            server_major_protocol_version: 1,
            _marker: PhantomData,
        };
        let mut out = Vec::new();
        write(Request::ConnectReply(reply), &mut out);

        assert_eq!(out, b"\x02\x00\x01\x00\x01\x00\x00\x00");
    }
}
