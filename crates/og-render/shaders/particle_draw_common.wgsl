// Draw-only emitter uniform (`EmitterDrawGpu` in particles_gpu/emitter.rs)
// and appearance helpers shared by particles.wgsl and particle_sprites.wgsl.
// Prepended after particle_common.wgsl.

struct EmitterDraw {
  transform: mat4x4<f32>,        // frame local -> world
  color: vec4<f32>,              // linear tint (system tint applied)
  size_t: array<vec4<f32>, 2>,
  size_v: array<vec4<f32>, 2>,
  height_t: array<vec4<f32>, 2>,
  height_v: array<vec4<f32>, 2>,
  spin_t: array<vec4<f32>, 2>,
  spin_v: array<vec4<f32>, 2>,
  color_t: array<vec4<f32>, 2>,
  color_v: array<vec4<f32>, 8>,
  color2_t: array<vec4<f32>, 2>,
  color2_v: array<vec4<f32>, 8>,
  sheet: vec4<f32>,              // columns, rows, frames, mode (0 off, 1 over life, 2 fps, 3 random)
  sheet2: vec4<f32>,             // x: cycles or fps, y: textured, z: blend frames, w: local space (1 local, 0 world)
  render: vec4<f32>,             // x: align, y: stretch, z: soft distance, w: blend (0 add, 1 alpha, 2 sub, 3 mul)
  counts: vec4<f32>,             // x: size keys, y: colour keys, z: height keys, w: second colour keys
  counts2: vec4<f32>,            // x: spin keys, y: hue turn (rad), z: trail newest sample age (s), w: unused
  look: vec4<f32>,               // x: lit, y: distortion, z: mesh mode (0 quad, 1 mesh, 2 effect mesh), w: roughness
  axis: vec4<f32>,               // xyz: upright/facing axis in emitter space; w: 1 effect mesh billboard
  trail: vec4<f32>,              // x: lifetime, y: width share, z: sample interval, w: newest sample index
  info: vec4<u32>,               // x: max particles, y: seed, z: unused, w: unused
};

fn size_curve(t: f32) -> f32 {
  let n = u32(emitter.counts.x);
  return curve_at(emitter.size_t, emitter.size_v, n, t);
}

// A particle's width and height at life `t` from its start size `s`
// (appearance in particles/cpu.rs).
fn particle_size(s: f32, t: f32) -> vec2<f32> {
  let w = s * size_curve(t);
  let n = u32(emitter.counts.z);
  if (n == 0u) { return vec2<f32>(w, w); }
  return vec2<f32>(w, s * curve_at(emitter.height_t, emitter.height_v, n, t));
}

// A particle's colour at life `t` with random value `r`: the colour curve,
// or a share of the way to the second one.
fn color_curve(t: f32, r: f32) -> vec4<f32> {
  let v = color_at(emitter.color_t, emitter.color_v, u32(emitter.counts.y), t);
  let n2 = u32(emitter.counts.w);
  if (n2 == 0u) { return v; }
  return mix(v, color_at(emitter.color2_t, emitter.color2_v, n2, t), color_share(r));
}

// A particle's rotation at age `age` of `life` (t = age / life):
// particle_rotation in particles.rs.
fn particle_rotation(misc: vec4<f32>, age: f32, life: f32, t: f32) -> f32 {
  let n = u32(emitter.counts2.x);
  if (n == 0u) { return misc.y + misc.z * age; }
  return misc.y + misc.z * life * curve_integral(emitter.spin_t, emitter.spin_v, n, t);
}
