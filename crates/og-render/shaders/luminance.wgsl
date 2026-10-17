// Auto exposure metering: the log2 luminance of the HDR frame reduced to one
// texel. The first pass samples a 64x64 grid of 2x2 bilinear taps; each next
// pass averages 8x8 texels with textureLoad (no float filtering needed):
// 64 -> 8 -> 1. Prepended by fullscreen.wgsl.
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;

@fragment
fn fs_meter(in: VsOut) -> @location(0) vec4<f32> {
  let texel = 1.0 / vec2<f32>(textureDimensions(source));
  var sum = 0.0;
  for (var y = 0; y < 2; y++) {
    for (var x = 0; x < 2; x++) {
      let o = (vec2<f32>(f32(x), f32(y)) - 0.5) * texel * 4.0;
      let c = textureSampleLevel(source, source_sampler, in.uv + o, 0.0).rgb;
      sum += log2(max(dot(c, LUMA), 1e-5));
    }
  }
  return vec4<f32>(sum * 0.25, 0.0, 0.0, 1.0);
}

@fragment
fn fs_reduce(in: VsOut) -> @location(0) vec4<f32> {
  let size = vec2<i32>(textureDimensions(source));
  let base = vec2<i32>(in.pos.xy) * 8;
  var sum = 0.0;
  for (var y = 0; y < 8; y++) {
    for (var x = 0; x < 8; x++) {
      sum += textureLoad(source, min(base + vec2<i32>(x, y), size - 1), 0).r;
    }
  }
  return vec4<f32>(sum / 64.0, 0.0, 0.0, 1.0);
}
