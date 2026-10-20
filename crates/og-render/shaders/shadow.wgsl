// Shadow passes: depth from the directional light's cascades, or distance /
// range from a point or spot light. The value also goes to an R32Float colour
// target: every backend (WebGL2 included) can fetch it as a plain texture, so
// the PCF compares are explicit and identical everywhere. One uniform slot per
// pass, selected with a dynamic offset (shadow_pass.wgsl, with the
// paraboloid projection of a point light's hemispheres). Prepended by
// noise.wgsl, material.wgsl (group 1: dissolving and hooked casters),
// skin_*.wgsl (group 2: skinned casters), the material shader hooks
// (hooks.rs) and shadow_pass.wgsl: a material with a shader casts through
// vs_hooked / vs_skinned_hooked and fs_hooked, displaced by its
// vertex_offset and cut by its surface_color's alpha as its lit pass is.

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
  @location(3) normal: vec3<f32>,
  @location(4) color: vec4<f32>,
};

// As pbr.wgsl's world_normal: the inverse transpose of the upper 3x3.
fn shadow_normal(model: mat4x4<f32>, n: vec3<f32>) -> vec3<f32> {
  let a = model[0].xyz;
  let b = model[1].xyz;
  let c = model[2].xyz;
  let cofactor = mat3x3<f32>(cross(b, c), cross(c, a), cross(a, b));
  return normalize(cofactor * n) * select(1.0, -1.0, dot(a, cross(b, c)) < 0.0);
}

fn shadow_vertex(model: mat4x4<f32>, v: VertexIn) -> VsOut {
  let world = model * vec4<f32>(v.position, 1.0);
  var o: VsOut;
  o.clip = shadow_clip(world);
  o.world = world.xyz;
  o.uv = v.uv;
  o.local_pos = v.position;
  o.normal = vec3<f32>(0.0, 1.0, 0.0);
  o.color = v.color;
  return o;
}

@vertex
fn vs(v: VertexIn, i: InstanceIn) -> VsOut {
  return shadow_vertex(mat4x4<f32>(i.m0, i.m1, i.m2, i.m3), v);
}

fn shadow_hook(o: VsOut) -> Hook {
  var h: Hook;
  h.world = o.world;
  h.local = o.local_pos;
  h.normal = o.normal;
  h.uv = o.uv;
  h.color = o.color;
  h.time = shadow_pass.time.x;
  return h;
}

// A caster with a material shader: moved by its vertex_offset, as lit.
fn shadow_vertex_hooked(model: mat4x4<f32>, v: VertexIn) -> VsOut {
  var o = shadow_vertex(model, v);
  o.normal = shadow_normal(model, v.normal);
  o.world = o.world + vertex_offset(shadow_hook(o), shader_params());
  o.clip = shadow_clip(vec4<f32>(o.world, 1.0));
  return o;
}

@vertex
fn vs_hooked(v: VertexIn, i: InstanceIn) -> VsOut {
  return shadow_vertex_hooked(mat4x4<f32>(i.m0, i.m1, i.m2, i.m3), v);
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

@vertex
fn vs_skinned_hooked(@builtin(vertex_index) vi: u32, v: VertexIn, i: SkinnedInstanceIn, s: SkinIn) -> VsOut {
  let m = morphed(v.position, v.normal, vi, i.palette, i.morph0, i.morph1);
  var mv = v;
  mv.position = m.position;
  mv.normal = m.normal;
  return shadow_vertex_hooked(mat4x4<f32>(i.m0, i.m1, i.m2, i.m3) * skin_matrix(s, i.palette), mv);
}

// `dx`, `dy`: the world position's derivatives, taken first (before any
// discard) for a paraboloid pass's ray to the triangle's plane.
fn shadow_value(in: VsOut, dx: vec3<f32>, dy: vec3<f32>) -> vec4<f32> {
  if (shadow_pass.light.w > 0.0) {
    return vec4<f32>(local_distance(in.world, in.clip, dx, dy), 0.0, 0.0, 1.0);
  }
  return vec4<f32>(in.clip.z, 0.0, 0.0, 1.0);
}

@fragment
fn fs(in: VsOut) -> @location(0) vec4<f32> {
  let dx = dpdx(in.world);
  let dy = dpdy(in.world);
  return shadow_value(in, dx, dy);
}

// Dissolving casters: the dissolved pixels cast no shadow.
@fragment
fn fs_dissolve(in: VsOut) -> @location(0) vec4<f32> {
  let dx = dpdx(in.world);
  let dy = dpdy(in.world);
  if (dissolve_at(in.uv, in.local_pos).w > 0.5) { discard; }
  return shadow_value(in, dx, dy);
}

// Casters with a material shader: what its surface_color cuts below the
// alpha test (and the dissolve) casts no shadow, as it is not drawn lit.
@fragment
fn fs_hooked(in: VsOut) -> @location(0) vec4<f32> {
  let dx = dpdx(in.world);
  let dy = dpdy(in.world);
  let base = surface_color(shadow_hook(in), shader_params(), material.base_color * textureSample(base_color_tex, material_sampler, in.uv) * in.color);
  let dissolved = dissolve_at(in.uv, in.local_pos).w > 0.5;
  if ((material.emissive.w >= 0.0 && base.a < material.emissive.w) || dissolved) { discard; }
  return shadow_value(in, dx, dy);
}
