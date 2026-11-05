// Extended sRGB surface transfer, after linear scene/UI composition.
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(0) @binding(2) var<uniform> output: vec4f; // scale, encode, peak / white, unused

@fragment
fn fs(v: VsOut) -> @location(0) vec4f {
  let color = clamp(sample_exact(source, source_sampler, v.uv).rgb, vec3f(0.0), vec3f(output.z)) * output.x;
  if (output.y > 0.5) {
    return vec4f(select(color * 12.92, 1.055 * pow(color, vec3f(1.0 / 2.4)) - 0.055, color > vec3f(0.0031308)), 1.0);
  }
  return vec4f(color, 1.0);
}
