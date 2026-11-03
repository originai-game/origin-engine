// The ripples over an og.WaterSurface (water_ripples.rs): a wave equation
// on a grid finer than the CPU's height field, stirred by the bodies that
// cut the surface (their wakes as they move, their splashes as they fall),
// stepped once per tick in `step` and added to the CPU's heights, bilinear,
// in `compose`, whose output the water draws instead of the CPU's heights.
// Picture only: nothing reads it back.

struct Params {
  // x: the ripples' grid side (points), y: the CPU field's side, z: the surface's side (m), w: the ripples' cell (m)
  size: vec4<f32>,
  // x: c² dt / cell² per substep, y: the velocity kept per substep, z: the substep's dt (s), w: the substeps a tick
  wave: vec4<f32>,
  // x: the tick's first cut, y: its cuts, z: 1 / the tick's dt, w: the surface's splash
  force: vec4<f32>,
};

// A body as it cut the surface after the tick before and after this one: its footprint's min and max (xz in the
// surface's frame), its depth below the rest level (prev, cur), its vertical speed.
struct Cut {
  prev: vec4<f32>,
  cur: vec4<f32>,
  // x: depth before, y: depth now, z: vertical speed (m/s)
  extra: vec4<f32>,
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var state_in: texture_2d<f32>;
@group(0) @binding(2) var state_out: texture_storage_2d<rg32float, write>;
@group(0) @binding(3) var<storage, read> cuts: array<Cut>;
@group(0) @binding(4) var coarse: texture_2d<f32>;
@group(0) @binding(5) var heights_out: texture_storage_2d<rgba16float, write>;

// How much of the cell at `p` (m, the surface's frame) a footprint covers: the ellipse inside its box, its edge
// softened over two cells. An empty box (max < min) covers nothing.
fn cover(p: vec2<f32>, box_: vec4<f32>) -> f32 {
  let half = (box_.zw - box_.xy) * 0.5;
  if (half.x <= 0.0 || half.y <= 0.0) { return 0.0; }
  let centre = (box_.xy + box_.zw) * 0.5;
  let d = length((p - centre) / half);
  let feather = 2.0 * params.size.w / max(min(half.x, half.y), 1e-4);
  return 1.0 - smoothstep(1.0 - feather, 1.0 + feather, d);
}

fn load(i: vec2<i32>) -> vec2<f32> {
  let n = i32(params.size.x);
  return textureLoad(state_in, clamp(i, vec2<i32>(0), vec2<i32>(n - 1)), 0).xy;
}

@compute @workgroup_size(8, 8)
fn advance(@builtin(global_invocation_id) id: vec3<u32>) {
  let n = u32(params.size.x);
  if (id.x >= n || id.y >= n) { return; }
  let i = vec2<i32>(id.xy);
  let here = load(i);
  let lap = load(i + vec2<i32>(-1, 0)).x + load(i + vec2<i32>(1, 0)).x + load(i + vec2<i32>(0, -1)).x
    + load(i + vec2<i32>(0, 1)).x - 4.0 * here.x;
  // Each body pushes the water it moves into down and lets it back where it leaves (its wake), and pushes it by its
  // vertical speed while it cuts the surface (its splash), the tick's push spread over its substeps.
  let p = (vec2<f32>(id.xy) * params.size.w) - vec2<f32>(params.size.z * 0.5);
  var push = 0.0;
  let first = u32(params.force.x);
  let count = u32(params.force.y);
  for (var k = 0u; k < count; k++) {
    let c = cuts[first + k];
    let before = cover(p, c.prev) * min(c.extra.x, 0.5);
    let now = cover(p, c.cur) * min(c.extra.y, 0.5);
    push += -(now - before) * params.force.z * 0.2 + c.extra.z * params.force.w * 0.06 * cover(p, c.cur);
  }
  let v = (here.y + lap * params.wave.x + push / params.wave.w) * params.wave.y;
  let h = clamp(here.x + v * params.wave.z, -1.0, 1.0);
  textureStore(state_out, i, vec4<f32>(h, v, 0.0, 0.0));
}

@compute @workgroup_size(8, 8)
fn compose(@builtin(global_invocation_id) id: vec3<u32>) {
  let n = u32(params.size.x);
  if (id.x >= n || id.y >= n) { return; }
  // The CPU's heights here, bilinear (its grid spans the same square).
  let m = i32(params.size.y);
  let g = vec2<f32>(id.xy) * params.size.w / (params.size.z / f32(m - 1));
  let c = clamp(vec2<i32>(floor(g)), vec2<i32>(0), vec2<i32>(m - 2));
  let f = clamp(g - vec2<f32>(c), vec2<f32>(0.0), vec2<f32>(1.0));
  let a = textureLoad(coarse, c, 0).r;
  let b = textureLoad(coarse, c + vec2<i32>(1, 0), 0).r;
  let d = textureLoad(coarse, c + vec2<i32>(0, 1), 0).r;
  let e = textureLoad(coarse, c + vec2<i32>(1, 1), 0).r;
  let base = mix(mix(a, b, f.x), mix(d, e, f.x), f.y);
  let ripple = textureLoad(state_in, vec2<i32>(id.xy), 0).x;
  textureStore(heights_out, vec2<i32>(id.xy), vec4<f32>(base + ripple, 0.0, 0.0, 1.0));
}

// Calm water: the state written by this group set to zero.
@compute @workgroup_size(8, 8)
fn calm(@builtin(global_invocation_id) id: vec3<u32>) {
  let n = u32(params.size.x);
  if (id.x >= n || id.y >= n) { return; }
  textureStore(state_out, vec2<i32>(id.xy), vec4<f32>(0.0));
}
