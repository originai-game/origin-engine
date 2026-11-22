// The radix-2 FFT the sea's compute passes run in workgroup memory, one copy
// for ocean_fft.wgsl (the FFT sea's fields) and sea_wakes.wgsl (the hulls'
// wakes): complex products, the bit-reversed load order, a texel's signed
// frequency and each stage's butterfly. Each module keeps its own
// workgroup arrays and loops over the stages itself, and declares the
// transform's size: N points, LOGN = log2 N. Prepended after the library
// (wgsl_library!).
//
// The transform (bit-reversed loads, radix-2 butterflies in shared memory,
// two complex signals per texel) is ported from Noniv/web-ocean's
// js/wgsl/sim.js (MIT, Copyright (c) 2026 Maksymilian Dendura; notice:
// third_party/ocean/).

fn cmul(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> { return vec2<f32>(a.x * b.x - a.y * b.y, a.x * b.y + a.y * b.x); }
fn mul_i(a: vec2<f32>) -> vec2<f32> { return vec2<f32>(-a.y, a.x); }

// Where element `i` is loaded so the butterflies leave the transform in order.
fn bitrev(i: u32) -> u32 { return reverseBits(i) >> (32u - LOGN); }

// Texel `i`'s frequency: 0 to N/2 - 1, then -N/2 to -1.
fn signed_index(i: u32) -> f32 { return f32(select(i32(i), i32(i) - i32(N), i >= N / 2u)); }

// Thread `t`'s butterfly (of N / 2) at stage `s` of the in-place transform
// y[j] = Σ x[n] e^{sign 2πi jn/N} of a bit-reversed input: elements i0 and
// i1 become x[i0] ± w x[i1], with a workgroup barrier after every stage.
struct Butterfly {
  i0: u32,
  i1: u32,
  w: vec2<f32>,
};

fn butterfly(t: u32, s: u32, sign: f32) -> Butterfly {
  let half = 1u << s;
  let pos = t & (half - 1u);
  var o: Butterfly;
  o.i0 = ((t >> s) << (s + 1u)) + pos;
  o.i1 = o.i0 + half;
  let ang = sign * PI * f32(pos) / f32(half);
  o.w = vec2<f32>(cos(ang), sin(ang));
  return o;
}
