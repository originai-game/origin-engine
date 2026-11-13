// og.Wind's field (wind.rs): the gust patches every presentation-only
// wind reads (grass, trees), its wave's phases and clock, each now and
// last frame (motion vectors), reduced in f64 on the CPU from the tick.
// No bindings: include it before a module that uses it.
//
// Ported from bevy_grass v0.20.3 (src/shaders/common.wesl, wind.wesl;
// third_party/bevy-grass). MIT License, Copyright (c) 2026 Gediminas
// Mikelenas: the full notice is in grass_common.wgsl and
// third_party/bevy-grass/LICENSE.

const WIND_TAU: f32 = 6.283185307;
// Seconds after which the wind's clock starts again (wind.rs TIME_PERIOD).
const WIND_TIME_PERIOD: f32 = 3600.0;
// Metres per cell of the gust noise, and its period in cells (wind.rs).
const WIND_GUST_CELL: f32 = 14.0;
const WIND_GUST_PERIOD: i32 = 256;

// og.Wind's travelling wave: its wavelength (scatter.rs WIND_WAVELENGTH).
const WIND_WAVELENGTH: f32 = 10.0;

// One frame of og.Wind (wind.rs WindFieldGpu).
struct WindField {
  dir: vec4<f32>,     // xy: where it blows (world x, z), z: its strength (0: still), w: its gust
  phase: vec4<f32>,   // xy: its wave's two phases (scatter.rs wind_phases), zw: last frame's
  offset: vec4<f32>,  // xy: how far the gust patches have travelled (cells, within the period), zw: last frame's
  time: vec4<f32>,    // x: seconds (within WIND_TIME_PERIOD), y: last frame's, zw: the frame's origin's x and z (origin.rs)
  // The simulated field (wind_field.rs): its fine level's min corner
  // (x, z, from the frame's origin), cell (m), and 1 when it runs (0: the
  // analytic wind alone); its coarse level's corner, cell, and cells a side.
  grid: vec4<f32>,
  grid_outer: vec4<f32>,
};

fn wind_hash2(a: u32, b: u32) -> u32 {
  return pcg(a ^ pcg(b));
}

// `frequency` (radians per second) rounded to whole cycles per
// WIND_TIME_PERIOD, so a sine of the clock does not jump where it wraps.
fn wind_periodic(frequency: f32) -> f32 {
  let step = WIND_TAU / WIND_TIME_PERIOD;
  return round(frequency / step) * step;
}

// Value noise in [0, 1], smooth, repeating every WIND_GUST_PERIOD cells.
fn wind_lattice(c: vec2<i32>) -> f32 {
  let m = WIND_GUST_PERIOD - 1;
  return hash_to_unit(wind_hash2(u32(c.x & m), u32(c.y & m) ^ 0x9e3779b9u));
}

fn wind_noise(p: vec2<f32>) -> f32 {
  let i = floor(p);
  let f = p - i;
  let u = f * f * (3.0 - 2.0 * f);
  let c = vec2<i32>(i);
  let a = wind_lattice(c);
  let b = wind_lattice(c + vec2<i32>(1, 0));
  let d = wind_lattice(c + vec2<i32>(0, 1));
  let e = wind_lattice(c + vec2<i32>(1, 1));
  return mix(mix(a, b, u.x), mix(d, e, u.x), u.y);
}

// The gust patches at world point `at` (x, z): two octaves of the noise
// scrolled downwind, in [0, 1]. The second octave's lattice is twice as
// fine and moves twice as far, so both repeat where the offset wraps.
fn wind_gust(w: WindField, at: vec2<f32>, previous: bool) -> f32 {
  let offset = select(w.offset.xy, w.offset.zw, previous);
  let p = at / WIND_GUST_CELL - offset;
  return 0.65 * wind_noise(p) + 0.35 * wind_noise(p * 2.0 + vec2<f32>(37.0, 11.0));
}

// The gusts' share at world point `at` (x, z), in [-1, 1]: the travelling
// wave of two octaves mixed with the gust patches, so a gust rolls across
// the field as a band and in patches. What every tier leans by (grass,
// scattered copies, cards, trees, swaying models) and what the simulated
// field is forced towards (wind_field.wgsl).
fn wind_gust_share(w: WindField, at: vec2<f32>, previous: bool) -> f32 {
  let dir = w.dir.xy;
  let phase = select(w.phase.xy, w.phase.zw, previous);
  let x = WIND_TAU / WIND_WAVELENGTH * dot(at, dir);
  let wave = (sin(x - phase.x) + 0.5 * sin(2.3 * x + 1.7 - phase.y)) / 1.5;
  return clamp(0.5 * wave + 1.4 * (wind_gust(w, at, previous) - 0.5), -1.0, 1.0);
}

// og.Wind's lean at world point `at` without the simulated field: downwind
// by its strength times one plus its gust times the gusts' share.
fn wind_lean_analytic(w: WindField, at: vec2<f32>, previous: bool) -> vec2<f32> {
  if (w.dir.z <= 0.0) { return vec2<f32>(0.0); }
  return w.dir.xy * w.dir.z * (1.0 + w.dir.w * wind_gust_share(w, at, previous));
}
