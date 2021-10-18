use kime_engine_backend::{
    InputEngineMode,
    InputEngineModeResult::{self, Continue, Exit, ExitHandled},
    Key,
};

use kime_engine_candidate::{client::Client, server::Response};

#[derive(Debug)]
pub struct HanjaMode {
    client: Option<Client>,
}

impl Default for HanjaMode {
    fn default() -> Self {
        Self::new()
    }
}

impl HanjaMode {
    pub fn new() -> Self {
        Self { client: None }
    }

    pub fn set_key(&mut self, key: &str) -> bool {
        if let Some(entires) = kime_engine_dict::lookup(key) {
            match Client::new(entires) {
                Ok(client) => {
                    self.client = Some(client);
                    true
                }
                Err(err) => {
                    #[cfg(debug_assertions)]
                    eprintln!("Can't spawn candidate window {:#?}", err);
                    false
                }
            }
        } else {
            false
        }
    }
}

impl InputEngineMode for HanjaMode {
    type ConfigData = ();

    fn press_key(&mut self, _: &(), _: Key, _: &mut String) -> InputEngineModeResult<bool> {
        self.reset();

        Exit
    }

    fn preedit_str(&self, _: &mut String) {}

    fn clear_preedit(&mut self, _: &mut String) -> InputEngineModeResult<()> {
        Continue(())
    }

    fn reset(&mut self) -> InputEngineModeResult<()> {
        self.client.take().and_then(|mut c| c.close().ok());

        ExitHandled(())
    }

    fn has_preedit(&self) -> bool {
        true
    }

    fn check_ready(&mut self, commit_buf: &mut String) -> InputEngineModeResult<bool> {
        match self.client.as_mut() {
            Some(client) => {
                if let Ok(msg) = client.try_recv_msg() {
                    match msg {
                        Some(Response::Quit) => ExitHandled(true),
                        Some(Response::Selected(res)) => {
                            commit_buf.push_str(&res);
                            ExitHandled(true)
                        }
                        None => Continue(false),
                    }
                } else {
                    Exit
                }
            }
            None => Exit,
        }
    }
}
