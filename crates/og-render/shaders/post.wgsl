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
  mode: vec4<f32>,      // x: tonemap (0 ACES, 1 none), y: upsample radius in texels, z: 1 when the lit image feeds bloom, w: 1 when the glow buffer does
  grade: vec4<f32>,     // x: LUT intensity (0 off), y: vignette intensity (0 off), z: vignette radius, w: vignette smoothness
  vignette: vec4<f32>,  // rgb: vignette colour, w: chromatic aberration at the corners (uv units, 0 off)
  grain: vec4<f32>,     // x: intensity (0 off), y: size in pixels, z: seed (0..1)
  flash: vec4<f32>,     // rgb: flash colour, w: amount (0 off)
  extra: vec4<f32>,     // x: 1 with god rays
};
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var linear_sampler: sampler;
@group(0) @binding(2) var<uniform> post: Post;
@group(0) @binding(3) var luminance: texture_2d<f32>;
@group(0) @binding(4) var bloom_tex: texture_2d<f32>;
@group(0) @binding(5) var grade_lut: texture_3d<f32>;
@group(0) @binding(6) var god_rays: texture_2d<f32>;

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
  var lit: vec3<f32>;
  if (post.vignette.w > 0.0) {
    // Chromatic aberration: red and blue pulled apart, growing towards the corners.
    let d = uv - 0.5;
    let off = d * length(d) * 2.0 * post.vignette.w;
    lit = vec3<f32>(
      textureSampleLevel(source, linear_sampler, uv - off, 0.0).r,
      textureSampleLevel(source, linear_sampler, uv, 0.0).g,
      textureSampleLevel(source, linear_sampler, uv + off, 0.0).b);
  } else {
    lit = textureSampleLevel(source, linear_sampler, uv, 0.0).rgb;
  }
  var c = lit * exposure();
  if (post.bloom.z > 0.0) {
    c += textureSampleLevel(bloom_tex, linear_sampler, uv, 0.0).rgb * post.bloom.z;
  }
  if (post.extra.x > 0.5) {
    c += textureSampleLevel(god_rays, linear_sampler, uv, 0.0).rgb * exposure();
  }
  if (post.mode.x > 0.5) { return clamp(c, vec3<f32>(0.0), vec3<f32>(1.0)); }
  return aces(c);
}

fn srgb_encode(c: vec3<f32>) -> vec3<f32> {
  let lo = c * 12.92;
  let hi = 1.055 * pow(c, vec3<f32>(1.0 / 2.4)) - 0.055;
  return select(hi, lo, c <= vec3<f32>(0.0031308));
}

fn srgb_decode(c: vec3<f32>) -> vec3<f32> {
  let lo = c / 12.92;
  let hi = pow((c + 0.055) / 1.055, vec3<f32>(2.4));
  return select(hi, lo, c <= vec3<f32>(0.04045));
}

fn grain_hash(p: vec2<u32>, seed: u32) -> f32 {
  var h = p.x * 1664525u + p.y * 1013904223u + seed * 2654435761u;
  h = (h ^ (h >> 16u)) * 2246822519u;
  h = (h ^ (h >> 13u)) * 3266489917u;
  h = h ^ (h >> 16u);
  return f32(h >> 8u) / 16777216.0;
}

// The display colour (linear 0..1): grading, vignette, grain, flash.
fn finished(uv: vec2<f32>, pos: vec2<f32>) -> vec3<f32> {
  var c = tonemapped(uv);
  if (post.grade.x > 0.0) {
    let n = f32(textureDimensions(grade_lut).x);
    let e = clamp(srgb_encode(clamp(c, vec3<f32>(0.0), vec3<f32>(1.0))), vec3<f32>(0.0), vec3<f32>(1.0));
    let graded = textureSampleLevel(grade_lut, linear_sampler, e * ((n - 1.0) / n) + 0.5 / n, 0.0).rgb;
    c = mix(c, srgb_decode(graded), post.grade.x);
  }
  if (post.grade.y > 0.0) {
    let d = length((uv - 0.5) * 2.0) / 1.41421356;
    let v = smoothstep(post.grade.z, post.grade.z + max(post.grade.w, 1e-3), d);
    c = mix(c, post.vignette.rgb, v * post.grade.y);
  }
  if (post.grain.x > 0.0) {
    let cell = vec2<u32>(pos / max(post.grain.y, 1.0));
    let g = grain_hash(cell, u32(post.grain.z * 16777216.0)) - 0.5;
    // Strongest in the mid-tones.
    let l = dot(c, LUMA);
    c = max(c + vec3<f32>(g * post.grain.x * (1.0 - abs(l * 2.0 - 1.0) * 0.6)), vec3<f32>(0.0));
  }
  if (post.flash.w > 0.0) {
    c = mix(c, post.flash.rgb, post.flash.w);
  }
  return c;
}

@fragment
fn fs(in: VsOut) -> @location(0) vec4<f32> {
  return vec4<f32>(finished(in.uv, in.pos.xy), 1.0);
}

@fragment
fn fs_srgb_encode(in: VsOut) -> @location(0) vec4<f32> {
  return vec4<f32>(srgb_encode(clamp(finished(in.uv, in.pos.xy), vec3<f32>(0.0), vec3<f32>(1.0))), 1.0);
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
  var out = vec3<f32>(0.0);
  if (post.mode.z > 0.5) {
    let c = downsample13(in.uv) * exposure();
    // Soft threshold (a quadratic knee around the threshold).
    let brightness = max(c.r, max(c.g, c.b));
    let knee = max(post.bloom.y, 1e-4);
    var soft = clamp(brightness - post.bloom.x + knee, 0.0, 2.0 * knee);
    soft = soft * soft / (4.0 * knee);
    let contribution = max(soft, brightness - post.bloom.x) / max(brightness, 1e-4);
    out = c * contribution;
  }
  if (post.mode.w > 0.5) {
    // The order-independent glow buffer joins without a threshold.
    out += textureSampleLevel(bloom_tex, linear_sampler, in.uv, 0.0).rgb * exposure();
  }
  return vec4<f32>(out, 1.0);
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
