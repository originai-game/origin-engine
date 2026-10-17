// Depth-only pass from the directional light.
struct ShadowCamera { view_proj: mat4x4<f32> };
@group(0) @binding(0) var<uniform> shadow_camera: ShadowCamera;

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

@vertex
fn vs(v: VertexIn, i: InstanceIn) -> @builtin(position) vec4<f32> {
  let model = mat4x4<f32>(i.m0, i.m1, i.m2, i.m3);
  return shadow_camera.view_proj * model * vec4<f32>(v.position, 1.0);
}

// Depth also goes to an R32Float colour target: every backend (WebGL2 included) can
// fetch it as a plain texture, so the PCF compares are explicit and identical everywhere.
@fragment
fn fs(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
  return vec4<f32>(p.z, 0.0, 0.0, 1.0);
}
