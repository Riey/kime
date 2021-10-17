use std::{num::NonZeroU32, sync::Arc};

use image::ImageBuffer;
use rusttype::Font;
use x11rb::{
    connection::Connection,
    protocol::xproto::{
        AtomEnum, ConfigureNotifyEvent, ConnectionExt as _, CreateGCAux, CreateWindowAux,
        EventMask, ExposeEvent, ImageFormat, PropMode, WindowClass, EXPOSE_EVENT,
    },
    wrapper::ConnectionExt as _,
};

pub struct PeWindow {
    preedit_window: NonZeroU32,
    preedit: String,
    gc: u32,
    text_pos: (u32, u32),
    text_scale: rusttype::Scale,
    font: Arc<Font<'static>>,
    image_buffer: ImageBuffer<image::Bgra<u8>, Vec<u8>>,
}

impl PeWindow {
    pub fn new(
        conn: &impl Connection,
        (font, font_size): (Arc<Font<'static>>, f64),
        app_win: Option<NonZeroU32>,
        spot_location: xim::Point,
        screen_num: usize,
    ) -> Result<Self, xim::ServerError> {
        let size = (font_size * 1.7) as u16;
        let size = (size, size);
        let gc = conn.generate_id()?;
        let preedit_window = conn.generate_id()?;
        // let colormap = conn.generate_id()?;
        // let (depth, visual_id) = choose_visual(conn, screen_num)?;

        let screen = &conn.setup().roots[screen_num];
        let pos = find_position(conn, screen.root, app_win, spot_location)?;

        conn.create_window(
            screen.root_depth,
            preedit_window,
            screen.root,
            pos.0,
            pos.1,
            size.0,
            size.1,
            0,
            WindowClass::INPUT_OUTPUT,
            0,
            &CreateWindowAux::default()
                .background_pixel(x11rb::NONE)
                .border_pixel(x11rb::NONE)
                .override_redirect(1u32)
                .event_mask(EventMask::EXPOSURE | EventMask::STRUCTURE_NOTIFY),
        )?
        .check()?;

        conn.create_gc(
            gc,
            preedit_window,
            &CreateGCAux::new()
                .background(screen.white_pixel)
                .foreground(screen.black_pixel),
        )?;

        let window_type = conn
            .intern_atom(false, b"_NET_WM_WINDOW_TYPE\0")?
            .reply()?
            .atom;
        let popup = conn
            .intern_atom(false, b"_NET_WM_WINDOW_TYPE_DOCK\0")?
            .reply()?
            .atom;

        conn.change_property32(
            PropMode::REPLACE,
            preedit_window,
            window_type,
            AtomEnum::ATOM,
            &[popup],
        )?;

        conn.change_property8(
            PropMode::REPLACE,
            preedit_window,
            AtomEnum::WM_CLASS,
            AtomEnum::STRING,
            b"kime\0kime\0",
        )?;

        conn.map_window(preedit_window)?.check()?;

        conn.flush()?;

        Ok(Self {
            preedit_window: NonZeroU32::new(preedit_window).unwrap(),
            preedit: String::with_capacity(10),
            gc,
            font,
            text_pos: ((font_size * 0.36) as _, (font_size * 0.36) as _),
            text_scale: rusttype::Scale::uniform(font_size as f32),
            image_buffer: ImageBuffer::new(size.0 as _, size.1 as _),
        })
    }

    pub fn clean(self, conn: &impl Connection) -> Result<(), xim::ServerError> {
        conn.destroy_window(self.preedit_window.get())?
            .ignore_error();
        conn.flush()?;

        Ok(())
    }

    pub fn window(&self) -> NonZeroU32 {
        self.preedit_window
    }

    fn redraw(&mut self, conn: &impl Connection) -> Result<(), xim::ServerError> {
        const BACKGROUND: image::Bgra<u8> = image::Bgra([255, 255, 255, 255]);
        const FOREGROUND: image::Bgra<u8> = image::Bgra([0, 0, 0, 255]);

        log::trace!("Redraw: {}", self.preedit);

        let rect = imageproc::rect::Rect::at(0, 0)
            .of_size(self.image_buffer.width(), self.image_buffer.height());
        imageproc::drawing::draw_filled_rect_mut(&mut self.image_buffer, rect, BACKGROUND);
        imageproc::drawing::draw_text_mut(
            &mut self.image_buffer,
            FOREGROUND,
            self.text_pos.0,
            self.text_pos.1,
            self.text_scale,
            &self.font,
            &self.preedit,
        );

        conn.put_image(
            ImageFormat::Z_PIXMAP,
            self.preedit_window.get(),
            self.gc,
            self.image_buffer.width() as _,
            self.image_buffer.height() as _,
            0,
            0,
            0,
            24,
            self.image_buffer.as_raw(),
        )?;

        Ok(())
    }

    pub fn expose(&mut self, conn: &impl Connection) -> Result<(), xim::ServerError> {
        self.redraw(conn)?;
        Ok(())
    }

    pub fn configure_notify(
        &mut self,
        e: ConfigureNotifyEvent,
        conn: &impl Connection,
    ) -> Result<(), xim::ServerError> {
        self.image_buffer = ImageBuffer::new(e.width as _, e.height as _);
        self.redraw(conn)?;
        Ok(())
    }

    pub fn refresh(&self, conn: &impl Connection) -> Result<(), xim::ServerError> {
        conn.send_event(
            false,
            self.preedit_window.get(),
            0u32,
            ExposeEvent {
                response_type: EXPOSE_EVENT,
                window: self.window().get(),
                width: 0,
                height: 0,
                x: 0,
                y: 0,
                sequence: 0,
                count: 0,
            },
        )?;
        conn.flush()?;

        Ok(())
    }

    pub fn set_preedit(&mut self, s: &str) {
        self.preedit.clear();
        self.preedit.push_str(s);
    }
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
