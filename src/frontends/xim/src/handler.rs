use std::num::NonZeroU32;

use crate::pe_window::PeWindow;
use ahash::AHashMap;
use x11rb::{
    protocol::xproto::{ConfigureNotifyEvent, KeyPressEvent, KEY_PRESS_EVENT},
    xcb_ffi::XCBConnection,
};
use xim::{
    x11rb::{HasConnection, X11rbServer},
    InputStyle, Server, ServerHandler,
};

use kime_engine_cffi::{
    Config, InputEngine, InputResultType, MODIFIER_ALT, MODIFIER_CONTROL, MODIFIER_SHIFT,
    MODIFIER_SUPER,
};

pub struct KimeData {
    engine: InputEngine,
    pe: Option<NonZeroU32>,
    show_preedit_window: bool,
}

impl KimeData {
    pub fn new(show_preedit_window: bool) -> Self {
        Self {
            engine: InputEngine::new(),
            pe: None,
            show_preedit_window,
        }
    }
}

pub struct KimeHandler {
    preedit_windows: AHashMap<NonZeroU32, PeWindow>,
    config: Config,
    screen_num: usize,
}

impl KimeHandler {
    pub fn new(screen_num: usize, config: Config) -> Self {
        Self {
            preedit_windows: AHashMap::new(),
            config,
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
        if !ic.user_data.show_preedit_window {
            // Don't spawn preedit window
            return Ok(());
        }

        if let Some(pe) = ic.user_data.pe.as_mut() {
            // Draw in server (already have pe_window)
            let pe = self.preedit_windows.get_mut(pe).unwrap();
            pe.set_preedit(ch);
            pe.refresh(server.conn())?;
        } else {
            // Draw in server
            let mut pe = PeWindow::new(
                server.conn(),
                self.config.xim_font(),
                ic.app_win(),
                ic.preedit_spot(),
                self.screen_num,
            )?;

            pe.set_preedit(ch);

            ic.user_data.pe = Some(pe.window());

            self.preedit_windows.insert(pe.window(), pe);
        }

        Ok(())
    }

    fn reset(
        &mut self,
        server: &mut X11rbServer<XCBConnection>,
        ic: &mut xim::InputContext<KimeData>,
    ) -> Result<(), xim::ServerError> {
        match ic.user_data.engine.reset() {
            '\0' => {}
            c => {
                self.clear_preedit(server, ic)?;
                self.commit(server, ic, c)?;
            }
        }

        Ok(())
    }

    fn clear_preedit(
        &mut self,
        server: &mut X11rbServer<XCBConnection>,
        ic: &mut xim::InputContext<KimeData>,
    ) -> Result<(), xim::ServerError> {
        if let Some(pe) = ic.user_data.pe.take() {
            // off-the-spot draw in server
            if let Some(w) = self.preedit_windows.remove(&pe) {
                log::trace!("Destory PeWindow: {}", w.window());
                w.clean(server.conn())?;
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

    fn commit2(
        &mut self,
        server: &mut X11rbServer<XCBConnection>,
        ic: &mut xim::InputContext<KimeData>,
        ch1: char,
        ch2: char,
    ) -> Result<(), xim::ServerError> {
        let mut buf = [0; 8];
        let len1 = ch1.len_utf8();
        let len2 = ch2.len_utf8();
        ch1.encode_utf8(&mut buf);
        ch2.encode_utf8(&mut buf[len1..]);

        let s = unsafe { std::str::from_utf8_unchecked(&buf[..len1 + len2]) };

        server.commit(ic, s)?;
        Ok(())
    }
}

impl ServerHandler<X11rbServer<XCBConnection>> for KimeHandler {
    type InputStyleArray = [InputStyle; 3];
    type InputContextData = KimeData;

    fn new_ic_data(
        &mut self,
        _server: &mut X11rbServer<XCBConnection>,
        input_style: InputStyle,
    ) -> Result<Self::InputContextData, xim::ServerError> {
        let mut show_preedit_window = true;

        // Use callback instead
        if input_style.contains(InputStyle::PREEDIT_CALLBACKS) {
            show_preedit_window = false;
        }

        // Don't show preedit window on Xwayland see #137
        if std::env::var("XDG_SESSION_TYPE")
            .map(|v| v == "wayland")
            .unwrap_or(false)
        {
            show_preedit_window = false;
        }

        Ok(KimeData::new(show_preedit_window))
    }

    fn input_styles(&self) -> Self::InputStyleArray {
        [
            // over-spot
            InputStyle::PREEDIT_NOTHING | InputStyle::STATUS_NOTHING,
            InputStyle::PREEDIT_POSITION | InputStyle::STATUS_NOTHING,
            InputStyle::PREEDIT_POSITION | InputStyle::STATUS_NONE,
            // // on-the-spot when enable this java awt doesn't work I don't know why
        ]
    }

    fn filter_events(&self) -> u32 {
        1
    }

    fn sync_mode(&self) -> bool {
        true
    }

    fn handle_connect(
        &mut self,
        _server: &mut X11rbServer<XCBConnection>,
    ) -> Result<(), xim::ServerError> {
        Ok(())
    }

    fn handle_set_ic_values(
        &mut self,
        server: &mut X11rbServer<XCBConnection>,
        input_context: &mut xim::InputContext<KimeData>,
    ) -> Result<(), xim::ServerError> {
        log::trace!("spot: {:?}", input_context.preedit_spot());

        match input_context.user_data.engine.preedit_char() {
            '\0' => {}
            preedit => {
                self.clear_preedit(server, input_context)?;
                self.preedit(server, input_context, preedit)?;
            }
        }

        Ok(())
    }

    fn handle_create_ic(
        &mut self,
        _server: &mut X11rbServer<XCBConnection>,
        input_context: &mut xim::InputContext<KimeData>,
    ) -> Result<(), xim::ServerError> {
        log::info!(
            "IC created style: {:?}, spot_location: {:?}",
            input_context.input_style(),
            input_context.preedit_spot()
        );

        Ok(())
    }

    fn handle_reset_ic(
        &mut self,
        _server: &mut X11rbServer<XCBConnection>,
        input_context: &mut xim::InputContext<Self::InputContextData>,
    ) -> Result<String, xim::ServerError> {
        log::trace!("reset_ic");

        match input_context.user_data.engine.reset() {
            '\0' => Ok(String::new()),
            c => Ok(c.to_string()),
        }
    }

    fn handle_forward_event(
        &mut self,
        server: &mut X11rbServer<XCBConnection>,
        input_context: &mut xim::InputContext<Self::InputContextData>,
        xev: &KeyPressEvent,
    ) -> Result<bool, xim::ServerError> {
        // skip release
        if xev.response_type != KEY_PRESS_EVENT {
            return Ok(false);
        }

        log::trace!("{:?}", xev);

        let mut state = 0;

        if xev.state & 0x1 != 0 {
            state |= MODIFIER_SHIFT;
        }

        if xev.state & 0x4 != 0 {
            state |= MODIFIER_CONTROL;
        }

        if xev.state & 0x8 != 0 {
            state |= MODIFIER_ALT;
        }

        if xev.state & 0x40 != 0 {
            state |= MODIFIER_SUPER;
        }

        let ret = input_context
            .user_data
            .engine
            .press_key(&self.config, xev.detail as u16, state);

        log::trace!("{:?}", ret);

        match ret.ty {
            InputResultType::Bypass => Ok(false),
            InputResultType::ToggleHangul => {
                input_context.user_data.engine.update_hangul_state();
                Ok(true)
            }
            InputResultType::ClearPreedit => {
                self.clear_preedit(server, input_context)?;
                Ok(true)
            }
            InputResultType::CommitBypass => {
                self.commit(server, input_context, ret.char1)?;
                self.clear_preedit(server, input_context)?;
                Ok(false)
            }
            InputResultType::Commit => {
                self.commit(server, input_context, ret.char1)?;
                self.clear_preedit(server, input_context)?;
                Ok(true)
            }
            InputResultType::CommitCommit => {
                self.commit2(server, input_context, ret.char1, ret.char2)?;
                self.clear_preedit(server, input_context)?;
                Ok(true)
            }
            InputResultType::CommitPreedit => {
                self.commit(server, input_context, ret.char1)?;
                self.preedit(server, input_context, ret.char2)?;
                Ok(true)
            }
            InputResultType::Preedit => {
                self.preedit(server, input_context, ret.char1)?;
                Ok(true)
            }
        }
    }

    fn handle_destory_ic(
        &mut self,
        server: &mut X11rbServer<XCBConnection>,
        input_context: xim::InputContext<Self::InputContextData>,
    ) -> Result<(), xim::ServerError> {
        log::info!("destroy_ic");

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

    fn handle_set_focus(
        &mut self,
        _server: &mut X11rbServer<XCBConnection>,
        input_context: &mut xim::InputContext<Self::InputContextData>,
    ) -> Result<(), xim::ServerError> {
        input_context.user_data.engine.update_hangul_state();
        Ok(())
    }

    fn handle_unset_focus(
        &mut self,
        server: &mut X11rbServer<XCBConnection>,
        input_context: &mut xim::InputContext<Self::InputContextData>,
    ) -> Result<(), xim::ServerError> {
        self.reset(server, input_context)
    }
}
