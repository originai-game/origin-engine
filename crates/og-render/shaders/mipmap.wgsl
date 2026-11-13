// Mip generation: each level is a fullscreen triangle sampling the level above
// (a view restricted to that one level) with a linear filter, i.e. a 2x2 box
// filter for power-of-two sizes. sRGB textures are averaged in linear space
// because sampling decodes and the sRGB target encodes.
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;

@fragment
fn fs(in: VsOut) -> @location(0) vec4<f32> {
  return textureSampleLevel(source, source_sampler, in.uv, 0.0);
}
