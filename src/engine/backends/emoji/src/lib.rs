use kime_engine_backend::{
    InputEngineMode,
    InputEngineModeResult::{self, Continue, Exit, ExitHandled},
    Key, KeyCode,
};
use kime_engine_backend_latin::LatinData;

#[derive(Clone)]
pub struct EmojiMode {
    buf: String,
}

impl EmojiMode {
    pub fn new() -> Self {
        Self {
            buf: String::with_capacity(16),
        }
    }
}

impl InputEngineMode for EmojiMode {
    type ConfigData = LatinData;

    fn press_key(
        &mut self,
        config: &LatinData,
        key: Key,
        _commit_buf: &mut String,
    ) -> InputEngineModeResult<bool> {
        if key.code == KeyCode::Backspace {
            if self.buf.pop().is_some() {
                Continue(true)
            } else {
                Exit
            }
        } else if key == Key::normal(KeyCode::Space) {
            self.buf.push(' ');
            Continue(true)
        } else if let Some(ch) = config.lookup(key) {
            self.buf.push(ch);
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
            self.buf.clear();
        }

        ExitHandled(())
    }

    fn reset(&mut self) -> InputEngineModeResult<()> {
        self.buf.clear();
        ExitHandled(())
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
