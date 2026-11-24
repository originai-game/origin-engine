// Value, gradient, cellular and curl noise over hash.wgsl's lattice, one copy
// for every shader (LIBRARY_WGSL, after color.wgsl).
// - `value_noise3` matches `og_render::particles::value_noise3` operation for operation.
// - `grad_noise2`, `grad_noise3` and their periodic/derivative/Worley variants
//   match `og_core::noise` operation for operation, so CPU simulation and GPU
//   procedural rendering draw identical fields for the same seed.

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

// ── Unified C2 gradient, cellular and curl noise (matches og_core::noise) ──

fn noise_hash2i(c: vec2<i32>, seed: u32) -> u32 {
  let h = (bitcast<u32>(c.x) * 73856093u)
    ^ (bitcast<u32>(c.y) * 19349663u)
    ^ (seed * 0x9e3779b9u);
  return pcg(h);
}

fn noise_hash3i(c: vec3<i32>, seed: u32) -> u32 {
  let h = (bitcast<u32>(c.x) * 73856093u)
    ^ (bitcast<u32>(c.y) * 19349663u)
    ^ (bitcast<u32>(c.z) * 83492791u)
    ^ (seed * 0x9e3779b9u);
  return pcg(h);
}

fn noise_wrap_i(i: i32, period: i32) -> i32 {
  if (period <= 0) {
    return i;
  }
  return ((i % period) + period) % period;
}

fn noise_fade(t: vec3<f32>) -> vec3<f32> {
  return t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
}

fn noise_dfade(t: vec3<f32>) -> vec3<f32> {
  return 30.0 * t * t * (t * (t - 2.0) + 1.0);
}

// 16 isotropic unit-length 2D gradients, matching og_core::noise::GRAD2.
fn noise_grad2(h: u32) -> vec2<f32> {
  switch (h >> 28u) {
    case 0u: { return vec2<f32>(1.0, 0.0); }
    case 1u: { return vec2<f32>(0.9238795, 0.38268343); }
    case 2u: { return vec2<f32>(0.70710677, 0.70710677); }
    case 3u: { return vec2<f32>(0.38268343, 0.9238795); }
    case 4u: { return vec2<f32>(0.0, 1.0); }
    case 5u: { return vec2<f32>(-0.38268343, 0.9238795); }
    case 6u: { return vec2<f32>(-0.70710677, 0.70710677); }
    case 7u: { return vec2<f32>(-0.9238795, 0.38268343); }
    case 8u: { return vec2<f32>(-1.0, 0.0); }
    case 9u: { return vec2<f32>(-0.9238795, -0.38268343); }
    case 10u: { return vec2<f32>(-0.70710677, -0.70710677); }
    case 11u: { return vec2<f32>(-0.38268343, -0.9238795); }
    case 12u: { return vec2<f32>(0.0, -1.0); }
    case 13u: { return vec2<f32>(0.38268343, -0.9238795); }
    case 14u: { return vec2<f32>(0.70710677, -0.70710677); }
    default: { return vec2<f32>(0.9238795, -0.38268343); }
  }
}

// 12 unbiased cube-edge 3D gradients, matching og_core::noise::GRAD3.
fn noise_grad3(h: u32) -> vec3<f32> {
  switch (h % 12u) {
    case 0u: { return vec3<f32>(1.0, 1.0, 0.0); }
    case 1u: { return vec3<f32>(-1.0, 1.0, 0.0); }
    case 2u: { return vec3<f32>(1.0, -1.0, 0.0); }
    case 3u: { return vec3<f32>(-1.0, -1.0, 0.0); }
    case 4u: { return vec3<f32>(1.0, 0.0, 1.0); }
    case 5u: { return vec3<f32>(-1.0, 0.0, 1.0); }
    case 6u: { return vec3<f32>(1.0, 0.0, -1.0); }
    case 7u: { return vec3<f32>(-1.0, 0.0, -1.0); }
    case 8u: { return vec3<f32>(0.0, 1.0, 1.0); }
    case 9u: { return vec3<f32>(0.0, -1.0, 1.0); }
    case 10u: { return vec3<f32>(0.0, 1.0, -1.0); }
    default: { return vec3<f32>(0.0, -1.0, -1.0); }
  }
}

// C2 quintic 2D gradient noise in [-1, 1], with optional integer periods (0 = none).
fn grad_noise2_periodic(p: vec2<f32>, period: vec2<i32>, seed: u32) -> f32 {
  let fl = floor(p);
  let i = vec2<i32>(fl);
  let f = p - fl;
  let u = f * f * f * (f * (f * 6.0 - 15.0) + 10.0);
  let x0 = noise_wrap_i(i.x, period.x);
  let x1 = noise_wrap_i(i.x + 1, period.x);
  let y0 = noise_wrap_i(i.y, period.y);
  let y1 = noise_wrap_i(i.y + 1, period.y);
  let n00 = dot(noise_grad2(noise_hash2i(vec2<i32>(x0, y0), seed)), f);
  let n10 = dot(noise_grad2(noise_hash2i(vec2<i32>(x1, y0), seed)), f - vec2<f32>(1.0, 0.0));
  let n01 = dot(noise_grad2(noise_hash2i(vec2<i32>(x0, y1), seed)), f - vec2<f32>(0.0, 1.0));
  let n11 = dot(noise_grad2(noise_hash2i(vec2<i32>(x1, y1), seed)), f - vec2<f32>(1.0, 1.0));
  return mix(mix(n00, n10, u.x), mix(n01, n11, u.x), u.y) * 1.41421356;
}

// C2 quintic 2D gradient noise in [-1, 1].
fn grad_noise2(p: vec2<f32>, seed: u32) -> f32 {
  return grad_noise2_periodic(p, vec2<i32>(0), seed);
}

// C2 2D gradient noise with analytic derivatives: vec3(value, dv/dx, dv/dy).
fn grad_noise2_deriv(p: vec2<f32>, seed: u32) -> vec3<f32> {
  let fl = floor(p);
  let i = vec2<i32>(fl);
  let f = p - fl;
  let u = f * f * f * (f * (f * 6.0 - 15.0) + 10.0);
  let du = 30.0 * f * f * (f * (f - 2.0) + 1.0);
  let g00 = noise_grad2(noise_hash2i(i, seed));
  let g10 = noise_grad2(noise_hash2i(i + vec2<i32>(1, 0), seed));
  let g01 = noise_grad2(noise_hash2i(i + vec2<i32>(0, 1), seed));
  let g11 = noise_grad2(noise_hash2i(i + vec2<i32>(1, 1), seed));
  let n00 = dot(g00, f);
  let n10 = dot(g10, f - vec2<f32>(1.0, 0.0));
  let n01 = dot(g01, f - vec2<f32>(0.0, 1.0));
  let n11 = dot(g11, f - vec2<f32>(1.0, 1.0));
  let k0 = n00;
  let k1 = n10 - n00;
  let k2 = n01 - n00;
  let k3 = n00 - n10 - n01 + n11;
  let val = k0 + k1 * u.x + k2 * u.y + k3 * u.x * u.y;
  let g = mix(mix(g00, g10, u.x), mix(g01, g11, u.x), u.y);
  let dv = g + du * vec2<f32>(k1 + k3 * u.y, k2 + k3 * u.x);
  return vec3<f32>(val, dv.x, dv.y) * 1.41421356;
}

// C2 quintic 3D gradient noise in [-1, 1], with optional integer periods (0 = none).
fn grad_noise3_periodic(p: vec3<f32>, period: vec3<i32>, seed: u32) -> f32 {
  let fl = floor(p);
  let i = vec3<i32>(fl);
  let f = p - fl;
  let u = noise_fade(f);
  let x0 = noise_wrap_i(i.x, period.x);
  let x1 = noise_wrap_i(i.x + 1, period.x);
  let y0 = noise_wrap_i(i.y, period.y);
  let y1 = noise_wrap_i(i.y + 1, period.y);
  let z0 = noise_wrap_i(i.z, period.z);
  let z1 = noise_wrap_i(i.z + 1, period.z);
  let n000 = dot(noise_grad3(noise_hash3i(vec3<i32>(x0, y0, z0), seed)), f);
  let n100 = dot(noise_grad3(noise_hash3i(vec3<i32>(x1, y0, z0), seed)), f - vec3<f32>(1.0, 0.0, 0.0));
  let n010 = dot(noise_grad3(noise_hash3i(vec3<i32>(x0, y1, z0), seed)), f - vec3<f32>(0.0, 1.0, 0.0));
  let n110 = dot(noise_grad3(noise_hash3i(vec3<i32>(x1, y1, z0), seed)), f - vec3<f32>(1.0, 1.0, 0.0));
  let n001 = dot(noise_grad3(noise_hash3i(vec3<i32>(x0, y0, z1), seed)), f - vec3<f32>(0.0, 0.0, 1.0));
  let n101 = dot(noise_grad3(noise_hash3i(vec3<i32>(x1, y0, z1), seed)), f - vec3<f32>(1.0, 0.0, 1.0));
  let n011 = dot(noise_grad3(noise_hash3i(vec3<i32>(x0, y1, z1), seed)), f - vec3<f32>(0.0, 1.0, 1.0));
  let n111 = dot(noise_grad3(noise_hash3i(vec3<i32>(x1, y1, z1), seed)), f - vec3<f32>(1.0, 1.0, 1.0));
  return mix(
    mix(mix(n000, n100, u.x), mix(n010, n110, u.x), u.y),
    mix(mix(n001, n101, u.x), mix(n011, n111, u.x), u.y),
    u.z,
  );
}

// C2 quintic 3D gradient noise in [-1, 1].
fn grad_noise3(p: vec3<f32>, seed: u32) -> f32 {
  return grad_noise3_periodic(p, vec3<i32>(0), seed);
}

// C2 3D gradient noise with analytic derivatives: vec4(value, dv/dx, dv/dy, dv/dz).
fn grad_noise3_deriv(p: vec3<f32>, seed: u32) -> vec4<f32> {
  let fl = floor(p);
  let i = vec3<i32>(fl);
  let f = p - fl;
  let u = noise_fade(f);
  let du = noise_dfade(f);
  let g000 = noise_grad3(noise_hash3i(i, seed));
  let g100 = noise_grad3(noise_hash3i(i + vec3<i32>(1, 0, 0), seed));
  let g010 = noise_grad3(noise_hash3i(i + vec3<i32>(0, 1, 0), seed));
  let g110 = noise_grad3(noise_hash3i(i + vec3<i32>(1, 1, 0), seed));
  let g001 = noise_grad3(noise_hash3i(i + vec3<i32>(0, 0, 1), seed));
  let g101 = noise_grad3(noise_hash3i(i + vec3<i32>(1, 0, 1), seed));
  let g011 = noise_grad3(noise_hash3i(i + vec3<i32>(0, 1, 1), seed));
  let g111 = noise_grad3(noise_hash3i(i + vec3<i32>(1, 1, 1), seed));
  let a = dot(g000, f);
  let b = dot(g100, f - vec3<f32>(1.0, 0.0, 0.0));
  let c = dot(g010, f - vec3<f32>(0.0, 1.0, 0.0));
  let d = dot(g110, f - vec3<f32>(1.0, 1.0, 0.0));
  let e = dot(g001, f - vec3<f32>(0.0, 0.0, 1.0));
  let f1 = dot(g101, f - vec3<f32>(1.0, 0.0, 1.0));
  let g = dot(g011, f - vec3<f32>(0.0, 1.0, 1.0));
  let h = dot(g111, f - vec3<f32>(1.0, 1.0, 1.0));
  let k0 = a;
  let k1 = b - a;
  let k2 = c - a;
  let k3 = e - a;
  let k4 = a - b - c + d;
  let k5 = a - c - e + g;
  let k6 = a - b - e + f1;
  let k7 = -a + b + c - d + e - f1 - g + h;
  let val = k0 + k1 * u.x + k2 * u.y + k3 * u.z
    + k4 * u.x * u.y + k5 * u.y * u.z + k6 * u.z * u.x + k7 * u.x * u.y * u.z;
  let ga = mix(mix(mix(g000, g100, u.x), mix(g010, g110, u.x), u.y),
               mix(mix(g001, g101, u.x), mix(g011, g111, u.x), u.y), u.z);
  let dv = ga + du * vec3<f32>(
    k1 + k4 * u.y + k6 * u.z + k7 * u.y * u.z,
    k2 + k5 * u.z + k4 * u.x + k7 * u.z * u.x,
    k3 + k6 * u.x + k5 * u.y + k7 * u.x * u.y,
  );
  return vec4<f32>(val, dv);
}

// Divergence-free 2D curl noise from C2 analytic Perlin derivatives.
fn grad_curl2(p: vec2<f32>, seed: u32) -> vec2<f32> {
  let d = grad_noise2_deriv(p, seed);
  return vec2<f32>(d.z, -d.y);
}

// Divergence-free 3D curl noise from three C2 analytic Perlin potentials.
fn grad_curl3(p: vec3<f32>, seed: u32) -> vec3<f32> {
  let a = grad_noise3_deriv(p, seed);
  let b = grad_noise3_deriv(p, seed + 0x9e3779b9u);
  let c = grad_noise3_deriv(p, seed + 0x3c6ef372u);
  return vec3<f32>(c.z - b.w, a.w - c.y, b.y - a.z);
}

// Tileable 3D Worley distance F1 (Euclidean), matching og_core::noise::worley3_periodic.
fn worley3_periodic(p: vec3<f32>, period: vec3<i32>, seed: u32) -> f32 {
  let fl = floor(p);
  let i = vec3<i32>(fl);
  let f = p - fl;
  var d1 = 1e9;
  for (var oz = -1; oz <= 1; oz += 1) {
    let cz = noise_wrap_i(i.z + oz, period.z);
    for (var oy = -1; oy <= 1; oy += 1) {
      let cy = noise_wrap_i(i.y + oy, period.y);
      for (var ox = -1; ox <= 1; ox += 1) {
        let cx = noise_wrap_i(i.x + ox, period.x);
        let h0 = noise_hash3i(vec3<i32>(cx, cy, cz), seed);
        let h1 = pcg(h0 ^ 0x6c8e9cf5u);
        let h2 = pcg(h1 ^ 0x94d049bbu);
        let pt = vec3<f32>(f32(ox), f32(oy), f32(oz))
          + vec3<f32>(0.075) + 0.85 * vec3<f32>(hash_to_unit(h0), hash_to_unit(h1), hash_to_unit(h2)) - f;
        d1 = min(d1, dot(pt, pt));
      }
    }
  }
  return sqrt(d1);
}
