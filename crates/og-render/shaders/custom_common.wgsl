// Full custom shaders (custom.rs): the binding contract a `mode full`
// shader writes against, shared by the lit, prepass and shadow modules.
// Prepended by the material bindings (material.wgsl, group 1), the joint
// palette (skin_*.wgsl) and instance.wgsl; followed by the shader's
// parameters (`Params`, `shader_params()`), its code, the defaults of the
// entries it leaves out, and the pass's wrappers (custom_lit.wgsl or
// custom_shadow.wgsl), which define `camera()`.
//
//   fn vertex(v: VertexInput, model: mat4x4<f32>, p: Params) -> VertexOutput
//   fn fragment(f: FragmentInput, p: Params) -> SurfaceOutput
//
// `model` is the instance's transform with a skinned mesh's joints folded
// in (and its morph targets already in v.position and v.normal); the
// engine projects VertexOutput.world with the pass's camera, so the lit
// pass, the depth prepass, every shadow pass, reflections and views all
// draw the mesh where the vertex entry put it. A SurfaceOutput alpha below
// the material's alphaTest discards the fragment in every pass (a
// dissolve's cut casts no shadow).

struct VertexInput {
  position: vec3<f32>,
  normal: vec3<f32>,
  uv: vec2<f32>,
  color: vec4<f32>,
  tangent: vec4<f32>,
};

struct VertexOutput {
  world: vec3<f32>,   // world position (projected by the engine)
  normal: vec3<f32>,  // world normal
  uv: vec2<f32>,
  color: vec4<f32>,
  data: vec4<f32>,    // free for the shader, interpolated to the fragment
};

struct FragmentInput {
  world: vec3<f32>,
  local: vec3<f32>,   // the vertex's object-space position
  normal: vec3<f32>,  // interpolated world normal (not normalised)
  uv: vec2<f32>,
  color: vec4<f32>,
  data: vec4<f32>,
  view: vec3<f32>,    // towards the eye (zero in shadow passes)
  pixel: vec2<f32>,   // the fragment's position in pixels
  time: f32,          // the scene clock in seconds
};

struct SurfaceOutput {
  color: vec4<f32>,   // lit: the albedo and alpha; unlit: the colour and alpha
  normal: vec3<f32>,  // world normal (lit)
  metallic: f32,
  roughness: f32,
  emissive: vec3<f32>,
};

// The pass's camera: the eye's view projection, position and forward
// axis (a shadow pass: the light's, forward zero), and the clock.
struct Camera {
  view_proj: mat4x4<f32>,
  position: vec3<f32>,
  forward: vec3<f32>,
  time: f32,
};

// The material's eight texture slots, by number (StandardMaterialDesc's
// map, metalnessMap, roughnessMap, normalMap, emissiveMap, aoMap,
// gradientMap, dissolveMap; white where unset, normalMap flat), with its
// sampler (repeat, trilinear).
fn tex0(uv: vec2<f32>) -> vec4<f32> { return textureSampleLevel(base_color_tex, material_sampler, uv, 0.0); }
fn tex1(uv: vec2<f32>) -> vec4<f32> { return textureSampleLevel(metalness_tex, material_sampler, uv, 0.0); }
fn tex2(uv: vec2<f32>) -> vec4<f32> { return textureSampleLevel(roughness_tex, material_sampler, uv, 0.0); }
fn tex3(uv: vec2<f32>) -> vec4<f32> { return textureSampleLevel(normal_tex, material_sampler, uv, 0.0); }
fn tex4(uv: vec2<f32>) -> vec4<f32> { return textureSampleLevel(emissive_tex, material_sampler, uv, 0.0); }
fn tex5(uv: vec2<f32>) -> vec4<f32> { return textureSampleLevel(ao_tex, material_sampler, uv, 0.0); }
fn tex6(uv: vec2<f32>) -> vec4<f32> { return textureSampleLevel(gradient_tex, material_sampler, uv, 0.0); }
fn tex7(uv: vec2<f32>) -> vec4<f32> { return textureSampleLevel(dissolve_tex, material_sampler, uv, 0.0); }

// Value noise in 0..1 over a 3D point (noise.wgsl), for dissolves and
// procedural patterns.
fn noise3(p: vec3<f32>) -> f32 {
  return value_noise3(p, 17u);
}

// The built-in vertex stage: the mesh where its transform puts it.
fn standard_vertex(v: VertexInput, model: mat4x4<f32>) -> VertexOutput {
  var o: VertexOutput;
  o.world = (model * vec4<f32>(v.position, 1.0)).xyz;
  let a = model[0].xyz;
  let b = model[1].xyz;
  let c = model[2].xyz;
  let cofactor = mat3x3<f32>(cross(b, c), cross(c, a), cross(a, b));
  o.normal = normalize(cofactor * v.normal) * select(1.0, -1.0, dot(a, cross(b, c)) < 0.0);
  o.uv = v.uv;
  o.color = v.color;
  o.data = vec4<f32>(0.0);
  return o;
}

// The material's own surface at a fragment: its colour times map and
// vertex colour, its metalness, roughness and emission.
fn standard_surface(f: FragmentInput) -> SurfaceOutput {
  var s: SurfaceOutput;
  s.color = material.base_color * tex0(f.uv) * f.color;
  s.normal = normalize(f.normal);
  s.metallic = material.mr.x;
  s.roughness = material.mr.y;
  s.emissive = material.emissive.rgb * tex4(f.uv).rgb;
  return s;
}

// The weighted joint matrices of a skinned vertex (as pbr.wgsl's).
fn custom_skin(joints: vec4<u32>, weights: vec4<f32>, palette: vec4<f32>) -> mat4x4<f32> {
  let total = weights.x + weights.y + weights.z + weights.w;
  if (total < 1e-6) {
    return mat4x4<f32>(vec4<f32>(1.0, 0.0, 0.0, 0.0), vec4<f32>(0.0, 1.0, 0.0, 0.0), vec4<f32>(0.0, 0.0, 1.0, 0.0), vec4<f32>(0.0, 0.0, 0.0, 1.0));
  }
  let w = weights / total;
  let base = u32(palette.x);
  let last = u32(max(palette.y, 1.0)) - 1u;
  return joint_matrix(base + min(joints.x, last)) * w.x
    + joint_matrix(base + min(joints.y, last)) * w.y
    + joint_matrix(base + min(joints.z, last)) * w.z
    + joint_matrix(base + min(joints.w, last)) * w.w;
}

// A skinned vertex's morph targets (as pbr.wgsl's `morphed`).
fn custom_morph(v: VertexInput, vi: u32, palette: vec4<f32>, w0: vec4<f32>, w1: vec4<f32>) -> VertexInput {
  var o = v;
  let targets = u32(palette.w);
  if (targets == 0u) { return o; }
  let base = u32(palette.z) + vi * targets * 2u;
  var w = array<f32, 8>(w0.x, w0.y, w0.z, w0.w, w1.x, w1.y, w1.z, w1.w);
  for (var t = 0u; t < min(targets, 8u); t++) {
    if (w[t] != 0.0) {
      o.position += morph_delta(base + t * 2u).xyz * w[t];
      o.normal += morph_delta(base + t * 2u + 1u).xyz * w[t];
    }
  }
  return o;
}

// The vertex buffers every custom entry reads (lib.rs VERTEX_LAYOUT and
// the instance layouts).
struct CustomVertexIn {
  @location(0) position: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
  @location(3) color: vec4<f32>,
  @location(15) tangent: vec4<f32>,
};
struct CustomInstanceIn {
  @location(4) m0: vec3<f32>,
  @location(5) m1: vec3<f32>,
  @location(6) m2: vec3<f32>,
  @location(7) m3: vec3<f32>,
};
struct CustomSkinnedIn {
  @location(4) m0: vec3<f32>,
  @location(5) m1: vec3<f32>,
  @location(6) m2: vec3<f32>,
  @location(7) m3: vec3<f32>,
  @location(8) palette: vec4<f32>,
  @location(11) morph0: vec4<f32>,
  @location(12) morph1: vec4<f32>,
};
struct CustomSkinIn {
  @location(9) joints: vec4<u32>,
  @location(10) weights: vec4<f32>,
};

struct CustomOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
  @location(3) color: vec4<f32>,
  @location(4) local: vec3<f32>,
  @location(5) data: vec4<f32>,
};

fn custom_input(v: CustomVertexIn) -> VertexInput {
  var o: VertexInput;
  o.position = v.position;
  o.normal = v.normal;
  o.uv = v.uv;
  o.color = v.color;
  o.tangent = v.tangent;
  return o;
}

// The shader's vertex entry for a plain or a skinned instance, as the
// pass's output (projected by `project`, the pass's camera).
fn custom_place(v: VertexInput, model: mat4x4<f32>) -> CustomOut {
  let o = vertex(v, model, shader_params());
  var r: CustomOut;
  r.clip = project(o.world);
  r.world = o.world;
  r.normal = o.normal;
  r.uv = o.uv;
  r.color = o.color;
  r.local = v.position;
  r.data = o.data;
  return r;
}

fn custom_plain(v: CustomVertexIn, i: CustomInstanceIn) -> CustomOut {
  return custom_place(custom_input(v), instance_matrix(i.m0, i.m1, i.m2, i.m3));
}

fn custom_skinned(vi: u32, v: CustomVertexIn, i: CustomSkinnedIn, s: CustomSkinIn) -> CustomOut {
  let model = instance_matrix(i.m0, i.m1, i.m2, i.m3) * custom_skin(s.joints, s.weights, i.palette);
  return custom_place(custom_morph(custom_input(v), vi, i.palette, i.morph0, i.morph1), model);
}

// The fragment entry's input and the shader's surface there.
fn custom_surface(in: CustomOut, view: vec3<f32>) -> SurfaceOutput {
  var f: FragmentInput;
  f.world = in.world;
  f.local = in.local;
  f.normal = in.normal;
  f.uv = in.uv;
  f.color = in.color;
  f.data = in.data;
  f.view = view;
  f.pixel = in.clip.xy;
  f.time = camera().time;
  return fragment(f, shader_params());
}

// Whether a surface is cut (alpha below the material's alphaTest).
fn custom_cut(s: SurfaceOutput) -> bool {
  return material.emissive.w >= 0.0 && s.color.a < material.emissive.w;
}
