use kime_engine_backend::{AHashMap, InputEngineBackend, Key, KeyCode};
use kime_engine_backend_latin::{load_layout, LatinConfig};

#[derive(Clone)]
pub struct MathEngine {
    math_mode: bool,
    buf: String,
    layout: AHashMap<Key, char>,
}

impl MathEngine {
    pub fn new(config: &LatinConfig) -> Self {
        Self {
            math_mode: false,
            buf: String::with_capacity(16),
            layout: load_layout(config),
        }
    }
}

impl InputEngineBackend for MathEngine {
    fn press_key(&mut self, key: Key, commit_buf: &mut String) -> bool {
        if key == Key::normal(KeyCode::Backslash) {
            if self.math_mode {
                // double backslash
                self.math_mode = false;
                commit_buf.push('\\');
            } else {
                self.math_mode = true;
            }

            return true;
        }

        if let Some(ch) = self.layout.get(&key) {
            if self.math_mode {
                self.buf.push(*ch);
            } else {
                commit_buf.push(*ch);
            }

            true
        } else {
            false
        }
    }

    fn clear_preedit(&mut self, commit_buf: &mut String) {
        if let Some(symbol) = kime_engine_dict::lookup_math_symbol(&self.buf) {
            commit_buf.push_str(symbol);
        }
        self.buf.clear();
        self.math_mode = false;
    }

    fn reset(&mut self) {
        self.buf.clear();
        self.math_mode = false;
    }

    fn preedit_str(&self, buf: &mut String) {
        if self.math_mode {
            buf.push('\\');
            buf.push_str(&self.buf);
        }
    }

    fn has_preedit(&self) -> bool {
        self.math_mode
    }
}
