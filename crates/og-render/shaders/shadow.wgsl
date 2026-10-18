// Shadow passes: depth from the directional light's cascades, or distance /
// range from a point or spot light. The value also goes to an R32Float colour
// target: every backend (WebGL2 included) can fetch it as a plain texture, so
// the PCF compares are explicit and identical everywhere. One uniform slot per
// pass, selected with a dynamic offset. Prepended by noise.wgsl,
// material.wgsl (group 1: dissolving casters) and skin_*.wgsl (group 2:
// skinned casters).
struct ShadowPass {
  view_proj: mat4x4<f32>,
  light: vec4<f32>,  // xyz: light position, w: range (0: directional, write depth)
};
@group(0) @binding(0) var<uniform> shadow_pass: ShadowPass;

struct VertexIn {
  @location(0) position: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
  @location(3) color: vec4<f32>,
};
struct InstanceIn {
  @location(4) m0: vec4<f32>,
  @location(5) m1: vec4<f32>,
  @location(6) m2: vec4<f32>,
  @location(7) m3: vec4<f32>,
};
struct SkinnedInstanceIn {
  @location(4) m0: vec4<f32>,
  @location(5) m1: vec4<f32>,
  @location(6) m2: vec4<f32>,
  @location(7) m3: vec4<f32>,
  @location(8) palette: vec4<f32>,
  @location(11) morph0: vec4<f32>,
  @location(12) morph1: vec4<f32>,
};
struct SkinIn {
  @location(9) joints: vec4<u32>,
  @location(10) weights: vec4<f32>,
};
struct VsOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world: vec3<f32>,
  @location(1) uv: vec2<f32>,
  @location(2) local_pos: vec3<f32>,
};

fn shadow_vertex(model: mat4x4<f32>, v: VertexIn) -> VsOut {
  let world = model * vec4<f32>(v.position, 1.0);
  var o: VsOut;
  o.clip = shadow_pass.view_proj * world;
  o.world = world.xyz;
  o.uv = v.uv;
  o.local_pos = v.position;
  return o;
}

@vertex
fn vs(v: VertexIn, i: InstanceIn) -> VsOut {
  return shadow_vertex(mat4x4<f32>(i.m0, i.m1, i.m2, i.m3), v);
}

fn skin_matrix(s: SkinIn, palette: vec4<f32>) -> mat4x4<f32> {
  let total = s.weights.x + s.weights.y + s.weights.z + s.weights.w;
  if (total < 1e-6) {
    return mat4x4<f32>(vec4<f32>(1.0, 0.0, 0.0, 0.0), vec4<f32>(0.0, 1.0, 0.0, 0.0), vec4<f32>(0.0, 0.0, 1.0, 0.0), vec4<f32>(0.0, 0.0, 0.0, 1.0));
  }
  let w = s.weights / total;
  let base = u32(palette.x);
  let last = u32(max(palette.y, 1.0)) - 1u;
  return joint_matrix(base + min(s.joints.x, last)) * w.x
    + joint_matrix(base + min(s.joints.y, last)) * w.y
    + joint_matrix(base + min(s.joints.z, last)) * w.z
    + joint_matrix(base + min(s.joints.w, last)) * w.w;
}


struct Morphed {
  position: vec3<f32>,
  normal: vec3<f32>,
};

// Blendshapes: the weighted position and normal deltas of vertex `vi`
// (palette.z: the mesh's first delta, palette.w: its targets).
fn morphed(position: vec3<f32>, normal: vec3<f32>, vi: u32, palette: vec4<f32>, w0: vec4<f32>, w1: vec4<f32>) -> Morphed {
  var m: Morphed;
  m.position = position;
  m.normal = normal;
  let targets = u32(palette.w);
  if (targets == 0u) { return m; }
  let base = u32(palette.z) + vi * targets * 2u;
  var w = array<f32, 8>(w0.x, w0.y, w0.z, w0.w, w1.x, w1.y, w1.z, w1.w);
  for (var t = 0u; t < min(targets, 8u); t++) {
    if (w[t] != 0.0) {
      m.position += morph_delta(base + t * 2u).xyz * w[t];
      m.normal += morph_delta(base + t * 2u + 1u).xyz * w[t];
    }
  }
  return m;
}

@vertex
fn vs_skinned(@builtin(vertex_index) vi: u32, v: VertexIn, i: SkinnedInstanceIn, s: SkinIn) -> VsOut {
  let m = morphed(v.position, v.normal, vi, i.palette, i.morph0, i.morph1);
  var mv = v;
  mv.position = m.position;
  return shadow_vertex(mat4x4<f32>(i.m0, i.m1, i.m2, i.m3) * skin_matrix(s, i.palette), mv);
}

fn shadow_value(in: VsOut) -> vec4<f32> {
  if (shadow_pass.light.w > 0.0) {
    return vec4<f32>(length(in.world - shadow_pass.light.xyz) / shadow_pass.light.w, 0.0, 0.0, 1.0);
  }
  return vec4<f32>(in.clip.z, 0.0, 0.0, 1.0);
}

@fragment
fn fs(in: VsOut) -> @location(0) vec4<f32> {
  return shadow_value(in);
}

// Dissolving casters: the dissolved pixels cast no shadow.
@fragment
fn fs_dissolve(in: VsOut) -> @location(0) vec4<f32> {
  if (dissolve_at(in.uv, in.local_pos).w > 0.5) { discard; }
  return shadow_value(in);
}
