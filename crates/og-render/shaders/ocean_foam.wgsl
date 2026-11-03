// The FFT sea's foam (R5, ocean_fft/): a tileable micro-structure texture made once, and two camera-centred world
// foam clipmaps (64 m and 512 m, 256 texels a side, addressed toroidally) stepped at fixed steps of the tick's time:
// foam is injected where the drawn surface folds (its Jacobian under a threshold), decays, and drifts downwind.
//
// Ported from Noniv/web-ocean's js/wgsl/sim.js (FOAM_TEX, FOAM_UPDATE) and common.js (pcgHash, gnoise, pnoise, pfbm)
// (MIT, Copyright (c) 2026 Maksymilian Dendura; third_party/ocean/). Changed here: the Jacobian is the drawn surface's,
// the tick's Gerstner waves (og_core::ocean, faded at the tick's grid cell, water.wgsl ocean_displace) with the
// cascades weighted where the tick leaves their band out; 256 texels a level, not 1024; the threshold from the wind's
// whitecap cover at each level's resolution (water.rs foam_thresholds) rather than a slider.

fn pcg_hash(v: u32) -> u32 {
  let s = v * 747796405u + 2891336453u;
  let w = ((s >> ((s >> 28u) + 4u)) ^ s) * 277803737u;
  return (w >> 22u) ^ w;
}

fn hash_grad(i: vec2<i32>) -> vec2<f32> {
  let h = pcg_hash(u32(i.x) * 1664525u + pcg_hash(u32(i.y) + 1013904223u));
  let a = f32(h) * (6.28318530718 / 4294967296.0);
  return vec2<f32>(cos(a), sin(a));
}

fn pnoise(p: vec2<f32>, period: i32) -> f32 {
  let i = vec2<i32>(floor(p));
  let f = p - floor(p);
  let u = f * f * f * (f * (f * 6.0 - 15.0) + 10.0);
  let i00 = (i % period + period) % period;
  let i10 = ((i + vec2<i32>(1, 0)) % period + period) % period;
  let i01 = ((i + vec2<i32>(0, 1)) % period + period) % period;
  let i11 = ((i + vec2<i32>(1, 1)) % period + period) % period;
  let a = dot(hash_grad(i00), f);
  let b = dot(hash_grad(i10), f - vec2<f32>(1.0, 0.0));
  let c = dot(hash_grad(i01), f - vec2<f32>(0.0, 1.0));
  let d = dot(hash_grad(i11), f - vec2<f32>(1.0, 1.0));
  return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn pfbm(p: vec2<f32>, period: i32, oct: i32) -> f32 {
  var s = 0.0;
  var a = 0.5;
  var q = p;
  var per = period;
  for (var o = 0; o < oct; o++) {
    s += a * pnoise(q, per);
    q *= 2.0;
    per *= 2;
    a *= 0.5;
  }
  return s;
}

// ── the micro-structure (web-ocean FOAM_TEX): R bubble lace, G finer lace, B soft density, A streak noise ──

@group(0) @binding(5) var detail_out: texture_storage_2d<rgba16float, write>;

const DETAIL: f32 = 256.0;

fn h2(i: vec2<i32>, salt: u32) -> vec2<f32> {
  let h = pcg_hash((u32(i.x) * 73856093u) ^ pcg_hash(u32(i.y) * 19349663u + salt));
  let hv = pcg_hash(h + 0x9e3779b9u);
  return vec2<f32>(f32(h >> 8u), f32(hv >> 8u)) / 16777216.0;
}

// "Swiss cheese" foam: 1 in the foam, 0 inside bubbles of their own radius.
fn bubbles(uv: vec2<f32>, period: i32, salt: u32, rmin: f32, rmax: f32) -> f32 {
  let w = vec2<f32>(pfbm(uv * f32(period) * 0.5, period / 2, 3), pfbm(uv * f32(period) * 0.5 + 17.0, period / 2, 3));
  let p = uv * f32(period) + w * 0.9;
  let ip = vec2<i32>(floor(p));
  let fp = p - floor(p);
  var v = 9.0;
  for (var j = -1; j <= 1; j++) {
    for (var i = -1; i <= 1; i++) {
      let o = vec2<i32>(i, j);
      let cell = ((ip + o) % period + period) % period;
      let hh = h2(cell, salt);
      let r = vec2<f32>(o) + hh * 0.9 + 0.05 - fp;
      let rad = mix(rmin, rmax, h2(cell, salt + 7u).x);
      v = min(v, length(r) / rad);
    }
  }
  return smoothstep(0.8, 1.25, v);
}

@compute @workgroup_size(8, 8, 1)
fn detail(@builtin(global_invocation_id) id: vec3<u32>) {
  let uv = (vec2<f32>(id.xy) + 0.5) / DETAIL;
  // Irregular holes from warped fbm (big sheets with ragged openings), then small round bubbles.
  let w1 = vec2<f32>(pfbm(uv * 5.0, 5, 3), pfbm(uv * 5.0 + vec2<f32>(3.7, 9.1), 5, 3));
  let m1 = smoothstep(-0.14, 0.06, pfbm(uv * 10.0 + w1 * 1.4, 10, 5));
  let w2 = vec2<f32>(pfbm(uv * 12.0 + 1.3, 12, 2), pfbm(uv * 12.0 + 7.9, 12, 2));
  let m2 = smoothstep(-0.16, 0.04, pfbm(uv * 27.0 + w2 * 0.9, 27, 4));
  let b3 = bubbles(uv, 29, 37u, 0.06, 0.45);
  let b4 = bubbles(uv, 61, 53u, 0.05, 0.42);
  let dens = pfbm(uv * 4.0, 4, 5) * 0.5 + 0.5;
  let fine_d = pfbm(uv * 16.0, 16, 3) * 0.5 + 0.5;
  let big = m1 * (0.55 + 0.45 * m2) * (0.75 + 0.25 * b3);
  let fine = 0.45 + 0.55 * b3 * (0.7 + 0.3 * b4);
  let streak = pfbm(uv * vec2<f32>(4.0, 16.0), 4, 5) * 0.5 + 0.5;
  textureStore(detail_out, id.xy, vec4<f32>(clamp(big * 0.6 + dens * 0.4, 0.0, 1.0),
    clamp(fine * 0.8 + fine_d * 0.2, 0.0, 1.0), dens, streak));
}

// ── the world foam clipmaps (web-ocean FOAM_UPDATE) ──

struct FoamParams {
  // Per level: the camera's texel now (xy) and at the last step (zw), absolute (the world's xz over the texel).
  snap: array<vec4<i32>, 2>,
  // Per level: the frame xz of the corner of texel snap.xy, the texel (m), unused.
  base: array<vec4<f32>, 2>,
  // x the step (s), y the foam's decay (1/s), z unused, w the injection's gain.
  p0: vec4<f32>,
  // x the growth of foam held by a fold, yz its drift downwind (m/s), w the cascades.
  p1: vec4<f32>,
  // xy the surface's centre (frame xz), z the tick's grid's even cell (m), w unused; then its half size (xy).
  tick: vec4<f32>,
  half: vec4<f32>,
  // Per level the Jacobian, as that level resolves it, under which foam is injected (water.rs foam_thresholds).
  thresholds: vec4<f32>,
  // As water.wgsl's WaterUniform: per cascade 1 / tile, uv at the frame's origin, tile; each band's middle; the
  // tick's waves.
  cascades: array<vec4<f32>, 3>,
  bands: vec4<f32>,
  swell: array<vec4<f32>, 64>,
};

const RES: i32 = 256;

@group(0) @binding(0) var<uniform> foam: FoamParams;
@group(0) @binding(1) var fields: texture_2d_array<f32>;
@group(0) @binding(2) var prev: texture_2d_array<f32>;
@group(0) @binding(3) var foam_out: texture_storage_2d_array<rgba16float, write>;
@group(0) @binding(4) var lin: sampler;

fn sea_keep(k: f32, cell: f32) -> f32 {
  return 1.0 - smoothstep(0.785, 1.571, k * cell);
}

// water.wgsl tick_cell.
fn tick_cell(p: vec2<f32>) -> f32 {
  let u = clamp((p - foam.tick.xy) / max(foam.half.xy, vec2<f32>(1e-3)), vec2<f32>(-1.0), vec2<f32>(1.0));
  var g = u;
  for (var i = 0; i < 8; i++) {
    let w = g * (0.15 + 0.85 * g * g);
    g -= (w - u) / (0.15 + 2.55 * g * g);
  }
  let slope = 0.15 + 2.55 * g * g;
  return foam.tick.z * max(slope.x, slope.y);
}

// The drawn surface's Jacobian at frame point p, its waves resolved at `ts` metres.
fn jacobian(p: vec2<f32>, ts: f32) -> f32 {
  let tc = tick_cell(p);
  let spacing = max(ts, tc);
  var jxx = 0.0;
  var jzz = 0.0;
  var jxz = 0.0;
  for (var i = 0u; i < 32u; i++) {
    let a = foam.swell[2u * i];
    let b = foam.swell[2u * i + 1u];
    let s = sin(dot(a.xy, p) + a.w) * sea_keep(length(a.xy), spacing);
    jxx -= b.x * a.x * s;
    jzz -= b.y * a.y * s;
    jxz -= b.x * a.y * s;
  }
  for (var c = 0u; c < u32(foam.p1.w); c++) {
    let t = foam.cascades[c];
    let w = 1.0 - sea_keep(foam.bands[c], tc);
    let lod = log2(max(ts / (t.w / 256.0), 1.0));
    if (w <= 0.0 || lod >= 4.5) { continue; }
    let uv = p * t.x + t.yz;
    let d = textureSampleLevel(fields, lin, uv, 3u * c + 1u, lod);
    let a = textureSampleLevel(fields, lin, uv, 3u * c, lod);
    jxx += d.z * w;
    jzz += d.w * w;
    jxz += a.w * w;
  }
  return (1.0 + jxx) * (1.0 + jzz) - jxz * jxz;
}

fn wrap_index(i: i32, lo: i32) -> i32 {
  return lo + (((i - lo) % RES) + RES) % RES;
}

@compute @workgroup_size(8, 8, 1)
fn update(@builtin(global_invocation_id) id: vec3<u32>) {
  let level = id.z;
  let sn = foam.snap[level];
  let ts = foam.base[level].z;
  let lo = sn.xy - vec2<i32>(RES / 2);
  let g = vec2<i32>(wrap_index(i32(id.x), lo.x), wrap_index(i32(id.y), lo.y));
  let plo = sn.zw - vec2<i32>(RES / 2);
  let in_prev = all(g >= plo) && all(g < plo + vec2<i32>(RES));
  let p = foam.base[level].xy + (vec2<f32>(g - sn.xy) + 0.5) * ts;
  let j = jacobian(p, ts);
  let dt = foam.p0.x;
  var last = vec4<f32>(0.0);
  if (in_prev) {
    // Semi-Lagrangian: the foam drifts downwind. (Texel g lies at index g mod RES in both steps: id.xy.)
    let back = (vec2<f32>(id.xy) + 0.5 - foam.p1.yz * dt / ts) / f32(RES);
    last = textureSampleLevel(prev, lin, back, level, 0.0);
  }
  let inject = clamp((foam.thresholds[level] - j) * foam.p0.w, 0.0, 1.0);
  var f = last.r * exp(-dt * foam.p0.y);
  f = max(f, inject);
  f = min(f + inject * dt * foam.p1.x, 1.6);
  // g: how fresh it is, for the denser, brighter foam of a crest breaking now.
  let fresh = max(last.g * exp(-dt * 1.6), inject);
  textureStore(foam_out, id.xy, level, vec4<f32>(f, fresh, j, 1.0));
}
