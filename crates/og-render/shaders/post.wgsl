// Exposure, bloom and the tonemap. Prepended by fullscreen.wgsl.
//
// Exposure is either manual (2^ev) or metered: the 1x1 log2-luminance texel
// is brought to its key (fullscreen.wgsl metered_key), plus compensation,
// clamped. Bloom: a prefilter (exposure, soft threshold) into the first level of a half-resolution chain,
// 13-tap downsamples, then tent upsamples added back level by level; the post
// pass adds the top level. `fs` writes linear values for an sRGB output (the
// hardware encodes); `fs_srgb_encode` encodes in the shader for a non-sRGB
// output (web canvases).
struct Post {
  exposure: vec4<f32>,  // x: manual ev, y: manual (0), auto (1) or adapted (2: `luminance` holds the EV), z: min ev, w: max ev
  bloom: vec4<f32>,     // x: threshold, y: knee, z: intensity (0: off), w: compensation
  mode: vec4<f32>,      // x: tonemap (0 ACES, 1 none, 2 AgX, 3 Khronos PBR Neutral), y: upsample radius in texels, z: 1 when the lit image feeds bloom, w: 1 when the glow buffer does
  grade: vec4<f32>,     // x: LUT intensity (0 off), y: vignette intensity (0 off), z: vignette radius, w: vignette smoothness
  vignette: vec4<f32>,  // rgb: vignette colour, w: chromatic aberration at the corners (uv units, 0 off)
  grain: vec4<f32>,     // x: intensity (0 off), y: size in pixels, z: seed (0..1)
  flash: vec4<f32>,     // rgb: flash colour, w: amount (0 off)
  extra: vec4<f32>,     // x: 1 with god rays, y: 1 when the lit image's alpha is the tone-mapped fraction (untonemapped sprites), z: 1 with sharpening, w: its strength
  unreal: vec4<f32>,    // x: 1 for three.js UnrealBloomPass bloom, y: its radius, z: the first bloom level's width, w: the contrast its high pass sees
  hdr: vec4<f32>,       // the HDR curves (post/display.rs `curve`): x headroom (0 SDR), y ACES and z AgX input scales
  flare: vec4<f32>,     // the lens flare (lens_flare.wgsl): x intensity (0 off), y 1 with the starburst, z its turn with the camera
};
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var linear_sampler: sampler;
@group(0) @binding(2) var<uniform> post: Post;
@group(0) @binding(3) var luminance: texture_2d<f32>;
@group(0) @binding(4) var bloom_tex: texture_2d<f32>;
@group(0) @binding(5) var grade_lut: texture_3d<f32>;
@group(0) @binding(6) var god_rays: texture_2d<f32>;
@group(0) @binding(7) var flare_tex: texture_2d<f32>;

fn exposure() -> f32 {
  return exposure_from(post.exposure, post.bloom.w, luminance);
}

fn tonemapped(uv: vec2<f32>) -> vec3<f32> {
  let curved = curve(uv);
  if (post.extra.y < 0.5) { return curved; }
  // Untonemapped sprites (sprite.wgsl `fs_untonemapped`) left the alpha at
  // the fraction of the pixel to tone map: the rest is their colour as
  // given, without exposure or the curve.
  let lit = sample_exact(source, linear_sampler, uv);
  var raw = lit.rgb;
  // The glow its neighbours spread over it (an authored LDR sky behind a
  // bright shot) still lands on it, curved on its own: without it the glow
  // shows only over tone-mapped pixels, outlining every mesh in front.
  if (post.bloom.z > 0.0) {
    let glow = textureSampleLevel(bloom_tex, linear_sampler, uv, 0.0).rgb * post.bloom.z;
    if (post.hdr.x > 0.0) { raw += tone_curve_hdr(glow, post.mode.x, post.hdr); } else { raw += tone_curve(glow, post.mode.x); }
  }
  return mix(clamp(raw, vec3<f32>(0.0), vec3<f32>(1.0)), curved, clamp(lit.a, 0.0, 1.0));
}

// Filament's starburst (colorGrading.mat `starburst`, google/filament
// 5ce4cbf19, Apache-2.0, third_party/lens-flare/filament/): the flare's
// halo (Filament: the whole flare) modulated by two copies of a random
// 256-texel row, read by the angle round the centre and turned in
// opposite directions with the camera, and left whole near the centre.
// Procedural here, no texture or asset: Filament fills its R8 row with
// 0.5 + 0.5 x uniform random values once and samples it linearly,
// repeating; `starburst_row` is that row's linear read, its values the
// engine's own hash (`grain_hash`).
fn starburst_row(u: f32) -> f32 {
  let x = u * 256.0 - 0.5;
  let i = floor(x);
  let a = 0.5 + 0.5 * grain_hash(vec2<u32>(u32(i32(i) & 255), 0u), 0x5eedu);
  let b = 0.5 + 0.5 * grain_hash(vec2<u32>(u32((i32(i) + 1) & 255), 0u), 0x5eedu);
  return mix(a, b, x - i);
}

fn starburst(uv: vec2<f32>) -> f32 {
  if (post.flare.y < 0.5) { return 1.0; }
  let offset = post.flare.z;
  let center = uv - vec2<f32>(0.5001);
  let d = max(length(center), 1e-6);
  let radial = atan2(center.y, center.x) / (2.0 * PI) + 0.5;
  let mask = starburst_row(fract(radial + offset)) * starburst_row(fract(radial - offset * 0.5));
  return clamp(mask + (1.0 - smoothstep(0.0, 0.3, d)), 0.0, 1.0);
}

// The lit image exposed, with bloom, the lens flare and god rays, through
// the tone curve (the HDR output's too).
fn curve(uv: vec2<f32>) -> vec3<f32> {
  var lit: vec3<f32>;
  if (post.vignette.w > 0.0) {
    // Chromatic aberration: red and blue pulled apart, growing towards the corners.
    let d = uv - 0.5;
    let off = d * length(d) * 2.0 * post.vignette.w;
    lit = vec3<f32>(
      textureSampleLevel(source, linear_sampler, uv - off, 0.0).r,
      sample_exact(source, linear_sampler, uv).g,
      textureSampleLevel(source, linear_sampler, uv + off, 0.0).b);
  } else {
    lit = sample_exact(source, linear_sampler, uv).rgb;
  }
  var c = lit * exposure();
  if (post.bloom.z > 0.0) {
    c += textureSampleLevel(bloom_tex, linear_sampler, uv, 0.0).rgb * post.bloom.z;
  }
  // Exposed already, as bloom is (lens_flare.wgsl).
  if (post.flare.x > 0.0) {
    // The starburst on the halo's share (its luma in the alpha) alone:
    // its rays from the centre would draw lines across the ghosts.
    let f = textureSampleLevel(flare_tex, linear_sampler, uv, 0.0);
    let halo = clamp(f.a / max(dot(f.rgb, LUMA), 1e-6), 0.0, 1.0);
    c += f.rgb * (post.flare.x * mix(1.0, starburst(uv), halo));
  }
  if (post.extra.x > 0.5) {
    c += textureSampleLevel(god_rays, linear_sampler, uv, 0.0).rgb * exposure();
  }
  if (post.hdr.x > 0.0) { return tone_curve_hdr(c, post.mode.x, post.hdr); }
  return tone_curve(c, post.mode.x);
}

fn grain_hash(p: vec2<u32>, seed: u32) -> f32 {
  var h = p.x * 1664525u + p.y * 1013904223u + seed * 2654435761u;
  h = (h ^ (h >> 16u)) * 2246822519u;
  h = (h ^ (h >> 13u)) * 3266489917u;
  h = h ^ (h >> 16u);
  return f32(h >> 8u) / 16777216.0;
}

// The graded display colour (linear 0..1): tone curve, LUT, vignette.
fn graded(uv: vec2<f32>) -> vec3<f32> {
  var c = tonemapped(uv);
  if (post.grade.x > 0.0) {
    let highlights = select(vec3<f32>(0.0), max(c - 1.0, vec3<f32>(0.0)), post.hdr.x > 0.0);
    let n = f32(textureDimensions(grade_lut).x);
    let e = clamp(srgb_encode(clamp(c, vec3<f32>(0.0), vec3<f32>(1.0))), vec3<f32>(0.0), vec3<f32>(1.0));
    let graded = textureSampleLevel(grade_lut, linear_sampler, e * ((n - 1.0) / n) + 0.5 / n, 0.0).rgb;
    c = mix(c, srgb_decode(graded) + highlights, post.grade.x);
  }
  if (post.grade.y > 0.0) {
    let d = length((uv - 0.5) * 2.0) / 1.41421356;
    let v = smoothstep(post.grade.z, post.grade.z + max(post.grade.w, 1e-3), d);
    c = mix(c, post.vignette.rgb, v * post.grade.y);
  }
  return c;
}

// Contrast-adaptive sharpening (PostEffects.sharpen and RenderDesc.taa_sharpen
// fused into Tonemap, post.extra.w its strength): evaluate graded(uv) once for
// the centre and measure local contrast from the 4 cardinal neighbours of
// `source` in perceptual luma space via direct integer loads.
fn cas_luma(px: vec2<i32>, last: vec2<i32>, exp_gain: f32) -> f32 {
  let l = max(luma(textureLoad(source, clamp(px, vec2<i32>(0), last), 0).rgb) * exp_gain, 0.0);
  return sqrt(l / (1.0 + l));
}

fn sharpened(uv: vec2<f32>, sharpen: bool) -> vec3<f32> {
  let center = graded(uv);
  if (!sharpen) { return center; }
  let dims = vec2<i32>(textureDimensions(source));
  let last = dims - 1;
  let px = clamp(vec2<i32>(uv * vec2<f32>(dims)), vec2<i32>(0), last);
  let exp_gain = exposure();
  let c_l = cas_luma(px, last, exp_gain);
  let n_l = cas_luma(px + vec2<i32>(0, -1), last, exp_gain);
  let w_l = cas_luma(px + vec2<i32>(-1, 0), last, exp_gain);
  let e_l = cas_luma(px + vec2<i32>(1, 0), last, exp_gain);
  let s_l = cas_luma(px + vec2<i32>(0, 1), last, exp_gain);
  let mn = min(c_l, min(min(n_l, s_l), min(w_l, e_l)));
  let mx = max(c_l, max(max(n_l, s_l), max(w_l, e_l)));
  let amp = sqrt(clamp(min(mn, 2.0 - mx) / max(mx, 1e-4), 0.0, 1.0));
  let w = amp * (-1.0 / mix(8.0, 5.0, post.extra.w));
  let sharp_l = clamp((c_l + (n_l + w_l + e_l + s_l) * w) / (1.0 + 4.0 * w), 0.0, 1.0);
  let delta = sharp_l - c_l;
  let highlights = select(vec3<f32>(0.0), max(center - 1.0, vec3<f32>(0.0)), post.hdr.x > 0.0);
  let enc = srgb_encode(clamp(center, vec3<f32>(0.0), vec3<f32>(1.0)));
  return srgb_decode(clamp(enc + vec3<f32>(delta), vec3<f32>(0.0), vec3<f32>(1.0))) + highlights;
}

// Display-linear colour (paper-white units in HDR): grading, vignette, sharpening, grain, flash.
fn finished(uv: vec2<f32>, pos: vec2<f32>) -> vec3<f32> {
  var c: vec3<f32>;
  c = sharpened(uv, post.extra.z > 0.5);
  if (post.grain.x > 0.0) {
    let cell = vec2<u32>(pos / max(post.grain.y, 1.0));
    let g = grain_hash(cell, u32(post.grain.z * 16777216.0)) - 0.5;
    // Strongest in the mid-tones.
    var l = dot(c, LUMA);
    if (post.hdr.x > 0.0) { l = clamp(l, 0.0, 1.0); }
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
fn fs_srgb_dither(in: VsOut) -> @location(0) vec4<f32> {
  var c = finished(in.uv, in.pos.xy);
  // 1-LSB perceptual triangular IGN dither for 8-bit hardware sRGB SDR outputs;
  // skip under HDR or Tonemap::None (mode.x == 1) so authored exact 2D colours
  // stay bit-for-bit intact.
  if (post.hdr.x <= 0.0 && abs(post.mode.x - 1.0) > 0.5) {
    let enc = srgb_encode(clamp(c, vec3<f32>(0.0), vec3<f32>(1.0)));
    let dither = (ign(in.pos.xy) + ign(in.pos.xy + vec2<f32>(17.0, 31.0)) - 1.0) / 255.0;
    c = srgb_decode(clamp(enc + vec3<f32>(dither), vec3<f32>(0.0), vec3<f32>(1.0)));
  }
  return vec4<f32>(c, 1.0);
}

@fragment
fn fs_srgb_encode(in: VsOut) -> @location(0) vec4<f32> {
  var enc = srgb_encode(clamp(finished(in.uv, in.pos.xy), vec3<f32>(0.0), vec3<f32>(1.0)));
  // 1-LSB triangular IGN dither to break 8-bit sky and shadow banding; skip
  // under Tonemap::None so authored exact 2D colours stay intact.
  if (abs(post.mode.x - 1.0) > 0.5) {
    let dither = (ign(in.pos.xy) + ign(in.pos.xy + vec2<f32>(17.0, 31.0)) - 1.0) / 255.0;
    enc = clamp(enc + vec3<f32>(dither), vec3<f32>(0.0), vec3<f32>(1.0));
  }
  return vec4<f32>(enc, 1.0);
}

// The bloom chain's source size (bloom_chain.wgsl): each bloom level is a
// texture of its own, so its size is the view's.
fn bloom_source_size() -> vec2<f32> {
  return vec2<f32>(textureDimensions(source));
}

// One tap of the 13-tap downsample (bloom_chain.wgsl): with untonemapped sprites (extra.y)
// the lit image's colour weighted by its alpha, the tone-mapped fraction,
// so those sprites neither bloom nor dim the bloom round them.
fn bloom_tap(uv: vec2<f32>) -> vec3<f32> {
  let s = textureSampleLevel(source, linear_sampler, uv, 0.0);
  if (post.extra.y > 0.5) { return s.rgb * clamp(s.a, 0.0, 1.0); }
  return s.rgb;
}

// three.js UnrealBloomPass (post.unreal.x). Mip i (0..4) is a Gaussian of
// kernel radius 3 + 2i (sigma the same, taps 0..radius-1 each way) in its own
// texels, run on mip i-1 (mip 0: the high pass at half resolution). Level i
// stores mip i times its mix weight, mix(f, 1.2 - f, radius), f = 1 - 0.2 i;
// the upsamples then add the levels (bilinear, as the composite reads them).
fn unreal_weight(level: i32) -> f32 {
  let f = 1.0 - 0.2 * f32(level);
  return mix(f, 1.2 - f, post.unreal.y);
}

fn unreal_tap(i: i32, k: i32) -> f32 {
  let x = f32(i);
  let r = f32(k);
  return 0.39894 * exp(-0.5 * x * x / (r * r)) / r;
}

// LuminosityHighPassShader: the colour where its luminance passes
// smoothstep(threshold, threshold + smoothWidth), else black. `lit` is the
// lit colour before exposure: graded first (0.18 (c / 0.18)^contrast per
// channel), then exposed, so the threshold stays an exposed brightness.
fn unreal_high_pass(lit: vec3<f32>) -> vec3<f32> {
  let c = 0.18 * pow(max(lit, vec3<f32>(0.0)) / 0.18, vec3<f32>(post.unreal.w)) * exposure();
  let v = luma(c);
  return c * smoothstep(post.bloom.x, post.bloom.x + post.bloom.y, v);
}

// The Gaussian of mip `level` at `uv`, its taps `texel` apart; strided to at
// most 7x7 taps (49 instead of up to 441 at level 4) while preserving the
// exact Gaussian sigma through bilinear filtering.
fn unreal_blur(uv: vec2<f32>, texel: vec2<f32>, level: i32) -> vec3<f32> {
  let k = 3 + 2 * level;
  let stride = max(1, (k - 1) / 3);
  let m = (k - 1) / stride;
  var sum = vec3<f32>(0.0);
  var norm = 0.0;
  for (var j = -m; j <= m; j++) {
    let sj = j * stride;
    let wj = unreal_tap(abs(sj), k);
    for (var i = -m; i <= m; i++) {
      let si = i * stride;
      let w = wj * unreal_tap(abs(si), k);
      let p = uv + texel * vec2<f32>(f32(si), f32(sj));
      var c: vec3<f32>;
      if (level == 0) {
        c = unreal_high_pass(bloom_tap(p));
      } else {
        c = textureSampleLevel(source, linear_sampler, p, 0.0).rgb;
      }
      sum += c * w;
      norm += w;
    }
  }
  return sum / norm;
}

@fragment
fn fs_bloom_prefilter(in: VsOut) -> @location(0) vec4<f32> {
  var out = vec3<f32>(0.0);
  if (post.mode.z > 0.5 && post.unreal.x > 0.5) {
    let half_size = max(textureDimensions(source) / 2u, vec2<u32>(1u));
    out = unreal_blur(in.uv, 1.0 / vec2<f32>(half_size), 0) * unreal_weight(0);
  } else if (post.mode.z > 0.5) {
    // Mip 0 from the lit image: with the Karis average, so a lone bright
    // pixel does not bloom into a block.
    let c = downsample13_karis(in.uv, exposure()) * exposure();
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
  if (post.mode.w > 0.0) {
    // The order-independent glow buffer joins without a threshold, scaled by
    // its relative intensity against the bloom composite weight.
    out += textureSampleLevel(bloom_tex, linear_sampler, in.uv, 0.0).rgb * exposure() * post.mode.w;
  }
  return vec4<f32>(out, 1.0);
}

@fragment
fn fs_bloom_down(in: VsOut) -> @location(0) vec4<f32> {
  if (post.unreal.x > 0.5) {
    let size = textureDimensions(source);
    let lv = i32(round(log2(post.unreal.z / f32(size.x))));
    if (lv >= 4) { return vec4<f32>(0.0, 0.0, 0.0, 1.0); }
    let texel = 1.0 / vec2<f32>(max(size / 2u, vec2<u32>(1u)));
    let c = unreal_blur(in.uv, texel, lv + 1) / unreal_weight(lv) * unreal_weight(lv + 1);
    return vec4<f32>(c, 1.0);
  }
  return vec4<f32>(downsample13(in.uv), 1.0);
}

// The tent upsample (bloom_chain.wgsl), added onto the larger level by blending.
@fragment
fn fs_bloom_up(in: VsOut) -> @location(0) vec4<f32> {
  if (post.unreal.x > 0.5) {
    return vec4<f32>(textureSampleLevel(source, linear_sampler, in.uv, 0.0).rgb, 1.0);
  }
  return vec4<f32>(bloom_tent(in.uv, post.mode.y), 1.0);
}
