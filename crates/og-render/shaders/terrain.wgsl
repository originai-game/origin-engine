// Terrain: a height field drawn as a geometry clipmap from a pyramid of
// tiles (terrain_tiles.rs). Every clipmap level is the same grid of quads
// (`level.grid.x` per side) around the camera, each level twice the
// spacing of the one inside it, its centre snapped to twice its spacing
// so its vertices stay on the terrain's grid. A ring level leaves out the
// quads the finer level covers (the index buffer has the four hole
// positions). Near its outer edge a level morphs into the coarser one
// (terrain_local_vertex), so where two levels meet both draw the same
// surface: no cracks and no pops.
//
// Heights come from the tile pyramid: every vertex reads the finest
// resident tile at its point (a tile fading in is mixed with its parent,
// which has fully faded in), so the height is one function of the point
// whichever clipmap level asks and levels meet without cracks. Tiles sit
// in texture-array slots; `table` maps every tile to its slot and fade.
// Pyramid levels are decimations of the field: a coarser tile standing in
// for one that has not arrived agrees with it on its own samples.
// Heights are read as f32 words and filtered here. Shared by the lit, prepass and shadow passes,
// each of which prepends its own group 0.

struct TerrainLayerGpu {
  color: vec4<f32>,     // rgb: tint (linear), w: roughness
  mapping: vec4<f32>,   // x: repeats per metre, y: 1 triplanar, z: normal strength, w: bits (1 albedo map, 2 normal map)
  slope: vec4<f32>,     // x: min (deg), y: max (deg), z: fade (deg), w: the albedo's layer (layer arrays)
  altitude: vec4<f32>,  // x: min (m), y: max (m), z: fade (m), w: the normal map's layer (layer arrays)
  offset: vec4<f32>,    // xyz: the layer's uv at the frame's origin, whole repeats dropped (origin.rs), w: displacement (world m)
  look: vec4<f32>,      // x: anti-tiling, y: height contrast, z: biplanar past this slope (deg, 0 none), w: macro variation
  // The hex-tiling lattice at the frame's origin, per projection plane
  // (xz, zy, xy): xy the origin's place in its skewed cell, zw the
  // origin cell's centre in uv, whole repeats dropped (terrain_lit.wgsl
  // plane_taps); computed in f64 on the CPU so the tiles stay put when
  // the origin moves.
  hex: array<vec4<f32>, 3>,
  hex_base: array<vec4<i32>, 2>,  // the origin cell's ids: xz.xy, zy.xy | xy.xy, -, -
  coarse: vec4<f32>,    // xyz: the uv at the frame's origin at a quarter of the frequency, whole repeats dropped, w: cell variation
};

struct TerrainUniform {
  model: mat4x4<f32>,   // the terrain's space (its centre at height 0, x and z along its sides) to the world
  normal: mat3x3<f32>,  // normals from the terrain's space to the world (inverse transpose)
  origin: vec4<f32>,    // xyz: the terrain's centre in the world, w: layers
  extent: vec4<f32>,    // xy: size along x and z (m), zw: samples along x and z
  params: vec4<f32>,    // x: height blend, y: 1 with a control map, z: 1 with holes, w: 1 with horizon shadows
  tiles: vec4<f32>,     // x: quads per tile, y: control texels per tile, z: pyramid levels, w: control pages (four layers each)
  displace: vec4<f32>,  // x: the displacement's reach (the terrain's horizontal metres, 0: none), y: world metres per unit of the terrain's y, z: the terrain's y in its horizontal metres, w: lying snow's lift (world m, terrain.rs snow_lift)
  view_proj: mat4x4<f32>,       // unjittered (prepass velocity)
  prev_view_proj: mat4x4<f32>,
  pyramid: array<vec4<f32>, 16>,  // per level: x its first tile, y tiles along x, z along z
  layers: array<TerrainLayerGpu, 16>,
  grass: TerrainGrass,  // the far grass (terrain_lit.wgsl grass_far)
};

// The colour an og.Grass field gives the ground past and between its
// blades (grass.rs FarGpu; grass_common.wgsl before this file).
struct TerrainGrass {
  color: vec4<f32>,   // rgb: the field's colour from afar (linear), w: its layer (< 0: no field)
  fade: vec4<f32>,    // x: where the tint starts (m from the camera), y: where it is whole, z: how much gusts brighten it, w: its strength (0..1)
  clump: vec4<f32>,   // x: clump size (the terrain's space), y: colour variation, z: dryness, w: the seed (24 bits)
  dry: vec4<f32>,     // rgb: dry grass (linear)
  wind: WindField,
};

// A terrain stamp's draw (terrain.rs TerrainStampDraw; bound as group 2
// by the stamps' passes in terrain_lit.wgsl and terrain_shadow.wgsl).
struct TerrainStampUniform {
  model: mat4x4<f32>,       // the mesh to the world
  normal: mat3x3<f32>,      // its normals to the world (inverse transpose)
  to_terrain: mat4x4<f32>,  // the world to the terrain's space
  params: vec4<f32>,        // x: blend (m), y: world metres per unit of the terrain's y
};

// Positions are in grid units of the finest level's spacing from the
// terrain's -x, -z corner: whole numbers, so two levels compute a shared
// vertex bit for bit the same.
struct TerrainLevel {
  center: vec4<f32>,    // xy: the level's centre (grid units), z: grid units per quad, w: level
  grid: vec4<f32>,      // x: quads per side, y: morph start (quads from the camera), z: morph width (0: none)
  camera: vec4<f32>,    // xy: the camera (grid units), z: metres per grid unit
  corners: vec4<f32>,   // xy: the coarser level's centre (grid units), z: this level's first corner in the corner cache, w: the coarser level's (terrain_vertex.wgsl)
};

// Every tile's slot and fade, four to a vector: (slot + 1) * 256 + fade * 255; 0 when not resident.
struct TerrainTable {
  entries: array<vec4<f32>, 1024>,
};

@group(1) @binding(0) var<uniform> terrain: TerrainUniform;
@group(1) @binding(1) var<uniform> level: TerrainLevel;
@group(1) @binding(2) var<storage, read> heights: array<f32>;
@group(1) @binding(3) var terrain_normals: texture_2d_array<f32>;
@group(1) @binding(4) var control_map: texture_2d_array<f32>;
@group(1) @binding(5) var albedo0: texture_2d<f32>;
@group(1) @binding(6) var albedo1: texture_2d<f32>;
@group(1) @binding(7) var albedo2: texture_2d<f32>;
@group(1) @binding(8) var albedo3: texture_2d<f32>;
@group(1) @binding(9) var normal0: texture_2d<f32>;
@group(1) @binding(10) var normal1: texture_2d<f32>;
@group(1) @binding(11) var normal2: texture_2d<f32>;
@group(1) @binding(12) var normal3: texture_2d<f32>;
@group(1) @binding(13) var layer_sampler: sampler;
@group(1) @binding(14) var map_sampler: sampler;
@group(1) @binding(15) var<uniform> table: TerrainTable;

// The -x, -z corner in the terrain's space.
fn terrain_min() -> vec2<f32> {
  return -terrain.extent.xy * 0.5;
}

// The field's sample spacing along x and z.
fn terrain_cell() -> vec2<f32> {
  return terrain.extent.xy / max(terrain.extent.zw - 1.0, vec2<f32>(1.0));
}

// Field-sample coordinates `s` on `lvl`'s grid: every 2^lvl-th sample and
// the last (whose cell may be shorter).
fn level_coord(lvl: u32, s: vec2<f32>) -> vec2<f32> {
  let step = f32(1u << lvl);
  let quads = max(terrain.extent.zw - 1.0, vec2<f32>(1.0));
  let n = ceil(quads / step) + 1.0;
  let last = (n - 2.0) * step;
  let span = max(quads - last, vec2<f32>(1e-6));
  let u = select(s / step, (n - 2.0) + (s - last) / span, s > last);
  return clamp(u, vec2<f32>(0.0), n - 1.0);
}

struct TileRef {
  slot: f32,          // < 0: not resident
  fade: f32,
  tile: vec2<f32>,    // the tile's position on its level
};

fn tile_ref(lvl: u32, u: vec2<f32>) -> TileRef {
  let p = terrain.pyramid[lvl];
  var r: TileRef;
  r.tile = clamp(floor(u / terrain.tiles.x), vec2<f32>(0.0), p.yz - 1.0);
  let i = u32(p.x + r.tile.y * p.y + r.tile.x);
  // Entry i: its vec4, then the lane by masks.
  let w = bitcast<vec4<u32>>(table.entries[i / 4u]);
  let odd = 0u - (i & 1u);
  let lo = w.x ^ ((w.x ^ w.y) & odd);
  let hi = w.z ^ ((w.z ^ w.w) & odd);
  let v = bitcast<f32>(lo ^ ((lo ^ hi) & (0u - ((i >> 1u) & 1u))));
  if (v < 255.5) {
    r.slot = -1.0;
    r.fade = 0.0;
    return r;
  }
  let slot = floor(v / 256.0);
  r.slot = slot - 1.0;
  r.fade = (v - slot * 256.0) / 255.0;
  return r;
}

// The same rows as TerrainLayout::heights_rows: heights, then horizon words.
fn height_word(p: vec2<i32>, slot: i32) -> f32 {
  let t = u32(terrain.tiles.x);
  let side = t + 3u;
  let horizon = (t + 7u) / 8u + 1u;
  let rows = side + (horizon * horizon * 4u + side - 1u) / side;
  return heights[(u32(slot) * rows + u32(p.y)) * side + u32(p.x)];
}

// The height at level coordinates `u` from a resident tile: bilinear
// between its four samples (the tile's border holds the neighbours').
fn slot_height(r: TileRef, u: vec2<f32>) -> f32 {
  let t = terrain.tiles.x;
  let l = u - r.tile * t + 1.0;
  let i = clamp(floor(l), vec2<f32>(0.0), vec2<f32>(t + 1.0));
  let f = clamp(l - i, vec2<f32>(0.0), vec2<f32>(1.0));
  let a = vec2<i32>(i);
  let b = min(a + 1, vec2<i32>(i32(t) + 2));
  let s = i32(r.slot);
  let h00 = height_word(a, s);
  let h10 = height_word(vec2<i32>(b.x, a.y), s);
  let h01 = height_word(vec2<i32>(a.x, b.y), s);
  let h11 = height_word(b, s);
  return mix(mix(h00, h10, f.x), mix(h01, h11, f.x), f.y);
}

struct Resident {
  lvl: u32,
  r: TileRef,
  u: vec2<f32>,
};

// The finest resident tile at or above `base` under field coordinates `s`.
fn resident_from(base: u32, s: vec2<f32>) -> Resident {
  let top = u32(terrain.tiles.z) - 1u;
  var o: Resident;
  o.lvl = min(base, top);
  o.u = level_coord(o.lvl, s);
  o.r = tile_ref(o.lvl, o.u);
  loop {
    if (o.r.slot >= 0.0 || o.lvl >= top) { break; }
    o.lvl += 1u;
    o.u = level_coord(o.lvl, s);
    o.r = tile_ref(o.lvl, o.u);
  }
  return o;
}

// The height (in the terrain's space) at field coordinates `s`, read at
// pyramid level `base` or the finest resident level above it.
fn terrain_height_at(base: u32, s: vec2<f32>) -> f32 {
  let top = u32(terrain.tiles.z) - 1u;
  let o = resident_from(base, s);
  if (o.r.slot < 0.0) { return 0.0; }
  var h = slot_height(o.r, o.u);
  if (o.r.fade < 1.0 && o.lvl < top) {
    let p = resident_from(o.lvl + 1u, s);
    if (p.r.slot >= 0.0) { h = mix(slot_height(p.r, p.u), h, o.r.fade); }
  }
  return h;
}

// A point of the terrain's space in the world.
fn terrain_world(local: vec3<f32>) -> vec3<f32> {
  return (terrain.model * vec4<f32>(local, 1.0)).xyz;
}

struct TerrainMaps {
  normal: vec3<f32>,    // in the terrain's space
  // The spread of the normals under the texel read (Toksvig: 1 / |mean|
  // - 1, 0 for one sample), which widens the specular lobe.
  variance: f32,
  // The layers' channels, four to a page (1 without a control map, or past its pages).
  control: array<vec4<f32>, 4>,
};

fn no_control() -> array<vec4<f32>, 4> {
  return array<vec4<f32>, 4>(vec4<f32>(1.0), vec4<f32>(1.0), vec4<f32>(1.0), vec4<f32>(1.0));
}

// A resident tile's normal map and control texels at level coordinates
// `u`; `gx`, `gy` are the field coordinates' screen derivatives.
fn tile_maps(lvl: u32, r: TileRef, u: vec2<f32>, gx: vec2<f32>, gy: vec2<f32>) -> TerrainMaps {
  let t = terrain.tiles.x;
  let step = f32(1u << lvl);
  let l = u - r.tile * t;
  let k = 1.0 / (step * (t + 1.0));
  // terrain_tiles.rs pack_normal: x and z of the mean's direction, its length.
  let texel = textureSampleGrad(terrain_normals, map_sampler, (l + 0.5) / (t + 1.0), i32(r.slot), gx * k, gy * k);
  let xz = texel.rg * 2.0 - 1.0;
  var m: TerrainMaps;
  m.normal = vec3<f32>(xz.x, sqrt(max(1.0 - dot(xz, xz), 0.0)), xz.y);
  m.variance = 1.0 / max(texel.b, 1e-3) - 1.0;
  m.control = no_control();
  if (terrain.params.y > 0.5) {
    let c = terrain.tiles.y;
    let ck = c / (t * step * (c + 2.0));
    let uv = (l / t * c + 1.0) / (c + 2.0);
    // Page p of slot s is layer s · pages + p; only the pages the layers use.
    let pages = u32(terrain.tiles.w);
    let used = (u32(terrain.origin.w) + 3u) / 4u;
    for (var p = 0u; p < min(pages, used); p++) {
      m.control[p] = textureSampleGrad(control_map, map_sampler, uv, i32(r.slot * f32(pages) + f32(p)), gx * ck, gy * ck);
    }
  }
  return m;
}

// The normal (terrain space) and control at field coordinates `s`, from
// the finest resident tile there (mixed with its parent while it fades in).
fn terrain_maps_at(s: vec2<f32>, gx: vec2<f32>, gy: vec2<f32>) -> TerrainMaps {
  let top = u32(terrain.tiles.z) - 1u;
  let o = resident_from(0u, s);
  if (o.r.slot < 0.0) {
    var none: TerrainMaps;
    none.normal = vec3<f32>(0.0, 1.0, 0.0);
    none.variance = 0.0;
    none.control = no_control();
    return none;
  }
  var m = tile_maps(o.lvl, o.r, o.u, gx, gy);
  if (o.r.fade < 1.0 && o.lvl < top) {
    let p = resident_from(o.lvl + 1u, s);
    if (p.r.slot >= 0.0) {
      let q = tile_maps(p.lvl, p.r, p.u, gx, gy);
      m.normal = mix(q.normal, m.normal, o.r.fade);
      m.variance = mix(q.variance, m.variance, o.r.fade);
      for (var p = 0u; p < 4u; p++) {
        m.control[p] = mix(q.control[p], m.control[p], o.r.fade);
      }
    }
  }
  m.normal = normalize(m.normal);
  return m;
}

// The horizon (terrain_tiles.rs HeightPyramid::horizon): per tile, every
// HORIZON_SPACING-th sample of its level, the elevation of the highest
// ground in 8 azimuths (k · 45° from +x towards +z), 12 bits each over
// -90° to 90°, two to a word, the words stored as numbers in the rows
// under the tile's heights.
const HORIZON_SPACING: f32 = 8.0;
const HORIZON_WORDS: u32 = 4u;

// One horizon texel's 8 elevations (radians) in a resident tile's slot.
fn horizon_texel(r: TileRef, i: vec2<u32>) -> array<f32, 8> {
  let t = u32(terrain.tiles.x);
  let hs = t + 3u;
  let side = (t + 7u) / 8u + 1u;
  let first = ((min(i.y, side - 1u) * side) + min(i.x, side - 1u)) * HORIZON_WORDS;
  var out: array<f32, 8>;
  for (var w = 0u; w < HORIZON_WORDS; w++) {
    let k = first + w;
    let v = u32(height_word(vec2<i32>(i32(k % hs), i32(hs + k / hs)), i32(r.slot)));
    for (var b = 0u; b < 2u; b++) {
      out[w * 2u + b] = (f32((v >> (12u * b)) & 4095u) / 4095.0 * 2.0 - 1.0) * 1.5707963;
    }
  }
  return out;
}

struct Horizon {
  // The elevations, bilinear between the four texels around the point.
  angles: array<f32, 8>,
  ok: bool,
};

// The horizon at field coordinates `s`, from the finest resident tile.
fn terrain_horizon(s: vec2<f32>) -> Horizon {
  var h: Horizon;
  let o = resident_from(0u, s);
  if (o.r.slot < 0.0) { return h; }
  let t = terrain.tiles.x;
  let side = ceil(t / HORIZON_SPACING) + 1.0;
  let g = clamp((o.u - o.r.tile * t) / t * (side - 1.0), vec2<f32>(0.0), vec2<f32>(side - 1.0));
  let i = vec2<u32>(floor(g));
  let f = g - floor(g);
  let a = horizon_texel(o.r, i);
  let b = horizon_texel(o.r, i + vec2<u32>(1u, 0u));
  let c = horizon_texel(o.r, i + vec2<u32>(0u, 1u));
  let d = horizon_texel(o.r, i + vec2<u32>(1u, 1u));
  for (var k = 0u; k < 8u; k++) {
    h.angles[k] = mix(mix(a[k], b[k], f.x), mix(c[k], d[k], f.x), f.y);
  }
  h.ok = true;
  return h;
}

// The horizon's elevation (radians) towards azimuth `phi` (radians from
// +x towards +z), between its two nearest azimuths.
fn horizon_towards(h: Horizon, phi: f32) -> f32 {
  let a = fract(phi / 6.2831853) * 8.0;
  let k = u32(floor(a)) % 8u;
  return mix(h.angles[k], h.angles[(k + 1u) % 8u], fract(a));
}

// The sky a point with an up-facing ground sees past its horizon, cosine
// weighted (each azimuth's slice open from its horizon up: cos² of the
// horizon's elevation above the plane), the mean of the 8 slices.
fn horizon_occlusion(h: Horizon) -> f32 {
  var open = 0.0;
  for (var k = 0u; k < 8u; k++) {
    let e = max(h.angles[k], 0.0);
    open += cos(e) * cos(e);
  }
  return open / 8.0;
}

// A roughness widened by the normals' spread under a texel (Toksvig 2005,
// for GGX as in Filament's specular anti-aliasing): alpha² grows by the
// variance, bounded so a far field never reads rougher than fully rough.
fn toksvig_roughness(roughness: f32, variance: f32) -> f32 {
  let a = roughness * roughness;
  return sqrt(sqrt(clamp(a * a + min(variance, 0.18), 0.0, 1.0)));
}

// The pipelines of a terrain with holes (terrain.rs terrain_passes);
// false leaves the hole test and its discard out.
override TERRAIN_HOLES: bool = true;

// Whether the field cell under field coordinates `s` is cut out: with
// holes, each tile height's lowest bit is the hole flag of the cell at
// its sample (terrain_tiles.rs mark_holes), read from the finest resident
// tile there.
fn terrain_hole(s: vec2<f32>) -> bool {
  if (!TERRAIN_HOLES || terrain.params.z < 0.5) { return false; }
  let o = resident_from(0u, s);
  if (o.r.slot < 0.0) { return false; }
  let t = terrain.tiles.x;
  let c = clamp(floor(o.u) - o.r.tile * t, vec2<f32>(0.0), vec2<f32>(t)) + 1.0;
  let h = height_word(vec2<i32>(c), i32(o.r.slot));
  return (bitcast<u32>(h) & 1u) != 0u;
}

// `terrain_hole` at a point of the terrain's space.
fn terrain_hole_at(local: vec3<f32>) -> bool {
  return terrain_hole((local.xz - terrain_min()) / terrain_cell());
}
