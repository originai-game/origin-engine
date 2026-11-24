//! Pooled GPU slab storage and cached bind groups for the batched particle
//! simulation, alive compaction and indirect draw paths.

use super::*;
use std::sync::Mutex;

/// Bytes between [`SimDispatchGpu`] entries (256-byte dynamic uniform stride).
pub(super) const SIM_DISPATCH_STRIDE: u64 = 256;
/// Bytes per indirect draw slot (padded to 32 bytes so both `DrawIndirectArgs`
/// and `DrawIndexedIndirectArgs` sit at `slot * 32` with `instance_count` at
/// word offset 1).
pub(super) const INDIRECT_STRIDE: u64 = 32;

/// First-fit coalescing slab allocator over element indices (`ribbons_gpu.rs`
/// `Ranges` pattern).
#[derive(Default)]
pub(super) struct SlabRanges {
    free: Vec<(u32, u32)>,
    pub(super) end: u32,
}

impl SlabRanges {
    pub(super) fn alloc(&mut self, len: u32) -> u32 {
        if len == 0 {
            return 0;
        }
        if let Some(i) = self.free.iter().position(|f| f.1 >= len) {
            let (base, l) = self.free[i];
            if l == len {
                self.free.remove(i);
            } else {
                self.free[i] = (base + len, l - len);
            }
            return base;
        }
        let base = self.end;
        self.end += len;
        base
    }

    pub(super) fn release(&mut self, base: u32, len: u32) {
        if len == 0 {
            return;
        }
        let mut i = self.free.partition_point(|f| f.0 < base);
        self.free.insert(i, (base, len));
        if i + 1 < self.free.len() && base + self.free[i].1 == self.free[i + 1].0 {
            self.free[i].1 += self.free[i + 1].1;
            self.free.remove(i + 1);
        }
        if i > 0 && self.free[i - 1].0 + self.free[i - 1].1 == self.free[i].0 {
            self.free[i - 1].1 += self.free[i].1;
            self.free.remove(i);
            i -= 1;
        }
        if i + 1 == self.free.len() && self.free[i].0 + self.free[i].1 == self.end {
            self.end = self.free[i].0;
            self.free.pop();
        }
    }

    pub(super) fn clear(&mut self) {
        self.free.clear();
        self.end = 0;
    }
}

/// Old pool buffers awaiting a GPU copy into their newly grown replacements
/// before the next simulation wave runs.
#[derive(Default)]
pub(super) struct PendingCopies {
    pub(super) particles: Option<wgpu::Buffer>,
    pub(super) trails: Option<wgpu::Buffer>,
    pub(super) sub_counters: Option<wgpu::Buffer>,
}

impl PendingCopies {
    pub(super) fn any(&self) -> bool {
        self.particles.is_some() || self.trails.is_some() || self.sub_counters.is_some()
    }
}

type SimFrameKey = (wgpu::TextureView, wgpu::TextureView, u64, u64, u64);
type SpriteCacheKey = (u64, u64, u64, wgpu::Buffer);

/// Cached bind groups rebuilt only when a referenced buffer grows or a texture
/// view changes (e.g. viewport resize).
#[derive(Default)]
pub(super) struct SimBindCache {
    pub(super) pending: PendingCopies,
    pub(super) sim_frame: Option<(SimFrameKey, wgpu::BindGroup)>,
    pub(super) sim_pool: Option<([u64; 6], wgpu::BindGroup)>,
    pub(super) compact: Option<([u64; 6], wgpu::BindGroup)>,
    pub(super) sprites: Option<(SpriteCacheKey, wgpu::BindGroup)>,
}

type DrawGroupKey = (wgpu::Buffer, wgpu::TextureView, wgpu::TextureView, u64);

/// Cached draw bind groups (main camera, secondary views, probe faces).
pub(super) type DrawGroupCache = Vec<(DrawGroupKey, wgpu::BindGroup)>;

/// One compute wave recorded by `prepare` for `simulate`.
#[derive(Clone, Copy, Debug)]
pub(super) enum SimWaveKind {
    Shift,
    Clear,
    Simulate,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct SimWave {
    pub(super) kind: SimWaveKind,
    pub(super) dispatch_slot: u32,
    pub(super) tile_count: u32,
}

/// Per-frame GPU simulation and compaction schedule built in `prepare`.
#[derive(Default)]
pub(super) struct FrameGpuPlan {
    pub(super) waves: Vec<SimWave>,
    pub(super) compact_tiles: u32,
}

fn make_buffer(
    device: &wgpu::Device,
    label: &str,
    size: u64,
    usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    crate::traffic::create_buffer(
        device,
        &wgpu::BufferDescriptor {
            label: Some(label),
            size: size.max(16),
            usage,
            mapped_at_creation: false,
        },
    )
}

/// Global Mega-Pool owning every GPU particle system's simulation slots, trail
/// rings, surface points, sub-emitter counters, compaction buffers and
/// indirect draw arguments.
pub(super) struct ParticlePool {
    // Persistent slab-allocated simulation buffers.
    pub(super) particle_ranges: SlabRanges,
    pub(super) particles: wgpu::Buffer,
    pub(super) particles_gen: u64,

    pub(super) trail_ranges: SlabRanges,
    pub(super) trails: wgpu::Buffer,
    pub(super) trails_gen: u64,

    pub(super) points_ranges: SlabRanges,
    pub(super) points: wgpu::Buffer,
    pub(super) points_gen: u64,
    pub(super) points_shadow: Vec<[f32; 4]>,
    pub(super) points_dirty: bool,

    pub(super) counter_ranges: SlabRanges,
    pub(super) sub_counters: wgpu::Buffer,
    pub(super) sub_counters_gen: u64,

    // Per-frame simulation descriptor / wave tile buffers.
    pub(super) sim_entries: wgpu::Buffer,
    pub(super) sim_entries_gen: u64,
    pub(super) sim_tiles: wgpu::Buffer,
    pub(super) sim_tiles_gen: u64,
    pub(super) sim_dispatch: wgpu::Buffer,
    pub(super) sim_dispatch_gen: u64,

    // Per-frame alive-compaction and indirect-draw buffers.
    pub(super) compact_tiles: wgpu::Buffer,
    pub(super) compact_tiles_gen: u64,
    pub(super) compact_particles: wgpu::Buffer,
    pub(super) compact_particles_gen: u64,
    pub(super) compact_trails: wgpu::Buffer,
    pub(super) compact_trails_gen: u64,
    pub(super) compact_info: wgpu::Buffer,
    pub(super) indirect_args: wgpu::Buffer,
    pub(super) indirect_args_gen: u64,

    // Scratch CPU buffers reused across frames (zero steady-state alloc).
    pub(super) sim_entries_cpu: Vec<EmitterSimGpu>,
    pub(super) sim_tiles_cpu: Vec<[u32; 4]>,
    pub(super) sim_dispatch_bytes: Vec<u8>,
    pub(super) compact_tiles_cpu: Vec<CompactTileGpu>,
    pub(super) indirect_init_cpu: Vec<[u32; 8]>,
    pub(super) sprite_bytes: Vec<u8>,

    pub(super) cache: Mutex<SimBindCache>,
}

impl ParticlePool {
    pub(super) fn new(device: &wgpu::Device) -> Self {
        let storage_rw = wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::COPY_DST;
        let storage_ro = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;
        ParticlePool {
            particle_ranges: SlabRanges::default(),
            particles: make_buffer(
                device,
                "particles pool",
                PARTICLE_STRIDE * 256,
                storage_rw | wgpu::BufferUsages::VERTEX,
            ),
            particles_gen: 1,
            trail_ranges: SlabRanges::default(),
            trails: make_buffer(
                device,
                "particle trails pool",
                16 * TRAIL_POINTS as u64 * 64,
                storage_rw | wgpu::BufferUsages::VERTEX,
            ),
            trails_gen: 1,
            points_ranges: SlabRanges::default(),
            points: make_buffer(device, "particle points pool", 16 * 64, storage_ro),
            points_gen: 1,
            points_shadow: Vec::new(),
            points_dirty: false,
            counter_ranges: SlabRanges::default(),
            sub_counters: make_buffer(device, "particle sub counters", 4 * 64, storage_rw),
            sub_counters_gen: 1,
            sim_entries: make_buffer(
                device,
                "particle sim entries",
                std::mem::size_of::<EmitterSimGpu>() as u64 * 16,
                storage_ro,
            ),
            sim_entries_gen: 1,
            sim_tiles: make_buffer(device, "particle sim tiles", 16 * 64, storage_ro),
            sim_tiles_gen: 1,
            sim_dispatch: make_buffer(
                device,
                "particle sim dispatch",
                SIM_DISPATCH_STRIDE * 8,
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            ),
            sim_dispatch_gen: 1,
            compact_tiles: make_buffer(
                device,
                "particle compact tiles",
                std::mem::size_of::<CompactTileGpu>() as u64 * 64,
                storage_ro,
            ),
            compact_tiles_gen: 1,
            compact_particles: make_buffer(
                device,
                "particles compact",
                PARTICLE_STRIDE * 256,
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX,
            ),
            compact_particles_gen: 1,
            compact_trails: make_buffer(
                device,
                "particle trails compact",
                16 * TRAIL_POINTS as u64 * 64,
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX,
            ),
            compact_trails_gen: 1,
            compact_info: make_buffer(
                device,
                "particle compact info",
                16,
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            ),
            indirect_args: make_buffer(
                device,
                "particle indirect args",
                INDIRECT_STRIDE * 64,
                wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::INDIRECT
                    | wgpu::BufferUsages::COPY_DST,
            ),
            indirect_args_gen: 1,
            sim_entries_cpu: Vec::new(),
            sim_tiles_cpu: Vec::new(),
            sim_dispatch_bytes: Vec::new(),
            compact_tiles_cpu: Vec::new(),
            indirect_init_cpu: Vec::new(),
            sprite_bytes: Vec::new(),
            cache: Mutex::new(SimBindCache::default()),
        }
    }

    pub(super) fn reset_allocations(&mut self) {
        self.particle_ranges.clear();
        self.trail_ranges.clear();
        self.points_ranges.clear();
        self.counter_ranges.clear();
        self.points_shadow.clear();
        self.points_dirty = false;
        let mut cache = self.cache.lock().expect("particle sim cache");
        cache.pending = PendingCopies::default();
    }

    /// Flushes any grow-copies left over from an aborted frame that never ran
    /// `simulate` so persistent particle state is never lost.
    pub(super) fn flush_pending_copies(&self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let pending = {
            let mut cache = self.cache.lock().expect("particle sim cache");
            if !cache.pending.any() {
                return;
            }
            std::mem::take(&mut cache.pending)
        };
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("particle pool grow copy"),
        });
        self.encode_copies(&mut encoder, &pending);
        queue.submit(Some(encoder.finish()));
    }

    pub(super) fn encode_copies(&self, encoder: &mut wgpu::CommandEncoder, p: &PendingCopies) {
        if let Some(old) = &p.particles {
            encoder.copy_buffer_to_buffer(old, 0, &self.particles, 0, old.size());
        }
        if let Some(old) = &p.trails {
            encoder.copy_buffer_to_buffer(old, 0, &self.trails, 0, old.size());
        }
        if let Some(old) = &p.sub_counters {
            encoder.copy_buffer_to_buffer(old, 0, &self.sub_counters, 0, old.size());
        }
    }

    /// Ensures the persistent particle, trail and sub-counter pools cover all
    /// live allocations, recording grow-copies for `simulate` when a buffer grows.
    pub(super) fn ensure_persistent(&mut self, device: &wgpu::Device) {
        let storage_rw = wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::COPY_DST;
        let need_p = u64::from(self.particle_ranges.end) * PARTICLE_STRIDE;
        if need_p > self.particles.size() {
            let grown = make_buffer(
                device,
                "particles pool",
                need_p.next_power_of_two(),
                storage_rw | wgpu::BufferUsages::VERTEX,
            );
            let old = std::mem::replace(&mut self.particles, grown);
            self.particles_gen += 1;
            self.cache
                .lock()
                .expect("particle sim cache")
                .pending
                .particles = Some(old);
        }
        let need_t = u64::from(self.trail_ranges.end) * TRAIL_POINTS as u64 * 16;
        if need_t > self.trails.size() {
            let grown = make_buffer(
                device,
                "particle trails pool",
                need_t.next_power_of_two(),
                storage_rw | wgpu::BufferUsages::VERTEX,
            );
            let old = std::mem::replace(&mut self.trails, grown);
            self.trails_gen += 1;
            self.cache
                .lock()
                .expect("particle sim cache")
                .pending
                .trails = Some(old);
        }
        let need_c = u64::from(self.counter_ranges.end) * 4;
        if need_c > self.sub_counters.size() {
            let grown = make_buffer(
                device,
                "particle sub counters",
                need_c.next_power_of_two(),
                storage_rw,
            );
            let old = std::mem::replace(&mut self.sub_counters, grown);
            self.sub_counters_gen += 1;
            self.cache
                .lock()
                .expect("particle sim cache")
                .pending
                .sub_counters = Some(old);
        }
    }

    pub(super) fn upload_points(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if !self.points_dirty || self.points_shadow.is_empty() {
            return;
        }
        let bytes: &[u8] = bytemuck::cast_slice(&self.points_shadow);
        if bytes.len() as u64 > self.points.size() {
            self.points = make_buffer(
                device,
                "particle points pool",
                (bytes.len() as u64).next_power_of_two(),
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            );
            self.points_gen += 1;
        }
        crate::traffic::write_buffer(queue, &self.points, 0, bytes);
        self.points_dirty = false;
    }

    pub(super) fn upload_sim_work(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let storage_ro = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;
        if !self.sim_entries_cpu.is_empty() {
            let bytes: &[u8] = bytemuck::cast_slice(&self.sim_entries_cpu);
            if bytes.len() as u64 > self.sim_entries.size() {
                self.sim_entries = make_buffer(
                    device,
                    "particle sim entries",
                    (bytes.len() as u64).next_power_of_two(),
                    storage_ro,
                );
                self.sim_entries_gen += 1;
            }
            crate::traffic::write_buffer(queue, &self.sim_entries, 0, bytes);
        }
        if !self.sim_tiles_cpu.is_empty() {
            let bytes: &[u8] = bytemuck::cast_slice(&self.sim_tiles_cpu);
            if bytes.len() as u64 > self.sim_tiles.size() {
                self.sim_tiles = make_buffer(
                    device,
                    "particle sim tiles",
                    (bytes.len() as u64).next_power_of_two(),
                    storage_ro,
                );
                self.sim_tiles_gen += 1;
            }
            crate::traffic::write_buffer(queue, &self.sim_tiles, 0, bytes);
        }
        if !self.sim_dispatch_bytes.is_empty() {
            let size = self.sim_dispatch_bytes.len() as u64;
            if size > self.sim_dispatch.size() {
                self.sim_dispatch = make_buffer(
                    device,
                    "particle sim dispatch",
                    size.next_power_of_two(),
                    wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                );
                self.sim_dispatch_gen += 1;
            }
            crate::traffic::write_buffer(queue, &self.sim_dispatch, 0, &self.sim_dispatch_bytes);
        }
    }

    pub(super) fn upload_compact_work(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        compact_particle_slots: u32,
        compact_trail_slots: u32,
    ) {
        if self.compact_tiles_cpu.is_empty() {
            return;
        }
        let tile_bytes: &[u8] = bytemuck::cast_slice(&self.compact_tiles_cpu);
        if tile_bytes.len() as u64 > self.compact_tiles.size() {
            self.compact_tiles = make_buffer(
                device,
                "particle compact tiles",
                (tile_bytes.len() as u64).next_power_of_two(),
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            );
            self.compact_tiles_gen += 1;
        }
        crate::traffic::write_buffer(queue, &self.compact_tiles, 0, tile_bytes);

        let need_p = u64::from(compact_particle_slots) * PARTICLE_STRIDE;
        if need_p > self.compact_particles.size() {
            self.compact_particles = make_buffer(
                device,
                "particles compact",
                need_p.next_power_of_two(),
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX,
            );
            self.compact_particles_gen += 1;
        }
        let need_t = u64::from(compact_trail_slots) * TRAIL_POINTS as u64 * 16;
        if need_t > self.compact_trails.size() {
            self.compact_trails = make_buffer(
                device,
                "particle trails compact",
                need_t.next_power_of_two(),
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX,
            );
            self.compact_trails_gen += 1;
        }
        let args_bytes: &[u8] = bytemuck::cast_slice(&self.indirect_init_cpu);
        if args_bytes.len() as u64 > self.indirect_args.size() {
            self.indirect_args = make_buffer(
                device,
                "particle indirect args",
                (args_bytes.len() as u64).next_power_of_two(),
                wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::INDIRECT
                    | wgpu::BufferUsages::COPY_DST,
            );
            self.indirect_args_gen += 1;
        }
        crate::traffic::write_buffer(queue, &self.indirect_args, 0, args_bytes);
        let info = [self.compact_tiles_cpu.len() as u32, 0, 0, 0];
        crate::traffic::write_buffer(queue, &self.compact_info, 0, bytemuck::cast_slice(&info));
    }
}
