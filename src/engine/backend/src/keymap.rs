// arraymap is super fast but take more memory (over 1MB)
#[cfg(feature = "array-keymap")]
mod arraymap {
    use crate::{Key, KeyCode, ModifierState};
    use serde::{
        de::{MapAccess, Visitor},
        Deserialize,
    };
    use std::{
        fmt,
        iter::{FromIterator, IntoIterator},
        marker::PhantomData,
        mem,
    };
    use strum::EnumCount;

    const KEYMAP_SIZE: usize = KeyCode::COUNT * (ModifierState::all().bits() as usize + 1);

    const fn key_to_idx(key: &Key) -> usize {
        (key.code as u32 + (KeyCode::COUNT as u32) * key.state.bits()) as usize
    }

    // fn idx_to_key(idx: usize) -> Key {
    //     let modifier = idx / KeyCode::COUNT;
    //     let code = idx % KeyCode::COUNT;
    //     Key::new(unsafe { mem::transmute(code as u32) }, ModifierState::from_bits_truncate(modifier as u32))
    // }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct KeyMap<V> {
        arr: [Option<V>; KEYMAP_SIZE],
    }

    impl<V: Copy> KeyMap<V> {
        pub fn new_copy() -> Self {
            Self {
                arr: [None; KEYMAP_SIZE],
            }
        }
    }

    impl<V> Default for KeyMap<V> {
        fn default() -> Self {
            Self::new()
        }
    }

    impl<V> KeyMap<V> {
        pub fn new() -> Self {
            let arr = unsafe {
                let mut arr: [mem::MaybeUninit<Option<V>>; KEYMAP_SIZE] =
                    mem::MaybeUninit::uninit().assume_init();
                for elem in &mut arr {
                    *elem = mem::MaybeUninit::new(None);
                }
                mem::transmute_copy(&arr)
            };
            Self { arr }
        }

        pub fn get(&self, key: &Key) -> Option<&V> {
            unsafe { self.arr.get_unchecked(key_to_idx(key)).as_ref() }
        }
        pub fn insert(&mut self, key: Key, value: V) {
            unsafe {
                *self.arr.get_unchecked_mut(key_to_idx(&key)) = Some(value);
            }
        }
    }

    impl<V> FromIterator<(Key, V)> for KeyMap<V> {
        fn from_iter<T: IntoIterator<Item = (Key, V)>>(iter: T) -> Self {
            let mut map = Self::new();
            for item in iter {
                map.insert(item.0, item.1);
            }
            map
        }
    }

    struct KeyMapVisitor<V>(PhantomData<V>);

    impl<'de, V> Visitor<'de> for KeyMapVisitor<V>
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

    impl<'de, V> Deserialize<'de> for KeyMap<V>
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
            let mut map = KeyMap::new_copy();
            map.insert(Key::normal(KeyCode::Backspace), 123);
            assert_eq!(map.get(Key::normal(KeyCode::Backspace)), &Some(123));
        }
    }
}

#[cfg(feature = "btreemap-keymap")]
mod btreemap {
    use crate::Key;
    pub type KeyMap<V> = std::collections::BTreeMap<Key, V>;
}

#[cfg(all(not(feature = "array-keymap"), not(feature = "btreemap-keymap"),))]
mod hashmap {
    use crate::Key;
    pub type KeyMap<V> = ahash::AHashMap<Key, V>;
}

#[cfg(feature = "array-keymap")]
pub use arraymap::KeyMap;

#[cfg(feature = "btreemap-keymap")]
pub use btreemap::KeyMap;

#[cfg(all(not(feature = "array-keymap"), not(feature = "btreemap-keymap"),))]
pub use hashmap::KeyMap;
