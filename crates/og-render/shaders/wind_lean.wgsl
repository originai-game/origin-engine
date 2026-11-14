// What every vegetation tier reads to move: the simulated wind field
// (wind_field.rs: grass, scattered copies and their cards, meshlet trees).
// Follows wind_common.wgsl; the textures are the caller's bindings, passed
// in. (The trample map's read is trample_read.wgsl.)

// One level of the simulated field, `rows` its first row in the texture:
// the lean (xy now, zw last drawn) bilinear at `rel` (x, z from the
// frame's origin) and how far inside the level it is (1 well inside, 0
// at and past its edge, over its outer eighth).
fn wind_field_level(tex: texture_2d<f32>, corner: vec2<f32>, cell: f32, n: f32, rows: f32, rel: vec2<f32>) -> vec4<f32> {
  let q = (rel - corner) / cell - 0.5;
  let inside = min(min(q.x, q.y), min(n - 1.0 - q.x, n - 1.0 - q.y)) / (0.125 * n);
  let c = clamp(q, vec2<f32>(0.0), vec2<f32>(n - 1.001));
  let i = vec2<i32>(floor(c));
  let f = c - floor(c);
  let r = i32(rows);
  let a = textureLoad(tex, vec2<i32>(i.x, r + i.y), 0);
  let b = textureLoad(tex, vec2<i32>(i.x + 1, r + i.y), 0);
  let d = textureLoad(tex, vec2<i32>(i.x, r + i.y + 1), 0);
  let e = textureLoad(tex, vec2<i32>(i.x + 1, r + i.y + 1), 0);
  return mix(mix(a, b, f.x), mix(d, e, f.x), f.y) * clamp(inside, 0.0, 1.0);
}

// The lean at `rel` (x, z from the frame's origin), now (xy) and as last
// drawn (zw), from the simulated field (wind_field.rs): its fine level
// near the camera, its coarse one beyond, and og.Wind's analytic lean past
// both (or everywhere when the field does not run: no compute).
fn wind_lean(tex: texture_2d<f32>, w: WindField, rel: vec2<f32>) -> vec4<f32> {
  let at = rel + w.time.zw;
  let analytic = vec4<f32>(wind_lean_analytic(w, at, false), wind_lean_analytic(w, at, true));
  if (w.grid.w < 0.5) { return analytic; }
  let n = w.grid_outer.w;
  let fine = wind_field_level(tex, w.grid.xy, w.grid.z, n, 0.0, rel);
  let coarse = wind_field_level(tex, w.grid_outer.xy, w.grid_outer.z, n, n, rel);
  // The levels' weights (inside() scaled their values): the fine where it
  // reaches, the coarse past it, the analytic past both.
  let wf = clamp(wind_field_weight(w.grid.xy, w.grid.z, n, rel), 0.0, 1.0);
  let wc = clamp(wind_field_weight(w.grid_outer.xy, w.grid_outer.z, n, rel), 0.0, 1.0);
  let far = coarse + analytic * (1.0 - wc);
  return fine + far * (1.0 - wf);
}

// wind_field_level's weight alone.
fn wind_field_weight(corner: vec2<f32>, cell: f32, n: f32, rel: vec2<f32>) -> f32 {
  let q = (rel - corner) / cell - 0.5;
  return min(min(q.x, q.y), min(n - 1.0 - q.x, n - 1.0 - q.y)) / (0.125 * n);
}
