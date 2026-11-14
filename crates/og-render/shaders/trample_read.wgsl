// Reading the trample map (trample.rs, grass_trample.wgsl): which four
// cells lie about a point and how they blend, apart from the loads, so a
// reader binds the map as its pipeline can (a texture where a stage has a
// slot, read-only storage where its sixteen are taken: terrain_lit.wgsl).

// The four cells about `at` (x, z from the frame's origin) in the map of
// `grid` (its min corner, cell and cells a side; 0: no map): the low and
// high cell, the point's share between them, and whether the map holds it.
struct TrampleTap {
  i: vec2<i32>,
  j: vec2<i32>,
  f: vec2<f32>,
  ok: bool,
};

fn trample_tap(grid: vec4<f32>, at: vec2<f32>) -> TrampleTap {
  var t: TrampleTap;
  t.ok = false;
  if (grid.w < 1.0) { return t; }
  let q = (at - grid.xy) / grid.z - 0.5;
  let n = grid.w;
  if (any(q < vec2<f32>(0.0)) || any(q > vec2<f32>(n - 1.0))) { return t; }
  // Bilinear, so a pusher's edge is smooth at the cells' size.
  t.i = vec2<i32>(floor(q));
  t.f = q - floor(q);
  t.j = min(t.i + vec2<i32>(1), vec2<i32>(i32(n) - 1));
  t.ok = true;
  return t;
}

// How the four cells of a tap (at i, (j.x, i.y), (i.x, j.y) and j) press
// what reaches up to `top` (y from the frame's origin), recovering over
// `recovery` seconds: xy the push away from the pusher (a share of the
// pressed thing's height, towards 1 flattened), z the crush (0..1, how far
// it is pressed down). What is taller than a pusher's bottom is pressed;
// what lies under a pusher passing above is not.
fn trample_blend(s00: vec4<f32>, s10: vec4<f32>, s01: vec4<f32>, s11: vec4<f32>, f: vec2<f32>, top: f32, recovery: f32) -> vec3<f32> {
  var push = vec2<f32>(0.0);
  var crush = 0.0;
  let w = array<f32, 4>((1.0 - f.x) * (1.0 - f.y), f.x * (1.0 - f.y), (1.0 - f.x) * f.y, f.x * f.y);
  let ss = array<vec4<f32>, 4>(s00, s10, s01, s11);
  for (var k = 0; k < 4; k++) {
    let c = ss[k];
    let amount = length(c.xy);
    if (amount <= 0.0 || top < c.z) { continue; }
    // Recovery: easing back up over `recovery` seconds since the press.
    let back = smoothstep(0.0, max(recovery, 1e-3), c.w);
    let a = amount * (1.0 - back) * w[k];
    push += c.xy / amount * a;
    crush += a;
  }
  return vec3<f32>(push, crush);
}

// How the map presses a blade or a scattered copy of height `tall` rooted
// at `root` (frame coordinates), read from a texture.
fn vegetation_trample(tex: texture_2d<f32>, grid: vec4<f32>, root: vec3<f32>, tall: f32, recovery: f32) -> vec3<f32> {
  let t = trample_tap(grid, root.xz);
  if (!t.ok) { return vec3<f32>(0.0); }
  return trample_blend(
    textureLoad(tex, t.i, 0),
    textureLoad(tex, vec2<i32>(t.j.x, t.i.y), 0),
    textureLoad(tex, vec2<i32>(t.i.x, t.j.y), 0),
    textureLoad(tex, t.j, 0),
    t.f, root.y + tall, recovery);
}
