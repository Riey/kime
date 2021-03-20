use crate::characters::KeyValue;
use crate::Key;
use kime_engine_backend::{AHashMap, KeyMap};

#[derive(Clone, Default)]
pub struct Layout {
    keymap: KeyMap<KeyValue>,
}

impl Layout {
    fn from_items(items: AHashMap<Key, String>) -> Self {
        let mut keymap = KeyMap::new();

        for (key, value) in items {
            let value = match value.parse::<KeyValue>() {
                Ok(value) => value,
                Err(_) => continue,
            };

            keymap.insert(key, value);
        }

        Self { keymap }
    }

    pub fn load_from(content: &str) -> Result<Self, serde_yaml::Error> {
        Ok(Self::from_items(serde_yaml::from_str(content)?))
    }

    pub fn lookup_kv(&self, key: Key) -> &Option<KeyValue> {
        self.keymap.get(key)
    }
}
