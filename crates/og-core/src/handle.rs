//! Generational handles: an index plus a generation, so a handle to a freed
//! slot is detected instead of aliasing whatever reuses the slot.
//!
//! - [`RawHandle`] is the untyped pair; it packs into a `u64`
//!   (`generation << 32 | index`).
//! - [`HandleAllocator`] hands out and frees raw handles (the ECS uses it for
//!   entity ids).
//! - [`Handle<T>`] and [`Pool<T>`] are the typed form for assets, meshes,
//!   materials, sounds and scenes.
//!
//! Invariants: generations start at 1, so the packed value `0` is never a live
//! handle; freeing bumps the slot's generation; a slot whose generation
//! reaches [`RETIRED_GENERATION`] (2^21) is retired and never handed out
//! again, so a live handle's generation is at most [`MAX_GENERATION`] and
//! `to_bits()` is at most 2^53 - 1: every handle and entity id is an exact
//! JavaScript number (`Number.MAX_SAFE_INTEGER`). Freed slots are reused
//! last-freed-first. A dead handle is an [`ErrorKind::DeadHandle`] error
//! value, never a panic.

use crate::error::{Error, ErrorKind, Result};
use std::fmt;
use std::marker::PhantomData;

/// The generation a slot reaches when it is freed for the last time: it is
/// retired and never handed out again. 2^21, so that `generation << 32 |
/// index` of every live handle stays below 2^53.
pub const RETIRED_GENERATION: u32 = 1 << 21;

/// The largest generation a live handle can have (`RETIRED_GENERATION - 1`).
pub const MAX_GENERATION: u32 = RETIRED_GENERATION - 1;

/// An untyped generational handle. `#[repr(C)]`: `index` then `generation`,
/// two little-endian `u32`, so a handle in memory is the `u64` of
/// [`RawHandle::to_bits`] and a host can read an array of them in place.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(C)]
pub struct RawHandle {
    index: u32,
    generation: u32,
}

impl RawHandle {
    pub const fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }

    pub const fn index(self) -> u32 {
        self.index
    }

    pub const fn generation(self) -> u32 {
        self.generation
    }

    /// `generation << 32 | index`. For every handle an allocator hands out
    /// this is at most 2^53 - 1, an exact JavaScript number.
    pub const fn to_bits(self) -> u64 {
        ((self.generation as u64) << 32) | self.index as u64
    }

    pub const fn from_bits(bits: u64) -> Self {
        Self {
            index: bits as u32,
            generation: (bits >> 32) as u32,
        }
    }
}

impl fmt::Debug for RawHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}v{}", self.index, self.generation)
    }
}

impl fmt::Display for RawHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}v{}", self.index, self.generation)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Slot {
    generation: u32,
    alive: bool,
}

/// One slot of a [`HandleAllocator`] as exported for snapshots.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlotState {
    pub generation: u32,
    pub alive: bool,
}

/// Allocates and frees [`RawHandle`]s.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HandleAllocator {
    slots: Vec<Slot>,
    free: Vec<u32>,
    live: usize,
}

impl HandleAllocator {
    pub fn new() -> Self {
        Self::default()
    }

    /// A new live handle: the most recently freed slot, or a new slot.
    pub fn alloc(&mut self) -> RawHandle {
        self.live += 1;
        if let Some(index) = self.free.pop() {
            let slot = &mut self.slots[index as usize];
            slot.alive = true;
            return RawHandle::new(index, slot.generation);
        }
        let index = u32::try_from(self.slots.len()).expect("more than u32::MAX handles");
        self.slots.push(Slot {
            generation: 1,
            alive: true,
        });
        RawHandle::new(index, 1)
    }

    /// Frees a live handle; its slot's generation moves on. A slot whose
    /// generation reaches [`RETIRED_GENERATION`] is retired, not reused.
    pub fn free(&mut self, handle: RawHandle) -> Result<()> {
        self.check(handle)?;
        let slot = &mut self.slots[handle.index as usize];
        slot.alive = false;
        self.live -= 1;
        slot.generation += 1;
        if slot.generation < RETIRED_GENERATION {
            self.free.push(handle.index);
        }
        Ok(())
    }

    pub fn is_alive(&self, handle: RawHandle) -> bool {
        self.slots
            .get(handle.index as usize)
            .is_some_and(|s| s.alive && s.generation == handle.generation)
    }

    /// `Ok` for a live handle, a [`ErrorKind::DeadHandle`] error otherwise.
    pub fn check(&self, handle: RawHandle) -> Result<()> {
        match self.slots.get(handle.index as usize) {
            Some(s) if s.alive && s.generation == handle.generation => Ok(()),
            Some(s) => Err(Error::new(
                ErrorKind::DeadHandle,
                format!(
                    "handle {handle} is dead (slot {} is at generation {}{})",
                    handle.index,
                    s.generation,
                    if s.alive { "" } else { ", free" }
                ),
            )),
            None => Err(Error::new(
                ErrorKind::DeadHandle,
                format!("handle {handle} was never allocated"),
            )),
        }
    }

    /// The live handle currently occupying `index`, if any.
    pub fn live_at(&self, index: u32) -> Option<RawHandle> {
        self.slots
            .get(index as usize)
            .filter(|s| s.alive)
            .map(|s| RawHandle::new(index, s.generation))
    }

    /// Number of live handles.
    pub fn len(&self) -> usize {
        self.live
    }

    pub fn is_empty(&self) -> bool {
        self.live == 0
    }

    /// Number of slots ever created (live, free and retired).
    pub fn slot_count(&self) -> usize {
        self.slots.len()
    }

    /// Every slot in index order, for snapshots.
    pub fn slots(&self) -> impl Iterator<Item = SlotState> + '_ {
        self.slots.iter().map(|s| SlotState {
            generation: s.generation,
            alive: s.alive,
        })
    }

    /// The free list in reuse order (the last entry is reused first).
    pub fn free_list(&self) -> &[u32] {
        &self.free
    }

    /// Rebuilds an allocator from [`Self::slots`] and [`Self::free_list`].
    /// Refused unless every generation is in `1..=RETIRED_GENERATION`, every
    /// free entry is a distinct dead slot below [`RETIRED_GENERATION`], and
    /// every dead slot outside the free list is retired (generation
    /// [`RETIRED_GENERATION`]).
    pub fn from_parts(slots: &[SlotState], free: &[u32]) -> Result<Self> {
        let invalid = |reason: String| Err(Error::new(ErrorKind::Parse, reason));
        let mut in_free = vec![false; slots.len()];
        for &index in free {
            match slots.get(index as usize) {
                None => return invalid(format!("free list names slot {index}, past the end")),
                Some(s) if s.alive => return invalid(format!("free list names live slot {index}")),
                Some(_) if in_free[index as usize] => {
                    return invalid(format!("free list names slot {index} twice"));
                }
                Some(_) => in_free[index as usize] = true,
            }
        }
        let mut live = 0;
        for (index, s) in slots.iter().enumerate() {
            if s.generation == 0 {
                return invalid(format!("slot {index} has generation 0"));
            }
            if s.generation > RETIRED_GENERATION {
                return invalid(format!(
                    "slot {index} has generation {}, past the retirement generation {RETIRED_GENERATION}",
                    s.generation
                ));
            }
            let retired = s.generation == RETIRED_GENERATION;
            if s.alive {
                if retired {
                    return invalid(format!("live slot {index} is at the retirement generation"));
                }
                live += 1;
            } else if in_free[index] && retired {
                return invalid(format!("free list names retired slot {index}"));
            } else if !in_free[index] && !retired {
                return invalid(format!("dead slot {index} is neither free nor retired"));
            }
        }
        Ok(Self {
            slots: slots
                .iter()
                .map(|s| Slot {
                    generation: s.generation,
                    alive: s.alive,
                })
                .collect(),
            free: free.to_vec(),
            live,
        })
    }
}

/// A typed generational handle into a [`Pool<T>`].
pub struct Handle<T> {
    raw: RawHandle,
    _marker: PhantomData<fn() -> T>,
}

impl<T> Handle<T> {
    pub const fn from_raw(raw: RawHandle) -> Self {
        Self {
            raw,
            _marker: PhantomData,
        }
    }

    pub const fn raw(self) -> RawHandle {
        self.raw
    }

    pub const fn to_bits(self) -> u64 {
        self.raw.to_bits()
    }

    pub const fn from_bits(bits: u64) -> Self {
        Self::from_raw(RawHandle::from_bits(bits))
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for Handle<T> {}
impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}
impl<T> Eq for Handle<T> {}
impl<T> std::hash::Hash for Handle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.raw.hash(state)
    }
}
impl<T> fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Handle({})", self.raw)
    }
}

/// Values addressed by generational [`Handle`]s.
#[derive(Clone, Debug)]
pub struct Pool<T> {
    alloc: HandleAllocator,
    values: Vec<Option<T>>,
}

impl<T> Default for Pool<T> {
    fn default() -> Self {
        Self {
            alloc: HandleAllocator::new(),
            values: Vec::new(),
        }
    }
}

impl<T> Pool<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, value: T) -> Handle<T> {
        let raw = self.alloc.alloc();
        let index = raw.index() as usize;
        if index == self.values.len() {
            self.values.push(Some(value));
        } else {
            self.values[index] = Some(value);
        }
        Handle::from_raw(raw)
    }

    pub fn get(&self, handle: Handle<T>) -> Result<&T> {
        self.alloc.check(handle.raw)?;
        Ok(self.values[handle.raw.index() as usize]
            .as_ref()
            .expect("live slot holds a value"))
    }

    pub fn get_mut(&mut self, handle: Handle<T>) -> Result<&mut T> {
        self.alloc.check(handle.raw)?;
        Ok(self.values[handle.raw.index() as usize]
            .as_mut()
            .expect("live slot holds a value"))
    }

    pub fn remove(&mut self, handle: Handle<T>) -> Result<T> {
        self.alloc.free(handle.raw)?;
        Ok(self.values[handle.raw.index() as usize]
            .take()
            .expect("live slot holds a value"))
    }

    pub fn contains(&self, handle: Handle<T>) -> bool {
        self.alloc.is_alive(handle.raw)
    }

    pub fn len(&self) -> usize {
        self.alloc.len()
    }

    pub fn is_empty(&self) -> bool {
        self.alloc.is_empty()
    }

    /// Live values in slot order.
    pub fn iter(&self) -> impl Iterator<Item = (Handle<T>, &T)> {
        self.values.iter().enumerate().filter_map(|(i, v)| {
            let value = v.as_ref()?;
            let raw = self.alloc.live_at(i as u32)?;
            Some((Handle::from_raw(raw), value))
        })
    }
}
