//! Parses of computed CSS text, kept by the text. A cascade's winners are
//! strings once their variables are substituted, and an element styled
//! again (a class toggled, a transition retargeted) mostly computes the
//! strings it computed before: the parse of each is a pure function of its
//! text, so it is kept on the thread that made it and answered again
//! instead of parsed again.

use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;

/// The most parses one generation keeps; past it, the generation before
/// is let go and this one becomes it.
const CAP: usize = 4096;

/// Parse results by their input. Kept per thread
/// (`thread_local!`), so the UIs of one thread share it and no lock is
/// taken; an answer is the same whichever UI asked first.
///
/// Two generations: what is kept goes into the young one, and a full
/// young one becomes the old one, the old one let go. An answer found in
/// the old one moves back to the young one, so what a screen keeps asking
/// for outlives any number of turns and only what nothing asked for in a
/// whole generation is parsed again.
pub(crate) struct Memo<K, V> {
    young: HashMap<K, V>,
    old: HashMap<K, V>,
}

impl<K: Hash + Eq, V> Default for Memo<K, V> {
    fn default() -> Self {
        Memo {
            young: HashMap::new(),
            old: HashMap::new(),
        }
    }
}

impl<K: Hash + Eq, V> Memo<K, V> {
    pub(crate) fn get<Q: Hash + Eq + ?Sized>(&mut self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
    {
        self.get_mut(key).map(|v| &*v)
    }

    pub(crate) fn get_mut<Q: Hash + Eq + ?Sized>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
    {
        if !self.young.contains_key(key) {
            let (k, v) = self.old.remove_entry(key)?;
            self.keep(k, v);
        }
        self.young.get_mut(key)
    }

    pub(crate) fn keep(&mut self, key: K, value: V) {
        if self.young.len() >= CAP {
            self.old = std::mem::take(&mut self.young);
        }
        self.young.insert(key, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What is asked for again survives the turn of generations; what is
    /// not is parsed again only after two. Ways it could fail: a full memo
    /// cleared at once (the hot entry lost), an old answer not moved back.
    #[test]
    fn hot_entries_outlive_generations() {
        let mut m = Memo::default();
        m.keep(u32::MAX, "hot");
        for k in 0..CAP as u32 * 3 {
            m.keep(k, "cold");
            if k % 1000 == 0 {
                assert_eq!(m.get(&u32::MAX), Some(&"hot"), "after {k}");
            }
        }
        assert_eq!(m.get(&u32::MAX), Some(&"hot"));
        assert_eq!(m.get(&0), None, "a cold entry is let go");
        assert!(m.young.len() + m.old.len() <= 2 * CAP);
    }
}
