mod impls;
mod reader;
mod types;
mod writer;

pub use self::reader::ReadError;
pub use self::types::*;
use crate::reader::Readable;
use crate::reader::Reader;
use crate::writer::{Writable, Writer};

pub fn read(bytes: &[u8]) -> Result<Request, ReadError> {
    let mut reader = Reader::new(bytes);
    Request::read(&mut reader)
}

pub fn write(request: Request, out: &mut Vec<u8>) {
    let mut writer = Writer::new(out);
    request.write(&mut writer);
}

#[cfg(test)]
mod tests {
    use crate::writer::Writer;
    use crate::*;
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

    const OPEN_REPLY: &[u8] = b"\x01\x00\x18\x00\x00\x00\x0a\x00\x0f\x00\x71\x75\x65\x72\x79\x49\x6e\x70\x75\x74\x53\x74\x79\x6c\x65\x00\x00\x00\x44\x01\x00\x00\x01\x00\x03\x00\x0a\x00\x69\x6e\x70\x75\x74\x53\x74\x79\x6c\x65\x02\x00\x05\x00\x0c\x00\x63\x6c\x69\x65\x6e\x74\x57\x69\x6e\x64\x6f\x77\x00\x00\x03\x00\x05\x00\x0b\x00\x66\x6f\x63\x75\x73\x57\x69\x6e\x64\x6f\x77\x00\x00\x00\x04\x00\x03\x00\x0c\x00\x66\x69\x6c\x74\x65\x72\x45\x76\x65\x6e\x74\x73\x00\x00\x05\x00\xff\x7f\x11\x00\x70\x72\x65\x65\x64\x69\x74\x41\x74\x74\x72\x69\x62\x75\x74\x65\x73\x00\x06\x00\xff\x7f\x10\x00\x73\x74\x61\x74\x75\x73\x41\x74\x74\x72\x69\x62\x75\x74\x65\x73\x00\x00\x07\x00\x0d\x00\x07\x00\x66\x6f\x6e\x74\x53\x65\x74\x00\x00\x00\x08\x00\x0b\x00\x04\x00\x61\x72\x65\x61\x00\x00\x09\x00\x0b\x00\x0a\x00\x61\x72\x65\x61\x4e\x65\x65\x64\x65\x64\x0a\x00\x03\x00\x08\x00\x63\x6f\x6c\x6f\x72\x4d\x61\x70\x00\x00\x0b\x00\x03\x00\x0b\x00\x73\x74\x64\x43\x6f\x6c\x6f\x72\x4d\x61\x70\x00\x00\x00\x0c\x00\x03\x00\x0a\x00\x66\x6f\x72\x65\x67\x72\x6f\x75\x6e\x64\x0d\x00\x03\x00\x0a\x00\x62\x61\x63\x6b\x67\x72\x6f\x75\x6e\x64\x0e\x00\x03\x00\x10\x00\x62\x61\x63\x6b\x67\x72\x6f\x75\x6e\x64\x50\x69\x78\x6d\x61\x70\x00\x00\x0f\x00\x0c\x00\x0c\x00\x73\x70\x6f\x74\x4c\x6f\x63\x61\x74\x69\x6f\x6e\x00\x00\x10\x00\x03\x00\x09\x00\x6c\x69\x6e\x65\x53\x70\x61\x63\x65\x00\x11\x00\x00\x00\x15\x00\x73\x65\x70\x61\x72\x61\x74\x6f\x72\x6f\x66\x4e\x65\x73\x74\x65\x64\x4c\x69\x73\x74\x00";
    fn open_reply_value() -> OpenReply<'static> {
        OpenReply {
            input_method_id: 1,
            xim_attributes: vec![Attr {
                id: 0,
                type_: AttrType::Style,
                name: XimString(b"queryInputStyle"),
            }],
            xic_attributes: vec![
                Attr {
                    id: 1,
                    type_: AttrType::Long,
                    name: XimString(b"inputStyle"),
                },
                Attr {
                    id: 2,
                    type_: AttrType::Window,
                    name: XimString(b"clientWindow"),
                },
                Attr {
                    id: 3,
                    type_: AttrType::Window,
                    name: XimString(b"focusWindow"),
                },
                Attr {
                    id: 4,
                    type_: AttrType::Long,
                    name: XimString(b"filterEvents"),
                },
                Attr {
                    id: 5,
                    type_: AttrType::NestedList,
                    name: XimString(b"preeditAttributes"),
                },
                Attr {
                    id: 6,
                    type_: AttrType::NestedList,
                    name: XimString(b"statusAttributes"),
                },
                Attr {
                    id: 7,
                    type_: AttrType::XFontSet,
                    name: XimString(b"fontSet"),
                },
                Attr {
                    id: 8,
                    type_: AttrType::XRectangle,
                    name: XimString(b"area"),
                },
                Attr {
                    id: 9,
                    type_: AttrType::XRectangle,
                    name: XimString(b"areaNeeded"),
                },
                Attr {
                    id: 10,
                    type_: AttrType::Long,
                    name: XimString(b"colorMap"),
                },
                Attr {
                    id: 11,
                    type_: AttrType::Long,
                    name: XimString(b"stdColorMap"),
                },
                Attr {
                    id: 12,
                    type_: AttrType::Long,
                    name: XimString(b"foreground"),
                },
                Attr {
                    id: 13,
                    type_: AttrType::Long,
                    name: XimString(b"background"),
                },
                Attr {
                    id: 14,
                    type_: AttrType::Long,
                    name: XimString(b"backgroundPixmap"),
                },
                Attr {
                    id: 15,
                    type_: AttrType::XPoint,
                    name: XimString(b"spotLocation"),
                },
                Attr {
                    id: 16,
                    type_: AttrType::Long,
                    name: XimString(b"lineSpace"),
                },
                Attr {
                    id: 17,
                    type_: AttrType::Separator,
                    name: XimString(b"separatorofNestedList"),
                },
            ],
        }
    }

    #[test]
    fn read_open_reply() {
        let mut reader = Reader::new(OPEN_REPLY);
        assert_eq!(OpenReply::read(&mut reader).unwrap(), open_reply_value());
    }

    #[test]
    fn write_open_reply() {
        let mut out = Vec::new();
        let value = open_reply_value();
        value.write(&mut Writer::new(&mut out));
        let new_value = OpenReply::read(&mut Reader::new(&out)).unwrap();
        assert_eq!(value, new_value);
        assert_eq!(OPEN_REPLY.len(), out.len());
        assert_eq!(OPEN_REPLY, out);
    }
}
