use x11rb::protocol::xproto::KeyPressEvent;
use x11rb::protocol::xproto::{EventMask, KEY_PRESS_EVENT};
use xim::{
    x11rb::{HasConnection, X11rbServer},
    InputStyle, Server, ServerHandler,
};

use crate::engine::{DubeolSik, InputEngine, InputResult};

pub struct KimeData {
    engine: InputEngine<DubeolSik>,
}

pub struct KimeHandler {}

impl KimeHandler {
    pub fn new() -> Self {
        Self {}
    }
}

impl<C: HasConnection> ServerHandler<X11rbServer<C>> for KimeHandler {
    type InputStyleArray = [InputStyle; 1];
    type InputContextData = KimeData;

    fn new_ic_data(&mut self) -> Self::InputContextData {
        KimeData {
            engine: InputEngine::new(DubeolSik::new()),
        }
    }

    fn input_styles(&self) -> Self::InputStyleArray {
        [InputStyle::PREEDITNOTHING | InputStyle::PREEDITNOTHING]
    }

    fn handle_connect(&mut self, _server: &mut X11rbServer<C>) -> Result<(), xim::ServerError> {
        Ok(())
    }

    fn handle_create_ic(
        &mut self,
        server: &mut X11rbServer<C>,
        input_context: &mut xim::InputContext<KimeData>,
    ) -> Result<(), xim::ServerError> {
        log::info!("Send event mask");
        server.set_event_mask(
            input_context.client_win(),
            input_context.input_method_id(),
            input_context.input_context_id(),
            EventMask::KeyPress | EventMask::KeyRelease,
            0,
            // EventMask::KeyPress | EventMask::KeyRelease,
        )?;

        Ok(())
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
                InputResult::CommitBypass(ch) => {
                    server.commit(
                        input_context.client_win(),
                        input_context.input_method_id(),
                        input_context.input_context_id(),
                        &ch.to_string(),
                    )?;
                    Ok(false)
                }
                InputResult::Commit(ch) => {
                    server.commit(
                        input_context.client_win(),
                        input_context.input_method_id(),
                        input_context.input_context_id(),
                        &ch.to_string(),
                    )?;
                    Ok(true)
                }
                InputResult::CommitPreedit(commit, _preedit) => {
                    server.commit(
                        input_context.client_win(),
                        input_context.input_method_id(),
                        input_context.input_context_id(),
                        &commit.to_string(),
                    )?;
                    Ok(true)
                }
                InputResult::Preedit(..) => Ok(true),
            }
        } else {
            Ok(false)
        }
    }

    fn handle_destory_ic(&mut self, _input_context: xim::InputContext<Self::InputContextData>) {}
}
