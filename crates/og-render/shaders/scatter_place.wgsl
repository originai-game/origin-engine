// Scatter placement on the GPU (scatter_place.rs): a TerrainScatter with
// `gpu` set has its chunks' copies placed here, once per chunk as the CPU
// path builds them, by the same rules as scatter.rs `Terrain::copy_ruled`
// (the layer's weight, holes, clusters, the water and the footprints of
// the scatters placed before it), into the chunk's instance buffer, which
// scatter_cull.wgsl then culls copy by copy. Prepended by wind_common.wgsl,
// grass_common.wgsl and terrain.wgsl (group 1: the terrain's uniform, tile
// arrays and table); the ground is read from the level-0 tiles alone, the
// samples the CPU path reads, never mixed with a coarser level.
//
// The footprint rule recurses: a copy keeps clear of the copies of each
// scatter placed before it, which themselves kept clear of the ones before
// them. A job evaluates it in dependency order, without readback or
// recursion (the layered placement of van Muijden, *GPU-Based Procedural
// Placement in Horizon Zero Dawn*, GDC 2017, with exact distances in place
// of density maps): `evaluate` runs once per earlier scatter (a stage),
// first placed first, over the cells of its grid that a later stage or the
// chunk can reach, and leaves one bit per cell, whether its copy grows
// under every rule; a later stage or `place` reads those bits, a copy's
// place following from its cell's hash alone. `place` then places the
// chunk's cells, compacted in the CPU's order (rows, then columns) by a
// prefix sum, and writes the chunk's two draws after its copies.

// scatter_place.rs LayerGpu: one scatter's placement.
struct PlaceLayer {
  shape: vec4<f32>,   // x: the grid's cell (the terrain's space), y, z: the scale's range, w: align
  rules: vec4<f32>,   // x: noise scale (m), y: noise threshold, z: footprint (m), w: avoid water (m)
  grid: vec4<u32>,    // x: the seed's hash, y: cells along x, z: cells along z, w: the layer
  region: vec4<u32>,  // x, y: its first evaluated cell (column, row), z, w: cells along x and z
  bits: vec4<u32>,    // x: its first word in `bits`, y: 1 when evaluated
};

// scatter_place.rs JobGpu: one chunk's placement, one row per dispatch.
struct PlaceJob {
  model: mat4x4<f32>,    // the terrain's space to the frame's
  normal: mat4x4<f32>,   // normals from the terrain's space to the frame's (inverse transpose)
  ground: vec4<f32>,     // x: the terrain's horizontal scale (world m per its metre), y: waters
  counts: vec4<u32>,     // x: layers (the stages, then the placed scatter), y: the layer this dispatch evaluates, z: the mesh's index count
  layers: array<PlaceLayer, 8>,
  water: array<mat4x4<f32>, 8>,  // the frame's to each surface's (its surface local y = 0)
  water_half: array<vec4<f32>, 8>,  // xy: each one's half extent along local x and z
};

@group(0) @binding(0) var<uniform> job: PlaceJob;
@group(0) @binding(1) var<storage, read_write> bits: array<atomic<u32>>;
// The chunk's copies (instances.rs GpuTransform, four vectors each), then
// at PLACE_TAIL its draws: an indexed indirect draw and a plain one.
@group(2) @binding(0) var<storage, read_write> out: array<vec4<f32>>;

const PLACE_LANES: u32 = 256u;
const PLACE_PER_LANE: u32 = 4u;
// scatter_cull.rs MAX_CHUNK_COPIES: the chunk's draws' first vector.
const PLACE_TAIL: u32 = 4096u;
const PLACE_TAU: f32 = 6.283185307179586;

// scatter.rs `hash` (lowbias32) and `unit`.
fn place_hash(x: u32) -> u32 {
  var h = x;
  h ^= h >> 16u;
  h *= 0x7feb352du;
  h ^= h >> 15u;
  h *= 0x846ca68bu;
  return h ^ (h >> 16u);
}

fn place_unit(x: u32) -> f32 {
  return f32(x >> 8u) / 16777216.0;
}

// The ground's raw level-0 height sample (column `c`, row `r`, clamped to
// the field), its lowest bit the hole flag of the cell at it with holes.
fn place_raw(c: u32, r: u32) -> f32 {
  let last = vec2<u32>(terrain.extent.zw) - 1u;
  let u = vec2<f32>(min(vec2<u32>(c, r), last));
  let t = terrain.tiles.x;
  let tr = tile_ref(0u, u);
  if (tr.slot < 0.0) { return 0.0; }
  let l = clamp(vec2<i32>(u - tr.tile * t) + 1, vec2<i32>(0), vec2<i32>(i32(t) + 2));
  return height_word(l, i32(tr.slot));
}

fn place_sample(c: u32, r: u32) -> f32 {
  let h = place_raw(c, r);
  if (terrain.params.z > 0.5) { return bitcast<f32>(bitcast<u32>(h) & ~1u); }
  return h;
}

// terrain.rs TerrainSampler::grid: sample coordinates of `p`.
fn place_grid(p: vec2<f32>) -> vec2<f32> {
  let t = (p + terrain.extent.xy * 0.5) / terrain_cell();
  return clamp(t, vec2<f32>(0.0), terrain.extent.zw - 1.0);
}

// TerrainSampler::height: the triangle under `p`, split along the
// diagonal og_core::math::heightfield_zigzag picks.
fn place_height(p: vec2<f32>) -> f32 {
  let t = place_grid(p);
  let i = min(floor(t), terrain.extent.zw - 2.0);
  let f = clamp(t - i, vec2<f32>(0.0), vec2<f32>(1.0));
  let c = u32(i.x);
  let r = u32(i.y);
  let h00 = place_sample(c, r);
  let h10 = place_sample(c + 1u, r);
  let h01 = place_sample(c, r + 1u);
  let h11 = place_sample(c + 1u, r + 1u);
  if (!(abs(h00 - h11) <= abs(h10 - h01))) {
    if (f.x + f.y <= 1.0) {
      return h00 + (h10 - h00) * f.x + (h01 - h00) * f.y;
    }
    return h11 + (h01 - h11) * (1.0 - f.x) + (h10 - h11) * (1.0 - f.y);
  }
  if (f.y >= f.x) {
    return h00 + (h11 - h01) * f.x + (h01 - h00) * f.y;
  }
  return h00 + (h10 - h00) * f.x + (h11 - h10) * f.y;
}

// TerrainSampler::sample_normal: central differences at a sample.
fn place_sample_normal(c: u32, r: u32) -> vec3<f32> {
  let cell = terrain_cell();
  let last = vec2<u32>(terrain.extent.zw) - 1u;
  let c0 = select(c - 1u, 0u, c == 0u);
  let c1 = min(c + 1u, last.x);
  let r0 = select(r - 1u, 0u, r == 0u);
  let r1 = min(r + 1u, last.y);
  let hx = (place_sample(c1, r) - place_sample(c0, r)) / (f32(max(c1 - c0, 1u)) * cell.x);
  let hz = (place_sample(c, r1) - place_sample(c, r0)) / (f32(max(r1 - r0, 1u)) * cell.y);
  return normalize(vec3<f32>(-hx, 1.0, -hz));
}

// TerrainSampler::normal: the samples' normals, bilinear.
fn place_normal(p: vec2<f32>) -> vec3<f32> {
  let t = place_grid(p);
  let i = min(floor(t), terrain.extent.zw - 2.0);
  let f = clamp(t - i, vec2<f32>(0.0), vec2<f32>(1.0));
  let c = u32(i.x);
  let r = u32(i.y);
  let lo = mix(place_sample_normal(c, r), place_sample_normal(c + 1u, r), f.x);
  let hi = mix(place_sample_normal(c, r + 1u), place_sample_normal(c + 1u, r + 1u), f.x);
  let n = mix(lo, hi, f.y);
  if (dot(n, n) < 1e-12) { return vec3<f32>(0.0, 1.0, 0.0); }
  return normalize(n);
}

// scatter.rs TileGround::hole: the flag of the cell under `p`.
fn place_hole(p: vec2<f32>) -> bool {
  if (terrain.params.z < 0.5) { return false; }
  let t = (p + terrain.extent.xy * 0.5) / terrain_cell();
  let n = terrain.extent.zw - 1.0;
  if (!(t.x >= 0.0 && t.y >= 0.0 && t.x <= n.x && t.y <= n.y)) { return false; }
  let c = min(u32(t.x), u32(terrain.extent.z) - 2u);
  let r = min(u32(t.y), u32(terrain.extent.w) - 2u);
  return (bitcast<u32>(place_raw(c, r)) & 1u) != 0u;
}

// scatter.rs TileGround::control: channel `k` at `p`, bilinear between the
// level-0 tile's texels (its border at 0), 1 past its pages.
fn place_control(p: vec2<f32>, k: u32) -> f32 {
  let pages = max(u32(terrain.tiles.w), 1u);
  if (terrain.params.y < 0.5 || k / 4u >= pages) { return 1.0; }
  let t = terrain.tiles.x;
  let s = (p + terrain.extent.xy * 0.5) / terrain_cell();
  let tr = tile_ref(0u, clamp(s, vec2<f32>(0.0), terrain.extent.zw - 1.0));
  if (tr.slot < 0.0) { return 1.0; }
  let c = terrain.tiles.y;
  let side = i32(c) + 2;
  let g = (s - tr.tile * t) / t * c + 0.5;
  let i = floor(g);
  let f = g - i;
  let layer = i32(tr.slot) * i32(pages) + i32(k / 4u);
  let a = vec2<i32>(i);
  let x0 = clamp(a.x, 0, side - 1);
  let x1 = clamp(a.x + 1, 0, side - 1);
  let y0 = clamp(a.y, 0, side - 1);
  let y1 = clamp(a.y + 1, 0, side - 1);
  let lane = k % 4u;
  let t00 = textureLoad(control_map, vec2<i32>(x0, y0), layer, 0)[lane];
  let t10 = textureLoad(control_map, vec2<i32>(x1, y0), layer, 0)[lane];
  let t01 = textureLoad(control_map, vec2<i32>(x0, y1), layer, 0)[lane];
  let t11 = textureLoad(control_map, vec2<i32>(x1, y1), layer, 0)[lane];
  let lo = t00 + (t10 - t00) * f.x;
  let hi = t01 + (t11 - t01) * f.x;
  return lo + (hi - lo) * f.y;
}

// scatter.rs `band` (terrain_lit.wgsl's).
fn place_band(x: f32, lo: f32, hi: f32, fade: f32) -> f32 {
  let f = max(fade, 1e-4);
  return smoothstep(lo - f, lo, x) * (1.0 - smoothstep(hi, hi + f, x));
}

// scatter.rs Terrain::weights_on: layer `layer`'s weight at `p`, its
// control channel times its slope and altitude rules over every layer's;
// the first layer where none shows.
fn place_weight(p: vec2<f32>, n: vec3<f32>, altitude: f32, layer: u32) -> f32 {
  let count = min(u32(terrain.origin.w), 16u);
  let angle = degrees(acos(clamp(n.y, -1.0, 1.0)));
  var sum = 0.0;
  var mine = 0.0;
  for (var i = 0u; i < count; i++) {
    let l = terrain.layers[i];
    let w = place_control(p, i) * place_band(angle, l.slope.x, l.slope.y, l.slope.z)
      * place_band(altitude, l.altitude.x, l.altitude.y, l.altitude.z);
    sum += w;
    if (i == layer) { mine = w; }
  }
  if (sum < 1e-4) { return select(0.0, 1.0, layer == 0u); }
  return mine / sum;
}

// scatter.rs `cluster_noise`: value noise of three octaves turned off the
// axes, 0..1.
fn place_lattice(q: vec2<f32>, salt: u32, seed: u32) -> f32 {
  let i = floor(q);
  let f = q - i;
  let u = f * f * (vec2<f32>(3.0) - 2.0 * f);
  let ix = i32(i.x);
  let iz = i32(i.y);
  var v: array<f32, 4>;
  for (var k = 0u; k < 4u; k++) {
    let x = bitcast<u32>(ix + i32(k & 1u));
    let z = bitcast<u32>(iz + i32(k >> 1u));
    v[k] = place_unit(place_hash((place_hash(x ^ salt) + z * 0x9e3779b9u) ^ seed));
  }
  let a = v[0] + (v[1] - v[0]) * u.x;
  let b = v[2] + (v[3] - v[2]) * u.x;
  return a + (b - a) * u.y;
}

fn place_cluster_noise(p: vec2<f32>, seed: u32) -> f32 {
  let c = cos(0.6);
  let s = sin(0.6);
  var q = p;
  var amp = 1.0;
  var sum = 0.0;
  var total = 0.0;
  for (var octave = 0u; octave < 3u; octave++) {
    sum += amp * place_lattice(q, octave * 0x632be5abu, seed);
    total += amp;
    q = vec2<f32>(c * q.x - s * q.y, s * q.x + c * q.y) * 2.03;
    amp *= 0.5;
  }
  return sum / total;
}

// Cell (x, z) of layer `li`'s grid: its hash and its place in the
// terrain's space (scatter.rs Terrain::copy_at).
struct PlaceCell {
  h0: u32,
  p: vec2<f32>,
};

fn place_cell(li: u32, x: u32, z: u32) -> PlaceCell {
  let l = job.layers[li];
  let key = z * l.grid.y + x;
  let h0 = place_hash(key ^ l.grid.x);
  var c: PlaceCell;
  c.h0 = h0;
  c.p = vec2<f32>(f32(x) + place_unit(place_hash(h0)), f32(z) + place_unit(place_hash(h0 ^ 1u))) * l.shape.x
    - terrain.extent.xy * 0.5;
  return c;
}

// Whether a copy of layer `li` stands in evaluated cell (x, z).
fn place_grown(li: u32, x: u32, z: u32) -> bool {
  let r = job.layers[li].region;
  if (x < r.x || z < r.y || x >= r.x + r.z || z >= r.y + r.w) { return false; }
  let k = (z - r.y) * r.z + (x - r.x);
  let word = atomicLoad(&bits[job.layers[li].bits.x + k / 32u]);
  return (word & (1u << (k % 32u))) != 0u;
}

// A copy of layer `li` grows at `c` (its ground `height`, the frame's
// `at`) under the water and the earlier layers' footprints.
fn place_clear(li: u32, c: PlaceCell, at: vec3<f32>) -> bool {
  let l = job.layers[li];
  if (l.rules.w > 0.0) {
    for (var w = 0u; w < u32(job.ground.y); w++) {
      let q = (job.water[w] * vec4<f32>(at, 1.0)).xyz;
      let half = job.water_half[w].xy;
      if (abs(q.x) <= half.x && abs(q.z) <= half.y && q.y < l.rules.w) { return false; }
    }
  }
  let along = max(job.ground.x, 1e-6);
  let half = terrain.extent.xy * 0.5;
  for (var j = 0u; j < li; j++) {
    let b = job.layers[j];
    let reach = max(l.rules.z, b.rules.z) / along;
    if (reach <= 0.0 || b.bits.y == 0u) { continue; }
    let cells = vec2<f32>(b.grid.yz);
    let lo = max(floor((c.p - reach + half) / b.shape.x), vec2<f32>(0.0));
    let hi = min(floor((c.p + reach + half) / b.shape.x), cells - 1.0);
    if (any(lo > hi)) { continue; }
    for (var bz = u32(lo.y); bz <= u32(hi.y); bz++) {
      for (var bx = u32(lo.x); bx <= u32(hi.x); bx++) {
        if (place_grown(j, bx, bz) && length(place_cell(j, bx, bz).p - c.p) < reach) {
          return false;
        }
      }
    }
  }
  return true;
}

// Cell (x, z) of layer `li` under every rule: its copy's transform's
// columns (xyz, and w its scale) where it grows, else a zero scale.
struct PlaceCopy {
  grows: bool,
  columns: array<vec3<f32>, 4>,
};

fn place_copy(li: u32, x: u32, z: u32, transform: bool) -> PlaceCopy {
  var o: PlaceCopy;
  o.grows = false;
  let l = job.layers[li];
  let c = place_cell(li, x, z);
  let half = terrain.extent.xy * 0.5;
  if (c.p.x > half.x || c.p.y > half.y) { return o; }
  let n = place_normal(c.p);
  let height = place_height(c.p);
  if (place_unit(place_hash(c.h0 ^ 2u)) >= place_weight(c.p, n, height, l.grid.w) || place_hole(c.p)) {
    return o;
  }
  if (l.rules.y > 0.0) {
    let v = place_cluster_noise(c.p * job.ground.x / max(l.rules.x, 1e-3), l.grid.x);
    let edge = smoothstep(l.rules.y - 0.04, l.rules.y + 0.04, v);
    if (place_unit(place_hash(c.h0 ^ 5u)) >= edge) { return o; }
  }
  let at = (job.model * vec4<f32>(c.p.x, height, c.p.y, 1.0)).xyz;
  if (!place_clear(li, c, at)) { return o; }
  o.grows = true;
  if (!transform) { return o; }
  // Upright, leaning `align` of the way to the ground's normal, turned
  // about it by the cell's hash (glam's Quat::from_rotation_arc from +y,
  // then the turn about y).
  let size = l.shape.y + (l.shape.z - l.shape.y) * place_unit(place_hash(c.h0 ^ 3u));
  var wn = (job.normal * vec4<f32>(n, 0.0)).xyz;
  wn = select(vec3<f32>(0.0, 1.0, 0.0), normalize(wn), dot(wn, wn) > 1e-12);
  var up = mix(vec3<f32>(0.0, 1.0, 0.0), wn, clamp(l.shape.w, 0.0, 1.0));
  up = select(vec3<f32>(0.0, 1.0, 0.0), normalize(up), dot(up, up) > 1e-12);
  var arc = vec4<f32>(0.0, 0.0, 0.0, 1.0);
  if (up.y < -1.0 + 1e-6) {
    arc = vec4<f32>(1.0, 0.0, 0.0, 0.0);
  } else if (up.y <= 1.0 - 1e-6) {
    arc = normalize(vec4<f32>(up.z, 0.0, -up.x, 1.0 + up.y));
  }
  let turn = place_unit(place_hash(c.h0 ^ 4u)) * PLACE_TAU * 0.5;
  let ty = vec4<f32>(0.0, sin(turn), 0.0, cos(turn));
  // The product arc · ty.
  let q = vec4<f32>(
    arc.w * ty.x + arc.x * ty.w + arc.y * ty.z - arc.z * ty.y,
    arc.w * ty.y - arc.x * ty.z + arc.y * ty.w + arc.z * ty.x,
    arc.w * ty.z + arc.x * ty.y - arc.y * ty.x + arc.z * ty.w,
    arc.w * ty.w - arc.x * ty.x - arc.y * ty.y - arc.z * ty.z);
  let x2 = q.x + q.x;
  let y2 = q.y + q.y;
  let z2 = q.z + q.z;
  let xx = q.x * x2;
  let xy = q.x * y2;
  let xz = q.x * z2;
  let yy = q.y * y2;
  let yz = q.y * z2;
  let zz = q.z * z2;
  let wx = q.w * x2;
  let wy = q.w * y2;
  let wz = q.w * z2;
  o.columns[0] = vec3<f32>(1.0 - (yy + zz), xy + wz, xz - wy) * size;
  o.columns[1] = vec3<f32>(xy - wz, 1.0 - (xx + zz), yz + wx) * size;
  o.columns[2] = vec3<f32>(xz + wy, yz - wx, 1.0 - (xx + yy)) * size;
  o.columns[3] = at;
  return o;
}

// One stage: whether each of the layer's evaluated cells grows, a bit each.
@compute @workgroup_size(64)
fn evaluate(@builtin(global_invocation_id) id: vec3<u32>) {
  let li = job.counts.y;
  let r = job.layers[li].region;
  let k = id.x;
  if (k >= r.z * r.w) { return; }
  if (place_copy(li, r.x + k % r.z, r.y + k / r.z, false).grows) {
    atomicOr(&bits[job.layers[li].bits.x + k / 32u], 1u << (k % 32u));
  }
}

var<workgroup> lane_counts: array<u32, PLACE_LANES>;

// The chunk: its copies, compacted in order, and its draws.
@compute @workgroup_size(256)
fn place(@builtin(local_invocation_index) lane: u32) {
  let li = job.counts.y;
  let r = job.layers[li].region;
  let n = r.z * r.w;
  var copies: array<PlaceCopy, PLACE_PER_LANE>;
  var mine = 0u;
  for (var j = 0u; j < PLACE_PER_LANE; j++) {
    let k = lane * PLACE_PER_LANE + j;
    copies[j].grows = false;
    if (k < n) {
      copies[j] = place_copy(li, r.x + k % r.z, r.y + k / r.z, true);
      if (copies[j].grows) { mine += 1u; }
    }
  }
  lane_counts[lane] = mine;
  workgroupBarrier();
  // An inclusive scan over the lanes (Hillis and Steele).
  for (var step = 1u; step < PLACE_LANES; step <<= 1u) {
    var add = 0u;
    if (lane >= step) { add = lane_counts[lane - step]; }
    workgroupBarrier();
    lane_counts[lane] += add;
    workgroupBarrier();
  }
  var at = lane_counts[lane] - mine;
  for (var j = 0u; j < PLACE_PER_LANE; j++) {
    if (copies[j].grows) {
      let c = copies[j].columns;
      // instances.rs GpuTransform: the columns' xyz, layer 0, no look.
      out[at * 4u] = vec4<f32>(c[0], c[1].x);
      out[at * 4u + 1u] = vec4<f32>(c[1].yz, c[2].xy);
      out[at * 4u + 2u] = vec4<f32>(c[2].z, c[3]);
      out[at * 4u + 3u] = vec4<f32>(0.0);
      at += 1u;
    }
  }
  if (lane == PLACE_LANES - 1u) {
    let total = bitcast<f32>(lane_counts[lane]);
    out[PLACE_TAIL] = vec4<f32>(bitcast<f32>(job.counts.z), total, 0.0, 0.0);
    out[PLACE_TAIL + 1u] = vec4<f32>(0.0, bitcast<f32>(6u), total, 0.0);
    out[PLACE_TAIL + 2u] = vec4<f32>(0.0);
  }
}
