use std::num::NonZeroU32;

use x11rb::{
    connection::Connection,
    protocol::{
        render::{self, ConnectionExt as _, PictType},
        xproto::{
            AtomEnum, ColormapAlloc, ConfigureNotifyEvent, ConnectionExt as _, CreateWindowAux,
            EventMask, PropMode, Visualid, Visualtype, WindowClass,
        },
    },
    rust_connection::ReplyError,
    wrapper::ConnectionExt as _,
    xcb_ffi::XCBConnection,
};
use xim::x11rb::HasConnection;

pub struct PeWindow {
    preedit_window: NonZeroU32,
    preedit: String,
    surface: cairo::XCBSurface,
    cr: cairo::Context,
    size: (u16, u16),
}

impl PeWindow {
    pub fn new(
        conn: &XCBConnection,
        app_win: Option<NonZeroU32>,
        spot_location: xim::Point,
        screen_num: usize,
    ) -> Result<Self, xim::ServerError> {
        let size = (30, 30);
        let preedit_window = conn.generate_id()?;
        let colormap = conn.generate_id()?;
        let (depth, visual_id) = choose_visual(conn, screen_num)?;

        let screen = &conn.setup().roots[screen_num];
        let pos = find_position(conn, screen.root, app_win, spot_location)?;

        conn.create_colormap(ColormapAlloc::None, colormap, screen.root, visual_id)?
            .check()?;

        conn.create_window(
            depth,
            preedit_window,
            screen.root,
            pos.0,
            pos.1,
            size.0,
            size.1,
            0,
            WindowClass::InputOutput,
            visual_id,
            &CreateWindowAux::default()
                .background_pixel(x11rb::NONE)
                .border_pixel(x11rb::NONE)
                .override_redirect(1u32)
                .event_mask(EventMask::Exposure | EventMask::StructureNotify)
                .colormap(colormap),
        )?
        .check()?;

        conn.free_colormap(colormap)?;

        let window_type = conn
            .intern_atom(false, b"_NET_WM_WINDOW_TYPE\0")?
            .reply()?
            .atom;
        let popup = conn
            .intern_atom(false, b"_NET_WM_WINDOW_TYPE_DOCK\0")?
            .reply()?
            .atom;

        conn.change_property32(
            PropMode::Replace,
            preedit_window,
            window_type,
            AtomEnum::ATOM,
            &[popup],
        )?;

        conn.change_property8(
            PropMode::Replace,
            preedit_window,
            AtomEnum::WM_CLASS,
            AtomEnum::STRING,
            b"kime\0kime\0",
        )?;

        let mut visual = find_xcb_visualtype(conn, visual_id).unwrap();
        let cairo_conn =
            unsafe { cairo::XCBConnection::from_raw_none(conn.get_raw_xcb_connection() as _) };
        let visual = unsafe { cairo::XCBVisualType::from_raw_none(&mut visual as *mut _ as _) };
        let surface = cairo::XCBSurface::create(
            &cairo_conn,
            &cairo::XCBDrawable(preedit_window),
            &visual,
            size.0 as _,
            size.1 as _,
        )
        .unwrap();

        conn.map_window(preedit_window)?.check()?;

        conn.flush()?;

        let cr = cairo::Context::new(&surface);

        cr.select_font_face(
            &crate::CONFIG.xim_preedit_font,
            cairo::FontSlant::Normal,
            cairo::FontWeight::Normal,
        );
        cr.set_font_size(15.0);

        Ok(Self {
            surface,
            cr,
            preedit_window: NonZeroU32::new(preedit_window).unwrap(),
            preedit: String::with_capacity(10),
            size,
        })
    }

    pub fn clean<C: HasConnection>(self, c: C) -> Result<(), xim::ServerError> {
        let conn = c.conn();
        conn.destroy_window(self.preedit_window.get())?
            .ignore_error();

        Ok(())
    }

    pub fn window(&self) -> NonZeroU32 {
        self.preedit_window
    }

    fn redraw(&mut self) {
        log::trace!("Redraw: {}", self.preedit);
        self.cr.set_source_rgb(1.0, 1.0, 1.0);
        self.cr.paint();

        if !self.preedit.is_empty() {
            self.cr.set_source_rgb(0.0, 0.0, 0.0);
            self.cr.move_to(6.0, 17.5);
            self.cr.show_text(&self.preedit);
        }

        self.surface.flush();
    }

    pub fn expose(&mut self) {
        self.redraw();
    }

    pub fn configure_notify(&mut self, e: ConfigureNotifyEvent) {
        self.size = (e.width, e.height);
        self.surface.set_size(e.width as _, e.height as _).unwrap();
        self.redraw();
    }

    pub fn set_preedit(&mut self, ch: char) {
        self.preedit.clear();
        self.preedit.push(ch);
        self.redraw();
    }
}

/// Choose a visual to use. This function tries to find a depth=32 visual and falls back to the
/// screen's default visual.
fn choose_visual(conn: &impl Connection, screen_num: usize) -> Result<(u8, Visualid), ReplyError> {
    let depth = 32;
    let screen = &conn.setup().roots[screen_num];

    // Try to use XRender to find a visual with alpha support
    let has_render = conn
        .extension_information(render::X11_EXTENSION_NAME)?
        .is_some();
    if has_render {
        let formats = conn.render_query_pict_formats()?.reply()?;
        // Find the ARGB32 format that must be supported.
        let format = formats
            .formats
            .iter()
            .filter(|info| (info.type_, info.depth) == (PictType::Direct, depth))
            .filter(|info| {
                let d = info.direct;
                (d.red_mask, d.green_mask, d.blue_mask, d.alpha_mask) == (0xff, 0xff, 0xff, 0xff)
            })
            .find(|info| {
                let d = info.direct;
                (d.red_shift, d.green_shift, d.blue_shift, d.alpha_shift) == (16, 8, 0, 24)
            });
        if let Some(format) = format {
            // Now we need to find the visual that corresponds to this format
            if let Some(visual) = formats.screens[screen_num]
                .depths
                .iter()
                .flat_map(|d| &d.visuals)
                .find(|v| v.format == format.id)
            {
                return Ok((format.depth, visual.visual));
            }
        }
    }
    Ok((screen.root_depth, screen.root_visual))
}

/// A rust version of XCB's `xcb_visualtype_t` struct. This is used in a FFI-way.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct xcb_visualtype_t {
    pub visual_id: u32,
    pub class: u8,
    pub bits_per_rgb_value: u8,
    pub colormap_entries: u16,
    pub red_mask: u32,
    pub green_mask: u32,
    pub blue_mask: u32,
    pub pad0: [u8; 4],
}

impl From<Visualtype> for xcb_visualtype_t {
    fn from(value: Visualtype) -> xcb_visualtype_t {
        xcb_visualtype_t {
            visual_id: value.visual_id,
            class: value.class.into(),
            bits_per_rgb_value: value.bits_per_rgb_value,
            colormap_entries: value.colormap_entries,
            red_mask: value.red_mask,
            green_mask: value.green_mask,
            blue_mask: value.blue_mask,
            pad0: [0; 4],
        }
    }
}

/// Find a `xcb_visualtype_t` based on its ID number
fn find_xcb_visualtype(conn: &impl Connection, visual_id: u32) -> Option<xcb_visualtype_t> {
    for root in &conn.setup().roots {
        for depth in &root.allowed_depths {
            for visual in &depth.visuals {
                if visual.visual_id == visual_id {
                    return Some((*visual).into());
                }
            }
        }
    }
    None
}

fn find_position(
    conn: &impl Connection,
    root: u32,
    app_win: Option<NonZeroU32>,
    spot_location: xim::Point,
) -> Result<(i16, i16), xim::ServerError> {
    match app_win {
        Some(app_win) => {
            let offset = conn
                .translate_coordinates(app_win.get(), root, spot_location.x, spot_location.y)?
                .reply()?;

            Ok((offset.dst_x, offset.dst_y))
        }
        _ => Ok((0, 0)),
    }
}
