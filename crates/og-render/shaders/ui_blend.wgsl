// What the UI's blends share (src/ui/pass.rs), after ui_common.wgsl: the
// group 2 a shape or a layer's composite reads what is under it from,
// and Compositing and Blending 1's blend modes over it.

struct BackdropParams {
  // x: 1 when `behind` holds sRGB-encoded values; y: 1 when it holds
  // premultiplied colour with its alpha (a layer's target: a group's
  // backdrop starts transparent); w: the pyramid's levels.
  params: vec4<f32>,
};
// The blur pyramid of what is under a backdrop (src/glass.rs), and a copy
// of the target as drawn so far (the blends that read it).
@group(2) @binding(0) var pyramid_tex: texture_2d<f32>;
@group(2) @binding(1) var behind_tex: texture_2d<f32>;
@group(2) @binding(2) var backdrop_sampler: sampler;
@group(2) @binding(3) var<uniform> backdrop: BackdropParams;

// Hard light (Compositing and Blending 1 §10.1.6): multiply or screen by
// the source; overlay is it with source and backdrop swapped.
fn hard_light(b: f32, s: f32) -> f32 {
  return select(1.0 - 2.0 * (1.0 - s) * (1.0 - b), 2.0 * s * b, s <= 0.5);
}

// The separable modes (Compositing and Blending 1 §10.1), backdrop `b`
// and source `s` in 0..1; `mode` is UiBlend's code (src/ui/build.rs).
fn blend_channel(b: f32, s: f32, mode: u32) -> f32 {
  switch mode {
    case 1u: { return b * s; }
    case 2u: { return b + s - b * s; }
    case 4u: { return hard_light(s, b); }
    case 7u: { return min(b, s); }
    case 8u: { return max(b, s); }
    case 9u: {
      // Colour burn.
      if (b >= 1.0) {
        return 1.0;
      }
      if (s <= 0.0) {
        return 0.0;
      }
      return 1.0 - min(1.0, (1.0 - b) / s);
    }
    case 10u: { return hard_light(b, s); }
    case 11u: { return abs(b - s); }
    case 12u: { return b + s - 2.0 * b * s; }
    case 5u: {
      // Soft light (the W3C compositing formula).
      if (s <= 0.5) {
        return b - (1.0 - 2.0 * s) * b * (1.0 - b);
      }
      let d = select(sqrt(b), ((16.0 * b - 12.0) * b + 4.0) * b, b <= 0.25);
      return b + (2.0 * s - 1.0) * (d - b);
    }
    default: {
      // Colour dodge.
      if (b <= 0.0) {
        return 0.0;
      }
      if (s >= 1.0) {
        return 1.0;
      }
      return min(1.0, b / (1.0 - s));
    }
  }
}

// The non-separable modes' helpers (Compositing and Blending 1 §10.2).
fn lum(c: vec3<f32>) -> f32 {
  return dot(c, vec3<f32>(0.3, 0.59, 0.11));
}

fn clip_color(c: vec3<f32>) -> vec3<f32> {
  let l = lum(c);
  let n = min(c.r, min(c.g, c.b));
  let x = max(c.r, max(c.g, c.b));
  var o = c;
  if (n < 0.0) {
    o = l + (o - l) * l / max(l - n, 1e-6);
  }
  if (x > 1.0) {
    o = l + (o - l) * (1.0 - l) / max(x - l, 1e-6);
  }
  return o;
}

fn set_lum(c: vec3<f32>, l: f32) -> vec3<f32> {
  return clip_color(c + (l - lum(c)));
}

fn sat(c: vec3<f32>) -> f32 {
  return max(c.r, max(c.g, c.b)) - min(c.r, min(c.g, c.b));
}

// The spec's SetSat: the largest channel to `s`, the smallest to 0, the
// middle scaled between them.
fn set_sat(c: vec3<f32>, s: f32) -> vec3<f32> {
  let n = min(c.r, min(c.g, c.b));
  let x = max(c.r, max(c.g, c.b));
  if (x <= n) {
    return vec3<f32>(0.0);
  }
  return (c - n) * s / (x - n);
}

// Straight linear colour `rgb` at alpha `a` blended by `mode` (UiBlend's
// code, src/ui/build.rs) over what is under device position `position`,
// as the target's alpha blending lays it: Compositing and Blending 1 §5.2,
// Cs' = (1 - ab) Cs + ab B(Cb, Cs), so over a group's transparent
// backdrop the colour is itself. Blends run on sRGB-encoded values, as
// CSS does, and decode back to the linear light drawn here; plus-lighter
// (3) adds in linear light.
fn blend_over(rgb: vec3<f32>, a: f32, mode: u32, position: vec2<f32>) -> vec4<f32> {
  if (mode == 0u) {
    return finish_color(rgb, a);
  }
  let size = vec2<i32>(textureDimensions(behind_tex));
  let at = clamp(vec2<i32>(position), vec2<i32>(0), size - 1);
  let t = textureLoad(behind_tex, at, 0);
  var b = t.rgb;
  var ab = 1.0;
  if (backdrop.params.y > 0.5) {
    ab = clamp(t.a, 0.0, 1.0);
    b = t.rgb / max(t.a, 1e-6);
  }
  if (backdrop.params.x > 0.5) {
    b = decode(b);
  }
  if (mode == 3u) {
    return finish_color(b * ab + rgb, a);
  }
  let src = encode(clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0)));
  let bb = encode(clamp(b, vec3<f32>(0.0), vec3<f32>(1.0)));
  var r: vec3<f32>;
  switch mode {
    case 13u: { r = set_lum(set_sat(src, sat(bb)), lum(bb)); }
    case 14u: { r = set_lum(set_sat(bb, sat(src)), lum(bb)); }
    case 15u: { r = set_lum(src, lum(bb)); }
    case 16u: { r = set_lum(bb, lum(src)); }
    default: {
      r = vec3<f32>(blend_channel(bb.r, src.r, mode), blend_channel(bb.g, src.g, mode), blend_channel(bb.b, src.b, mode));
    }
  }
  r = mix(src, clamp(r, vec3<f32>(0.0), vec3<f32>(1.0)), ab);
  return finish_color(decode(r), a);
}
