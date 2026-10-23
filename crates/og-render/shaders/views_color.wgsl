// The colour resolves of a view (views.wgsl).
@group(0) @binding(0) var source: texture_2d<f32>;

@fragment
fn fs_color(in: VsOut) -> @location(0) vec4<f32> {
  let c = textureLoad(source, texel(in.pos), 0).rgb * r.params.x;
  var t: vec3<f32>;
  if (r.params.y > 1.5) { t = agx(c); } else if (r.params.y > 0.5) { t = clamp(c, vec3<f32>(0.0), vec3<f32>(1.0)); } else { t = aces(c); }
  if (r.params.w > 0.5) { t = srgb_encode(t); }
  return vec4<f32>(t, 1.0);
}

@fragment
fn fs_linear(in: VsOut) -> @location(0) vec4<f32> {
  return vec4<f32>(textureLoad(source, texel(in.pos), 0).rgb, 1.0);
}
