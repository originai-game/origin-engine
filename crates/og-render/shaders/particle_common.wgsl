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
  noise_origin: array<vec4<f32>, 2>,  // the turbulence's lattice at the render origin: xyz whole cells, then the rest (origin.rs)
  look: vec4<f32>,         // x: 1 lit, y: distortion (fraction of the screen's height), z: 1 a mesh per particle, w: a lit mesh's roughness (1 matte)
  speed_t: array<vec4<f32>, 2>,  // speed over life (against the velocity)
  speed_v: array<vec4<f32>, 2>,
  spin_t: array<vec4<f32>, 2>,   // angular velocity over life
  spin_v: array<vec4<f32>, 2>,
  attract: array<vec4<f32>, 4>,  // xyz position (simulation space), w strength
  attract_r: vec4<f32>,          // each attractor's radius (0: 1 / (1 + d²))
  field: vec4<f32>,        // x: 0 none, 1 curl, 2 grid; y strength; z frequency (curl) or size (grid); w scroll (curl) or cells (grid)
  field_origin: array<vec4<f32>, 2>,  // the field's lattice at the render origin: xyz whole cells, then the rest
  motion: vec4<f32>,       // xyz: the emitter's velocity this tick, w: the fraction a particle inherits
  counts2: vec4<f32>,      // x: speed keys (0 none), y: spin keys (0 none), z: attractors, w: where a grid's vectors start in mesh_points
};

// A curve over life (keys `ts`, `vs`, `n` of them) at `t`.
fn curve_at(ts: array<vec4<f32>, 2>, vs: array<vec4<f32>, 2>, n: u32, t: f32) -> f32 {
  var v = key4(vs, 0u);
  for (var i = 1u; i < n; i++) {
    let t0 = key4(ts, i - 1u);
    let t1 = key4(ts, i);
    if (t >= t1) {
      v = key4(vs, i);
    } else if (t > t0) {
      v = mix(key4(vs, i - 1u), key4(vs, i), (t - t0) / max(t1 - t0, 1e-6));
      break;
    } else {
      break;
    }
  }
  return v;
}

// The area under a curve from 0 to `t` (Curve::integral in particles.rs).
fn curve_integral(ts: array<vec4<f32>, 2>, vs: array<vec4<f32>, 2>, n: u32, t: f32) -> f32 {
  if (n == 0u) { return t; }
  var area = 0.0;
  var pt = 0.0;
  var pv = key4(vs, 0u);
  for (var i = 0u; i < n; i++) {
    let kt = key4(ts, i);
    let kv = key4(vs, i);
    if (kt <= pt) {
      pv = kv;
      continue;
    }
    if (t <= kt) {
      let f = max((t - pt) / (kt - pt), 0.0);
      let v = pv + (kv - pv) * f;
      return area + max(t - pt, 0.0) * (pv + v) * 0.5;
    }
    area += (kt - pt) * (pv + kv) * 0.5;
    pt = kt;
    pv = kv;
  }
  return area + max(t - pt, 0.0) * pv;
}

// A particle's rotation at age `age` of `life` (t = age / life):
// particle_rotation in particles.rs.
fn particle_rotation(e: Emitter, misc: vec4<f32>, age: f32, life: f32, t: f32) -> f32 {
  let n = u32(e.counts2.y);
  if (n == 0u) { return misc.y + misc.z * age; }
  return misc.y + misc.z * life * curve_integral(e.spin_t, e.spin_v, n, t);
}

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
