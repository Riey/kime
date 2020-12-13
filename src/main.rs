use ahash::AHashMap;
use static_assertions::_core::marker::PhantomData;
use std::convert::{TryFrom, TryInto};
use std::iter;
use x11rb::connection::Connection;
use x11rb::protocol::{xproto::*, Event, Request};
use x11rb::wrapper::ConnectionExt as _;
use x11rb::{atom_manager, COPY_DEPTH_FROM_PARENT, CURRENT_TIME, NONE};

const TRANSPORT_MAX: u32 = 20;
const XIM_ATTRIBUTES: &[(xim::XimString, xim::AttrType)] = &[
        //(xim::XimString(b"queryInputStyle"), xim::AttrType::Style)
        ];
const XIC_ATTRIBUTES: &[(xim::XimString, xim::AttrType)] = &[];
// &[(xim::XimString(b"inputStyle"), xim::AttrType::Long)];
const SERVER_PROTOCOL: (u32, u32) = (2, 0);

fn atom_name(conn: &impl ConnectionExt, atom: Atom) -> anyhow::Result<String> {
    let name = conn.get_atom_name(atom)?.reply()?;
    Ok(String::from_utf8(name.name)?)
}

struct KimeConnection {
    com_win: Window,
    client_win: Window,
    atoms: KimeAtoms,
    buf: Vec<u8>,
    im_id: u16,
}

impl KimeConnection {
    pub fn new(
        conn: &impl Connection,
        atoms: KimeAtoms,
        im_id: u16,
        msg: ClientMessageEvent,
    ) -> anyhow::Result<Self> {
        let com_win = conn.generate_id()?;
        conn.create_window(
            COPY_DEPTH_FROM_PARENT,
            com_win,
            msg.window,
            0,
            0,
            1,
            1,
            0,
            WindowClass::CopyFromParent,
            0,
            &Default::default(),
        )?;

        let [client_win, ..] = msg.data.as_data32();

        let ev = ClientMessageEvent {
            response_type: CLIENT_MESSAGE_EVENT,
            window: client_win,
            type_: atoms.XIM_XCONNECT,
            format: 32,
            data: [
                com_win,
                SERVER_PROTOCOL.0,
                SERVER_PROTOCOL.1,
                TRANSPORT_MAX,
                0,
            ]
            .into(),
            sequence: 0,
        };

        conn.send_event(false, client_win, 0u32, ev)?.check()?;
        conn.flush()?;

        Ok(Self {
            client_win,
            com_win,
            atoms,
            im_id,
            buf: Vec::with_capacity(512),
        })
    }

    pub fn com_win(&self) -> Window {
        self.com_win
    }

    fn send_reply(
        &mut self,
        conn: &(impl Connection + ConnectionExt),
        reply: xim::Request,
    ) -> anyhow::Result<()> {
        log::trace!("Send reply: {:?}", reply);
        xim::write(reply, &mut self.buf);

        if self.buf.len() > TRANSPORT_MAX as usize {
            let mut data = [0; 5];
            data[0] = self.buf.len() as _;
            data[1] = self.atoms.KIME_COMM;
            log::trace!("Send property");

            conn.change_property8(
                PropMode::Append,
                self.client_win,
                self.atoms.KIME_COMM,
                AtomEnum::STRING,
                &self.buf,
            )?
            .check()?;
            conn.send_event(
                false,
                self.client_win,
                0u32,
                PropertyNotifyEvent {
                    window: self.client_win,
                    atom: self.atoms.KIME_COMM,
                    state: Property::NewValue,
                    time: CURRENT_TIME,
                    sequence: 0,
                    response_type: PROPERTY_NOTIFY_EVENT,
                },
            )?
            .check()?;
        // conn.send_event(
        //     false,
        //     self.client_win,
        //     0u32,
        //     ClientMessageEvent {
        //         window: self.client_win,
        //         type_: self.atoms.XIM_PROTOCOL,
        //         format: 32,
        //         sequence: 0,
        //         response_type: CLIENT_MESSAGE_EVENT,
        //         data: ClientMessageData::from(data),
        //     },
        // )?
        // .check()?;
        } else {
            self.buf.resize(20, 0);

            let data = <[u8; 20]>::try_from(self.buf.as_slice())?.into();
            log::trace!("Send CM, {:?}", data);

            conn.send_event(
                false,
                self.client_win,
                0u32,
                ClientMessageEvent {
                    window: self.client_win,
                    type_: self.atoms.XIM_PROTOCOL,
                    format: 8,
                    sequence: 0,
                    response_type: CLIENT_MESSAGE_EVENT,
                    data,
                },
            )?
            .check()?;
        }

        self.buf.clear();

        Ok(())
    }

    fn proc_request(
        &mut self,
        conn: &(impl Connection + ConnectionExt),
        req: xim::Request,
    ) -> anyhow::Result<()> {
        log::trace!("Get request: {:?}", req);

        match req {
            xim::Request::Connect(connect) => {
                let reply = xim::Request::ConnectReply(xim::ConnectReply {
                    server_major_protocol_version: connect.client_major_protocol_version,
                    server_minor_protocol_version: connect.client_minor_protocol_version,
                    _marker: PhantomData,
                });
                self.send_reply(conn, reply)?;
            }
            xim::Request::Open(xim::Open { name }) => {
                log::trace!("Open {}", name);
                let reply = xim::Request::OpenReply(xim::OpenReply {
                    input_method_id: self.im_id,
                    xim_attributes: XIM_ATTRIBUTES
                        .iter()
                        .copied()
                        .enumerate()
                        .map(|(id, (name, type_))| xim::Attr {
                            name,
                            type_,
                            id: id as _,
                        })
                        .collect(),
                    xic_attributes: XIC_ATTRIBUTES
                        .iter()
                        .copied()
                        .enumerate()
                        .map(|(id, (name, type_))| xim::Attr {
                            name,
                            type_,
                            id: id as _,
                        })
                        .collect(),
                });
                self.send_reply(conn, reply)?;
            }
            xim::Request::QueryExtension(query) => {
                for ex in query.extensions.iter() {
                    log::trace!("Requested extension: {}", ex);
                }
                let reply = xim::Request::QueryExtensionReply(xim::QueryExtensionReply {
                    input_method_id: query.input_method_id,
                    extensions: vec![],
                });
                self.send_reply(conn, reply)?;
            }
            xim::Request::ConnectReply(..)
            | xim::Request::OpenReply(..)
            | xim::Request::QueryExtensionReply(..) => {
                return Err(anyhow::anyhow!("Invalid request"))
            }
        };

        conn.flush()?;

        Ok(())
    }

    pub fn get_msg(
        &mut self,
        conn: &(impl Connection + ConnectionExt),
        msg: ClientMessageEvent,
    ) -> anyhow::Result<()> {
        if msg.format == 32 {
            let [length, atom, ..] = msg.data.as_data32();
            let data = conn
                .get_property(true, msg.window, atom, AtomEnum::Any, 0, length)?
                .reply()?
                .value;
            log::trace!("prop data: {:?}", data);
            self.proc_request(conn, xim::read(&data)?)?;
        } else {
            if msg.type_ == self.atoms.XIM_PROTOCOL {
                let data = msg.data.as_data8();
                self.proc_request(conn, xim::read(&data)?)?;
            } else if msg.type_ == self.atoms.XIM_MOREDATA {
                return Err(anyhow::anyhow!("MOREDATA not yet support"));
            } else {
                log::error!("Unknown client message");
            }
        }

        Ok(())
    }
}

atom_manager! {
    KimeAtoms: KimeAtomCookie {
        XIM_SERVERS,
        XIM_XCONNECT: b"_XIM_XCONNECT",
        XIM_PROTOCOL: b"_XIM_PROTOCOL",
        XIM_MOREDATA: b"_XIM_MOREDATA",
        KIME_COMM: b"KIME_COMM",
        LOCALES,
        TRANSPORT,
        SERVER_NAME: b"@server=kime",
    }
}

struct KimeContext<C: Connection + ConnectionExt + Send + Sync> {
    conn: C,
    im_win: u32,
    atoms: KimeAtoms,
    clients: AHashMap<Window, KimeConnection>,
}

impl<C: Connection + ConnectionExt + Send + Sync> KimeContext<C> {
    pub fn init(conn: C, screen_num: usize) -> anyhow::Result<Self> {
        let screen = &conn.setup().roots[screen_num];
        let window = screen.root;
        let im_win = conn.generate_id()?;

        conn.create_window(
            COPY_DEPTH_FROM_PARENT,
            im_win,
            window,
            0,
            0,
            1,
            1,
            0,
            WindowClass::CopyFromParent,
            screen.root_visual,
            &Default::default(),
        )?;
        log::debug!("im_win: {}", im_win);

        log::debug!("set XIM_SERVERS");

        let atoms = KimeAtoms::new(&conn)?.reply()?;
        let server_reply = conn
            .get_property(false, window, atoms.XIM_SERVERS, AtomEnum::ATOM, 0, 8196)?
            .reply()?;

        if server_reply.type_ != NONE
            && (server_reply.type_ != AtomEnum::ATOM.into() || server_reply.format != 32)
        {
            return Err(anyhow::anyhow!("Invalid reply ty"));
        }

        let mut found = false;

        for prop in server_reply.value {
            if prop == atoms.SERVER_NAME as u8 {
                found = true;

                let owner = conn.get_selection_owner(atoms.SERVER_NAME)?.reply()?.owner;

                if owner != NONE {
                    return Err(anyhow::anyhow!("Already running"));
                } else {
                    conn.set_selection_owner(im_win, atoms.SERVER_NAME, CURRENT_TIME)?;
                }

                break;
            }
        }

        if !found {
            conn.set_selection_owner(im_win, atoms.SERVER_NAME, CURRENT_TIME)?;
            conn.change_property32(
                PropMode::Prepend,
                window,
                atoms.XIM_SERVERS,
                AtomEnum::ATOM,
                &[atoms.SERVER_NAME],
            )?;
        } else {
            conn.change_property32(
                PropMode::Prepend,
                window,
                atoms.XIM_SERVERS,
                AtomEnum::ATOM,
                &[],
            )?;
        }

        conn.flush()?;

        Ok(Self {
            conn,
            im_win,
            atoms,
            clients: AHashMap::new(),
        })
    }

    fn send_selection_notify(
        &mut self,
        req: SelectionRequestEvent,
        data: &str,
    ) -> anyhow::Result<()> {
        let e = SelectionNotifyEvent {
            response_type: SELECTION_NOTIFY_EVENT,
            property: req.property,
            time: req.time,
            target: req.target,
            selection: req.selection,
            requestor: req.requestor,
            sequence: 0,
        };

        self.conn
            .change_property8(
                PropMode::Replace,
                req.requestor,
                req.property,
                req.target,
                data.as_bytes(),
            )?
            .check()?;
        self.conn
            .send_event(false, req.requestor, 0u32, e)?
            .check()?;
        self.conn.flush()?;

        Ok(())
    }

    fn notify_transport(&mut self, req: SelectionRequestEvent) -> anyhow::Result<()> {
        log::info!("send transport");
        self.send_selection_notify(req, "@transport=X/")
    }

    fn notify_locale(&mut self, req: SelectionRequestEvent) -> anyhow::Result<()> {
        log::info!("send locale");
        self.send_selection_notify(req, "@locale=en_US")
    }

    fn client_msg(&mut self, msg: ClientMessageEvent) -> anyhow::Result<()> {
        log::trace!("client msg ty: {}", atom_name(&self.conn, msg.type_)?);

        if msg.type_ == self.atoms.XIM_XCONNECT {
            let connection = KimeConnection::new(&self.conn, self.atoms, 0, msg)?;
            self.clients.insert(connection.com_win(), connection);
        } else {
            match self.clients.get_mut(&msg.window) {
                Some(client) => {
                    client.get_msg(&self.conn, msg)?;
                }
                None => {
                    log::error!("Packet for unknown window {}", msg.window);
                }
            }
        }

        Ok(())
    }

    pub fn event_loop(&mut self) -> anyhow::Result<()> {
        loop {
            let ev = self.conn.wait_for_event()?;

            log::trace!("ev: {:?}", ev);

            match ev {
                Event::SelectionRequest(req) => {
                    if req.property == self.atoms.LOCALES {
                        self.notify_locale(req)?;
                    } else if req.property == self.atoms.TRANSPORT {
                        self.notify_transport(req)?;
                    } else {
                        let name = self.conn.get_atom_name(req.property)?.reply()?.name;
                        log::info!("ignore unknown {}", String::from_utf8(name)?);
                    }
                }
                Event::ClientMessage(msg) => {
                    self.client_msg(msg)?;
                }
                Event::Error(err) => {
                    log::error!("X11 Error occur: {:?}", err);
                }
                Event::PropertyNotify(notify) => {}
                _ => {}
            }
        }
    }
}

fn main() {
    pretty_env_logger::init();
    let (conn, screen_num) = x11rb::connect(None).expect("Connect x11");
    let mut ctx = KimeContext::init(conn, screen_num).expect("Create context");
    ctx.event_loop().unwrap();
}
