use x11rb::connection::Connection;
use x11rb::protocol::{xproto::*, Event};
use x11rb::wrapper::ConnectionExt as _;
use x11rb::{atom_manager, COPY_DEPTH_FROM_PARENT, CURRENT_TIME, NONE};

atom_manager! {
    KimeAtoms: KimeAtomCookie {
        XIM_SERVERS,
        LOCALES,
        TRANSPORT,
        SERVER_NAME: b"@server=kime",
    }
}

struct KimeContext<C: Connection + ConnectionExt + Send + Sync> {
    conn: C,
    im_win: u32,
    atoms: KimeAtoms,
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
            1,
            WindowClass::InputOutput,
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
        })
    }

    pub fn event_loop(&mut self) -> anyhow::Result<()> {
        loop {
            let ev = self.conn.wait_for_event()?;

            log::trace!("ev: {:?}", ev);
        }
    }
}

fn main() {
    pretty_env_logger::init();
    let (conn, screen_num) = x11rb::connect(None).expect("Connect x11");
    let mut ctx = KimeContext::init(conn, screen_num).expect("Create context");
    ctx.event_loop().unwrap();
}
