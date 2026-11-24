//! Generation-keyed [`wgpu::BindGroup`] cache so passes reuse bind groups
//! across frames until a bound buffer, texture view or generation changes.

#![allow(dead_code)]

use std::collections::HashMap;
use std::hash::Hash;

/// Single-entry bind-group cache keyed by `K`.
#[derive(Debug)]
pub(crate) struct SingleBindCache<K> {
    entry: Option<(K, wgpu::BindGroup)>,
}

impl<K> Default for SingleBindCache<K> {
    fn default() -> Self {
        Self { entry: None }
    }
}

impl<K: PartialEq> SingleBindCache<K> {
    /// Returns the cached bind group if `key` matches, otherwise rebuilds it
    /// with `make`.
    pub(crate) fn get_or_insert_with(
        &mut self,
        key: K,
        make: impl FnOnce() -> wgpu::BindGroup,
    ) -> &wgpu::BindGroup {
        let hit = self.entry.as_ref().is_some_and(|(k, _)| *k == key);
        if !hit {
            self.entry = Some((key, make()));
        }
        &self.entry.as_ref().expect("populated above").1
    }

    pub(crate) fn clear(&mut self) {
        self.entry = None;
    }
}

/// Multi-entry bind-group cache with frame-based stale entry eviction.
#[derive(Debug)]
pub(crate) struct BindCache<K> {
    entries: HashMap<K, (u64, wgpu::BindGroup)>,
    frame: u64,
    keep_frames: u64,
}

impl<K: Eq + Hash> BindCache<K> {
    pub(crate) fn new(keep_frames: u64) -> Self {
        Self {
            entries: HashMap::new(),
            frame: 0,
            keep_frames: keep_frames.max(1),
        }
    }

    /// Advances the frame counter and drops entries unused for `keep_frames`.
    pub(crate) fn begin_frame(&mut self) {
        self.frame = self.frame.wrapping_add(1);
        let cutoff = self.frame.saturating_sub(self.keep_frames);
        self.entries.retain(|_, (last, _)| *last >= cutoff);
    }

    /// Fetches or builds the bind group for `key`, marking it used this frame.
    pub(crate) fn get_or_insert_with(
        &mut self,
        key: K,
        make: impl FnOnce() -> wgpu::BindGroup,
    ) -> &wgpu::BindGroup {
        let frame = self.frame;
        let entry = self.entries.entry(key).or_insert_with(|| (frame, make()));
        entry.0 = frame;
        &entry.1
    }

    pub(crate) fn clear(&mut self) {
        self.entries.clear();
    }
}
