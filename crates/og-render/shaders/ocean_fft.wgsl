// The FFT sea's wave fields (R5, ocean_fft/): each cascade's spectrum evolved to the frame's time and transformed to
// the surface, one 256-texel row or column per workgroup in workgroup memory, rows first, then columns.
//
// The transform is fft.wgsl's (ported from Noniv/web-ocean's js/wgsl/sim.js, MIT, Copyright (c) 2026 Maksymilian
// Dendura); the spectrum's layout and its time folding (a·e^{∓iωt}, ω = 2π m / 4096 s, m·phase split for float32) from goldflag/blue-water-sea-and-
// sky's ocean/shaders/fft.wgsl (MIT, Copyright (c) 2026 Bill Yang). Notices: third_party/ocean/.
//
// Packed per texel as one complex signal: (∂h/∂x + i ∂h/∂z), so one inverse
// transform yields both slopes (re = ∂h/∂x, im = ∂h/∂z) with 4x less workgroup
// memory, storage bandwidth and butterfly ALU than the unused 4-signal layout.
// Written per cascade c into the fields array: layer 2c (∂h/∂x, ∂h/∂z), 2c + 1
// the slopes' moments that the mips average (LEAN mapping: (∂h/∂x)², (∂h/∂z)², ∂h/∂x ∂h/∂z).

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

@group(0) @binding(0) var<uniform> fft: FftParams;
// Per texel (Re a, Im a, ±m, 0), cascades one after another, rows of x.
@group(0) @binding(1) var<storage, read> spectrum: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> row_slope: array<vec2<f32>>;
@group(0) @binding(3) var<storage, read> col_slope: array<vec2<f32>>;
@group(0) @binding(4) var fields: texture_storage_2d_array<rgba16float, write>;

var<workgroup> ws: array<vec2<f32>, 256>;

// The inverse transform of ws in place (input already bit-reversed): y[j] = Σ x[n] e^{+2πi jn/N}.
fn butterflies(t: u32) {
  for (var s = 0u; s < LOGN; s++) {
    let f = butterfly(t, s, 1.0);
    let b0 = ws[f.i0];
    let b1 = ws[f.i1];
    let tb = cmul(b1, f.w);
    ws[f.i0] = b0 + tb;
    ws[f.i1] = b0 - tb;
    workgroupBarrier();
  }
}

// The packed slope spectrum (∂h/∂x + i ∂h/∂z) of texel (x, z) of cascade c at the frame's time.
fn evolve(x: u32, z: u32, c: u32) -> vec2<f32> {
  let a = spectrum[(c * N + z) * N + x];
  let dk = fft.dk[c];
  let kx = signed_index(x) * dk;
  let kz = signed_index(z) * dk;
  let k = sqrt(kx * kx + kz * kz);
  if (k < 1e-6 || a.z == 0.0) {
    return vec2<f32>(0.0);
  }
  // H = a e^{-2πi m phase}, m phase split as 256 high + low so float32 keeps about 1e-5 of a turn.
  let phase = fft.time.x;
  let high = floor(a.z / 256.0);
  let low = a.z - high * 256.0;
  let turn = fract(high * fract(phase * 256.0) + low * phase) * 2.0 * PI;
  let c_ = cos(turn);
  let s_ = sin(turn);
  let h = vec2<f32>(a.x * c_ + a.y * s_, a.y * c_ - a.x * s_);
  let ih = mul_i(h);
  return ih * kx + mul_i(ih * kz);
}

// Rows: workgroup (z, 0, c) evolves and transforms row z of cascade c along x.
@compute @workgroup_size(128)
fn rows(@builtin(workgroup_id) wg: vec3<u32>, @builtin(local_invocation_index) t: u32) {
  let z = wg.x;
  let c = wg.z;
  let base = (c * N + z) * N;
  let e0 = t;
  let e1 = t + N / 2u;
  ws[bitrev(e0)] = evolve(e0, z, c);
  ws[bitrev(e1)] = evolve(e1, z, c);
  workgroupBarrier();
  butterflies(t);
  row_slope[base + e0] = ws[e0];
  row_slope[base + e1] = ws[e1];
}

fn emit(x: u32, z: u32, c: u32, s: vec2<f32>) {
  let p = vec2<u32>(x, z);
  textureStore(fields, p, 2u * c, vec4<f32>(s, 0.0, 0.0));
  textureStore(fields, p, 2u * c + 1u, vec4<f32>(s.x * s.x, s.y * s.y, s.x * s.y, 0.0));
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
  ws[bitrev(e0)] = col_slope[i0];
  ws[bitrev(e1)] = col_slope[i1];
  workgroupBarrier();
  butterflies(t);
  emit(x, e0, c, ws[e0]);
  emit(x, e1, c, ws[e1]);
}
