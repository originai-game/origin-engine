// The depth resolve of a view (views.wgsl): its depth prepass's linear
// view depth (world units along its forward axis, the main prepass's
// `lindepth`) normalised from near (0) to far (1) in rgb. A colour target,
// so every tier loads it (GLSL ES 3.0 cannot load a depth buffer).
@group(0) @binding(0) var depth_tex: texture_2d<f32>;

@fragment
fn fs_depth(in: VsOut) -> @location(0) vec4<f32> {
  let e = textureLoad(depth_tex, texel(in.pos), 0).r;
  let n = r.depth.x;
  let f = r.depth.y;
  let v = clamp((e - n) / max(f - n, 1e-6), 0.0, 1.0);
  return vec4<f32>(v, v, v, 1.0);
}
