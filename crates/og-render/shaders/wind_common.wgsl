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

// One frame of og.Wind (wind.rs WindFieldGpu).
struct WindField {
  dir: vec4<f32>,     // xy: where it blows (world x, z), z: its strength (0: still), w: its gust
  phase: vec4<f32>,   // xy: its wave's two phases (scatter.rs wind_phases), zw: last frame's
  offset: vec4<f32>,  // xy: how far the gust patches have travelled (cells, within the period), zw: last frame's
  time: vec4<f32>,    // x: seconds (within WIND_TIME_PERIOD), y: last frame's, zw: the frame's origin's x and z (origin.rs)
};

// PCG hash (Jarzynski and Olano, 2020). grass.rs `hash`.
fn wind_hash(x: u32) -> u32 {
  let state = x * 747796405u + 2891336453u;
  let word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
  return (word >> 22u) ^ word;
}

fn wind_hash2(a: u32, b: u32) -> u32 {
  return wind_hash(a ^ wind_hash(b));
}

// A hash in [0, 1) with 24 bits.
fn wind_unit(h: u32) -> f32 {
  return f32(h >> 8u) / 16777216.0;
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
  return wind_unit(wind_hash2(u32(c.x & m), u32(c.y & m) ^ 0x9e3779b9u));
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
