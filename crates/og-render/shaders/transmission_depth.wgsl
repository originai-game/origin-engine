// Start the on-demand volume buffer with opaque linear depth. The exit
// pass writes RGB only, preserving this depth in A without another
// texture slot in the physical/lightmapped shaders. Read the depth the
// opaque main pass already stored, not a duplicate geometry prepass.
// volume.rs prepends opaque_depth_at for single-sample or MSAA depth.

@vertex
fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  return fullscreen_clip(i);
}

@fragment
fn fs(@builtin(position) p: vec4<f32>) -> @location(0) vec4<u32> {
  let z = opaque_depth_at(vec2<i32>(p.xy));
  var depth = 1e8;
  if (z > 0.0) {
    let uv = p.xy / vec2<f32>(textureDimensions(opaque_depth));
    let ndc = (uv * 2.0 - 1.0) * vec2<f32>(1.0, -1.0);
    // Along the pixel's ray from its Reverse-Z near point (z_ndc = 1.0):
    // at t along the ray, clip_z = 1/near.w + t * along.z and
    // clip_w = 1/near.w + t * along.w, so z = clip_z / clip_w solves to
    // t = (z - 1.0) / (near.w * (along.z - z * along.w)).
    let near = frame.inv_view_proj * vec4<f32>(ndc, 1.0, 1.0);
    let dir = pixel_ray(ndc);
    let along = frame.view_proj * vec4<f32>(dir, 0.0);
    let t = (z - 1.0) / (near.w * (along.z - z * along.w));
    depth = dot(near.xyz / near.w - frame.view_pos.xyz, frame.view_forward.xyz) + t * dot(dir, frame.view_forward.xyz);
  }
  return vec4<u32>(0u, 0u, 0u, bitcast<u32>(depth));
}
