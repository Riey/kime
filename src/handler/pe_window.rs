use std::num::NonZeroU32;

use x11rb::{
    connection::Connection,
    protocol::xproto::{
        AtomEnum, ConnectionExt, CreateGCAux, CreateWindowAux, EventMask, ExposeEvent, Gcontext,
        PropMode, Window, WindowClass,
    },
    wrapper::ConnectionExt as _,
    COPY_DEPTH_FROM_PARENT,
};
use xim::x11rb::HasConnection;

pub struct PeWindow {
    preedit_window: NonZeroU32,
    gc: Gcontext,
    preedit: String,
    size: (u16, u16),
}

impl PeWindow {
    pub fn new<C: HasConnection>(c: C) -> Result<Self, xim::ServerError> {
        let size = (1, 1);
        let conn = c.conn();
        let preedit_window = conn.generate_id()?;
        let gc = conn.generate_id()?;

        let screen = &conn.setup().roots[0];

        conn.create_window(
            COPY_DEPTH_FROM_PARENT,
            preedit_window,
            screen.root,
            0,
            0,
            size.0,
            size.1,
            0,
            WindowClass::InputOutput,
            screen.root_visual,
            &CreateWindowAux::default()
                .background_pixel(screen.black_pixel)
                .event_mask(EventMask::Exposure | EventMask::StructureNotify),
        )?
        .check()?;

        conn.create_gc(
            gc,
            preedit_window,
            &CreateGCAux::default()
                .foreground(screen.white_pixel)
                .background(screen.black_pixel),
        )?
        .check()?;

        let window_type = conn
            .intern_atom(false, b"_NET_WM_WINDOW_TYPE")?
            .reply()?
            .atom;
        let popup = conn
            .intern_atom(false, b"_NET_WM_WINDOW_TYPE_POPUP_MENU")?
            .reply()?
            .atom;

        conn.change_property32(
            PropMode::Replace,
            preedit_window,
            window_type,
            AtomEnum::ATOM,
            &[popup],
        )?;

        conn.map_window(preedit_window)?.check()?;

        conn.flush()?;
        Ok(Self {
            preedit_window: NonZeroU32::new(preedit_window).unwrap(),
            preedit: String::with_capacity(10),
            gc,
            size,
        })
    }

    pub fn clean<C: HasConnection>(self, c: C) -> Result<(), xim::ServerError> {
        let conn = c.conn();
        conn.free_gc(self.gc)?.ignore_error();
        conn.unmap_window(self.preedit_window.get())?.ignore_error();

        Ok(())
    }

    pub fn window(&self) -> NonZeroU32 {
        self.preedit_window
    }

    pub fn expose<C: HasConnection>(
        &mut self,
        c: C,
        e: ExposeEvent,
    ) -> Result<(), xim::ServerError> {
        self.size = (e.width, e.height);
        c.conn()
            .poly_text8(self.preedit_window.get(), self.gc, 0, 0, b"ABC")?;
        Ok(())
    }

    pub fn set_preedit(&mut self, ch: char) {
        self.preedit.clear();
        self.preedit.push(ch);
    }
}
