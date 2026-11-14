// Value noise over hash.wgsl's lattice, one copy for every shader
// (LIBRARY_WGSL, after color.wgsl). `value_noise3` matches
// `og_render::particles::value_noise3` operation for operation, so the CPU
// and GPU particle paths draw the same random numbers.

// Bilinear value noise with a smoothstep fade over the plane, in [0, 1):
// the lattice's layer `layer` (z), so two users with different layers draw
// unrelated patterns.
fn value_noise2(p: vec2<f32>, layer: i32) -> f32 {
  let fl = floor(p);
  let f = p - fl;
  let u = f * f * (3.0 - 2.0 * f);
  let c = vec2<i32>(fl);
  let a = lattice3(vec3<i32>(c, layer), 0u);
  let b = lattice3(vec3<i32>(c + vec2<i32>(1, 0), layer), 0u);
  let d = lattice3(vec3<i32>(c + vec2<i32>(0, 1), layer), 0u);
  let e = lattice3(vec3<i32>(c + vec2<i32>(1, 1), layer), 0u);
  return mix(mix(a, b, u.x), mix(d, e, u.x), u.y);
}

// Trilinear value noise with a smoothstep fade, in [0, 1).
fn value_noise3(p: vec3<f32>, seed: u32) -> f32 {
  return value_noise3_at(p, vec3<i32>(0), seed);
}

// value_noise3 at `p` plus the whole lattice cells `whole` (a point far
// from the world's zero, from its place relative to the render origin:
// origin.rs).
fn value_noise3_at(p: vec3<f32>, whole: vec3<i32>, seed: u32) -> f32 {
  let fl = floor(p);
  let i = vec3<i32>(fl) + whole;
  let f = p - fl;
  let u = f * f * (3.0 - 2.0 * f);
  let c000 = lattice3(i, seed);
  let c100 = lattice3(i + vec3<i32>(1, 0, 0), seed);
  let c010 = lattice3(i + vec3<i32>(0, 1, 0), seed);
  let c110 = lattice3(i + vec3<i32>(1, 1, 0), seed);
  let c001 = lattice3(i + vec3<i32>(0, 0, 1), seed);
  let c101 = lattice3(i + vec3<i32>(1, 0, 1), seed);
  let c011 = lattice3(i + vec3<i32>(0, 1, 1), seed);
  let c111 = lattice3(i + vec3<i32>(1, 1, 1), seed);
  let x00 = c000 + (c100 - c000) * u.x;
  let x10 = c010 + (c110 - c010) * u.x;
  let x01 = c001 + (c101 - c001) * u.x;
  let x11 = c011 + (c111 - c011) * u.x;
  let y0 = x00 + (x10 - x00) * u.y;
  let y1 = x01 + (x11 - x01) * u.y;
  return y0 + (y1 - y0) * u.z;
}
