use kime_engine_backend::{InputEngineBackend, Key, KeyMap};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum LatinLayout {
    Qwerty,
    Dvorak,
    Colemak,
}

#[derive(Serialize, Deserialize)]
pub struct LatinConfig {
    pub layout: LatinLayout,
}

impl Default for LatinConfig {
    fn default() -> Self {
        Self {
            layout: LatinLayout::Qwerty,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct LatinData {
    keymap: KeyMap<char>,
}

impl LatinData {
    pub fn new(config: &LatinConfig) -> Self {
        Self {
            keymap: load_layout(config),
        }
    }

    #[inline]
    pub fn lookup(&self, key: Key) -> Option<char> {
        self.keymap.get(key)
    }
}

fn load_layout(config: &LatinConfig) -> KeyMap<char> {
    let layout = match config.layout {
        LatinLayout::Qwerty => include_str!("../data/qwerty.yaml"),
        LatinLayout::Dvorak => include_str!("../data/dvorak.yaml"),
        LatinLayout::Colemak => include_str!("../data/colemak.yaml"),
    };
    serde_yaml::from_str(layout).unwrap_or_default()
}

#[derive(Clone)]
pub struct LatinEngine(());

impl LatinEngine {
    pub fn new() -> Self {
        Self(())
    }
}

impl InputEngineBackend for LatinEngine {
    type ConfigData = LatinData;

    fn press_key(&mut self, config: &LatinData, key: Key, commit_buf: &mut String) -> bool {
        if let Some(ch) = config.lookup(key) {
            commit_buf.push(ch);
            true
        } else {
            false
        }
    }

    fn clear_preedit(&mut self, _commit_buf: &mut String) {}
    fn reset(&mut self) {}

    fn has_preedit(&self) -> bool {
        false
    }

    fn preedit_str(&self, _buf: &mut String) {}
}
