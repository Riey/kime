mod input_result;
mod keycode;

pub use ahash::AHashMap;
pub use keycode::{Key, KeyCode, ModifierState};

pub use input_result::InputResult;

pub trait InputEngineBackend {
    /// Press key
    /// # Return
    /// `true` means key has handled
    fn press_key(&mut self, key: Key, commit_buf: &mut String) -> bool;
    /// Clear current preedit string this function may change commit string
    fn clear_preedit(&mut self, commit_buf: &mut String);
    /// Clear engine state
    fn reset(&mut self);
    /// Get preedit string
    fn preedit_str(&self, buf: &mut String);
    /// Is have preedit
    fn has_preedit(&self) -> bool;
}

pub enum InputEngineModeResult<T> {
    Continue(T),
    Exit,
}

pub trait InputEngineMode {
    /// Press key
    /// # Return
    /// `Ok(true)` means key has handled
    fn press_key(&mut self, key: Key, commit_buf: &mut String) -> InputEngineModeResult<bool>;
    /// Clear current preedit string this function may change commit string
    fn clear_preedit(&mut self, commit_buf: &mut String) -> InputEngineModeResult<()>;
    /// Clear engine state
    fn reset(&mut self) -> InputEngineModeResult<()>;
    /// Get preedit string
    fn preedit_str(&self, buf: &mut String);
    /// Is have preedit
    fn has_preedit(&self) -> bool;
}
