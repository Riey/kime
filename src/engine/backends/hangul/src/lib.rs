mod characters;
mod layout;
mod state;

use std::{borrow::Cow, collections::BTreeMap};

use enumset::{EnumSet, EnumSetType};
use kime_engine_backend::{InputEngineBackend, Key, KeyCode};
use serde::{Deserialize, Serialize};

pub use layout::Layout;
pub use state::HangulEngine;

#[derive(Hash, Serialize, Deserialize, Debug, EnumSetType)]
#[enumset(serialize_repr = "list")]
pub enum Addon {
    ComposeChoseongSsang,
    ComposeJungseongSsang,
    ComposeJongseongSsang,
    DecomposeChoseongSsang,
    DecomposeJungseongSsang,
    DecomposeJongseongSsang,

    /// ㅏ + ㄱ = 가
    /// ㄱ + $ㄱ + ㅏ = 각
    FlexibleComposeOrder,

    /// 안 + ㅣ = 아니
    TreatJongseongAsChoseong,
    /// 읅 + ㄱ = 을ㄲ
    TreatJongseongAsChoseongCompose,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PreeditJohabLevel {
    /// Always use johab encoding
    Always,
    /// Use johab encoding when it's needed
    Needed,
    /// Never use johab encoding
    Never,
}

impl Default for PreeditJohabLevel {
    fn default() -> Self {
        PreeditJohabLevel::Needed
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct HangulConfig {
    pub layout: String,
    pub word_commit: bool,
    pub preedit_johab: PreeditJohabLevel,
    pub addons: BTreeMap<String, EnumSet<Addon>>,
}

impl Default for HangulConfig {
    fn default() -> Self {
        Self {
            layout: "dubeolsik".into(),
            word_commit: false,
            preedit_johab: PreeditJohabLevel::default(),
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

pub struct HangulData {
    layout: Layout,
    addons: EnumSet<Addon>,
    preedit_johab: PreeditJohabLevel,
    word_commit: bool,
}

impl Default for HangulData {
    fn default() -> Self {
        Self::new(&HangulConfig::default(), builtin_layouts())
    }
}

impl HangulData {
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
            preedit_johab: config.preedit_johab,
            word_commit: config.word_commit,
        }
    }

    pub const fn preedit_johab(&self) -> PreeditJohabLevel {
        self.preedit_johab
    }

    pub const fn word_commit(&self) -> bool {
        self.word_commit
    }
}

impl InputEngineBackend for HangulEngine {
    type ConfigData = HangulData;

    fn press_key(&mut self, config: &HangulData, key: Key, commit_buf: &mut String) -> bool {
        if key.code == KeyCode::Backspace {
            self.backspace(config.addons, commit_buf)
        } else if let Some(kv) = config.layout.lookup_kv(key) {
            self.key(kv, config.addons, commit_buf)
        } else {
            false
        }
    }

    #[inline]
    fn clear_preedit(&mut self, commit_buf: &mut String) {
        self.clear_preedit(commit_buf);
    }

    #[inline]
    fn reset(&mut self) {
        self.reset();
    }

    #[inline]
    fn has_preedit(&self) -> bool {
        self.has_preedit()
    }

    fn preedit_str(&self, buf: &mut String) {
        self.preedit_str(buf);
    }
}

pub fn builtin_layouts() -> impl Iterator<Item = (Cow<'static, str>, Layout)> {
    BUILTIN_LAYOUTS
        .iter()
        .copied()
        .filter_map(|(name, layout)| Layout::load_from(layout).ok().map(|l| (name.into(), l)))
}
