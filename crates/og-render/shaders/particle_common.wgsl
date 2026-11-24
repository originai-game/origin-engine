// Shared particle types and curve utilities (`ParticleViewGpu`, `ParticleState`).
// Prepended after noise.wgsl by particle sim, draw, sprites and compaction shaders.

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
