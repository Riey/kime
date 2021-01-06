mod pe_window;

use std::num::NonZeroU32;

use ahash::AHashMap;
use pe_window::PeWindow;
use x11rb::{
    protocol::xproto::{ConfigureNotifyEvent, EventMask, KeyPressEvent, KEY_PRESS_EVENT},
    xcb_ffi::XCBConnection,
};
use xim::{
    x11rb::{HasConnection, X11rbServer},
    InputStyle, Server, ServerHandler,
};

use crate::engine::{InputEngine, InputResult, Layout};

pub struct KimeData {
    engine: InputEngine,
    pe: Option<NonZeroU32>,
}

impl KimeData {
    pub fn new() -> Self {
        Self {
            engine: InputEngine::new(Layout::dubeolsik()),
            pe: None,
        }
    }
}

pub struct KimeHandler {
    preedit_windows: AHashMap<NonZeroU32, PeWindow>,
    screen_num: usize,
}

impl KimeHandler {
    pub fn new(screen_num: usize) -> Self {
        Self {
            preedit_windows: AHashMap::new(),
            screen_num,
        }
    }
}

impl KimeHandler {
    pub fn expose(&mut self, window: u32) {
        if let Some(win) = NonZeroU32::new(window) {
            if let Some(pe) = self.preedit_windows.get_mut(&win) {
                pe.expose();
            }
        }
    }

    pub fn configure_notify(&mut self, e: ConfigureNotifyEvent) {
        if let Some(win) = NonZeroU32::new(e.window) {
            if let Some(pe) = self.preedit_windows.get_mut(&win) {
                pe.configure_notify(e);
            }
        }
    }

    fn preedit(
        &mut self,
        server: &mut X11rbServer<XCBConnection>,
        ic: &mut xim::InputContext<KimeData>,
        ch: char,
    ) -> Result<(), xim::ServerError> {
        if ic.input_style().contains(InputStyle::PREEDIT_CALLBACKS) {
            log::trace!("Preedit callback {}", ch);
            // on-the-spot send preedit callback
            let mut buf = [0; 4];
            let s = ch.encode_utf8(&mut buf);
            server.preedit_draw(ic, s)?;
        } else if let Some(pe) = ic.user_data.pe.as_mut() {
            // off-the-spot draw in server (already have pe_window)
            self.preedit_windows.get_mut(pe).unwrap().set_preedit(ch);
        } else {
            // off-the-spot draw in server
            let mut pe = PeWindow::new(server.conn(), ic.app_win(), self.screen_num)?;
            pe.set_preedit(ch);

            ic.user_data.pe = Some(pe.window());

            self.preedit_windows.insert(pe.window(), pe);
        }

        Ok(())
    }

    fn clear_preedit(
        &mut self,
        c: &XCBConnection,
        ic: &mut xim::InputContext<KimeData>,
    ) -> Result<(), xim::ServerError> {
        if let Some(pe) = ic.user_data.pe.take() {
            // off-the-spot draw in server
            if let Some(w) = self.preedit_windows.remove(&pe) {
                log::trace!("Destory PeWindow: {}", w.window());
                w.clean(c)?;
            }
        }

        Ok(())
    }

    fn commit(
        &mut self,
        server: &mut X11rbServer<XCBConnection>,
        ic: &mut xim::InputContext<KimeData>,
        ch: char,
    ) -> Result<(), xim::ServerError> {
        let mut buf = [0; 4];
        let s = ch.encode_utf8(&mut buf);
        server.commit(ic, s)?;
        Ok(())
    }
}

impl ServerHandler<X11rbServer<XCBConnection>> for KimeHandler {
    type InputStyleArray = [InputStyle; 7];
    type InputContextData = KimeData;

    fn new_ic_data(
        &mut self,
        _server: &mut X11rbServer<XCBConnection>,
        _input_style: InputStyle,
    ) -> Result<Self::InputContextData, xim::ServerError> {
        Ok(KimeData::new())
    }

    fn input_styles(&self) -> Self::InputStyleArray {
        [
            // root
            InputStyle::PREEDIT_NOTHING | InputStyle::PREEDIT_NOTHING,
            // off-the-spot
            InputStyle::PREEDIT_POSITION | InputStyle::STATUS_AREA,
            InputStyle::PREEDIT_POSITION | InputStyle::STATUS_NOTHING,
            InputStyle::PREEDIT_POSITION | InputStyle::STATUS_NONE,
            // on-the-spot
            InputStyle::PREEDIT_CALLBACKS | InputStyle::STATUS_AREA,
            InputStyle::PREEDIT_CALLBACKS | InputStyle::STATUS_NOTHING,
            InputStyle::PREEDIT_CALLBACKS | InputStyle::STATUS_NONE,
        ]
    }

    fn handle_connect(
        &mut self,
        _server: &mut X11rbServer<XCBConnection>,
    ) -> Result<(), xim::ServerError> {
        Ok(())
    }

    fn handle_set_ic_values(
        &mut self,
        _server: &mut X11rbServer<XCBConnection>,
        _input_context: &mut xim::InputContext<KimeData>,
    ) -> Result<(), xim::ServerError> {
        Ok(())
    }

    fn handle_create_ic(
        &mut self,
        server: &mut X11rbServer<XCBConnection>,
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
        _server: &mut X11rbServer<XCBConnection>,
        input_context: &mut xim::InputContext<Self::InputContextData>,
    ) -> Result<String, xim::ServerError> {
        Ok(input_context.user_data.engine.reset())
    }

    fn handle_forward_event(
        &mut self,
        server: &mut X11rbServer<XCBConnection>,
        input_context: &mut xim::InputContext<Self::InputContextData>,
        xev: &KeyPressEvent,
    ) -> Result<bool, xim::ServerError> {
        if xev.response_type == KEY_PRESS_EVENT {
            let shift = (xev.state & 0x1) != 0;
            let ctrl = (xev.state & 0x4) != 0;

            let ret = input_context
                .user_data
                .engine
                .key_press(xev.detail, shift, ctrl);
            log::trace!("ret: {:?}", ret);

            match ret {
                InputResult::Bypass => Ok(false),
                InputResult::Consume => Ok(true),
                InputResult::ClearPreedit => {
                    self.clear_preedit(server.conn(), input_context)?;
                    Ok(true)
                }
                InputResult::CommitBypass(ch) => {
                    self.commit(server, input_context, ch)?;
                    self.clear_preedit(server.conn(), input_context)?;
                    Ok(false)
                }
                InputResult::Commit(ch) => {
                    self.commit(server, input_context, ch)?;
                    self.clear_preedit(server.conn(), input_context)?;
                    Ok(true)
                }
                InputResult::CommitPreedit(commit, preedit) => {
                    self.commit(server, input_context, commit)?;
                    self.preedit(server, input_context, preedit)?;
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
        server: &mut X11rbServer<XCBConnection>,
        input_context: xim::InputContext<Self::InputContextData>,
    ) -> Result<(), xim::ServerError> {
        if let Some(pe) = input_context.user_data.pe {
            self.preedit_windows.remove(&pe).unwrap().clean(&*server)?;
        }

        Ok(())
    }

    fn handle_preedit_start(
        &mut self,
        _server: &mut X11rbServer<XCBConnection>,
        _input_context: &mut xim::InputContext<Self::InputContextData>,
    ) -> Result<(), xim::ServerError> {
        log::info!("preedit started");

        Ok(())
    }

    fn handle_caret(
        &mut self,
        _server: &mut X11rbServer<XCBConnection>,
        _input_context: &mut xim::InputContext<Self::InputContextData>,
        _position: i32,
    ) -> Result<(), xim::ServerError> {
        Ok(())
    }
}
