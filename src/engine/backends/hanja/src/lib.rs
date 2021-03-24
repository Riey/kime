use kime_engine_backend::{
    InputEngineMode,
    InputEngineModeResult::{self, Continue, Exit},
    Key, KeyCode,
};

#[derive(Debug, Clone)]
pub struct HanjaMode {
    hanja_entires: &'static [(&'static str, &'static str)],
    index: usize,
    max_index: usize,
}

impl Default for HanjaMode {
    fn default() -> Self {
        Self::new()
    }
}

impl HanjaMode {
    pub fn new() -> Self {
        Self {
            hanja_entires: &[],
            index: 0,
            max_index: 0,
        }
    }

    pub fn set_key(&mut self, key: &str) -> bool {
        if let Some(entires) = kime_engine_dict::lookup(key) {
            self.hanja_entires = entires;
            self.index = 0;
            self.max_index = if entires.len() % 10 == 0 {
                (entires.len() / 10) - 1
            } else {
                entires.len() / 10
            };
            true
        } else {
            false
        }
    }
}

impl InputEngineMode for HanjaMode {
    type ConfigData = ();

    fn press_key(
        &mut self,
        _: &(),
        key: Key,
        commit_buf: &mut String,
    ) -> InputEngineModeResult<bool> {
        match key.code {
            KeyCode::Left | KeyCode::PageUp => {
                self.index = self.index.checked_sub(1).unwrap_or(self.max_index);
                Continue(true)
            }
            KeyCode::Right | KeyCode::PageDown => {
                if self.index == self.max_index {
                    self.index = 0;
                } else {
                    self.index += 1;
                }
                Continue(true)
            }
            KeyCode::One
            | KeyCode::Two
            | KeyCode::Three
            | KeyCode::Four
            | KeyCode::Five
            | KeyCode::Six
            | KeyCode::Seven
            | KeyCode::Eight
            | KeyCode::Nine
            | KeyCode::Zero => {
                let idx = key.code as usize - KeyCode::One as usize;
                if let Some(entry) = self.hanja_entires.get(self.index * 10 + idx) {
                    commit_buf.push_str(entry.0);
                    Exit
                } else {
                    Continue(true)
                }
            }
            _ => Exit,
        }
    }

    fn preedit_str(&self, buf: &mut String) {
        let range = (self.index * 10)..(self.index * 10 + 10).min(self.hanja_entires.len());

        use std::fmt::Write;
        write!(buf, "{}/{} ", self.index, self.max_index).ok();

        for (idx, entry) in self.hanja_entires[range].iter().enumerate() {
            write!(buf, "[{}] {}({})", idx + 1, entry.0, entry.1).ok();
        }
    }

    fn clear_preedit(&mut self, _commit_buf: &mut String) -> InputEngineModeResult<()> {
        Continue(())
    }

    fn reset(&mut self) -> InputEngineModeResult<()> {
        Exit
    }

    fn has_preedit(&self) -> bool {
        true
    }
}
