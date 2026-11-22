// Auto exposure metering: the log2 luminance of the HDR frame reduced to one
// texel. The first pass samples a 64x64 grid of 2x2 bilinear taps; each next
// pass averages 8x8 texels with textureLoad (no float filtering needed):
// 64 -> 8 -> 1. A texel holds the weighted log2 luminance in r and its
// weight in g, so the mean is r / g (fullscreen.wgsl `metered_log`).
// `fs_meter` weighs every tap 1; `fs_meter_tonemapped`, for a frame whose
// lit image's alpha is the fraction of each pixel the tone curve takes
// (untonemapped sprites, a camera's background texture), weighs each tap
// by it: what is drawn as given never moves the exposure, and a frame of
// nothing else still meters it. Prepended by fullscreen.wgsl.
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
  return vec4<f32>(sum * 0.25, 1.0, 0.0, 1.0);
}

@fragment
fn fs_meter_tonemapped(in: VsOut) -> @location(0) vec4<f32> {
  let texel = 1.0 / vec2<f32>(textureDimensions(source));
  var sum = vec2<f32>(0.0);
  for (var y = 0; y < 2; y++) {
    for (var x = 0; x < 2; x++) {
      let o = (vec2<f32>(f32(x), f32(y)) - 0.5) * texel * 4.0;
      let c = textureSampleLevel(source, source_sampler, in.uv + o, 0.0);
      let w = max(clamp(c.a, 0.0, 1.0), 1e-3);
      sum += vec2<f32>(log2(max(dot(c.rgb, LUMA), 1e-5)) * w, w);
    }
  }
  return vec4<f32>(sum * 0.25, 0.0, 1.0);
}

@fragment
fn fs_reduce(in: VsOut) -> @location(0) vec4<f32> {
  let size = vec2<i32>(textureDimensions(source));
  let base = vec2<i32>(in.pos.xy) * 8;
  var sum = vec2<f32>(0.0);
  for (var y = 0; y < 8; y++) {
    for (var x = 0; x < 8; x++) {
      sum += textureLoad(source, min(base + vec2<i32>(x, y), size - 1), 0).rg;
    }
  }
  return vec4<f32>(sum / 64.0, 0.0, 1.0);
}
