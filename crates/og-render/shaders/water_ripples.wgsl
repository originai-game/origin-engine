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
  // x: the foam kept per substep, y: the foam a breaking crest makes a second, z: the highest crest (m; troughs half as deep),
  // w: the fraction of the square over which the ripples fade out at its edge
  foam: vec4<f32>,
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
// (height, vertical velocity, foam, 0) a point.
@group(0) @binding(1) var state_in: texture_2d<f32>;
@group(0) @binding(2) var state_out: texture_storage_2d<rgba32float, write>;
@group(0) @binding(3) var<storage, read> cuts: array<Cut>;
@group(0) @binding(4) var coarse: texture_2d<f32>;
// What the water draws: (the CPU's heights plus the ripples, the ripples alone, their foam, 0).
@group(0) @binding(5) var heights_out: texture_storage_2d<rgba16float, write>;

// How much of the cell at `p` (m, the surface's frame) a footprint covers: the ellipse inside its box grown by
// `grow` metres, its edge softened over a cell. An empty box (max < min) covers nothing.
fn cover(p: vec2<f32>, box_: vec4<f32>, grow: f32) -> f32 {
  let half = (box_.zw - box_.xy) * 0.5;
  if (half.x <= 0.0 || half.y <= 0.0) { return 0.0; }
  let centre = (box_.xy + box_.zw) * 0.5;
  let r = half + vec2<f32>(grow);
  let d = length((p - centre) / r);
  let feather = params.size.w / max(min(r.x, r.y), 1e-4);
  return 1.0 - smoothstep(1.0 - feather, 1.0 + feather, d);
}

// 1 inside the square, falling to 0 over the last `params.foam.w` of it at each edge: the ripples die there, so the
// square's edge never shows.
fn inside_square(id: vec2<u32>) -> f32 {
  let n = params.size.x - 1.0;
  let q = vec2<f32>(id) / max(n, 1.0);
  let d = min(min(q.x, q.y), min(1.0 - q.x, 1.0 - q.y));
  return smoothstep(0.0, max(params.foam.w, 1e-3), d);
}

fn load(i: vec2<i32>) -> vec4<f32> {
  let n = i32(params.size.x);
  return textureLoad(state_in, clamp(i, vec2<i32>(0), vec2<i32>(n - 1)), 0);
}

// How much of the cell at `p` the bodies of this substep's tick fill.
fn held(p: vec2<f32>) -> f32 {
  var solid = 0.0;
  for (var k = 0u; k < u32(params.force.y); k++) {
    solid = max(solid, cover(p, cuts[u32(params.force.x) + k].cur, 0.0));
  }
  return solid;
}

@compute @workgroup_size(8, 8)
fn advance(@builtin(global_invocation_id) id: vec3<u32>) {
  let n = u32(params.size.x);
  if (id.x >= n || id.y >= n) { return; }
  let i = vec2<i32>(id.xy);
  let here = load(i);
  let l = load(i + vec2<i32>(-1, 0));
  let r = load(i + vec2<i32>(1, 0));
  let u = load(i + vec2<i32>(0, -1));
  let d = load(i + vec2<i32>(0, 1));
  let lap = l.x + r.x + u.x + d.x - 4.0 * here.x;
  // A body is solid: the water it fills is held still (no hole under it). The water it displaces rises in a ring
  // around it: as it sinks in or falls (its splash), ahead of it as it moves (its bow wave), and falls back behind it
  // (the trough of its wake). The tick's push is spread over its substeps.
  let p = (vec2<f32>(id.xy) * params.size.w) - vec2<f32>(params.size.z * 0.5);
  let cell = params.size.w;
  var solid = 0.0;
  var near = 0.0;
  var push = 0.0;
  let first = u32(params.force.x);
  let count = u32(params.force.y);
  for (var k = 0u; k < count; k++) {
    let c = cuts[first + k];
    let half = min(c.cur.z - c.cur.x, c.cur.w - c.cur.y) * 0.5;
    let ring = max(3.0 * cell, half * 0.6);
    let body = cover(p, c.cur, 0.0);
    solid = max(solid, body);
    near = max(near, cover(p, c.cur, cell * 1.5));
    let around = cover(p, c.cur, ring) - body;
    // Sinking in (the depth growing) or falling raises the ring; rising out lets it down.
    let sink = (min(c.extra.y, 0.5) - min(c.extra.x, 0.5)) * params.force.z + max(-c.extra.z, 0.0) * params.force.w;
    push += around * sink * 0.15;
    // Moving across: up where the ring arrives, down where it leaves.
    let bow = cover(p, c.cur, ring) - cover(p, c.prev, ring);
    push += bow * min(c.extra.y, 0.5) * params.force.z * 0.15;
  }
  let edge = inside_square(id.xy);
  let free = 1.0 - smoothstep(0.3, 0.7, solid);
  var v = (here.y + lap * params.wave.x + push / params.wave.w) * params.wave.y;
  // The square's edge soaks the waves up instead of throwing them back.
  v *= mix(0.9, 1.0, edge) * free;
  let cap = params.foam.z;
  let h = clamp(here.x + v * params.wave.z, -0.5 * cap, cap) * mix(0.95, 1.0, edge) * free;
  // Foam: made where a crest breaks, spread to the neighbours, thinning out; never on the body or right against it
  // (clear water around it). A crest breaks past its limiting steepness (Stokes's crest of 120°, its faces some 0.58;
  // whitecaps start from some 0.4: Longuet-Higgins's kinematic limit, a crest's downward acceleration g/2, is that
  // steepness in deep water, where a wave's acceleration is g times its slope). Steepness, not this grid's c² ∇²h:
  // its waves all run at one speed, so a short one's acceleration grows with its wavenumber squared, and the finest
  // ripples of a splash read as breaking long after it. A body's cells are held at rest, so across them the slope is
  // taken as a wall's (the water beside mirrored), not a step down to the rest level. (A crest made foam whenever it
  // stood high and rising: the ring of water a floating body holds up, bobbing with it, foamed over without end and
  // drew the body's funnel as an opaque white disc.)
  let spread = (l.z + r.z + u.z + d.z) * 0.25;
  let hl = mix(l.x, here.x, held(p - vec2<f32>(cell, 0.0)));
  let hr = mix(r.x, here.x, held(p + vec2<f32>(cell, 0.0)));
  let hu = mix(u.x, here.x, held(p - vec2<f32>(0.0, cell)));
  let hd = mix(d.x, here.x, held(p + vec2<f32>(0.0, cell)));
  let steep = length(vec2<f32>(hr - hl, hd - hu)) / (2.0 * cell);
  let crest = step(0.0, here.x) * step(hl + hr + hu + hd, 4.0 * here.x);
  let made = smoothstep(0.4, 0.6, steep) * crest * (1.0 - solid) * params.foam.y * params.wave.z;
  let foam = clamp((mix(here.z, spread, 0.2) + made) * params.foam.x, 0.0, 1.0) * (1.0 - near) * edge;
  textureStore(state_out, i, vec4<f32>(h, v, foam, 0.0));
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
  let s = textureLoad(state_in, vec2<i32>(id.xy), 0);
  let fade = inside_square(id.xy);
  let ripple = s.x * fade;
  textureStore(heights_out, vec2<i32>(id.xy), vec4<f32>(base + ripple, ripple, s.z * fade, 0.0));
}

// Calm water: the state written by this group set to zero.
@compute @workgroup_size(8, 8)
fn calm(@builtin(global_invocation_id) id: vec3<u32>) {
  let n = u32(params.size.x);
  if (id.x >= n || id.y >= n) { return; }
  textureStore(state_out, vec2<i32>(id.xy), vec4<f32>(0.0));
}
