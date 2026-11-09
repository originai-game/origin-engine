// Compositing and Blending 1's blend modes (src/ui/pass.rs): functions
// only, shared by the UI's blends (ui_blend.wgsl) and feBlend
// (ui_filter.wgsl).

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

