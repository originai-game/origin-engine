// Batched pooled particle simulation (compute): all active GPU systems share
// one Mega-Pool (`particles_pool`, `trails_pool`, `points_pool`) and simulate
// in merged wave dispatches (`sim_tiles`) — one dispatch per wave across every
// system in the scene. Sub-emitters (`Birth`, `Death`, `Collision`, `Life`)
// spawn deterministically directly into their child slabs on the GPU.
// Prepended by noise.wgsl, particle_common.wgsl and scene_colliders.wgsl.

struct SimDispatch {
  tile_offset: u32,
  tile_count: u32,
  _pad0: u32,
  _pad1: u32,
};

struct EmitterSim {
  transform: mat4x4<f32>,        // end-of-tick local -> world (or shift translation)
  prev_transform: mat4x4<f32>,   // start-of-tick local -> world (for sub-tick spawn lerp)
  shape: vec4<f32>,              // x: kind (0..5), y: a, z: b, w: 1 when prev_transform active
  shape2: vec4<f32>,             // xyz: box half extents, w: spread (rad)
  ranges0: vec4<f32>,            // lifetime min, max, speed min, max
  ranges1: vec4<f32>,            // size min, max, rotation min, max (rad)
  ranges2: vec4<f32>,            // angular velocity min, max (rad/s), space (0 world, 1 local), dt
  forces: vec4<f32>,             // gravity xyz, drag
  noise: vec4<f32>,              // strength, frequency, scroll, time (s)
  collision: vec4<f32>,          // x: depth on, y: bounce, z: friction, w: colliders on
  noise_origin: array<vec4<f32>, 2>,
  speed_t: array<vec4<f32>, 2>,
  speed_v: array<vec4<f32>, 2>,
  attract: array<vec4<f32>, 4>,
  attract_r: vec4<f32>,
  field: vec4<f32>,              // x: 0 none, 1 curl, 2 grid; y strength; z freq/size; w scroll/cells
  field_origin: array<vec4<f32>, 2>,
  motion: vec4<f32>,             // xyz: emitter velocity, w: inherit_velocity
  air: vec4<f32>,                // xyz: carrying air, w: force_fields share
  counts: vec4<f32>,             // x: speed keys, y: attractors, z: emission share, w: die on hit
  trail: vec4<f32>,              // x: lifetime, y: width share, z: interval, w: sample index (-1 none)
  spawn: vec4<u32>,              // x: first (low 32), y: count, z: first % max, w: bursts
  info: vec4<u32>,               // x: max particles, y: seed, z: collider list, w: points_base (vec4 offset)
  pool: vec4<u32>,               // x: particle_base, y: trail_base, z: grid_base (vec4 offset), w: (sub_count << 20) | sub_base_vec4 (or clear counter idx)
};

// Group 0: frame bindings (2 storage buffers: colliders, force_fields).
@group(0) @binding(0) var<uniform> dispatch: SimDispatch;
@group(0) @binding(1) var<uniform> view: ParticleView;
@group(0) @binding(2) var scene_depth: texture_2d<f32>;
@group(0) @binding(3) var scene_gbuffer: texture_2d<f32>;
@group(0) @binding(4) var<storage, read> colliders: array<vec4<f32>>;
@group(0) @binding(5) var<storage, read> force_fields: array<vec4<f32>>;

// Group 1: pooled simulation buffers (6 storage buffers; total = 8, WebGPU Compat limit).
@group(1) @binding(0) var<storage, read> sim_entries: array<EmitterSim>;
// Holds SimTile entries (entry_idx, slot_offset, 0, 0) followed by packed 4-vec4 SubEmitDesc records.
@group(1) @binding(1) var<storage, read> sim_tiles: array<vec4<u32>>;
@group(1) @binding(2) var<storage, read> points_pool: array<vec4<f32>>;
@group(1) @binding(3) var<storage, read_write> particles_pool: array<Particle>;
@group(1) @binding(4) var<storage, read_write> trails_pool: array<vec4<f32>>;
@group(1) @binding(5) var<storage, read_write> sub_counters: array<atomic<u32>>;

const TRAIL_POINTS: u32 = 8u;
const IMPACT_SPEED: f32 = 0.5;

fn rand(seed: u32, n: u32, stream: u32) -> f32 {
  return hash_unit(seed ^ pcg(n + stream * 0x9e3779b9u));
}

fn cone_dir(axis: vec3<f32>, angle: f32, r0: f32, r1: f32) -> vec3<f32> {
  let cos_max = cos(min(angle, PI));
  let cos_t = 1.0 - r0 * (1.0 - cos_max);
  let sin_t = sqrt(max(1.0 - cos_t * cos_t, 0.0));
  let phi = r1 * 2.0 * PI;
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

fn shape_sample(ei: u32, r: vec4<f32>) -> Sample {
  var s: Sample;
  s.pos = vec3<f32>(0.0);
  s.dir = vec3<f32>(0.0, 1.0, 0.0);
  let sh = sim_entries[ei].shape;
  let kind = u32(sh.x);
  if (kind == 1u) {
    let d = cone_dir(vec3<f32>(0.0, 1.0, 0.0), PI, r.x, r.y);
    var k = 1.0;
    if (sh.z < 0.5) { k = pow(r.z, 1.0 / 3.0); }
    s.pos = d * sh.y * k;
    s.dir = d;
  } else if (kind == 2u) {
    s.dir = cone_dir(vec3<f32>(0.0, 1.0, 0.0), sh.y, r.x, r.y);
    let a = r.z * 2.0 * PI;
    let rr = sqrt(r.w) * sh.z;
    s.pos = vec3<f32>(cos(a) * rr, 0.0, sin(a) * rr);
  } else if (kind == 3u) {
    s.pos = (vec3<f32>(r.x, r.y, r.z) * 2.0 - vec3<f32>(1.0)) * sim_entries[ei].shape2.xyz;
  } else if (kind == 4u || kind == 5u) {
    let count = u32(sh.y);
    if (count > 0u) {
      let base = sim_entries[ei].info.w;
      let i = min(u32(r.x * f32(count)), count - 1u);
      s.pos = points_pool[base + i * 2u].xyz;
      s.dir = points_pool[base + i * 2u + 1u].xyz;
    }
  }
  return s;
}

fn spawn(ei: u32, n: u32, k: u32) -> Particle {
  let seed = sim_entries[ei].info.y;
  let r0 = sim_entries[ei].ranges0;
  let r1 = sim_entries[ei].ranges1;
  let r2 = sim_entries[ei].ranges2;
  let life = max(mix(r0.x, r0.y, rand(seed, n, 0u)), 1e-3);
  let speed = mix(r0.z, r0.w, rand(seed, n, 1u));
  let size = mix(r1.x, r1.y, rand(seed, n, 2u));
  let rot = mix(r1.z, r1.w, rand(seed, n, 3u));
  let spin = mix(r2.x, r2.y, rand(seed, n, 4u));
  let s = shape_sample(ei, vec4<f32>(rand(seed, n, 6u), rand(seed, n, 7u), rand(seed, n, 8u), rand(seed, n, 11u)));
  var dir = s.dir;
  let spread = sim_entries[ei].shape2.w;
  if (spread > 0.0) { dir = cone_dir(dir, spread, rand(seed, n, 9u), rand(seed, n, 10u)); }
  let dt = r2.w;
  let bursts = sim_entries[ei].spawn.w;
  var age = 0.0;
  var frac = 1.0;
  if (k >= bursts) {
    let c = f32(max(sim_entries[ei].spawn.y - bursts, 1u));
    let u = (f32(k - bursts) + 0.5) / c;
    age = u * dt;
    frac = 1.0 - u;
  }
  var pos = s.pos;
  var vel = dir * speed;
  if (r2.z < 0.5) {
    // Sub-tick emitter motion interpolation when continuous spawns occur while moving.
    let m = sim_entries[ei].transform;
    if (sim_entries[ei].shape.w > 0.5 && k >= bursts) {
      let pm = sim_entries[ei].prev_transform;
      let p0 = (pm * vec4<f32>(pos, 1.0)).xyz;
      let p1 = (m * vec4<f32>(pos, 1.0)).xyz;
      let v0 = (pm * vec4<f32>(vel, 0.0)).xyz;
      let v1 = (m * vec4<f32>(vel, 0.0)).xyz;
      pos = mix(p0, p1, frac);
      vel = mix(v0, v1, frac) + sim_entries[ei].motion.xyz * sim_entries[ei].motion.w;
    } else {
      pos = (m * vec4<f32>(pos, 1.0)).xyz;
      vel = (m * vec4<f32>(vel, 0.0)).xyz + sim_entries[ei].motion.xyz * sim_entries[ei].motion.w;
    }
  }
  pos += vel * age;
  // What the emission leaves out takes its slot unborn (cpu.rs `spawn`).
  if (rand(seed, n, 15u) >= sim_entries[ei].counts.z) { age = life; }
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
  return points_pool[base + u32((w.z * n + w.y) * n + w.x)].xyz;
}

// A grid field's vector at `g` (cells, tiling), trilinear (grid_sample).
fn grid_sample(ei: u32, g: vec3<f32>) -> vec3<f32> {
  let n = max(i32(sim_entries[ei].field.w), 1);
  let base = sim_entries[ei].pool.z;
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
fn forces(ei: u32, pos: vec3<f32>) -> vec3<f32> {
  var a = vec3<f32>(0.0);
  let n_att = u32(sim_entries[ei].counts.y);
  for (var i = 0u; i < n_att; i++) {
    let at = sim_entries[ei].attract[i];
    let d = at.xyz - pos;
    let dist = length(d);
    if (dist < 1e-4) { continue; }
    let r = sim_entries[ei].attract_r[i];
    var f = at.w / (1.0 + dist * dist);
    if (r > 0.0) { f = at.w * max(1.0 - dist / r, 0.0); }
    a += d * (f / dist);
  }
  let fld = sim_entries[ei].field;
  let kind = u32(fld.x);
  if (kind == 1u) {
    let q = pos * fld.z + sim_entries[ei].field_origin[1].xyz + vec3<f32>(sim_entries[ei].noise.w * fld.w);
    a += curl_noise(q, vec3<i32>(sim_entries[ei].field_origin[0].xyz), sim_entries[ei].info.y) * fld.y;
  } else if (kind == 2u) {
    let g = pos / fld.z * fld.w + sim_entries[ei].field_origin[1].xyz;
    a += grid_sample(ei, g) * fld.y;
  }
  return a;
}

// `v` turned by the unit quaternion `q`.
fn quat_turn(q: vec4<f32>, v: vec3<f32>) -> vec3<f32> {
  return v + 2.0 * cross(q.xyz, cross(q.xyz, v) + q.w * v);
}

fn unit_or_zero(v: vec3<f32>) -> vec3<f32> {
  let l = length(v);
  return select(vec3<f32>(0.0), v / l, l > 0.0);
}

// The og.ForceFields' acceleration at `p` (og-env's `Forces::at`).
fn force_field_at(p: vec3<f32>) -> vec3<f32> {
  var a = vec3<f32>(0.0);
  let n = u32(force_fields[0].x);
  for (var i = 0u; i < n; i++) {
    let b = 1u + i * 6u;
    let at = force_fields[b].xyz;
    let kind = u32(force_fields[b].w + 0.5);
    let q = force_fields[b + 1u];
    let scale = abs(force_fields[b + 2u].xyz);
    let strength = force_fields[b + 2u].w;
    let dir = force_fields[b + 3u].xyz;
    var weight = 1.0;
    if (force_fields[b + 3u].w > 0.5) {
      let size = force_fields[b + 4u].xyz;
      let radius = force_fields[b + 4u].w;
      let fade = force_fields[b + 5u].x;
      var outside: f32;
      if (radius > 0.0) {
        outside = max(length(p - at) - radius * max(scale.x, max(scale.y, scale.z)), 0.0);
      } else {
        let local = quat_turn(vec4<f32>(-q.xyz, q.w), p - at);
        outside = length(max(abs(local) - abs(size) * scale * 0.5, vec3<f32>(0.0)));
      }
      if (outside > 0.0) {
        weight = select(0.0, max(1.0 - outside / fade, 0.0), fade > 0.0);
      }
    }
    if (weight <= 0.0) { continue; }
    let off = p - at;
    var d = dir;
    if (kind == 1u) {
      d = unit_or_zero(off);
    } else if (kind == 2u) {
      d = cross(dir, unit_or_zero(off - dir * dot(dir, off)));
    }
    a += d * (strength * weight);
  }
  return a;
}

struct StepOut {
  p: Particle,
  impact: bool,
};

fn integrate(ei: u32, p: Particle) -> StepOut {
  let dt = sim_entries[ei].ranges2.w;
  var pos = p.pos_age.xyz;
  var vel = p.vel_life.xyz;
  vel += sim_entries[ei].forces.xyz * dt;
  let push = sim_entries[ei].air.w;
  if (push != 0.0) {
    vel += force_field_at(pos) * (push * dt);
  }
  let ns = sim_entries[ei].noise;
  if (ns.x != 0.0) {
    let seed = sim_entries[ei].info.y;
    let w = vec3<i32>(sim_entries[ei].noise_origin[0].xyz);
    let q = pos * ns.y + sim_entries[ei].noise_origin[1].xyz + vec3<f32>(ns.w * ns.z);
    let n = vec3<f32>(value_noise3_at(q, w, seed ^ 0x1111u), value_noise3_at(q, w, seed ^ 0x2222u), value_noise3_at(q, w, seed ^ 0x3333u)) * 2.0 - vec3<f32>(1.0);
    vel += n * ns.x * dt;
  }
  vel += forces(ei, pos) * dt;
  let air = sim_entries[ei].air.xyz;
  vel = air + (vel - air) * (1.0 / (1.0 + max(sim_entries[ei].forces.w, 0.0) * dt));
  let before = pos;
  var speed_scale = 1.0;
  let speed_keys = u32(sim_entries[ei].counts.x);
  if (speed_keys > 0u) {
    let t = clamp(p.pos_age.w / max(p.vel_life.w, 1e-6), 0.0, 1.0);
    speed_scale = curve_at(sim_entries[ei].speed_t, sim_entries[ei].speed_v, speed_keys, t);
  }
  pos += vel * (dt * speed_scale);
  var hit = false;
  var impact = false;
  let col = sim_entries[ei].collision;
  if (col.w > 0.5) {
    let list = sim_entries[ei].info.z;
    if (!push_out(before, 0.0, list).hit) {
      let h = push_out(pos, 0.0, list);
      if (h.hit) {
        let vn = dot(vel, h.n);
        if (vn < 0.0) {
          let normal_part = h.n * vn;
          vel = (vel - normal_part) * (1.0 - col.z) - normal_part * col.y;
          pos = before + vel * dt;
          hit = true;
          impact = (-vn) > IMPACT_SPEED;
          let again = push_out(pos, 0.0, list);
          if (again.hit) { pos = again.p; }
        }
      }
    }
  } else if (col.x > 0.5 && sim_entries[ei].ranges2.z < 0.5) {
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
          let nrm = oct_decode(g);
          let vn = dot(vel, nrm);
          if (vn < 0.0) {
            let normal_part = nrm * vn;
            vel = (vel - normal_part) * (1.0 - col.z) - normal_part * col.y;
            pos = before + vel * dt;
            hit = true;
            impact = (-vn) > IMPACT_SPEED;
          }
        }
      }
    }
  }
  var o = p;
  o.pos_age = vec4<f32>(pos, select(p.pos_age.w + dt, p.vel_life.w, hit && sim_entries[ei].counts.w > 0.5));
  o.vel_life = vec4<f32>(vel, p.vel_life.w);
  return StepOut(o, impact);
}

// ---------------------------------------------------------------------------
// GPU sub-emitters (`Birth`, `Death`, `Collision`, `Life`).
// Each sub-emitter record occupies 4 vec4<u32> elements in `sim_tiles`:
//   d0: (child_entry_idx, counter_idx, trigger, count_or_burst_count)
//   d1: bitcast f32 (inherit_vel, rate, duration, looping)
//   d2: bitcast f32 burst0 (time, count, cycles, interval)
//   d3: bitcast f32 burst1 (time, count, cycles, interval)
// ---------------------------------------------------------------------------

fn child_bursts_in_cycle(b0: vec4<f32>, b1: vec4<f32>, nb: u32, end: f32, dur: f32) -> u32 {
  var n = 0u;
  for (var bi = 0u; bi < nb; bi++) {
    let b = select(b0, b1, bi == 1u);
    let cycles = max(u32(b.z + 0.5), 1u);
    let interval = max(b.w, 0.0);
    let bcount = u32(b.y + 0.5);
    let btime = max(b.x, 0.0);
    for (var ci = 0u; ci < cycles; ci++) {
      let t = btime + f32(ci) * interval;
      if ((t >= dur && dur > 0.0) || t >= end) { break; }
      n += bcount;
    }
  }
  return n;
}

fn child_spawned_before(d1: vec4<f32>, b0: vec4<f32>, b1: vec4<f32>, nb: u32, elapsed: f32) -> u32 {
  let dur = max(d1.z, 1e-6);
  let looping = d1.w > 0.5;
  let end = select(min(elapsed, dur), elapsed, looping);
  if (end <= 0.0) { return 0u; }
  let continuous = u32(floor(max(d1.y, 0.0) * end));
  var bursts = 0u;
  if (looping) {
    let full = u32(floor(end / dur));
    let per_cycle = child_bursts_in_cycle(b0, b1, nb, dur, dur);
    bursts = full * per_cycle + child_bursts_in_cycle(b0, b1, nb, end - f32(full) * dur, dur);
  } else {
    bursts = child_bursts_in_cycle(b0, b1, nb, end, dur);
  }
  return continuous + bursts;
}

fn spawn_child_one(child_ei: u32, counter_idx: u32, at: vec3<f32>, inherited: vec3<f32>, n_hash: u32) {
  let cmax = max(sim_entries[child_ei].info.x, 1u);
  let cseed = sim_entries[child_ei].info.y;
  let cbase = sim_entries[child_ei].pool.x;
  let r0 = sim_entries[child_ei].ranges0;
  let r1 = sim_entries[child_ei].ranges1;
  let r2 = sim_entries[child_ei].ranges2;
  let life = max(mix(r0.x, r0.y, rand(cseed, n_hash, 0u)), 1e-3);
  let speed = mix(r0.z, r0.w, rand(cseed, n_hash, 1u));
  let size = mix(r1.x, r1.y, rand(cseed, n_hash, 2u));
  let rot = mix(r1.z, r1.w, rand(cseed, n_hash, 3u));
  let spin = mix(r2.x, r2.y, rand(cseed, n_hash, 4u));
  let s = shape_sample(child_ei, vec4<f32>(rand(cseed, n_hash, 6u), rand(cseed, n_hash, 7u), rand(cseed, n_hash, 8u), rand(cseed, n_hash, 11u)));
  var dir = s.dir;
  let spread = sim_entries[child_ei].shape2.w;
  if (spread > 0.0) { dir = cone_dir(dir, spread, rand(cseed, n_hash, 9u), rand(cseed, n_hash, 10u)); }
  var age = 0.0;
  if (rand(cseed, n_hash, 15u) >= sim_entries[child_ei].counts.z) { age = life; }
  var cp: Particle;
  cp.pos_age = vec4<f32>(at + s.pos, age);
  cp.vel_life = vec4<f32>(dir * speed + inherited, life);
  cp.misc = vec4<f32>(size, rot, spin, rand(cseed, n_hash, 5u));
  let ring_n = atomicAdd(&sub_counters[counter_idx], 1u);
  particles_pool[cbase + (ring_n % cmax)] = cp;
}

fn emit_sub_events(
  ei: u32,
  p: Particle,
  pid: u32,
  a0: f32,
  a1: f32,
  born: bool,
  died: bool,
  collided: bool,
) {
  let sub_meta = sim_entries[ei].pool.w;
  let sub_count = sub_meta >> 20u;
  if (sub_count == 0u) { return; }
  let sub_base = sub_meta & 0xfffffu;
  var at = p.pos_age.xyz;
  var vel = p.vel_life.xyz;
  if (sim_entries[ei].ranges2.z > 0.5) {
    at = (sim_entries[ei].transform * vec4<f32>(at, 1.0)).xyz;
    vel = (sim_entries[ei].transform * vec4<f32>(vel, 0.0)).xyz;
  }
  for (var sk = 0u; sk < sub_count; sk++) {
    let base = sub_base + sk * 4u;
    let d0 = sim_tiles[base];
    let child_ei = d0.x;
    let counter_idx = d0.y;
    let trigger = d0.z;
    let count = d0.w;
    let d1 = bitcast<vec4<f32>>(sim_tiles[base + 1u]);
    let inherited = vel * d1.x;
    let cmax = max(sim_entries[child_ei].info.x, 1u);
    if (trigger == 3u) {
      let b0 = bitcast<vec4<f32>>(sim_tiles[base + 2u]);
      let b1 = bitcast<vec4<f32>>(sim_tiles[base + 3u]);
      let s0 = child_spawned_before(d1, b0, b1, count, a0);
      let s1 = child_spawned_before(d1, b0, b1, count, a1);
      let n_emit = min(s1 - s0, cmax);
      for (var em = 0u; em < n_emit; em++) {
        let ord = s0 + em;
        let n_hash = pcg(pid ^ pcg(ord + 1u) ^ pcg(sk * 0x9e3779b9u + 7u));
        spawn_child_one(child_ei, counter_idx, at, inherited, n_hash);
      }
    } else {
      let fire = ((trigger == 0u) && born) || ((trigger == 1u) && died) || ((trigger == 2u) && collided);
      if (fire) {
        let n_emit = min(count, cmax);
        let tick_hash = bitcast<u32>(sim_entries[ei].noise.w);
        for (var em = 0u; em < n_emit; em++) {
          let n_hash = pcg(pid ^ pcg(tick_hash + em + 1u) ^ pcg(sk * 0x9e3779b9u + trigger * 17u));
          spawn_child_one(child_ei, counter_idx, at, inherited, n_hash);
        }
      }
    }
  }
}

@compute @workgroup_size(64)
fn simulate(@builtin(workgroup_id) wid: vec3<u32>, @builtin(local_invocation_id) lid: vec3<u32>) {
  if (wid.x >= dispatch.tile_count) { return; }
  let tile = sim_tiles[dispatch.tile_offset + wid.x];
  let ei = tile.x;
  let slot = tile.y + lid.x;
  let max_particles = sim_entries[ei].info.x;
  if (slot >= max_particles) { return; }
  let p_idx = sim_entries[ei].pool.x + slot;
  let count = sim_entries[ei].spawn.y;
  let off = (slot + max_particles - sim_entries[ei].spawn.z) % max_particles;
  let trails = sim_entries[ei].trail.x > 0.0;
  let t_base = sim_entries[ei].pool.y;
  if (off < count) {
    let k = off + ((count - 1u - off) / max_particles) * max_particles;
    let pid = sim_entries[ei].spawn.x + k;
    let p = spawn(ei, pid, k);
    particles_pool[p_idx] = p;
    let alive = p.pos_age.w < p.vel_life.w;
    if (trails && alive) {
      for (var i = 0u; i < TRAIL_POINTS; i++) {
        trails_pool[(t_base + slot) * TRAIL_POINTS + i] = vec4<f32>(p.pos_age.xyz, 0.0);
      }
    }
    if (alive && ((sim_entries[ei].pool.w >> 20u) != 0u)) {
      let a1 = min(p.pos_age.w, p.vel_life.w);
      emit_sub_events(ei, p, pid, 0.0, a1, true, false, false);
    }
    return;
  }
  let p = particles_pool[p_idx];
  if (p.pos_age.w < p.vel_life.w) {
    let before_age = p.pos_age.w;
    let res = integrate(ei, p);
    let q = res.p;
    particles_pool[p_idx] = q;
    let alive = q.pos_age.w < q.vel_life.w;
    if (trails && (sim_entries[ei].trail.w >= 0.0) && alive) {
      trails_pool[(t_base + slot) * TRAIL_POINTS + u32(sim_entries[ei].trail.w)] = vec4<f32>(q.pos_age.xyz, 0.0);
    }
    if ((sim_entries[ei].pool.w >> 20u) != 0u) {
      let first_m = sim_entries[ei].spawn.z;
      let last_m = (first_m + max_particles - 1u) % max_particles;
      let back = (last_m + max_particles - slot) % max_particles;
      let pid = sim_entries[ei].spawn.x - 1u - back;
      let a1 = min(q.pos_age.w, q.vel_life.w);
      let a0 = min(before_age, a1);
      emit_sub_events(ei, q, pid, a0, a1, false, !alive, res.impact);
    }
  }
}

// Empties every slot of reset systems (and resets any child's atomic counter).
@compute @workgroup_size(64)
fn clear(@builtin(workgroup_id) wid: vec3<u32>, @builtin(local_invocation_id) lid: vec3<u32>) {
  if (wid.x >= dispatch.tile_count) { return; }
  let tile = sim_tiles[dispatch.tile_offset + wid.x];
  let ei = tile.x;
  let slot = tile.y + lid.x;
  let max_particles = sim_entries[ei].info.x;
  if (slot >= max_particles) { return; }
  var p: Particle;
  p.pos_age = vec4<f32>(0.0);
  p.vel_life = vec4<f32>(0.0);
  p.misc = vec4<f32>(0.0);
  particles_pool[sim_entries[ei].pool.x + slot] = p;
  if (slot == 0u && sim_entries[ei].pool.w != 0xffffffffu) {
    atomicStore(&sub_counters[sim_entries[ei].pool.w], 0u);
  }
}

// Moves every slot by the entry's translation (a move of the render origin).
@compute @workgroup_size(64)
fn shift(@builtin(workgroup_id) wid: vec3<u32>, @builtin(local_invocation_id) lid: vec3<u32>) {
  if (wid.x >= dispatch.tile_count) { return; }
  let tile = sim_tiles[dispatch.tile_offset + wid.x];
  let ei = tile.x;
  let slot = tile.y + lid.x;
  let max_particles = sim_entries[ei].info.x;
  if (slot >= max_particles) { return; }
  let p_idx = sim_entries[ei].pool.x + slot;
  let delta = sim_entries[ei].transform[3].xyz;
  var p = particles_pool[p_idx];
  p.pos_age = vec4<f32>(p.pos_age.xyz + delta, p.pos_age.w);
  particles_pool[p_idx] = p;
  if (sim_entries[ei].trail.x > 0.0) {
    let t_base = sim_entries[ei].pool.y;
    for (var i = 0u; i < TRAIL_POINTS; i++) {
      let at = (t_base + slot) * TRAIL_POINTS + i;
      trails_pool[at] = vec4<f32>(trails_pool[at].xyz + delta, 0.0);
    }
  }
}
