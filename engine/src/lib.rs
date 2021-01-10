mod characters;
mod config;
mod state;

use self::characters::{Choseong, Jongseong, Jungseong, KeyValue};
use ahash::AHashMap;
use serde::{Deserialize, Serialize};
use xkbcommon::xkb;

pub use self::config::Config;
pub use self::state::CharacterState;

#[derive(Clone, Default)]
pub struct Layout {
    keymap: AHashMap<xkb::Keysym, KeyValue>,
}

#[derive(Serialize, Deserialize)]
struct ValueItem {
    #[serde(default)]
    cho: Option<Choseong>,
    #[serde(default)]
    jung: Option<Jungseong>,
    #[serde(default)]
    jong: Option<Jongseong>,
    #[serde(default)]
    pass: Option<char>,
}

impl Layout {
    fn from_items(items: AHashMap<String, ValueItem>) -> Self {
        let mut keymap = AHashMap::new();

        for (key, value) in items {
            let key = xkb::keysym_from_name(&key, xkb::KEYSYM_NO_FLAGS);

            let value = match value {
                ValueItem {
                    pass: Some(pass), ..
                } => KeyValue::Pass(pass),
                ValueItem {
                    cho: Some(cho),
                    jong: Some(jong),
                    ..
                } => KeyValue::ChoJong(cho, jong),
                ValueItem { cho: Some(cho), .. } => KeyValue::Choseong(cho),
                ValueItem {
                    jong: Some(jong), ..
                } => KeyValue::Jongseong(jong),
                ValueItem {
                    jung: Some(jung), ..
                } => KeyValue::Jungseong(jung),
                _ => continue,
            };

            keymap.insert(key, value);
        }

        Self { keymap }
    }

    pub fn load_from(content: &str) -> Option<Self> {
        Some(Self::from_items(serde_yaml::from_str(content).ok()?))
    }

    pub fn map_key(&self, state: &mut CharacterState, sym: xkb::Keysym) -> InputResult {
        if sym == xkb::KEY_BackSpace {
            state.backspace()
        } else {
            if let Some(v) = self.keymap.get(&sym) {
                match *v {
                    KeyValue::Pass(pass) => {
                        if let Some(commit) = state.reset() {
                            InputResult::CommitCommit(commit, pass)
                        } else {
                            InputResult::Commit(pass)
                        }
                    }
                    KeyValue::ChoJong(cho, jong) => state.cho_jong(cho, jong),
                    KeyValue::Jungseong(jung) => state.jung(jung),
                    KeyValue::Choseong(cho) => state.cho(cho),
                    KeyValue::Jongseong(jong) => state.jong(jong),
                }
            } else {
                InputResult::Bypass
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputResult {
    ClearPreedit,
    Preedit(char),
    Consume,
    Bypass,
    Commit(char),
    CommitBypass(char),
    /// (commit, preedit)
    CommitPreedit(char, char),
    CommitCommit(char, char),
}

pub struct XkbContext {
    _ctx: xkb::Context,
    _keymap: xkb::Keymap,
    state: xkb::State,
}

impl XkbContext {
    pub fn new() -> Self {
        let ctx = xkb::Context::new(0);
        let keymap = xkb::Keymap::new_from_names(&ctx, "", "", "", "qwerty", None, 0).unwrap();
        let state = xkb::State::new(&keymap);

        Self {
            _ctx: ctx,
            _keymap: keymap,
            state,
        }
    }
}

pub struct InputEngine {
    state: CharacterState,
    xkb_ctx: XkbContext,
    enable_hangul: bool,
}

impl InputEngine {
    pub fn new() -> Self {
        Self {
            state: CharacterState::default(),
            xkb_ctx: XkbContext::new(),
            enable_hangul: false,
        }
    }

    pub fn set_enable_hangul(&mut self, enable: bool) {
        self.enable_hangul = enable;
    }

    fn bypass(&mut self, commit: Option<char>) -> InputResult {
        match (self.state.reset(), commit) {
            (Some(preedit), Some(commit)) => InputResult::CommitCommit(preedit, commit),
            (Some(preedit), None) => InputResult::CommitBypass(preedit),
            (None, Some(commit)) => InputResult::Commit(commit),
            (None, None) => InputResult::Bypass,
        }
    }

    /// Use pre-computed keysym
    pub fn press_key_sym(&mut self, sym: xkb::Keysym, config: &Config) -> InputResult {
        match sym {
            xkb::KEY_Escape if config.esc_turn_off => {
                if self.enable_hangul {
                    self.enable_hangul = false;
                    self.bypass(None)
                } else {
                    InputResult::Bypass
                }
            }
            sym if config.hangul_symbols.contains(&sym) => {
                self.enable_hangul = !self.enable_hangul;
                InputResult::Consume
            }
            sym if self.enable_hangul => config.layout.map_key(&mut self.state, sym),
            sym => {
                let commit = unsafe { std::char::from_u32_unchecked(xkb::keysym_to_utf32(sym)) };

                self.bypass(if !commit.is_ascii_control() {
                    Some(commit)
                } else {
                    None
                })
            }
        }
    }

    /// Use hardward keycode
    pub fn key_event(
        &mut self,
        keycode: xkb::Keycode,
        press: bool,
        config: &Config,
    ) -> InputResult {
        self.xkb_ctx.state.update_key(
            keycode,
            if press {
                xkb::KeyDirection::Down
            } else {
                xkb::KeyDirection::Up
            },
        );

        // Skip when release event
        if !press {
            return InputResult::Bypass;
        }

        // Skip when ctrl pressed
        if self
            .xkb_ctx
            .state
            .mod_name_is_active(xkb::MOD_NAME_CTRL, xkb::STATE_MODS_DEPRESSED)
        {
            return self.bypass(None);
        }

        let sym = self.xkb_ctx.state.key_get_one_sym(keycode);

        self.press_key_sym(sym, config)
    }

    #[inline]
    pub fn preedit_char(&self) -> char {
        self.state.to_char()
    }

    #[inline]
    pub fn reset(&mut self) -> Option<char> {
        self.state.reset()
    }
}
