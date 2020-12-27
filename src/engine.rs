mod compose;
mod dubeolsik;
mod keycode;

pub use self::dubeolsik::DubeolSik;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputResult {
    Preedit(char),
    Commit(char),
    Bypass,
    CommitBypass(char),
}

pub trait InputLayout {
    fn map_key(&mut self, keycode: u8) -> InputResult;
}

pub struct InputEngine<Layout: InputLayout> {
    enable_hangul: bool,
    layout: Layout,
}

impl<Layout: InputLayout> InputEngine<Layout> {
    pub fn new(layout: Layout) -> Self {
        Self {
            enable_hangul: true,
            layout,
        }
    }

    pub fn key_press(&mut self, keycode: u8) -> InputResult {
        if self.enable_hangul {
            self.layout.map_key(keycode)
        } else {
            InputResult::Bypass
        }
    }
}
