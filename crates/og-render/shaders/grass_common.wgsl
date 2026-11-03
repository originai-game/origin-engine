// Grass (grass.rs): what the generation pass, the blade draws and the
// terrain's far tint share. No bindings.
//
// Ported from bevy_grass v0.20.3 (src/shaders/common.wesl, wind.wesl;
// third_party/bevy-grass). MIT License, Copyright (c) 2026 Gediminas
// Mikelenas. Permission is hereby granted, free of charge, to any person
// obtaining a copy of this software and associated documentation files
// (the "Software"), to deal in the Software without restriction,
// including without limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of the Software,
// and to permit persons to whom the Software is furnished to do so,
// subject to the following conditions: The above copyright notice and
// this permission notice shall be included in all copies or substantial
// portions of the Software. THE SOFTWARE IS PROVIDED "AS IS", WITHOUT
// WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO
// THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
// NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE
// LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
// WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

const GRASS_TAU: f32 = 6.283185307;
// Seconds after which the wind's clock starts again (grass.rs TIME_PERIOD).
const GRASS_TIME_PERIOD: f32 = 3600.0;
// A blade's width is stored up to this many times the field's.
const GRASS_WIDTH_RANGE: f32 = 4.0;
// Vertex indices between one level of detail and the next (grass.rs LOD_STRIDE).
const GRASS_LOD_STRIDE: u32 = 64u;
// Metres per cell of the gust noise, and its period in cells (grass.rs).
const GRASS_GUST_CELL: f32 = 14.0;
const GRASS_GUST_PERIOD: i32 = 256;
// og.Wind's wave (scatter.rs WIND_WAVELENGTH).
const GRASS_WAVELENGTH: f32 = 10.0;
// Steps of last frame's bend per unit of bend.
const GRASS_BEND_STEPS: f32 = 128.0;

// One frame of og.Wind for the grass (grass.rs GrassWindGpu).
struct GrassWind {
  dir: vec4<f32>,     // xy: where it blows (world x, z), z: its strength (0: still), w: its gust
  phase: vec4<f32>,   // xy: its wave's two phases (scatter.rs wind_phases), zw: last frame's
  offset: vec4<f32>,  // xy: how far the gust patches have travelled (cells, within the period), zw: last frame's
  time: vec4<f32>,    // x: seconds (within GRASS_TIME_PERIOD), y: last frame's, zw: the frame's origin's x and z (origin.rs)
};

// PCG hash (Jarzynski and Olano, 2020). grass.rs `hash`.
fn grass_hash(x: u32) -> u32 {
  let state = x * 747796405u + 2891336453u;
  let word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
  return (word >> 22u) ^ word;
}

fn grass_hash2(a: u32, b: u32) -> u32 {
  return grass_hash(a ^ grass_hash(b));
}

// A hash in [0, 1) with 24 bits.
fn grass_unit(h: u32) -> f32 {
  return f32(h >> 8u) / 16777216.0;
}

// `frequency` (radians per second) rounded to whole cycles per
// GRASS_TIME_PERIOD, so a sine of the clock does not jump where it wraps.
fn grass_periodic(frequency: f32) -> f32 {
  let step = GRASS_TAU / GRASS_TIME_PERIOD;
  return round(frequency / step) * step;
}

// Value noise in [0, 1], smooth, repeating every GRASS_GUST_PERIOD cells.
fn grass_lattice(c: vec2<i32>) -> f32 {
  let m = GRASS_GUST_PERIOD - 1;
  return grass_unit(grass_hash2(u32(c.x & m), u32(c.y & m) ^ 0x9e3779b9u));
}

fn grass_noise(p: vec2<f32>) -> f32 {
  let i = floor(p);
  let f = p - i;
  let u = f * f * (3.0 - 2.0 * f);
  let c = vec2<i32>(i);
  let a = grass_lattice(c);
  let b = grass_lattice(c + vec2<i32>(1, 0));
  let d = grass_lattice(c + vec2<i32>(0, 1));
  let e = grass_lattice(c + vec2<i32>(1, 1));
  return mix(mix(a, b, u.x), mix(d, e, u.x), u.y);
}

// The gust patches at world point `at` (x, z): two octaves of the noise
// scrolled downwind, in [0, 1]. The second octave's lattice is twice as
// fine and moves twice as far, so both repeat where the offset wraps.
fn grass_gust(w: GrassWind, at: vec2<f32>, previous: bool) -> f32 {
  let offset = select(w.offset.xy, w.offset.zw, previous);
  let p = at / GRASS_GUST_CELL - offset;
  return 0.65 * grass_noise(p) + 0.35 * grass_noise(p * 2.0 + vec2<f32>(37.0, 11.0));
}

// How far og.Wind bends a blade standing at world point `at`: its tip's
// offset as a share of its height, in world x and z. The lean is og.Wind's
// (strength times 1 plus gust times its wave), the wave of the scattered
// copies' sway mixed with the gust patches, so a gust rolls across the
// field as a band and in patches; each blade also sways on its own (by
// its seed, in [0, 1)). Softer stems (`stiffness` towards 0) bend further.
fn grass_bend(w: GrassWind, at: vec2<f32>, seed: f32, stiffness: f32, sway: f32, previous: bool) -> vec2<f32> {
  let strength = w.dir.z * sway;
  if (strength <= 0.0) { return vec2<f32>(0.0); }
  let dir = w.dir.xy;
  let phase = select(w.phase.xy, w.phase.zw, previous);
  let x = GRASS_TAU / GRASS_WAVELENGTH * dot(at, dir);
  let wave = (sin(x - phase.x) + 0.5 * sin(2.3 * x + 1.7 - phase.y)) / 1.5;
  let gust = grass_gust(w, at, previous);
  let g = clamp(0.5 * wave + 1.4 * (gust - 0.5), -1.0, 1.0);
  let flex = mix(1.6, 0.4, clamp(stiffness, 0.0, 1.0));
  let push = strength * (1.0 + w.dir.w * g) * flex;
  let time = select(w.time.x, w.time.y, previous);
  let frequency = grass_periodic(mix(1.1, 2.4, stiffness) * (0.85 + 0.3 * seed));
  let s = sin(time * frequency + seed * GRASS_TAU);
  let across = vec2<f32>(-dir.y, dir.x);
  let wobble = (dir * 0.5 + across * 0.5) * s * (0.03 + 0.3 * strength) * flex;
  return clamp(dir * push + wobble, vec2<f32>(-1.0), vec2<f32>(1.0));
}

// The shiver of a full-detail blade's tip, in metres sideways: `t` 0 at
// the root and 1 at the tip.
fn grass_flutter(w: GrassWind, amount: f32, seed: f32, t: f32, height: f32, previous: bool) -> f32 {
  let time = select(w.time.x, w.time.y, previous);
  let shiver = sin(time * grass_periodic(11.0) + seed * 37.0 + t * 4.0);
  return shiver * 0.06 * w.dir.z * amount * t * t * height;
}

// The clump (a Voronoi cell of side `size`) at point `p` of the terrain's
// space: x its tint, y another value of it (its height and dryness), both
// in [0, 1); zw the point's offset from the clump's centre.
fn grass_clump(p: vec2<f32>, size: f32, seed: u32) -> vec4<f32> {
  let q = p / size;
  let base = floor(q);
  var best = 1e9;
  var id = 0u;
  var centre = q;
  for (var dz = -1; dz <= 1; dz++) {
    for (var dx = -1; dx <= 1; dx++) {
      let c = base + vec2<f32>(f32(dx), f32(dz));
      let ci = vec2<i32>(c);
      let h = grass_hash2(grass_hash2(bitcast<u32>(ci.x), bitcast<u32>(ci.y)), seed);
      let site = c + vec2<f32>(grass_unit(h), grass_unit(grass_hash(h)));
      let d = dot(site - q, site - q);
      if (d < best) {
        best = d;
        id = h;
        centre = site;
      }
    }
  }
  return vec4<f32>(grass_unit(grass_hash(id ^ 0x5bd1e995u)), grass_unit(grass_hash(id ^ 0x1b873593u)), (q - centre) * size);
}

// The colour of dried grass as bright as `c`.
fn grass_straw(c: vec3<f32>) -> vec3<f32> {
  return vec3<f32>(dot(c, vec3<f32>(0.3, 0.59, 0.11))) * vec3<f32>(1.25, 1.1, 0.6);
}

// One axis of a blade's bend in 16 bits: the bend in the low 12, and last
// frame's as a 4-bit step from it.
fn grass_pack_bend(bend: f32, previous: f32) -> u32 {
  let now = u32(round(clamp(bend * 0.5 + 0.5, 0.0, 1.0) * 4095.0));
  let decoded = f32(now) / 4095.0 * 2.0 - 1.0;
  let step = clamp(round((previous - decoded) * GRASS_BEND_STEPS), -8.0, 7.0);
  return now | (u32(i32(step) + 8) << 12u);
}

// This frame's and last frame's bend from `grass_pack_bend`'s 16 bits.
fn grass_unpack_bend(bits: u32) -> vec2<f32> {
  let now = f32(bits & 0xfffu) / 4095.0 * 2.0 - 1.0;
  let step = f32(i32((bits >> 12u) & 0xfu) - 8);
  return vec2<f32>(now, now + step / GRASS_BEND_STEPS);
}

// A blade as the generation pass makes it and the draws read it, 16 bytes:
// x the root's x and z (16 bits each, within `extent` of the frame's
// instance origin), y its height (16 bits, within `range`), its facing and
// seed (8 each), z its height and width (shares of the field's tallest and
// of GRASS_WIDTH_RANGE times its width, 8 each), its clump's tint (8) and
// its level of detail (2: 3 is no blade), w its bend now and last frame
// (grass_pack_bend, x and z).
struct GrassBlade {
  root: vec3<f32>,
  facing: f32,
  height: f32,
  width: f32,
  seed: f32,
  clump: f32,
  lod: u32,
  bend: vec2<f32>,
  previous_bend: vec2<f32>,
};

const GRASS_NO_BLADE: u32 = 3u;

fn grass_q(v: f32, max_value: f32) -> u32 {
  return u32(round(clamp(v, 0.0, 1.0) * max_value));
}

fn grass_pack(b: GrassBlade, origin: vec4<f32>, range: f32) -> vec4<u32> {
  let r = b.root - origin.xyz;
  let x = grass_q(r.x / (2.0 * origin.w) + 0.5, 65535.0);
  let z = grass_q(r.z / (2.0 * origin.w) + 0.5, 65535.0);
  let y = grass_q(r.y / (2.0 * range) + 0.5, 65535.0);
  let facing = u32(fract(b.facing) * 256.0) & 0xffu;
  let seed = grass_q(b.seed, 255.0);
  return vec4<u32>(
    x | (z << 16u),
    y | (facing << 16u) | (seed << 24u),
    grass_q(b.height, 255.0) | (grass_q(b.width, 255.0) << 8u) | (grass_q(b.clump, 255.0) << 16u) | ((b.lod & 3u) << 24u),
    grass_pack_bend(b.bend.x, b.previous_bend.x) | (grass_pack_bend(b.bend.y, b.previous_bend.y) << 16u),
  );
}

fn grass_unpack(words: vec4<u32>, origin: vec4<f32>, range: f32) -> GrassBlade {
  var b: GrassBlade;
  b.root = origin.xyz + vec3<f32>(
    (f32(words.x & 0xffffu) / 65535.0 - 0.5) * 2.0 * origin.w,
    (f32(words.y & 0xffffu) / 65535.0 - 0.5) * 2.0 * range,
    (f32(words.x >> 16u) / 65535.0 - 0.5) * 2.0 * origin.w,
  );
  b.facing = f32((words.y >> 16u) & 0xffu) / 256.0;
  b.seed = f32(words.y >> 24u) / 255.0;
  b.height = f32(words.z & 0xffu) / 255.0;
  b.width = f32((words.z >> 8u) & 0xffu) / 255.0;
  b.clump = f32((words.z >> 16u) & 0xffu) / 255.0;
  b.lod = (words.z >> 24u) & 3u;
  let bx = grass_unpack_bend(words.w & 0xffffu);
  let bz = grass_unpack_bend(words.w >> 16u);
  b.bend = vec2<f32>(bx.x, bz.x);
  b.previous_bend = vec2<f32>(bx.y, bz.y);
  return b;
}

// The three control points of a blade `height` tall whose tip is offset
// by `offset` (a share of its height, world x and z), drooping by `droop`
// (0..1), relative to its root: a quadratic Bézier whose length stays the
// blade's (two passes scaling it by its length's estimate).
fn grass_curve(height: f32, offset: vec2<f32>, droop_wanted: f32) -> array<vec3<f32>, 3> {
  let wanted = offset * height;
  let leaning = dot(wanted, wanted) > 1e-12;
  let droop = select(0.0, droop_wanted, leaning);
  let horizontal = min(length(wanted) + droop * 0.7 * height, height * 0.95);
  let side = select(vec2<f32>(0.0), normalize(wanted) * horizontal, horizontal > 1e-6 && leaning);
  let p0 = vec3<f32>(0.0);
  var p2 = vec3<f32>(side.x, sqrt(max(height * height - horizontal * horizontal, 0.0)) * (1.0 - 0.7 * droop), side.y);
  let ratio = horizontal / max(height, 1e-6);
  var p1 = vec3<f32>(0.0, height * max(1.0 - ratio, 0.05 * max(ratio, 1.0)), 0.0);
  p1.y += (height * 0.9 - p1.y) * min(droop * 2.0, 1.0);
  for (var i = 0; i < 2; i++) {
    let chord = distance(p2, p0);
    let polygon = distance(p1, p0) + distance(p2, p1);
    let scale = height / max((2.0 * chord + polygon) / 3.0, 1e-6);
    p1 = p0 + (p1 - p0) * scale;
    p2 = p1 + (p2 - p1) * scale;
  }
  return array<vec3<f32>, 3>(p0, p1, p2);
}
