// Grass (grass.rs): what the generation pass, the blade draws and the
// terrain's far tint share. No bindings; needs wind_common.wgsl before it.
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
// A blade's width is stored up to this many times the field's.
const GRASS_WIDTH_RANGE: f32 = 4.0;
// Vertex indices between one level of detail and the next (grass.rs LOD_STRIDE).
const GRASS_LOD_STRIDE: u32 = 64u;
// Steps of last frame's bend per unit of bend.
const GRASS_BEND_STEPS: f32 = 128.0;
// How much a gust brightens the blades it presses down (grass.rs GUST_SHEEN).
const GRASS_GUST_SHEEN: f32 = 1.5;

// How far the wind bends a blade whose spot leans by `lean` (world x, z:
// wind_common.wgsl wind_lean, the simulated field shared with every other
// tier, or og.Wind's analytic lean without it): its tip's offset as a
// share of its height. Each blade also sways on its own (by its seed, in
// [0, 1)) across and along the lean. Softer stems (`stiffness` towards 0)
// bend further.
fn grass_bend(w: WindField, lean: vec2<f32>, seed: f32, stiffness: f32, sway: f32, previous: bool) -> vec2<f32> {
  let l = lean * sway;
  let strength = length(l);
  if (strength <= 1e-6) { return vec2<f32>(0.0); }
  let dir = l / strength;
  let flex = mix(1.6, 0.4, clamp(stiffness, 0.0, 1.0));
  let time = select(w.time.x, w.time.y, previous);
  let frequency = wind_periodic(mix(1.1, 2.4, stiffness) * (0.85 + 0.3 * seed));
  let s = sin(time * frequency + seed * GRASS_TAU);
  let across = vec2<f32>(-dir.y, dir.x);
  let wobble = (dir * 0.5 + across * 0.5) * s * (0.03 + 0.3 * strength) * flex;
  return clamp(l * flex + wobble, vec2<f32>(-1.0), vec2<f32>(1.0));
}

// The shiver of a full-detail blade's tip, in metres sideways: `t` 0 at
// the root and 1 at the tip.
fn grass_flutter(w: WindField, amount: f32, seed: f32, t: f32, height: f32, previous: bool) -> f32 {
  let time = select(w.time.x, w.time.y, previous);
  let shiver = sin(time * wind_periodic(11.0) + seed * 37.0 + t * 4.0);
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
      let h = wind_hash2(wind_hash2(bitcast<u32>(ci.x), bitcast<u32>(ci.y)), seed);
      let site = c + vec2<f32>(hash_to_unit(h), hash_to_unit(pcg(h)));
      let d = dot(site - q, site - q);
      if (d < best) {
        best = d;
        id = h;
        centre = site;
      }
    }
  }
  return vec4<f32>(hash_to_unit(pcg(id ^ 0x5bd1e995u)), hash_to_unit(pcg(id ^ 0x1b873593u)), (q - centre) * size);
}

// How much brighter grass at world point `at` shows where a gust presses
// it down (its paler, glossier sides turned up), by `amount` (grass.rs
// GUST_SHEEN): the same on the blades and on the ground past them, so a
// gust band rolls across both.
fn grass_gust_sheen(w: WindField, at: vec2<f32>, amount: f32) -> f32 {
  let g = wind_gust(w, at, false);
  return 1.0 + amount * w.dir.z * (smoothstep(0.4, 0.75, g) - 0.2);
}

// The share of clumps that are dry at world point `at`, `dryness` of them
// overall: a few scattered everywhere, most gathered into drifts tens of
// metres across (two octaves of the noise, turned off the world's axes)
// with stretches of green between, their edges soft over metres.
fn grass_dry_share(at: vec2<f32>, dryness: f32) -> f32 {
  let q = at / 26.0;
  let r = vec2<f32>(q.x * 0.8 - q.y * 0.6, q.x * 0.6 + q.y * 0.8);
  let drift = 0.7 * wind_noise(r + vec2<f32>(13.0, 71.0)) + 0.3 * wind_noise(r * 2.7 + vec2<f32>(5.0, 29.0));
  return min(dryness * (0.35 + 2.6 * smoothstep(0.36, 0.66, drift)), 1.0);
}

// Whether a clump of tint `clump` at world point `at` is dry: a blade's
// clump is, or is not (grass_dry_share).
fn grass_dry(clump: f32, at: vec2<f32>, dryness: f32) -> f32 {
  return step(fract(clump * 7.31 + 0.17), grass_dry_share(at, dryness));
}

// What a field's blades show on average where they are too far to draw
// one by one: the ground past them and the thinning far blades take it,
// so the field looks from afar as its blades do near.
struct GrassMean {
  sun: f32,  // the sunlight on what is seen, for the light flat ground takes
  through: f32,  // the sunlight its backlit blades scatter to the eye, for the sun's own
  sky: f32,  // the sky it sees (grass_blade.wgsl grass_occlusion's)
  t: f32,    // how far up a blade it is, 0 the root (and the floor), 1 the tip
  reach: f32,  // how far along the ground the eye's ray runs through the blades, m
};

// How high what is seen of a blade stands for its height: a blade tapers
// to its tip (grass_blade.wgsl, 1 - t^1.4), so its lower part shows more.
const GRASS_MEAN_TAPER: f32 = 0.7;
// The sky an upright blade's side takes for the ground's top: half the
// sky, and the dim field around it.
const GRASS_MEAN_SIDE_SKY: f32 = 0.6;
// The glow of backlit blades for one blade's (grass.wgsl's forward lobe,
// 1.5 times its translucency).
const GRASS_MEAN_THROUGH: f32 = 1.5;

// The mean of a canopy (extinction `k` per metre of a ray, `top` m high:
// grass.wgsl grass_canopy's slab; its blades pass `through` of the light
// behind them) on ground of normal `n`, seen from `v` under a sun
// towards `l`. The eye's ray ends `d` deep with density
// k/v_up·exp(-k·d/v_up), or on the floor; the sun reaches that depth with
// exp(-k·d/l_up); both through the same gaps where the eye looks along
// the sun (the hot spot: Kuusk, "The hot spot effect in plant canopy
// reflectance", 1991, as one joint extinction), so a field seen down-sun
// shows no shade and one seen from above shows its dark floor. Its
// blades stand upright facing every way: they take 2/π of the level
// sunlight on the side turned to it (more than flat ground under a low
// sun, less under a high one), and show the eye that side down-sun and
// the other up-sun, where the light comes through them (lighting.wgsl
// leaf_transmission's two parts; it crosses the blades between as
// grass.wgsl grass_transmittance has it, less than the canopy's shade:
// here halfway between it and none). Fitted to the blades' own frames
// from five heights under a high and a low sun.
fn grass_mean(k: f32, top: f32, through: f32, n: vec3<f32>, v: vec3<f32>, l: vec3<f32>) -> GrassMean {
  let v_up = dot(n, v);
  let l_up = max(dot(n, l), 0.0);
  let eye = 1.0 / max(v_up, 0.1);
  let sun = 1.0 / max(l_up, 0.08);
  let depth = k * top;
  let hot = smoothstep(0.6, 1.0, dot(v, l));
  // The floor the eye reaches, and the sun with it.
  let floor_seen = exp(-depth * eye);
  let floor_lit = floor_seen * exp(-depth * (sun - sqrt(eye * sun) * hot));
  let joint = eye + sun - sqrt(eye * sun) * hot;
  let blades_lit = eye / joint * (1.0 - exp(-depth * joint));
  // The blades' sides for the ground's top.
  let v_flat = v - n * v_up;
  let l_flat = l - n * l_up;
  let level = 0.637 * length(l_flat);
  let down_sun = dot(v_flat, l_flat) / max(length(v_flat) * length(l_flat), 1e-4);
  let front = 0.5 + 0.5 * down_sun * (1.0 - max(v_up, 0.0));
  var m: GrassMean;
  m.sun = min(min(level * front / max(l_up, 0.05), 3.0) * blades_lit + floor_lit, 1.0);
  let into_sun = pow(max(-dot(v, l), 0.0), 6.0);
  m.through = level * (1.0 - front) * sqrt(blades_lit * (1.0 - floor_seen))
    * through * (GRASS_MEAN_THROUGH * into_sun + 0.16) * smoothstep(0.0, 0.1, dot(n, l));
  // The mean depth seen, the floor at `top`, the blades GRASS_MEAN_TAPER
  // as high as the slab has them.
  let slab = (1.0 - floor_seen) / max(k * eye, 1e-4);
  let deep = mix(top * (1.0 - floor_seen), slab - top * floor_seen, GRASS_MEAN_TAPER) + top * floor_seen;
  let d = 1.0 + 0.6 * k * deep;
  m.sky = mix(GRASS_MEAN_SIDE_SKY, 1.0, floor_seen) / (d * d);
  m.t = clamp(1.0 - deep / max(top, 1e-4), 0.0, 1.0);
  // How far along the ground the eye's ray runs through the blades.
  m.reach = (1.0 - floor_seen) / max(k, 1e-4) * sqrt(max(1.0 - v_up * v_up, 0.0));
  return m;
}

// The clumps of grass_clump at `p` as the ground shows them from afar:
// x the tint, y how dry (a clump is dry or not: grass_dry, `dry_share` of
// them), each the nearest clump's, turning to its neighbour's over the
// edge between them (blades of both stand there) and to the field's mean
// where a pixel, `footprint` m across, holds several clumps.
fn grass_clump_far(p: vec2<f32>, size: f32, seed: u32, dry_share: f32, footprint: f32) -> vec2<f32> {
  let q = p / size;
  let base = floor(q);
  var best = vec2<f32>(1e9);
  var ids = vec2<u32>(0u);
  for (var dz = -1; dz <= 1; dz++) {
    for (var dx = -1; dx <= 1; dx++) {
      let c = base + vec2<f32>(f32(dx), f32(dz));
      let ci = vec2<i32>(c);
      let h = wind_hash2(wind_hash2(bitcast<u32>(ci.x), bitcast<u32>(ci.y)), seed);
      let site = c + vec2<f32>(hash_to_unit(h), hash_to_unit(pcg(h)));
      let d = dot(site - q, site - q);
      if (d < best.x) {
        best = vec2<f32>(d, best.x);
        ids = vec2<u32>(h, ids.x);
      } else if (d < best.y) {
        best.y = d;
        ids.y = h;
      }
    }
  }
  // The tint as a blade stores it (8 bits), so the same clumps are dry.
  let tint = round(vec2<f32>(hash_to_unit(pcg(ids.x ^ 0x5bd1e995u)), hash_to_unit(pcg(ids.y ^ 0x5bd1e995u))) * 255.0) / 255.0;
  let dry = step(fract(tint * 7.31 + 0.17), vec2<f32>(dry_share));
  let pixel = footprint / size;
  let edge = clamp(0.12 + pixel, 0.12, 1.0);
  let own = 0.5 + 0.5 * smoothstep(0.0, edge, sqrt(best.y) - sqrt(best.x));
  let near = mix(vec2<f32>(tint.y, dry.y), vec2<f32>(tint.x, dry.x), own);
  return mix(near, vec2<f32>(0.5, dry_share), smoothstep(0.5, 2.0, pixel));
}

// A blade's colour `t` up it, without its clump's and its own variation
// (grass_blade.wgsl grass_color).
fn grass_mean_color(base: vec3<f32>, tip: vec3<f32>, t: f32) -> vec3<f32> {
  return mix(base, tip, pow(t, 0.7));
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
// of GRASS_WIDTH_RANGE times its width, 8 each), its clump's tint (8), its
// level of detail (2: 3 is no blade) and its kind (2: grass.rs
// Grass::kinds), w its bend now and last frame (grass_pack_bend, x and z).
struct GrassBlade {
  root: vec3<f32>,
  facing: f32,
  height: f32,
  width: f32,
  seed: f32,
  clump: f32,
  lod: u32,
  kind: u32,
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
    grass_q(b.height, 255.0) | (grass_q(b.width, 255.0) << 8u) | (grass_q(b.clump, 255.0) << 16u) | ((b.lod & 3u) << 24u) | ((b.kind & 3u) << 26u),
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
  b.kind = (words.z >> 26u) & 3u;
  let bx = grass_unpack_bend(words.w & 0xffffu);
  let bz = grass_unpack_bend(words.w >> 16u);
  b.bend = vec2<f32>(bx.x, bz.x);
  b.previous_bend = vec2<f32>(bx.y, bz.y);
  return b;
}

// The four control points of a blade `height` tall whose tip is offset
// by `offset` (a share of its height, world x and z), drooping by `droop`
// (0..1), relative to its root: a cubic Bezier as Ghost of Tsushima's
// blades (Wohllaib, *Procedural Grass in Ghost of Tsushima*, GDC 2021):
// the stem rises from the root, the midpoint control carries the tilt
// and the arch, and the tip lands where the bend puts it, the curve
// scaled so its length stays the blade's (Gravesen's length estimate for
// a cubic, the mean of its chord and its control polygon, twice).
fn grass_curve(height: f32, offset: vec2<f32>, droop_wanted: f32) -> array<vec3<f32>, 4> {
  let wanted = offset * height;
  let leaning = dot(wanted, wanted) > 1e-12;
  let droop = select(0.0, droop_wanted, leaning);
  let horizontal = min(length(wanted) + droop * 0.7 * height, height * 0.95);
  let dir = select(vec2<f32>(0.0), normalize(wanted), horizontal > 1e-6 && leaning);
  let ratio = horizontal / max(height, 1e-6);
  let p0 = vec3<f32>(0.0);
  var p3 = vec3<f32>(dir.x * horizontal, sqrt(max(height * height - horizontal * horizontal, 0.0)) * (1.0 - 0.7 * droop), dir.y * horizontal);
  // The stem: straight up a third of the blade, leaning a little.
  var p1 = vec3<f32>(dir.x * horizontal * 0.05, height * 0.33, dir.y * horizontal * 0.05);
  // The midpoint control: most of the lean, high enough for the arch.
  let arch = mix(height * max(0.72 * (1.0 - 0.5 * ratio), 0.3), height * 0.95, min(droop * 2.0, 1.0));
  var p2 = vec3<f32>(dir.x * horizontal * 0.6, arch, dir.y * horizontal * 0.6);
  for (var i = 0; i < 2; i++) {
    let chord = distance(p3, p0);
    let polygon = distance(p1, p0) + distance(p2, p1) + distance(p3, p2);
    let scale = height / max((chord + polygon) * 0.5, 1e-6);
    p1 *= scale;
    p2 *= scale;
    p3 *= scale;
  }
  return array<vec3<f32>, 4>(p0, p1, p2, p3);
}

// The point of cubic `c` at `t`.
fn grass_curve_point(c: array<vec3<f32>, 4>, t: f32) -> vec3<f32> {
  let u = 1.0 - t;
  return c[0] * (u * u * u) + c[1] * (3.0 * u * u * t) + c[2] * (3.0 * u * t * t) + c[3] * (t * t * t);
}

// The (unnormalised) tangent of cubic `c` at `t`.
fn grass_curve_tangent(c: array<vec3<f32>, 4>, t: f32) -> vec3<f32> {
  let u = 1.0 - t;
  return (c[1] - c[0]) * (3.0 * u * u) + (c[2] - c[1]) * (6.0 * u * t) + (c[3] - c[2]) * (3.0 * t * t);
}
