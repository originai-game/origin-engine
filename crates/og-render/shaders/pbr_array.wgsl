// The array-map variant of the standard lit module (material.rs
// `array_lit_source`): appended to it after its base colour and normal
// textures became 2D arrays sampled at `map_layer` and `normal_layer`,
// VertexOut and PrepassOut gained the flat `layer`, and its `vs`, `fs`,
// `vs_prepass` and `fs_prepass` were renamed `*_standard`. These take
// their place, so the lit and prepass pipeline builders make the array
// pipelines from this module unchanged.
//
// An instance of an array-map material carries its layer plus one in its
// transform's m0.w (lib.rs `build_batches`; 0: the material's own
// layers), where an affine transform holds 0: it is read here and zeroed
// before the transform is used, so positions are the standard path's.

var<private> map_layer: i32;
var<private> normal_layer: i32;

// The layers a fragment samples: the instance's for both maps, or the
// material's mapLayer and normalMapLayer (MaterialFactors.coat.zw).
fn array_layers(instance_layer: u32) {
  let own = instance_layer > 0u;
  map_layer = select(i32(material.coat.z), i32(instance_layer) - 1, own);
  normal_layer = select(i32(material.coat.w), i32(instance_layer) - 1, own);
}

fn array_model(m0: vec4<f32>, m1: vec4<f32>, m2: vec4<f32>, m3: vec4<f32>) -> mat4x4<f32> {
  return mat4x4<f32>(vec4<f32>(m0.xyz, 0.0), m1, m2, m3);
}

fn array_instance_layer(m0: vec4<f32>) -> u32 {
  return u32(max(m0.w, 0.0) + 0.5);
}

@vertex
fn vs(v: VertexIn, i: InstanceIn) -> VertexOut {
  var o = lit_vertex(array_model(i.m0, i.m1, i.m2, i.m3), v);
  o.layer = array_instance_layer(i.m0);
  return o;
}

@fragment
fn fs(in: VertexOut) -> @location(0) vec4<f32> {
  array_layers(in.layer);
  return lit_fragment(in);
}

@vertex
fn vs_prepass(v: VertexIn, i: InstanceIn, prev: PrevIn) -> PrepassOut {
  var o = prepass_vertex(array_model(i.m0, i.m1, i.m2, i.m3), array_model(prev.p0, prev.p1, prev.p2, prev.p3), v);
  o.layer = array_instance_layer(i.m0);
  return o;
}

@fragment
fn fs_prepass(in: PrepassOut) -> PrepassTargets {
  array_layers(in.layer);
  return prepass_targets(in);
}
