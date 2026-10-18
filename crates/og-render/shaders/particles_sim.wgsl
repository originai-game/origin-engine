// The particle simulation (full tier): one invocation per slot of a
// system's ring, one dispatch per tick. A slot either receives the newest
// particle the tick spawns into it or, while alive, integrates one fixed
// step; then live world-space particles bounce off the depth prepass. The
// spawn and step formulas are `spawn` and `integrate` in particles.rs.
// Prepended by noise.wgsl and particle_common.wgsl.

@group(0) @binding(0) var<uniform> emitter: Emitter;
@group(0) @binding(1) var<uniform> view: ParticleView;
@group(0) @binding(2) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(3) var<storage, read> mesh_points: array<vec4<f32>>;
@group(0) @binding(4) var scene_depth: texture_2d<f32>;
@group(0) @binding(5) var scene_gbuffer: texture_2d<f32>;

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
  } else if (kind == 4u) {
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
    vel = (emitter.transform * vec4<f32>(vel, 0.0)).xyz;
  }
  let dt = emitter.ranges2.w;
  let bursts = emitter.spawn.w;
  var age = 0.0;
  if (k >= bursts) {
    let c = f32(max(emitter.spawn.y - bursts, 1u));
    age = (f32(k - bursts) + 0.5) / c * dt;
  }
  pos += vel * age;
  var p: Particle;
  p.pos_age = vec4<f32>(pos, age);
  p.vel_life = vec4<f32>(vel, life);
  p.misc = vec4<f32>(size, rot, spin, rand(seed, n, 5u));
  return p;
}

fn integrate(p: Particle) -> Particle {
  let dt = emitter.ranges2.w;
  var pos = p.pos_age.xyz;
  var vel = p.vel_life.xyz;
  vel += emitter.forces.xyz * dt;
  if (emitter.noise.x != 0.0) {
    let seed = emitter.info.y;
    let q = pos * emitter.noise.y + vec3<f32>(emitter.noise.w * emitter.noise.z);
    let n = vec3<f32>(value_noise3(q, seed ^ 0x1111u), value_noise3(q, seed ^ 0x2222u), value_noise3(q, seed ^ 0x3333u)) * 2.0 - vec3<f32>(1.0);
    vel += n * emitter.noise.x * dt;
  }
  vel *= 1.0 / (1.0 + max(emitter.forces.w, 0.0) * dt);
  let before = pos;
  pos += vel * dt;
  if (emitter.collision.x > 0.5 && emitter.ranges2.z < 0.5) {
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
