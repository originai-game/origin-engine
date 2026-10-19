// Terrain (phase 2): a height field drawn as a geometry clipmap. Every
// level is the same grid of quads (`level.grid.x` per side) around the
// camera, each level twice the spacing of the one inside it, its centre
// snapped to twice its spacing so its vertices stay on the world grid.
// A ring level leaves out the quads the finer level covers (the index
// buffer has the four hole positions). Near its outer edge a level's odd
// vertices slide onto the coarser grid (the morph), so where two levels
// meet both have the same vertices and heights: no cracks and no pops.
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
  view_proj: mat4x4<f32>,       // unjittered (prepass velocity)
  prev_view_proj: mat4x4<f32>,
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

@group(1) @binding(0) var<uniform> terrain: TerrainUniform;
@group(1) @binding(1) var<uniform> level: TerrainLevel;
@group(1) @binding(2) var heights: texture_2d<f32>;
@group(1) @binding(3) var terrain_normals: texture_2d<f32>;
@group(1) @binding(4) var control_map: texture_2d<f32>;
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

// The -x, -z corner in the terrain's space.
fn terrain_min() -> vec2<f32> {
  return -terrain.extent.xy * 0.5;
}

// The height (in the terrain's space) at its xz: bilinear between the four
// samples around it, clamped to the edge.
fn terrain_height(xz: vec2<f32>) -> f32 {
  let n = vec2<i32>(terrain.extent.zw);
  let cell = terrain.extent.xy / max(terrain.extent.zw - 1.0, vec2<f32>(1.0));
  let t = clamp((xz - terrain_min()) / cell, vec2<f32>(0.0), vec2<f32>(n - 1));
  let i = clamp(vec2<i32>(floor(t)), vec2<i32>(0), max(n - 2, vec2<i32>(0)));
  let f = clamp(t - vec2<f32>(i), vec2<f32>(0.0), vec2<f32>(1.0));
  let j = min(i + 1, n - 1);
  let a = textureLoad(heights, i, 0).r;
  let b = textureLoad(heights, vec2<i32>(j.x, i.y), 0).r;
  let c = textureLoad(heights, vec2<i32>(i.x, j.y), 0).r;
  let d = textureLoad(heights, j, 0).r;
  return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
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
  return vec3<f32>(xz.x, terrain_height(xz), xz.y);
}

// A point of the terrain's space in the world.
fn terrain_world(local: vec3<f32>) -> vec3<f32> {
  return (terrain.model * vec4<f32>(local, 1.0)).xyz;
}

// The clipmap vertex `vi` in world space.
fn terrain_vertex(vi: u32) -> vec3<f32> {
  return terrain_world(terrain_local_vertex(vi));
}
