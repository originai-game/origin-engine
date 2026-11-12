// W2: a sea's surface curvature about the camera (water_caustics.wgsl), a uniform (the lit pipelines' storage
// buffers are spent): per texel its Hessian (xx, zz, xz) as three signed 10-bit words, each the square root of its
// share of CAUSTIC_H (finer near zero, where most of the sea's curvature lies).
struct CausticGrid { w: array<vec4<u32>, 1024> };
@group(0) @binding(12) var<uniform> caustic_grid: CausticGrid;
const CAUSTIC_H: f32 = 8.0;

fn caustic_word(q: u32) -> f32 {
  let s = (f32(q & 1023u) - 511.0) / 511.0;
  return sign(s) * s * s * CAUSTIC_H;
}

fn caustic_h(i: vec2<i32>, n: i32) -> vec3<f32> {
  let k = u32(clamp(i.y, 0, n - 1) * n + clamp(i.x, 0, n - 1));
  let w = caustic_grid.w[k / 4u][k % 4u];
  return vec3<f32>(caustic_word(w), caustic_word(w >> 10u), caustic_word(w >> 20u));
}

// The waves' own caustics at `p`, `depth` under a sea's surface (water_caustics.wgsl): the irradiance the surface's
// curvature focuses there, 1 / |det(I + d c H)| at the surface point the refracted sun came through, less 1 (0: as
// the flat surface lights it). Past the focus the rays cross and the pattern is chaos the grid cannot hold: the
// determinant's magnitude is held off zero by how far the receiver lies past the waves' focus.
fn wave_caustics(p: vec3<f32>, depth: f32) -> f32 {
  let g = frame.caustics_cells[0];
  let n = i32(g.w);
  let l = normalize(frame.light_dir.xyz);
  if (l.y <= 0.0) { return 0.0; }
  let t = refract(-l, vec3<f32>(0.0, 1.0, 0.0), 1.0 / 1.33);
  let s = p.xz - t.xz * (depth / max(-t.y, 0.05));
  let u = (s - g.xy) / g.z;
  let i = vec2<i32>(floor(u));
  let f = u - floor(u);
  let h = mix(mix(caustic_h(i, n), caustic_h(i + vec2<i32>(1, 0), n), f.x),
              mix(caustic_h(i + vec2<i32>(0, 1), n), caustic_h(i + vec2<i32>(1, 1), n), f.x), f.y);
  // Its edge fades out (the grid holds 25.6 m about the camera).
  let edge = max(abs(u.x / f32(n) - 0.5), abs(u.y / f32(n) - 0.5)) * 2.0;
  let keep = 1.0 - smoothstep(0.7, 1.0, edge);
  let k = depth * 0.248 / max(-t.y, 0.05);
  let det = (1.0 + k * h.x) * (1.0 + k * h.y) - k * k * h.z * h.z;
  let floor_det = 0.25 + 0.04 * depth;
  let e = 1.0 / max(abs(det), floor_det);
  return clamp(e - 1.0, -0.8, 3.0) * keep;
}

// The ripples that focus a lake's caustics: ten travelling waves of about one length, their directions spread round
// the compass. Per wave: its whole cycles across a tile of 16 cells along x and z (so the pattern is exact wherever
// the frame's origin lies: caustics.rs), its share of the curvature over the square of its cycles, and the square
// root of its cycles (its speed: a deep-water wave's).
const RIPPLES: i32 = 10;
// Their curvature (1/m) at 0.7 cells per metre: a breeze's wavelets, a metre or two long and a few centimetres high.
const RIPPLE_CURVATURE: f32 = 1.8;
// How far past the flat surface's light an octave has bent it where it is drawn alone: its bright lines just formed.
const RIPPLE_FOCUS: f32 = 0.35;
// The lines' share of the sun's light on the bed. They reach six times the flat surface's light where the callers
// add them times the albedo and the sun (a diffuse surface returns 1 / pi of that), so at their full value every
// line clipped to white: over pi they are as bright as the light that makes them.
const RIPPLE_LIGHT: f32 = 0.31830989;

fn ripple(i: i32) -> vec4<f32> {
  var waves = array<vec4<f32>, 10>(
    vec4<f32>(13.0, 2.0, 0.005780, 3.6267), vec4<f32>(17.0, 9.0, 0.002432, 4.3859),
    vec4<f32>(9.0, 12.0, 0.004444, 3.8730), vec4<f32>(6.0, 22.0, 0.001538, 4.7754),
    vec4<f32>(-2.0, 15.0, 0.004367, 3.8901), vec4<f32>(-9.0, 18.0, 0.002222, 4.4860),
    vec4<f32>(-12.0, 9.0, 0.004444, 3.8730), vec4<f32>(-21.0, 11.0, 0.001423, 4.8690),
    vec4<f32>(-16.0, 3.0, 0.003774, 4.0347), vec4<f32>(-23.0, -6.0, 0.001416, 4.8754));
  return waves[i];
}

// The light one octave of the ripples focuses at `cell` (cells of the surface, from the tile's corner), less 1: the
// ripples `octave` times two as long, bending the light `k` past the flat surface's (the focus near 1 over their
// curvature). 1 / |det(I + k H)|, as `wave_caustics` has a sea's, with H the ripples' own (each a cosine: no
// texture). The light is moved, not added: the pattern is over its own mean (a fit, within 6 % for every `k`).
fn ripple_light(cell: vec2<f32>, octave: f32, k: f32, seconds: f32) -> f32 {
  let size = exp2(octave);
  let u = fract(cell / (16.0 * size)) * 6.2831853;
  let rate = seconds * inverseSqrt(size);
  var h = vec3<f32>(0.0);
  for (var i = 0; i < RIPPLES; i++) {
    let w = ripple(i);
    let a = cos(dot(w.xy, u) + rate * w.w + f32(i) * 2.4 + octave * 1.7) * w.z;
    h += vec3<f32>(w.x * w.x, w.y * w.y, w.x * w.y) * a;
  }
  h *= RIPPLE_CURVATURE;
  let det = (1.0 + k * h.x) * (1.0 + k * h.y) - k * k * h.z * h.z;
  let e = min(1.0 / max(abs(det), 0.15 + 0.12 * k), 6.0);
  let k2 = k * k;
  let mean = (1.0 + 28.1 * k2) / (1.0 + 0.034 * k2 + 18.3 * k2 * k + 17.7 * k2 * k2);
  return e / mean - 1.0;
}

// A lake's caustics at `p`, `depth` under its surface: what its ripples focus there, traced back along the
// refracted sun to the surface. Ripples of every length cross a lake, the longer ones flatter (their slope as the
// square root of their length), so each depth has the ripples that focus on it: the pattern is drawn in octaves,
// the two about the receiver's depth blended, and its cells grow with depth as its two-thirds power (a hand wide
// in the shallows, metres wide ten metres down) while the lines stay as sharp. Where a pixel holds whole cells
// they fade to their mean.
fn ripple_caustics(p: vec3<f32>, depth: f32) -> f32 {
  let l = normalize(frame.light_dir.xyz);
  if (l.y <= 0.0) { return 0.0; }
  let scale = frame.caustics.z;
  let t = refract(-l, vec3<f32>(0.0, 1.0, 0.0), 1.0 / 1.33);
  let run = depth / max(-t.y, 0.05);
  let cell = (p.xz - t.xz * run) * scale + frame.caustics_cells[0].xy;
  let k = run * 0.248 * scale / 0.7;
  // An octave up the ripples are twice as long and 2^1.5 as flat.
  let at = clamp(log2(max(k / RIPPLE_FOCUS, 1e-4)) / 1.5, -2.0, 4.0);
  let octave = floor(at);
  let k0 = k * exp2(-1.5 * octave);
  // The pixel's reach along the bed (a pixel some 1.2 mrad wide, stretched by a grazing view) in the ripples' radians.
  let to = frame.view_pos.xyz - p;
  let dist = length(to);
  let reach = select(0.0012 * dist * dist / max(abs(to.y), 0.25 * dist), 0.0, frame.view_pos.w > 0.5)
    * 6.7 * scale * exp2(-octave);
  let keep = 1.0 - smoothstep(vec2<f32>(0.5), vec2<f32>(2.0), vec2<f32>(reach, reach * 0.5));
  // The sun's seconds to the ripples': a deep-water wave's sqrt(g k), k a cycle of the tile.
  let seconds = frame.time.x * 1.963 * sqrt(scale);
  var light = 0.0;
  if (keep.x > 0.0) { light = ripple_light(cell, octave, k0, seconds) * keep.x * (1.0 - (at - octave)); }
  if (keep.y > 0.0 && at > octave) {
    light += ripple_light(cell, octave + 1.0, k0 * 0.35355339, seconds) * keep.y * (at - octave);
  }
  return clamp(light, -0.8, 5.0);
}

// Caustics under the frame's water, fading in below the surface: under a lake its ripples' (`ripple_caustics`),
// under a sea (W2) its waves' own (`wave_caustics`).
// Whether a pipeline has the caustics' code (pipeline_features.rs).
override OG_CAUSTICS: bool = true;

fn caustics(p: vec3<f32>) -> f32 {
  if (!OG_CAUSTICS) { return 0.0; }
  let c = frame.caustics;
  if (c.y <= 0.0) { return 0.0; }
  let area = frame.caustics_area;
  if (any(p.xz < area.xy) || any(p.xz > area.zw)) { return 0.0; }
  let depth = c.x - p.y;
  if (depth <= 0.0) { return 0.0; }
  let fade = c.y * smoothstep(0.0, 0.4, depth) * exp(-depth * c.w);
  if (frame.caustics_cells[1].w > 0.5) { return wave_caustics(p, depth) * fade; }
  return ripple_caustics(p, depth) * fade * RIPPLE_LIGHT;
}
