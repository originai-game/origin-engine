// SMAA reference/ WGSL port: MIT, third_party/smaa/LICENSE{,-wgsl}.txt.
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;

@fragment
fn fs(v: VsOut) -> @location(0) vec4f {
  let offsets = smaaEdgeDetectionOffsets(v.uv, 1.0 / vec2f(textureDimensions(source)));
  let edges = smaaLumaEdgeDetection(source, source_sampler, v.uv,
    offsets[0], offsets[1], offsets[2], vec2f(SMAA_THRESHOLD));
  return vec4f(edges, 0.0, 0.0);
}
