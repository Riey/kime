use crate::{Key, KeyCode, ModifierState};
use enum_map::EnumMap;
use serde::{
    de::{MapAccess, Visitor},
    Deserialize,
};
use std::{
    fmt,
    iter::{FromIterator, IntoIterator},
    marker::PhantomData,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyMap<V> {
    arr: EnumMap<KeyCode, [Option<V>; 2]>,
}

impl<V: Copy> Default for KeyMap<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: Copy> KeyMap<V> {
    pub fn new() -> Self {
        Self {
            arr: EnumMap::default(),
        }
    }

    pub fn get(&self, key: Key) -> Option<V> {
        if key.state.intersects(!ModifierState::SHIFT) {
            None
        } else {
            // SAFETY: key.state <= 0x1
            unsafe { *self.arr[key.code].get_unchecked(key.state.bits() as usize) }
        }
    }

    /// Store `value` for `key`, ignoring keys this map cannot hold.
    ///
    /// Only the unmodified and Shift variants of a keycode get a slot, and
    /// `get` returns `None` for anything else — its `get_unchecked` is
    /// sound only because of that. A key carrying another modifier is
    /// therefore unreachable whatever we do with it here, so it is
    /// dropped: layout and translation-layer files are user-authored, and
    /// naming such a key must not kill kime.
    pub fn insert(&mut self, key: Key, value: V) {
        if key.state.intersects(!ModifierState::SHIFT) {
            return;
        }

        self.arr[key.code][key.state.bits() as usize] = Some(value);
    }
}

impl<V: Copy> FromIterator<(Key, V)> for KeyMap<V> {
    fn from_iter<T: IntoIterator<Item = (Key, V)>>(iter: T) -> Self {
        let mut map = Self::new();
        for item in iter {
            map.insert(item.0, item.1);
        }
        map
    }
}

struct KeyMapVisitor<V>(PhantomData<V>);

impl<'de, V: Copy> Visitor<'de> for KeyMapVisitor<V>
where
    V: Deserialize<'de>,
{
    type Value = KeyMap<V>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("KeyMap")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut ret = KeyMap::new();

        while let Some(entry) = map.next_entry()? {
            ret.insert(entry.0, entry.1);
        }

        Ok(ret)
    }
}

impl<'de, V: Copy> Deserialize<'de> for KeyMap<V>
where
    V: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(KeyMapVisitor(PhantomData))
    }
}

#[cfg(test)]
mod tests {
    use super::{Key, KeyCode, KeyMap, ModifierState};

    #[test]
    fn insert() {
        let mut map = KeyMap::new();
        map.insert(Key::normal(KeyCode::Backspace), 123);
        assert_eq!(map.get(Key::normal(KeyCode::Backspace)), Some(123));
    }

    #[test]
    fn insert_shift() {
        let mut map = KeyMap::new();
        map.insert(Key::shift(KeyCode::Q), 123);
        assert_eq!(map.get(Key::shift(KeyCode::Q)), Some(123));
        assert_eq!(map.get(Key::normal(KeyCode::Q)), None);
    }

    /// A key with a modifier other than Shift has no slot: `get` rejects
    /// it, so `insert` must drop it rather than index past the two it
    /// keeps. User-authored layout and translation-layer files can name
    /// such keys, and a panic there kills kime on a config typo (#793).
    #[test]
    fn insert_ignores_non_shift_modifiers() {
        for state in [
            ModifierState::CONTROL,
            ModifierState::ALT,
            ModifierState::SUPER,
            ModifierState::CONTROL | ModifierState::SHIFT,
            ModifierState::CONTROL | ModifierState::ALT | ModifierState::SUPER,
        ] {
            let mut map = KeyMap::new();
            let key = Key::new(KeyCode::T, state);
            map.insert(key, 123);
            assert_eq!(map.get(key), None, "{key} must not be stored");
        }
    }
}
