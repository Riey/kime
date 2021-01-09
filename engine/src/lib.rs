mod characters;
mod dubeolsik;
mod state;

use self::characters::{Choseong, Jongseong, Jungseong, KeyValue};
use ahash::AHashMap;
use serde::{Deserialize, Serialize};
use xkbcommon::xkb;

pub use self::state::CharacterState;

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
    pub fn dubeolsik() -> Self {
        Self::load_from(self::dubeolsik::DUBEOLSIK_LAYOUT)
    }

    pub fn load_from(content: &str) -> Self {
        let mut keymap = AHashMap::new();

        let items: AHashMap<String, ValueItem> = serde_yaml::from_str(content).unwrap();

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

    pub fn map_key(&self, state: &mut CharacterState, sym: xkb::Keysym) -> InputResult {
        if sym == xkb::KEY_BackSpace {
            state.backspace()
        } else {
            if let Some(v) = self.keymap.get(&sym) {
                match *v {
                    KeyValue::Pass(pass) => {
                        if let Some(preedit) = state.preedit_char() {
                            state.reset();
                            InputResult::CommitCommit(preedit, pass)
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
        let keymap = xkb::Keymap::new_from_names(&ctx, "", "", "", "", None, 0).unwrap();
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
    layout: Layout,
    xkb_ctx: XkbContext,
    enable_hangul: bool,
}

impl InputEngine {
    pub fn new(layout: Layout) -> Self {
        Self {
            state: CharacterState::default(),
            layout,
            xkb_ctx: XkbContext::new(),
            enable_hangul: false,
        }
    }

    fn bypass(&mut self, commit: Option<char>) -> InputResult {
        match (self.state.reset(), commit) {
            (Some(preedit), Some(commit)) => InputResult::CommitCommit(preedit, commit),
            (Some(preedit), None) => InputResult::CommitBypass(preedit),
            (None, Some(commit)) => InputResult::Commit(commit),
            (None, None) => InputResult::Bypass,
        }
    }

    pub fn key_event(&mut self, keycode: u32, press: bool) -> InputResult {
        self.xkb_ctx.state.update_key(
            keycode,
            if press {
                xkb::KeyDirection::Down
            } else {
                xkb::KeyDirection::Up
            },
        );

        // Skip when ctrl pressed
        if self
            .xkb_ctx
            .state
            .mod_name_is_active(xkb::MOD_NAME_CTRL, xkb::STATE_MODS_DEPRESSED)
        {
            return self.bypass(None);
        }

        // Skip when release event
        if !press {
            return InputResult::Bypass;
        }

        let sym = self.xkb_ctx.state.key_get_one_sym(keycode);

        match sym {
            xkb::KEY_Hangul | xkb::KEY_Henkan | xkb::KEY_Alt_R => {
                self.enable_hangul = !self.enable_hangul;
                InputResult::Consume
            }
            sym if self.enable_hangul => self.layout.map_key(&mut self.state, sym),
            sym => {
                let commit = unsafe { std::char::from_u32_unchecked(xkb::keysym_to_utf32(sym)) };

                self.bypass(if commit.is_ascii_alphanumeric() {
                    Some(commit)
                } else {
                    None
                })
            }
        }
    }

    #[inline]
    pub fn preedit_char(&self) -> Option<char> {
        self.state.preedit_char()
    }

    #[inline]
    pub fn reset(&mut self) -> Option<char> {
        self.state.reset()
    }
}
