use kime_engine_backend::{
    InputEngineMode,
    InputEngineModeResult::{self, Continue},
    Key, KeyCode,
};
use kime_engine_backend_latin::LatinData;
use kime_engine_dict::math_symbol_key::*;

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_style() {
        use kime_engine_dict::math_symbol_key::*;

        assert_eq!(crate::parse_style("sf"), Style::SF);
        assert_eq!(crate::parse_style("bf"), Style::BF);
        assert_eq!(crate::parse_style("it"), Style::IT);
        assert_eq!(crate::parse_style("tt"), Style::TT);
        assert_eq!(crate::parse_style("bb"), Style::BB);
        assert_eq!(crate::parse_style("scr"), Style::SCR);
        assert_eq!(crate::parse_style("cal"), Style::CAL);
        assert_eq!(crate::parse_style("frak"), Style::FRAK);
        assert_eq!(crate::parse_style("fruk"), Style::NONE);
        assert_eq!(crate::parse_style("bfit"), Style::BF | Style::IT);
        assert_eq!(
            crate::parse_style("bfsfit"),
            Style::SF | Style::BF | Style::IT
        );
    }
}

#[derive(Clone)]
pub struct MathMode {
    math_mode: bool,
    buf: String,
}

impl MathMode {
    pub fn new() -> Self {
        Self {
            math_mode: false,
            buf: String::with_capacity(16),
        }
    }
}

fn parse_style(style_str: &str) -> Style {
    let mut buf: &str = style_str;
    let mut style = Style::NONE;

    loop {
        let style_new = if buf.is_empty() {
            return style;
        } else if let Some(_buf) = buf.strip_prefix("sf") {
            buf = _buf;
            Style::SF
        } else if let Some(_buf) = buf.strip_prefix("bf") {
            buf = _buf;
            Style::BF
        } else if let Some(_buf) = buf.strip_prefix("it") {
            buf = _buf;
            Style::IT
        } else if let Some(_buf) = buf.strip_prefix("tt") {
            buf = _buf;
            Style::TT
        } else if let Some(_buf) = buf.strip_prefix("bb") {
            buf = _buf;
            Style::BB
        } else if let Some(_buf) = buf.strip_prefix("scr") {
            buf = _buf;
            Style::SCR
        } else if let Some(_buf) = buf.strip_prefix("cal") {
            buf = _buf;
            Style::CAL
        } else if let Some(_buf) = buf.strip_prefix("frak") {
            buf = _buf;
            Style::FRAK
        } else {
            return Style::NONE;
        };

        style |= style_new;
    }
}

impl InputEngineMode for MathMode {
    type ConfigData = LatinData;

    fn press_key(
        &mut self,
        config: &LatinData,
        key: Key,
        commit_buf: &mut String,
    ) -> InputEngineModeResult<bool> {
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

        if let Some(ch) = config.lookup(&key) {
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
        let mut iter = self.buf.split('.');
        if let Some(first) = iter.next() {
            if let Some(second) = iter.next() {
                let style = parse_style(first);
                if let Some(symbol) = kime_engine_dict::lookup_math_symbol(&second, style) {
                    commit_buf.push_str(symbol);
                }
            } else {
                if let Some(symbol) = kime_engine_dict::lookup_math_symbol(&first, Style::NONE) {
                    commit_buf.push_str(symbol);
                }
            }
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
