//! Terrain streaming (phase 3): an unedited terrain's tiles reach the
//! renderer as the camera needs them instead of being decoded whole at
//! load. Every frame the streamer asks og-render which tiles the view wants
//! (`og_render::terrain_tiles_wanted`, the function the renderer ranks
//! with), decodes the missing ones away from the frame (natively on a
//! worker thread; on the web the host's DecompressionStream inflates them
//! and the engine's thread finishes them within a time budget), keeps the
//! decoded ones in a cache of twice the terrain's tile budget, and hands
//! the renderer at most [`NEW_PER_FRAME`] new tiles a frame, each after its
//! parent. A capture settles first: every wanted tile is decoded and shown
//! at once, so captured frames do not depend on decode timing. So does the
//! first frame that draws a terrain (`terrain::extract`): no frame draws a
//! terrain before its tiles arrived, which auto exposure would meter.
//!
//! Nothing here touches the world: the tick reads terrains through
//! physics' height field and `og.terrain.sample`, never through what is
//! resident, so streaming cannot change a tick's result.

use og_cook::runtime::CookedTerrain;
use og_core::IdMap;
use og_render::{RenderScene, TerrainHandle, TerrainTile, TerrainView};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// New tiles handed to the renderer per frame (each is one upload of about
/// 160 KiB with its mips): what keeps a fast camera from a hitch.
pub(crate) const NEW_PER_FRAME: usize = 8;
/// Tiles decoding at once.
const IN_FLIGHT: usize = 8;
/// Milliseconds a frame may spend finishing inflated tiles on the engine's
/// thread (normals and mips), when the host inflated them.
const FINISH_MS: f64 = 2.0;

type Key = (usize, u32);

/// What the streamer did, for `og.stream` and `dump_stream`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct StreamStats {
    /// Tiles the renderer holds now, over every terrain.
    pub resident: u32,
    /// Tiles the views want that are not held yet.
    pub waiting: u32,
    /// Tiles decoding now.
    pub in_flight: u32,
    /// Decoded tiles kept (resident or cached).
    pub cached: u32,
    pub cached_bytes: u64,
    /// Tiles decoded since the engine started, and the longest any frame
    /// spent on the engine's thread for them (ms).
    pub decoded: u64,
    pub worst_frame_ms: f64,
    /// Tiles handed to the renderer this frame.
    pub handed: u32,
}

struct Cached {
    tile: Arc<TerrainTile>,
    used: u64,
}

#[cfg(not(target_arch = "wasm32"))]
struct Worker {
    jobs: std::sync::mpsc::Sender<(Key, CookedTerrain)>,
    done: std::sync::mpsc::Receiver<(Key, Result<TerrainTile, String>)>,
}

#[cfg(not(target_arch = "wasm32"))]
impl Worker {
    fn new() -> Worker {
        let (jobs, rx) = std::sync::mpsc::channel::<(Key, CookedTerrain)>();
        let (tx, done) = std::sync::mpsc::channel();
        std::thread::Builder::new()
            .name("og-terrain-decode".into())
            .spawn(move || {
                while let Ok((key, terrain)) = rx.recv() {
                    let tile = terrain.tile(key.1).map_err(|e| e.to_string());
                    if tx.send((key, tile)).is_err() {
                        break;
                    }
                }
            })
            .expect("a decode thread");
        Worker { jobs, done }
    }
}

/// A tile the host is inflating: its keys, then its inflated parts.
struct Inflating {
    heights: u64,
    control: Option<u64>,
    got_heights: Option<Vec<u8>>,
    got_control: Option<Vec<u8>>,
    /// The host answered a part as failed. Kept across frames: the host
    /// forgets a key once it has answered it, so a failure seen while the
    /// frame's budget is spent is decoded locally on a later frame.
    failed: bool,
}

#[derive(Default)]
pub(crate) struct Streamer {
    frame: u64,
    cache: BTreeMap<Key, Cached>,
    in_flight: BTreeSet<Key>,
    inflating: BTreeMap<Key, Inflating>,
    next_key: u64,
    #[cfg(not(target_arch = "wasm32"))]
    worker: Option<Worker>,
    pub stats: StreamStats,
}

/// One streamed terrain drawn this frame.
pub(crate) struct Drawn<'a> {
    pub index: usize,
    pub render: TerrainHandle,
    pub cooked: &'a CookedTerrain,
    pub terrain: &'a og_render::Terrain,
    /// Metres around the camera where copies scattered over it need its finest tiles.
    pub detail: f32,
}

fn tile_bytes(t: &TerrainTile) -> u64 {
    (t.heights.len() * 4 + t.normals.len() * 2 + t.control.as_ref().map_or(0, |c| c.len())) as u64
}

impl Streamer {
    /// Forgets a terrain's tiles (its data changed: an edit or a restore).
    pub(crate) fn forget(&mut self, index: usize) {
        self.cache.retain(|k, _| k.0 != index);
        self.inflating.retain(|k, _| k.0 != index);
        self.in_flight.retain(|k| k.0 != index);
    }

    fn submit(
        &mut self,
        key: Key,
        cooked: &CookedTerrain,
        hooks: Option<&dyn super::Hooks>,
    ) -> bool {
        if let Some(h) = hooks
            && let Some((zh, zc)) = cooked.tile_zlib(key.1)
        {
            let hk = self.next_key;
            if h.inflate_start(hk, zh) {
                self.next_key += 1;
                let control = zc.map(|zc| {
                    let ck = self.next_key;
                    self.next_key += 1;
                    // A host that took the heights takes the control too.
                    let _ = h.inflate_start(ck, zc);
                    ck
                });
                self.inflating.insert(
                    key,
                    Inflating {
                        heights: hk,
                        control,
                        got_heights: None,
                        got_control: None,
                        failed: false,
                    },
                );
                self.in_flight.insert(key);
                return true;
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let w = self.worker.get_or_insert_with(Worker::new);
            if w.jobs.send((key, cooked.clone())).is_ok() {
                self.in_flight.insert(key);
                return true;
            }
        }
        false
    }

    fn store(&mut self, key: Key, tile: Result<TerrainTile, String>, notes: &mut Vec<String>) {
        self.in_flight.remove(&key);
        match tile {
            Ok(t) => {
                self.stats.decoded += 1;
                self.cache.insert(
                    key,
                    Cached {
                        tile: Arc::new(t),
                        used: self.frame,
                    },
                );
            }
            Err(e) => notes.push(format!("terrain tile {}: {e}", key.1)),
        }
    }

    /// Takes what finished decoding; with `wait`, blocks for every tile in
    /// flight. `finish` finishes host-inflated tiles on this thread (within
    /// [`FINISH_MS`] unless waiting).
    fn collect(
        &mut self,
        cooked: &IdMap<usize, CookedTerrain>,
        hooks: Option<&dyn super::Hooks>,
        wait: bool,
        notes: &mut Vec<String>,
    ) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut done = Vec::new();
            if let Some(w) = &self.worker {
                while let Ok(r) = w.done.try_recv() {
                    done.push(r);
                }
                if wait {
                    let native = self
                        .in_flight
                        .iter()
                        .filter(|k| !self.inflating.contains_key(k))
                        .count();
                    for _ in done.len()..native {
                        match w.done.recv() {
                            Ok(r) => done.push(r),
                            Err(_) => break,
                        }
                    }
                }
            }
            for (key, tile) in done {
                self.store(key, tile, notes);
            }
        }
        let start = hooks.map_or(0.0, |h| h.now_ms());
        let keys: Vec<Key> = self.inflating.keys().copied().collect();
        for key in keys {
            let Some(c) = cooked.get(&key.0) else {
                self.inflating.remove(&key);
                self.in_flight.remove(&key);
                continue;
            };
            let Some(h) = hooks else { break };
            let job = self.inflating.get_mut(&key).expect("listed");
            if !job.failed && job.got_heights.is_none() {
                match h.inflate_take(job.heights) {
                    super::Fetch::Ready(b) => job.got_heights = Some(b),
                    super::Fetch::Failed(_) => job.failed = true,
                    super::Fetch::Pending => {}
                }
            }
            // Taken even after the heights failed, so the host lets go of it.
            if let Some(ck) = job.control
                && job.got_control.is_none()
            {
                match h.inflate_take(ck) {
                    super::Fetch::Ready(b) => job.got_control = Some(b),
                    super::Fetch::Failed(_) => job.failed = true,
                    super::Fetch::Pending => {}
                }
            }
            let ready =
                job.got_heights.is_some() && (job.control.is_none() || job.got_control.is_some());
            let failed = job.failed;
            let over = !wait && h.now_ms() - start > FINISH_MS;
            if failed && !over {
                // The host could not: inflate it here.
                self.inflating.remove(&key);
                let tile = c.tile(key.1).map_err(|e| e.to_string());
                self.store(key, tile, notes);
            } else if ready && !over {
                let job = self.inflating.remove(&key).expect("listed");
                let tile = c
                    .tile_from(
                        key.1,
                        job.got_heights.as_deref().unwrap_or_default(),
                        job.got_control.as_deref(),
                    )
                    .map_err(|e| e.to_string());
                self.store(key, tile, notes);
            } else if wait {
                // A host that has not answered in time: decode it here.
                self.inflating.remove(&key);
                let tile = c.tile(key.1).map_err(|e| e.to_string());
                self.store(key, tile, notes);
            }
        }
    }

    /// One frame: what the views want of each streamed terrain, decoded or
    /// requested, and handed to the renderer (all at once and without a
    /// fade when `settle`).
    pub(crate) fn update(
        &mut self,
        drawn: &[Drawn],
        scene: &mut RenderScene,
        view: TerrainView,
        settle: bool,
        hooks: Option<&dyn super::Hooks>,
        notes: &mut Vec<String>,
    ) {
        self.frame += 1;
        let t0 = hooks.map_or(0.0, |h| h.now_ms());
        let cooked: IdMap<usize, CookedTerrain> =
            IdMap::from_iter(drawn.iter().map(|d| (d.index, d.cooked.clone())));
        self.collect(&cooked, hooks, false, notes);
        let mut handed = 0usize;
        let mut waiting = 0u32;
        let mut keep: BTreeSet<Key> = BTreeSet::new();
        let mut budget_tiles = 0usize;
        for d in drawn {
            let layout = d.cooked.layout;
            let capacity = (d.terrain.lod.budget() / layout.slot_bytes().max(1))
                .clamp(8, 256)
                .min(layout.tile_count() as u64) as usize;
            budget_tiles += capacity;
            let view = TerrainView {
                detail: d.detail,
                ..view
            };
            let wanted = og_render::terrain_tiles_wanted(
                &layout,
                d.terrain,
                d.cooked.range,
                &view,
                capacity,
            );
            // Ask for what is missing, best first.
            for &t in &wanted {
                let key = (d.index, t);
                if self.cache.contains_key(&key) || self.in_flight.contains(&key) {
                    continue;
                }
                if settle {
                    let tile = d.cooked.tile(t).map_err(|e| e.to_string());
                    self.store(key, tile, notes);
                } else if self.in_flight.len() < IN_FLIGHT && !self.submit(key, d.cooked, hooks) {
                    // No way to decode away from the frame: here, within the budget.
                    let over = hooks.is_some_and(|h| h.now_ms() - t0 > FINISH_MS);
                    if !over {
                        let tile = d.cooked.tile(t).map_err(|e| e.to_string());
                        self.store(key, tile, notes);
                    }
                }
            }
            if settle {
                self.collect(&cooked, hooks, true, notes);
            }
            // Hand over what has arrived, parents first.
            let held = scene.terrain_tiles(d.render).cloned().unwrap_or_default();
            let mut shown: BTreeSet<u32> = BTreeSet::new();
            for &t in &wanted {
                let key = (d.index, t);
                let parent_shown = layout.parent(t).is_none_or(|p| shown.contains(&p));
                let Some(c) = self.cache.get_mut(&key) else {
                    waiting += 1;
                    continue;
                };
                c.used = self.frame;
                keep.insert(key);
                if !parent_shown {
                    waiting += 1;
                    continue;
                }
                match held.get(&t) {
                    Some(h) if Arc::ptr_eq(h, &c.tile) => {
                        shown.insert(t);
                    }
                    _ if settle || handed < NEW_PER_FRAME => {
                        scene.set_terrain_tile(d.render, t, Some(c.tile.clone()));
                        shown.insert(t);
                        handed += 1;
                    }
                    _ => waiting += 1,
                }
            }
            for t in held.keys() {
                if !shown.contains(t) {
                    scene.set_terrain_tile(d.render, *t, None);
                }
            }
        }
        // The cache keeps twice the tile budgets, least recently used out first.
        let limit = budget_tiles * 2 + 16;
        if self.cache.len() > limit {
            let mut old: Vec<(u64, Key)> = self
                .cache
                .iter()
                .filter(|(k, _)| !keep.contains(k))
                .map(|(k, c)| (c.used, *k))
                .collect();
            og_core::sort::sort(&mut old);
            for (_, k) in old.into_iter().take(self.cache.len() - limit) {
                self.cache.remove(&k);
            }
        }
        let ms = hooks.map_or(0.0, |h| h.now_ms()) - t0;
        self.stats = StreamStats {
            resident: drawn
                .iter()
                .map(|d| scene.terrain_tiles(d.render).map_or(0, |t| t.len() as u32))
                .sum(),
            waiting,
            in_flight: self.in_flight.len() as u32,
            cached: self.cache.len() as u32,
            cached_bytes: self.cache.values().map(|c| tile_bytes(&c.tile)).sum(),
            decoded: self.stats.decoded,
            worst_frame_ms: self.stats.worst_frame_ms.max(ms),
            handed: handed as u32,
        };
    }
}
