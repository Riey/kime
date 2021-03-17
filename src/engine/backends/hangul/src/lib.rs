mod characters;
mod layout;
mod state;

use layout::Layout;
use state::{HangulState, HanjaState};
use std::{borrow::Cow, collections::BTreeMap};

use enumset::{EnumSet, EnumSetType};
use kime_engine_backend::{InputEngineBackend, Key, KeyCode};
use serde::{Deserialize, Serialize};

#[derive(Hash, Serialize, Deserialize, Debug, EnumSetType)]
#[enumset(serialize_as_list)]
pub enum Addon {
    ComposeChoseongSsang,
    ComposeJungseongSsang,
    ComposeJongseongSsang,
    DecomposeChoseongSsang,
    DecomposeJungseongSsang,
    DecomposeJongseongSsang,

    /// ㅏ + ㄱ = 가
    FlexibleComposeOrder,

    /// 안 + ㅣ = 아니
    TreatJongseongAsChoseong,
    /// 읅 + ㄱ = 을ㄲ
    TreatJongseongAsChoseongCompose,
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct HangulConfig {
    pub layout: String,
    pub word_commit: bool,
    pub addons: BTreeMap<String, EnumSet<Addon>>,
}

impl Default for HangulConfig {
    fn default() -> Self {
        Self {
            layout: "dubeolsik".into(),
            word_commit: false,
            addons: vec![
                ("all".into(), Addon::ComposeChoseongSsang.into()),
                ("dubeolsik".into(), Addon::TreatJongseongAsChoseong.into()),
            ]
            .into_iter()
            .collect(),
        }
    }
}

pub const BUILTIN_LAYOUTS: &'static [(&'static str, &'static str)] = &[
    ("dubeolsik", include_str!("../data/dubeolsik.yaml")),
    (
        "sebeolsik-3-90",
        include_str!("../data/sebeolsik-3-90.yaml"),
    ),
    (
        "sebeolsik-3-91",
        include_str!("../data/sebeolsik-3-91.yaml"),
    ),
    (
        "sebeolsik-3sin-1995",
        include_str!("../data/sebeolsik-3sin-1995.yaml"),
    ),
    (
        "sebeolsik-3sin-p2",
        include_str!("../data/sebeolsik-3sin-p2.yaml"),
    ),
];

#[derive(Clone)]
pub struct HangulEngine {
    layout: Layout,
    addons: EnumSet<Addon>,
    hanja_state: Option<HanjaState>,
    state: HangulState,
}

impl Default for HangulEngine {
    fn default() -> Self {
        Self::new(&HangulConfig::default(), builtin_layouts())
    }
}

impl HangulEngine {
    #[cfg(unix)]
    pub fn from_config_with_dir(config: &HangulConfig, dir: &xdg::BaseDirectories) -> Self {
        let custom_layouts = dir
            .list_config_files("layouts")
            .into_iter()
            .filter_map(|path| {
                let name = path.file_stem()?.to_str()?;

                Layout::load_from(std::fs::read_to_string(&path).ok()?.as_str())
                    .ok()
                    .map(move |l| (name.to_string().into(), l))
            });

        Self::new(config, custom_layouts.chain(builtin_layouts()))
    }

    pub fn new(
        config: &HangulConfig,
        mut layouts: impl Iterator<Item = (Cow<'static, str>, Layout)>,
    ) -> Self {
        Self {
            layout: layouts
                .find_map(|(name, layout)| {
                    if name == config.layout {
                        Some(layout)
                    } else {
                        None
                    }
                })
                .unwrap_or_default(),
            addons: config.addons.get("all").copied().unwrap_or_default().union(
                config
                    .addons
                    .get(&config.layout)
                    .copied()
                    .unwrap_or_default(),
            ),
            hanja_state: None,
            state: HangulState::new(config.word_commit),
        }
    }

    pub fn enable_hanja_mode(&mut self) -> bool {
        let hangul = self.state.get_hanja_char();
        self.state.reset();

        if hangul != '\0' {
            self.hanja_state = HanjaState::new(hangul);
        }

        self.hanja_state.is_some()
    }

    fn try_hanja_press_key(&mut self, key: Key, commit_buf: &mut String) -> bool {
        if let Some(ref mut hanja_state) = self.hanja_state {
            if hanja_state.press_key(key) {
                true
            } else {
                commit_buf.push(hanja_state.current_hanja());
                self.hanja_state = None;
                false
            }
        } else {
            false
        }
    }
}

impl InputEngineBackend for HangulEngine {
    fn press_key(&mut self, key: Key, commit_buf: &mut String) -> bool {
        if key.code == KeyCode::Backspace {
            if self.hanja_state.take().is_some() {
                true
            } else {
                self.state.backspace(self.addons, commit_buf)
            }
        } else if self.try_hanja_press_key(key, commit_buf) {
            true
        } else if let Some(kv) = self.layout.lookup_kv(&key) {
            self.state.key(kv, self.addons, commit_buf)
        } else {
            false
        }
    }

    #[inline]
    fn clear_preedit(&mut self, commit_buf: &mut String) {
        if let Some(hanja) = self.hanja_state.take() {
            commit_buf.push(hanja.current_hanja());
        } else {
            self.state.clear_preedit(commit_buf);
        }
    }

    #[inline]
    fn reset(&mut self) {
        self.hanja_state = None;
        self.state.reset();
    }

    #[inline]
    fn has_preedit(&self) -> bool {
        self.hanja_state.is_some() || self.state.has_preedit()
    }

    fn preedit_str(&self, buf: &mut String) {
        if let Some(hanja) = self.hanja_state.as_ref() {
            hanja.preedit_str(buf);
        } else {
            self.state.preedit_str(buf);
        }
    }
}

pub fn builtin_layouts() -> impl Iterator<Item = (Cow<'static, str>, Layout)> {
    BUILTIN_LAYOUTS
        .iter()
        .copied()
        .filter_map(|(name, layout)| Layout::load_from(layout).ok().map(|l| (name.into(), l)))
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
