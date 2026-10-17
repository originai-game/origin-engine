// Shadow passes: depth from the directional light's cascades, or distance /
// range from a point or spot light. The value also goes to an R32Float colour
// target: every backend (WebGL2 included) can fetch it as a plain texture, so
// the PCF compares are explicit and identical everywhere. One uniform slot per
// pass, selected with a dynamic offset.
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
struct VsOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world: vec3<f32>,
};

@vertex
fn vs(v: VertexIn, i: InstanceIn) -> VsOut {
  let model = mat4x4<f32>(i.m0, i.m1, i.m2, i.m3);
  let world = model * vec4<f32>(v.position, 1.0);
  var o: VsOut;
  o.clip = shadow_pass.view_proj * world;
  o.world = world.xyz;
  return o;
}

@fragment
fn fs(in: VsOut) -> @location(0) vec4<f32> {
  if (shadow_pass.light.w > 0.0) {
    return vec4<f32>(length(in.world - shadow_pass.light.xyz) / shadow_pass.light.w, 0.0, 0.0, 1.0);
  }
  return vec4<f32>(in.clip.z, 0.0, 0.0, 1.0);
}
