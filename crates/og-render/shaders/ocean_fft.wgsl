// The FFT sea's wave fields (R5, ocean_fft/): each cascade's spectrum evolved to the frame's time and transformed to
// the surface, one 256-texel row or column per workgroup in workgroup memory, rows first, then columns.
//
// The transform (bit-reversed loads, radix-2 butterflies in shared memory, two packed complex signals per texel) is
// ported from Noniv/web-ocean's js/wgsl/sim.js (MIT, Copyright (c) 2026 Maksymilian Dendura); the spectrum's layout
// and its time folding (a·e^{∓iωt}, ω = 2π m / 4096 s, m·phase split for float32) from goldflag/blue-water-sea-and-
// sky's ocean/shaders/fft.wgsl (MIT, Copyright (c) 2026 Bill Yang). Notices: third_party/ocean/.
//
// Packed per texel, as web-ocean: A = (Dx + i Dz, h + i ∂Dx/∂z), B = (∂h/∂x + i ∂h/∂z, ∂Dx/∂x + i ∂Dz/∂z), each a real
// field's Hermitian spectrum plus i times another's, so one inverse transform yields both. Written per cascade c into
// the fields array: layer 3c (Dx, h, Dz, ∂Dx/∂z), 3c + 1 (∂h/∂x, ∂h/∂z, ∂Dx/∂x, ∂Dz/∂z), 3c + 2 the slopes' moments
// ((∂h/∂x)², (∂h/∂z)², ∂h/∂x ∂h/∂z, foam) that the mips average (LEAN mapping).

struct FftParams {
  // x: N, y: log2 N, z: the cascade count.
  tiles: vec4<u32>,
  // x: the phase, the time over 4096 s (0 to 1), y: the choppiness λ.
  time: vec4<f32>,
  // Per cascade: its wavenumber step 2π / size.
  dk: vec4<f32>,
};

const N: u32 = 256u;
const LOGN: u32 = 8u;
const TAU: f32 = 6.283185307179586;

@group(0) @binding(0) var<uniform> fft: FftParams;
// Per texel (Re a, Im a, ±m, 0), cascades one after another, rows of x.
@group(0) @binding(1) var<storage, read> spectrum: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> row_a: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read_write> row_b: array<vec4<f32>>;
@group(0) @binding(4) var<storage, read> col_a: array<vec4<f32>>;
@group(0) @binding(5) var<storage, read> col_b: array<vec4<f32>>;
@group(0) @binding(6) var fields: texture_storage_2d_array<rgba16float, write>;

var<workgroup> wa: array<vec4<f32>, 256>;
var<workgroup> wb: array<vec4<f32>, 256>;

fn cmul(a: vec2<f32>, b: vec2<f32>) -> vec2<f32> { return vec2<f32>(a.x * b.x - a.y * b.y, a.x * b.y + a.y * b.x); }
fn mul_i(a: vec2<f32>) -> vec2<f32> { return vec2<f32>(-a.y, a.x); }
fn bitrev(i: u32) -> u32 { return reverseBits(i) >> (32u - LOGN); }
fn signed_index(i: u32) -> f32 { return f32(select(i32(i), i32(i) - i32(N), i >= N / 2u)); }

// The inverse transform of wa and wb in place (their input already bit-reversed): y[j] = Σ x[n] e^{+2πi jn/N}.
fn butterflies(t: u32) {
  for (var s = 0u; s < LOGN; s++) {
    let half = 1u << s;
    let pos = t & (half - 1u);
    let i0 = ((t >> s) << (s + 1u)) + pos;
    let i1 = i0 + half;
    let ang = 3.14159265358979 * f32(pos) / f32(half);
    let w = vec2<f32>(cos(ang), sin(ang));
    let a0 = wa[i0];
    let a1 = wa[i1];
    let b0 = wb[i0];
    let b1 = wb[i1];
    let ta = vec4<f32>(cmul(a1.xy, w), cmul(a1.zw, w));
    let tb = vec4<f32>(cmul(b1.xy, w), cmul(b1.zw, w));
    wa[i0] = a0 + ta;
    wa[i1] = a0 - ta;
    wb[i0] = b0 + tb;
    wb[i1] = b0 - tb;
    workgroupBarrier();
  }
}

// The packed spectra (A, B) of texel (x, z) of cascade c at the frame's time.
fn evolve(x: u32, z: u32, c: u32) -> array<vec4<f32>, 2> {
  let a = spectrum[(c * N + z) * N + x];
  let dk = fft.dk[c];
  let kx = signed_index(x) * dk;
  let kz = signed_index(z) * dk;
  let k = sqrt(kx * kx + kz * kz);
  var out: array<vec4<f32>, 2>;
  if (k < 1e-6 || a.z == 0.0) {
    out[0] = vec4<f32>(0.0);
    out[1] = vec4<f32>(0.0);
    return out;
  }
  // H = a e^{-2πi m phase}, m phase split as 256 high + low so float32 keeps about 1e-5 of a turn.
  let phase = fft.time.x;
  let high = floor(a.z / 256.0);
  let low = a.z - high * 256.0;
  let turn = fract(high * fract(phase * 256.0) + low * phase) * TAU;
  let c_ = cos(turn);
  let s_ = sin(turn);
  let h = vec2<f32>(a.x * c_ + a.y * s_, a.y * c_ - a.x * s_);
  let ih = mul_i(h);
  let lam = fft.time.y;
  let ik = 1.0 / k;
  let dx = ih * (lam * kx * ik);
  let dz = ih * (lam * kz * ik);
  let jxz = h * (-lam * kx * kz * ik);
  let sx = ih * kx;
  let sz = ih * kz;
  let jxx = h * (-lam * kx * kx * ik);
  let jzz = h * (-lam * kz * kz * ik);
  out[0] = vec4<f32>(dx + mul_i(dz), h + mul_i(jxz));
  out[1] = vec4<f32>(sx + mul_i(sz), jxx + mul_i(jzz));
  return out;
}

// Rows: workgroup (z, 0, c) evolves and transforms row z of cascade c along x.
@compute @workgroup_size(128)
fn rows(@builtin(workgroup_id) wg: vec3<u32>, @builtin(local_invocation_index) t: u32) {
  let z = wg.x;
  let c = wg.z;
  let base = (c * N + z) * N;
  let e0 = t;
  let e1 = t + N / 2u;
  let v0 = evolve(e0, z, c);
  let v1 = evolve(e1, z, c);
  wa[bitrev(e0)] = v0[0];
  wb[bitrev(e0)] = v0[1];
  wa[bitrev(e1)] = v1[0];
  wb[bitrev(e1)] = v1[1];
  workgroupBarrier();
  butterflies(t);
  row_a[base + e0] = wa[e0];
  row_a[base + e1] = wa[e1];
  row_b[base + e0] = wb[e0];
  row_b[base + e1] = wb[e1];
}

fn emit(x: u32, z: u32, c: u32, a: vec4<f32>, b: vec4<f32>) {
  let p = vec2<u32>(x, z);
  textureStore(fields, p, 3u * c, vec4<f32>(a.x, a.z, a.y, a.w));
  textureStore(fields, p, 3u * c + 1u, b);
  textureStore(fields, p, 3u * c + 2u, vec4<f32>(b.x * b.x, b.y * b.y, b.x * b.y, 0.0));
}

// Columns: workgroup (x, 0, c) transforms column x of cascade c along z and writes the fields.
@compute @workgroup_size(128)
fn columns(@builtin(workgroup_id) wg: vec3<u32>, @builtin(local_invocation_index) t: u32) {
  let x = wg.x;
  let c = wg.z;
  let e0 = t;
  let e1 = t + N / 2u;
  let i0 = (c * N + e0) * N + x;
  let i1 = (c * N + e1) * N + x;
  wa[bitrev(e0)] = col_a[i0];
  wa[bitrev(e1)] = col_a[i1];
  wb[bitrev(e0)] = col_b[i0];
  wb[bitrev(e1)] = col_b[i1];
  workgroupBarrier();
  butterflies(t);
  emit(x, e0, c, wa[e0], wb[e0]);
  emit(x, e1, c, wa[e1], wb[e1]);
}
