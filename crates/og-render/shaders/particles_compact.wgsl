// Alive-particle compaction and indirect draw argument generation (compute):
// gathers only living, non-zero-size particles from the Mega-Pool into dense
// per-system vertex slices and writes DrawIndirect / DrawIndexedIndirect
// instance_count (word 1 of each 8-word slot). Count, scan tile counts in
// system order, then scatter: workgroup scheduling never changes blend order.
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
  first_tile: u32,
  end_tile: u32,
  live_count: u32,
  live_offset: u32,
};

@group(0) @binding(0) var<storage, read_write> compact_tiles: array<CompactTile>;
@group(0) @binding(1) var<storage, read> particles_pool: array<Particle>;
@group(0) @binding(2) var<storage, read> trails_pool: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read_write> compact_particles: array<Particle>;
@group(0) @binding(4) var<storage, read_write> compact_trails: array<vec4<f32>>;
@group(0) @binding(5) var<storage, read_write> indirect_args: array<atomic<u32>>;
@group(0) @binding(6) var<uniform> compact_info: vec4<u32>;  // x: tile_count

const TRAIL_POINTS: u32 = 8u;

var<workgroup> wg_prefix: array<u32, 64>;

fn prefix(alive: bool, l: u32) -> u32 {
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
  return wg_prefix[l];
}

@compute @workgroup_size(64)
fn count(@builtin(workgroup_id) wid: vec3<u32>, @builtin(local_invocation_id) lid: vec3<u32>) {
  if (wid.x >= compact_info.x) { return; }
  let tile = compact_tiles[wid.x];
  let slot = tile.slot_offset + lid.x;
  var alive = false;
  if (slot < tile.count) {
    let p = particles_pool[tile.src_base + slot];
    alive = (p.pos_age.w < p.vel_life.w) && (p.misc.x > 0.0);
  }
  prefix(alive, lid.x);
  if (lid.x == 0u) { compact_tiles[wid.x].live_count = wg_prefix[63]; }
}

// One invocation per system scans counts, not particles. The separate
// dispatch makes every tile count visible before any offset is read.
@compute @workgroup_size(1)
fn scan(@builtin(global_invocation_id) id: vec3<u32>) {
  if (id.x >= compact_info.x) { return; }
  if (id.x != compact_tiles[id.x].first_tile) { return; }
  let tile = compact_tiles[id.x];
  var total = 0u;
  for (var i = tile.first_tile; i < tile.end_tile; i++) {
    compact_tiles[i].live_offset = total;
    total += compact_tiles[i].live_count;
  }
  atomicStore(&indirect_args[tile.compact_idx * 8u + 1u], total);
  if (tile.compact_idx2 != 0xffffffffu) {
    atomicStore(&indirect_args[tile.compact_idx2 * 8u + 1u], total);
  }
}

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
  let local_end = prefix(alive, l);
  if (alive) {
    let local_idx = local_end - 1u;
    let out_slot = tile.dst_base + tile.live_offset + local_idx;
    compact_particles[out_slot] = p;
    if (tile.trail_src_base != 0xffffffffu) {
      let src_t = (tile.trail_src_base + slot) * TRAIL_POINTS;
      let dst_t = (tile.trail_dst_base + tile.live_offset + local_idx) * TRAIL_POINTS;
      for (var k = 0u; k < TRAIL_POINTS; k++) {
        compact_trails[dst_t + k] = trails_pool[src_t + k];
      }
    }
  }
}
