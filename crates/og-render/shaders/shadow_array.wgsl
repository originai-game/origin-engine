// The array-map variant of the shadow module (material.rs
// `array_shadow_source`): appended after its `vs` and `vs_lean` were
// renamed `*_standard`. An array-map material's instance carries its layer
// in m0.w (pbr_array.wgsl), zeroed here before the transform is used, so
// its depths are the standard path's.

@vertex
fn vs(v: VertexIn, i: InstanceIn) -> VsOut {
  return shadow_vertex(mat4x4<f32>(vec4<f32>(i.m0.xyz, 0.0), i.m1, i.m2, i.m3), v);
}

@vertex
fn vs_lean(@location(0) position: vec3<f32>, i: InstanceIn) -> LeanVsOut {
  let world = mat4x4<f32>(vec4<f32>(i.m0.xyz, 0.0), i.m1, i.m2, i.m3) * vec4<f32>(position, 1.0);
  var o: LeanVsOut;
  o.clip = shadow_clip(world);
  o.world = world.xyz;
  return o;
}
