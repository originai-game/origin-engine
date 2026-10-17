// Exposure, bloom and the tonemap. Prepended by fullscreen.wgsl.
//
// Exposure is either manual (2^ev) or metered: the 1x1 log2-luminance texel
// is brought to middle grey, plus compensation, clamped. Bloom: a prefilter
// (exposure, soft threshold) into the first level of a half-resolution chain,
// 13-tap downsamples, then tent upsamples added back level by level; the post
// pass adds the top level. `fs` writes linear values for an sRGB output (the
// hardware encodes); `fs_srgb_encode` encodes in the shader for a non-sRGB
// output (web canvases).
struct Post {
  exposure: vec4<f32>,  // x: manual ev, y: auto (1) or manual (0), z: min ev, w: max ev
  bloom: vec4<f32>,     // x: threshold, y: knee, z: intensity (0: off), w: compensation
  mode: vec4<f32>,      // x: tonemap (0 ACES, 1 none), y: upsample radius in texels
};
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var linear_sampler: sampler;
@group(0) @binding(2) var<uniform> post: Post;
@group(0) @binding(3) var luminance: texture_2d<f32>;
@group(0) @binding(4) var bloom_tex: texture_2d<f32>;

fn exposure() -> f32 {
  if (post.exposure.y > 0.5) {
    let avg = textureLoad(luminance, vec2<i32>(0), 0).r;
    let ev = clamp(log2(0.18) - avg + post.bloom.w, post.exposure.z, post.exposure.w);
    return exp2(ev);
  }
  return exp2(post.exposure.x);
}

fn aces(x: vec3<f32>) -> vec3<f32> {
  let a = 2.51; let b = 0.03; let c = 2.43; let d = 0.59; let e = 0.14;
  return clamp((x * (a * x + b)) / (x * (c * x + d) + e), vec3<f32>(0.0), vec3<f32>(1.0));
}

fn tonemapped(uv: vec2<f32>) -> vec3<f32> {
  var c = textureSampleLevel(source, linear_sampler, uv, 0.0).rgb * exposure();
  if (post.bloom.z > 0.0) {
    c += textureSampleLevel(bloom_tex, linear_sampler, uv, 0.0).rgb * post.bloom.z;
  }
  if (post.mode.x > 0.5) { return clamp(c, vec3<f32>(0.0), vec3<f32>(1.0)); }
  return aces(c);
}

@fragment
fn fs(in: VsOut) -> @location(0) vec4<f32> {
  return vec4<f32>(tonemapped(in.uv), 1.0);
}

fn srgb_encode(c: vec3<f32>) -> vec3<f32> {
  let lo = c * 12.92;
  let hi = 1.055 * pow(c, vec3<f32>(1.0 / 2.4)) - 0.055;
  return select(hi, lo, c <= vec3<f32>(0.0031308));
}

@fragment
fn fs_srgb_encode(in: VsOut) -> @location(0) vec4<f32> {
  return vec4<f32>(srgb_encode(tonemapped(in.uv)), 1.0);
}

// The 13-tap downsample (Jimenez, "Next generation post processing in Call of Duty").
fn downsample13(uv: vec2<f32>) -> vec3<f32> {
  let t = 1.0 / vec2<f32>(textureDimensions(source));
  let s = textureSampleLevel(source, linear_sampler, uv, 0.0).rgb;
  let a = textureSampleLevel(source, linear_sampler, uv + t * vec2<f32>(-2.0, -2.0), 0.0).rgb;
  let b = textureSampleLevel(source, linear_sampler, uv + t * vec2<f32>(0.0, -2.0), 0.0).rgb;
  let c = textureSampleLevel(source, linear_sampler, uv + t * vec2<f32>(2.0, -2.0), 0.0).rgb;
  let d = textureSampleLevel(source, linear_sampler, uv + t * vec2<f32>(-2.0, 0.0), 0.0).rgb;
  let f = textureSampleLevel(source, linear_sampler, uv + t * vec2<f32>(2.0, 0.0), 0.0).rgb;
  let g = textureSampleLevel(source, linear_sampler, uv + t * vec2<f32>(-2.0, 2.0), 0.0).rgb;
  let h = textureSampleLevel(source, linear_sampler, uv + t * vec2<f32>(0.0, 2.0), 0.0).rgb;
  let i = textureSampleLevel(source, linear_sampler, uv + t * vec2<f32>(2.0, 2.0), 0.0).rgb;
  let j = textureSampleLevel(source, linear_sampler, uv + t * vec2<f32>(-1.0, -1.0), 0.0).rgb;
  let k = textureSampleLevel(source, linear_sampler, uv + t * vec2<f32>(1.0, -1.0), 0.0).rgb;
  let l = textureSampleLevel(source, linear_sampler, uv + t * vec2<f32>(-1.0, 1.0), 0.0).rgb;
  let m = textureSampleLevel(source, linear_sampler, uv + t * vec2<f32>(1.0, 1.0), 0.0).rgb;
  return s * 0.125 + (a + c + g + i) * 0.03125 + (b + d + f + h) * 0.0625 + (j + k + l + m) * 0.125;
}

@fragment
fn fs_bloom_prefilter(in: VsOut) -> @location(0) vec4<f32> {
  let c = downsample13(in.uv) * exposure();
  // Soft threshold (a quadratic knee around the threshold).
  let brightness = max(c.r, max(c.g, c.b));
  let knee = max(post.bloom.y, 1e-4);
  var soft = clamp(brightness - post.bloom.x + knee, 0.0, 2.0 * knee);
  soft = soft * soft / (4.0 * knee);
  let contribution = max(soft, brightness - post.bloom.x) / max(brightness, 1e-4);
  return vec4<f32>(c * contribution, 1.0);
}

@fragment
fn fs_bloom_down(in: VsOut) -> @location(0) vec4<f32> {
  return vec4<f32>(downsample13(in.uv), 1.0);
}

// A 3x3 tent over the smaller level, added onto the larger one by blending.
@fragment
fn fs_bloom_up(in: VsOut) -> @location(0) vec4<f32> {
  let t = post.mode.y / vec2<f32>(textureDimensions(source));
  var c = textureSampleLevel(source, linear_sampler, in.uv, 0.0).rgb * 4.0;
  c += textureSampleLevel(source, linear_sampler, in.uv + t * vec2<f32>(-1.0, 0.0), 0.0).rgb * 2.0;
  c += textureSampleLevel(source, linear_sampler, in.uv + t * vec2<f32>(1.0, 0.0), 0.0).rgb * 2.0;
  c += textureSampleLevel(source, linear_sampler, in.uv + t * vec2<f32>(0.0, -1.0), 0.0).rgb * 2.0;
  c += textureSampleLevel(source, linear_sampler, in.uv + t * vec2<f32>(0.0, 1.0), 0.0).rgb * 2.0;
  c += textureSampleLevel(source, linear_sampler, in.uv + t * vec2<f32>(-1.0, -1.0), 0.0).rgb;
  c += textureSampleLevel(source, linear_sampler, in.uv + t * vec2<f32>(1.0, -1.0), 0.0).rgb;
  c += textureSampleLevel(source, linear_sampler, in.uv + t * vec2<f32>(-1.0, 1.0), 0.0).rgb;
  c += textureSampleLevel(source, linear_sampler, in.uv + t * vec2<f32>(1.0, 1.0), 0.0).rgb;
  return vec4<f32>(c / 16.0, 1.0);
}
