// The scene's colliders as scene_colliders.rs `pack` writes them, read
// from `colliders` (an array<vec4<f32>> the including shader binds):
// colliders[0].x the records, colliders[0].y the first word after them;
// each record 8 vec4s from colliders[1]; a list (a word offset after the
// records, 0xffffffff for none) is a count then record indices; height
// samples follow. `push_out` moves a point (a sphere of `radius`) out of
// a list's colliders. Shared with the GPU visual-body solver.

struct Hit { p: vec3<f32>, n: vec3<f32>, hit: bool }

fn collider_record(r: u32, k: u32) -> vec4<f32> { return colliders[1u + r * 8u + k]; }

fn collider_word(w: u32) -> f32 { return colliders[w / 4u][w % 4u]; }

fn collider_unit(v: vec3<f32>) -> vec3<f32> {
  let l = length(v);
  if (l > 1e-12) { return v / l; }
  return vec3<f32>(0.0, 1.0, 0.0);
}

fn closest_on_segment(p: vec3<f32>, a: vec3<f32>, b: vec3<f32>) -> vec3<f32> {
  let ab = b - a;
  let t = clamp(dot(p - a, ab) / max(dot(ab, ab), 1e-12), 0.0, 1.0);
  return a + ab * t;
}

fn push_round(p: vec3<f32>, c: vec3<f32>, reach: f32) -> Hit {
  let d = p - c;
  let len = length(d);
  if (len >= reach) { return Hit(p, vec3<f32>(0.0), false); }
  var n = vec3<f32>(0.0, 1.0, 0.0);
  if (len > 1e-9) { n = d / len; }
  return Hit(c + n * reach, n, true);
}

fn push_out(p_in: vec3<f32>, radius: f32, list: u32) -> Hit {
  var out = Hit(p_in, vec3<f32>(0.0), false);
  if (list == 0xffffffffu) { return out; }
  let base = bitcast<u32>(colliders[0].y);
  let words = base + list;
  let count = bitcast<u32>(collider_word(words));
  for (var k = 0u; k < count; k++) {
    let r = bitcast<u32>(collider_word(words + 1u + k));
    let head = collider_record(r, 0u);
    let kind = bitcast<u32>(head.x);
    var h = Hit(out.p, vec3<f32>(0.0), false);
    if (kind == 0u) {
      // Plane: solid below dot(n, p) = offset.
      let pl = collider_record(r, 1u);
      let depth = dot(pl.xyz, out.p) - pl.w;
      if (depth < radius) { h = Hit(out.p + pl.xyz * (radius - depth), pl.xyz, true); }
    } else if (kind == 1u) {
      let s = collider_record(r, 1u);
      h = push_round(out.p, s.xyz, s.w + radius);
    } else if (kind == 2u) {
      let a = collider_record(r, 1u);
      let b = collider_record(r, 2u);
      h = push_round(out.p, closest_on_segment(out.p, a.xyz, b.xyz), a.w + radius);
    } else if (kind == 3u) {
      // Oriented box: centre, unit axes, half extents (w).
      let c = collider_record(r, 1u).xyz;
      let ax = collider_record(r, 2u);
      let ay = collider_record(r, 3u);
      let az = collider_record(r, 4u);
      let half = vec3<f32>(ax.w, ay.w, az.w);
      let d = out.p - c;
      let q = vec3<f32>(dot(d, ax.xyz), dot(d, ay.xyz), dot(d, az.xyz));
      let outside = abs(q) - half;
      if (all(outside < vec3<f32>(0.0))) {
        // Inside: out through the nearest face.
        var axis = 0u;
        var least = outside.x;
        if (outside.y > least) { least = outside.y; axis = 1u; }
        if (outside.z > least) { least = outside.z; axis = 2u; }
        var l = q;
        var nl = vec3<f32>(0.0);
        let s = select(-1.0, 1.0, q[axis] >= 0.0);
        l[axis] = s * (half[axis] + radius);
        nl[axis] = s;
        let n = ax.xyz * nl.x + ay.xyz * nl.y + az.xyz * nl.z;
        h = Hit(c + ax.xyz * l.x + ay.xyz * l.y + az.xyz * l.z, n, true);
      } else {
        let cl = clamp(q, -half, half);
        let near = c + ax.xyz * cl.x + ay.xyz * cl.y + az.xyz * cl.z;
        h = push_round(out.p, near, radius);
      }
    } else if (kind == 4u) {
      // Height field patch: world to the field's local space (rows 1-3),
      // local to world (rows 4-6), then (origin x, origin z, cell,
      // samples per row and rows) and the heights after the records.
      let m0 = collider_record(r, 1u); let m1 = collider_record(r, 2u); let m2 = collider_record(r, 3u);
      let f0 = collider_record(r, 4u); let f1 = collider_record(r, 5u); let f2 = collider_record(r, 6u);
      let info = collider_record(r, 7u);
      let p4 = vec4<f32>(out.p, 1.0);
      let l = vec3<f32>(dot(m0, p4), dot(m1, p4), dot(m2, p4));
      let side = bitcast<u32>(info.w) & 0xffffu;
      let rows_n = bitcast<u32>(info.w) >> 16u;
      let g = (vec2<f32>(l.x, l.z) - info.xy) / info.z;
      if (g.x >= 0.0 && g.y >= 0.0 && g.x <= f32(side - 1u) && g.y <= f32(rows_n - 1u)) {
        let cell = min(floor(g), vec2<f32>(f32(side - 2u), f32(rows_n - 2u)));
        let f = g - cell;
        let first = bitcast<u32>(head.y) + u32(cell.y) * side + u32(cell.x);
        let h00 = collider_word(first);
        let h10 = collider_word(first + 1u);
        let h01 = collider_word(first + side);
        let h11 = collider_word(first + side + 1u);
        let height = mix(mix(h00, h10, f.x), mix(h01, h11, f.x), f.y);
        let local_radius = radius * head.z;
        if (l.y < height + local_radius) {
          let dx = mix(h10 - h00, h11 - h01, f.y) / info.z;
          let dz = mix(h01 - h00, h11 - h10, f.x) / info.z;
          let nl = collider_unit(vec3<f32>(-dx, 1.0, -dz));
          let lp = vec4<f32>(l.x, height + local_radius, l.z, 1.0);
          let wp = vec3<f32>(dot(f0, lp), dot(f1, lp), dot(f2, lp));
          // Normals: the inverse transpose of local to world is world to
          // local's transpose.
          let n = collider_unit(vec3<f32>(m0.x, m1.x, m2.x) * nl.x + vec3<f32>(m0.y, m1.y, m2.y) * nl.y + vec3<f32>(m0.z, m1.z, m2.z) * nl.z);
          h = Hit(wp, n, true);
        }
      }
    }
    if (h.hit) {
      out = Hit(h.p, h.n, true);
    }
  }
  return out;
}
