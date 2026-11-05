// The debris layer (debris.rs): fine pieces of broken props as rigid
// bodies simulated on the GPU, colliding with the scene's colliders
// (scene_colliders.wgsl, prepended) at their hull's vertices, and drawn as
// ordinary instances whose transforms `write` puts into the instance
// buffers in place. Visual only: nothing reads them back.
//
// The solver is XPBD's rigid bodies (Müller, Macklin, Chentanez, Jeschke,
// Kim, "Detailed Rigid Body Simulation with Extended Position Based
// Dynamics", SCA 2020, algorithms 2 and 3): substeps, each integrating,
// projecting the contacts out of the colliders with static friction, then
// the velocities from the moved poses with dynamic friction and
// restitution. A piece is one body against a static world, so its mass
// cancels: the solver works per unit mass.

struct Params {
  // x: bodies, y: ticks this frame, z: substeps a tick, w: writes
  counts: vec4<u32>,
  // x: gravity (m/s², down), y: the tick's length (s), zw: unused
  step: vec4<f32>,
  // xyz: the render origin's move since the last frame (added to every
  // pose), w: unused
  shift: vec4<f32>,
  // x: this dispatch's first write (its count is counts.w)
  writes_at: vec4<u32>,
}

struct Body {
  pos: vec4<f32>,       // xyz; w: the friction coefficient
  rot: vec4<f32>,       // quaternion (x, y, z, w)
  vel: vec4<f32>,       // xyz; w: the restitution
  ang: vec4<f32>,       // xyz; w: ticks it has been still
  prev_pos: vec4<f32>,  // the pose the last frame drew (motion vectors)
  prev_rot: vec4<f32>,
  scale: vec4<f32>,     // xyz; w: how far from home it may go
  inv_inertia: vec4<f32>, // the body frame's diagonal, per unit mass
  home: vec4<f32>,      // where it broke off
  // x: hull points, y: the ticks a body first seen this frame steps (from
  // the tick it broke off), z: flags (1 asleep, 2 gone), w: 1 while first
  // seen
  info: vec4<u32>,
}

struct Write {
  // x: the record in `instances`, y: the body, z: the record in
  // `previous` (0xffffffff: none), w: unused
  at: vec4<u32>,
  // The body frame to the mesh: its columns.
  c0: vec4<f32>,
  c1: vec4<f32>,
  c2: vec4<f32>,
  c3: vec4<f32>,
}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read_write> bodies: array<Body>;
@group(0) @binding(2) var<storage, read> hull: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read> colliders: array<vec4<f32>>;
@group(0) @binding(4) var<storage, read> lists: array<u32>;
@group(0) @binding(5) var<storage, read> writes: array<Write>;
@group(0) @binding(6) var<storage, read_write> instances: array<vec4<f32>>;
@group(0) @binding(7) var<storage, read_write> previous: array<vec4<f32>>;

const POINTS: u32 = 16u;
const ASLEEP: u32 = 1u;
const GONE: u32 = 2u;
// Still for this many ticks (below SLEEP_V and SLEEP_W), a body sleeps.
const SLEEP_TICKS: f32 = 20.0;
const SLEEP_V: f32 = 0.08;
const SLEEP_W: f32 = 0.3;
const MAX_SPEED: f32 = 60.0;
// The most ticks a body catches up in one frame.
const MAX_CATCH_UP: u32 = 600u;

fn qmul(a: vec4<f32>, b: vec4<f32>) -> vec4<f32> {
  return vec4<f32>(
    a.w * b.xyz + b.w * a.xyz + cross(a.xyz, b.xyz),
    a.w * b.w - dot(a.xyz, b.xyz));
}

fn qrot(q: vec4<f32>, v: vec3<f32>) -> vec3<f32> {
  let t = 2.0 * cross(q.xyz, v);
  return v + q.w * t + cross(q.xyz, t);
}

fn qconj(q: vec4<f32>) -> vec4<f32> { return vec4<f32>(-q.xyz, q.w); }

// The world inverse inertia (per unit mass) applied to `v`.
fn inv_inertia(b: Body, q: vec4<f32>, v: vec3<f32>) -> vec3<f32> {
  return qrot(q, b.inv_inertia.xyz * qrot(qconj(q), v));
}

// The generalised inverse mass of a unit-mass body at arm `r` along `n`.
fn gen_inv_mass(b: Body, q: vec4<f32>, r: vec3<f32>, n: vec3<f32>) -> f32 {
  let rn = cross(r, n);
  return 1.0 + dot(rn, inv_inertia(b, q, rn));
}

// Moves the body by the positional impulse `p` at arm `r` (eq. 6-9).
fn apply_position(b: Body, x: ptr<function, vec3<f32>>, q: ptr<function, vec4<f32>>, r: vec3<f32>, p: vec3<f32>) {
  *x = *x + p;
  let w = inv_inertia(b, *q, cross(r, p));
  *q = normalize(*q + 0.5 * qmul(vec4<f32>(w, 0.0), *q));
}

fn step_body(i: u32, ticks: u32) {
  var b = bodies[i];
  var x = b.pos.xyz;
  var q = b.rot;
  var v = b.vel.xyz;
  var w = b.ang.xyz;
  var still = b.ang.w;
  var flags = b.info.z;
  let list = lists[i];
  let n_points = min(b.info.x, POINTS);
  let substeps = max(params.counts.z, 1u);
  let h = params.step.y / f32(substeps);
  let gravity = vec3<f32>(0.0, -params.step.x, 0.0);
  let friction = b.pos.w;
  let restitution = b.vel.w;
  for (var t = 0u; t < ticks; t++) {
    if ((flags & (ASLEEP | GONE)) != 0u) { break; }
    for (var s = 0u; s < substeps; s++) {
      let x_prev = x;
      let q_prev = q;
      x = x + h * v + h * h * gravity;
      v = v + h * gravity;
      q = normalize(q + 0.5 * h * qmul(vec4<f32>(w, 0.0), q));
      // Contacts: each hull point pushed out of the colliders.
      var normals: array<vec4<f32>, 16>;
      var arms: array<vec3<f32>, 16>;
      var pre_vn: array<f32, 16>;
      for (var k = 0u; k < n_points; k++) {
        normals[k] = vec4<f32>(0.0);
        let local = hull[i * POINTS + k].xyz * b.scale.xyz;
        let r = qrot(q, local);
        let p = x + r;
        let hit = push_out(p, 0.0, list);
        if (!hit.hit) { continue; }
        let n = hit.n;
        let depth = dot(hit.p - p, n);
        if (depth <= 0.0) { continue; }
        let wn = gen_inv_mass(b, q, r, n);
        let dl = depth / wn;
        apply_position(b, &x, &q, r, dl * n);
        // Static friction: the point's slide since the substep began,
        // undone while the friction can hold it (|Δλt| < μ λn).
        let r0 = qrot(q_prev, local);
        let r1 = qrot(q, local);
        let slide = (x + r1) - (x_prev + r0);
        let st = slide - n * dot(slide, n);
        let len = length(st);
        if (len > 1e-7) {
          let tdir = st / len;
          let wt = gen_inv_mass(b, q, r1, tdir);
          let dlt = len / wt;
          if (dlt < friction * dl) {
            apply_position(b, &x, &q, r1, -dlt * tdir);
          }
        }
        // The point's normal speed before this substep's solve.
        let vel_pre = v + cross(w, r);
        pre_vn[k] = dot(n, vel_pre);
        normals[k] = vec4<f32>(n, dl);
        arms[k] = qrot(q, local);
      }
      // Velocities from the poses.
      v = (x - x_prev) / h;
      let dq = qmul(q, qconj(q_prev));
      w = 2.0 * dq.xyz / h;
      if (dq.w < 0.0) { w = -w; }
      // Dynamic friction and restitution at each contact.
      for (var k = 0u; k < n_points; k++) {
        let c = normals[k];
        if (c.w <= 0.0) { continue; }
        let n = c.xyz;
        let r = arms[k];
        let vel = v + cross(w, r);
        let vn = dot(n, vel);
        let vt = vel - n * vn;
        var dv = vec3<f32>(0.0);
        let vt_len = length(vt);
        if (vt_len > 1e-6) {
          // |f_n| h = λn / h.
          dv = dv - vt / vt_len * min(friction * c.w / h, vt_len);
        }
        var e = restitution;
        if (abs(vn) <= 2.0 * params.step.x * h) { e = 0.0; }
        dv = dv + n * (-vn + max(-e * pre_vn[k], 0.0));
        let wn = gen_inv_mass(b, q, r, normalize(dv + vec3<f32>(1e-12)));
        let p = dv / wn;
        v = v + p;
        w = w + inv_inertia(b, q, cross(r, p));
      }
    }
    let speed = length(v);
    if (speed > MAX_SPEED) { v = v * (MAX_SPEED / speed); }
    if (speed < SLEEP_V && length(w) < SLEEP_W) {
      still = still + 1.0;
      if (still >= SLEEP_TICKS) {
        flags = flags | ASLEEP;
        v = vec3<f32>(0.0);
        w = vec3<f32>(0.0);
      }
    } else {
      still = 0.0;
    }
    if (length(x - b.home.xyz) > b.scale.w) { flags = flags | GONE; }
  }
  b.pos = vec4<f32>(x, b.pos.w);
  b.rot = q;
  b.vel = vec4<f32>(v, b.vel.w);
  b.ang = vec4<f32>(w, still);
  b.info.z = flags;
  bodies[i] = b;
}

@compute @workgroup_size(64)
fn step(@builtin(global_invocation_id) id: vec3<u32>) {
  let i = id.x;
  if (i >= params.counts.x) { return; }
  let shift = params.shift.xyz;
  var b = bodies[i];
  b.pos = vec4<f32>(b.pos.xyz + shift, b.pos.w);
  b.home = vec4<f32>(b.home.xyz + shift, b.home.w);
  // The pose the last frame drew, for this frame's motion vectors.
  b.prev_pos = vec4<f32>(b.pos.xyz, 0.0);
  b.prev_rot = b.rot;
  // A body first seen this frame steps from the tick it broke off; the
  // others, this frame's ticks.
  var ticks = params.counts.y;
  if (b.info.w == 1u) { ticks = min(b.info.y, MAX_CATCH_UP); }
  b.info.y = 0u;
  b.info.w = 0u;
  bodies[i] = b;
  step_body(i, ticks);
}

fn columns(x: vec3<f32>, q: vec4<f32>, s: vec3<f32>, wr: Write) -> array<vec3<f32>, 4> {
  let ax = qrot(q, vec3<f32>(s.x, 0.0, 0.0));
  let ay = qrot(q, vec3<f32>(0.0, s.y, 0.0));
  let az = qrot(q, vec3<f32>(0.0, 0.0, s.z));
  let m = mat4x4<f32>(vec4<f32>(ax, 0.0), vec4<f32>(ay, 0.0), vec4<f32>(az, 0.0), vec4<f32>(x, 1.0));
  let l = mat4x4<f32>(wr.c0, wr.c1, wr.c2, wr.c3);
  let o = m * l;
  return array<vec3<f32>, 4>(o[0].xyz, o[1].xyz, o[2].xyz, o[3].xyz);
}

// An instance record's 12 transform words (instance.wgsl): three vec4s.
fn store(buf: u32, at: u32, c: array<vec3<f32>, 4>) {
  let a = vec4<f32>(c[0], c[1].x);
  let b = vec4<f32>(c[1].yz, c[2].xy);
  let d = vec4<f32>(c[2].z, c[3]);
  if (buf == 0u) {
    instances[at * 4u] = a;
    instances[at * 4u + 1u] = b;
    instances[at * 4u + 2u] = d;
  } else {
    previous[at * 4u] = a;
    previous[at * 4u + 1u] = b;
    previous[at * 4u + 2u] = d;
  }
}

@compute @workgroup_size(64)
fn write(@builtin(global_invocation_id) id: vec3<u32>) {
  let k = id.x;
  if (k >= params.counts.w) { return; }
  let wr = writes[params.writes_at.x + k];
  let b = bodies[wr.at.y];
  // A piece gone past its reach is drawn at no size.
  var s = b.scale.xyz;
  if ((b.info.z & GONE) != 0u) { s = vec3<f32>(0.0); }
  store(0u, wr.at.x, columns(b.pos.xyz, b.rot, s, wr));
  if (wr.at.z != 0xffffffffu) {
    store(1u, wr.at.z, columns(b.prev_pos.xyz, b.prev_rot, s, wr));
  }
}
