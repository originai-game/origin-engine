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
  render: vec4<f32>,       // x: align (0 camera, 1 velocity, 2 upright, 3 facing), y: stretch, z: soft distance, w: blend (0 additive, 1 alpha, 2 subtractive, 3 multiply)
  counts: vec4<f32>,       // x: size keys, y: colour keys, z: the share of the spawns that are born, w: the hue turn (rad)
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
  air: vec4<f32>,          // xyz: what drag closes on, in the simulation space (the emitter's `wind` share of the air)
  height_t: array<vec4<f32>, 2>,  // the height over life (counts3.x keys; 0: the size curve's)
  height_v: array<vec4<f32>, 2>,
  counts3: vec4<f32>,      // x: height keys, y: second colour keys (0 none), z: a trail's newest sample's age (s), w: 1 a particle dies where it hits
  axis: vec4<f32>,         // xyz: an upright or facing particle's axis, in the emitter's space; w: 1 an effect mesh facing the camera
  color2_t: array<vec4<f32>, 2>,  // the second colour over life (counts3.y keys)
  color2_v: array<vec4<f32>, 8>,
  trail: vec4<f32>,        // x: a trail's lifetime (0 none), y: its width (a share of the particle's), z: seconds between samples, w: a tick's sample index (-1 none) or, drawing, the newest's
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
fn particle_rotation(misc: vec4<f32>, age: f32, life: f32, t: f32) -> f32 {
  let n = u32(emitter.counts2.y);
  if (n == 0u) { return misc.y + misc.z * age; }
  return misc.y + misc.z * life * curve_integral(emitter.spin_t, emitter.spin_v, n, t);
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

fn size_curve(t: f32) -> f32 {
  let n = u32(emitter.counts.x);
  var v = key4(emitter.size_v, 0u);
  for (var i = 1u; i < n; i++) {
    let t0 = key4(emitter.size_t, i - 1u);
    let t1 = key4(emitter.size_t, i);
    if (t >= t1) {
      v = key4(emitter.size_v, i);
    } else if (t > t0) {
      v = mix(key4(emitter.size_v, i - 1u), key4(emitter.size_v, i), (t - t0) / max(t1 - t0, 1e-6));
      break;
    } else {
      break;
    }
  }
  return v;
}

// A particle's width and height at life `t` from its start size `s`
// (appearance in particles/cpu.rs): the size curve, and the height curve
// for the height where the emitter has one.
fn particle_size(s: f32, t: f32) -> vec2<f32> {
  let w = s * size_curve(t);
  let n = u32(emitter.counts3.x);
  if (n == 0u) { return vec2<f32>(w, w); }
  return vec2<f32>(w, s * curve_at(emitter.height_t, emitter.height_v, n, t));
}

// A colour curve over life (keys `ts`, `vs`, `n` of them) at `t`.
fn color_at(ts: array<vec4<f32>, 2>, vs: array<vec4<f32>, 8>, n: u32, t: f32) -> vec4<f32> {
  var v = vs[0];
  for (var i = 1u; i < n; i++) {
    let t0 = key4(ts, i - 1u);
    let t1 = key4(ts, i);
    if (t >= t1) {
      v = vs[i];
    } else if (t > t0) {
      v = mix(vs[i - 1u], vs[i], (t - t0) / max(t1 - t0, 1e-6));
      break;
    } else {
      break;
    }
  }
  return v;
}

// Where a particle's colour sits between the two curves, from its random
// value (`color_share` in og-render-data's cpu.rs).
fn color_share(r: f32) -> f32 {
  return fract(r * 61.0);
}

// A particle's colour at life `t` with random value `r`: the colour curve,
// or a share of the way to the second one.
fn color_curve(t: f32, r: f32) -> vec4<f32> {
  let v = color_at(emitter.color_t, emitter.color_v, u32(emitter.counts.y), t);
  let n2 = u32(emitter.counts3.y);
  if (n2 == 0u) { return v; }
  return mix(v, color_at(emitter.color2_t, emitter.color2_v, n2, t), color_share(r));
}
