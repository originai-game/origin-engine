// Grass generation (grass.rs): the compute passes that decide which
// blades a field draws this frame. Prepended by terrain.wgsl (group 1: the
// terrain's uniform, tile arrays and table, a compute group of its own)
// and grass_common.wgsl.
//
// The CPU lists the field's 16 m chunks the view (and, when the grass
// casts, the sun's reach) needs, nearest first, each cut into slices of
// GRASS_SLICE candidates (`slices`). Then, for each field:
//
// 1. `evaluate`: one workgroup per slice. Each candidate is a point of a
//    blue-noise tile copied over the chunk; it is kept while its rank is
//    below the density there (the layer's weight times the distance
//    table), off the terrain's holes, inside the view or its shadow's;
//    it takes a level of detail and the wind's bend and is written, packed,
//    at its own slot. The slice counts its blades per level.
// 2. `scan`: one workgroup. An exclusive prefix sum of the slices' counts
//    per level, the instance buffer's header and the indirect draws.
// 3. `compact`: one workgroup per slice. Each blade goes to its level's
//    first place plus its slice's prefix plus its rank in the slice.
//
// Every blade's place follows from the candidates alone, so the same
// frame lays out the same instances (bevy_grass appended with atomics).
//
// Ported from bevy_grass v0.20.3 (src/shaders/generate.wesl;
// third_party/bevy-grass). MIT License, Copyright (c) 2026 Gediminas
// Mikelenas: the full notice is in grass_common.wgsl and
// third_party/bevy-grass/LICENSE.

const GRASS_SLICE: u32 = 512u;
const GRASS_TILE_POINTS: u32 = 1024u;

// grass.rs GenGpu.
struct GrassGen {
  planes: array<vec4<f32>, 6>,  // the view's frustum, inside: dot(xyz, p) + w >= 0
  eye: vec4<f32>,       // xyz: the camera (frame coordinates), w: 1 orthographic
  origin: vec4<f32>,    // xyz: where blade positions are stored from, w: their extent either side (m)
  lod: vec4<f32>,       // x: where LOD 1 starts, y: LOD 2 (m), z: the band they dither over (m), w: the radius (m)
  lod_density: vec4<f32>,  // x: the share kept where LOD 2 starts, y: where the fade starts, z: the further share from LOD 1 on, w: where the fade starts (share of the radius)
  field: vec4<f32>,     // x: candidates per square metre (the terrain's space), y: blades per square metre there, z: a tile's side, w: tiles per chunk side
  chunk: vec4<f32>,     // x: a chunk's side (the terrain's space), y: the layer, z: the seed (24 bits), w: the widest a thinned blade widens
  shape: vec4<f32>,     // x: the tallest blade (m), y: height variation, z: lean, w: clump lean
  clump: vec4<f32>,     // x: clump size (the terrain's space), y: tufts, z: the height range blade roots are stored over (m either side), w: the root width (m)
  shadow: vec4<f32>,    // xyz: where a blade's shadow lands per metre of its height, w: 1 when it casts
  motion: vec4<f32>,    // x: sway, y: stiffness
  counts: vec4<u32>,    // x: slices, y: blades the instance buffer holds, z: rows of LOD 0, w: LOD 1's and 2's (8 bits each)
  wind: GrassWind,
};

@group(0) @binding(0) var<uniform> gen: GrassGen;
// The blue-noise tile (grass.rs blue_noise_tile): two points to a vector.
@group(0) @binding(1) var<uniform> tile: array<vec4<f32>, 512>;
// Per slice: x, y its chunk, z its first candidate in the chunk, w how many.
@group(0) @binding(2) var<storage, read> slices: array<vec4<u32>>;
// GRASS_SLICE packed blades per slice (level GRASS_NO_BLADE: none).
@group(0) @binding(3) var<storage, read_write> scratch: array<vec4<u32>>;
// Per slice: its blades per level, then (after `scan`) the blades before them in their level.
@group(0) @binding(4) var<storage, read_write> counts: array<u32>;
// [0]: xyz each level's first instance, w the blades; then the blades.
@group(0) @binding(5) var<storage, read_write> instances: array<vec4<u32>>;
// Three indexed indirect draws, one per level (index count, instances, first index, base vertex, first instance).
@group(0) @binding(6) var<storage, read_write> draws: array<u32, 15>;

fn grass_visible(center: vec3<f32>, radius: f32) -> bool {
  for (var i = 0; i < 6; i++) {
    if (dot(gen.planes[i].xyz, center) + gen.planes[i].w < -radius) { return false; }
  }
  return true;
}

// The distance table: 1 to the start of LOD 1, lod_density.x where LOD 2
// starts, lod_density.y where the fade starts, 0 at the radius. grass.rs
// DistanceLod::thinning.
fn grass_thinning(d: f32) -> f32 {
  let lod1 = gen.lod.x;
  let lod2 = gen.lod.y;
  let radius = gen.lod.w;
  let edge = radius * gen.lod_density.w;
  if (d <= lod1) { return 1.0; }
  if (d <= lod2) { return mix(1.0, gen.lod_density.x, clamp((d - lod1) / max(lod2 - lod1, 1e-4), 0.0, 1.0)); }
  if (d <= edge) { return mix(gen.lod_density.x, gen.lod_density.y, clamp((d - lod2) / max(edge - lod2, 1e-4), 0.0, 1.0)); }
  return mix(gen.lod_density.y, 0.0, clamp((d - edge) / max(radius - edge, 1e-4), 0.0, 1.0));
}

// grass.rs DistanceLod::falloff: the table times the further share kept
// from LOD 1's band on.
fn grass_falloff(d: f32) -> f32 {
  let into = clamp((d - (gen.lod.x - gen.lod.z * 0.5)) / gen.lod.z, 0.0, 1.0);
  return grass_thinning(d) * mix(1.0, gen.lod_density.z, into);
}

// grass.rs DistanceLod::lod: inside a band round each start the blade
// changes level when its own dither says so.
fn grass_lod(d: f32, dither: f32) -> u32 {
  var lod = 0u;
  if (dither < (d - (gen.lod.x - gen.lod.z * 0.5)) / gen.lod.z) { lod += 1u; }
  if (dither < (d - (gen.lod.y - gen.lod.z * 0.5)) / gen.lod.z) { lod += 1u; }
  return lod;
}

// How much wider a blade draws where `falloff` of them are kept, up to `widest`.
fn grass_coverage(falloff: f32, widest: f32) -> f32 {
  return min(1.0 / sqrt(max(falloff, 1.0 / (widest * widest))), widest);
}

fn grass_band(x: f32, lo: f32, hi: f32, fade: f32) -> f32 {
  let f = max(fade, 1e-4);
  return smoothstep(lo - f, lo, x) * (1.0 - smoothstep(hi, hi + f, x));
}

// The grass layer's weight at a point (world `p`, world normal `n`, the
// control channels `c`): its control channel times its slope and altitude
// rules over every layer's (scatter.rs Terrain::layer_weights); the first
// layer where none shows.
fn grass_layer_weight(p: vec3<f32>, n: vec3<f32>, c: array<vec4<f32>, 4>) -> f32 {
  let count = min(u32(terrain.origin.w), 16u);
  let layer = u32(gen.chunk.y);
  let angle = degrees(acos(clamp(n.y, -1.0, 1.0)));
  let altitude = p.y - terrain.origin.y;
  var sum = 0.0;
  var mine = 0.0;
  for (var i = 0u; i < count; i++) {
    let l = terrain.layers[i];
    let w = c[i / 4u][i % 4u] * grass_band(angle, l.slope.x, l.slope.y, l.slope.z) * grass_band(altitude, l.altitude.x, l.altitude.y, l.altitude.z);
    sum += w;
    if (i == layer) { mine = w; }
  }
  if (sum < 1e-4) { return select(0.0, 1.0, layer == 0u); }
  return mine / sum;
}

// Candidate `index` of slice `s`'s chunk, packed (level GRASS_NO_BLADE:
// no blade there).
fn grass_evaluate(s: vec4<u32>, index: u32) -> vec4<u32> {
  let none = vec4<u32>(0u, 0u, GRASS_NO_BLADE << 24u, 0u);
  let tiles = u32(gen.field.w);
  let cells = tiles * tiles;
  let point = index / cells;
  let cell = index % cells;
  let seed_bits = u32(gen.chunk.z);
  let cell_hash = grass_hash2(grass_hash2(s.x, s.y), grass_hash2(cell, seed_bits));
  let offset = vec2<f32>(grass_unit(cell_hash), grass_unit(grass_hash(cell_hash)));
  let pair = tile[point / 2u];
  let tp = select(pair.xy, pair.zw, (point & 1u) == 1u);
  let p = fract(tp + offset);
  let corner = vec2<f32>(f32(s.x), f32(s.y)) * gen.chunk.x;
  var rel = corner + (vec2<f32>(f32(cell % tiles), f32(cell / tiles)) + p) * gen.field.z;
  // Clumps: Voronoi cells of the terrain's space. Tufts pull a blade part
  // of the way to its clump's centre, each clump by its own share.
  let clump = grass_clump(rel, gen.clump.x, seed_bits);
  rel -= clump.zw * min(gen.clump.y * (0.6 + 0.8 * clump.y), 0.95);
  if (any(rel < vec2<f32>(0.0)) || any(rel >= terrain.extent.xy)) { return none; }
  let rank = (f32(point) + 0.5) / f32(GRASS_TILE_POINTS);
  let samples = rel / terrain_cell();
  if (terrain_hole(samples)) { return none; }
  let local = terrain_min() + rel;
  let maps = terrain_maps_at(samples, vec2<f32>(0.0), vec2<f32>(0.0));
  let root = terrain_world(vec3<f32>(local.x, terrain_height_at(0u, samples), local.y));
  let normal = normalize(terrain.normal * maps.normal);
  var falloff = 1.0;
  var d = 0.0;
  if (gen.eye.w < 0.5) {
    d = distance(root, gen.eye.xyz);
    falloff = grass_falloff(d);
  }
  let keep = clamp(gen.field.y * grass_layer_weight(root, normal, maps.control) * falloff / gen.field.x, 0.0, 1.0);
  if (rank >= keep) { return none; }

  let seed = grass_hash2(cell_hash, point);
  // Blades of a clump share most of their height.
  let tall = gen.shape.x * (1.0 - gen.shape.y * mix(grass_unit(grass_hash(seed ^ 0x68u)), clump.y, 0.6));
  if (tall < 0.01) { return none; }
  var seen = grass_visible(root + vec3<f32>(0.0, tall * 0.5, 0.0), tall);
  if (!seen && gen.shadow.w > 0.5) {
    let reach = gen.shadow.xyz * tall;
    seen = grass_visible(root + reach * 0.5, length(reach) * 0.5 + tall * 0.25);
  }
  if (!seen) { return none; }

  var b: GrassBlade;
  b.root = root;
  b.lod = 0u;
  if (gen.eye.w < 0.5) { b.lod = grass_lod(d, grass_unit(grass_hash(seed ^ 0x10du))); }
  b.facing = grass_unit(grass_hash(seed ^ 0x66u));
  b.height = tall / max(gen.shape.x, 1e-3);
  let width = gen.clump.w * (0.75 + 0.5 * grass_unit(grass_hash(seed ^ 0x77u))) * grass_coverage(falloff, gen.chunk.w);
  b.width = width / (GRASS_WIDTH_RANGE * max(gen.clump.w, 1e-5));
  b.seed = grass_unit(grass_hash(seed ^ 0x5eu));
  b.clump = clump.x;
  // The lean at rest: along the blade's face by a share of `lean`, and
  // out from its clump's centre by `clump lean` (more at its rim).
  let face = vec2<f32>(sin(b.facing * GRASS_TAU), cos(b.facing * GRASS_TAU));
  let out_local = clump.zw / max(gen.clump.x * 0.5, 1e-4);
  let out = (terrain.model * vec4<f32>(out_local.x, 0.0, out_local.y, 0.0)).xz;
  let out_len = length(out);
  let away = select(vec2<f32>(0.0), out / max(out_len, 1e-6) * min(out_len, 1.0), out_len > 1e-4);
  let rest = face * gen.shape.z * (0.3 + 0.7 * grass_unit(grass_hash(seed ^ 0x6cu))) + away * gen.shape.w;
  let at = root.xz + gen.wind.time.zw;
  b.bend = clamp(rest + grass_bend(gen.wind, at, b.seed, gen.motion.y, gen.motion.x, false), vec2<f32>(-1.0), vec2<f32>(1.0));
  b.previous_bend = clamp(rest + grass_bend(gen.wind, at, b.seed, gen.motion.y, gen.motion.x, true), vec2<f32>(-1.0), vec2<f32>(1.0));
  return grass_pack(b, gen.origin, gen.clump.z);
}

var<workgroup> slice_counts: array<atomic<u32>, 3>;

@compute @workgroup_size(64)
fn evaluate(@builtin(workgroup_id) group: vec3<u32>, @builtin(local_invocation_index) lane: u32) {
  let w = group.x;
  let s = slices[w];
  if (lane < 3u) { atomicStore(&slice_counts[lane], 0u); }
  workgroupBarrier();
  for (var k = lane; k < GRASS_SLICE; k += 64u) {
    var words = vec4<u32>(0u, 0u, GRASS_NO_BLADE << 24u, 0u);
    if (k < s.w) { words = grass_evaluate(s, s.z + k); }
    scratch[w * GRASS_SLICE + k] = words;
    let lod = (words.z >> 24u) & 3u;
    if (lod < GRASS_NO_BLADE) { atomicAdd(&slice_counts[lod], 1u); }
  }
  workgroupBarrier();
  if (lane < 3u) { counts[w * 3u + lane] = atomicLoad(&slice_counts[lane]); }
}

var<workgroup> partial: array<vec3<u32>, 256>;
var<workgroup> totals: vec3<u32>;

@compute @workgroup_size(256)
fn scan(@builtin(local_invocation_index) lane: u32) {
  let n = gen.counts.x;
  let per = (n + 255u) / 256u;
  let first = min(lane * per, n);
  let last = min(first + per, n);
  var sum = vec3<u32>(0u);
  for (var i = first; i < last; i++) {
    sum += vec3<u32>(counts[i * 3u], counts[i * 3u + 1u], counts[i * 3u + 2u]);
  }
  partial[lane] = sum;
  workgroupBarrier();
  if (lane == 0u) {
    var run = vec3<u32>(0u);
    for (var i = 0u; i < 256u; i++) {
      let v = partial[i];
      partial[i] = run;
      run += v;
    }
    totals = run;
  }
  workgroupBarrier();
  var run = partial[lane];
  for (var i = first; i < last; i++) {
    let c = vec3<u32>(counts[i * 3u], counts[i * 3u + 1u], counts[i * 3u + 2u]);
    counts[i * 3u] = run.x;
    counts[i * 3u + 1u] = run.y;
    counts[i * 3u + 2u] = run.z;
    run += c;
  }
  if (lane == 0u) {
    // What the buffer holds, nearest level first.
    let cap = gen.counts.y;
    let c0 = min(totals.x, cap);
    let c1 = min(totals.y, cap - c0);
    let c2 = min(totals.z, cap - c0 - c1);
    instances[0] = vec4<u32>(1u, 1u + c0, 1u + c0 + c1, c0 + c1 + c2);
    let kept = array<u32, 3>(c0, c1, c2);
    let rows = array<u32, 3>(gen.counts.z, gen.counts.w & 0xffu, (gen.counts.w >> 8u) & 0xffu);
    var first_index = 0u;
    for (var l = 0u; l < 3u; l++) {
      let index_count = 3u * (2u * rows[l] - 1u);
      draws[l * 5u] = index_count;
      draws[l * 5u + 1u] = kept[l];
      draws[l * 5u + 2u] = first_index;
      draws[l * 5u + 3u] = l * GRASS_LOD_STRIDE;
      draws[l * 5u + 4u] = 0u;
      first_index += index_count;
    }
  }
}

var<workgroup> lane_lods: array<u32, 64>;
var<workgroup> placed: array<u32, 3>;

@compute @workgroup_size(64)
fn compact(@builtin(workgroup_id) group: vec3<u32>, @builtin(local_invocation_index) lane: u32) {
  let w = group.x;
  if (lane < 3u) { placed[lane] = counts[w * 3u + lane]; }
  let header = instances[0];
  for (var round = 0u; round < GRASS_SLICE / 64u; round++) {
    let words = scratch[w * GRASS_SLICE + round * 64u + lane];
    let lod = (words.z >> 24u) & 3u;
    lane_lods[lane] = lod;
    workgroupBarrier();
    if (lod < GRASS_NO_BLADE) {
      var rank = 0u;
      for (var j = 0u; j < lane; j++) {
        rank += select(0u, 1u, lane_lods[j] == lod);
      }
      let at = placed[lod] + rank;
      if (at < draws[lod * 5u + 1u]) { instances[header[lod] + at] = words; }
    }
    workgroupBarrier();
    if (lane < 3u) {
      var n = 0u;
      for (var j = 0u; j < 64u; j++) {
        n += select(0u, 1u, lane_lods[j] == lane);
      }
      placed[lane] += n;
    }
    workgroupBarrier();
  }
}
