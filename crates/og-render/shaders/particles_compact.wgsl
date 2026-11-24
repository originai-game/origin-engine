// Alive-particle compaction and indirect draw argument generation (compute):
// gathers only living, non-zero-size particles from the Mega-Pool into dense
// per-system vertex slices and increments DrawIndirect / DrawIndexedIndirect
// instance_count (word 1 of each 8-word slot). Single-workgroup systems
// (<= 64 slots) preserve exact slot order via workgroup prefix scan.
// Prepended by particle_common.wgsl.

struct CompactTile {
  src_base: u32,
  dst_base: u32,
  trail_src_base: u32,
  trail_dst_base: u32,
  slot_offset: u32,
  count: u32,
  compact_idx: u32,
  compact_idx2: u32,    // optional paired trail indirect slot (0xffffffff: none)
};

@group(0) @binding(0) var<storage, read> compact_tiles: array<CompactTile>;
@group(0) @binding(1) var<storage, read> particles_pool: array<Particle>;
@group(0) @binding(2) var<storage, read> trails_pool: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read_write> compact_particles: array<Particle>;
@group(0) @binding(4) var<storage, read_write> compact_trails: array<vec4<f32>>;
@group(0) @binding(5) var<storage, read_write> indirect_args: array<atomic<u32>>;
@group(0) @binding(6) var<uniform> compact_info: vec4<u32>;  // x: tile_count

const TRAIL_POINTS: u32 = 8u;

var<workgroup> wg_prefix: array<u32, 64>;
var<workgroup> wg_base: u32;

@compute @workgroup_size(64)
fn compact(@builtin(workgroup_id) wid: vec3<u32>, @builtin(local_invocation_id) lid: vec3<u32>) {
  if (wid.x >= compact_info.x) { return; }
  let tile = compact_tiles[wid.x];
  let l = lid.x;
  let slot = tile.slot_offset + l;
  let valid = slot < tile.count;
  var p: Particle;
  var alive = false;
  if (valid) {
    p = particles_pool[tile.src_base + slot];
    alive = (p.pos_age.w < p.vel_life.w) && (p.misc.x > 0.0);
  }
  // Hillis-Steele inclusive prefix sum across the 64-slot workgroup.
  wg_prefix[l] = select(0u, 1u, alive);
  workgroupBarrier();
  for (var stride = 1u; stride < 64u; stride = stride << 1u) {
    var add = 0u;
    if (l >= stride) { add = wg_prefix[l - stride]; }
    workgroupBarrier();
    wg_prefix[l] += add;
    workgroupBarrier();
  }
  let total = wg_prefix[63];
  if (l == 0u) {
    if (total > 0u) {
      wg_base = atomicAdd(&indirect_args[tile.compact_idx * 8u + 1u], total);
      if (tile.compact_idx2 != 0xffffffffu) {
        atomicAdd(&indirect_args[tile.compact_idx2 * 8u + 1u], total);
      }
    } else {
      wg_base = 0u;
    }
  }
  workgroupBarrier();
  if (alive) {
    let local_idx = wg_prefix[l] - 1u;
    let out_slot = tile.dst_base + wg_base + local_idx;
    compact_particles[out_slot] = p;
    if (tile.trail_src_base != 0xffffffffu) {
      let src_t = (tile.trail_src_base + slot) * TRAIL_POINTS;
      let dst_t = (tile.trail_dst_base + wg_base + local_idx) * TRAIL_POINTS;
      for (var k = 0u; k < TRAIL_POINTS; k++) {
        compact_trails[dst_t + k] = trails_pool[src_t + k];
      }
    }
  }
}
