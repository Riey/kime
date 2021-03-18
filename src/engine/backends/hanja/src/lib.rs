use kime_engine_backend::{
    InputEngineMode,
    InputEngineModeResult::{self, Continue, Exit},
    Key, KeyCode,
};

#[derive(Debug, Clone)]
pub struct HanjaMode {
    hanja_entires: &'static [(char, &'static str)],
    index: usize,
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
        }
    }

    pub fn set_key(&mut self, key: char) -> bool {
        if let Some(entires) = kime_engine_dict::lookup(key) {
            self.hanja_entires = entires;
            self.index = 0;
            true
        } else {
            false
        }
    }

    fn current_hanja(&self) -> char {
        self.hanja_entires[self.index].0
    }

    fn try_index(&mut self, index: usize) {
        self.index = index.min(self.hanja_entires.len() - 1);
    }
}

impl InputEngineMode for HanjaMode {
    fn press_key(&mut self, key: Key, commit_buf: &mut String) -> InputEngineModeResult<bool> {
        match key.code {
            KeyCode::Left => {
                self.index = self
                    .index
                    .checked_sub(1)
                    .unwrap_or(self.hanja_entires.len() - 1);
                Continue(true)
            }
            KeyCode::Right => {
                self.try_index(self.index + 1);
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
            | KeyCode::Nine => {
                self.try_index(key.code as usize - KeyCode::One as usize);
                Continue(true)
            }
            _ => {
                commit_buf.push(self.current_hanja());
                Exit
            }
        }
    }

    fn preedit_str(&self, buf: &mut String) {
        let current_hanja = &self.hanja_entires[self.index];

        for (prev_hanja, _) in self.hanja_entires[..self.index].iter() {
            buf.push(*prev_hanja);
        }

        buf.push('/');
        buf.push(current_hanja.0);
        buf.push('(');
        buf.push_str(current_hanja.1);
        buf.push(')');

        if let Some(next_hanjas) = self.hanja_entires.get(self.index + 1..) {
            for (next_hanja, _) in next_hanjas {
                buf.push(*next_hanja);
            }
        }
    }

    fn clear_preedit(&mut self, commit_buf: &mut String) -> InputEngineModeResult<()> {
        commit_buf.push(self.current_hanja());
        self.reset()
    }

    fn reset(&mut self) -> InputEngineModeResult<()> {
        Exit
    }

    fn has_preedit(&self) -> bool {
        true
    }
}
