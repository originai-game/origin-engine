// SMAA reference/ WGSL port and area/search textures: MIT,
// third_party/smaa/LICENSE{,-wgsl}.txt.
@group(0) @binding(0) var edges: texture_2d<f32>;
@group(0) @binding(1) var linear_sampler: sampler;
@group(0) @binding(2) var area: texture_2d<f32>;
@group(0) @binding(3) var search: texture_2d<f32>;

@fragment
fn fs(v: VsOut) -> @location(0) vec4f {
  let size = vec2f(textureDimensions(edges));
  let texel = 1.0 / size;
  let offsets = smaaBlendingWeightCalculationOffsets(v.uv, texel);
  return smaaBlendingWeightCalculation(edges, linear_sampler, area, linear_sampler,
    search, linear_sampler, size, texel, v.uv, v.uv * size,
    offsets[0], offsets[1], offsets[2], vec4f(0.0));
}
