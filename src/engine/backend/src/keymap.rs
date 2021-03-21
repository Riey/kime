use crate::{Key, KeyCode, ModifierState};
use serde::{
    de::{MapAccess, Visitor},
    Deserialize,
};
use std::{
    fmt,
    iter::{FromIterator, IntoIterator},
    marker::PhantomData,
};
use enum_map::EnumMap;
use strum::EnumCount;

const KEYMAP_SIZE: usize = KeyCode::COUNT * 2;

#[inline]
const fn key_to_idx(key: Key) -> usize {
    (key.code as u32 + (KeyCode::COUNT as u32) * key.state.bits()) as usize
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyMap<V> {
    arr: EnumMap<KeyCode, [Option<V>; 2]>,
    // arr: [Option<V>; KEYMAP_SIZE],
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
        // Self { arr: [None; KEYMAP_SIZE] }
    }

    pub fn get(&self, key: Key) -> Option<V> {
        if key.state.intersects(!ModifierState::SHIFT) {
            None
        } else {
            self.arr[key.code][key.state.bits() as usize]
        }

        // debug_assert!(key_to_idx(key) < self.arr.len());
        // unsafe { *self.arr.get_unchecked(key_to_idx(key)) }
    }

    /// Key must don't have shift modifier
    pub fn insert(&mut self, key: Key, value: V) {
        self.arr[key.code][key.state.bits() as usize] = Some(value);
        // assert!(!key.state.intersects(!ModifierState::SHIFT));
        // debug_assert!(key_to_idx(key) < self.arr.len());
        // unsafe {
        //     *self.arr.get_unchecked_mut(key_to_idx(key)) = Some(value);
        // }
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
    use super::{Key, KeyCode, KeyMap};

    #[test]
    fn insert() {
        let mut map = KeyMap::new();
        map.insert(Key::normal(KeyCode::Backspace), 123);
        assert_eq!(map.get(Key::normal(KeyCode::Backspace)), Some(123));
    }
}
