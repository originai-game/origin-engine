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
  exposure: vec4<f32>,  // x: manual ev, y: manual (0), auto (1) or adapted (2: `luminance` holds the EV), z: min ev, w: max ev
  bloom: vec4<f32>,     // x: threshold, y: knee, z: intensity (0: off), w: compensation
  mode: vec4<f32>,      // x: tonemap (0 ACES, 1 none, 2 AgX), y: upsample radius in texels, z: 1 when the lit image feeds bloom, w: 1 when the glow buffer does
  grade: vec4<f32>,     // x: LUT intensity (0 off), y: vignette intensity (0 off), z: vignette radius, w: vignette smoothness
  vignette: vec4<f32>,  // rgb: vignette colour, w: chromatic aberration at the corners (uv units, 0 off)
  grain: vec4<f32>,     // x: intensity (0 off), y: size in pixels, z: seed (0..1)
  flash: vec4<f32>,     // rgb: flash colour, w: amount (0 off)
  extra: vec4<f32>,     // x: 1 with god rays, y: 1 when the lit image's alpha is the tone-mapped fraction (untonemapped sprites)
};
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var linear_sampler: sampler;
@group(0) @binding(2) var<uniform> post: Post;
@group(0) @binding(3) var luminance: texture_2d<f32>;
@group(0) @binding(4) var bloom_tex: texture_2d<f32>;
@group(0) @binding(5) var grade_lut: texture_3d<f32>;
@group(0) @binding(6) var god_rays: texture_2d<f32>;

fn exposure() -> f32 {
  if (post.exposure.y > 1.5) {
    return exp2(textureLoad(luminance, vec2<i32>(0), 0).r);
  }
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

// AgX, base look: the fit Blender 4, Filament and three.js's AgXToneMapping
// use. Linear sRGB in and out; the curve works in an inset Rec.2020 log
// space from -12.47 to +4.03 EV around middle grey.
fn agx_contrast(x: vec3<f32>) -> vec3<f32> {
  let x2 = x * x;
  let x4 = x2 * x2;
  return 15.5 * x4 * x2 - 40.14 * x4 * x + 31.96 * x4 - 6.868 * x2 * x + 0.4298 * x2 + 0.1191 * x - 0.00232;
}

fn agx(x: vec3<f32>) -> vec3<f32> {
  let srgb_to_2020 = mat3x3<f32>(
    vec3<f32>(0.6274, 0.0691, 0.0164),
    vec3<f32>(0.3293, 0.9195, 0.0880),
    vec3<f32>(0.0433, 0.0113, 0.8956));
  let rec2020_to_srgb = mat3x3<f32>(
    vec3<f32>(1.6605, -0.1246, -0.0182),
    vec3<f32>(-0.5876, 1.1329, -0.1006),
    vec3<f32>(-0.0728, -0.0083, 1.1187));
  let inset = mat3x3<f32>(
    vec3<f32>(0.856627153315983, 0.137318972929847, 0.11189821299995),
    vec3<f32>(0.0951212405381588, 0.761241990602591, 0.0767994186031903),
    vec3<f32>(0.0482516061458583, 0.101439036467562, 0.811302368396859));
  let outset = mat3x3<f32>(
    vec3<f32>(1.1271005818144368, -0.1413297634984383, -0.14132976349843826),
    vec3<f32>(-0.11060664309660323, 1.157823702216272, -0.11060664309660294),
    vec3<f32>(-0.016493938717834573, -0.016493938717834257, 1.2519364065950405));
  let min_ev = -12.47393;
  let max_ev = 4.026069;
  var c = inset * (srgb_to_2020 * x);
  c = clamp((log2(max(c, vec3<f32>(1e-10))) - min_ev) / (max_ev - min_ev), vec3<f32>(0.0), vec3<f32>(1.0));
  c = outset * agx_contrast(c);
  c = pow(max(c, vec3<f32>(0.0)), vec3<f32>(2.2));
  return clamp(rec2020_to_srgb * c, vec3<f32>(0.0), vec3<f32>(1.0));
}

fn tonemapped(uv: vec2<f32>) -> vec3<f32> {
  let curved = curve(uv);
  if (post.extra.y < 0.5) { return curved; }
  // Untonemapped sprites (sprite.wgsl `fs_untonemapped`) left the alpha at
  // the fraction of the pixel to tone map: the rest is their colour as
  // given, without exposure, bloom or the curve.
  let lit = textureSampleLevel(source, linear_sampler, uv, 0.0);
  return mix(clamp(lit.rgb, vec3<f32>(0.0), vec3<f32>(1.0)), curved, clamp(lit.a, 0.0, 1.0));
}

// The lit image exposed, with bloom and god rays, through the tone curve.
fn curve(uv: vec2<f32>) -> vec3<f32> {
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
  if (post.mode.x > 1.5) { return agx(c); }
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

// One tap of the 13-tap downsample: with untonemapped sprites (extra.y)
// the lit image's colour weighted by its alpha, the tone-mapped fraction,
// so those sprites neither bloom nor dim the bloom round them.
fn bloom_tap(uv: vec2<f32>) -> vec3<f32> {
  let s = textureSampleLevel(source, linear_sampler, uv, 0.0);
  if (post.extra.y > 0.5) { return s.rgb * clamp(s.a, 0.0, 1.0); }
  return s.rgb;
}

// The 13-tap downsample (Jimenez, "Next generation post processing in Call of Duty").
fn downsample13(uv: vec2<f32>) -> vec3<f32> {
  let t = 1.0 / vec2<f32>(textureDimensions(source));
  let s = bloom_tap(uv);
  let a = bloom_tap(uv + t * vec2<f32>(-2.0, -2.0));
  let b = bloom_tap(uv + t * vec2<f32>(0.0, -2.0));
  let c = bloom_tap(uv + t * vec2<f32>(2.0, -2.0));
  let d = bloom_tap(uv + t * vec2<f32>(-2.0, 0.0));
  let f = bloom_tap(uv + t * vec2<f32>(2.0, 0.0));
  let g = bloom_tap(uv + t * vec2<f32>(-2.0, 2.0));
  let h = bloom_tap(uv + t * vec2<f32>(0.0, 2.0));
  let i = bloom_tap(uv + t * vec2<f32>(2.0, 2.0));
  let j = bloom_tap(uv + t * vec2<f32>(-1.0, -1.0));
  let k = bloom_tap(uv + t * vec2<f32>(1.0, -1.0));
  let l = bloom_tap(uv + t * vec2<f32>(-1.0, 1.0));
  let m = bloom_tap(uv + t * vec2<f32>(1.0, 1.0));
  return s * 0.125 + (a + c + g + i) * 0.03125 + (b + d + f + h) * 0.0625 + (j + k + l + m) * 0.125;
}

@fragment
fn fs_bloom_prefilter(in: VsOut) -> @location(0) vec4<f32> {
  var out = vec3<f32>(0.0);
  if (post.mode.z > 0.5) {
    let c = downsample13(in.uv) * exposure();
    // (With untonemapped sprites the taps are weighted by the lit image's
    // alpha, `bloom_tap`, so they never bloom.)
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
