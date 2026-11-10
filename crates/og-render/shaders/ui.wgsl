// The UI pass's shapes (src/ui/pass.rs), after ui_common.wgsl and
// ui_blend.wgsl: fills,
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

const FIT_FILL: u32 = 0u;
const FIT_COVER: u32 = 1u;
const FIT_CONTAIN: u32 = 2u;
const FIT_NONE: u32 = 3u;
const FIT_NINE: u32 = 4u;
const FIT_PATTERN: u32 = 5u;
const FIT_BACKGROUND: u32 = 6u;
const FIT_BORDER: u32 = 7u;
const FIT_SCALE_DOWN: u32 = 8u;

struct ImageUv {
  // Where to sample.
  uv: vec2<f32>,
  // The same without its wraps (tiles, slices), for the sampling gradients.
  raw: vec2<f32>,
  // 0 where the image draws nothing (outside a contained image, a
  // nine-slice's empty centre).
  mask: f32,
};

// Tiles across a middle `mid` CSS px long, `q` CSS px into it: `tile`
// long, repeated (1, from its start or, `centered`, one tile centred),
// rounded to a whole number (2) or spaced (3: whole tiles, the room left
// shared out between and around them). The fraction of the tile, its
// unwrapped twin (tiles), and 1 where a tile is drawn.
fn tile_mid(q: f32, mid: f32, tile: f32, rep: u32, centered: bool) -> vec3<f32> {
  var w = max(tile, 1e-4);
  var o = 0.0;
  var pitch = w;
  var n = 1e9;
  if (rep == 2u) {
    w = mid / max(round(mid / w), 1.0);
    pitch = w;
  } else if (rep == 3u) {
    n = floor(mid / w);
    let gap = (mid - n * w) / (n + 1.0);
    pitch = w + gap;
    o = gap;
  } else if (centered) {
    o = (mid - w) * 0.5;
  }
  let k = (q - o) / pitch;
  let t = fract(k) * pitch / w;
  var drawn = select(0.0, 1.0, t <= 1.0);
  if (rep == 3u && (k < 0.0 || k >= n)) {
    drawn = 0.0;
  }
  return vec3<f32>(min(t, 1.0), k * pitch / w, drawn);
}

// One axis of a nine-slice: `x` CSS px from the box's start of `len`, the
// border `a`, `b` drawn (CSS px) of source border `sa`, `sb` (fractions of
// the region) — the fraction of the region to sample, its unwrapped twin,
// 1 in the centre, and 1 where it draws (not in a space's gap).
fn nine_axis(x: f32, len: f32, a: f32, b: f32, sa: f32, sb: f32, edges: u32, src_len: f32) -> vec4<f32> {
  if (x < a) {
    return vec4<f32>(x / max(a, 1e-4) * sa, x / max(a, 1e-4) * sa, 0.0, 1.0);
  }
  if (x > len - b) {
    let t = (x - (len - b)) / max(b, 1e-4);
    let f = 1.0 - sb + t * sb;
    return vec4<f32>(f, f, 0.0, 1.0);
  }
  let mid = max(len - a - b, 1e-4);
  let src_mid = max(1.0 - sa - sb, 1e-4);
  let t = (x - a) / mid;
  if (edges == 0u) {
    let f = sa + t * src_mid;
    return vec4<f32>(f, f, 1.0, 1.0);
  }
  // A tile is the source's middle at the border's scale (its pixels per
  // CSS px).
  let scale = select(sa * src_len / max(a, 1e-4), sb * src_len / max(b, 1e-4), a <= 0.0);
  let r = tile_mid(x - a, mid, src_mid * src_len / max(scale, 1e-4), edges, false);
  return vec4<f32>(sa + r.x * src_mid, sa + r.y * src_mid, 1.0, r.z);
}

// One axis of a CSS border image (CSS Backgrounds 3 §6.2): `x` CSS px
// across `len`, the widths `a`, `b` drawn of the slices `sa`, `sb` (image
// px of `src`), the middle at `scale` CSS px an image px filled by `rep`
// (0 stretch, 1 repeat centred, 2 round, 3 space). The fraction of the
// image, its unwrapped twin, 1 where it draws, and 1 in the middle.
fn border_axis(x: f32, len: f32, a: f32, b: f32, sa: f32, sb: f32, src: f32, scale: f32, rep: u32) -> vec4<f32> {
  let n = max(src, 1e-4);
  if (x < a) {
    let f = x / max(a, 1e-4) * sa / n;
    return vec4<f32>(f, f, 1.0, 0.0);
  }
  if (x > len - b) {
    let f = (src - sb + (x - (len - b)) / max(b, 1e-4) * sb) / n;
    return vec4<f32>(f, f, 1.0, 0.0);
  }
  let mid = max(len - a - b, 1e-4);
  let smid = max(src - sa - sb, 1e-4);
  if (rep == 0u) {
    let f = (sa + (x - a) / mid * smid) / n;
    return vec4<f32>(f, f, 1.0, 1.0);
  }
  let r = tile_mid(x - a, mid, smid * scale, rep, true);
  return vec4<f32>((sa + r.x * smid) / n, (sa + r.y * smid) / n, r.z, 1.0);
}

// A border width over its slice: the CSS px an image px of that edge is
// drawn at; 0 when either is (no scale).
fn border_scale(w: f32, slice: f32) -> f32 {
  return select(0.0, w / max(slice, 1e-6), w > 0.0 && slice > 0.0);
}

// One axis of a CSS background layer: `x` CSS px from the box's start of
// `len`, the image `w` long placed at `origin`, repeated by `rep` (0
// repeat, 1 no-repeat, 2 space, 3 round, whose `w` is already rounded):
// the fraction of the image to sample, its unwrapped twin, and 1 where an
// image is drawn (not in a gap, not past a lone image).
fn background_axis(x: f32, len: f32, w: f32, origin: f32, rep: u32) -> vec3<f32> {
  let iw = max(w, 1e-4);
  var o = origin;
  var pitch = iw;
  var tiled = rep != 1u;
  if (rep == 2u) {
    // Space: whole images from edge to edge, the room left between them;
    // fewer than two fit, one placed by its position.
    let n = floor(len / iw);
    if (n >= 2.0) {
      pitch = iw + (len - n * iw) / (n - 1.0);
      o = 0.0;
    } else {
      tiled = false;
    }
  }
  let q = x - o;
  if (!tiled) {
    let t = q / iw;
    return vec3<f32>(clamp(t, 0.0, 1.0), t, select(0.0, 1.0, t >= 0.0 && t <= 1.0));
  }
  let k = q / pitch;
  let t = fract(k) * pitch / iw;
  return vec3<f32>(min(t, 1.0), k * pitch / iw, select(0.0, 1.0, t <= 1.0));
}

fn image_uv(s: UiInstance, p: vec2<f32>) -> ImageUv {
  var out: ImageUv;
  out.mask = 1.0;
  let fit = (s.kinds.y >> 4u) & 15u;
  let size = 2.0 * s.center_half.zw;
  let pos = p + s.center_half.zw;
  var img = max(s.paint2.zw, vec2<f32>(1.0));
  if ((s.kinds.y & 15u) == PAINT_IMAGE && s.paint2.z <= 0.0) {
    // No size given: the region's own pixels.
    img = max(abs(s.paint.zw - s.paint.xy) * vec2<f32>(textureDimensions(sprite_tex, 0)), vec2<f32>(1.0));
  }
  let density = select(1.0, s.extra.y, (s.kinds.y & 15u) == PAINT_IMAGE);
  img = img / density;
  var f = pos / max(size, vec2<f32>(1e-4));
  switch fit {
    case 1u, 2u, 8u: {
      // Cover crops, contain fits (scale-down never above its own size);
      // the focal point and offset place it.
      var k = select(max(size.x / img.x, size.y / img.y), min(size.x / img.x, size.y / img.y), fit != FIT_COVER);
      if (fit == FIT_SCALE_DOWN) {
        k = min(k, 1.0);
      }
      let drawn = img * k;
      let origin = (size - drawn) * s.paint2.xy + s.paint3.xy;
      f = (pos - origin) / drawn;
      if (fit != FIT_COVER && (any(f < vec2<f32>(0.0)) || any(f > vec2<f32>(1.0)))) {
        out.mask = 0.0;
      }
    }
    case 3u: {
      let origin = (size - img) * s.paint2.xy + s.paint3.xy;
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
      out.mask = ax.w * ay.w;
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
    case 6u: {
      // A CSS background layer (UiBackground): its natural size the
      // region's texels, sized (paint2.xy given, -1 auto; cover 1,
      // contain 2 in the centre's bits), rounded, placed (paint3) and
      // repeated per axis (the edges' bits).
      // A gradient's natural size is its own (paint2.zw); the
      // positioning area is paint4's edges in from the box's.
      var nat = max(abs(s.paint.zw - s.paint.xy) * vec2<f32>(textureDimensions(sprite_tex, 0)), vec2<f32>(1.0));
      nat = nat / density;
      if ((s.kinds.y & 15u) == PAINT_GRADIENT_IMAGE) {
        nat = s.paint2.zw;
      }
      let pos = pos - s.paint4.xy;
      let size = size - s.paint4.xy - s.paint4.zw;
      let sizing = (s.kinds.y >> 12u) & 3u;
      let g = s.paint2.xy;
      var w = nat;
      if (sizing == 1u) {
        w = nat * max(size.x / nat.x, size.y / nat.y);
      } else if (sizing == 2u) {
        w = nat * min(size.x / nat.x, size.y / nat.y);
      } else if (g.x >= 0.0 && g.y >= 0.0) {
        w = g;
      } else if (g.x >= 0.0) {
        w = vec2<f32>(g.x, g.x * nat.y / nat.x);
      } else if (g.y >= 0.0) {
        w = vec2<f32>(g.y * nat.x / nat.y, g.y);
      }
      let rx = (s.kinds.y >> 8u) & 3u;
      let ry = (s.kinds.y >> 10u) & 3u;
      let given = w;
      if (rx == 3u) {
        w.x = size.x / max(round(size.x / max(given.x, 1e-4)), 1.0);
      }
      if (ry == 3u) {
        w.y = size.y / max(round(size.y / max(given.y, 1e-4)), 1.0);
      }
      // Round on one axis keeps the ratio on the other when its size is auto.
      if (rx == 3u && ry != 3u && sizing == 0u && g.y < 0.0) {
        w.y = given.y * w.x / max(given.x, 1e-4);
      }
      if (ry == 3u && rx != 3u && sizing == 0u && g.x < 0.0) {
        w.x = given.x * w.y / max(given.y, 1e-4);
      }
      let origin = s.paint3.xy * (size - w) + s.paint3.zw;
      let ax = background_axis(pos.x, size.x, w.x, origin.x, rx);
      let ay = background_axis(pos.y, size.y, w.y, origin.y, ry);
      out.mask = ax.z * ay.z;
      out.uv = mix(s.paint.xy, s.paint.zw, vec2<f32>(ax.x, ay.x));
      out.raw = mix(s.paint.xy, s.paint.zw, vec2<f32>(ax.y, ay.y));
      return out;
    }
    case 7u: {
      // A CSS border image (UiBorderImage): slices paint3 (image px; a
      // negative one a fraction of the image), widths paint4 (CSS px; a
      // negative one its slice's px), left, top, right, bottom, scaled
      // down together where they overlap. The middle of the top and
      // bottom rows tiles at its own edge's scale, the middle row at the
      // top's (else the bottom's); columns likewise from the left and
      // right (CSS Backgrounds 3 §6.2).
      var nat = max(abs(s.paint.zw - s.paint.xy) * vec2<f32>(textureDimensions(sprite_tex, 0)), vec2<f32>(1.0));
      if ((s.kinds.y & 15u) == PAINT_GRADIENT_IMAGE) {
        nat = s.paint2.zw;
      }
      let img = nat;
      let natv = vec4<f32>(nat.x, nat.y, nat.x, nat.y);
      let sl = min(select(s.paint3, -s.paint3 * natv, s.paint3 < vec4<f32>(0.0)), natv);
      var wd = select(s.paint4, sl / density, s.paint4 < vec4<f32>(0.0));
      let fit = min(min(size.x / max(wd.x + wd.z, 1e-4), size.y / max(wd.y + wd.w, 1e-4)), 1.0);
      wd = wd * fit;
      let top = border_scale(wd.y, sl.y);
      let bottom = border_scale(wd.w, sl.w);
      let left = border_scale(wd.x, sl.x);
      let right = border_scale(wd.z, sl.z);
      var fx = select(top, bottom, pos.y > size.y - wd.w);
      if (fx <= 0.0) {
        fx = select(bottom, top, pos.y > size.y - wd.w);
      }
      var fy = select(left, right, pos.x > size.x - wd.z);
      if (fy <= 0.0) {
        fy = select(right, left, pos.x > size.x - wd.z);
      }
      let rx = (s.kinds.y >> 8u) & 3u;
      let ry = (s.kinds.y >> 10u) & 3u;
      let ax = border_axis(pos.x, size.x, wd.x, wd.z, sl.x, sl.z, img.x, select(1.0, fx, fx > 0.0), rx);
      let ay = border_axis(pos.y, size.y, wd.y, wd.w, sl.y, sl.w, img.y, select(1.0, fy, fy > 0.0), ry);
      out.mask = ax.z * ay.z;
      if (((s.kinds.y >> 12u) & 1u) == 0u && ax.w > 0.5 && ay.w > 0.5) {
        out.mask = 0.0;
      }
      out.uv = mix(s.paint.xy, s.paint.zw, vec2<f32>(ax.x, ay.x));
      out.raw = mix(s.paint.xy, s.paint.zw, vec2<f32>(ax.y, ay.y));
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
    // Screen, add and the Porter-Duff operators blend premultiplied.
    case 2u, 3u, 17u, 18u, 19u, 20u: { return vec4<f32>(col * a, a); }
    default: { return vec4<f32>(col, a); }
  }
}

fn shape_color(s: UiInstance, in: UiOut, im: ImageUv, gx: vec2<f32>, gy: vec2<f32>) -> vec4<f32> {
  var c = flat_paint(s, in.local, in.position.xy);
  let kind = s.kinds.y & 15u;
  if (kind == PAINT_IMAGE) {
    var t = vec4<f32>(0.0);
    if (((s.kinds.y >> 17u) & 1u) == 1u) {
      // CSS image-rendering pixelated or crisp-edges: the centre of the
      // texel under it, from the top level, so the sampler's filter
      // (linear or nearest) reads that one texel.
      let dims = vec2<f32>(textureDimensions(sprite_tex, 0));
      let texel = (floor(im.uv * dims) + vec2<f32>(0.5)) / dims;
      t = textureSampleLevel(sprite_tex, sprite_sampler, texel, 0.0);
    } else {
      t = textureSampleGrad(sprite_tex, sprite_sampler, im.uv, gx, gy);
    }
    c = t * s.color * vec4<f32>(1.0, 1.0, 1.0, im.mask);
  } else if (kind == PAINT_GRADIENT_IMAGE) {
    c = gradient_color(u32(s.extra.y), im.uv, s.paint2.zw, in.position.xy);
    c.a *= im.mask;
  }
  if (((s.kinds.y >> 16u) & 1u) == 1u && (kind == PAINT_IMAGE || kind == PAINT_GRADIENT_IMAGE)) {
    // A luminance mask (CSS Masking 1 §7.1): the luminance of its
    // sRGB-encoded colour times its alpha.
    let l = dot(encode(clamp(c.rgb, vec3<f32>(0.0), vec3<f32>(1.0))), vec3<f32>(0.2125, 0.7154, 0.0721));
    c = vec4<f32>(1.0, 1.0, 1.0, c.a * l);
  }
  return c;
}

@fragment
fn fs_shape(in: UiOut) -> @location(0) vec4<f32> {
  let s = instances[in.instance];
  let px = local_px(in.local);
  let im = image_uv(s, in.local);
  let gx = dpdxFine(im.raw);
  let gy = dpdyFine(im.raw);
  let cov = mode_coverage(s, in.local, px);
  let c = shape_color(s, in, im, gx, gy);
  let a = clamp(cov * c.a * s.extra.x * instance_clip(s, in.position.xy), 0.0, 1.0);
  return blend_out(c.rgb, a, (s.kinds.z >> 4u) & 31u);
}

// The blends that read what is under the shape: the backdrop's colour by
// the formula, laid over it by the shape's coverage.
@fragment
fn fs_shape_dst(in: UiOut) -> @location(0) vec4<f32> {
  let s = instances[in.instance];
  let px = local_px(in.local);
  let im = image_uv(s, in.local);
  let gx = dpdxFine(im.raw);
  let gy = dpdyFine(im.raw);
  let cov = mode_coverage(s, in.local, px);
  let c = shape_color(s, in, im, gx, gy);
  let a = clamp(cov * c.a * s.extra.x * instance_clip(s, in.position.xy), 0.0, 1.0);
  return blend_over(c.rgb, a, (s.kinds.z >> 4u) & 31u, in.position.xy);
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
// A backdrop brighter than `limit` (a luminance; 0 is none) scaled down to
// it, losing its colour as it goes: a bright scene lifts the glass evenly.
fn backdrop_limit(rgb: vec3<f32>, limit: f32) -> vec3<f32> {
  let luma = dot(rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
  if (limit <= 0.0 || luma <= limit) {
    return rgb;
  }
  let k = limit / luma;
  return mix(vec3<f32>(luma), rgb, k * k) * k;
}

@fragment
fn fs_backdrop(in: UiOut) -> @location(0) vec4<f32> {
  let s = instances[in.instance];
  let px = local_px(in.local);
  let d = shape_distance(s, in.local).x;
  let cov = clamp(0.5 - d / px, 0.0, 1.0);
  let flags = u32(s.extra.z) & 3u;
  let top_light = f32(u32(s.extra.z) >> 2u) / 255.0;
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
      let mi = s.kinds.y >> 8u;
      if (mi > 0u) {
        // CSS's backdrop-filter colour functions as one matrix over
        // encoded values, its opacity over the unblurred backdrop.
        let e = vec4<f32>(encode(max(rgb, vec3<f32>(0.0))), 1.0);
        let o = clamp(vec3<f32>(dot(points[mi - 1u], e), dot(points[mi], e), dot(points[mi + 1u], e)), vec3<f32>(0.0), vec3<f32>(1.0));
        let sharp = textureSampleLevel(pyramid_tex, backdrop_sampler, at, 0.0).rgb;
        rgb = mix(sharp, decode(o), clamp(points[mi + 2u].x, 0.0, 1.0));
      }
      if (srgb) {
        // CSS's backdrop-filter and its tint work on encoded values.
        rgb = encode(rgb);
        let luma = dot(rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
        rgb = max(mix(vec3<f32>(luma), rgb, s.mode.y) * s.mode.z, vec3<f32>(0.0));
        rgb = backdrop_limit(rgb, s.extra.y);
        rgb = decode(mix(min(rgb, vec3<f32>(1.0)), encode(s.color.rgb), s.color.a));
      } else {
        let luma = dot(rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
        rgb = mix(vec3<f32>(luma), rgb, s.mode.y) * s.mode.z;
        rgb = backdrop_limit(rgb, s.extra.y);
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
    // The lit top edge: the outline's top 1 px, strongest at the top's
    // middle, gone half way down the sides and softened at the corners.
    if (top_light > 0.0) {
      let band = cov - clamp(0.5 - (d + 1.0) / px, 0.0, 1.0);
      let half = max(s.center_half.zw, vec2<f32>(1e-4));
      let down = smoothstep(0.5, 1.0, -in.local.y / half.y);
      let across = 1.0 - 0.7 * smoothstep(0.6, 1.0, abs(in.local.x) / half.x);
      let k = clamp(band / cov, 0.0, 1.0) * top_light * down * across;
      if (srgb) {
        rgb = decode(mix(encode(rgb), vec3<f32>(1.0), k));
      } else {
        rgb = mix(rgb, vec3<f32>(1.0), k);
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
