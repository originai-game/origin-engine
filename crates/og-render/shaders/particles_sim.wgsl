// The particle simulation (compute): one invocation per slot of a
// system's ring, one dispatch per tick. A slot either receives the newest
// particle the tick spawns into it or, while alive, integrates one fixed
// step; then live world-space particles bounce off the depth prepass or,
// where the emitter names them (emitter.collision.w; its list in
// emitter.info.z), the scene's colliders. Nothing is read back. The spawn
// and step formulas are `spawn`, `integrate` and `collide` in
// particles.rs. Prepended by noise.wgsl, particle_common.wgsl and
// scene_colliders.wgsl.

@group(0) @binding(0) var<uniform> emitter: Emitter;
@group(0) @binding(1) var<uniform> view: ParticleView;
@group(0) @binding(2) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(3) var<storage, read> mesh_points: array<vec4<f32>>;
@group(0) @binding(4) var scene_depth: texture_2d<f32>;
@group(0) @binding(5) var scene_gbuffer: texture_2d<f32>;
@group(0) @binding(6) var<storage, read> colliders: array<vec4<f32>>;

const TAU: f32 = 6.28318530718;

fn rand(seed: u32, n: u32, stream: u32) -> f32 {
  return hash_unit(seed ^ pcg(n + stream * 0x9e3779b9u));
}

fn cone_dir(axis: vec3<f32>, angle: f32, r0: f32, r1: f32) -> vec3<f32> {
  let cos_max = cos(min(angle, 3.14159265359));
  let cos_t = 1.0 - r0 * (1.0 - cos_max);
  let sin_t = sqrt(max(1.0 - cos_t * cos_t, 0.0));
  let phi = r1 * TAU;
  var helper = vec3<f32>(0.0, 1.0, 0.0);
  if (abs(axis.y) >= 0.999) { helper = vec3<f32>(1.0, 0.0, 0.0); }
  let t = normalize(cross(helper, axis));
  let b = cross(axis, t);
  let d = t * (cos(phi) * sin_t) + b * (sin(phi) * sin_t) + axis * cos_t;
  let l = length(d);
  if (l < 1e-20) { return axis; }
  return d / l;
}

struct Sample { pos: vec3<f32>, dir: vec3<f32> };

fn shape_sample(r: vec4<f32>) -> Sample {
  var s: Sample;
  s.pos = vec3<f32>(0.0);
  s.dir = vec3<f32>(0.0, 1.0, 0.0);
  let kind = u32(emitter.shape.x);
  if (kind == 1u) {
    let d = cone_dir(vec3<f32>(0.0, 1.0, 0.0), 3.14159265359, r.x, r.y);
    var k = 1.0;
    if (emitter.shape.z < 0.5) { k = pow(r.z, 1.0 / 3.0); }
    s.pos = d * emitter.shape.y * k;
    s.dir = d;
  } else if (kind == 2u) {
    s.dir = cone_dir(vec3<f32>(0.0, 1.0, 0.0), emitter.shape.y, r.x, r.y);
    let a = r.z * TAU;
    let rr = sqrt(r.w) * emitter.shape.z;
    s.pos = vec3<f32>(cos(a) * rr, 0.0, sin(a) * rr);
  } else if (kind == 3u) {
    s.pos = (vec3<f32>(r.x, r.y, r.z) * 2.0 - vec3<f32>(1.0)) * emitter.shape2.xyz;
  } else if (kind == 4u || kind == 5u) {
    let count = u32(emitter.shape.y);
    if (count > 0u) {
      let i = min(u32(r.x * f32(count)), count - 1u);
      s.pos = mesh_points[i * 2u].xyz;
      s.dir = mesh_points[i * 2u + 1u].xyz;
    }
  }
  return s;
}

fn spawn(n: u32, k: u32) -> Particle {
  let seed = emitter.info.y;
  let life = max(mix(emitter.ranges0.x, emitter.ranges0.y, rand(seed, n, 0u)), 1e-3);
  let speed = mix(emitter.ranges0.z, emitter.ranges0.w, rand(seed, n, 1u));
  let size = mix(emitter.ranges1.x, emitter.ranges1.y, rand(seed, n, 2u));
  let rot = mix(emitter.ranges1.z, emitter.ranges1.w, rand(seed, n, 3u));
  let spin = mix(emitter.ranges2.x, emitter.ranges2.y, rand(seed, n, 4u));
  let s = shape_sample(vec4<f32>(rand(seed, n, 6u), rand(seed, n, 7u), rand(seed, n, 8u), rand(seed, n, 11u)));
  var dir = s.dir;
  if (emitter.shape2.w > 0.0) { dir = cone_dir(dir, emitter.shape2.w, rand(seed, n, 9u), rand(seed, n, 10u)); }
  var pos = s.pos;
  var vel = dir * speed;
  if (emitter.ranges2.z < 0.5) {
    pos = (emitter.transform * vec4<f32>(pos, 1.0)).xyz;
    vel = (emitter.transform * vec4<f32>(vel, 0.0)).xyz + emitter.motion.xyz * emitter.motion.w;
  }
  let dt = emitter.ranges2.w;
  let bursts = emitter.spawn.w;
  var age = 0.0;
  if (k >= bursts) {
    let c = f32(max(emitter.spawn.y - bursts, 1u));
    age = (f32(k - bursts) + 0.5) / c * dt;
  }
  pos += vel * age;
  // What the emission leaves out takes its slot unborn (cpu.rs `spawn`).
  if (rand(seed, n, 15u) >= emitter.counts.z) { age = life; }
  var p: Particle;
  p.pos_age = vec4<f32>(pos, age);
  p.vel_life = vec4<f32>(vel, life);
  p.misc = vec4<f32>(size, rot, spin, rand(seed, n, 5u));
  return p;
}

// One of the curl's derivatives: d psi_k / d axis, central differences.
fn curl_d(q: vec3<f32>, whole: vec3<i32>, seed: u32, k: u32, axis: vec3<f32>) -> f32 {
  let h = 0.05;
  let s = seed ^ (0x4444u + k * 0x1111u);
  return (value_noise3_at(q + axis * h, whole, s) - value_noise3_at(q - axis * h, whole, s)) / (2.0 * h);
}

// Curl noise (curl_noise in particles.rs).
fn curl_noise(q: vec3<f32>, whole: vec3<i32>, seed: u32) -> vec3<f32> {
  let x = vec3<f32>(1.0, 0.0, 0.0);
  let y = vec3<f32>(0.0, 1.0, 0.0);
  let z = vec3<f32>(0.0, 0.0, 1.0);
  return vec3<f32>(
    curl_d(q, whole, seed, 2u, y) - curl_d(q, whole, seed, 1u, z),
    curl_d(q, whole, seed, 0u, z) - curl_d(q, whole, seed, 2u, x),
    curl_d(q, whole, seed, 1u, x) - curl_d(q, whole, seed, 0u, y)) * 0.25;
}

fn grid_at(n: i32, base: u32, c: vec3<i32>) -> vec3<f32> {
  let w = ((c % n) + vec3<i32>(n)) % n;
  return mesh_points[base + u32((w.z * n + w.y) * n + w.x)].xyz;
}

// A grid field's vector at `g` (cells, tiling), trilinear (grid_sample).
fn grid_sample(g: vec3<f32>) -> vec3<f32> {
  let n = max(i32(emitter.field.w), 1);
  let base = u32(emitter.counts2.w);
  let fl = floor(g);
  let f = g - fl;
  let i = vec3<i32>(fl);
  let x00 = mix(grid_at(n, base, i), grid_at(n, base, i + vec3<i32>(1, 0, 0)), f.x);
  let x10 = mix(grid_at(n, base, i + vec3<i32>(0, 1, 0)), grid_at(n, base, i + vec3<i32>(1, 1, 0)), f.x);
  let x01 = mix(grid_at(n, base, i + vec3<i32>(0, 0, 1)), grid_at(n, base, i + vec3<i32>(1, 0, 1)), f.x);
  let x11 = mix(grid_at(n, base, i + vec3<i32>(0, 1, 1)), grid_at(n, base, i + vec3<i32>(1, 1, 1)), f.x);
  return mix(mix(x00, x10, f.y), mix(x01, x11, f.y), f.z);
}

// The attractors' pull and the field at `pos` (forces in particles.rs).
fn forces(pos: vec3<f32>) -> vec3<f32> {
  var a = vec3<f32>(0.0);
  for (var i = 0u; i < u32(emitter.counts2.z); i++) {
    let at = emitter.attract[i];
    let d = at.xyz - pos;
    let dist = length(d);
    if (dist < 1e-4) { continue; }
    let r = emitter.attract_r[i];
    var f = at.w / (1.0 + dist * dist);
    if (r > 0.0) { f = at.w * max(1.0 - dist / r, 0.0); }
    a += d * (f / dist);
  }
  let kind = u32(emitter.field.x);
  if (kind == 1u) {
    let q = pos * emitter.field.z + emitter.field_origin[1].xyz + vec3<f32>(emitter.noise.w * emitter.field.w);
    a += curl_noise(q, vec3<i32>(emitter.field_origin[0].xyz), emitter.info.y) * emitter.field.y;
  } else if (kind == 2u) {
    let g = pos / emitter.field.z * emitter.field.w + emitter.field_origin[1].xyz;
    a += grid_sample(g) * emitter.field.y;
  }
  return a;
}

fn integrate(p: Particle) -> Particle {
  let dt = emitter.ranges2.w;
  var pos = p.pos_age.xyz;
  var vel = p.vel_life.xyz;
  vel += emitter.forces.xyz * dt;
  if (emitter.noise.x != 0.0) {
    let seed = emitter.info.y;
    // `pos` is from the render origin: the lattice's cells there are added.
    let w = vec3<i32>(emitter.noise_origin[0].xyz);
    let q = pos * emitter.noise.y + emitter.noise_origin[1].xyz + vec3<f32>(emitter.noise.w * emitter.noise.z);
    let n = vec3<f32>(value_noise3_at(q, w, seed ^ 0x1111u), value_noise3_at(q, w, seed ^ 0x2222u), value_noise3_at(q, w, seed ^ 0x3333u)) * 2.0 - vec3<f32>(1.0);
    vel += n * emitter.noise.x * dt;
  }
  vel += forces(pos) * dt;
  vel *= 1.0 / (1.0 + max(emitter.forces.w, 0.0) * dt);
  let before = pos;
  var speed_scale = 1.0;
  let speed_keys = u32(emitter.counts2.x);
  if (speed_keys > 0u) {
    let t = clamp(p.pos_age.w / max(p.vel_life.w, 1e-6), 0.0, 1.0);
    speed_scale = curve_at(emitter.speed_t, emitter.speed_v, speed_keys, t);
  }
  pos += vel * (dt * speed_scale);
  if (emitter.collision.w > 0.5) {
    // The scene's colliders, as the CPU path's `collide` meets them: a
    // step from outside to inside reflects the normal part of the
    // velocity by `bounce`, takes `friction` off the rest and is taken
    // again from before; one that starts inside (born in a brazier's box)
    // leaves freely.
    let list = emitter.info.z;
    if (!push_out(before, 0.0, list).hit) {
      let h = push_out(pos, 0.0, list);
      if (h.hit) {
        let vn = dot(vel, h.n);
        if (vn < 0.0) {
          let normal_part = h.n * vn;
          vel = (vel - normal_part) * (1.0 - emitter.collision.z) - normal_part * emitter.collision.y;
          pos = before + vel * dt;
          // A corner or a second shape the new step still enters: out to its surface.
          let again = push_out(pos, 0.0, list);
          if (again.hit) { pos = again.p; }
        }
      }
    }
  } else if (emitter.collision.x > 0.5 && emitter.ranges2.z < 0.5) {
    // The prepass depth along the camera's forward axis at the particle's pixel.
    let clip = view.view_proj * vec4<f32>(pos, 1.0);
    if (clip.w > 0.0) {
      let ndc = clip.xy / clip.w;
      let uv = vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
      if (all(uv >= vec2<f32>(0.0)) && all(uv < vec2<f32>(1.0))) {
        let size = vec2<i32>(textureDimensions(scene_depth));
        let px = clamp(vec2<i32>(uv * vec2<f32>(size)), vec2<i32>(0), size - 1);
        let surface = textureLoad(scene_depth, px, 0).r;
        let depth = dot(pos - view.cam_pos.xyz, view.cam_forward.xyz);
        let thickness = max(0.25, length(vel) * dt * 2.0);
        if (depth > surface && depth < surface + thickness) {
          let g = textureLoad(scene_gbuffer, px, 0).xy;
          var nrm = vec3<f32>(g, 1.0 - abs(g.x) - abs(g.y));
          if (nrm.z < 0.0) {
            nrm = vec3<f32>((1.0 - abs(nrm.yx)) * select(vec2<f32>(-1.0), vec2<f32>(1.0), nrm.xy >= vec2<f32>(0.0)), nrm.z);
          }
          nrm = normalize(nrm);
          let vn = dot(vel, nrm);
          if (vn < 0.0) {
            let normal_part = nrm * vn;
            vel = (vel - normal_part) * (1.0 - emitter.collision.z) - normal_part * emitter.collision.y;
            pos = before + vel * dt;
          }
        }
      }
    }
  }
  var o = p;
  o.pos_age = vec4<f32>(pos, p.pos_age.w + dt);
  o.vel_life = vec4<f32>(vel, p.vel_life.w);
  return o;
}

@compute @workgroup_size(64)
fn simulate(@builtin(global_invocation_id) id: vec3<u32>) {
  let slot = id.x;
  let max_particles = emitter.info.x;
  if (slot >= max_particles) { return; }
  let count = emitter.spawn.y;
  let off = (slot + max_particles - emitter.spawn.z) % max_particles;
  if (off < count) {
    let k = off + ((count - 1u - off) / max_particles) * max_particles;
    particles[slot] = spawn(emitter.spawn.x + k, k);
    return;
  }
  let p = particles[slot];
  if (p.pos_age.w < p.vel_life.w) {
    particles[slot] = integrate(p);
  }
}

// Empties every slot (a system that restarts).
@compute @workgroup_size(64)
fn clear(@builtin(global_invocation_id) id: vec3<u32>) {
  if (id.x >= emitter.info.x) { return; }
  var p: Particle;
  p.pos_age = vec4<f32>(0.0);
  p.vel_life = vec4<f32>(0.0);
  p.misc = vec4<f32>(0.0);
  particles[id.x] = p;
}

// Moves every slot by the entry's translation (a move of the render
// origin, origin.rs): world-space particles keep their place in the world.
@compute @workgroup_size(64)
fn shift(@builtin(global_invocation_id) id: vec3<u32>) {
  if (id.x >= emitter.info.x) { return; }
  var p = particles[id.x];
  p.pos_age = vec4<f32>(p.pos_age.xyz + emitter.transform[3].xyz, p.pos_age.w);
  particles[id.x] = p;
}
