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
  if (z < 1.0) {
    let uv = p.xy / vec2<f32>(textureDimensions(opaque_depth));
    let clip = vec4<f32>((uv * 2.0 - 1.0) * vec2<f32>(1.0, -1.0), z, 1.0);
    let world = frame.inv_view_proj * clip;
    depth = dot(world.xyz / world.w - frame.view_pos.xyz, frame.view_forward.xyz);
  }
  return vec4<u32>(0u, 0u, 0u, bitcast<u32>(depth));
}
