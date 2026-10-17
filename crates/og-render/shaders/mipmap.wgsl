// Mip generation: each level is a fullscreen triangle sampling the level above
// (a view restricted to that one level) with a linear filter, i.e. a 2x2 box
// filter for power-of-two sizes. sRGB textures are averaged in linear space
// because sampling decodes and the sRGB target encodes.
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;

struct VsOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };

@vertex
fn vs(@builtin(vertex_index) i: u32) -> VsOut {
  var o: VsOut;
  let x = f32(i32(i & 1u) * 4 - 1);
  let y = f32(i32(i & 2u) * 2 - 1);
  o.pos = vec4<f32>(x, y, 0.0, 1.0);
  o.uv = vec2<f32>(x * 0.5 + 0.5, 0.5 - y * 0.5);
  return o;
}

@fragment
fn fs(in: VsOut) -> @location(0) vec4<f32> {
  return textureSampleLevel(source, source_sampler, in.uv, 0.0);
}
