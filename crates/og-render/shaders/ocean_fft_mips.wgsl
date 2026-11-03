// The FFT sea's fields' mip chain (ocean_fft/): each level the box average of the one above, every layer, so the slopes'
// moments average with the slopes (LEAN mapping: E[s²] − E[s]² is the slope variance under a texel of any level).

@group(0) @binding(0) var src: texture_2d_array<f32>;
@group(0) @binding(1) var dst: texture_storage_2d_array<rgba16float, write>;

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
  let size = textureDimensions(dst);
  if (id.x >= size.x || id.y >= size.y) { return; }
  let p = vec2<i32>(id.xy) * 2;
  let l = i32(id.z);
  let v = 0.25 * (textureLoad(src, p, l, 0) + textureLoad(src, p + vec2<i32>(1, 0), l, 0)
    + textureLoad(src, p + vec2<i32>(0, 1), l, 0) + textureLoad(src, p + vec2<i32>(1, 1), l, 0));
  textureStore(dst, id.xy, id.z, v);
}
