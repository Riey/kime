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
use strum::EnumCount;

const MAP_SIZE: usize = KeyCode::COUNT * (ModifierState::all().bits() as usize + 1);

#[inline]
const fn key_to_idx(key: Key) -> usize {
    (key.code as u32 + (key.state.bits() * KeyCode::COUNT as u32)) as usize
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyMap<V> {
    arr: [Option<V>; MAP_SIZE],
}

impl<V: Copy> Default for KeyMap<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: Copy> KeyMap<V> {
    pub fn new() -> Self {
        Self {
            arr: [None; MAP_SIZE],
        }
    }

    pub fn get(&self, key: Key) -> Option<V> {
        unsafe { *self.arr.get_unchecked(key_to_idx(key)) }
    }

    pub fn insert(&mut self, key: Key, value: V) {
        unsafe {
            *self.arr.get_unchecked_mut(key_to_idx(key)) = Some(value);
        }
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
