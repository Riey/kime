use x11rb::protocol::xproto::KeyPressEvent;
use xim::{
    x11rb::{HasConnection, X11rbServer},
    InputStyle, Server, ServerHandler,
};

#[derive(Default)]
pub struct KimeData {}

pub struct KimeHandler {}

impl KimeHandler {
    pub fn new() -> Self {
        Self {}
    }
}

impl<C: HasConnection> ServerHandler<X11rbServer<C>> for KimeHandler {
    type InputStyleArray = [InputStyle; 1];
    type InputContextData = KimeData;

    fn input_styles(&self) -> Self::InputStyleArray {
        [InputStyle::PREEDITNOTHING | InputStyle::PREEDITNOTHING]
    }

    fn handle_connect(&mut self, server: &mut X11rbServer<C>) -> Result<(), xim::ServerError> {
        Ok(())
    }

    fn handle_create_ic(
        &mut self,
        server: &mut X11rbServer<C>,
        input_context: &mut xim::InputContext<KimeData>,
    ) -> Result<(), xim::ServerError> {
        server.commit(input_context.client_win(), input_context.input_method_id(), input_context.input_context_id(), "가나다")?;
        Ok(())
    }

    fn handle_forward_event(
        &mut self,
        server: &mut X11rbServer<C>,
        input_context: &mut xim::InputContext<Self::InputContextData>,
        xev: &KeyPressEvent,
    ) -> Result<bool, xim::ServerError> {
        Ok(true)
    }
}
