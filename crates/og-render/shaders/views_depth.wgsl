// The depth resolve of a view (views.wgsl): its depth buffer as the linear
// view depth from near (0) to far (1) in rgb. Prepended by `alias
// ViewDepth = ...;` (single or multisampled).
@group(0) @binding(0) var depth_tex: ViewDepth;

@fragment
fn fs_depth(in: VsOut) -> @location(0) vec4<f32> {
  let d = textureLoad(depth_tex, texel(in.pos), 0);
  let n = r.depth.x;
  let f = r.depth.y;
  var z = d;
  if (r.depth.z < 0.5) {
    // Perspective (depth 0..1, right-handed): the eye distance, normalised.
    let e = n * f / max(f - d * (f - n), 1e-6);
    z = (e - n) / max(f - n, 1e-6);
  }
  let v = clamp(z, 0.0, 1.0);
  return vec4<f32>(v, v, v, 1.0);
}
