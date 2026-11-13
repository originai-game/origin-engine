// The simulated wind field (wind_field.rs): a camera-centred 2D velocity
// field in two levels (fine near the camera, coarse beyond), advanced once
// per tick on the GPU by a stable-fluids step (Stam, *Stable Fluids*,
// SIGGRAPH 1999: semi-Lagrangian advection, forces, a Jacobi pressure
// solve and projection), forced towards og.Wind's mean in its travelling
// gusts (wind_common.wgsl wind_gust_share), stirred by curl-noise
// turbulence and pushed by WindMotors (Rockenbeck, *Blowing from the West:
// Simulating Wind in Ghost of Tsushima*, GDC 2021: motors in a field every
// tier reads). Each cell also carries what vegetation leans by: a damped
// spring driven by the velocity, so gusts arrive and settle rather than
// snapping. Visual only and never read back: a function of the ticks it
// stepped and their inputs, the same on every run.
//
// Prepended by wind_common.wgsl. One bind group for every entry point;
// each is dispatched over (n / 8, n / 8, 2): x, y the cell, z the level.

struct Sim {
  // Per level: xy its min corner in the world (absolute x, z), z its cell
  // (m); the cells its grid moved since the last step (read at c + shift)
  // in shift.xy, shift.z 1 to start it again from the wind's target, w 1
  // when the lean last drawn is kept through the restart (motion vectors).
  corner: array<vec4<f32>, 2>,
  shift: array<vec4<i32>, 2>,
  // og.Wind as WindField (its gusts and wave at this tick: `previous` false).
  wind: WindField,
  // x: the reference speed (m/s; og.Wind.speed, at least 1), y: the lean
  // per m/s (og.Wind.strength / the reference speed, or WIND_CALM_LEAN),
  // z: turbulence (0..1), w: the gust patches' size (m).
  drive: vec4<f32>,
  // x: the step (s), y: 1 on the first step of a frame (the drawn lean
  // moves to `previous`), 2 on the last step of a start with no frame
  // drawn before (`previous` is the lean this step reaches: no motion),
  // z: the motors, w: cells a side.
  step: vec4<f32>,
  // Per motor two rows: (x, z absolute, radius, strength m/s) and
  // (direction x, z, kind: 0 omni, 1 directional, 2 vortex, 0).
  motors: array<vec4<f32>, 32>,
};

@group(0) @binding(0) var<uniform> sim: Sim;
// The state: velocity (xy, m/s) and lean (zw); the lean's rate (xy) and
// its value as last drawn (zw). `_a` persists; `_b` holds a step's middle.
@group(0) @binding(1) var<storage, read_write> vel_a: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> aux_a: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read_write> vel_b: array<vec4<f32>>;
@group(0) @binding(4) var<storage, read_write> aux_b: array<vec4<f32>>;
// Pressure (kept as the next step's first guess), its other half, and the
// velocity's divergence.
@group(0) @binding(5) var<storage, read_write> pressure: array<f32>;
@group(0) @binding(6) var<storage, read_write> pressure_b: array<f32>;
@group(0) @binding(7) var<storage, read_write> divergence: array<f32>;
// What every tier reads (wind_common.wgsl wind_lean): n x 2n, the fine
// level's rows then the coarse level's, each texel the lean now (xy) and
// as last drawn (zw).
@group(0) @binding(8) var lean_out: texture_storage_2d<rgba16float, write>;

// The time over which the velocity relaxes towards og.Wind's (s).
const WIND_RELAX: f32 = 0.6;
// The lean's spring: its natural frequency (Hz) and damping ratio.
const WIND_SPRING_HZ: f32 = 1.6;
const WIND_SPRING_DAMPING: f32 = 0.45;
// The most a cell leans.
const WIND_MAX_LEAN: f32 = 1.5;

fn n_side() -> i32 { return i32(sim.step.w); }

fn cell_index(c: vec2<i32>, level: i32) -> i32 {
  let n = n_side();
  return level * n * n + c.y * n + c.x;
}

fn inside_grid(c: vec2<i32>) -> bool {
  let n = n_side();
  return c.x >= 0 && c.y >= 0 && c.x < n && c.y < n;
}

fn clamp_cell(c: vec2<i32>) -> vec2<i32> {
  return clamp(c, vec2<i32>(0), vec2<i32>(n_side() - 1));
}

// The centre of cell `c` of `level` in the world (absolute x, z).
fn cell_world(c: vec2<f32>, level: i32) -> vec2<f32> {
  let k = sim.corner[level];
  return k.xy + (c + 0.5) * k.z;
}

// og.Wind's velocity at world point `at`: downwind at the reference speed
// times one plus its gust times the gusts' share.
fn target_velocity(at: vec2<f32>) -> vec2<f32> {
  let w = sim.wind;
  if (w.dir.z <= 0.0) { return vec2<f32>(0.0); }
  // The gust patches at og.Wind.gustSize (wind_gust's cell is WIND_GUST_CELL).
  let scaled = at * (WIND_GUST_CELL / max(sim.drive.w, 1.0));
  let g = wind_gust_share(w, scaled, false);
  return w.dir.xy * sim.drive.x * (1.0 + w.dir.w * g);
}

// What a cell leans by in a velocity `u`.
fn lean_of(u: vec2<f32>) -> vec2<f32> {
  let l = u * sim.drive.y;
  let m = length(l);
  return select(l, l * (WIND_MAX_LEAN / m), m > WIND_MAX_LEAN);
}

// The previous step's velocity of `level` at cell coordinates `q` (cell
// units, the previous grid's), bilinear; the wind's target off the grid.
fn sample_velocity(q: vec2<f32>, level: i32, at: vec2<f32>) -> vec2<f32> {
  let n = f32(n_side());
  if (q.x < -0.5 || q.y < -0.5 || q.x > n - 0.5 || q.y > n - 0.5) {
    return target_velocity(at);
  }
  let c = clamp(q, vec2<f32>(0.0), vec2<f32>(n - 1.001));
  let i = vec2<i32>(floor(c));
  let f = c - floor(c);
  let a = vel_a[cell_index(i, level)].xy;
  let b = vel_a[cell_index(i + vec2<i32>(1, 0), level)].xy;
  let d = vel_a[cell_index(i + vec2<i32>(0, 1), level)].xy;
  let e = vel_a[cell_index(i + vec2<i32>(1, 1), level)].xy;
  return mix(mix(a, b, f.x), mix(d, e, f.x), f.y);
}

// Turbulence: the curl of a scalar noise drifting downwind, so it adds no
// divergence (Bridson, Hourihan, Nordenstam, *Curl-noise for procedural
// fluid flow*, SIGGRAPH 2007), in m/s at the reference speed.
fn curl_noise(at: vec2<f32>) -> vec2<f32> {
  let w = sim.wind;
  let size = max(sim.drive.w, 1.0) * 0.5;
  let p = at / size - w.offset.xy * (WIND_GUST_CELL / size) + vec2<f32>(91.0, 17.0);
  let e = 0.25;
  let nx = wind_noise(p + vec2<f32>(e, 0.0)) - wind_noise(p - vec2<f32>(e, 0.0));
  let ny = wind_noise(p + vec2<f32>(0.0, e)) - wind_noise(p - vec2<f32>(0.0, e));
  return vec2<f32>(ny, -nx) / (2.0 * e);
}

// The motors' push at world point `at` (m/s): each within its radius,
// fading to its edge as (1 - d / r)²; omni outward, directional along its
// direction, vortex around it (counter-clockwise seen from above).
fn motor_velocity(at: vec2<f32>) -> vec2<f32> {
  var u = vec2<f32>(0.0);
  let count = u32(sim.step.z);
  for (var i = 0u; i < count; i++) {
    let a = sim.motors[2u * i];
    let b = sim.motors[2u * i + 1u];
    let d = at - a.xy;
    let r = max(a.z, 1e-3);
    let dist = length(d);
    if (dist >= r) { continue; }
    let fall = (1.0 - dist / r) * (1.0 - dist / r);
    let out = select(vec2<f32>(0.0), d / dist, dist > 1e-4);
    var dir = out;
    if (b.z > 1.5) {
      dir = vec2<f32>(out.y, -out.x);
    } else if (b.z > 0.5) {
      dir = b.xy;
    }
    u += dir * a.w * fall;
  }
  return u;
}

// Advection and forces: the previous step's velocity carried along itself
// (from where the grid's scroll put it), relaxed towards og.Wind's, pushed
// by the motors and stirred; the lean and its rate carried by the scroll
// alone (vegetation does not move with the air).
@compute @workgroup_size(8, 8, 1)
fn advect(@builtin(global_invocation_id) id: vec3<u32>) {
  let c = vec2<i32>(id.xy);
  let level = i32(id.z);
  if (!inside_grid(c) || level > 1) { return; }
  let i = cell_index(c, level);
  let at = cell_world(vec2<f32>(c), level);
  let sh = sim.shift[level];
  let src = c + sh.xy;
  let dt = sim.step.x;
  let cell = sim.corner[level].z;
  if (sh.z != 0 || !inside_grid(src)) {
    let u = target_velocity(at);
    let l = lean_of(u);
    // A restart after drawn frames keeps what they last drew, so the
    // first frame after a jump still moves from it.
    var drawn = l;
    if (sh.z != 0 && sh.w != 0 && inside_grid(src)) { drawn = vel_a[cell_index(src, level)].zw; }
    vel_b[i] = vec4<f32>(u, l);
    aux_b[i] = vec4<f32>(0.0, 0.0, drawn);
    return;
  }
  let j = cell_index(src, level);
  let here = vel_a[j];
  let back = vec2<f32>(src) - here.xy * dt / cell;
  var u = sample_velocity(back, level, at - here.xy * dt);
  let goal = target_velocity(at) + motor_velocity(at);
  u += (goal - u) * (1.0 - exp(-dt / WIND_RELAX));
  u += curl_noise(at) * sim.drive.z * sim.drive.x * dt / WIND_RELAX;
  vel_b[i] = vec4<f32>(u, here.zw);
  aux_b[i] = aux_a[j];
}

// The velocity at cell `c` less the motors' push there: what the
// projection keeps free of divergence.
fn unforced(c: vec2<i32>, level: i32) -> vec2<f32> {
  let k = clamp_cell(c);
  return vel_b[cell_index(k, level)].xy - motor_velocity(cell_world(vec2<f32>(k), level));
}

// The velocity's divergence (per cell), the solid edges mirrored, less
// the motors' own. The field is a slice of the air near the ground: a
// motor's outflow (an omni blast spreading from its centre, a gust
// fading at a directional's edge) is fed from and lost to the air above,
// a source the projection must keep (Stam, *Stable Fluids*, 1999, §3's
// sources; Ghost of Tsushima's motors push the air outward). Projected
// whole, an omni motor's push cancelled itself and moved nothing.
@compute @workgroup_size(8, 8, 1)
fn divergence_pass(@builtin(global_invocation_id) id: vec3<u32>) {
  let c = vec2<i32>(id.xy);
  let level = i32(id.z);
  if (!inside_grid(c) || level > 1) { return; }
  let l = unforced(c - vec2<i32>(1, 0), level).x;
  let r = unforced(c + vec2<i32>(1, 0), level).x;
  let b = unforced(c - vec2<i32>(0, 1), level).y;
  let t = unforced(c + vec2<i32>(0, 1), level).y;
  let i = cell_index(c, level);
  divergence[i] = 0.5 * (r - l + t - b);
  if (sim.shift[level].z != 0) { pressure[i] = 0.0; }
}

fn jacobi(c: vec2<i32>, level: i32, from_b: bool) -> f32 {
  let ls = cell_index(clamp_cell(c - vec2<i32>(1, 0)), level);
  let rs = cell_index(clamp_cell(c + vec2<i32>(1, 0)), level);
  let bs = cell_index(clamp_cell(c - vec2<i32>(0, 1)), level);
  let ts = cell_index(clamp_cell(c + vec2<i32>(0, 1)), level);
  var sum = 0.0;
  if (from_b) {
    sum = pressure_b[ls] + pressure_b[rs] + pressure_b[bs] + pressure_b[ts];
  } else {
    sum = pressure[ls] + pressure[rs] + pressure[bs] + pressure[ts];
  }
  return (sum - divergence[cell_index(c, level)]) * 0.25;
}

// One Jacobi iteration of the pressure, `pressure` into `pressure_b`.
@compute @workgroup_size(8, 8, 1)
fn jacobi_a(@builtin(global_invocation_id) id: vec3<u32>) {
  let c = vec2<i32>(id.xy);
  let level = i32(id.z);
  if (!inside_grid(c) || level > 1) { return; }
  pressure_b[cell_index(c, level)] = jacobi(c, level, false);
}

// And back.
@compute @workgroup_size(8, 8, 1)
fn jacobi_b(@builtin(global_invocation_id) id: vec3<u32>) {
  let c = vec2<i32>(id.xy);
  let level = i32(id.z);
  if (!inside_grid(c) || level > 1) { return; }
  pressure[cell_index(c, level)] = jacobi(c, level, true);
}

// Projection (the pressure's gradient taken out, leaving swirls rather
// than sources), then the lean's spring stepped towards what the velocity
// leans it by; the state back into `_a` and the lean into `lean_out`.
@compute @workgroup_size(8, 8, 1)
fn project(@builtin(global_invocation_id) id: vec3<u32>) {
  let c = vec2<i32>(id.xy);
  let level = i32(id.z);
  if (!inside_grid(c) || level > 1) { return; }
  let i = cell_index(c, level);
  let pl = pressure[cell_index(clamp_cell(c - vec2<i32>(1, 0)), level)];
  let pr = pressure[cell_index(clamp_cell(c + vec2<i32>(1, 0)), level)];
  let pb = pressure[cell_index(clamp_cell(c - vec2<i32>(0, 1)), level)];
  let pt = pressure[cell_index(clamp_cell(c + vec2<i32>(0, 1)), level)];
  let v = vel_b[i];
  let u = v.xy - 0.5 * vec2<f32>(pr - pl, pt - pb);
  var lean = v.zw;
  var a = aux_b[i];
  if (sim.step.y > 0.5 && sim.shift[level].z == 0) { a = vec4<f32>(a.xy, lean); }
  let goal = lean_of(u);
  let omega = WIND_TAU * WIND_SPRING_HZ;
  // Semi-implicit Euler in sub-steps of at most 1/60 s, stable for the
  // spring's frequency.
  let steps = max(ceil(sim.step.x * 60.0), 1.0);
  let h = sim.step.x / steps;
  var rate = a.xy;
  for (var k = 0.0; k < steps; k += 1.0) {
    rate += (omega * omega * (goal - lean) - 2.0 * WIND_SPRING_DAMPING * omega * rate) * h;
    lean += rate * h;
  }
  let m = length(lean);
  if (m > WIND_MAX_LEAN) { lean *= WIND_MAX_LEAN / m; }
  if (sim.step.y > 1.5) { a = vec4<f32>(a.xy, lean); }
  vel_a[i] = vec4<f32>(u, lean);
  aux_a[i] = vec4<f32>(rate, a.zw);
  textureStore(lean_out, vec2<i32>(c.x, level * n_side() + c.y), vec4<f32>(lean, a.zw));
}
