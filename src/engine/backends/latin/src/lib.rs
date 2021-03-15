use kime_engine_core::{AHashMap, InputEngine, Key};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum LatinLayout {
    Qwerty,
    Colemak,
}

#[derive(Serialize, Deserialize)]
pub struct LatinConfig {
    layout: LatinLayout,
}

impl Default for LatinConfig {
    fn default() -> Self {
        Self {
            layout: LatinLayout::Qwerty,
        }
    }
}

#[derive(Clone)]
pub struct LatinEngine {
    layout: AHashMap<Key, char>,
    buf: String,
}

impl LatinEngine {
    pub fn new(config: &LatinConfig) -> Self {
        let layout = match config.layout {
            LatinLayout::Qwerty => include_str!("../data/qwerty.yaml"),
            LatinLayout::Colemak => include_str!("../data/colemak.yaml"),
        };
        Self {
            layout: serde_yaml::from_str(layout).unwrap_or_default(),
            buf: String::with_capacity(16),
        }
    }
}

impl InputEngine for LatinEngine {
    fn press_key(&mut self, key: Key) -> bool {
        if let Some(ch) = self.layout.get(&key) {
            self.buf.push(*ch);
            true
        } else {
            false
        }
    }

    fn clear_commit(&mut self) {
        self.buf.clear();
    }

    fn clear_preedit(&mut self) {}

    fn reset(&mut self) {
        self.buf.clear();
    }

    fn has_preedit(&self) -> bool {
        false
    }

    fn preedit_str(&self, _buf: &mut String) {}

    fn commit_str(&self) -> &str {
        &self.buf
    }

    fn pass(&mut self, s: &str) {
        self.buf.push_str(s);
    }
}
