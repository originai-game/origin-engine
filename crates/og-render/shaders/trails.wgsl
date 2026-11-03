// Trail histories on the GPU (ribbons_gpu.rs): each trail keeps a ring of
// the positions its head passed, one per elapsed tick, for its style's
// lifetime, and writes its strip's points (head first) for ribbon.wgsl's
// `vs_strip` to face each camera. One invocation per trail and frame;
// stepped by the scene's ticks, never by the wall clock. Picture only:
// nothing reads it back.
//
// A trail's range of `ring`: [base] is its header (x: newest index, y:
// points held, z: last tick), [base + 1 + i] its points (xyz: position
// bits, w: tick), a ring of `cap`.

struct Job {
  head: vec4<f32>,     // xyz: the head this frame
  shift: vec4<f32>,    // xyz: added to every remembered point (origin.rs)
  base: u32,
  cap: u32,
  tick: u32,
  flags: u32,          // 1: start again (new, restyled, back in time)
  style: u32,
  out_first: u32,
  out_count: u32,
  pad: u32,
};

struct Style {
  life_ticks: u32,
  min_distance: f32,
  width_keys: u32,
  color_keys: u32,
  // 0: stretched over the length, else the length of one repeat
  tile: f32,
  textured: u32,
  pad0: u32,
  pad1: u32,
  width: array<vec4<f32>, 8>,    // x: time, y: width
  color_t: array<vec4<f32>, 2>,  // the colour keys' times, four a vector
  color: array<vec4<f32>, 8>,
};

struct StripPoint {
  position: vec3<f32>,
  width: f32,
  color: vec4<f32>,
  u: f32,
  flags: u32,          // 1: has a point before, 2: after, 4: textured
  pad0: u32,
  pad1: u32,
};

@group(0) @binding(0) var<storage, read> jobs: array<Job>;
@group(0) @binding(1) var<storage, read> styles: array<Style>;
@group(0) @binding(2) var<storage, read_write> ring: array<vec4<u32>>;
@group(0) @binding(3) var<storage, read_write> points: array<StripPoint>;
@group(0) @binding(4) var<uniform> count: vec4<u32>;  // x: jobs

fn ring_at(job: Job, head: u32, k: u32) -> u32 {
  // The k-th newest point.
  return job.base + 1u + (head + job.cap - k) % job.cap;
}

fn ring_pos(i: u32) -> vec3<f32> {
  return bitcast<vec3<f32>>(ring[i].xyz);
}

fn width_at(s: Style, t: f32) -> f32 {
  let n = min(s.width_keys, 8u);
  var v = s.width[0].y;
  for (var i = 1u; i < n; i++) {
    let t0 = s.width[i - 1u].x;
    let t1 = s.width[i].x;
    if (t >= t1) {
      v = s.width[i].y;
    } else if (t > t0) {
      v = mix(s.width[i - 1u].y, s.width[i].y, (t - t0) / max(t1 - t0, 1e-6));
      break;
    } else {
      break;
    }
  }
  return v;
}

fn color_time(s: Style, i: u32) -> f32 {
  return s.color_t[i / 4u][i % 4u];
}

fn color_at(s: Style, t: f32) -> vec4<f32> {
  let n = min(s.color_keys, 8u);
  var v = s.color[0];
  for (var i = 1u; i < n; i++) {
    let t0 = color_time(s, i - 1u);
    let t1 = color_time(s, i);
    if (t >= t1) {
      v = s.color[i];
    } else if (t > t0) {
      v = mix(s.color[i - 1u], s.color[i], (t - t0) / max(t1 - t0, 1e-6));
      break;
    } else {
      break;
    }
  }
  return v;
}

fn write_point(job: Job, s: Style, o: u32, p: vec3<f32>, along: f32, total: f32, flags: u32) {
  let f = along / total;
  var u = f;
  if (s.tile > 0.0) {
    u = along / s.tile;
  }
  points[job.out_first + o] = StripPoint(p, width_at(s, f), color_at(s, f), u, flags, 0u, 0u);
}

@compute @workgroup_size(64)
fn advance(@builtin(global_invocation_id) id: vec3<u32>) {
  if (id.x >= count.x) {
    return;
  }
  let job = jobs[id.x];
  let s = styles[job.style];
  var head = ring[job.base].x;
  var held = ring[job.base].y;
  var last = ring[job.base].z;
  if ((job.flags & 1u) != 0u) {
    head = 0u;
    held = 0u;
    last = job.tick;
  }
  // Carried across a move of the render origin.
  if (any(job.shift.xyz != vec3<f32>(0.0))) {
    for (var k = 0u; k < held; k++) {
      let i = ring_at(job, head, k);
      ring[i] = vec4<u32>(bitcast<vec3<u32>>(ring_pos(i) + job.shift.xyz), ring[i].w);
    }
  }
  let now = job.head.xyz;
  if (held == 0u) {
    head = 0u;
    ring[job.base + 1u] = vec4<u32>(bitcast<vec3<u32>>(now), job.tick);
    held = 1u;
  } else if (job.tick > last) {
    // One sample per elapsed tick along the path since the last frame.
    let start = ring_pos(ring_at(job, head, 0u));
    let steps = min(job.tick - last, max(s.life_ticks, 1u) + 1u);
    for (var n = 1u; n <= steps; n++) {
      let p = mix(start, now, f32(n) / f32(steps));
      let at = job.tick - (steps - n);
      if (distance(p, ring_pos(ring_at(job, head, 0u))) >= s.min_distance) {
        head = (head + 1u) % job.cap;
        ring[job.base + 1u + head] = vec4<u32>(bitcast<vec3<u32>>(p), at);
        held = min(held + 1u, job.cap);
      }
    }
  }
  last = job.tick;
  // Forgotten past the lifetime, oldest first.
  while (held > 0u && job.tick - ring[ring_at(job, head, held - 1u)].w > s.life_ticks) {
    held -= 1u;
  }
  ring[job.base] = vec4<u32>(head, held, last, 0u);

  // The strip: the live head, then the history newest to oldest, each
  // point further than 1e-5 from the one before.
  var total = 0.0;
  var kept = 1u;
  var prev = now;
  for (var k = 0u; k < held; k++) {
    let p = ring_pos(ring_at(job, head, k));
    let d = distance(p, prev);
    if (d > 1e-5) {
      total += d;
      kept += 1u;
      prev = p;
    }
  }
  var o = 0u;
  if (kept >= 2u) {
    let tex = select(0u, 4u, s.textured != 0u);
    let span = max(total, 1e-6);
    write_point(job, s, 0u, now, 0.0, span, tex | 2u);
    o = 1u;
    var along = 0.0;
    prev = now;
    for (var k = 0u; k < held; k++) {
      let p = ring_pos(ring_at(job, head, k));
      let d = distance(p, prev);
      if (d > 1e-5) {
        along += d;
        prev = p;
        let more = select(0u, 2u, o + 1u < kept);
        write_point(job, s, o, p, along, span, tex | 1u | more);
        o += 1u;
      }
    }
  }
  // The rest of the trail's range draws nothing.
  for (; o < job.out_count; o++) {
    points[job.out_first + o] = StripPoint(vec3<f32>(0.0), 0.0, vec4<f32>(0.0), 0.0, 0u, 0u, 0u);
  }
}
