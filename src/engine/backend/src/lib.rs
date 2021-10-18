mod input_result;
mod keycode;
mod keymap;

pub use keycode::{Key, KeyCode, ModifierState};
pub use keymap::KeyMap;

pub use input_result::InputResult;

pub trait InputEngineBackend {
    type ConfigData;

    /// Press key
    /// # Return
    /// `true` means key has handled
    fn press_key(&mut self, config: &Self::ConfigData, key: Key, commit_buf: &mut String) -> bool;
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
    ExitHandled(T),
    Exit,
}

pub trait InputEngineMode {
    type ConfigData;

    /// Press key
    /// # Return
    /// `Ok(true)` means key has handled
    fn press_key(
        &mut self,
        config: &Self::ConfigData,
        key: Key,
        commit_buf: &mut String,
    ) -> InputEngineModeResult<bool>;
    /// Clear current preedit string this function may change commit string
    fn clear_preedit(&mut self, commit_buf: &mut String) -> InputEngineModeResult<()>;
    /// Clear engine state
    fn reset(&mut self) -> InputEngineModeResult<()>;
    /// Get preedit string
    fn preedit_str(&self, buf: &mut String);
    /// Is have preedit
    fn has_preedit(&self) -> bool;
    /// Is now ready
    #[allow(unused_variables)]
    fn check_ready(&mut self, commit_buf: &mut String) -> InputEngineModeResult<bool> {
        InputEngineModeResult::Continue(true)
    }
}
