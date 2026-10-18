// Joint palette and morph deltas as Rgba32Float textures of 1024 texels
// per row, four texels (columns) per joint (WebGL2: no storage buffers in
// the vertex stage).
@group(SKIN_GROUP) @binding(0) var joint_palette: texture_2d<f32>;
@group(SKIN_GROUP) @binding(1) var morph_deltas: texture_2d<f32>;

fn row_texel(t: texture_2d<f32>, i: u32) -> vec4<f32> {
  return textureLoad(t, vec2<i32>(i32(i % 1024u), i32(i / 1024u)), 0);
}

fn joint_matrix(i: u32) -> mat4x4<f32> {
  let b = i * 4u;
  return mat4x4<f32>(row_texel(joint_palette, b), row_texel(joint_palette, b + 1u), row_texel(joint_palette, b + 2u), row_texel(joint_palette, b + 3u));
}

fn morph_delta(i: u32) -> vec4<f32> {
  return row_texel(morph_deltas, i);
}
