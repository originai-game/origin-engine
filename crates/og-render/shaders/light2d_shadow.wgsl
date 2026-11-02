// 2D shadows: one row per shadow-casting light of an RG32F polar map,
// `angles` texels wide. Texel c of row r is the ray from light r's centre
// at angle 2π (c + ½) / angles, traced against that light's occluder
// edges (the CPU keeps those on its layers that reach its radius and
// drops an occluder holding the light): x is the distance to the nearest
// occluder, y the distance where the ray leaves that occluder, so a pixel
// between them (inside the occluder) is not shadowed by it, and one past
// y is. A ray that meets nothing holds FAR in both.

const FAR: f32 = 1e20;
const NONE: u32 = 0xffffffffu;

struct Shadows {
  info: vec4<u32>,                  // x: angles, y: rows
  rows: array<vec4<f32>, 64>,       // per row: the light's centre, radius
  ranges: array<vec4<u32>, 64>,     // per row: its first edge and how many
};
struct Edge {
  ab: vec4<f32>,                    // the segment's ends
  id: vec4<u32>,                    // x: its occluder
};
@group(0) @binding(0) var<uniform> shadows: Shadows;
@group(0) @binding(1) var<storage, read> edges: array<Edge>;

@vertex
fn vs(@builtin(vertex_index) vi: u32) -> @builtin(position) vec4<f32> {
  let p = vec2<f32>(f32((vi << 1u) & 2u), f32(vi & 2u));
  return vec4<f32>(p * 2.0 - 1.0, 0.0, 1.0);
}

fn cross2(a: vec2<f32>, b: vec2<f32>) -> f32 {
  return a.x * b.y - a.y * b.x;
}

// How far along the unit ray (o, d) it meets the segment ab, or FAR.
fn hit(o: vec2<f32>, d: vec2<f32>, ab: vec4<f32>) -> f32 {
  let s = ab.zw - ab.xy;
  let den = cross2(d, s);
  if (abs(den) < 1e-12) { return FAR; }
  let ao = ab.xy - o;
  let t = cross2(ao, s) / den;
  let u = cross2(ao, d) / den;
  if (t >= 0.0 && u >= 0.0 && u <= 1.0) { return t; }
  return FAR;
}

@fragment
fn fs(@builtin(position) pos: vec4<f32>) -> @location(0) vec2<f32> {
  let col = u32(pos.x);
  let row = min(u32(pos.y), 63u);
  let theta = (f32(col) + 0.5) / f32(max(shadows.info.x, 1u)) * 6.283185307179586;
  let d = vec2<f32>(cos(theta), sin(theta));
  let o = shadows.rows[row].xy;
  let range = shadows.ranges[row];
  var near = FAR;
  var id = NONE;
  for (var i = range.x; i < range.x + range.y; i++) {
    let t = hit(o, d, edges[i].ab);
    if (t < near) {
      near = t;
      id = edges[i].id.x;
    }
  }
  if (id == NONE) { return vec2<f32>(FAR, FAR); }
  // The first place past `near` where the ray crosses that occluder's
  // outline again: where it leaves it (a concave one may be entered again
  // further on; that span shadows as any occluder does).
  var exit = FAR;
  for (var i = range.x; i < range.x + range.y; i++) {
    if (edges[i].id.x != id) { continue; }
    let t = hit(o, d, edges[i].ab);
    if (t > near + 1e-4 && t < exit) { exit = t; }
  }
  return vec2<f32>(near, select(exit, near, exit >= FAR));
}
