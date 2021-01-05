mod compose;
mod dubeolsik;
mod keycode;

pub use self::dubeolsik::DubeolSik;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputResult {
    ClearPreedit,
    Preedit(char),
    Commit(char),
    Consume,
    Bypass,
    CommitBypass(char),
    /// (commit, preedit)
    CommitPreedit(char, char),
}

pub trait InputLayout {
    fn map_key(&mut self, keycode: u8) -> InputResult;
    fn reset(&mut self) -> Option<char>;
}

pub struct InputEngine<Layout: InputLayout> {
    enable_hangul: bool,
    layout: Layout,
}

impl<Layout: InputLayout> InputEngine<Layout> {
    pub fn new(layout: Layout) -> Self {
        Self {
            enable_hangul: false,
            layout,
        }
    }

    pub fn key_press(&mut self, keycode: u8, _shift: bool, ctrl: bool) -> InputResult {
        // skip ctrl
        if ctrl {
            return InputResult::Bypass;
        }

        if matches!(keycode, keycode::HENKAN | keycode::R_ALT) {
            log::trace!("Trigger hangul");
            self.enable_hangul = !self.enable_hangul;
            return InputResult::Consume;
        }

        if self.enable_hangul {
            self.layout.map_key(keycode)
        } else {
            InputResult::Bypass
        }
    }

    pub fn reset(&mut self) -> String {
        self.layout.reset().map_or(String::new(), Into::into)
    }
}
