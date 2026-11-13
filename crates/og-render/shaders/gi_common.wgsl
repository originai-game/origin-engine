// Probe global illumination (gi_probes.rs): what the lit modules' sampling
// (gi_probes.wgsl, after frame.wgsl), the trace (gi_trace.wgsl) and the
// blend (gi_blend.wgsl) share. A port of Wicked Engine's DDGI
// (ShaderInterop_DDGI.h, ddgi_raytraceCS.hlsl, ddgi_updateCS.hlsl; MIT,
// Copyright (c) Turánszki János, third_party/wicked-ddgi), itself after
// diharaw's hybrid-rendering (MIT) and Majercik et al., "Dynamic Diffuse
// Global Illumination with Ray-Traced Irradiance Fields" (JCGT 2019).
//
// The probes live in the light volumes' buffer (frame binding 10,
// lightmap.rs), after the baked and realtime volumes, so no lit binding is
// added. The header's rows GI_HEADER_AT.. describe them:
//   row 0: x cascades in use (0: off), y the bounce's scale, z the normal
//          bias and w the view bias (in spacings)
//   per cascade c, rows 1 + 2c and 2 + 2c: (its first cell, spacing) and
//          (its probes along x, y, z, its first probe's vec4)
// A cascade is a grid of probes ahead of the camera (gi_probes.rs `plan`)
// on the world's cells of its spacing: the probe of world cell w sits at
// w × spacing (plus its relocation offset) and is stored at slot w mod
// counts, so the grid scrolls with the camera without moving what it
// keeps. A probe's record
// (GI_STRIDE vec4):
//   0: its state: xyz its offset (spacings), w its key (+ active, - inside
//      geometry, 0 never traced): which world cell the slot holds now
//   1..4: its irradiance over π, `[c1.x, c1.y, c1.z, c0]` per channel
//   4..: its distance moments (mean, mean²) on a GI_DEPTH² octahedral map,
//      two texels a vec4

const GI_HEADER_AT: u32 = 16u;
const GI_CASCADES: u32 = 4u;
const GI_DEPTH: u32 = 12u;
const GI_STRIDE: u32 = 76u;
const GI_SH_AT: u32 = 1u;
const GI_DEPTH_AT: u32 = 4u;
// The radiance's octahedral texels the blend filters in time (6 × 6).
const GI_RADIANCE: u32 = 6u;
const GI_PI: f32 = 3.14159265359;

// The texel of an n × n octahedral map that texel (x, y), up to one past
// an edge, stands for: across an edge the map folds over itself, so a
// bilinear tap there reads the mirrored texel (Wicked's border copies,
// DDGI_DEPTH_BORDER_OFFSETS, computed instead of stored). gi_probes.rs
// `oct_wrap` is the same function, tested.
fn gi_oct_wrap(p: vec2<i32>, n: i32) -> vec2<i32> {
  let c = clamp(p, vec2<i32>(0), vec2<i32>(n - 1));
  let out_x = p.x < 0 || p.x >= n;
  let out_y = p.y < 0 || p.y >= n;
  if (out_x && out_y) { return vec2<i32>(n - 1) - c; }
  if (out_x) { return vec2<i32>(c.x, n - 1 - c.y); }
  if (out_y) { return vec2<i32>(n - 1 - c.x, c.y); }
  return p;
}

// The slot of world cell `w` in a grid of `counts` probes, and the key
// that says which of the cells sharing that slot it holds (gi_probes.rs
// `slot_and_key`, tested): the cell's wrap count along each axis, 8 bits
// each, plus one (0 is a slot never traced).
fn gi_slot(w: vec3<i32>, counts: vec3<i32>) -> vec3<i32> {
  return ((w % counts) + counts) % counts;
}

fn gi_key(w: vec3<i32>, counts: vec3<i32>) -> f32 {
  let g = ((w - gi_slot(w, counts)) / counts) & vec3<i32>(255);
  return f32(g.x + g.y * 256 + g.z * 65536 + 1);
}

fn gi_flat(s: vec3<i32>, counts: vec3<i32>) -> u32 {
  return u32((s.z * counts.y + s.y) * counts.x + s.x);
}

// Spherical Fibonacci point `i` of `n` (diharaw's gi_ray_trace.rgen, as
// Wicked's ddgi_raytraceCS), turned by the frame's random rotation: the
// directions a probe's rays take this frame. The first GI_FIXED_RAYS are
// not turned, so the relocation reads the same rays every frame.
const GI_FIXED_RAYS: u32 = 16u;

fn gi_fibonacci(i: f32, n: f32) -> vec3<f32> {
  let golden = 0.61803398875;
  let phi = 2.0 * GI_PI * fract(i * golden);
  let cos_theta = 1.0 - (2.0 * i + 1.0) / n;
  let sin_theta = sqrt(clamp(1.0 - cos_theta * cos_theta, 0.0, 1.0));
  return vec3<f32>(cos(phi) * sin_theta, sin(phi) * sin_theta, cos_theta);
}

fn gi_ray_dir(i: u32, n: u32, rotation: mat3x3<f32>) -> vec3<f32> {
  if (i < GI_FIXED_RAYS) { return gi_fibonacci(f32(i), f32(GI_FIXED_RAYS)); }
  return normalize(rotation * gi_fibonacci(f32(i - GI_FIXED_RAYS), f32(n - GI_FIXED_RAYS)));
}
