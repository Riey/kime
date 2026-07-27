use crate::characters::KeyValue;
use crate::Key;
use kime_engine_backend::KeyMap;
use serde::Deserialize;
use std::collections::HashMap;

/// The newest layout file format version this version of kime understands.
///
/// Version 1 is the versioned form of the original flat-map format:
///
/// ```yaml
/// version: 1
/// keys:
///   Q: ㅂ
/// ```
///
/// Files without a `version` field keep being parsed as the legacy flat map.
pub const LAYOUT_FORMAT_VERSION: u32 = 1;

#[derive(Debug)]
pub enum LayoutError {
    /// The file is not valid YAML or doesn't match the layout schema.
    Parse(serde_yaml::Error),
    /// The file declares a format version newer than this kime supports.
    UnsupportedVersion { version: u32 },
}

impl std::fmt::Display for LayoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(err) => err.fmt(f),
            Self::UnsupportedVersion { version } => write!(
                f,
                "layout format version {} is not supported, this version of kime supports up to version {}",
                version, LAYOUT_FORMAT_VERSION
            ),
        }
    }
}

impl std::error::Error for LayoutError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Parse(err) => Some(err),
            Self::UnsupportedVersion { .. } => None,
        }
    }
}

impl From<serde_yaml::Error> for LayoutError {
    fn from(err: serde_yaml::Error) -> Self {
        Self::Parse(err)
    }
}

/// Probe that only extracts the `version` field, ignoring everything else,
/// so the version can be checked before the rest of the file is parsed.
#[derive(Deserialize)]
struct VersionProbe {
    #[serde(default)]
    version: Option<u32>,
}

impl VersionProbe {
    /// The format version this file is written in.
    ///
    /// A file without a `version` field is format version 1: the original
    /// flat-map format predates the field, so its absence is semantically
    /// identical to `version: 1` (not merely "legacy accepted").
    fn format_version(&self) -> u32 {
        self.version.unwrap_or(1)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VersionedLayout {
    #[allow(dead_code)]
    version: u32,
    keys: HashMap<Key, String>,
}

#[derive(Clone, Default)]
pub struct Layout {
    keymap: KeyMap<KeyValue>,
}

impl Layout {
    pub fn from_items(items: HashMap<Key, String>) -> Self {
        let mut keymap = KeyMap::default();

        for (key, value) in items {
            let value = match value.parse::<KeyValue>() {
                Ok(value) => value,
                Err(_) => continue,
            };

            keymap.insert(key, value);
        }

        Self { keymap }
    }

    pub fn load_from(content: &str) -> Result<Self, LayoutError> {
        // Two-step parse so each shape reports its own error instead of the
        // vague "did not match any variant" from an untagged enum. The probe
        // also checks the version before the rest of the file is parsed: a
        // future format may change the schema, and the unsupported version
        // error is more useful than a schema mismatch.
        let probe: VersionProbe = serde_yaml::from_str(content)?;
        let version = probe.format_version();

        if version > LAYOUT_FORMAT_VERSION {
            return Err(LayoutError::UnsupportedVersion { version });
        }

        let items: HashMap<Key, String> = if probe.version.is_some() {
            serde_yaml::from_str::<VersionedLayout>(content)?.keys
        } else {
            // Legacy shape: a flat `Key: value` map without a version field,
            // read as format version 1.
            serde_yaml::from_str(content)?
        };

        Ok(Self::from_items(items))
    }

    #[inline]
    pub fn lookup_kv(&self, key: Key) -> Option<KeyValue> {
        self.keymap.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kime_engine_backend::KeyCode;

    // `1: 1` covers keys and values that YAML would otherwise read as
    // numbers instead of strings (cf. the number row in dubeolsik.yaml).
    const FLAT: &str = "
Q: ㅂ$ㅂ
S-Q: ㅃ
W: ㅈ
1: 1
";

    const VERSIONED: &str = "
version: 1
keys:
  Q: ㅂ$ㅂ
  S-Q: ㅃ
  W: ㅈ
  1: 1
";

    const KEYS: [Key; 4] = [
        Key::normal(KeyCode::Q),
        Key::shift(KeyCode::Q),
        Key::normal(KeyCode::W),
        Key::normal(KeyCode::One),
    ];

    #[test]
    fn flat_legacy_layout_parses() {
        let layout = Layout::load_from(FLAT).expect("flat legacy layout must keep parsing");

        for key in KEYS {
            assert!(layout.lookup_kv(key).is_some(), "missing key {key}");
        }
    }

    #[test]
    fn versioned_layout_parses_like_flat() {
        let flat = Layout::load_from(FLAT).expect("flat layout parses");
        let versioned = Layout::load_from(VERSIONED).expect("versioned layout parses");

        for key in KEYS {
            assert_eq!(
                flat.lookup_kv(key),
                versioned.lookup_kv(key),
                "mismatch for key {key}"
            );
        }
    }

    #[test]
    fn missing_version_is_treated_as_version_1() {
        // The defaulting decision itself: a file without a `version` field
        // resolves to format version 1, not merely "legacy accepted".
        let probe: VersionProbe = serde_yaml::from_str(FLAT).expect("flat layout probes");
        assert_eq!(probe.version, None);
        assert_eq!(probe.format_version(), 1);

        let probe: VersionProbe = serde_yaml::from_str(VERSIONED).expect("versioned layout probes");
        assert_eq!(probe.version, Some(1));
        assert_eq!(probe.format_version(), 1);

        // And end to end: the same key set with and without `version: 1`
        // produces identical layouts.
        let flat = Layout::load_from(FLAT).expect("flat layout parses");
        let versioned = Layout::load_from(VERSIONED).expect("versioned layout parses");

        for key in KEYS {
            assert_eq!(
                flat.lookup_kv(key),
                versioned.lookup_kv(key),
                "mismatch for key {key}"
            );
        }
    }

    #[test]
    fn future_version_is_rejected() {
        let err = Layout::load_from("version: 99\nkeys:\n  Q: ㅂ\n")
            .err()
            .expect("future format version must be rejected");

        assert!(
            matches!(err, LayoutError::UnsupportedVersion { version: 99 }),
            "unexpected error: {err:?}"
        );

        let msg = err.to_string();
        assert!(
            msg.contains("99"),
            "error must name the file version: {msg}"
        );
        assert!(
            msg.contains(&LAYOUT_FORMAT_VERSION.to_string()),
            "error must name the supported version: {msg}"
        );
    }

    #[test]
    fn builtin_layouts_all_parse() {
        for (name, content) in crate::BUILTIN_LAYOUTS {
            Layout::load_from(content).unwrap_or_else(|err| panic!("{name}: {err}"));
        }
    }
}
