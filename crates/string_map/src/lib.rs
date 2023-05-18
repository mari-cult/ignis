use std::{collections::BTreeMap, fmt};

pub(crate) use crate::key::Key;

pub use crate::{
    fst_iter::{StartsWith, SubSequence},
    iter::{IntoIter, IntoKeys, IntoValues, Iter, IterMut, Keys, Values, ValuesMut},
};

mod fst_iter;
mod key;

pub(crate) mod iter;

/// A lexicographically ordered string-key map.
///
/// Essentially just a [`BTreeMap`](BTreeMap) with an associated
/// [finite state automata](fst::Set).
pub struct StringMap<V> {
    pub(crate) map: BTreeMap<Key, V>,
    pub(crate) fst: fst::Set<Vec<u8>>,
}

impl<V> StringMap<V> {
    /// Create an empty `StringMap`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
            fst: fst::Set::default(),
        }
    }

    /// Returns the number of elements in the map.
    #[inline]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns `true` if the map contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Update internal finite state automata.
    fn update_fst(&mut self) {
        let fst = fst::Set::from_iter(self.map.keys());

        self.fst = unsafe { fst.unwrap_unchecked() };
    }

    /// Insert a key-value pair into the map.
    ///
    /// If the map did not have this key present, None is returned.
    ///
    /// If the map did have this key present, the value is updated,
    /// and the old value is returned. The key is not updated.
    pub fn insert<K>(&mut self, key: K, value: V) -> Option<V>
    where
        K: AsRef<str>,
    {
        let value = self.map.insert(Key::from(key.as_ref()), value);

        self.update_fst();

        value
    }

    #[inline]
    pub fn starts_with<'a>(&'a self, prefix: &'a str) -> StartsWith<'a, V> {
        StartsWith::new(self, prefix)
    }

    #[inline]
    pub fn sub_sequence<'a>(&'a self, sequence: &'a str) -> SubSequence<'a, V> {
        SubSequence::new(self, sequence)
    }

    #[inline]
    pub fn into_keys(self) -> IntoKeys<V> {
        IntoKeys {
            iter: self.map.into_keys(),
        }
    }

    #[inline]
    pub fn into_values(self) -> IntoValues<V> {
        IntoValues {
            iter: self.map.into_values(),
        }
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, V> {
        Iter {
            iter: self.map.iter(),
        }
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, V> {
        IterMut {
            iter: self.map.iter_mut(),
        }
    }

    #[inline]
    pub fn keys(&self) -> Keys<'_, V> {
        Keys {
            iter: self.map.keys(),
        }
    }

    #[inline]
    pub fn values(&self) -> Values<'_, V> {
        Values {
            iter: self.map.values(),
        }
    }

    #[inline]
    pub fn values_mut(&mut self) -> ValuesMut<'_, V> {
        ValuesMut {
            iter: self.map.values_mut(),
        }
    }
}

impl<V> Default for StringMap<V> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<V> IntoIterator for StringMap<V> {
    type Item = (Box<str>, V);
    type IntoIter = IntoIter<V>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            iter: self.map.into_iter(),
        }
    }
}

impl<K: AsRef<str>, V> FromIterator<(K, V)> for StringMap<V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let map = iter.into_iter().map(kv).collect();

        let mut map = StringMap {
            map,
            ..Default::default()
        };

        map.update_fst();
        map
    }
}

impl<K: AsRef<str>, V> Extend<(K, V)> for StringMap<V> {
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        let iter = iter.into_iter().map(kv);

        self.map.extend(iter);
        self.update_fst();
    }
}

impl<'a, K: AsRef<str>, V: Copy> Extend<(&'a K, &'a V)> for StringMap<V> {
    fn extend<T: IntoIterator<Item = (&'a K, &'a V)>>(&mut self, iter: T) {
        let iter = iter.into_iter().map(kv_ref);

        self.map.extend(iter);
        self.update_fst();
    }
}

impl<V> fmt::Debug for StringMap<V>
where
    V: fmt::Debug,
{
    #[inline]
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.map, fmt)
    }
}

#[inline]
fn kv<K: AsRef<str>, V>((key, value): (K, V)) -> (Key, V) {
    (Key::from(key.as_ref()), value)
}

#[inline]
fn kv_ref<'a, K: AsRef<str>, V: Copy>((key, value): (&'a K, &'a V)) -> (Key, V) {
    (Key::from(key.as_ref()), *value)
}
