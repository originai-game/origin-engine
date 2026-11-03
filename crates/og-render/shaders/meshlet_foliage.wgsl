// Anchor-coherent branch/leaf motion adapted from SeedThree's wind.js,
// Copyright (c) 2026 SkyeShark, MIT (third_party/vegetation/SeedThree.LICENSE).
// GPU Gems 3 ch. 16 and Pivot Painter: technique references, no source copied.
// Hierarchy: leaf about its attachment, branch about its pivot, then trunk
// about its root. The cook keeps anchors categorical through every LOD.
// No bindings: callers supply the same current/previous state in every pass.

struct FoliageWind {
  dir_strength: vec4f, // xyz world wind direction, w lean
  gust_top: vec4f,     // gust, model height, two reduced wind_phases
  origin: vec4f,       // xyz current render origin, w wavenumber
};

// SeedThree's canopy dome, blended before the inverse-transpose and normal
// mapping. The centre comes from all leaf anchors, not the trunk's bounds.
fn foliage_normal(n: vec3f, anchor: vec3f, center: vec3f, strength: f32) -> vec3f {
  if (strength <= 0.0) { return n; }
  let radial = anchor - center;
  var dome = vec3f(0.0, 1.0, 0.0);
  if (dot(radial, radial) > 1e-12) {
    dome = normalize(normalize(radial) + vec3f(0.0, 0.45, 0.0));
  }
  let blended = mix(n, dome, clamp(strength, 0.0, 1.0));
  if (dot(blended, blended) <= 1e-12) { return dome; }
  return normalize(blended);
}

fn foliage_rotate(p: vec3f, axis: vec3f, angle: f32) -> vec3f {
  let c = cos(angle);
  return p * c + cross(axis, p) * sin(angle) + axis * dot(axis, p) * (1.0 - c);
}

// Identical travelling wave to scatter_sway; the grass shared-wind include
// replaces this sampler when R6's gust field lands. Detail phases stay here.
fn foliage_wave(at: vec3f, wind: FoliageWind) -> f32 {
  let x = wind.origin.w * dot(at + wind.origin.xyz, wind.dir_strength.xyz);
  return (sin(x - wind.gust_top.z) + 0.5 * sin(2.3 * x + 1.7 - wind.gust_top.w)) / 1.5;
}

// World-space offset, added to model * p (not transformed a second time).
// f0 = branch pivot/stiffness; f1 = leaf anchor/flutter. Bark has flutter 0.
fn foliage_offset(p: vec3f, f0: vec4f, f1: vec4f, model: mat4x4f, wind: FoliageWind) -> vec3f {
  if (wind.dir_strength.w <= 0.0 || wind.gust_top.y <= 0.0) { return vec3f(0.0); }
  // Bring the wind into model space before bending: rotated and scaled
  // instances still lean with the world wind, not with their own +x.
  let a = model[0].xyz;
  let b = model[1].xyz;
  let c = model[2].xyz;
  let d = wind.dir_strength.xyz;
  let inverse_d = vec3f(dot(cross(b, c), d), dot(cross(c, a), d), dot(cross(a, b), d)) / dot(a, cross(b, c));
  let horizontal = vec3f(inverse_d.x, 0.0, inverse_d.z);
  if (dot(horizontal, horizontal) <= 1e-12) { return vec3f(0.0); }
  let direction = normalize(horizontal);
  let axis = cross(vec3f(0.0, 1.0, 0.0), direction);
  let root = model[3].xyz;
  let lean = wind.dir_strength.w * (1.0 + wind.gust_top.x * foliage_wave(root, wind));
  let branch_world = (model * vec4f(f0.xyz, 1.0)).xyz;
  let branch_phase = wind.origin.w * dot(branch_world + wind.origin.xyz, wind.dir_strength.xyz);
  let branch_wave = (sin(branch_phase - wind.gust_top.z) + 0.5 * sin(2.3 * branch_phase + 1.7 - wind.gust_top.w)) / 1.5;
  let leaf_phase = dot(f1.xyz, vec3f(1.7, 2.3, 1.1)) + branch_phase;
  // Integer harmonics of the reduced phases are continuous at phase wrap.
  let flutter = (sin(3.0 * wind.gust_top.z + leaf_phase) + 0.5 * sin(2.0 * wind.gust_top.w + leaf_phase)) / 1.5;
  let leaf_axis = normalize(direction + vec3f(0.0, 0.6, 0.0));
  var moved = f1.xyz + foliage_rotate(p - f1.xyz, leaf_axis, atan(0.35 * lean * f1.w) * flutter);
  moved = f0.xyz + foliage_rotate(moved - f0.xyz, axis, atan(lean * (1.0 - f0.w)) * branch_wave);
  let height = clamp(moved.y / wind.gust_top.y, 0.0, 1.0);
  moved = foliage_rotate(moved, axis, atan(lean * height * height));
  return (model * vec4f(moved - p, 0.0)).xyz;
}

// A stable dither seed. Pass the absolute root, not a camera-relative or
// sorted slot index; rebasing and culling must not reshuffle the dither.
fn foliage_seed(world_root: vec3f) -> u32 {
  var h = 2166136261u;
  for (var i = 0u; i < 3u; i++) {
    h = (h ^ bitcast<u32>(world_root[i])) * 16777619u;
  }
  return h ^ (h >> 16u);
}
