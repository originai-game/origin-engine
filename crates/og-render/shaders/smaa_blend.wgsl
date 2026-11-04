// SMAA reference/ WGSL port: MIT, third_party/smaa/LICENSE{,-wgsl}.txt.
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var linear_sampler: sampler;
@group(0) @binding(2) var weights: texture_2d<f32>;

fn blend(uv: vec2f) -> vec4f {
  let texel = 1.0 / vec2f(textureDimensions(source));
  return smaaNeighborhoodBlending(source, linear_sampler, weights, linear_sampler,
    texel, uv, smaaNeighborhoodBlendingOffset(uv, texel));
}

@fragment
fn fs(v: VsOut) -> @location(0) vec4f {
  return blend(v.uv);
}

@fragment
fn fs_srgb_encode(v: VsOut) -> @location(0) vec4f {
  let c = blend(v.uv);
  return vec4f(select(c.rgb * 12.92,
    1.055 * pow(max(c.rgb, vec3f(0.0)), vec3f(1.0 / 2.4)) - 0.055,
    c.rgb > vec3f(0.0031308)), c.a);
}
