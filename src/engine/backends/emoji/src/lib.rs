use kime_engine_backend::{
    AHashMap, InputEngineMode,
    InputEngineModeResult::{self, Continue, Exit},
    Key, KeyCode,
};
use kime_engine_backend_latin::{load_layout, LatinConfig};

#[derive(Clone)]
pub struct EmojiMode {
    buf: String,
    layout: AHashMap<Key, char>,
}

impl EmojiMode {
    pub fn new(config: &LatinConfig) -> Self {
        Self {
            buf: String::with_capacity(16),
            layout: load_layout(config),
        }
    }
}

impl InputEngineMode for EmojiMode {
    fn press_key(&mut self, key: Key, _commit_buf: &mut String) -> InputEngineModeResult<bool> {
        if key.code == KeyCode::Backspace {
            if self.buf.pop().is_some() {
                Continue(true)
            } else {
                Exit
            }
        } else if let Some(ch) = self.layout.get(&key) {
            self.buf.push(*ch);
            Continue(true)
        } else {
            Continue(false)
        }
    }

    fn clear_preedit(&mut self, commit_buf: &mut String) -> InputEngineModeResult<()> {
        if !self.buf.is_empty() {
            if let Some(anno) = kime_engine_dict::search_unicode_annotations(&self.buf).next() {
                commit_buf.push_str(anno.codepoint);
            }
        }

        Exit
    }

    fn reset(&mut self) -> InputEngineModeResult<()> {
        Exit
    }

    fn preedit_str(&self, buf: &mut String) {
        buf.push_str(&self.buf);
        for anno in kime_engine_dict::search_unicode_annotations(&self.buf).take(5) {
            buf.push_str(anno.codepoint);
            buf.push('(');
            buf.push_str(anno.tts);
            buf.push(')');
        }
    }

    fn has_preedit(&self) -> bool {
        true
    }
}
