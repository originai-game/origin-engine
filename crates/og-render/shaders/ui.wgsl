// The UI pass's shapes (src/ui/pass.rs), after ui_common.wgsl: fills,
// strokes, shadows, image paints, blends, backdrops and vectors. Every
// fragment entry returns a straight-alpha colour in the target's encoding,
// alpha its coverage, so the pipelines blend as the overlay does; the
// multiply, screen and add pipelines take what `blend_out` returns for
// them.

struct SpriteParams {
  // The sprite layout's (overlay.wgsl): unused here.
  msdf: vec4<f32>,
};
@group(1) @binding(0) var sprite_tex: texture_2d<f32>;
@group(1) @binding(1) var sprite_sampler: sampler;
@group(1) @binding(2) var<uniform> sprite_params: SpriteParams;

struct BackdropParams {
  // x: 1 when `behind` holds sRGB-encoded values; w: the pyramid's levels.
  params: vec4<f32>,
};
// The blur pyramid of what is under a backdrop (src/glass.rs), and a copy
// of the target as drawn so far (the blends that read it).
@group(2) @binding(0) var pyramid_tex: texture_2d<f32>;
@group(2) @binding(1) var behind_tex: texture_2d<f32>;
@group(2) @binding(2) var backdrop_sampler: sampler;
@group(2) @binding(3) var<uniform> backdrop: BackdropParams;

const FIT_FILL: u32 = 0u;
const FIT_COVER: u32 = 1u;
const FIT_CONTAIN: u32 = 2u;
const FIT_NONE: u32 = 3u;
const FIT_NINE: u32 = 4u;
const FIT_PATTERN: u32 = 5u;

struct ImageUv {
  // Where to sample.
  uv: vec2<f32>,
  // The same without its wraps (tiles, slices), for the sampling gradients.
  raw: vec2<f32>,
  // 0 where the image draws nothing (outside a contained image, a
  // nine-slice's empty centre).
  mask: f32,
};

// One axis of a nine-slice: `x` CSS px from the box's start of `len`, the
// border `a`, `b` drawn (CSS px) of source border `sa`, `sb` (fractions of
// the region) — the fraction of the region to sample, its unwrapped twin,
// and 1 in the centre.
fn nine_axis(x: f32, len: f32, a: f32, b: f32, sa: f32, sb: f32, edges: u32, src_len: f32) -> vec3<f32> {
  if (x < a) {
    return vec3<f32>(x / max(a, 1e-4) * sa, x / max(a, 1e-4) * sa, 0.0);
  }
  if (x > len - b) {
    let t = (x - (len - b)) / max(b, 1e-4);
    let f = 1.0 - sb + t * sb;
    return vec3<f32>(f, f, 0.0);
  }
  let mid = max(len - a - b, 1e-4);
  let src_mid = max(1.0 - sa - sb, 1e-4);
  let t = (x - a) / mid;
  if (edges == 0u) {
    let f = sa + t * src_mid;
    return vec3<f32>(f, f, 1.0);
  }
  // A tile is the source's middle at the border's scale (its pixels per
  // CSS px); round fits a whole number of them.
  let scale = select(sa * src_len / max(a, 1e-4), sb * src_len / max(b, 1e-4), a <= 0.0);
  var tile = src_mid * src_len / max(scale, 1e-4);
  if (edges == 2u) {
    tile = mid / max(round(mid / max(tile, 1e-4)), 1.0);
  }
  let k = (x - a) / max(tile, 1e-4);
  return vec3<f32>(sa + fract(k) * src_mid, sa + k * src_mid, 1.0);
}

fn image_uv(s: UiInstance, p: vec2<f32>) -> ImageUv {
  var out: ImageUv;
  out.mask = 1.0;
  let fit = (s.kinds.y >> 4u) & 15u;
  let size = 2.0 * s.center_half.zw;
  let pos = p + s.center_half.zw;
  let img = max(s.paint2.zw, vec2<f32>(1.0));
  var f = pos / max(size, vec2<f32>(1e-4));
  switch fit {
    case 1u, 2u: {
      // Cover crops, contain fits; the focal point places it.
      let k = select(max(size.x / img.x, size.y / img.y), min(size.x / img.x, size.y / img.y), fit == FIT_CONTAIN);
      let drawn = img * k;
      let origin = (size - drawn) * s.paint2.xy;
      f = (pos - origin) / drawn;
      if (fit == FIT_CONTAIN && (any(f < vec2<f32>(0.0)) || any(f > vec2<f32>(1.0)))) {
        out.mask = 0.0;
      }
    }
    case 3u: {
      let origin = (size - img) * s.paint2.xy;
      f = (pos - origin) / img;
      if (any(f < vec2<f32>(0.0)) || any(f > vec2<f32>(1.0))) {
        out.mask = 0.0;
      }
    }
    case 4u: {
      let src = s.paint3 / vec4<f32>(img.x, img.y, img.x, img.y);
      let edges = (s.kinds.y >> 8u) & 15u;
      let ax = nine_axis(pos.x, size.x, s.paint4.x, s.paint4.z, src.x, src.z, edges, img.x);
      let ay = nine_axis(pos.y, size.y, s.paint4.y, s.paint4.w, src.y, src.w, edges, img.y);
      let center = ((s.kinds.y >> 12u) & 1u) == 1u;
      if (!center && ax.z > 0.5 && ay.z > 0.5) {
        out.mask = 0.0;
      }
      let uv0 = mix(s.paint.xy, s.paint.zw, vec2<f32>(ax.x, ay.x));
      let raw = mix(s.paint.xy, s.paint.zw, vec2<f32>(ax.y, ay.y));
      out.uv = uv0;
      out.raw = raw;
      return out;
    }
    case 5u: {
      // A tile of paint2.xy CSS px turned by geo2.w, moved by paint3.xy
      // and scrolling by paint3.zw CSS px a second of the UI clock.
      let tile = max(s.paint2.xy, vec2<f32>(1e-3));
      let moved = pos - s.paint3.xy - s.paint3.zw * frame.origin.z;
      let r = rotate2(moved, -s.geo2.w);
      let k = r / tile;
      out.uv = mix(s.paint.xy, s.paint.zw, fract(k));
      out.raw = mix(s.paint.xy, s.paint.zw, k);
      return out;
    }
    default: {}
  }
  let c = clamp(f, vec2<f32>(0.0), vec2<f32>(1.0));
  out.uv = mix(s.paint.xy, s.paint.zw, c);
  out.raw = mix(s.paint.xy, s.paint.zw, f);
  return out;
}

// The colour as the pipeline of blend `blend` blends it: multiply takes
// mix(1, colour, alpha) times the destination, screen and add the
// premultiplied colour (screen: over one minus itself).
fn blend_out(rgb: vec3<f32>, a: f32, blend: u32) -> vec4<f32> {
  var col = rgb;
  if (frame.output.w > 0.5) {
    col = encode(rgb);
  }
  switch blend {
    case 1u: { return vec4<f32>(mix(vec3<f32>(1.0), col, a), 1.0); }
    case 2u, 3u: { return vec4<f32>(col * a, a); }
    default: { return vec4<f32>(col, a); }
  }
}

fn shape_color(s: UiInstance, in: UiOut, im: ImageUv, gx: vec2<f32>, gy: vec2<f32>) -> vec4<f32> {
  var c = flat_paint(s, in.local, in.position.xy);
  if ((s.kinds.y & 15u) == PAINT_IMAGE) {
    let t = textureSampleGrad(sprite_tex, sprite_sampler, im.uv, gx, gy);
    c = t * s.color * vec4<f32>(1.0, 1.0, 1.0, im.mask);
  }
  return c;
}

@fragment
fn fs_shape(in: UiOut) -> @location(0) vec4<f32> {
  let s = instances[in.instance];
  let px = local_px(in.local);
  let im = image_uv(s, in.local);
  let gx = dpdx(im.raw);
  let gy = dpdy(im.raw);
  let cov = mode_coverage(s, in.local, px);
  let c = shape_color(s, in, im, gx, gy);
  let a = clamp(cov * c.a * s.extra.x * instance_clip(s, in.position.xy), 0.0, 1.0);
  return blend_out(c.rgb, a, (s.kinds.z >> 4u) & 15u);
}

fn blend_channel(b: f32, s: f32, mode: u32) -> f32 {
  switch mode {
    case 4u: {
      // Overlay: multiply or screen by the backdrop.
      return select(1.0 - 2.0 * (1.0 - s) * (1.0 - b), 2.0 * s * b, b <= 0.5);
    }
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

// The blends that read what is under the shape: the backdrop's colour by
// the formula, laid over it by the shape's coverage.
@fragment
fn fs_shape_dst(in: UiOut) -> @location(0) vec4<f32> {
  let s = instances[in.instance];
  let px = local_px(in.local);
  let im = image_uv(s, in.local);
  let gx = dpdx(im.raw);
  let gy = dpdy(im.raw);
  let cov = mode_coverage(s, in.local, px);
  let c = shape_color(s, in, im, gx, gy);
  let a = clamp(cov * c.a * s.extra.x * instance_clip(s, in.position.xy), 0.0, 1.0);
  let size = vec2<i32>(textureDimensions(behind_tex));
  let at = clamp(vec2<i32>(in.position.xy), vec2<i32>(0), size - 1);
  var b = textureLoad(behind_tex, at, 0).rgb;
  if (backdrop.params.x > 0.5) {
    b = decode(b);
  }
  let mode = (s.kinds.z >> 4u) & 15u;
  let src = clamp(c.rgb, vec3<f32>(0.0), vec3<f32>(1.0));
  let bb = clamp(b, vec3<f32>(0.0), vec3<f32>(1.0));
  let r = vec3<f32>(blend_channel(bb.r, src.r, mode), blend_channel(bb.g, src.g, mode), blend_channel(bb.b, src.b, mode));
  return finish_color(r, a);
}

// A glass shadow (offset xy, blur, spread) of the shape at local `p`: its
// coverage 0..1, as a drop shadow's (mode_coverage).
fn glass_shadow(s: UiInstance, p: vec2<f32>, sh: vec4<f32>, px: f32) -> f32 {
  let q = p - sh.xy;
  let sigma = max(0.5 * max(sh.z, 0.0), 0.5 * px);
  let kind = s.kinds.x & 15u;
  let corner = (s.kinds.x >> 4u) & 15u;
  if (kind == GEO_RECT && corner == CORNER_ROUND) {
    let half = max(s.center_half.zw + vec2<f32>(sh.w), vec2<f32>(0.0));
    let r = clamp(corner_radius(q, s.geo) + sh.w, 0.0, min(half.x, half.y));
    return clamp(rounded_box_shadow(q, half, r, sigma), 0.0, 1.0);
  }
  let d = shape_distance(s, q).x - sh.w;
  return clamp(0.5 - 0.5 * erf1(d * 0.70710678 / sigma), 0.0, 1.0);
}

// A backdrop: what is under the shape blurred (the pyramid's level for
// the radius), its saturation and brightness scaled, tinted, masked to the
// shape. Glass is one: its material (ui::UiGlass) adds a hairline inside
// the outline (mode.w wide, mode2 its colour) and up to two soft shadows
// outside it (paint..paint4); flag 1 (extra.z) draws the tint opaque and
// reads nothing under it, flag 2 mixes in sRGB-encoded values as CSS does.
// With none of them, the plain frosted backdrop, as it always was.
@fragment
fn fs_backdrop(in: UiOut) -> @location(0) vec4<f32> {
  let s = instances[in.instance];
  let px = local_px(in.local);
  let d = shape_distance(s, in.local).x;
  let cov = clamp(0.5 - d / px, 0.0, 1.0);
  let flags = u32(s.extra.z);
  let solid = (flags & 1u) != 0u;
  let srgb = (flags & 2u) != 0u;
  let clip = s.extra.x * instance_clip(s, in.position.xy);
  // Outside the shape only its shadows draw: nothing under it is read.
  var rgb = vec3<f32>(0.0);
  if (cov > 0.0) {
    if (solid) {
      rgb = s.color.rgb;
    } else {
      let levels = backdrop.params.w;
      let blur = max(s.mode.x, 0.0) * frame.output.z;
      let lod = clamp(log2(max(blur, 1.0)) - 1.0, 0.0, max(levels - 1.0, 0.0));
      let at = in.position.xy / frame.output.xy;
      rgb = textureSampleLevel(pyramid_tex, backdrop_sampler, at, lod).rgb;
      if (srgb) {
        // CSS's backdrop-filter and its tint work on encoded values.
        rgb = encode(rgb);
        let luma = dot(rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
        rgb = max(mix(vec3<f32>(luma), rgb, s.mode.y) * s.mode.z, vec3<f32>(0.0));
        rgb = decode(mix(min(rgb, vec3<f32>(1.0)), encode(s.color.rgb), s.color.a));
      } else {
        let luma = dot(rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
        rgb = mix(vec3<f32>(luma), rgb, s.mode.y) * s.mode.z;
        rgb = mix(rgb, s.color.rgb, s.color.a);
      }
    }
    // The hairline: the band from the outline to `edge` inside it.
    let edge = max(s.mode.w, 0.0);
    if (edge > 0.0 && s.mode2.a > 0.0) {
      let band = cov - clamp(0.5 - (d + edge) / px, 0.0, 1.0);
      let k = clamp(band * s.mode2.a / cov, 0.0, 1.0);
      if (srgb) {
        rgb = decode(mix(encode(rgb), encode(s.mode2.rgb), k));
      } else {
        rgb = mix(rgb, s.mode2.rgb, k);
      }
    }
  }
  let ga = clamp(cov * s.extra.x * instance_clip(s, in.position.xy), 0.0, 1.0);
  if (cov >= 1.0 || (s.paint2.a <= 0.0 && s.paint4.a <= 0.0)) {
    return finish_color(rgb, ga);
  }
  // The shadows, outside the shape only (as under a translucent CSS box),
  // the first over the second, and the glass over them.
  var sa = 0.0;
  var sc = vec3<f32>(0.0);
  for (var k = 1; k >= 0; k -= 1) {
    let sh = select(s.paint3, s.paint, k == 0);
    let col = select(s.paint4, s.paint2, k == 0);
    if (col.a > 0.0) {
      var a = glass_shadow(s, in.local, sh, px) * col.a;
      if (srgb && frame.output.w < 0.5) {
        // Blended in linear light: as dark as CSS's over encoded values.
        a = 1.0 - pow(1.0 - a, 2.2);
      }
      sc = mix(sc * sa, col.rgb, a) / max(sa + a * (1.0 - sa), 1e-4);
      sa = sa + a * (1.0 - sa);
    }
  }
  sa = sa * (1.0 - cov) * clip;
  let a = ga + sa * (1.0 - ga);
  let c = (rgb * ga + sc * sa * (1.0 - ga)) / max(a, 1e-4);
  return finish_color(c, a);
}

// ── vectors ──────────────────────────────────────────────────────────────

// A cooked vector's vertex (UiVectorVertex) drawn by instance `ii`: its
// view box (geo) fitted into the instance's box, the edge vertices moved
// half a device pixel along the outline's normal, in (-1) or out (+1).
@vertex
fn vs_vector(@location(0) pn: vec4<f32>, @location(1) ap: vec2<u32>, @builtin(instance_index) ii: u32) -> UiOut {
  let s = instances[ii];
  let vb = s.geo;
  let k = min(2.0 * s.center_half.z / max(vb.z, 1e-4), 2.0 * s.center_half.w / max(vb.w, 1e-4));
  var local = (pn.xy - (vb.xy + 0.5 * vb.zw)) * k;
  let side = bitcast<f32>(ap.x);
  let det = abs(s.xform.x * s.xform.w - s.xform.y * s.xform.z);
  let half_px = 0.5 / (frame.output.z * sqrt(max(det, 1e-8)));
  local += pn.zw * side * half_px;
  var o: UiOut;
  o.position = device_position(s.center_half.xy + mat_apply(s.xform, local));
  o.local = pn.xy;
  o.instance = ii;
  o.vpaint = ap.y;
  o.aa = select(1.0, 0.0, side > 0.0);
  return o;
}

@fragment
fn fs_vector(in: UiOut) -> @location(0) vec4<f32> {
  let s = instances[in.instance];
  let vb = s.geo;
  let k = min(2.0 * s.center_half.z / max(vb.z, 1e-4), 2.0 * s.center_half.w / max(vb.w, 1e-4));
  let f = (in.local - vb.xy) / max(vb.zw, vec2<f32>(1e-4));
  let c = gradient_color(u32(s.extra.y) + in.vpaint, f, vb.zw * k, in.position.xy) * s.color;
  let a = clamp(in.aa * c.a * s.extra.x * instance_clip(s, in.position.xy), 0.0, 1.0);
  return finish_color(c.rgb, a);
}
