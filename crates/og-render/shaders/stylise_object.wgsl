// The object pass of Stylise's object space (stylise_object.rs): after the
// depth-normal prepass, the opaque mesh instances are drawn again against
// the prepass's depth (no depth write, a few units of bias towards the
// camera, so only the visible surface passes) into an Rgba32Uint target:
// xyz the bits of the pixel's position in its instance's own space, w its
// normal there (two 15-bit octahedral components) with bit 0 set, so 0 is
// "no instance" (the sky, terrain, scattered copies). The stylise passes
// lay the canvas, the hatching and the halftone at that position, so a
// moving object carries its pattern.
//
// The instance's space is in metres: the mesh position times the length
// of each of the model's axes (a box scaled 10 x 1 x 1 keeps square
// strokes), the normal through the inverse of that scale. Vertices are
// placed by lit_place, as every lit and prepass vertex stage, so they land
// where the prepass put them. Appended to the lit shader's source
// (lit_source_with with the built-in hooks).

struct ObjectOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) object_pos: vec3<f32>,
  @location(1) object_normal: vec3<f32>,
};

// The length of each of the model's axes: metres per mesh unit.
fn axis_scale(model: mat4x4<f32>) -> vec3<f32> {
  return max(vec3<f32>(length(model[0].xyz), length(model[1].xyz), length(model[2].xyz)), vec3<f32>(1e-6));
}

fn object_vertex(model: mat4x4<f32>, scale: vec3<f32>, v: VertexIn) -> ObjectOut {
  let p = lit_place(model, v.position, v.normal, v.uv, v.color);
  var o: ObjectOut;
  o.clip = p.clip;
  o.object_pos = v.position * scale;
  o.object_normal = v.normal / scale;
  return o;
}

@vertex
fn vs_object(v: VertexIn, i: InstanceIn) -> ObjectOut {
  // An array-map material's instance carries its layer in m0.w, 0 in an
  // affine transform (pbr_array.wgsl): zeroed, the transform is the same.
  let model = mat4x4<f32>(vec4<f32>(i.m0.xyz, 0.0), i.m1, i.m2, i.m3);
  return object_vertex(model, axis_scale(model), v);
}

// A skinned instance: the bind pose (after its blendshapes) is its space,
// so the pattern follows the skin as it deforms.
@vertex
fn vs_object_skinned(@builtin(vertex_index) vi: u32, v: VertexIn, i: SkinnedInstanceIn, s: SkinIn) -> ObjectOut {
  let instance = mat4x4<f32>(i.m0, i.m1, i.m2, i.m3);
  let model = instance * skin_matrix(s, i.palette);
  return object_vertex(model, axis_scale(instance), morph_vertex(v, vi, i));
}

@fragment
fn fs_object(in: ObjectOut) -> @location(0) vec4<u32> {
  var n = in.object_normal;
  if (dot(n, n) < 1e-20) { n = vec3<f32>(0.0, 1.0, 0.0); }
  let e = clamp(oct_encode(normalize(n)) * 0.5 + 0.5, vec2<f32>(0.0), vec2<f32>(1.0));
  let q = vec2<u32>(round(e * 32767.0));
  return vec4<u32>(bitcast<vec3<u32>>(in.object_pos), 1u | (q.x << 1u) | (q.y << 16u));
}
