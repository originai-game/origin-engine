// The array-map variant of the standard lit module (material.rs
// `array_lit_source`): appended to it after its base colour and normal
// textures became 2D arrays sampled at `map_layer` and `normal_layer`,
// VertexOut and PrepassOut gained the flat `layer`, and its `vs`, `fs`,
// `vs_prepass` and `fs_prepass` were renamed `*_standard`. These take
// their place, so the lit and prepass pipeline builders make the array
// pipelines from this module unchanged.
//
// An instance of an array-map material carries its layer plus one beside
// its transform in the instance stream (instances.rs `GpuTransform::layer`,
// pbr.wgsl `InstanceFxIn` x, read by `look_layer`; 0: the material's own
// layers), so positions are the standard path's.

var<private> map_layer: i32;
var<private> normal_layer: i32;

// The layers a fragment samples: the instance's for both maps, or the
// material's mapLayer and normalMapLayer (MaterialFactors.coat.zw).
fn array_layers(instance_layer: u32) {
  let own = instance_layer > 0u;
  map_layer = select(i32(material.coat.z), i32(instance_layer) - 1, own);
  normal_layer = select(i32(material.coat.w), i32(instance_layer) - 1, own);
}

// The instance's layer plus one (0: the material's).
fn array_instance_layer(a: InstanceFxIn) -> u32 {
  return u32(max(look_layer(a.look), 0.0) + 0.5);
}

@vertex
fn vs(v: VertexIn, i: InstanceIn, a: InstanceFxIn) -> VertexOut {
  var o = lit_vertex(instance_matrix(i.m0, i.m1, i.m2, i.m3), v);
  o.layer = array_instance_layer(a);
  o.look = a.look;
  return o;
}

@fragment
fn fs(in: VertexOut, @builtin(front_facing) front_facing: bool) -> @location(0) vec4<f32> {
  hook_front = select(-1.0, 1.0, front_facing);
  array_layers(in.layer);
  return lit_fragment(in);
}

@vertex
fn vs_prepass(v: VertexIn, i: InstanceIn, prev: PrevIn, a: InstanceFxIn) -> PrepassOut {
  var o = prepass_vertex(instance_matrix(i.m0, i.m1, i.m2, i.m3), instance_matrix(prev.p0, prev.p1, prev.p2, prev.p3), v);
  o.layer = array_instance_layer(a);
  return o;
}

@fragment
fn fs_prepass(in: PrepassOut, @builtin(front_facing) front_facing: bool) -> PrepassTargets {
  hook_front = select(-1.0, 1.0, front_facing);
  array_layers(in.layer);
  return prepass_targets(in);
}
