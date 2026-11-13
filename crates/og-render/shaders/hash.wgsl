// The integer hashes every shader draws its random numbers from, one copy
// (`og_core::hash` is the same functions, operation for operation, so a CPU
// path and a GPU path draw the same numbers). Integer operations only: the
// `fract(sin(x) * 43758.5453)` hashes these replace scaled the last bits of
// sin, which GPUs round differently, into unrelated values, so a cell lit
// on one GPU was dark on another.
//
// With common.wgsl it is LIBRARY_WGSL: the frame group's text starts with
// it (FRAME_WGSL), and a module without the frame group puts it first.

// PCG's output permutation (Jarzynski and Olano, "Hash functions for GPU
// rendering", 2020).
fn pcg(v: u32) -> u32 {
  let state = v * 747796405u + 2891336453u;
  let word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
  return (word >> 22u) ^ word;
}

// lowbias32 (Chris Wellons, public domain): what places the scattered
// copies, as scatter/placement.rs does on the CPU.
fn hash_lowbias(x: u32) -> u32 {
  var h = x;
  h ^= h >> 16u;
  h *= 0x7feb352du;
  h ^= h >> 15u;
  h *= 0x846ca68bu;
  return h ^ (h >> 16u);
}

// A uniform float in [0, 1) from the top 24 bits of hashed bits `h`.
fn hash_to_unit(h: u32) -> f32 {
  return f32(h >> 8u) / 16777216.0;
}

// A uniform float in [0, 1): `v` hashed (pcg).
fn hash_unit(v: u32) -> f32 {
  return hash_to_unit(pcg(v));
}

// A uniform float in [0, 1) for an integer lattice cell and a seed: the
// value noise's lattice (noise.wgsl), the stars, the caustic cells, the
// foam and the weather's cells.
fn lattice3(c: vec3<i32>, seed: u32) -> f32 {
  let h = (bitcast<u32>(c.x) * 73856093u) ^ (bitcast<u32>(c.y) * 19349663u) ^ (bitcast<u32>(c.z) * 83492791u) ^ seed;
  return hash_unit(h);
}
