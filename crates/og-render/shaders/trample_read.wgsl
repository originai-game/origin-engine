// How the trample map (trample.rs, grass_trample.wgsl) presses a blade or
// a scattered copy of height `tall` rooted at `root` (frame coordinates), `grid` the map's min
// corner (x, z from the frame's origin), cell and cells a side (0: no
// map), recovering over `recovery` seconds: xy the push away from the
// pusher (a share of the blade's height, towards 1 flattened), z the crush
// (0..1, how far the blade is pressed down). A blade taller than a
// pusher's bottom is pressed; one under a pusher passing above is not.
fn vegetation_trample(tex: texture_2d<f32>, grid: vec4<f32>, root: vec3<f32>, tall: f32, recovery: f32) -> vec3<f32> {
  if (grid.w < 1.0) { return vec3<f32>(0.0); }
  let q = (root.xz - grid.xy) / grid.z - 0.5;
  let n = grid.w;
  if (any(q < vec2<f32>(0.0)) || any(q > vec2<f32>(n - 1.0))) { return vec3<f32>(0.0); }
  // Bilinear, so a pusher's edge is smooth at the cells' size.
  let i = vec2<i32>(floor(q));
  let f = q - floor(q);
  let j = min(i + vec2<i32>(1), vec2<i32>(i32(n) - 1));
  let s00 = textureLoad(tex, i, 0);
  let s10 = textureLoad(tex, vec2<i32>(j.x, i.y), 0);
  let s01 = textureLoad(tex, vec2<i32>(i.x, j.y), 0);
  let s11 = textureLoad(tex, j, 0);
  var push = vec2<f32>(0.0);
  var crush = 0.0;
  let w = array<f32, 4>((1.0 - f.x) * (1.0 - f.y), f.x * (1.0 - f.y), (1.0 - f.x) * f.y, f.x * f.y);
  let ss = array<vec4<f32>, 4>(s00, s10, s01, s11);
  for (var k = 0; k < 4; k++) {
    let c = ss[k];
    let amount = length(c.xy);
    if (amount <= 0.0 || root.y + tall < c.z) { continue; }
    // Recovery: easing back up over `recovery` seconds since the press.
    let back = smoothstep(0.0, max(recovery, 1e-3), c.w);
    let a = amount * (1.0 - back) * w[k];
    push += c.xy / amount * a;
    crush += a;
  }
  return vec3<f32>(push, crush);
}
