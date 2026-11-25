// The array-map variant of the shadow module (material.rs
// `array_shadow_source`): appended after its `vs`, `vs_lean` and `fs_dissolve`
// were renamed `*_standard`.

var<private> map_layer: u32;
var<private> normal_layer: u32;

@vertex
fn vs(v: VertexIn, i: InstanceIn, @location(8) look: vec4<f32>) -> VsOut {
  var o = shadow_vertex(instance_matrix(i.m0, i.m1, i.m2, i.m3), v);
  o.hidden = look_hidden(look);
  o.layer = u32(max(look_layer(look), 0.0));
  return o;
}

@vertex
fn vs_lean(@location(0) position: vec3<f32>, i: InstanceIn) -> LeanVsOut {
  return lean_caster(position, i);
}

@fragment
fn fs_dissolve(in: VsOut, @builtin(front_facing) front: bool) -> @location(0) vec4<f32> {
  map_layer = select(u32(material.uv[1].z), in.layer - 1u, in.layer > 0u);
  return shadow_dissolve(in, front);
}
