// Sky pass: the environment's sky cubemap drawn as the background. A
// fullscreen triangle at the far plane (depth 1, LessEqual, no depth write)
// after the opaque batches, so it only shades pixels no geometry covered.
// Prepended by frame.wgsl.

struct SkyOut { @builtin(position) pos: vec4<f32>, @location(0) ndc: vec2<f32> };

@vertex
fn vs_sky(@builtin(vertex_index) i: u32) -> SkyOut {
  var o: SkyOut;
  let x = f32(i32(i & 1u) * 4 - 1);
  let y = f32(i32(i & 2u) * 2 - 1);
  o.pos = vec4<f32>(x, y, 1.0, 1.0);
  o.ndc = vec2<f32>(x, y);
  return o;
}

@fragment
fn fs_sky(in: SkyOut) -> @location(0) vec4<f32> {
  // The view ray through the pixel: near point to far point (perspective and orthographic).
  let near = frame.inv_view_proj * vec4<f32>(in.ndc, 0.0, 1.0);
  let far = frame.inv_view_proj * vec4<f32>(in.ndc, 1.0, 1.0);
  let dir = normalize(far.xyz / far.w - near.xyz / near.w);
  return vec4<f32>(textureSampleLevel(sky_map, ibl_sampler, dir, 0.0).rgb, 1.0);
}
