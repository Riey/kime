use kime_engine_backend::{InputEngineBackend, Key, KeyMap};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum LatinLayout {
    Qwerty,
    Dvorak,
    Colemak,
}

impl Default for LatinLayout {
    fn default() -> Self {
        Self::Qwerty
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct LatinConfig {
    pub layout: LatinLayout,
    pub preferred_direct: bool,
}

impl Default for LatinConfig {
    fn default() -> Self {
        Self {
            layout: LatinLayout::Qwerty,
            preferred_direct: true,
        }
    }
}

impl LatinConfig {
    /// Returns `true` when the configured `layout` has no effect.
    ///
    /// With `preferred_direct` enabled kime passes key events through to the
    /// OS/firmware layout instead of applying the embedded latin layout, so a
    /// non-`Qwerty` `layout` (e.g. `Dvorak`, `Colemak`) is silently ignored.
    /// `Qwerty` is excluded because its mapping matches a plain pass-through.
    /// See <https://github.com/Riey/kime/issues/626>.
    pub fn layout_ignored(&self) -> bool {
        self.preferred_direct && !matches!(self.layout, LatinLayout::Qwerty)
    }
}

pub struct LatinData {
    keymap: KeyMap<char>,
}

impl LatinData {
    pub fn new(config: &LatinConfig) -> Self {
        if config.layout_ignored() {
            log::warn!(
                "latin.layout is set to a non-Qwerty layout but latin.preferred_direct is true, \
                 so the layout is ignored and key events are passed through to the OS layout. \
                 Set latin.preferred_direct to false to use the embedded latin layout."
            );
        }

        Self {
            keymap: load_layout(config),
        }
    }

    #[inline]
    pub fn lookup(&self, key: Key) -> Option<char> {
        self.keymap.get(key)
    }
}

fn load_layout(config: &LatinConfig) -> KeyMap<char> {
    let layout = match config.layout {
        LatinLayout::Qwerty => include_str!("../data/qwerty.yaml"),
        LatinLayout::Dvorak => include_str!("../data/dvorak.yaml"),
        LatinLayout::Colemak => include_str!("../data/colemak.yaml"),
    };
    serde_yaml::from_str(layout).unwrap_or_default()
}

#[derive(Clone)]
pub struct LatinEngine {
    preferred_direct: bool,
}

impl LatinEngine {
    pub fn new(preferred_direct: bool) -> Self {
        Self { preferred_direct }
    }
}

impl InputEngineBackend for LatinEngine {
    type ConfigData = LatinData;

    fn press_key(&mut self, config: &LatinData, key: Key, commit_buf: &mut String) -> bool {
        if self.preferred_direct {
            false
        } else {
            if let Some(ch) = config.lookup(key) {
                commit_buf.push(ch);
                true
            } else {
                false
            }
        }
    }

    fn clear_preedit(&mut self, _commit_buf: &mut String) {}
    fn reset(&mut self) {}

    fn has_preedit(&self) -> bool {
        false
    }

    fn preedit_str(&self, _buf: &mut String) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use kime_engine_backend::KeyCode;

    #[test]
    fn embedded_dvorak_layout_is_loaded() {
        // The embedded Dvorak layout remaps the physical `Q` key to `'`.
        let data = LatinData::new(&LatinConfig {
            layout: LatinLayout::Dvorak,
            preferred_direct: false,
        });
        assert_eq!(data.lookup(Key::normal(KeyCode::Q)), Some('\''));
        // Regression guard: these keys used invalid YAML backslash escapes
        // (`\,`, `\<`, `\.`, `\>`, `\;`) which made the whole layout fail to
        // parse and silently fall back to an empty keymap (#626).
        assert_eq!(data.lookup(Key::normal(KeyCode::W)), Some(','));
        assert_eq!(data.lookup(Key::shift(KeyCode::W)), Some('<'));
        assert_eq!(data.lookup(Key::normal(KeyCode::E)), Some('.'));
        assert_eq!(data.lookup(Key::shift(KeyCode::E)), Some('>'));
        assert_eq!(data.lookup(Key::shift(KeyCode::Z)), Some(';'));
        // And an ordinary letter mapping still works.
        assert_eq!(data.lookup(Key::normal(KeyCode::K)), Some('t'));
    }

    #[test]
    fn embedded_layouts_parse_completely() {
        // A malformed entry makes serde fall back to an empty map via
        // `unwrap_or_default()`, so a full keymap proves the file parsed.
        for layout in [
            LatinLayout::Qwerty,
            LatinLayout::Dvorak,
            LatinLayout::Colemak,
        ] {
            let data = LatinData::new(&LatinConfig {
                layout,
                preferred_direct: false,
            });
            assert!(
                data.lookup(Key::normal(KeyCode::A)).is_some()
                    && data.lookup(Key::normal(KeyCode::Backslash)).is_some(),
                "embedded layout failed to parse and fell back to empty keymap",
            );
        }
    }

    #[test]
    fn layout_ignored_detects_conflicting_config() {
        // Non-Qwerty layout + preferred_direct => layout silently ignored (#626).
        assert!(LatinConfig {
            layout: LatinLayout::Dvorak,
            preferred_direct: true,
        }
        .layout_ignored());
        assert!(LatinConfig {
            layout: LatinLayout::Colemak,
            preferred_direct: true,
        }
        .layout_ignored());

        // Qwerty + preferred_direct is fine: pass-through yields the same chars.
        assert!(!LatinConfig {
            layout: LatinLayout::Qwerty,
            preferred_direct: true,
        }
        .layout_ignored());

        // Non-Qwerty without preferred_direct actually applies the layout.
        assert!(!LatinConfig {
            layout: LatinLayout::Dvorak,
            preferred_direct: false,
        }
        .layout_ignored());

        // The default config (Qwerty + preferred_direct) must never warn.
        assert!(!LatinConfig::default().layout_ignored());
    }

    #[test]
    fn preferred_direct_bypasses_dvorak_layout() {
        // Reproduces #626: with preferred_direct the Dvorak mapping never applies;
        // the key is not consumed and nothing is committed.
        let data = LatinData::new(&LatinConfig {
            layout: LatinLayout::Dvorak,
            preferred_direct: true,
        });
        let mut engine = LatinEngine::new(true);
        let mut buf = String::new();
        let consumed = engine.press_key(&data, Key::normal(KeyCode::Q), &mut buf);
        assert!(!consumed);
        assert!(buf.is_empty());

        // Without preferred_direct the same key commits the Dvorak char.
        let mut engine = LatinEngine::new(false);
        let mut buf = String::new();
        let consumed = engine.press_key(&data, Key::normal(KeyCode::Q), &mut buf);
        assert!(consumed);
        assert_eq!(buf, "'");
    }
}
