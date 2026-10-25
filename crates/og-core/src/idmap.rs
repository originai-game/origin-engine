//! [`IdMap`]: an ordered map from integer ids to values whose tree is
//! compiled once.
//!
//! A `BTreeMap<K, V>` monomorphises the whole B-tree (search, insertion with
//! its splits, removal with its merges and steals, iteration, drop) for
//! every pair of key and value types: about three kilobytes of wasm a pair,
//! and the engine keeps per-entity state in dozens of them (the B-tree was
//! the module's largest code family, `tests/evidence/r4/size-3/NOTES.md`).
//! An `IdMap` keeps its entries in a `Vec<(K, V)>` and orders them through
//! one `BTreeMap<u64, u32>` from the key's bits to the entry's slot, which is
//! the same type for every map, so the tree is compiled once in this crate;
//! each map adds only the vector's moves.
//!
//! It behaves as the `BTreeMap` it replaces: every iteration (`iter`,
//! `keys`, `values`, `retain`, `into_iter`, `Debug`, serialisation) is in
//! ascending key order, `insert` returns the value it replaces, `collect`
//! and `extend` keep the last value of a repeated key, and it serialises as
//! a map of its entries in key order, so a save written from one is the
//! same bytes as from the other. A removal moves the last slot into the
//! freed one, which no iteration can see.

use std::collections::BTreeMap;
use std::fmt;
use std::marker::PhantomData;
use std::ops::{Bound, RangeBounds};

/// A key of an [`IdMap`]: an integer whose order is the order of its bits.
pub trait IdKey: Copy + Ord {
    /// The key as a `u64`, ordered as the key is.
    fn bits(self) -> u64;
}

impl IdKey for u64 {
    #[inline]
    fn bits(self) -> u64 {
        self
    }
}

impl IdKey for u32 {
    #[inline]
    fn bits(self) -> u64 {
        u64::from(self)
    }
}

impl IdKey for usize {
    #[inline]
    fn bits(self) -> u64 {
        self as u64
    }
}

/// The key bits to slot index every [`IdMap`] orders through: one type, so
/// its tree is compiled once.
#[derive(Clone, Default)]
struct Index(BTreeMap<u64, u32>);

impl Index {
    #[inline(never)]
    fn get(&self, key: u64) -> Option<usize> {
        self.0.get(&key).map(|&i| i as usize)
    }

    /// Records `key` at `slot`, which is new.
    #[inline(never)]
    fn insert(&mut self, key: u64, slot: usize) {
        let slot = u32::try_from(slot).expect("IdMap: more than u32::MAX entries");
        self.0.insert(key, slot);
    }

    #[inline(never)]
    fn remove(&mut self, key: u64) -> Option<usize> {
        self.0.remove(&key).map(|i| i as usize)
    }

    /// Moves `key`, which is recorded, to `slot`.
    #[inline(never)]
    fn moved(&mut self, key: u64, slot: usize) {
        if let Some(i) = self.0.get_mut(&key) {
            *i = slot as u32;
        }
    }

    #[inline(never)]
    fn clear(&mut self) {
        self.0.clear();
    }

    /// The slots in key order.
    #[inline(never)]
    fn order(&self) -> Vec<u32> {
        self.0.values().copied().collect()
    }

    #[inline(never)]
    fn first(&self) -> Option<usize> {
        self.0.first_key_value().map(|(_, &i)| i as usize)
    }

    #[inline(never)]
    fn last(&self) -> Option<usize> {
        self.0.last_key_value().map(|(_, &i)| i as usize)
    }

    fn slots(&self) -> Slots<'_> {
        Slots(self.0.values())
    }

    #[inline(never)]
    fn range(&self, start: Bound<u64>, end: Bound<u64>) -> RangeSlots<'_> {
        RangeSlots(self.0.range((start, end)))
    }
}

/// The slots of a range of an [`Index`]'s keys, in key order.
struct RangeSlots<'a>(std::collections::btree_map::Range<'a, u64, u32>);

impl Iterator for RangeSlots<'_> {
    type Item = usize;

    #[inline(never)]
    fn next(&mut self) -> Option<usize> {
        self.0.next().map(|(_, &i)| i as usize)
    }
}

impl DoubleEndedIterator for RangeSlots<'_> {
    #[inline(never)]
    fn next_back(&mut self) -> Option<usize> {
        self.0.next_back().map(|(_, &i)| i as usize)
    }
}

/// The slots of an [`Index`] in key order.
#[derive(Clone)]
struct Slots<'a>(std::collections::btree_map::Values<'a, u64, u32>);

impl Iterator for Slots<'_> {
    type Item = usize;

    #[inline(never)]
    fn next(&mut self) -> Option<usize> {
        self.0.next().map(|&i| i as usize)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl DoubleEndedIterator for Slots<'_> {
    #[inline(never)]
    fn next_back(&mut self) -> Option<usize> {
        self.0.next_back().map(|&i| i as usize)
    }
}

impl ExactSizeIterator for Slots<'_> {}

/// An ordered map from integer ids to values, as `BTreeMap<K, V>` is, with
/// one tree compiled for every map (the module's docs).
#[derive(Clone)]
pub struct IdMap<K, V> {
    index: Index,
    entries: Vec<(K, V)>,
}

impl<K, V> Default for IdMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> IdMap<K, V> {
    /// An empty map.
    pub const fn new() -> Self {
        Self {
            index: Index(BTreeMap::new()),
            entries: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn clear(&mut self) {
        self.index.clear();
        self.entries.clear();
    }

    /// The entries in key order.
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            slots: self.index.slots(),
            entries: &self.entries,
        }
    }

    /// The entries in key order, the values mutable.
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        IterMut {
            slots: self.index.slots(),
            entries: self.entries.as_mut_ptr(),
            len: self.entries.len(),
            _entries: PhantomData,
        }
    }

    /// The keys in order.
    pub fn keys(&self) -> impl DoubleEndedIterator<Item = &K> + ExactSizeIterator + Clone {
        self.iter().map(|(k, _)| k)
    }

    /// The values in key order.
    pub fn values(&self) -> impl DoubleEndedIterator<Item = &V> + ExactSizeIterator + Clone {
        self.iter().map(|(_, v)| v)
    }

    /// The values in key order, mutable.
    pub fn values_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut V> + ExactSizeIterator {
        self.iter_mut().map(|(_, v)| v)
    }

    /// The entry of the least key.
    pub fn first_key_value(&self) -> Option<(&K, &V)> {
        self.index.first().map(|i| {
            let (k, v) = &self.entries[i];
            (k, v)
        })
    }

    /// The entry of the greatest key.
    pub fn last_key_value(&self) -> Option<(&K, &V)> {
        self.index.last().map(|i| {
            let (k, v) = &self.entries[i];
            (k, v)
        })
    }

    /// The keys in order, the map consumed.
    pub fn into_keys(mut self) -> impl DoubleEndedIterator<Item = K> + ExactSizeIterator {
        self.drain_ordered().into_iter().map(|(k, _)| k)
    }

    /// The values in key order, the map consumed.
    pub fn into_values(mut self) -> impl DoubleEndedIterator<Item = V> + ExactSizeIterator {
        self.drain_ordered().into_iter().map(|(_, v)| v)
    }

    /// The values in key order, the map left empty.
    fn drain_ordered(&mut self) -> Vec<(K, V)> {
        let order = self.index.order();
        self.index.clear();
        let mut taken: Vec<Option<(K, V)>> = std::mem::take(&mut self.entries)
            .into_iter()
            .map(Some)
            .collect();
        order
            .into_iter()
            .map(|i| {
                taken[i as usize]
                    .take()
                    .expect("IdMap: a slot ordered twice")
            })
            .collect()
    }
}

impl<K: IdKey, V> IdMap<K, V> {
    pub fn get(&self, key: &K) -> Option<&V> {
        self.index.get(key.bits()).map(|i| &self.entries[i].1)
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.index.get(key.bits()).map(|i| &mut self.entries[i].1)
    }

    /// The key as stored and its value.
    pub fn get_key_value(&self, key: &K) -> Option<(&K, &V)> {
        self.index.get(key.bits()).map(|i| {
            let (k, v) = &self.entries[i];
            (k, v)
        })
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.index.get(key.bits()).is_some()
    }

    /// Sets `key` to `value`, returning the value it had.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        match self.index.get(key.bits()) {
            Some(i) => Some(std::mem::replace(&mut self.entries[i].1, value)),
            None => {
                self.index.insert(key.bits(), self.entries.len());
                self.entries.push((key, value));
                None
            }
        }
    }

    /// Removes `key`, returning its value.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.remove_entry(key).map(|(_, v)| v)
    }

    /// Removes `key`, returning the key as stored and its value.
    pub fn remove_entry(&mut self, key: &K) -> Option<(K, V)> {
        let i = self.index.remove(key.bits())?;
        let entry = self.entries.swap_remove(i);
        if let Some((moved, _)) = self.entries.get(i) {
            self.index.moved(moved.bits(), i);
        }
        Some(entry)
    }

    /// The entries whose keys are in `range`, in key order (panicking where
    /// `BTreeMap::range` does).
    pub fn range(&self, range: impl RangeBounds<K>) -> impl DoubleEndedIterator<Item = (&K, &V)> {
        let bits = |b: Bound<&K>| match b {
            Bound::Included(k) => Bound::Included(k.bits()),
            Bound::Excluded(k) => Bound::Excluded(k.bits()),
            Bound::Unbounded => Bound::Unbounded,
        };
        let slots = self
            .index
            .range(bits(range.start_bound()), bits(range.end_bound()));
        slots.map(move |i| {
            let (k, v) = &self.entries[i];
            (k, v)
        })
    }

    /// Removes and returns the entry of the least key.
    pub fn pop_first(&mut self) -> Option<(K, V)> {
        let key = self.entries[self.index.first()?].0;
        self.remove_entry(&key)
    }

    /// Removes and returns the entry of the greatest key.
    pub fn pop_last(&mut self) -> Option<(K, V)> {
        let key = self.entries[self.index.last()?].0;
        self.remove_entry(&key)
    }

    /// Keeps the entries `keep` answers true for, visiting them in key
    /// order as `BTreeMap::retain` does.
    pub fn retain(&mut self, mut keep: impl FnMut(&K, &mut V) -> bool) {
        let mut gone = Vec::new();
        for (k, v) in self.iter_mut() {
            if !keep(k, v) {
                gone.push(*k);
            }
        }
        for k in gone {
            self.remove(&k);
        }
    }

    /// The value of `key`, inserted by `make` when there is none.
    pub fn get_or_insert_with(&mut self, key: K, make: impl FnOnce() -> V) -> &mut V {
        let i = match self.index.get(key.bits()) {
            Some(i) => i,
            None => {
                let i = self.entries.len();
                self.index.insert(key.bits(), i);
                self.entries.push((key, make()));
                i
            }
        };
        &mut self.entries[i].1
    }

    /// The entry of `key`, as `BTreeMap::entry` gives it.
    pub fn entry(&mut self, key: K) -> Entry<'_, K, V> {
        Entry { map: self, key }
    }
}

/// An [`IdMap`]'s place for one key: [`IdMap::entry`].
pub struct Entry<'a, K, V> {
    map: &'a mut IdMap<K, V>,
    key: K,
}

impl<'a, K: IdKey, V> Entry<'a, K, V> {
    pub fn key(&self) -> &K {
        &self.key
    }

    pub fn or_insert(self, value: V) -> &'a mut V {
        self.map.get_or_insert_with(self.key, || value)
    }

    pub fn or_insert_with(self, make: impl FnOnce() -> V) -> &'a mut V {
        self.map.get_or_insert_with(self.key, make)
    }

    pub fn or_default(self) -> &'a mut V
    where
        V: Default,
    {
        self.map.get_or_insert_with(self.key, V::default)
    }

    pub fn and_modify(self, f: impl FnOnce(&mut V)) -> Self {
        if let Some(v) = self.map.get_mut(&self.key) {
            f(v);
        }
        self
    }
}

/// An [`IdMap`]'s entries in key order.
pub struct Iter<'a, K, V> {
    slots: Slots<'a>,
    entries: &'a [(K, V)],
}

impl<K, V> Clone for Iter<'_, K, V> {
    fn clone(&self) -> Self {
        Iter {
            slots: self.slots.clone(),
            entries: self.entries,
        }
    }
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        let (k, v) = &self.entries[self.slots.next()?];
        Some((k, v))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.slots.size_hint()
    }
}

impl<K, V> DoubleEndedIterator for Iter<'_, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let (k, v) = &self.entries[self.slots.next_back()?];
        Some((k, v))
    }
}

impl<K, V> ExactSizeIterator for Iter<'_, K, V> {}

/// An [`IdMap`]'s entries in key order, the values mutable.
pub struct IterMut<'a, K, V> {
    slots: Slots<'a>,
    entries: *mut (K, V),
    len: usize,
    _entries: PhantomData<&'a mut [(K, V)]>,
}

impl<'a, K, V> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.slots.next()?;
        assert!(i < self.len);
        // SAFETY: the index maps each key to a distinct slot below `len`,
        // and the iterator yields each key once, so no two references it
        // hands out alias; it borrows the map mutably for 'a, so the
        // entries neither move nor change length meanwhile.
        let (k, v) = unsafe { &mut *self.entries.add(i) };
        Some((&*k, v))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.slots.size_hint()
    }
}

impl<K, V> DoubleEndedIterator for IterMut<'_, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let i = self.slots.next_back()?;
        assert!(i < self.len);
        // SAFETY: as in `next`; both ends draw from one sequence of slots.
        let (k, v) = unsafe { &mut *self.entries.add(i) };
        Some((&*k, v))
    }
}

impl<K, V> ExactSizeIterator for IterMut<'_, K, V> {}

// SAFETY: an IterMut is a `&mut [(K, V)]` handed out entry by entry.
unsafe impl<K: Sync, V: Send> Send for IterMut<'_, K, V> {}
// SAFETY: as above; shared access to it reads nothing through the pointer.
unsafe impl<K: Sync, V: Sync> Sync for IterMut<'_, K, V> {}

impl<'a, K, V> IntoIterator for &'a IdMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K, V> IntoIterator for &'a mut IdMap<K, V> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<K, V> IntoIterator for IdMap<K, V> {
    type Item = (K, V);
    type IntoIter = std::vec::IntoIter<(K, V)>;

    /// The entries in key order.
    fn into_iter(mut self) -> Self::IntoIter {
        self.drain_ordered().into_iter()
    }
}

impl<K: IdKey, V> Extend<(K, V)> for IdMap<K, V> {
    fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iter: I) {
        for (k, v) in iter {
            self.insert(k, v);
        }
    }
}

impl<K: IdKey, V> FromIterator<(K, V)> for IdMap<K, V> {
    /// The map of the pairs; a repeated key keeps its last value, as
    /// `BTreeMap`'s `collect` does.
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let mut map = Self::new();
        map.extend(iter);
        map
    }
}

impl<K: IdKey, V> std::ops::Index<&K> for IdMap<K, V> {
    type Output = V;

    fn index(&self, key: &K) -> &V {
        self.get(key).expect("IdMap: no entry for the key")
    }
}

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for IdMap<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<K: IdKey, V: PartialEq> PartialEq for IdMap<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().zip(other.iter()).all(|(a, b)| a == b)
    }
}

impl<K: serde::Serialize, V: serde::Serialize> serde::Serialize for IdMap<K, V> {
    /// A map of the entries in key order, as `BTreeMap` writes itself.
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_map(self.iter())
    }
}

impl<'de, K: IdKey + serde::Deserialize<'de>, V: serde::Deserialize<'de>> serde::Deserialize<'de>
    for IdMap<K, V>
{
    /// A map, a repeated key keeping its last value, as `BTreeMap` reads
    /// itself.
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visit<K, V>(PhantomData<(K, V)>);
        impl<'de, K: IdKey + serde::Deserialize<'de>, V: serde::Deserialize<'de>>
            serde::de::Visitor<'de> for Visit<K, V>
        {
            type Value = IdMap<K, V>;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a map")
            }

            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut access: A,
            ) -> Result<Self::Value, A::Error> {
                let mut map = IdMap::new();
                while let Some((k, v)) = access.next_entry()? {
                    map.insert(k, v);
                }
                Ok(map)
            }
        }
        d.deserialize_map(Visit(PhantomData))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Rng;

    /// Random insertions, removals, retains, entries and pops, across key
    /// ranges dense and sparse, applied to an `IdMap` and to the
    /// `BTreeMap` it replaces: after every operation the two answer the
    /// same, iterate the same entries in the same order (forwards, backwards,
    /// mutably), visit the same entries in `retain`, and serialise to the
    /// same text.
    #[test]
    fn behaves_as_the_btreemap_it_replaces() {
        let mut rng = Rng::new(11);
        for span in [4i64, 40, 4000, 0] {
            let mut ours: IdMap<u64, (u32, String)> = IdMap::new();
            let mut theirs: BTreeMap<u64, (u32, String)> = BTreeMap::new();
            for step in 0..3000u32 {
                // Span 0: keys anywhere in u64, the high bit included.
                let k = match span {
                    0 => u64::from(rng.next_u32()) << 32 | u64::from(rng.next_u32() % 8),
                    _ => rng.integer(0, span) as u64,
                };
                let v = (step, format!("v{step}"));
                match rng.integer(0, 9) {
                    0..=2 => assert_eq!(ours.insert(k, v.clone()), theirs.insert(k, v)),
                    3 | 4 => assert_eq!(ours.remove(&k), theirs.remove(&k)),
                    5 => {
                        let (mut a, mut b) = (Vec::new(), Vec::new());
                        let cut = rng.integer(0, 3) as u32;
                        ours.retain(|k, v| {
                            a.push(*k);
                            v.0 += 1;
                            (k + u64::from(v.0)) % 4 != u64::from(cut)
                        });
                        theirs.retain(|k, v| {
                            b.push(*k);
                            v.0 += 1;
                            (k + u64::from(v.0)) % 4 != u64::from(cut)
                        });
                        assert_eq!(a, b, "retain visits in key order");
                    }
                    6 => {
                        ours.entry(k).or_default().0 += 1;
                        theirs.entry(k).or_default().0 += 1;
                    }
                    7 => {
                        if let (Some(a), Some(b)) = (ours.get_mut(&k), theirs.get_mut(&k)) {
                            a.1.push('!');
                            b.1.push('!');
                        }
                    }
                    8 => assert_eq!(ours.pop_first(), theirs.pop_first()),
                    _ => assert_eq!(ours.pop_last(), theirs.pop_last()),
                }
                assert_eq!(ours.len(), theirs.len());
                assert_eq!(ours.get(&k), theirs.get(&k));
                assert_eq!(ours.contains_key(&k), theirs.contains_key(&k));
                assert_eq!(ours.first_key_value(), theirs.first_key_value());
                assert_eq!(ours.last_key_value(), theirs.last_key_value());
                if step % 50 == 0 {
                    assert!(ours.iter().eq(theirs.iter()));
                    assert!(ours.iter().rev().eq(theirs.iter().rev()));
                    assert!(ours.keys().eq(theirs.keys()));
                    assert!(ours.values().eq(theirs.values()));
                    let a: Vec<_> = ours.iter_mut().map(|(k, v)| (*k, v.clone())).collect();
                    let b: Vec<_> = theirs.iter_mut().map(|(k, v)| (*k, v.clone())).collect();
                    assert_eq!(a, b);
                    assert_eq!(
                        serde_json::to_string(&ours).unwrap(),
                        serde_json::to_string(&theirs).unwrap()
                    );
                    assert_eq!(format!("{ours:?}"), format!("{theirs:?}"));
                    // The physics save's codec: the length prefix and the
                    // entries in key order, byte for byte.
                    assert_eq!(
                        bincode::serialize(&ours).unwrap(),
                        bincode::serialize(&theirs).unwrap()
                    );
                }
            }
            assert!(ours.clone().into_iter().eq(theirs.clone()));
        }
    }

    /// `collect` and deserialising keep the last value of a repeated key, as
    /// `BTreeMap`'s do; a map written by one reads back as the other.
    #[test]
    fn collects_and_reads_back_as_the_btreemap() {
        let pairs = [(5u32, 'a'), (1, 'b'), (5, 'c'), (3, 'd'), (1, 'e')];
        let ours: IdMap<u32, char> = pairs.into_iter().collect();
        let theirs: BTreeMap<u32, char> = pairs.into_iter().collect();
        assert!(ours.iter().eq(theirs.iter()));
        let text = r#"{"9":1,"2":2,"9":3,"0":4}"#;
        let ours: IdMap<u64, u8> = serde_json::from_str(text).unwrap();
        let theirs: BTreeMap<u64, u8> = serde_json::from_str(text).unwrap();
        assert!(ours.iter().eq(theirs.iter()));
        let back: BTreeMap<u64, u8> =
            serde_json::from_str(&serde_json::to_string(&ours).unwrap()).unwrap();
        assert_eq!(back, theirs);
    }
}
