use x11rb::protocol::xproto::{EventMask, KEY_PRESS_EVENT};
use x11rb::{
    connection::Connection,
    protocol::xproto::{
        ConnectionExt, CreateGCAux, CreateWindowAux, Gcontext, KeyPressEvent, Pixmap, Window,
        WindowClass,
    },
    COPY_DEPTH_FROM_PARENT,
};
use xim::{
    x11rb::{HasConnection, X11rbServer},
    InputStyle, Server, ServerHandler,
};

use crate::engine::{DubeolSik, InputEngine, InputResult};

pub struct KimeData {
    engine: InputEngine<DubeolSik>,
    preedit_window: Window,
    pixmap: Pixmap,
    gc: Gcontext,
}

impl KimeData {
    pub fn new<C: HasConnection>(c: C, input_style: InputStyle) -> Result<Self, xim::ServerError> {
        let conn = c.conn();
        let preedit_window;
        let pixmap;
        let gc;

        if input_style.contains(InputStyle::PREEDITCALLBACKS) {
            preedit_window = x11rb::NONE;
            pixmap = x11rb::NONE;
            gc = x11rb::NONE;
        } else {
            preedit_window = conn.generate_id()?;
            pixmap = conn.generate_id()?;
            gc = conn.generate_id()?;

            let screen = &conn.setup().roots[0];

            conn.create_window(
                COPY_DEPTH_FROM_PARENT,
                preedit_window,
                screen.root,
                0,
                0,
                1,
                1,
                0,
                WindowClass::InputOutput,
                screen.root_visual,
                &CreateWindowAux::default()
                    .background_pixel(screen.black_pixel)
                    .border_pixel(screen.white_pixel)
                    .event_mask(EventMask::Exposure | EventMask::StructureNotify),
            )?
            .check()?;

            conn.create_pixmap(COPY_DEPTH_FROM_PARENT, pixmap, screen.root, 1, 1)?
                .check()?;

            conn.create_gc(
                gc,
                pixmap,
                &CreateGCAux::default()
                    .foreground(screen.white_pixel)
                    .background(screen.black_pixel),
            )?
            .check()?;
        }

        Ok(Self {
            engine: InputEngine::new(DubeolSik::new()),
            preedit_window,
            pixmap,
            gc,
        })
    }

    pub fn clean<C: HasConnection>(self, c: C) -> Result<(), xim::ServerError> {
        if self.preedit_window != x11rb::NONE {
            let conn = c.conn();
            conn.free_pixmap(self.pixmap)?.ignore_error();
            conn.free_gc(self.gc)?.ignore_error();
        }

        Ok(())
    }
}

pub struct KimeHandler {
    buf: String,
}

impl KimeHandler {
    pub fn new() -> Self {
        Self {
            buf: String::with_capacity(10),
        }
    }
}

impl KimeHandler {
    fn preedit<C: HasConnection>(
        &mut self,
        server: &mut X11rbServer<C>,
        ic: &mut xim::InputContext<KimeData>,
        ch: char,
    ) -> Result<(), xim::ServerError> {
        self.buf.push(ch);
        if ic.input_style().contains(InputStyle::PREEDITCALLBACKS) {
            // on-the-spot send preedit callback
            server.preedit_draw(ic, &self.buf)?;
        } else {
            // off-the-spot draw in server
        }
        self.buf.clear();

        Ok(())
    }

    fn commit<C: HasConnection>(
        &mut self,
        server: &mut X11rbServer<C>,
        ic: &mut xim::InputContext<KimeData>,
        ch: char,
    ) -> Result<(), xim::ServerError> {
        self.buf.push(ch);
        server.commit(ic, &self.buf)?;
        self.buf.clear();
        Ok(())
    }
}

impl<C: HasConnection> ServerHandler<X11rbServer<C>> for KimeHandler {
    type InputStyleArray = [InputStyle; 4];
    type InputContextData = KimeData;

    fn new_ic_data(
        &mut self,
        server: &mut X11rbServer<C>,
        input_style: InputStyle,
    ) -> Result<Self::InputContextData, xim::ServerError> {
        KimeData::new(&*server, input_style)
    }

    fn input_styles(&self) -> Self::InputStyleArray {
        [
            InputStyle::PREEDITNOTHING | InputStyle::PREEDITNOTHING,
            InputStyle::PREEDITPOSITION | InputStyle::STATUSAREA,
            InputStyle::PREEDITPOSITION | InputStyle::STATUSNOTHING,
            InputStyle::PREEDITPOSITION | InputStyle::STATUSNONE,
        ]
    }

    fn handle_connect(&mut self, _server: &mut X11rbServer<C>) -> Result<(), xim::ServerError> {
        Ok(())
    }

    fn handle_set_ic_values(
        &mut self,
        _server: &mut X11rbServer<C>,
        _input_context: &mut xim::InputContext<KimeData>,
    ) -> Result<(), xim::ServerError> {
        Ok(())
    }

    fn handle_create_ic(
        &mut self,
        server: &mut X11rbServer<C>,
        input_context: &mut xim::InputContext<KimeData>,
    ) -> Result<(), xim::ServerError> {
        log::info!(
            "IC created style: {:?}, spot_location: {:?}",
            input_context.input_style(),
            input_context.preedit_spot()
        );
        server.set_event_mask(
            input_context,
            EventMask::KeyPress | EventMask::KeyRelease,
            0,
            // EventMask::KeyPress | EventMask::KeyRelease,
        )?;

        Ok(())
    }

    fn handle_reset_ic(
        &mut self,
        _server: &mut X11rbServer<C>,
        input_context: &mut xim::InputContext<Self::InputContextData>,
    ) -> Result<String, xim::ServerError> {
        Ok(input_context.user_data.engine.reset())
    }

    fn handle_forward_event(
        &mut self,
        server: &mut X11rbServer<C>,
        input_context: &mut xim::InputContext<Self::InputContextData>,
        xev: &KeyPressEvent,
    ) -> Result<bool, xim::ServerError> {
        if xev.response_type == KEY_PRESS_EVENT {
            let ret = input_context.user_data.engine.key_press(xev.detail);
            log::trace!("ret: {:?}", ret);

            match ret {
                InputResult::Bypass => Ok(false),
                InputResult::Consume => Ok(true),
                InputResult::CommitBypass(ch) => {
                    self.commit(server, input_context, ch)?;
                    Ok(false)
                }
                InputResult::Commit(ch) => {
                    self.commit(server, input_context, ch)?;
                    Ok(true)
                }
                InputResult::CommitPreedit(commit, preedit) => {
                    self.preedit(server, input_context, preedit)?;
                    self.commit(server, input_context, commit)?;
                    Ok(true)
                }
                InputResult::Preedit(preedit) => {
                    self.preedit(server, input_context, preedit)?;
                    Ok(true)
                }
            }
        } else {
            Ok(false)
        }
    }

    fn handle_destory_ic(
        &mut self,
        server: &mut X11rbServer<C>,
        input_context: xim::InputContext<Self::InputContextData>,
    ) -> Result<(), xim::ServerError> {
        input_context.user_data.clean(&*server)
    }

    fn handle_preedit_start(
        &mut self,
        _server: &mut X11rbServer<C>,
        _input_context: &mut xim::InputContext<Self::InputContextData>,
    ) -> Result<(), xim::ServerError> {
        log::info!("preedit started");

        Ok(())
    }

    fn handle_caret(
        &mut self,
        _server: &mut X11rbServer<C>,
        _input_context: &mut xim::InputContext<Self::InputContextData>,
        _position: i32,
    ) -> Result<(), xim::ServerError> {
        Ok(())
    }
}
