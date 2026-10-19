// Terrain: a height field drawn as a geometry clipmap from a pyramid of
// tiles (terrain_tiles.rs). Every clipmap level is the same grid of quads
// (`level.grid.x` per side) around the camera, each level twice the
// spacing of the one inside it, its centre snapped to twice its spacing
// so its vertices stay on the terrain's grid. A ring level leaves out the
// quads the finer level covers (the index buffer has the four hole
// positions). Near its outer edge a level's odd vertices slide onto the
// coarser grid (the morph), so where two levels meet both have the same
// vertices and heights: no cracks and no pops.
//
// Heights come from the tile pyramid: every vertex reads the finest
// resident tile at its point (a tile fading in is mixed with its parent,
// which has fully faded in), so the height is one function of the point
// whichever clipmap level asks and levels meet without cracks. Tiles sit
// in texture-array slots; `table` maps every tile to its slot and fade.
// Pyramid levels are decimations of the field: a coarser tile standing in
// for one that has not arrived agrees with it on its own samples.
// Heights are read with textureLoad and filtered here (R32Float is not
// filterable on WebGL2). Shared by the lit, prepass and shadow passes,
// each of which prepends its own group 0.

struct TerrainLayerGpu {
  color: vec4<f32>,     // rgb: tint (linear), w: roughness
  mapping: vec4<f32>,   // x: repeats per metre, y: 1 triplanar, z: normal strength, w: bits (1 albedo map, 2 normal map)
  slope: vec4<f32>,     // x: min (deg), y: max (deg), z: fade (deg)
  altitude: vec4<f32>,  // x: min (m), y: max (m), z: fade (m)
};

struct TerrainUniform {
  model: mat4x4<f32>,   // the terrain's space (its centre at height 0, x and z along its sides) to the world
  normal: mat3x3<f32>,  // normals from the terrain's space to the world (inverse transpose)
  origin: vec4<f32>,    // xyz: the terrain's centre in the world, w: layers
  extent: vec4<f32>,    // xy: size along x and z (m), zw: samples along x and z
  params: vec4<f32>,    // x: height blend, y: 1 with a control map
  tiles: vec4<f32>,     // x: quads per tile, y: control texels per tile, z: pyramid levels
  view_proj: mat4x4<f32>,       // unjittered (prepass velocity)
  prev_view_proj: mat4x4<f32>,
  pyramid: array<vec4<f32>, 16>,  // per level: x its first tile, y tiles along x, z along z
  layers: array<TerrainLayerGpu, 4>,
};

// Positions are in grid units of the finest level's spacing from the
// terrain's -x, -z corner: whole numbers, so two levels compute a shared
// vertex bit for bit the same.
struct TerrainLevel {
  center: vec4<f32>,    // xy: the level's centre (grid units), z: grid units per quad, w: level
  grid: vec4<f32>,      // x: quads per side, y: morph start (quads from the camera), z: morph width (0: none)
  camera: vec4<f32>,    // xy: the camera (grid units), z: metres per grid unit
};

// Every tile's slot and fade, four to a vector: (slot + 1) * 256 + fade * 255; 0 when not resident.
struct TerrainTable {
  entries: array<vec4<f32>, 1024>,
};

@group(1) @binding(0) var<uniform> terrain: TerrainUniform;
@group(1) @binding(1) var<uniform> level: TerrainLevel;
@group(1) @binding(2) var heights: texture_2d_array<f32>;
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
  let v = table.entries[i / 4u][i % 4u];
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
  let h00 = textureLoad(heights, a, s, 0).r;
  let h10 = textureLoad(heights, vec2<i32>(b.x, a.y), s, 0).r;
  let h01 = textureLoad(heights, vec2<i32>(a.x, b.y), s, 0).r;
  let h11 = textureLoad(heights, b, s, 0).r;
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

// The clipmap vertex `vi` of this level in the terrain's space (after the
// morph and clamped to the terrain).
fn terrain_local_vertex(vi: u32) -> vec3<f32> {
  let n = u32(level.grid.x);
  let g = vec2<f32>(f32(vi % (n + 1u)), f32(vi / (n + 1u)));
  let s = level.center.z;
  var c = level.center.xy + (g - f32(n) * 0.5) * s;
  if (level.grid.z > 0.0) {
    let d = max(abs(c.x - level.camera.x), abs(c.y - level.camera.y)) / s;
    let alpha = clamp((d - level.grid.y) / level.grid.z, 0.0, 1.0);
    // Odd vertices collapse onto their even neighbour towards -x and -z:
    // at alpha 1 the level's triangles are the coarser level's.
    let odd = g - 2.0 * floor(g * 0.5);
    c = c - odd * (s * alpha);
  }
  let unit = level.camera.z;
  c = clamp(c, vec2<f32>(0.0), terrain.extent.xy / unit);
  let xz = terrain_min() + c * unit;
  let samples = c * unit / terrain_cell();
  return vec3<f32>(xz.x, terrain_height_at(0u, samples), xz.y);
}

// A point of the terrain's space in the world.
fn terrain_world(local: vec3<f32>) -> vec3<f32> {
  return (terrain.model * vec4<f32>(local, 1.0)).xyz;
}

// The clipmap vertex `vi` in world space.
fn terrain_vertex(vi: u32) -> vec3<f32> {
  return terrain_world(terrain_local_vertex(vi));
}

struct TerrainMaps {
  normal: vec3<f32>,    // in the terrain's space
  control: vec4<f32>,   // the layers' channels (1 without a control map)
};

// A resident tile's normal map and control texels at level coordinates
// `u`; `gx`, `gy` are the field coordinates' screen derivatives.
fn tile_maps(lvl: u32, r: TileRef, u: vec2<f32>, gx: vec2<f32>, gy: vec2<f32>) -> TerrainMaps {
  let t = terrain.tiles.x;
  let step = f32(1u << lvl);
  let l = u - r.tile * t;
  let k = 1.0 / (step * (t + 1.0));
  let xz = textureSampleGrad(terrain_normals, map_sampler, (l + 0.5) / (t + 1.0), i32(r.slot), gx * k, gy * k).rg;
  var m: TerrainMaps;
  m.normal = vec3<f32>(xz.x, sqrt(max(1.0 - dot(xz, xz), 0.0)), xz.y);
  m.control = vec4<f32>(1.0);
  if (terrain.params.y > 0.5) {
    let c = terrain.tiles.y;
    let ck = c / (t * step * (c + 2.0));
    m.control = textureSampleGrad(control_map, map_sampler, (l / t * c + 1.0) / (c + 2.0), i32(r.slot), gx * ck, gy * ck);
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
    none.control = vec4<f32>(1.0);
    return none;
  }
  var m = tile_maps(o.lvl, o.r, o.u, gx, gy);
  if (o.r.fade < 1.0 && o.lvl < top) {
    let p = resident_from(o.lvl + 1u, s);
    if (p.r.slot >= 0.0) {
      let q = tile_maps(p.lvl, p.r, p.u, gx, gy);
      m.normal = mix(q.normal, m.normal, o.r.fade);
      m.control = mix(q.control, m.control, o.r.fade);
    }
  }
  m.normal = normalize(m.normal);
  return m;
}
