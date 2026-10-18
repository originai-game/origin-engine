// Trails, ribbons and beams: camera-facing strips built on the CPU,
// blended into the HDR target after the opaque scene (depth tested, no
// depth write). u runs along the strip, v across it. Untextured strips get a
// soft edge across v. Prepended by frame.wgsl.

@group(1) @binding(0) var ribbon_tex: texture_2d<f32>;
@group(1) @binding(1) var ribbon_sampler: sampler;

struct RibbonIn {
  @location(0) position_v: vec4<f32>,
  @location(1) color: vec4<f32>,
  @location(2) extra: vec4<f32>,  // x: u, y: 1 when textured
};

struct RibbonOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) uv: vec2<f32>,
  @location(1) color: vec4<f32>,
  @location(2) textured: f32,
};

@vertex
fn vs(r: RibbonIn) -> RibbonOut {
  var o: RibbonOut;
  o.clip = frame.view_proj * vec4<f32>(r.position_v.xyz, 1.0);
  o.uv = vec2<f32>(r.extra.x, r.position_v.w);
  o.color = r.color;
  o.textured = r.extra.y;
  return o;
}

@fragment
fn fs(in: RibbonOut) -> @location(0) vec4<f32> {
  let tex = textureSample(ribbon_tex, ribbon_sampler, in.uv);
  var c = in.color;
  if (in.textured > 0.5) {
    c *= tex;
  } else {
    let e = 1.0 - abs(in.uv.y * 2.0 - 1.0);
    c.a *= e * e * (3.0 - 2.0 * e);
  }
  if (c.a <= 0.0) { discard; }
  return c;
}
