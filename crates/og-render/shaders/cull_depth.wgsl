// The occluders' depth for GPU culling (cull_gpu.rs): last frame's
// visible occluders drawn with this frame's (jittered) view, with the
// same vertex arithmetic as the lit pass, so the depth matches its own.

struct View {
  view_proj: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> view: View;

@vertex
fn vs(@location(0) position: vec3<f32>,
      @location(4) m0: vec3<f32>,
      @location(5) m1: vec3<f32>,
      @location(6) m2: vec3<f32>,
      @location(7) m3: vec3<f32>) -> @builtin(position) vec4<f32> {
  return occluder_clip(position, m0, m1, m2, m3);
}

fn occluder_clip(position: vec3<f32>, m0: vec3<f32>, m1: vec3<f32>, m2: vec3<f32>, m3: vec3<f32>) -> vec4<f32> {
  let world = instance_matrix(m0, m1, m2, m3) * vec4<f32>(position, 1.0);
  return mul_point(view.view_proj, world);
}

// `vs` from a compact vertex (compact.rs): the position in the mesh's unit
// box, the instance matrix holding the dequantisation.
@vertex
fn vs_compact(@location(0) position: vec4<f32>,
              @location(4) m0: vec3<f32>,
              @location(5) m1: vec3<f32>,
              @location(6) m2: vec3<f32>,
              @location(7) m3: vec3<f32>) -> @builtin(position) vec4<f32> {
  return occluder_clip(position.xyz, m0, m1, m2, m3);
}
