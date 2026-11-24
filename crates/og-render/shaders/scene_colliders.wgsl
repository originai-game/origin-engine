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
      let cell_size = vec2<f32>(info.z, head.z);
      let g = (vec2<f32>(l.x, l.z) - info.xy) / max(cell_size, vec2<f32>(1e-6));
      if (side >= 2u && rows_n >= 2u && info.z > 0.0 && g.x >= 0.0 && g.y >= 0.0 && g.x <= f32(side - 1u) && g.y <= f32(rows_n - 1u)) {
        let cell = min(floor(g), vec2<f32>(f32(side - 2u), f32(rows_n - 2u)));
        let hole = bitcast<u32>(head.w) + u32(cell.y) * side + u32(cell.x);
        if collider_word(hole) > 0.5 { continue; }
        let f = g - cell;
        let first = bitcast<u32>(head.y) + u32(cell.y) * side + u32(cell.x);
        let h00 = collider_word(first);
        let h10 = collider_word(first + 1u);
        let h01 = collider_word(first + side);
        let h11 = collider_word(first + side + 1u);
        // The terrain's own diagonal rule and piecewise-planar surface.
        var slope = vec2<f32>(h10 - h00, h01 - h00);
        var height = h00 + dot(slope, f);
        if abs(h00 - h11) <= abs(h10 - h01) {
          if f.y >= f.x {
            slope = vec2<f32>(h11 - h01, h01 - h00);
          } else {
            slope = vec2<f32>(h10 - h00, h11 - h10);
          }
          height = h00 + dot(slope, f);
        } else if f.x + f.y > 1.0 {
          slope = vec2<f32>(h11 - h01, h11 - h10);
          height = h11 - dot(slope, 1.0 - f);
        }
        {
          let dx = slope.x / cell_size.x;
          let dz = slope.y / cell_size.y;
          let nl = collider_unit(vec3<f32>(-dx, 1.0, -dz));
          let lp = vec4<f32>(l.x, height, l.z, 1.0);
          let wp = vec3<f32>(dot(f0, lp), dot(f1, lp), dot(f2, lp));
          // Normals: the inverse transpose of local to world is world to
          // local's transpose.
          let n = collider_unit(m0.xyz * nl.x + m1.xyz * nl.y + m2.xyz * nl.z);
          let depth = dot(out.p - wp, n);
          if depth < radius { h = Hit(out.p + n * (radius - depth), n, true); }
        }
      }
    } else if kind == 5u || kind == 6u {
      let m0 = collider_record(r, 1u); let m1 = collider_record(r, 2u); let m2 = collider_record(r, 3u);
      let p4 = vec4<f32>(out.p, 1.0);
      let l = vec3<f32>(dot(m0, p4), dot(m1, p4), dot(m2, p4));
      var distance = -1e30;
      var normal = vec3<f32>(0.0, 1.0, 0.0);
      if kind == 5u {
        let shape = collider_record(r, 4u);
        let radial = length(l.xz);
        let q = vec2<f32>(radial - shape.x, abs(l.y) - shape.y);
        let radial_n = vec3<f32>(select(vec2<f32>(1.0, 0.0), l.xz / max(radial, 1e-12), radial > 1e-12).x, 0.0,
          select(vec2<f32>(1.0, 0.0), l.xz / max(radial, 1e-12), radial > 1e-12).y);
        let cap_n = vec3<f32>(0.0, select(-1.0, 1.0, l.y >= 0.0), 0.0);
        let outside = max(q, vec2<f32>(0.0));
        distance = length(outside) + min(max(q.x, q.y), 0.0);
        normal = select(cap_n, radial_n, q.x > q.y);
        if all(q > vec2<f32>(0.0)) { normal = normalize(radial_n * q.x + cap_n * q.y); }
        let wn = m0.xyz * normal.x + m1.xyz * normal.y + m2.xyz * normal.z;
        distance /= max(length(wn), 1e-12);
        normal = collider_unit(wn);
      } else {
        for (var j = 0u; j < bitcast<u32>(head.z); j++) {
          let at = bitcast<u32>(head.y) + j * 4u;
          let plane = vec4<f32>(collider_word(at), collider_word(at + 1u), collider_word(at + 2u), collider_word(at + 3u));
          let wn = m0.xyz * plane.x + m1.xyz * plane.y + m2.xyz * plane.z;
          let d = (dot(plane.xyz, l) - plane.w) / max(length(wn), 1e-12);
          if d > distance { distance = d; normal = collider_unit(wn); }
        }
      }
      if distance < radius { h = Hit(out.p + normal * (radius - distance), normal, true); }
    }
    if (h.hit) {
      out = Hit(h.p, h.n, true);
    }
  }
  return out;
}
