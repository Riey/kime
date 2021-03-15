mod input_result;
mod keycode;

pub use ahash::AHashMap;
pub use keycode::{Key, KeyCode, ModifierState};

pub use input_result::InputResult;

pub trait InputEngine {
    /// Press key
    /// # Return
    /// `true` means key has handled
    fn press_key(&mut self, key: Key) -> bool;
    /// Clear current commit string
    fn clear_commit(&mut self);
    /// Clear current preedit string this function may change commit string
    fn clear_preedit(&mut self);
    /// Clear current preedit string this function must not change commit string
    fn remove_preedit(&mut self) {
        self.clear_preedit();
    }
    /// Clear engine state
    fn reset(&mut self);
    /// Get preedit string
    fn preedit_str(&self, buf: &mut String);
    /// Get commit string
    fn commit_str(&self) -> &str;
    /// Is have preedit
    fn has_preedit(&self) -> bool;
    /// Append string to commit_string
    fn pass(&mut self, s: &str);
}
