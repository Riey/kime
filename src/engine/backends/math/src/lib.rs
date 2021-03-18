use kime_engine_backend::{
    AHashMap, InputEngineMode,
    InputEngineModeResult::{self, Continue},
    Key, KeyCode,
};
use kime_engine_backend_latin::{load_layout, LatinConfig};
use kime_engine_dict::math_symbol_key::*;

#[derive(Clone)]
pub struct MathMode {
    math_mode: bool,
    buf: String,
    layout: AHashMap<Key, char>,
}

impl MathMode {
    pub fn new(config: &LatinConfig) -> Self {
        Self {
            math_mode: false,
            buf: String::with_capacity(16),
            layout: load_layout(config),
        }
    }
}

impl InputEngineMode for MathMode {
    fn press_key(&mut self, key: Key, commit_buf: &mut String) -> InputEngineModeResult<bool> {
        if key == Key::normal(KeyCode::Backslash) {
            if self.math_mode {
                // double backslash
                self.math_mode = false;
                commit_buf.push('\\');
            } else {
                self.math_mode = true;
            }

            return Continue(true);
        }

        if self.math_mode && key.code == KeyCode::Backspace {
            if self.buf.pop().is_none() {
                self.math_mode = false;
            }

            return Continue(true);
        }

        if let Some(ch) = self.layout.get(&key) {
            if self.math_mode {
                self.buf.push(*ch);
            } else {
                commit_buf.push(*ch);
            }

            Continue(true)
        } else {
            Continue(false)
        }
    }

    fn clear_preedit(&mut self, commit_buf: &mut String) -> InputEngineModeResult<()> {
        if let Some(symbol) = kime_engine_dict::lookup_math_symbol(&self.buf, STYLE_NONE) {
            commit_buf.push_str(symbol);
        }
        self.buf.clear();
        self.math_mode = false;
        Continue(())
    }

    fn reset(&mut self) -> InputEngineModeResult<()> {
        self.buf.clear();
        self.math_mode = false;
        Continue(())
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
