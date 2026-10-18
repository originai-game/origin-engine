// Hashes and noise shared by the effect shaders. `pcg` and `value_noise3`
// match `og_render::particles::{pcg, value_noise3}` operation for operation,
// so the CPU and GPU particle paths draw the same random numbers.

fn pcg(v: u32) -> u32 {
  let state = v * 747796405u + 2891336453u;
  let word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
  return (word >> 22u) ^ word;
}

// A uniform float in [0, 1) from 24 bits of a hash.
fn hash_unit(v: u32) -> f32 {
  return f32(pcg(v) >> 8u) / 16777216.0;
}

fn lattice3(c: vec3<i32>, seed: u32) -> f32 {
  let h = (bitcast<u32>(c.x) * 73856093u) ^ (bitcast<u32>(c.y) * 19349663u) ^ (bitcast<u32>(c.z) * 83492791u) ^ seed;
  return hash_unit(h);
}

// Trilinear value noise with a smoothstep fade, in [0, 1).
fn value_noise3(p: vec3<f32>, seed: u32) -> f32 {
  let fl = floor(p);
  let i = vec3<i32>(fl);
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
