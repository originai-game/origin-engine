//! Parses of computed CSS text, kept by the text. A cascade's winners are
//! strings once their variables are substituted, and an element styled
//! again (a class toggled, a transition retargeted) mostly computes the
//! strings it computed before: the parse of each is a pure function of its
//! text, so it is kept on the thread that made it and answered again
//! instead of parsed again.

use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;

/// The most parses one memo keeps; past it, it starts again empty.
const CAP: usize = 4096;

/// Parse results by their input. Kept per thread
/// (`thread_local!`), so the UIs of one thread share it and no lock is
/// taken; an answer is the same whichever UI asked first.
pub(crate) struct Memo<K, V> {
    map: HashMap<K, V>,
}

impl<K: Hash + Eq, V> Default for Memo<K, V> {
    fn default() -> Self {
        Memo {
            map: HashMap::new(),
        }
    }
}

impl<K: Hash + Eq, V> Memo<K, V> {
    pub(crate) fn get<Q: Hash + Eq + ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
    {
        self.map.get(key)
    }

    pub(crate) fn get_mut<Q: Hash + Eq + ?Sized>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
    {
        self.map.get_mut(key)
    }

    pub(crate) fn keep(&mut self, key: K, value: V) {
        if self.map.len() >= CAP {
            self.map.clear();
        }
        self.map.insert(key, value);
    }
}
