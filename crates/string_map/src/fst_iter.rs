use {
    crate::{Key, StringMap},
    fst::{
        automaton::{self, Automaton},
        set, IntoStreamer, Streamer,
    },
    std::{collections::BTreeMap, str},
};

struct AutomatonIter<'a, A: Automaton + 'a, V: 'a> {
    map: &'a BTreeMap<Key, V>,
    stream: set::Stream<'a, A>,
}

pub struct StartsWith<'a, V: 'a> {
    iter: AutomatonIter<'a, automaton::StartsWith<automaton::Str<'a>>, V>,
}

pub struct SubSequence<'a, V: 'a> {
    iter: AutomatonIter<'a, automaton::Subsequence<'a>, V>,
}

impl<'a, A: Automaton + 'a, V: 'a> AutomatonIter<'a, A, V> {
    #[inline]
    pub fn new(map: &'a StringMap<V>, automaton: A) -> Self {
        Self {
            map: &map.map,
            stream: map.fst.search(automaton).into_stream(),
        }
    }
}

impl<'a, V: 'a> StartsWith<'a, V> {
    #[inline]
    pub fn new(map: &'a StringMap<V>, prefix: &'a str) -> Self {
        Self {
            iter: AutomatonIter::new(map, automaton::Str::new(prefix).starts_with()),
        }
    }
}

impl<'a, V: 'a> SubSequence<'a, V> {
    #[inline]
    pub fn new(map: &'a StringMap<V>, sequence: &'a str) -> Self {
        Self {
            iter: AutomatonIter::new(map, automaton::Subsequence::new(sequence)),
        }
    }
}

impl<'a, A: Automaton + 'a, V: 'a> Iterator for AutomatonIter<'a, A, V> {
    type Item = (&'a str, &'a V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.stream.next().and_then(|key| {
            let key = unsafe { str::from_utf8_unchecked(key) };

            self.map.get_key_value(key).map(crate::iter::kv)
        })
    }
}

macro_rules! impl_iter {
    ($ident:ident) => {
        impl<'a, V: 'a> Iterator for $ident<'a, V> {
            type Item = (&'a str, &'a V);

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                self.iter.next()
            }
        }
    };
}

impl_iter!(StartsWith);
impl_iter!(SubSequence);
