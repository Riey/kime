use crate::{keycode::Key, KeyCode, Layout, ModifierState};
use ahash::AHashMap;
use enum_map::{enum_map, Enum, EnumMap};
use enumset::{EnumSet, EnumSetType};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

mod serde_enummap_default {
    use super::InputCategory;
    use enum_map::EnumMap;
    use serde::{
        de::{MapAccess, Visitor},
        ser::SerializeMap,
        Deserializer, Serializer,
    };

    type My = EnumMap<InputCategory, String>;

    pub fn serialize<S: Serializer>(map: &My, serializer: S) -> Result<S::Ok, S::Error> {
        let mut ser_map = serializer.serialize_map(Some(2))?;

        for (k, v) in map {
            ser_map.serialize_entry(&k, &v)?;
        }

        ser_map.end()
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<My, D::Error> {
        deserializer.deserialize_map(MapVisitor)
    }

    struct MapVisitor;

    impl<'de> Visitor<'de> for MapVisitor {
        type Value = My;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("InputCategory map")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut ret = enum_map::enum_map! {
                _ => "direct".into(),
            };

            while let Some((k, v)) = map.next_entry()? {
                ret[k] = v;
            }

            Ok(ret)
        }
    }
}

#[derive(Hash, Serialize, Deserialize, EnumSetType)]
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

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Enum)]
#[repr(u32)]
pub enum InputCategory {
    Latin,
    Hangul,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum HotkeyBehavior {
    Toggle(InputCategory, InputCategory),
    Switch(InputCategory),
    Commit,
    Emoji,
    Hanja,
}

impl HotkeyBehavior {
    pub const fn toggle_hangul_latin() -> Self {
        Self::Toggle(InputCategory::Hangul, InputCategory::Latin)
    }

    pub const fn switch(category: InputCategory) -> Self {
        Self::Switch(category)
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum HotkeyResult {
    Consume,
    Bypass,
    ConsumeIfProcessed,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Hotkey {
    behavior: HotkeyBehavior,
    result: HotkeyResult,
}

impl Hotkey {
    pub const fn new(behavior: HotkeyBehavior, result: HotkeyResult) -> Self {
        Self { behavior, result }
    }

    pub const fn behavior(self) -> HotkeyBehavior {
        self.behavior
    }
    pub const fn result(self) -> HotkeyResult {
        self.result
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum IconColor {
    White,
    Black,
}

impl Default for IconColor {
    fn default() -> Self {
        Self::Black
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct RawConfig {
    pub default_category: InputCategory,
    #[serde(with = "self::serde_enummap_default")]
    pub category_layout: EnumMap<InputCategory, String>,
    pub global_category_state: bool,
    pub word_commit: bool,
    pub icon_color: IconColor,
    pub hotkeys: BTreeMap<Key, Hotkey>,
    pub layout_addons: BTreeMap<String, EnumSet<Addon>>,
    pub xim_preedit_font: (String, f64),
}

impl Default for RawConfig {
    fn default() -> Self {
        Self {
            default_category: InputCategory::Latin,
            global_category_state: false,
            word_commit: false,
            icon_color: IconColor::default(),
            hotkeys: [
                (
                    Key::normal(KeyCode::Esc),
                    Hotkey::new(
                        HotkeyBehavior::switch(InputCategory::Latin),
                        HotkeyResult::Bypass,
                    ),
                ),
                (
                    Key::normal(KeyCode::AltR),
                    Hotkey::new(HotkeyBehavior::toggle_hangul_latin(), HotkeyResult::Consume),
                ),
                (
                    Key::normal(KeyCode::Muhenkan),
                    Hotkey::new(HotkeyBehavior::toggle_hangul_latin(), HotkeyResult::Consume),
                ),
                (
                    Key::normal(KeyCode::Hangul),
                    Hotkey::new(HotkeyBehavior::toggle_hangul_latin(), HotkeyResult::Consume),
                ),
                (
                    Key::super_(KeyCode::Space),
                    Hotkey::new(HotkeyBehavior::toggle_hangul_latin(), HotkeyResult::Consume),
                ),
                (
                    Key::normal(KeyCode::F9),
                    Hotkey::new(HotkeyBehavior::Hanja, HotkeyResult::Consume),
                ),
                (
                    Key::new(KeyCode::E, ModifierState::CONTROL | ModifierState::ALT),
                    Hotkey::new(HotkeyBehavior::Emoji, HotkeyResult::ConsumeIfProcessed),
                ),
                (
                    Key::normal(KeyCode::ControlR),
                    Hotkey::new(HotkeyBehavior::Hanja, HotkeyResult::Consume),
                ),
                (
                    Key::normal(KeyCode::HangulHanja),
                    Hotkey::new(HotkeyBehavior::Hanja, HotkeyResult::Consume),
                ),
            ]
            .iter()
            .copied()
            .collect(),
            category_layout: enum_map! {
                InputCategory::Latin => "direct".into(),
                InputCategory::Hangul => "dubeolsik".into(),
            },
            layout_addons: vec![
                ("all".into(), EnumSet::only(Addon::ComposeChoseongSsang)),
                (
                    "dubeolsik".into(),
                    EnumSet::only(Addon::TreatJongseongAsChoseong),
                ),
            ]
            .into_iter()
            .collect(),
            xim_preedit_font: ("D2Coding".to_string(), 15.0),
        }
    }
}

pub struct Config {
    pub layouts: EnumMap<InputCategory, Layout>,
    pub global_category_state: bool,
    pub hotkeys: AHashMap<Key, Hotkey>,
    pub default_category: InputCategory,
    pub category_layout: EnumMap<InputCategory, String>,
    pub layout_addons: EnumMap<InputCategory, EnumSet<Addon>>,
    pub icon_color: IconColor,
    pub word_commit: bool,
    pub xim_preedit_font: (String, f64),
}

impl Default for Config {
    fn default() -> Self {
        Self::from_raw_config(RawConfig::default())
    }
}

pub const BUILTIN_LAYOUTS: &'static [(&'static str, &'static str)] = &[
    ("direct", include_str!("../data/direct.yaml")),
    ("qwerty", include_str!("../data/qwerty.yaml")),
    ("colemak", include_str!("../data/colemak.yaml")),
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

fn builtin_layouts() -> impl Iterator<Item = (String, Layout)> {
    BUILTIN_LAYOUTS
        .iter()
        .copied()
        .filter_map(|(name, layout)| {
            Layout::load_from(layout)
                .ok()
                .map(|l| (name.to_string(), l))
        })
}

impl Config {
    pub fn new(layouts: EnumMap<InputCategory, Layout>, raw: RawConfig) -> Self {
        let all_addons = raw.layout_addons.get("all").copied().unwrap_or_default();

        Self {
            global_category_state: raw.global_category_state,
            default_category: raw.default_category,
            layouts,
            word_commit: raw.word_commit,
            icon_color: raw.icon_color,
            layout_addons: (|category| {
                let name = &raw.category_layout[category];
                all_addons.union(raw.layout_addons.get(name).copied().unwrap_or_default())
            })
            .into(),
            category_layout: raw.category_layout,
            hotkeys: raw.hotkeys.into_iter().collect(),
            xim_preedit_font: raw.xim_preedit_font,
        }
    }

    pub fn from_layout_map(mut layouts: AHashMap<String, Layout>, raw: RawConfig) -> Self {
        Self::new(
            (|category| {
                let name = raw.category_layout[category].as_str();
                layouts.remove(name).unwrap_or_default()
            })
            .into(),
            raw,
        )
    }

    pub fn from_raw_config(raw: RawConfig) -> Self {
        Self::from_layout_map(builtin_layouts().collect(), raw)
    }

    #[cfg(unix)]
    pub fn from_raw_config_with_dir(raw: RawConfig, dir: xdg::BaseDirectories) -> Self {
        let custom_layouts = dir
            .list_config_files("layouts")
            .into_iter()
            .filter_map(|path| {
                let name = path.file_stem()?.to_str()?;

                Layout::load_from(std::fs::read_to_string(&path).ok()?.as_str())
                    .ok()
                    .map(|l| (name.to_string(), l))
            });

        Self::from_layout_map(builtin_layouts().chain(custom_layouts).collect(), raw)
    }

    #[cfg(unix)]
    pub fn load_from_config_dir() -> Option<Self> {
        let dir = xdg::BaseDirectories::with_prefix("kime").ok()?;

        let raw = dir
            .find_config_file("config.yaml")
            .and_then(|config| serde_yaml::from_reader(std::fs::File::open(config).ok()?).ok())
            .unwrap_or_default();

        Some(Self::from_raw_config_with_dir(raw, dir))
    }

    pub fn word_commit(&self) -> bool {
        self.word_commit
    }
}
