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
    Config, InputEngine, InputResult_CONSUMED, InputResult_HAS_COMMIT, InputResult_HAS_PREEDIT,
    InputResult_LANGUAGE_CHANGED, ModifierState_ALT, ModifierState_CONTROL, ModifierState_SHIFT,
    ModifierState_SUPER,
};

pub struct KimeData {
    engine: InputEngine,
    pe: Option<NonZeroU32>,
    show_preedit_window: bool,
}

impl KimeData {
    pub fn new(config: &Config, show_preedit_window: bool) -> Self {
        Self {
            engine: InputEngine::new(config),
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

    fn preedit_draw(
        &mut self,
        server: &mut X11rbServer<XCBConnection>,
        ic: &mut xim::UserInputContext<KimeData>,
    ) -> Result<(), xim::ServerError> {
        server.preedit_draw(&mut ic.ic, ic.user_data.engine.preedit_str())?;
        Ok(())
    }

    fn preedit(
        &mut self,
        server: &mut X11rbServer<XCBConnection>,
        user_ic: &mut xim::UserInputContext<KimeData>,
    ) -> Result<(), xim::ServerError> {
        if user_ic
            .ic
            .input_style()
            .contains(InputStyle::PREEDIT_CALLBACKS)
        {
            self.preedit_draw(server, user_ic)?;
            return Ok(());
        }

        if !user_ic.user_data.show_preedit_window {
            return Ok(());
        }

        if user_ic.user_data.engine.preedit_str().is_empty() {
            return Ok(());
        }

        if let Some(pe) = user_ic.user_data.pe.as_mut() {
            // Draw in server (already have pe_window)
            let pe = self.preedit_windows.get_mut(pe).unwrap();
            pe.set_preedit(user_ic.user_data.engine.preedit_str());
            pe.refresh(server.conn())?;
        } else {
            // Draw in server
            let mut pe = PeWindow::new(
                server.conn(),
                self.config.xim_font(),
                user_ic.ic.app_win(),
                user_ic.ic.preedit_spot(),
                self.screen_num,
            )?;

            pe.set_preedit(user_ic.user_data.engine.preedit_str());
            user_ic.user_data.pe = Some(pe.window());

            self.preedit_windows.insert(pe.window(), pe);
        }

        Ok(())
    }

    fn reset(
        &mut self,
        server: &mut X11rbServer<XCBConnection>,
        user_ic: &mut xim::UserInputContext<KimeData>,
    ) -> Result<(), xim::ServerError> {
        user_ic.user_data.engine.clear_preedit();

        self.clear_preedit(server, user_ic)?;
        self.commit(server, user_ic)?;

        user_ic.user_data.engine.reset();

        Ok(())
    }

    fn clear_preedit(
        &mut self,
        server: &mut X11rbServer<XCBConnection>,
        user_ic: &mut xim::UserInputContext<KimeData>,
    ) -> Result<(), xim::ServerError> {
        if user_ic
            .ic
            .input_style()
            .contains(InputStyle::PREEDIT_CALLBACKS)
        {
            server.preedit_draw(&mut user_ic.ic, "")?;
            return Ok(());
        }

        if let Some(pe) = user_ic.user_data.pe.take() {
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
        user_ic: &mut xim::UserInputContext<KimeData>,
    ) -> Result<(), xim::ServerError> {
        let s = user_ic.user_data.engine.commit_str();
        if !s.is_empty() {
            server.commit(&user_ic.ic, s)?;
        }
        Ok(())
    }
}

// PRESS | RELEASE
const EVENT_MASK: u32 = 3;

impl ServerHandler<X11rbServer<XCBConnection>> for KimeHandler {
    type InputStyleArray = [InputStyle; 6];
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

        Ok(KimeData::new(&self.config, show_preedit_window))
    }

    fn input_styles(&self) -> Self::InputStyleArray {
        [
            InputStyle::PREEDIT_NOTHING | InputStyle::STATUS_NOTHING,
            InputStyle::PREEDIT_POSITION | InputStyle::STATUS_NONE,
            InputStyle::PREEDIT_POSITION | InputStyle::STATUS_NOTHING,
            InputStyle::PREEDIT_POSITION | InputStyle::STATUS_CALLBACKS,
            InputStyle::PREEDIT_CALLBACKS | InputStyle::STATUS_NOTHING,
            InputStyle::PREEDIT_CALLBACKS | InputStyle::STATUS_CALLBACKS,
        ]
    }

    fn filter_events(&self) -> u32 {
        EVENT_MASK
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
        user_ic: &mut xim::UserInputContext<KimeData>,
    ) -> Result<(), xim::ServerError> {
        log::debug!("spot: {:?}", user_ic.ic.preedit_spot());

        self.clear_preedit(server, user_ic)?;
        self.preedit(server, user_ic)?;

        Ok(())
    }

    fn handle_create_ic(
        &mut self,
        server: &mut X11rbServer<XCBConnection>,
        user_ic: &mut xim::UserInputContext<KimeData>,
    ) -> Result<(), xim::ServerError> {
        log::info!(
            "IC created style: {:?}, spot_location: {:?}",
            user_ic.ic.input_style(),
            user_ic.ic.preedit_spot()
        );

        server.set_event_mask(&user_ic.ic, EVENT_MASK, 0)?;

        Ok(())
    }

    fn handle_reset_ic(
        &mut self,
        server: &mut X11rbServer<XCBConnection>,
        user_ic: &mut xim::UserInputContext<Self::InputContextData>,
    ) -> Result<String, xim::ServerError> {
        log::trace!("reset_ic");
        self.reset(server, user_ic).map(|_| String::new())
    }

    fn handle_forward_event(
        &mut self,
        server: &mut X11rbServer<XCBConnection>,
        user_ic: &mut xim::UserInputContext<Self::InputContextData>,
        xev: &KeyPressEvent,
    ) -> Result<bool, xim::ServerError> {
        // skip release
        if xev.response_type != KEY_PRESS_EVENT {
            return Ok(false);
        }

        log::trace!("{:?}", xev);

        let mut state = 0;

        if xev.state & 0x1 != 0 {
            state |= ModifierState_SHIFT;
        }

        if xev.state & 0x4 != 0 {
            state |= ModifierState_CONTROL;
        }

        if xev.state & 0x8 != 0 {
            state |= ModifierState_ALT;
        }

        if xev.state & 0x40 != 0 {
            state |= ModifierState_SUPER;
        }

        let ret = user_ic
            .user_data
            .engine
            .press_key(&self.config, xev.detail as u16, state);

        log::trace!("{:?}", ret);

        if ret & InputResult_LANGUAGE_CHANGED != 0 {
            user_ic.user_data.engine.update_layout_state();
        }

        if ret & InputResult_HAS_PREEDIT == 0 {
            self.clear_preedit(server, user_ic)?;
        }

        if ret & InputResult_HAS_COMMIT != 0 {
            self.commit(server, user_ic)?;
            user_ic.user_data.engine.clear_commit();
        }

        if ret & InputResult_HAS_PREEDIT != 0 {
            self.preedit(server, user_ic)?;
        }

        Ok(ret & InputResult_CONSUMED != 0)
    }

    fn handle_destory_ic(
        &mut self,
        server: &mut X11rbServer<XCBConnection>,
        user_ic: xim::UserInputContext<Self::InputContextData>,
    ) -> Result<(), xim::ServerError> {
        log::info!("destroy_ic");

        if let Some(pe) = user_ic.user_data.pe {
            self.preedit_windows.remove(&pe).unwrap().clean(&*server)?;
        }

        Ok(())
    }

    fn handle_set_focus(
        &mut self,
        _server: &mut X11rbServer<XCBConnection>,
        user_ic: &mut xim::UserInputContext<Self::InputContextData>,
    ) -> Result<(), xim::ServerError> {
        user_ic.user_data.engine.update_layout_state();
        Ok(())
    }

    fn handle_unset_focus(
        &mut self,
        server: &mut X11rbServer<XCBConnection>,
        user_ic: &mut xim::UserInputContext<Self::InputContextData>,
    ) -> Result<(), xim::ServerError> {
        self.reset(server, user_ic)
    }
}
