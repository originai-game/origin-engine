// The emitter and view uniforms shared by the particle simulation and the
// particle draw (`EmitterGpu` and `ParticleViewGpu` in particles_gpu.rs).
// Prepended after noise.wgsl.

struct Emitter {
  transform: mat4x4<f32>,  // the tick's (sim) or frame's (draw) local -> world
  shape: vec4<f32>,        // x: 0 point, 1 sphere, 2 cone, 3 box, 4 mesh; sphere: y radius, z surface; cone: y angle (rad), z radius; mesh: y points
  shape2: vec4<f32>,       // xyz: box half extents, w: spread (rad)
  ranges0: vec4<f32>,      // lifetime min, max, speed min, max
  ranges1: vec4<f32>,      // size min, max, rotation min, max (rad)
  ranges2: vec4<f32>,      // angular velocity min, max (rad/s), space (0 world, 1 local), dt
  forces: vec4<f32>,       // gravity xyz, drag
  noise: vec4<f32>,        // strength, frequency, scroll, time (s)
  collision: vec4<f32>,    // x: on, y: bounce, z: friction
  color: vec4<f32>,        // linear tint
  size_t: array<vec4<f32>, 2>,
  size_v: array<vec4<f32>, 2>,
  color_t: array<vec4<f32>, 2>,
  color_v: array<vec4<f32>, 8>,
  sheet: vec4<f32>,        // columns, rows, frames, mode (0 off, 1 over life, 2 fps, 3 random)
  sheet2: vec4<f32>,       // x: cycles or fps, y: 1 with a texture
  render: vec4<f32>,       // x: align (0 camera, 1 velocity), y: stretch, z: soft distance, w: blend (0 additive, 1 alpha)
  counts: vec4<f32>,       // x: size keys, y: colour keys
  spawn: vec4<u32>,        // first index (low 32 bits), count, first % max, bursts
  info: vec4<u32>,         // max particles, seed
};

struct ParticleView {
  view_proj: mat4x4<f32>,  // unjittered (collision)
  cam_pos: vec4<f32>,      // w: 1 orthographic
  cam_forward: vec4<f32>,
  cam_right: vec4<f32>,
  cam_up: vec4<f32>,
  screen: vec4<f32>,       // w, h, 1/w, 1/h
};

struct Particle {
  pos_age: vec4<f32>,
  vel_life: vec4<f32>,
  misc: vec4<f32>,         // start size, start rotation, angular velocity, random
};

fn key4(a: array<vec4<f32>, 2>, i: u32) -> f32 {
  let v = a[i / 4u];
  let j = i % 4u;
  if (j == 0u) { return v.x; }
  if (j == 1u) { return v.y; }
  if (j == 2u) { return v.z; }
  return v.w;
}

fn size_curve(e: Emitter, t: f32) -> f32 {
  let n = u32(e.counts.x);
  var v = key4(e.size_v, 0u);
  for (var i = 1u; i < n; i++) {
    let t0 = key4(e.size_t, i - 1u);
    let t1 = key4(e.size_t, i);
    if (t >= t1) {
      v = key4(e.size_v, i);
    } else if (t > t0) {
      v = mix(key4(e.size_v, i - 1u), key4(e.size_v, i), (t - t0) / max(t1 - t0, 1e-6));
      break;
    } else {
      break;
    }
  }
  return v;
}

fn color_curve(e: Emitter, t: f32) -> vec4<f32> {
  let n = u32(e.counts.y);
  var v = e.color_v[0];
  for (var i = 1u; i < n; i++) {
    let t0 = key4(e.color_t, i - 1u);
    let t1 = key4(e.color_t, i);
    if (t >= t1) {
      v = e.color_v[i];
    } else if (t > t0) {
      v = mix(e.color_v[i - 1u], e.color_v[i], (t - t0) / max(t1 - t0, 1e-6));
      break;
    } else {
      break;
    }
  }
  return v;
}
