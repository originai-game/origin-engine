// The trample map (trample.rs): a camera-centred grid of 0.25 m cells,
// stepped once per tick on the GPU, recording where GrassPushers pressed
// the vegetation (Wohllaib, *Procedural Grass in Ghost of Tsushima*, GDC
// 2021: blades bend away from the player and stay crushed, recovering
// slowly). Each cell holds the push (xy: away from the pusher, its length
// the crush, 0..1), the lowest a pusher reached over it (z, world y from
// the frame's origin: blades shorter than that are not touched), and the
// seconds since it was last pressed (w), so each field recovers at its own
// rate where it reads it (grass_trample_at). Visual only: never read back.
//
// One bind group; both entry points are dispatched over (n / 8, n / 8).

struct Trample {
  // xy: the grid's min corner (absolute world x, z), z: its cell (m), w:
  // cells a side.
  corner: vec4<f32>,
  // xy: the cells the grid moved since the last step, z: 1 to start again
  // (nothing pressed), w: the step (s).
  shift: vec4<f32>,
  // x: the pushers, y: the frame origin's y (absolute).
  count: vec4<f32>,
  // Per pusher: (x, z absolute, radius, strength 0..1), (bottom y
  // absolute, top y absolute, 0, 0).
  pushers: array<vec4<f32>, 64>,
};

@group(0) @binding(0) var<uniform> trample: Trample;
@group(0) @binding(1) var<storage, read_write> state: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> scratch: array<vec4<f32>>;
@group(0) @binding(3) var trample_out: texture_storage_2d<rgba32float, write>;

// Seconds a cell's age stops counting at (any recovery is over by then).
const TRAMPLE_MAX_AGE: f32 = 600.0;
// Above this a pusher is too high over the ground to touch it (m).
const TRAMPLE_NONE: f32 = 1e6;

fn trample_n() -> i32 { return i32(trample.corner.w); }

fn trample_inside(c: vec2<i32>) -> bool {
  let n = trample_n();
  return c.x >= 0 && c.y >= 0 && c.x < n && c.y < n;
}

// Carries the state along the grid's scroll, ages it, and presses it under
// this tick's pushers: the push points away from the pusher's axis, its
// crush a smooth fall to the capsule's edge; a cell pressed again keeps
// the stronger push and starts its age again.
@compute @workgroup_size(8, 8, 1)
fn press(@builtin(global_invocation_id) id: vec3<u32>) {
  let c = vec2<i32>(id.xy);
  if (!trample_inside(c)) { return; }
  let n = trample_n();
  let src = c + vec2<i32>(trample.shift.xy);
  var s = vec4<f32>(0.0, 0.0, TRAMPLE_NONE, TRAMPLE_MAX_AGE);
  if (trample.shift.z < 0.5 && trample_inside(src)) {
    s = state[src.y * n + src.x];
    s.w = min(s.w + trample.shift.w, TRAMPLE_MAX_AGE);
  }
  let at = trample.corner.xy + (vec2<f32>(c) + 0.5) * trample.corner.z;
  var lowest = TRAMPLE_NONE;
  let count = u32(trample.count.x);
  // First pass: find the lowest active pusher over this cell so flying pushers
  // cannot override a ground pusher or freeze a ground trail's recovery.
  for (var i = 0u; i < count; i++) {
    let a = trample.pushers[2u * i];
    let b = trample.pushers[2u * i + 1u];
    if (b.y < b.x) { continue; }
    let r = max(a.z, 1e-3);
    if (length(at - a.xy) >= r) { continue; }
    lowest = min(lowest, b.x);
  }
  var best = 0.0;
  var dir = vec2<f32>(1.0, 0.0);
  var bottom = TRAMPLE_NONE;
  for (var i = 0u; i < count; i++) {
    let a = trample.pushers[2u * i];
    let b = trample.pushers[2u * i + 1u];
    if (b.y < b.x || b.x > lowest + 2.0) { continue; }
    let d = at - a.xy;
    let dist = length(d);
    let r = max(a.z, 1e-3);
    if (dist >= r) { continue; }
    let crush = a.w * smoothstep(r, 0.35 * r, dist);
    bottom = min(bottom, b.x);
    if (crush > best) {
      best = crush;
      // At the very centre a blade is pressed flat along a fixed way.
      dir = select(vec2<f32>(1.0, 0.0), d / dist, dist > 1e-4);
    }
  }
  if (best > 0.0) {
    let old = length(s.xy);
    let has_old = old > 0.0 && s.w < TRAMPLE_MAX_AGE;
    // Ignore a pusher passing high above an existing ground track so it
    // neither overwrites the track direction nor freezes its recovery timer.
    if (!has_old || bottom <= s.z + 2.0) {
      // Still under a ground-level pusher since the last step: keep the
      // strongest press until the pusher leaves, then age from there.
      if (has_old && s.w < 0.5 && old > best && abs(bottom - s.z) <= 2.0) {
        best = old;
        dir = s.xy / old;
      }
      let next_bottom = select(bottom, min(bottom, s.z), has_old && abs(bottom - s.z) <= 2.0);
      s = vec4<f32>(dir * best, next_bottom, 0.0);
    }
  }
  scratch[c.y * n + c.x] = s;
}

// The step's result back into the state (absolute bottom y, immune to
// WorldOrigin.y shifts) and the frame-relative map the fields read.
@compute @workgroup_size(8, 8, 1)
fn store(@builtin(global_invocation_id) id: vec3<u32>) {
  let c = vec2<i32>(id.xy);
  if (!trample_inside(c)) { return; }
  let i = c.y * trample_n() + c.x;
  let s = scratch[i];
  state[i] = s;
  let rel_y = select(s.z - trample.count.y, TRAMPLE_NONE, s.z >= TRAMPLE_NONE * 0.5);
  textureStore(trample_out, c, vec4<f32>(s.xy, rel_y, s.w));
}
