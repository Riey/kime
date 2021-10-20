use kime_engine_backend::{
    InputEngineMode,
    InputEngineModeResult::{self, Continue, Exit, ExitHandled},
    Key,
};

use kime_engine_candidate::client::Client;

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
            self.client = Client::new(entires).ok();
            self.client.is_some()
        } else {
            false
        }
    }
}

impl InputEngineMode for HanjaMode {
    type ConfigData = ();

    fn press_key(
        &mut self,
        _: &(),
        _: Key,
        commit_buf: &mut String,
    ) -> InputEngineModeResult<bool> {
        self.clear_preedit(commit_buf);

        Exit
    }

    fn preedit_str(&self, _: &mut String) {}

    fn clear_preedit(&mut self, commit_buf: &mut String) -> InputEngineModeResult<()> {
        if let Some(mut client) = self.client.take() {
            if let Some(res) = client.close().ok().flatten() {
                commit_buf.push_str(&res);
            }
        }

        Exit
    }

    fn reset(&mut self) -> InputEngineModeResult<()> {
        self.client.take().and_then(|mut c| c.close().ok());

        ExitHandled(())
    }

    fn has_preedit(&self) -> bool {
        true
    }
}
