// The clouds' noise volume (clouds.rs): 64³ texels tiling in every axis.
//   r: Perlin–Worley, the base shapes (value noise fBm billowed by Worley
//      fBm), equalised: uniform over 0..1, so a threshold of 1 - c leaves
//      a fraction c of the volume in cloud
//   g: Worley fBm at 8 cells, b: at 16 (the detail that erodes the edges)
//   a: value noise fBm at 2 cells (the coverage that varies across the
//      sky), equalised the same way
// Every lattice point is hashed from its cell wrapped by the frequency, so
// the volume tiles.

@group(0) @binding(0) var noise_out: texture_storage_3d<rgba8unorm, write>;

const N: u32 = 64u;

fn hash(c: vec3<i32>, salt: u32) -> f32 {
  var h = (bitcast<u32>(c.x) * 73856093u) ^ (bitcast<u32>(c.y) * 19349663u) ^ (bitcast<u32>(c.z) * 83492791u) ^ (salt * 2654435761u);
  h = h * 747796405u + 2891336453u;
  h = ((h >> ((h >> 28u) + 4u)) ^ h) * 277803737u;
  h = (h >> 22u) ^ h;
  return f32(h >> 8u) / 16777216.0;
}

fn wrap(c: vec3<i32>, f: i32) -> vec3<i32> {
  return ((c % f) + f) % f;
}

// 1 at a feature point, falling to 0 a cell away (inverted F1).
fn worley(p: vec3<f32>, f: f32) -> f32 {
  let q = p * f;
  let cell = vec3<i32>(floor(q));
  let fi = i32(f);
  var d = 1e9;
  for (var z = -1; z <= 1; z++) {
    for (var y = -1; y <= 1; y++) {
      for (var x = -1; x <= 1; x++) {
        let c = cell + vec3<i32>(x, y, z);
        let w = wrap(c, fi);
        let fp = vec3<f32>(c) + vec3<f32>(hash(w, 1u), hash(w, 2u), hash(w, 3u));
        let v = q - fp;
        d = min(d, dot(v, v));
      }
    }
  }
  return 1.0 - clamp(sqrt(d), 0.0, 1.0);
}

fn value(p: vec3<f32>, f: f32) -> f32 {
  let q = p * f;
  let c = vec3<i32>(floor(q));
  let t = fract(q);
  let s = t * t * (3.0 - 2.0 * t);
  let fi = i32(f);
  var v = 0.0;
  for (var k = 0; k < 8; k++) {
    let o = vec3<i32>(k & 1, (k >> 1) & 1, (k >> 2) & 1);
    let w = mix(1.0 - s, s, vec3<f32>(o));
    v += hash(wrap(c + o, fi), 7u) * w.x * w.y * w.z;
  }
  return v;
}

fn worley_fbm(p: vec3<f32>, f: f32) -> f32 {
  return worley(p, f) * 0.625 + worley(p, f * 2.0) * 0.25 + worley(p, f * 4.0) * 0.125;
}

fn value_fbm(p: vec3<f32>, f: f32) -> f32 {
  return (value(p, f) * 0.5 + value(p, f * 2.0) * 0.25 + value(p, f * 4.0) * 0.125) / 0.875;
}

// The quantiles of the two channels before equalising, at sixteenths
// (measured over this volume: 0.85 Perlin–Worley + 0.15 value noise, whose
// Perlin–Worley alone is 0 over a fifth of it; and the 2-cell value fBm).
const SHAPE_Q = array<f32, 17>(0.0156, 0.0533, 0.065, 0.0776, 0.1004, 0.1412, 0.1773, 0.2097, 0.2398, 0.2686, 0.2967, 0.3258, 0.356, 0.3895, 0.4286, 0.4815, 0.7485);
const COVER_Q = array<f32, 17>(0.1036, 0.2575, 0.2984, 0.3276, 0.3498, 0.3671, 0.3821, 0.3958, 0.4095, 0.4238, 0.438, 0.4526, 0.4685, 0.4883, 0.5128, 0.5483, 0.8053);

// `x` through the cumulative distribution its quantiles `q` sample.
fn equalise(x: f32, q: array<f32, 17>) -> f32 {
  if (x <= q[0]) { return 0.0; }
  for (var i = 1; i < 17; i++) {
    if (x <= q[i]) {
      return (f32(i - 1) + (x - q[i - 1]) / max(q[i] - q[i - 1], 1e-5)) / 16.0;
    }
  }
  return 1.0;
}

@compute @workgroup_size(4, 4, 4)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
  if (any(id >= vec3<u32>(N))) { return; }
  let p = (vec3<f32>(id) + 0.5) / f32(N);
  let w = worley_fbm(p, 4.0);
  let v = value_fbm(p, 4.0);
  // Perlin–Worley (Schneider 2015): the value noise remapped from the
  // Worley fBm's dark gaps up to 1, so the shapes billow.
  let pw = clamp((v - (1.0 - w)) / max(w, 1e-3) * 0.5 + 0.5 * w, 0.0, 1.0);
  let shape = equalise(pw * 0.85 + v * 0.15, SHAPE_Q);
  let cover = equalise(value_fbm(p, 2.0), COVER_Q);
  textureStore(noise_out, vec3<i32>(id), vec4<f32>(shape, worley_fbm(p, 8.0), worley_fbm(p, 16.0), cover));
}
