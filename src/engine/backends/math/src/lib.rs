use kime_engine_backend::{
    AHashMap, InputEngineMode,
    InputEngineModeResult::{self, Continue},
    Key, KeyCode,
};
use kime_engine_backend_latin::{load_layout, LatinConfig};
use kime_engine_dict::math_symbol_key::*;

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_style() {
        use kime_engine_dict::math_symbol_key::*;

        assert_eq!(crate::parse_style("sf"), STYLE_SF);
        assert_eq!(crate::parse_style("bf"), STYLE_BF);
        assert_eq!(crate::parse_style("it"), STYLE_IT);
        assert_eq!(crate::parse_style("tt"), STYLE_TT);
        assert_eq!(crate::parse_style("bb"), STYLE_BB);
        assert_eq!(crate::parse_style("scr"), STYLE_SCR);
        assert_eq!(crate::parse_style("cal"), STYLE_CAL);
        assert_eq!(crate::parse_style("frak"), STYLE_FRAK);
        assert_eq!(crate::parse_style("fruk"), STYLE_NONE);
        assert_eq!(crate::parse_style("bfit"), STYLE_BF | STYLE_IT);
        assert_eq!(crate::parse_style("bfsfit"), STYLE_SF | STYLE_BF | STYLE_IT);
    }
}

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

fn take_str(s: &str, n: usize) -> &str {
    if s.len() >= n {
        &s[0..n]
    } else {
        s
    }
}

fn parse_style(style_str: &str) -> Style {
    let mut buf: &str = style_str;
    let mut style = STYLE_NONE;

    loop {
        let style_new = match take_str(buf, 2) {
            "" => return style,
            "sf" => {
                buf = &buf[2..];
                STYLE_SF
            }
            "bf" => {
                buf = &buf[2..];
                STYLE_BF
            }
            "it" => {
                buf = &buf[2..];
                STYLE_IT
            }
            "tt" => {
                buf = &buf[2..];
                STYLE_TT
            }
            "bb" => {
                buf = &buf[2..];
                STYLE_BB
            }
            "sc" => {
                if let "r" = take_str(&buf[2..], 1) {
                    buf = &buf[3..];
                    STYLE_SCR
                } else {
                    return STYLE_NONE;
                }
            }
            "ca" => {
                if let "l" = take_str(&buf[2..], 1) {
                    buf = &buf[3..];
                    STYLE_CAL
                } else {
                    return STYLE_NONE;
                }
            }
            "fr" => {
                if let "ak" = take_str(&buf[2..], 2) {
                    buf = &buf[4..];
                    STYLE_FRAK
                } else {
                    return STYLE_NONE;
                }
            }
            _ => return STYLE_NONE,
        };

        style |= style_new;
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
        let mut iter = self.buf.split('.');
        if let Some(first) = iter.next() {
            if let Some(second) = iter.next() {
                let style = parse_style(first);
                if let Some(symbol) = kime_engine_dict::lookup_math_symbol(&second, style) {
                    commit_buf.push_str(symbol);
                }
            } else {
                if let Some(symbol) = kime_engine_dict::lookup_math_symbol(&first, STYLE_NONE) {
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
