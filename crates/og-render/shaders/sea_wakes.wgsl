// W3: the wakes of the hulls in a sea (sea_wakes.rs), a camera-centred square of dynamic waves over the FFT sea,
// stepped once per tick of the wakes and drawn on top of it (water.wgsl `sea_wake`). Picture only: buoyancy stays on
// the tick's waves.
//
// The waves are eWave's (Tessendorf, "eWave: Using an Exponential Solver on the iWave Problem", 2014; the algorithm,
// re-derived here): linear deep water, h'' = -g sqrt(-laplacian) (h + p), solved exactly per Fourier mode over the
// tick, each wave vector k turning (U, V) by its own omega = sqrt(g |k|), so every wavelength runs at deep water's own
// speed, a moving hull draws a Kelvin wedge of 19.5 degrees with its transverse and diverging waves (rather than the
// Mach cone of a plain wave equation), and the step is stable at any tick. A hull is a pressure patch p, its draft
// under its footprint: a still hull settles in its own dent with no waves, a moving one radiates its bow wave, its wake
// and its stern's trough. The grid's edge soaks the waves up in real space, so the transform's wrap never shows.
// Foam is laid where a crest grows steep enough to break and along a moving hull's sides, then spreads and thins.
//
// A step is four passes: `press` moves the grid with the camera and writes the hulls' dent and their side foam (state
// A into B); `rows` transforms (h + p, v) along x; `columns` along z, turns each mode, and transforms back along z;
// `unrows` back along x into the new state (B and the spectrum into A). `compose` writes what the sea draws.
//
// The transform (bit-reversed loads, radix-2 butterflies in shared memory, two complex signals per texel) is
// ocean_fft.wgsl's, ported from Noniv/web-ocean's js/wgsl/sim.js (MIT, Copyright (c) 2026 Maksymilian Dendura;
// notice: third_party/ocean/).

struct Params {
  // x: the grid's side (points, N), y: its cell (m), z: the step's dt (s), w: the waves' damping (1/s)
  size: vec4<f32>,
  // xy: the cells the grid moved since the last step (`press` reads from i + shift), z: the tick's first cut, w: its cuts
  shift: vec4<f32>,
  // xy: the grid's centre (frame xz), z: dynamicWaves (the drawn waves' scale), w: the foam kept a step
  centre: vec4<f32>,
  // x: the foam laid a step at full steepness, y: the edge's fade (a fraction of the side), z: the hulls' side foam a
  // step, w: the highest crest (m; a trough twice as deep)
  foam: vec4<f32>,
};

// A hull as it cut the sea after the tick: its footprint's centre (frame xz) and own x axis (xz), then its half
// extents along its axes, its draft (m) and its speed over the water (m/s).
struct Cut {
  frame: vec4<f32>,
  extra: vec4<f32>,
};

const N: u32 = 256u;
const LOGN: u32 = 8u;
const PI: f32 = 3.14159265358979;
const G: f32 = 9.81;

@group(0) @binding(0) var<uniform> params: Params;
// (height, vertical velocity, foam, the hulls' dent) a point.
@group(0) @binding(1) var state_in: texture_2d<f32>;
@group(0) @binding(2) var state_out: texture_storage_2d<rgba32float, write>;
@group(0) @binding(3) var<storage, read> cuts: array<Cut>;
// The spectrum between the passes: per texel (h + p, v) as two complex values, rows of x.
@group(0) @binding(4) var<storage, read_write> spectrum: array<vec4<f32>>;
// What the sea draws: (height, d height / dx, d height / dz, foam), faded at the edge and under the hulls.
@group(0) @binding(5) var wake_out: texture_storage_2d<rgba16float, write>;

var<workgroup> wa: array<vec4<f32>, 256>;

fn cmul(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> { return vec2<f32>(a.x * b.x - a.y * b.y, a.x * b.y + a.y * b.x); }
fn bitrev(i: u32) -> u32 { return reverseBits(i) >> (32u - LOGN); }
fn signed_index(i: u32) -> f32 { return f32(select(i32(i), i32(i) - i32(N), i >= N / 2u)); }

// The transform of wa in place (its input already bit-reversed): y[j] = Σ x[n] e^{sign 2πi jn/N}.
fn butterflies(t: u32, sign: f32) {
  for (var s = 0u; s < LOGN; s++) {
    let half = 1u << s;
    let pos = t & (half - 1u);
    let i0 = ((t >> s) << (s + 1u)) + pos;
    let i1 = i0 + half;
    let ang = sign * PI * f32(pos) / f32(half);
    let w = vec2<f32>(cos(ang), sin(ang));
    let a0 = wa[i0];
    let a1 = wa[i1];
    let ta = vec4<f32>(cmul(a1.xy, w), cmul(a1.zw, w));
    wa[i0] = a0 + ta;
    wa[i1] = a0 - ta;
    workgroupBarrier();
  }
}

// The state at grid point `at`, calm water off the grid.
fn load(at: vec2<i32>) -> vec4<f32> {
  if (any(at < vec2<i32>(0)) || any(at >= vec2<i32>(i32(N)))) { return vec4<f32>(0.0); }
  return textureLoad(state_in, at, 0);
}

// The point's xz in the frame.
fn frame_xz(i: vec2<i32>) -> vec2<f32> {
  return params.centre.xy + (vec2<f32>(i) - vec2<f32>(0.5 * f32(N - 1u))) * params.size.y;
}

// How much of the point at `p` a hull covers: its rectangle grown by `grow` metres, its corners rounded, its edge
// softened over `soft` metres.
fn hull_cover(p: vec2<f32>, c: Cut, grow: f32, soft: f32) -> f32 {
  let d = p - c.frame.xy;
  let a = c.frame.zw;
  let local = abs(vec2<f32>(dot(d, a), dot(d, vec2<f32>(-a.y, a.x))));
  let half = c.extra.xy + vec2<f32>(grow);
  let round_ = min(half.x, half.y) * 0.5;
  let q = local - half + vec2<f32>(round_);
  let out = length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - round_;
  return 1.0 - smoothstep(-soft, soft, out);
}

// 1 inside the square, falling to 0 over its last `params.foam.y` at each edge: the waves die there unseen.
fn inside(i: vec2<i32>) -> f32 {
  let q = vec2<f32>(i) / f32(N - 1u);
  let d = min(min(q.x, q.y), min(1.0 - q.x, 1.0 - q.y));
  return smoothstep(0.0, max(params.foam.y, 1e-3), d);
}

@compute @workgroup_size(8, 8)
fn press(@builtin(global_invocation_id) id: vec3<u32>) {
  if (id.x >= N || id.y >= N) { return; }
  let i = vec2<i32>(id.xy);
  let p = frame_xz(i);
  let cell = params.size.y;
  var dent = 0.0;
  var side_foam = 0.0;
  let first = u32(params.shift.z);
  for (var k = 0u; k < u32(params.shift.w); k++) {
    let c = cuts[first + k];
    let soft = max(1.5 * cell, 0.3 * min(c.extra.x, c.extra.y));
    dent = max(dent, c.extra.z * hull_cover(p, c, 0.0, soft));
    // Contact foam: a band some two cells wide along the hull's sides, as it moves.
    let band = hull_cover(p, c, 2.5 * cell, cell) - hull_cover(p, c, 0.0, cell);
    side_foam = max(side_foam, max(band, 0.0) * smoothstep(0.5, 3.0, c.extra.w));
  }
  let s = load(i + vec2<i32>(params.shift.xy));
  textureStore(state_out, i, vec4<f32>(s.xy, min(s.z + side_foam * params.foam.z, 1.0), dent));
}

// Rows: workgroup z transforms row z of (h + p, v) along x.
@compute @workgroup_size(128)
fn rows(@builtin(workgroup_id) wg: vec3<u32>, @builtin(local_invocation_index) t: u32) {
  let z = wg.x;
  for (var e = t; e < N; e += 128u) {
    let s = textureLoad(state_in, vec2<u32>(e, z), 0);
    wa[bitrev(e)] = vec4<f32>(s.x + s.w, 0.0, s.y, 0.0);
  }
  workgroupBarrier();
  butterflies(t, -1.0);
  for (var e = t; e < N; e += 128u) {
    spectrum[z * N + e] = wa[e];
  }
}

// Columns: workgroup x transforms column x along z, turns each mode over the step, and transforms it back.
@compute @workgroup_size(128)
fn columns(@builtin(workgroup_id) wg: vec3<u32>, @builtin(local_invocation_index) t: u32) {
  let x = wg.x;
  for (var e = t; e < N; e += 128u) {
    wa[bitrev(e)] = spectrum[e * N + x];
  }
  workgroupBarrier();
  butterflies(t, -1.0);
  let dk = 2.0 * PI / (f32(N) * params.size.y);
  let dt = params.size.z;
  var turned: array<vec4<f32>, 2>;
  for (var r = 0u; r < 2u; r++) {
    let e = t + r * 128u;
    let k = length(vec2<f32>(signed_index(x), signed_index(e))) * dk;
    let w = sqrt(G * k);
    let c = cos(w * dt);
    // sin(w dt) / w, dt for the mean.
    let sw = select(sin(w * dt) / max(w, 1e-6), dt, w < 1e-4);
    // The damping: the grid's own plus viscosity's k², which takes the cell-sized waves the hulls' edges make within a
    // few ticks and leaves the wake's.
    let keep = exp(-(params.size.w + 0.6 * k * k * params.size.y * params.size.y) * dt);
    let m = wa[e];
    let u = m.xy;
    let v = m.zw;
    turned[r] = vec4<f32>((u * c + v * sw) * keep, (v * c - u * (w * w * sw)) * keep);
  }
  workgroupBarrier();
  for (var r = 0u; r < 2u; r++) {
    let e = t + r * 128u;
    wa[bitrev(e)] = turned[r];
  }
  workgroupBarrier();
  butterflies(t, 1.0);
  for (var e = t; e < N; e += 128u) {
    spectrum[e * N + x] = wa[e];
  }
}

// Back along x: workgroup z writes row z of the new state (the height less the dent, the velocity, the foam).
@compute @workgroup_size(128)
fn unrows(@builtin(workgroup_id) wg: vec3<u32>, @builtin(local_invocation_index) t: u32) {
  let z = wg.x;
  for (var e = t; e < N; e += 128u) {
    wa[bitrev(e)] = spectrum[z * N + e];
  }
  workgroupBarrier();
  butterflies(t, 1.0);
  let norm = 1.0 / f32(N * N);
  let cap = params.foam.w;
  for (var e = t; e < N; e += 128u) {
    let i = vec2<i32>(i32(e), i32(z));
    let s = load(i);
    let edge = inside(i);
    let h = clamp(wa[e].x * norm - s.w, -2.0 * cap, cap) * mix(0.96, 1.0, edge);
    let v = wa[e].z * norm * mix(0.9, 1.0, edge);
    // Foam where a crest grows steep enough to break (slopes past some 0.2, as a Stokes wave's limit nears; the step's
    // start), spread to its neighbours a little and thinning.
    let ea = load(i + vec2<i32>(1, 0));
    let we = load(i + vec2<i32>(-1, 0));
    let so = load(i + vec2<i32>(0, 1));
    let no = load(i + vec2<i32>(0, -1));
    let slope = length(vec2<f32>(ea.x - we.x, so.x - no.x)) / (2.0 * params.size.y);
    let breaking = smoothstep(0.18, 0.4, slope) * step(0.0, s.x) * (1.0 - smoothstep(0.02, 0.2, s.w));
    let spread = (ea.z + we.z + so.z + no.z) * 0.25;
    let foam = clamp(mix(s.z, spread, 0.1) * params.centre.w + breaking * params.foam.x, 0.0, 1.0) * edge;
    textureStore(state_out, i, vec4<f32>(h, v, foam, s.w));
  }
}

@compute @workgroup_size(8, 8)
fn compose(@builtin(global_invocation_id) id: vec3<u32>) {
  if (id.x >= N || id.y >= N) { return; }
  let i = vec2<i32>(id.xy);
  let here = load(i);
  // Under a hull the water is the hull's: nothing drawn there (its dent is the pressure, not a hole).
  let open_ = 1.0 - smoothstep(0.02, 0.2, here.w);
  let fade = inside(i) * params.centre.z * open_;
  let grad = vec2<f32>(load(i + vec2<i32>(1, 0)).x - load(i - vec2<i32>(1, 0)).x,
    load(i + vec2<i32>(0, 1)).x - load(i - vec2<i32>(0, 1)).x) / (2.0 * params.size.y);
  textureStore(wake_out, i, vec4<f32>(here.x * fade, grad * fade, here.z * inside(i) * min(params.centre.z, 1.0)));
}

// Calm water: the state written by this group set to zero.
@compute @workgroup_size(8, 8)
fn calm(@builtin(global_invocation_id) id: vec3<u32>) {
  if (id.x >= N || id.y >= N) { return; }
  textureStore(state_out, vec2<i32>(id.xy), vec4<f32>(0.0));
}
