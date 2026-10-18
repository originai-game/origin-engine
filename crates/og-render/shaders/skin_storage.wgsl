// Joint palette and morph deltas as read-only storage buffers (WebGPU,
// Metal, Vulkan).
@group(SKIN_GROUP) @binding(0) var<storage, read> joint_palette: array<mat4x4<f32>>;
@group(SKIN_GROUP) @binding(1) var<storage, read> morph_deltas: array<vec4<f32>>;

fn joint_matrix(i: u32) -> mat4x4<f32> {
  return joint_palette[i];
}

fn morph_delta(i: u32) -> vec4<f32> {
  return morph_deltas[i];
}
