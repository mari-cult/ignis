//! Forward all impls of `BTreeMap` iterators here.

use {
    crate::Key,
    std::{collections::btree_map, fmt, iter::FusedIterator},
};

/// An owning iterator over the entries of a `StringMap`.
pub struct IntoIter<V> {
    pub(crate) iter: btree_map::IntoIter<Key, V>,
}

/// An owning iterator over the keys of a `StringMap`.
pub struct IntoKeys<V> {
    pub(crate) iter: btree_map::IntoKeys<Key, V>,
}

/// An owning iterator over the values of a `StringMap`.
pub struct IntoValues<V> {
    pub(crate) iter: btree_map::IntoValues<Key, V>,
}

/// An iterator over the entries of `StringMap`.
pub struct Iter<'a, V> {
    pub(crate) iter: btree_map::Iter<'a, Key, V>,
}

/// A mutable iterator over the entries of `StringMap`.
pub struct IterMut<'a, V> {
    pub(crate) iter: btree_map::IterMut<'a, Key, V>,
}

/// An iterator over the keys of `StringMap`.
pub struct Keys<'a, V> {
    pub(crate) iter: btree_map::Keys<'a, Key, V>,
}

/// An iterator over the values of `StringMap`.
pub struct Values<'a, V> {
    pub(crate) iter: btree_map::Values<'a, Key, V>,
}

/// A mutable iterator over the values of `StringMap`.
pub struct ValuesMut<'a, V> {
    pub(crate) iter: btree_map::ValuesMut<'a, Key, V>,
}

/// Implement common traits for iterators.
macro_rules! impl_iter {
    ($ident:ident) => {
        impl<V> ExactSizeIterator for $ident<V> {
            #[inline]
            fn len(&self) -> usize {
                self.iter.len()
            }
        }

        impl<V> FusedIterator for $ident<V> {}

        impl<V: fmt::Debug> fmt::Debug for $ident<V> {
            #[inline]
            fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Debug::fmt(&self.iter, fmt)
            }
        }
    };
    ($ident:ident<$lifetime:lifetime>) => {
        impl<$lifetime, V> ExactSizeIterator for $ident<$lifetime, V> {
            #[inline]
            fn len(&self) -> usize {
                self.iter.len()
            }
        }

        impl<$lifetime, V> FusedIterator for $ident<$lifetime, V> {}

        impl<$lifetime, V: fmt::Debug> fmt::Debug for $ident<$lifetime, V> {
            #[inline]
            fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Debug::fmt(&self.iter, fmt)
            }
        }
    };
}

impl_iter!(IntoIter);
impl_iter!(IntoKeys);
impl_iter!(IntoValues);
impl_iter!(Iter<'a>);
impl_iter!(IterMut<'a>);
impl_iter!(Keys<'a>);
impl_iter!(Values<'a>);
impl_iter!(ValuesMut<'a>);

// https://doc.rust-lang.org/1.69.0/src/alloc/collections/btree/map.rs.html#1691
impl<V> Iterator for IntoIter<V> {
    type Item = (Box<str>, V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(key, value)| (key.into(), value))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<V> DoubleEndedIterator for IntoIter<V> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter
            .next_back()
            .map(|(key, value)| (key.into(), value))
    }
}

// https://doc.rust-lang.org/1.69.0/src/alloc/collections/btree/map.rs.html#1983
impl<V> Iterator for IntoKeys<V> {
    type Item = Box<str>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(Into::into)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        self.iter.last().map(Into::into)
    }

    #[inline]
    fn min(mut self) -> Option<Self::Item> {
        self.iter.next().map(Into::into)
    }

    #[inline]
    fn max(mut self) -> Option<Self::Item> {
        self.iter.next_back().map(Into::into)
    }
}

impl<V> DoubleEndedIterator for IntoKeys<V> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(Into::into)
    }
}

// https://doc.rust-lang.org/1.69.0/src/alloc/collections/btree/map.rs.html#2025
impl<V> Iterator for IntoValues<V> {
    type Item = V;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        self.iter.last()
    }
}

impl<V> DoubleEndedIterator for IntoValues<V> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(Into::into)
    }
}

// https://doc.rust-lang.org/1.69.0/src/alloc/collections/btree/map.rs.html#1482
impl<'a, V: 'a> Iterator for Iter<'a, V> {
    type Item = (&'a str, &'a V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(kv)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        self.iter.last().map(kv)
    }

    #[inline]
    fn min(mut self) -> Option<Self::Item> {
        self.iter.next().map(kv)
    }

    #[inline]
    fn max(mut self) -> Option<Self::Item> {
        self.iter.next_back().map(kv)
    }
}

impl<'a, V: 'a> DoubleEndedIterator for Iter<'a, V> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(kv)
    }
}

// https://doc.rust-lang.org/1.69.0/src/alloc/collections/btree/map.rs.html#1551
impl<'a, V: 'a> Iterator for IterMut<'a, V> {
    type Item = (&'a str, &'a mut V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(kv_mut)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        self.iter.last().map(kv_mut)
    }

    #[inline]
    fn min(mut self) -> Option<Self::Item> {
        self.iter.next().map(kv_mut)
    }

    #[inline]
    fn max(mut self) -> Option<Self::Item> {
        self.iter.next_back().map(kv_mut)
    }
}

impl<'a, V: 'a> DoubleEndedIterator for IterMut<'a, V> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(kv_mut)
    }
}

// https://doc.rust-lang.org/1.69.0/src/alloc/collections/btree/map.rs.html#1723
impl<'a, V: 'a> Iterator for Keys<'a, V> {
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(k)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        self.iter.last().map(k)
    }

    #[inline]
    fn min(mut self) -> Option<Self::Item> {
        self.iter.next().map(k)
    }

    #[inline]
    fn max(mut self) -> Option<Self::Item> {
        self.iter.next_back().map(k)
    }
}

impl<'a, V: 'a> DoubleEndedIterator for Keys<'a, V> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(k)
    }
}

// https://doc.rust-lang.org/1.69.0/src/alloc/collections/btree/map.rs.html#1772
impl<'a, V: 'a> Iterator for Values<'a, V> {
    type Item = &'a V;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        self.iter.last()
    }
}

impl<'a, V: 'a> DoubleEndedIterator for Values<'a, V> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }
}

impl<'a, V: 'a> Iterator for ValuesMut<'a, V> {
    type Item = &'a mut V;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        self.iter.last()
    }
}

impl<'a, V: 'a> DoubleEndedIterator for ValuesMut<'a, V> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()
    }
}

#[inline]
fn k(key: &Key) -> &str {
    key
}

#[inline]
pub fn kv<'a, V: 'a>((key, value): (&'a Key, &'a V)) -> (&'a str, &'a V) {
    (key, value)
}

#[inline]
fn kv_mut<'a, V: 'a>((key, value): (&'a Key, &'a mut V)) -> (&'a str, &'a mut V) {
    (key, value)
}
