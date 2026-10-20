// The array-map variant of the shadow module (material.rs
// `array_shadow_source`): appended after its `vs` and `vs_lean` were
// renamed `*_standard`. An array-map material's instance carries its layer
// beside its transform (instances.rs `GpuTransform::layer`), which a
// shadow does not read, so these are the standard stages: its depths are
// the standard path's.

@vertex
fn vs(v: VertexIn, i: InstanceIn) -> VsOut {
  return shadow_vertex(instance_matrix(i.m0, i.m1, i.m2, i.m3), v);
}

@vertex
fn vs_lean(@location(0) position: vec3<f32>, i: InstanceIn) -> LeanVsOut {
  return lean_caster(position, i);
}
