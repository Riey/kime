use x11rb::connection::Connection;
use x11rb::protocol::{xproto::*, Event};
use x11rb::wrapper::ConnectionExt as _;
use x11rb::{COPY_DEPTH_FROM_PARENT, CURRENT_TIME, NONE};

fn main_loop() -> anyhow::Result<()> {
    let (conn, screen_num) = x11rb::connect(None)?;
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
    let xim_server = conn.intern_atom(false, b"XIM_SERVERS")?.reply()?.atom;
    let server_name = conn.intern_atom(false, b"@server=kime")?.reply()?.atom;
    let server_reply = conn
        .get_property(false, window, xim_server, AtomEnum::ATOM, 0, 8196)?
        .reply()?;

    if server_reply.type_ != NONE && (server_reply.type_ != 4 || server_reply.format != 32) {
        return Err(anyhow::anyhow!("Invalid reply ty"));
    }

    let mut found = false;

    for prop in server_reply.value {
        if prop == server_name as u8 {
            found = true;

            let owner = conn.get_selection_owner(server_name)?.reply()?.owner;

            if owner != NONE {
                return Err(anyhow::anyhow!("Already running"));
            } else {
                conn.set_selection_owner(im_win, server_name, CURRENT_TIME)?;
            }

            break;
        }
    }

    if !found {
        conn.set_selection_owner(im_win, server_name, CURRENT_TIME)?;
        conn.change_property32(
            PropMode::Prepend,
            window,
            xim_server,
            AtomEnum::ATOM,
            &[server_name],
        )?;
    } else {
        conn.change_property32(PropMode::Prepend, window, xim_server, AtomEnum::ATOM, &[])?;
    }

    conn.flush()?;

    loop {
        let ev = conn.wait_for_event()?;

        log::trace!("ev: {:?}", ev);
    }
}

fn main() {
    pretty_env_logger::init();
    main_loop().unwrap();
}
