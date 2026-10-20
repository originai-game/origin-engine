// The stylisation passes over the tonemapped image (stylise.rs, the IDL's
// Stylise): after the tonemap, at the logical resolution, each a fullscreen
// pass from one LDR target into the next (or the output, encoding to sRGB
// in the shader where the output is not an sRGB format):
//
//   paint     painterly smoothing (a Kuwahara filter, its sectors turned
//             along the image's edges by `anisotropy`), pigment pooling at
//             the edges, colour bleeding, a canvas laid over the frame;
//   lines     cross-hatching from the tone, halftone dots on a rotated
//             grid, ink lines on luminance edges;
//   quantize  posterised channels and/or the nearest palette colour, mixed
//             by an ordered (Bayer), blue-noise or interleaved-gradient
//             dither;
//   upscale   the logical frame scaled up by a whole number with nearest
//             filtering, centred in the output with the background round it.
//
// Prepended by fullscreen.wgsl. Every pass binds the same layout.
struct Stylise {
  screen: vec4<f32>,          // logical w, h, 1/w, 1/h
  output: vec4<f32>,          // output w, h, the whole-number upscale, 0
  offset: vec4<f32>,          // xy: where the scaled frame starts in the output, z: 1 with a canvas, w: palette colours
  background: vec4<f32>,      // the display colour round the frame (linear)
  quantize: vec4<f32>,        // x: tones per channel (0: none), y: dither (0 none, 2/4/8 Bayer, -1 interleaved gradient, -2 blue noise), z: strength in steps, w: cell size
  halftone: vec4<f32>,        // x: strength, y: pitch, z: angle (rad)
  halftone_color: vec4<f32>,
  hatching: vec4<f32>,        // x: strength, y: spacing, z: levels, w: angle (rad)
  hatching_color: vec4<f32>,
  paint: vec4<f32>,           // x: radius, y: anisotropy, z: edge darkening, w: bleed
  canvas: vec4<f32>,          // x: strength, y: texels per logical pixel
  ink: vec4<f32>,             // x: 1 on, y: threshold, z: width
  ink_color: vec4<f32>,
};
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var clamp_sampler: sampler;
@group(0) @binding(2) var<uniform> st: Stylise;
@group(0) @binding(3) var palette: texture_2d<f32>;   // 256 x 1 sRGB colours, linear when sampled
@group(0) @binding(4) var canvas: texture_2d<f32>;
@group(0) @binding(5) var repeat_sampler: sampler;
@group(0) @binding(6) var blue_noise: texture_2d<f32>;  // 64 x 64 thresholds (blue_noise.rs), R8

fn srgb_encode(c: vec3<f32>) -> vec3<f32> {
  let lo = c * 12.92;
  let hi = 1.055 * pow(max(c, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.4)) - 0.055;
  return select(hi, lo, c <= vec3<f32>(0.0031308));
}

fn srgb_decode(c: vec3<f32>) -> vec3<f32> {
  let lo = c / 12.92;
  let hi = pow((c + 0.055) / 1.055, vec3<f32>(2.4));
  return select(hi, lo, c <= vec3<f32>(0.04045));
}

fn load(px: vec2<i32>) -> vec3<f32> {
  let size = vec2<i32>(textureDimensions(source));
  return textureLoad(source, clamp(px, vec2<i32>(0), size - 1), 0).rgb;
}

fn luma(c: vec3<f32>) -> f32 {
  return dot(c, LUMA);
}

// The tone as displayed (sRGB-encoded luminance), which the print looks
// quantise and stroke.
fn tone(c: vec3<f32>) -> f32 {
  return luma(srgb_encode(c));
}

// --- paint ---------------------------------------------------------------

// The image gradient at `px` (Sobel over the encoded luminance).
fn gradient(px: vec2<i32>) -> vec2<f32> {
  let tl = tone(load(px + vec2<i32>(-1, -1)));
  let t = tone(load(px + vec2<i32>(0, -1)));
  let tr = tone(load(px + vec2<i32>(1, -1)));
  let l = tone(load(px + vec2<i32>(-1, 0)));
  let r = tone(load(px + vec2<i32>(1, 0)));
  let bl = tone(load(px + vec2<i32>(-1, 1)));
  let b = tone(load(px + vec2<i32>(0, 1)));
  let br = tone(load(px + vec2<i32>(1, 1)));
  return vec2<f32>((tr + 2.0 * r + br) - (tl + 2.0 * l + bl), (bl + 2.0 * b + br) - (tl + 2.0 * t + tr));
}

// A Kuwahara filter: of the four sectors round the pixel, the mean of the
// one with the least variance, so flat regions merge into strokes while
// edges stay sharp. `anisotropy` turns and stretches the sectors along the
// edge through the pixel.
fn kuwahara(px: vec2<i32>, radius: f32, anisotropy: f32) -> vec3<f32> {
  let r = i32(ceil(radius));
  let g = gradient(px);
  let mag = length(g);
  var angle = 0.0;
  var stretch = 1.0;
  if (anisotropy > 0.0 && mag > 1e-3) {
    // Strokes run along the edge: perpendicular to the gradient.
    angle = atan2(g.y, g.x) + 1.5707963;
    stretch = 1.0 + anisotropy * 2.0;
  }
  let ca = cos(angle);
  let sa = sin(angle);
  var best = vec3<f32>(0.0);
  var best_var = 1e9;
  for (var s = 0; s < 4; s++) {
    let sx = select(-1, 1, (s & 1) == 1);
    let sy = select(-1, 1, (s & 2) == 2);
    var sum = vec3<f32>(0.0);
    var sum2 = vec3<f32>(0.0);
    var n = 0.0;
    for (var y = 0; y <= r; y++) {
      for (var x = 0; x <= r; x++) {
        // The sector's offset, turned and stretched along the stroke.
        let o = vec2<f32>(f32(x * sx), f32(y * sy));
        let t = vec2<f32>(o.x * ca - o.y * sa, o.x * sa + o.y * ca);
        let e = vec2<f32>(t.x * stretch, t.y / stretch);
        if (length(e) > radius * stretch) { continue; }
        let c = load(px + vec2<i32>(round(e)));
        sum += c;
        sum2 += c * c;
        n += 1.0;
      }
    }
    let mean = sum / max(n, 1.0);
    let v = sum2 / max(n, 1.0) - mean * mean;
    let variance = v.r + v.g + v.b;
    if (variance < best_var) {
      best_var = variance;
      best = mean;
    }
  }
  return best;
}

fn paint(in: VsOut) -> vec3<f32> {
  let px = vec2<i32>(in.pos.xy);
  var c = load(px);
  if (st.paint.x > 0.0) {
    c = kuwahara(px, st.paint.x, st.paint.y);
  }
  if (st.paint.w > 0.0) {
    // Wet in wet: colour from a ring two radii out bleeds in.
    let d = i32(max(st.paint.x, 1.0)) * 2;
    let around = (load(px + vec2<i32>(d, 0)) + load(px + vec2<i32>(-d, 0)) + load(px + vec2<i32>(0, d)) + load(px + vec2<i32>(0, -d))) * 0.25;
    c = mix(c, around, st.paint.w * 0.5);
  }
  if (st.paint.z > 0.0) {
    // Pigment pools where the colour changes: darker at the edges of regions.
    let g = length(gradient(px));
    c *= 1.0 - st.paint.z * clamp(g * 1.5, 0.0, 1.0) * 0.6;
  }
  if (st.offset.z > 0.5 && st.canvas.x > 0.0) {
    let size = vec2<f32>(textureDimensions(canvas));
    let uv = in.pos.xy * st.canvas.y / size;
    let grain = textureSample(canvas, repeat_sampler, uv).r;
    c *= mix(1.0, grain * 0.5 + 0.5, st.canvas.x);
  }
  return c;
}

@fragment
fn fs_paint(in: VsOut) -> @location(0) vec4<f32> {
  return vec4<f32>(paint(in), 1.0);
}

@fragment
fn fs_paint_srgb(in: VsOut) -> @location(0) vec4<f32> {
  return vec4<f32>(srgb_encode(clamp(paint(in), vec3<f32>(0.0), vec3<f32>(1.0))), 1.0);
}

// --- lines ---------------------------------------------------------------

// A stroke set: lines at `angle` with `spacing`, 1 on the line.
fn stroke(p: vec2<f32>, angle: f32, spacing: f32, width: f32) -> f32 {
  let d = p.x * cos(angle) + p.y * sin(angle);
  let f = abs(fract(d / spacing) - 0.5) * spacing;
  return 1.0 - smoothstep(width * 0.5, width * 0.5 + 0.75, f);
}

fn lines(in: VsOut) -> vec3<f32> {
  let px = vec2<i32>(in.pos.xy);
  let p = in.pos.xy;
  var c = load(px);
  let t = tone(c);
  if (st.hatching.x > 0.0) {
    // Darker tones add stroke sets, each turned a little further, over a
    // paper-light ground.
    let levels = max(st.hatching.z, 1.0);
    var ink = 0.0;
    for (var k = 0.0; k < levels; k += 1.0) {
      let threshold = 1.0 - (k + 1.0) / (levels + 1.0);
      if (t < threshold) {
        let turn = st.hatching.w + k * (1.5707963 / max(levels - 1.0, 1.0)) + select(0.0, 1.5707963, (i32(k) & 1) == 1);
        ink = max(ink, stroke(p, turn, st.hatching.y, 1.0) * clamp((threshold - t) * levels * 2.0, 0.3, 1.0));
      }
    }
    let ground = mix(c, vec3<f32>(1.0), st.hatching.x * 0.85);
    c = mix(ground, st.hatching_color.rgb, ink * st.hatching.x);
  }
  if (st.halftone.x > 0.0) {
    // Dots on a rotated grid, larger where the image is darker.
    let a = st.halftone.z;
    let r = vec2<f32>(p.x * cos(a) - p.y * sin(a), p.x * sin(a) + p.y * cos(a)) / st.halftone.y;
    let cell = fract(r) - 0.5;
    let radius = (1.0 - t) * 0.72;
    let dot_cover = 1.0 - smoothstep(radius - 0.08, radius + 0.08, length(cell));
    let paper = mix(c, vec3<f32>(1.0), st.halftone.x);
    c = mix(paper, st.halftone_color.rgb, dot_cover * st.halftone.x);
  }
  if (st.ink.x > 0.5) {
    // Luminance edges, widened by the max over the line's width.
    let w = max(i32(round(st.ink.z * 0.5)), 0);
    var edge = 0.0;
    for (var y = -w; y <= w; y++) {
      for (var x = -w; x <= w; x++) {
        edge = max(edge, length(gradient(px + vec2<i32>(x, y))));
      }
    }
    let k = smoothstep(st.ink.y, st.ink.y + 0.1, edge * 0.25);
    c = mix(c, st.ink_color.rgb, k);
  }
  return c;
}

@fragment
fn fs_lines(in: VsOut) -> @location(0) vec4<f32> {
  return vec4<f32>(lines(in), 1.0);
}

@fragment
fn fs_lines_srgb(in: VsOut) -> @location(0) vec4<f32> {
  return vec4<f32>(srgb_encode(clamp(lines(in), vec3<f32>(0.0), vec3<f32>(1.0))), 1.0);
}

// --- quantize ------------------------------------------------------------

fn bayer(px: vec2<u32>, n: u32) -> f32 {
  // The ordered dither threshold of an n x n Bayer matrix (n 2, 4, 8),
  // bit-interleaved as the recursive definition gives it, in 0..1.
  var x = px.x % n;
  var y = px.y % n;
  var v = 0u;
  var bits = 0u;
  var size = n;
  while (size > 1u) {
    size = size >> 1u;
    let bx = (x / size) & 1u;
    let by = (y / size) & 1u;
    v = (v << 2u) | ((bx ^ by) << 1u) | by;
    bits += 2u;
  }
  return (f32(v) + 0.5) / f32(1u << bits);
}

// Interleaved gradient noise (Jimenez): blue-noise-like without a texture.
fn ign(p: vec2<f32>) -> f32 {
  return fract(52.9829189 * fract(0.06711056 * p.x + 0.00583715 * p.y));
}

// The void-and-cluster tile, repeated: byte v is the threshold (v + 0.5) / 256.
fn blue(cell: vec2<u32>) -> f32 {
  let v = textureLoad(blue_noise, vec2<i32>(cell % vec2<u32>(64u)), 0).r;
  return (round(v * 255.0) + 0.5) / 256.0;
}

fn dither_at(pos: vec2<f32>) -> f32 {
  let cell = vec2<u32>(pos / max(st.quantize.w, 1.0));
  if (st.quantize.y < -1.5) { return blue(cell); }
  if (st.quantize.y < 0.0) { return ign(vec2<f32>(cell)); }
  if (st.quantize.y >= 2.0) { return bayer(cell, u32(st.quantize.y)); }
  return 0.5;
}

// Oklab, for the nearest palette colour by perceived difference.
fn oklab(c: vec3<f32>) -> vec3<f32> {
  let l = 0.4122214708 * c.r + 0.5363325363 * c.g + 0.0514459929 * c.b;
  let m = 0.2119034982 * c.r + 0.6806995451 * c.g + 0.1073969566 * c.b;
  let s = 0.0883024619 * c.r + 0.2817188376 * c.g + 0.6299787005 * c.b;
  let l3 = pow(max(l, 0.0), 1.0 / 3.0);
  let m3 = pow(max(m, 0.0), 1.0 / 3.0);
  let s3 = pow(max(s, 0.0), 1.0 / 3.0);
  return vec3<f32>(
    0.2104542553 * l3 + 0.7936177850 * m3 - 0.0040720468 * s3,
    1.9779984951 * l3 - 2.4285922050 * m3 + 0.4505937099 * s3,
    0.0259040371 * l3 + 0.7827717662 * m3 - 0.8086757660 * s3);
}

fn quantize(in: VsOut) -> vec3<f32> {
  let px = vec2<i32>(in.pos.xy);
  var c = load(px);
  let d = (dither_at(in.pos.xy) - 0.5) * st.quantize.z;
  if (st.quantize.x >= 2.0) {
    // Steps in display space, the dither spread across one step.
    let n = st.quantize.x - 1.0;
    let e = srgb_encode(clamp(c, vec3<f32>(0.0), vec3<f32>(1.0)));
    c = srgb_decode(clamp(floor(e * n + 0.5 + d) / n, vec3<f32>(0.0), vec3<f32>(1.0)));
  }
  let count = i32(st.offset.w);
  if (count > 0) {
    // The nearest palette colour of the dithered colour (the dither moves
    // it by a share of a palette step, so neighbours alternate).
    let shifted = clamp(c + vec3<f32>(d * 0.12), vec3<f32>(0.0), vec3<f32>(1.0));
    let want = oklab(shifted);
    var best = c;
    var best_d = 1e9;
    for (var i = 0; i < count; i++) {
      let p = textureLoad(palette, vec2<i32>(i, 0), 0).rgb;
      let delta = oklab(p) - want;
      let dist = dot(delta, delta);
      if (dist < best_d) {
        best_d = dist;
        best = p;
      }
    }
    c = best;
  }
  return c;
}

@fragment
fn fs_quantize(in: VsOut) -> @location(0) vec4<f32> {
  return vec4<f32>(quantize(in), 1.0);
}

@fragment
fn fs_quantize_srgb(in: VsOut) -> @location(0) vec4<f32> {
  return vec4<f32>(srgb_encode(clamp(quantize(in), vec3<f32>(0.0), vec3<f32>(1.0))), 1.0);
}

// --- upscale -------------------------------------------------------------

fn upscale(in: VsOut) -> vec3<f32> {
  let scale = max(st.output.z, 1.0);
  let p = (in.pos.xy - st.offset.xy) / scale;
  let size = vec2<f32>(textureDimensions(source));
  if (p.x < 0.0 || p.y < 0.0 || p.x >= size.x || p.y >= size.y) {
    return st.background.rgb;
  }
  return textureLoad(source, vec2<i32>(floor(p)), 0).rgb;
}

@fragment
fn fs_upscale(in: VsOut) -> @location(0) vec4<f32> {
  return vec4<f32>(upscale(in), 1.0);
}

@fragment
fn fs_upscale_srgb(in: VsOut) -> @location(0) vec4<f32> {
  return vec4<f32>(srgb_encode(clamp(upscale(in), vec3<f32>(0.0), vec3<f32>(1.0))), 1.0);
}
