// The clouds' noise volume (clouds.rs): 128³ texels tiling in every axis,
// after Schneider, "The Real-time Volumetric Cloudscapes of Horizon Zero
// Dawn" (SIGGRAPH 2015) and "Nubis: Authoring Real-Time Volumetric
// Cloudscapes with the Decima Engine" (SIGGRAPH 2017); third_party/nubis-clouds.md.
//   r: the base shape: a Perlin fBm dilated by a low-frequency Worley fBm
//      (Perlin-Worley), then carved by Worley fBms at 4, 8 and 16 cells,
//      which the papers do at each sample (their g, b and a channels) and
//      this bakes; equalised: uniform over 0..1, so a threshold of 1 - c
//      leaves a fraction c of the volume in cloud
//   g: Worley fBm at 8 cells, b: at 16 (the detail that erodes the edges)
//   a: the weather map, where the clouds stand: a 2D value fBm at 4 cells
//      (the same down every column of the volume), equalised the same way
// Every lattice point is hashed from its cell wrapped by the frequency, so
// the volume tiles.

@group(0) @binding(0) var noise_out: texture_storage_3d<rgba8unorm, write>;

const N: u32 = 128u;

fn hash(c: vec3<i32>, salt: u32) -> f32 {
  return lattice3(c, salt * 2654435761u);
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

// Gradient noise (Perlin 2002, the quintic fade), about -0.7 to 0.7.
fn perlin(p: vec3<f32>, f: f32) -> f32 {
  let q = p * f;
  let c = vec3<i32>(floor(q));
  let t = fract(q);
  let s = t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
  let fi = i32(f);
  var v = 0.0;
  for (var k = 0; k < 8; k++) {
    let o = vec3<i32>(k & 1, (k >> 1) & 1, (k >> 2) & 1);
    let w = wrap(c + o, fi);
    let g = vec3<f32>(hash(w, 4u), hash(w, 5u), hash(w, 6u)) * 2.0 - 1.0;
    let m = mix(1.0 - s, s, vec3<f32>(o));
    v += dot(g / max(length(g), 1e-4), t - vec3<f32>(o)) * m.x * m.y * m.z;
  }
  return v;
}

// Value noise over x and y alone (the weather map).
fn value2(p: vec2<f32>, f: f32) -> f32 {
  let q = p * f;
  let c = vec2<i32>(floor(q));
  let t = fract(q);
  let s = t * t * (3.0 - 2.0 * t);
  let fi = i32(f);
  var v = 0.0;
  for (var k = 0; k < 4; k++) {
    let o = vec2<i32>(k & 1, k >> 1);
    let m = mix(1.0 - s, s, vec2<f32>(o));
    v += hash(wrap(vec3<i32>(c + o, 0), fi), 7u) * m.x * m.y;
  }
  return v;
}

// The quantiles of the two channels before equalising, at sixteenths
// (measured over this volume).
const SHAPE_Q = array<f32, 17>(0.6125, 0.7444, 0.7671, 0.7823, 0.7945, 0.8054, 0.8154, 0.8248, 0.8337, 0.8426, 0.8515, 0.8606, 0.87, 0.8805, 0.8926, 0.9091, 0.9815);
const COVER_Q = array<f32, 17>(0.1028, 0.2098, 0.2403, 0.2592, 0.282, 0.3022, 0.3226, 0.3409, 0.3591, 0.3832, 0.4147, 0.4532, 0.4991, 0.5439, 0.5923, 0.6439, 0.7682);

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
  let w4 = worley(p, 4.0);
  let w8 = worley(p, 8.0);
  let w16 = worley(p, 16.0);
  let w32 = worley(p, 32.0);
  // Worley fBms at 4, 8 and 16 cells (the last without the octave a texel
  // cannot hold).
  let f4 = w4 * 0.625 + w8 * 0.25 + w16 * 0.125;
  let f8 = w8 * 0.625 + w16 * 0.25 + w32 * 0.125;
  let f16 = w16 * 0.75 + w32 * 0.25;
  let pf = clamp((perlin(p, 4.0) * 0.5 + perlin(p, 8.0) * 0.25 + perlin(p, 16.0) * 0.125) * 1.2 + 0.5, 0.0, 1.0);
  // Perlin-Worley: the Perlin fBm fills the Worley cells' gaps, so the
  // shapes billow; then the three fBms carve it, as the papers' sampler
  // does (remap(r, fbm - 1, 1, 0, 1)).
  let pw = f4 + pf * (1.0 - f4);
  let carve = f4 * 0.625 + f8 * 0.25 + f16 * 0.125;
  let shape = equalise((pw - carve + 1.0) / (2.0 - carve), SHAPE_Q);
  let cover = equalise((value2(p.xy, 4.0) * 0.5 + value2(p.xy, 8.0) * 0.25 + value2(p.xy, 16.0) * 0.125) / 0.875, COVER_Q);
  textureStore(noise_out, vec3<i32>(id), vec4<f32>(shape, f8, f16, cover));
}
