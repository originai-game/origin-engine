// Morph deltas and the joint palette in one Rgba32Float texture of 1024
// texels per row (WebGL2: no storage buffers in the vertex stage, and one
// texture slot to spare): the morph deltas from row 0, the palette after
// them, four texels (columns) per joint; the renderer offsets every
// instance's first joint past the morph rows.
@group(SKIN_GROUP) @binding(0) var skin_data: texture_2d<f32>;

fn row_texel(i: u32) -> vec4<f32> {
  return textureLoad(skin_data, vec2<i32>(i32(i % 1024u), i32(i / 1024u)), 0);
}

fn joint_matrix(i: u32) -> mat4x4<f32> {
  let b = i * 4u;
  return mat4x4<f32>(row_texel(b), row_texel(b + 1u), row_texel(b + 2u), row_texel(b + 3u));
}

fn morph_delta(i: u32) -> vec4<f32> {
  return row_texel(i);
}
