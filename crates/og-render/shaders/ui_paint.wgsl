// The UI's paint pass (src/ui/paint.rs): og-ui's paint arena drawn with one
// pipeline per target format, every variant a branch on the instance's kind,
// so nothing is compiled after boot. Each instance is a box in CSS pixels
// (origin top left, y down) with its clip: a box rounded per corner in a
// colour or a gradient, a border ring, an outer or inset Gaussian box
// shadow, an MSDF glyph of the bound font page, a glyph's raster in the
// bound page of the glyph atlas, or a built-in icon's stroked segments.
// Colours are linear and straight until the end, which encodes them: the
// target stores what is returned, so the pass blends sRGB-encoded values, as
// CSS does. Where the output is no such target, the pass draws a layer that
// fs_layer draws over it (src/ui/paint.rs).
//
// A glyph's coverage goes through Skia's text preblend first, with Chrome
// on Linux's gamma and contrast: a port of SkTMaskGamma (Skia,
// src/core/SkMaskGamma.cpp; BSD-3-Clause, Copyright (c) 2011 Google Inc.;
// third_party/skia-mask-gamma), its tables built by paint.rs
// `preblend_table`.

struct PaintFrame {
  // xy: the target in device pixels; z: device pixels per CSS pixel.
  output: vec4<f32>,
  // x: one code value of the target (a gradient's dither); zw: the
  // target's top left on the output, device pixels (a group's layer).
  extra: vec4<f32>,
  // The preblend's eight rows of 256 coverages, a byte each.
  preblend: array<vec4<u32>, 128>,
};
@group(0) @binding(0) var<uniform> frame: PaintFrame;

struct PaintGpu {
  // x, y, width, height, CSS px (a glyph's: its quad).
  rect: vec4<f32>,
  // x0, y0, x1, y1, CSS px.
  clip: vec4<f32>,
  color: vec4<f32>,
  // A glyph's atlas region, u0 v0 u1 v1.
  uv: vec4<f32>,
  // A box's corner radii, top left, top right, bottom right, bottom left.
  r: vec4<f32>,
  // A glyph's weight in x; a border's widths, top right bottom left; a
  // shadow's offset (xy), blur (z) and spread (w); CSS px.
  p: vec4<f32>,
  // x: the kind (0 solid, 1 gradient, 2 glyph, 3 icon, 4 border, 5 shadow,
  // 6 inset shadow, 7 image, 8 a polyline's segment: ends in uv, its
  // neighbours' far ends in r, width and flags in p, 9 a glyph's raster,
  // 10 a group's composite: its result's top left and size on the output
  // in uv, device px), y and z its gradient, or its first segment and their
  // count, or an image's flags (1 fill, 2 tile), or a raster's tier (0
  // coverage, 1 colour, 2 distance field), or a group's flags (z: 1 cut to
  // its rounded box), w its transform (0 none).
  k: vec4<u32>,
  // An image's radial wipe: the fraction hidden, the start angle, 1 for
  // counter-clockwise; w its mask, a gradient's index + 1 (0 none).
  w: vec4<f32>,
  // x: its colour matrix's first row in `gradients` + 1, y: its rounded
  // clip's (0 none).
  f: vec4<u32>,
};
// A data texture (data_texture.rs), a quad nine texels in a row
// (paint.rs `PaintGpu`): a vertex stage reads no storage under WebGPU's
// compatibility mode.
@group(0) @binding(1) var quads: texture_2d<u32>;

fn paint_quad(i: u32) -> PaintGpu {
  var t: array<vec4<u32>, 9>;
  for (var k = 0u; k < 9u; k++) {
    t[k] = textureLoad(quads, data_at(i * 9u + k), 0);
  }
  return PaintGpu(
    bitcast<vec4<f32>>(t[0]), bitcast<vec4<f32>>(t[1]), bitcast<vec4<f32>>(t[2]),
    bitcast<vec4<f32>>(t[3]), bitcast<vec4<f32>>(t[4]), bitcast<vec4<f32>>(t[5]),
    t[6], bitcast<vec4<f32>>(t[7]), t[8],
  );
}
// Three entries a gradient: x the angle (radians, CSS: 0 up, clockwise), y
// its first stop, z its stops, w its shape (0 linear, 1 radial, 2 conic);
// then its centre (xy, CSS px from the box's centre) and a radial one's radii
// (zw); then a mask's box (x y w h, CSS px before transforms). After the
// gradients, five rows a colour matrix: its rows r, g, b, a, then offsets;
// then four a rounded clip: its box (x0 y0 x1 y1), its radii, and the inverse
// of its transform, the linear part (a b c d) then the translation (xy) and
// z 1 when it is inverted (drawn only outside).
@group(0) @binding(2) var<storage, read> gradients: array<vec4<f32>>;
// Two entries a stop: its colour (linear, straight), then its offset in x.
@group(0) @binding(3) var<storage, read> stops: array<vec4<f32>>;
// The built-in icons' segments, a b in their 24-unit space.
@group(0) @binding(4) var<storage, read> segments: array<vec4<f32>>;
// Paint-time transforms, two entries each: CSS's matrix a b c d, then e f;
// a data texture (data_texture.rs) the vertex stage reads.
@group(0) @binding(5) var transforms: texture_2d<f32>;

@group(1) @binding(0) var page: texture_2d<f32>;
@group(1) @binding(1) var page_sampler: sampler;
// x: the distance range in atlas pixels, yz: the atlas size.
@group(1) @binding(2) var<uniform> page_params: vec4<f32>;

struct PaintOut {
  @builtin(position) position: vec4<f32>,
  // The fragment's place in the box, CSS px from its centre.
  @location(0) local: vec2<f32>,
  @location(1) uv: vec2<f32>,
  @location(2) @interpolate(flat, either) index: u32,
  // Device pixels per CSS pixel of the box, its transform's scale included.
  @location(3) @interpolate(flat, either) aa: f32,
  // The fragment's place in CSS px before transforms, which a mask reads.
  @location(4) at: vec2<f32>,
};

@vertex
fn vs_paint(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> PaintOut {
  var corners = array<vec2<f32>, 6>(
    vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0),
    vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0));
  // Load only the texels the vertex stage needs (3 for most quads, 4 for
  // outer shadows) instead of all 9 texels of PaintGpu.
  let base = ii * 9u;
  let rect = bitcast<vec4<f32>>(textureLoad(quads, data_at(base), 0));
  let uv = bitcast<vec4<f32>>(textureLoad(quads, data_at(base + 3u), 0));
  let k = textureLoad(quads, data_at(base + 6u), 0);
  let c = corners[vi];
  // What the quad covers around the box's centre: a box and a device pixel
  // for its antialiased edge; a glyph's quad, which holds its field's reach;
  // a shadow's moved, grown box and its blur's reach (three deviations).
  var m = vec4<f32>(1.0, 0.0, 0.0, 1.0);
  var t = vec2<f32>(0.0);
  if (k.w != 0u) {
    m = textureLoad(transforms, data_at(2u * k.w), 0);
    t = textureLoad(transforms, data_at(2u * k.w + 1u), 0).xy;
  }
  let aa = frame.output.z * sqrt(max(abs(m.x * m.w - m.y * m.z), 1e-6));
  let px = 1.0 / max(aa, 1e-3);
  let half = 0.5 * rect.zw;
  var lo = -half - vec2<f32>(px);
  var hi = half + vec2<f32>(px);
  if (k.x == 7u || k.x == 9u || (k.x == 10u && (k.z & 1u) == 0u)) {
    lo = -half;
    hi = half;
  } else if (k.x == 5u) {
    let p = bitcast<vec4<f32>>(textureLoad(quads, data_at(base + 5u), 0));
    let reach = max(half + vec2<f32>(p.w + 1.5 * p.z + px), vec2<f32>(0.0));
    lo = p.xy - reach;
    hi = p.xy + reach;
  }
  let local = mix(lo, hi, c);
  let at = rect.xy + half + local;
  // The transform moves the quad; the box's distances stay its own.
  let css = vec2<f32>(m.x * at.x + m.z * at.y, m.y * at.x + m.w * at.y) + t;
  // On the output, then on the target, which may be a layer of it.
  let screen = css * frame.output.z;
  let device = screen - frame.extra.zw;
  var o: PaintOut;
  o.position = vec4<f32>(device.x / frame.output.x * 2.0 - 1.0, 1.0 - device.y / frame.output.y * 2.0, 0.0, 1.0);
  o.local = local;
  o.uv = vec2<f32>(mix(uv.x, uv.z, c.x), mix(uv.y, uv.w, c.y));
  if (k.x == 10u) {
    // A group's result where this corner lands on the output: a pixel's
    // centre reads its texel's centre.
    o.uv = (screen - uv.xy) / uv.zw;
  }
  o.index = ii;
  o.aa = aa;
  o.at = at;
  return o;
}

// srgb_encode and srgb_decode are color.wgsl's (the library, prepended).

// A box of half size `half` with corner radii `r` (top left, top right,
// bottom right, bottom left; y down): its signed distance at `p`.
fn sd_round_box(p: vec2<f32>, half: vec2<f32>, r: vec4<f32>) -> f32 {
  var k = select(r.x, r.y, p.x > 0.0);
  if (p.y > 0.0) {
    k = select(r.w, r.z, p.x > 0.0);
  }
  let rad = clamp(k, 0.0, max(min(half.x, half.y), 0.0));
  let q = abs(p) - half + vec2<f32>(rad);
  return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - rad;
}

// A distance's coverage over a device pixel (`scale` of them a CSS px).
fn edge(d: f32, scale: f32) -> f32 {
  return clamp(0.5 - d * scale, 0.0, 1.0);
}

// erf, to 5e-4 (Abramowitz and Stegun's 7.1.26 family, Winitzki's form).
fn erf_approx(x: f32) -> f32 {
  let x2 = x * x;
  let a = 0.147;
  let v = sqrt(1.0 - exp(-x2 * (1.2732395 + a * x2) / (1.0 + a * x2)));
  return select(-v, v, x >= 0.0);
}

// A distance's coverage blurred by CSS's blur radius `blur` (a Gaussian of
// deviation blur / 2, as CSS draws a shadow); sharp under half a pixel.
fn blurred(d: f32, blur: f32, scale: f32) -> f32 {
  let sigma = 0.5 * blur;
  if (sigma * scale < 0.5) {
    return edge(d, scale);
  }
  return 0.5 - 0.5 * erf_approx(d / (sigma * 1.4142135));
}

// Radii `r` grown by `by` for a shadow's spread; a square corner stays
// square, as CSS keeps it.
fn grown(r: vec4<f32>, by: f32) -> vec4<f32> {
  return select(vec4<f32>(0.0), max(r + vec4<f32>(by), vec4<f32>(0.0)), r > vec4<f32>(0.0));
}

// Gradient `gi` at `p` in a box of half size `half`: CSS's linear, radial
// or conic gradient, mixed in premultiplied sRGB as CSS mixes it by default.
fn gradient_at(gi: u32, p: vec2<f32>, half: vec2<f32>) -> vec4<f32> {
  let g = gradients[3u * gi];
  let shape = gradients[3u * gi + 1u];
  var t: f32;
  if (g.w > 1.5) {
    // Around the centre, in turns from the start angle (0 up, clockwise).
    let d = p - shape.xy;
    let a = atan2(d.x, -d.y) - g.x;
    t = fract(a / 6.2831853);
  } else if (g.w > 0.5) {
    // Out from the centre: 1 on the ending ellipse.
    t = length((p - shape.xy) / max(shape.zw, vec2<f32>(1e-4)));
  } else {
    let dir = vec2<f32>(sin(g.x), -cos(g.x));
    let len = abs(2.0 * half.x * dir.x) + abs(2.0 * half.y * dir.y);
    t = dot(p, dir) / max(len, 1e-4) + 0.5;
  }
  let first = u32(g.y);
  let count = max(u32(g.z), 1u);
  var lo = 0u;
  for (var k = 1u; k < count; k++) {
    if (stops[2u * (first + k) + 1u].x <= t) {
      lo = k;
    }
  }
  let hi = min(lo + 1u, count - 1u);
  let a = stops[2u * (first + lo)];
  let b = stops[2u * (first + hi)];
  let oa = stops[2u * (first + lo) + 1u].x;
  let ob = stops[2u * (first + hi) + 1u].x;
  let f = select(clamp((t - oa) / max(ob - oa, 1e-6), 0.0, 1.0), 0.0, hi == lo);
  let pa = vec4<f32>(srgb_encode(a.rgb) * a.a, a.a);
  let pb = vec4<f32>(srgb_encode(b.rgb) * b.a, b.a);
  let m = mix(pa, pb, f);
  if (m.a <= 0.0) {
    return vec4<f32>(0.0);
  }
  return vec4<f32>(srgb_decode(m.rgb / m.a), m.a);
}

// One axis of a nine-slice: `x` CSS px into a box `w` wide whose start and
// end slices are `a` and `b` CSS px wide on screen and `sa` and `sb` texture
// pixels in a region `n` pixels long. The texture pixel it samples, kept
// half a texel inside its own slice so filtering never bleeds a neighbour
// in; 1 in the centre; and the texture pixels per CSS px of that slice,
// which the mip level is chosen by (the region's scale across the whole box
// is not any slice's: a wide art squeezed into a chip would blur its edges).
fn slice_axis(x: f32, w: f32, a: f32, b: f32, sa: f32, sb: f32, n: f32, tile: bool) -> vec3<f32> {
  if (x < a) {
    return vec3<f32>(inside(x / max(a, 1e-4) * sa, 0.0, sa), 0.0, sa / max(a, 1e-4));
  }
  if (x > w - b) {
    return vec3<f32>(inside(n - (w - x) / max(b, 1e-4) * sb, n - sb, n), 0.0, sb / max(b, 1e-4));
  }
  let centre = max(n - sa - sb, 1e-4);
  let span = max(w - a - b, 1e-4);
  var t = (x - a) / span;
  var k = centre / span;
  if (tile) {
    // Whole tiles, the source's scale along the slices (CSS's round).
    let s = select(1.0, a / sa, sa > 0.0 && a > 0.0);
    let tiles = max(round(span / (centre * s)), 1.0);
    t = fract(t * tiles);
    k *= tiles;
  }
  return vec3<f32>(inside(sa + t * centre, sa, n - sb), 1.0, k);
}

// `v` kept half a texel inside `lo..hi` (the middle of a slice too thin).
fn inside(v: f32, lo: f32, hi: f32) -> f32 {
  return clamp(v, min(lo + 0.5, 0.5 * (lo + hi)), max(hi - 0.5, 0.5 * (lo + hi)));
}

// The distance from `p` to the segment `a` `b`.
fn segment_distance(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
  let ab = b - a;
  let h = clamp(dot(p - a, ab) / max(dot(ab, ab), 1e-8), 0.0, 1.0);
  return length(p - a - ab * h);
}

fn median(r: f32, g: f32, b: f32) -> f32 {
  return max(min(r, g), min(max(r, g), b));
}

// A built-in icon at `p` (its 24-unit space): the distance to its nearest
// segment less half the 2-unit stroke (round caps and joins).
fn icon_distance(p: vec2<f32>, first: u32, count: u32) -> f32 {
  var best = 1e9;
  for (var k = 0u; k < count; k++) {
    let s = segments[first + k];
    let ab = s.zw - s.xy;
    let h = clamp(dot(p - s.xy, ab) / max(dot(ab, ab), 1e-8), 0.0, 1.0);
    best = min(best, length(p - s.xy - ab * h));
  }
  return best - 1.0;
}

// Skia's preblend of a glyph's coverage `c` for an ink `rgb` (linear):
// the row of the ink's luminance as Skia takes it (its sRGB code values at
// gamma 1.2, the top three bits of eight), the coverage's entry in it.
fn preblend(c: f32, rgb: vec3<f32>) -> f32 {
  let luma = dot(pow(clamp(srgb_encode(rgb), vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>(1.2)), vec3<f32>(0.2126, 0.7152, 0.0722));
  let row = u32(round(pow(luma, 1.0 / 1.2) * 255.0)) >> 5u;
  let i = row * 256u + u32(round(clamp(c, 0.0, 1.0) * 255.0));
  let word = frame.preblend[i / 16u][(i / 4u) % 4u];
  return f32((word >> ((i % 4u) * 8u)) & 0xffu) / 255.0;
}

@fragment
fn fs_paint(in: PaintOut) -> @location(0) vec4<f32> {
  // Sampled before any branch: derivatives need uniform control flow.
  let m = textureSample(page, page_sampler, in.uv);
  let screen_tex = vec2<f32>(1.0) / fwidthFine(in.uv);
  let local_dx = dpdx(in.local);
  let local_dy = dpdy(in.local);
  let q = paint_quad(in.index);
  let css = (in.position.xy + frame.extra.zw) / frame.output.z;
  var cut = css.x < q.clip.x || css.y < q.clip.y || css.x >= q.clip.z || css.y >= q.clip.w;
  let scale = in.aa;
  let half = 0.5 * q.rect.zw;
  var rgb = q.color.rgb;
  var a = q.color.a;
  var dither = false;
  switch q.k.x {
    case 1u: {
      let g = gradient_at(q.k.y, in.local, half);
      rgb = g.rgb;
      a *= g.a;
      dither = true;
      a *= edge(sd_round_box(in.local, half, q.r), scale);
    }
    case 9u: {
      if (q.k.y == 0u) {
        // Coverage through Skia's preblend, then blended in encoded
        // values, as Chrome's text is.
        a *= preblend(m.r, rgb);
      } else if (q.k.y == 1u) {
        // A colour glyph: premultiplied, sRGB-encoded.
        rgb = srgb_decode(m.rgb / max(m.a, 1e-4));
        a *= m.a;
      } else {
        // A distance field, `p.z` atlas pixels across: grown by `p.x`, and
        // blurred by `p.y` (CSS's radius, a deviation of half). Sharp fills
        // use the multi-channel median for crisp corners; strokes and glows
        // use the true-SDF alpha channel (`m.a`) to avoid corner spikes.
        let size = vec2<f32>(textureDimensions(page));
        let px_range = max(0.5 * dot(vec2<f32>(q.p.z) / size, screen_tex), 1.0);
        let grow = min(q.p.x * scale, max(0.5 * px_range - 0.75, 0.0));
        let sd = select(median(m.r, m.g, m.b), m.a, q.p.x > 0.0 || q.p.y > 0.0);
        let d = px_range * (sd - 0.5) + grow;
        if (q.p.y * scale > 1.0) {
          a *= 0.5 + 0.5 * erf_approx(d / (0.5 * q.p.y * scale * 1.4142135));
        } else {
          a *= clamp(d + 0.5, 0.0, 1.0);
        }
      }
    }
    case 3u: {
      let side = max(min(q.rect.z, q.rect.w), 1e-3);
      let p = (in.local + vec2<f32>(0.5 * side)) * (24.0 / side);
      let d = icon_distance(p, q.k.y, q.k.z) * (side / 24.0);
      a *= clamp(0.5 - d * scale, 0.0, 1.0);
    }
    case 4u: {
      // The ring between the box and its inner edge, each side in by its
      // width, each corner's radius less its wider side's.
      let w = q.p;
      let ihalf = half - 0.5 * vec2<f32>(w.w + w.y, w.x + w.z);
      let ic = 0.5 * vec2<f32>(w.w - w.y, w.x - w.z);
      let ir = max(q.r - vec4<f32>(max(w.x, w.w), max(w.x, w.y), max(w.z, w.y), max(w.z, w.w)), vec4<f32>(0.0));
      var inner = 0.0;
      if (ihalf.x > 0.0 && ihalf.y > 0.0) {
        inner = edge(sd_round_box(in.local - ic, ihalf, ir), scale);
      }
      let ring = edge(sd_round_box(in.local, half, q.r), scale) * (1.0 - inner);
      a *= ring;
      if (q.k.z != 0u) {
        // The side a pixel is on: the nearest edge in units of its width,
        // so two sides meet on the line from the outer to the inner corner.
        let e = vec4<f32>(in.local.y + half.y, half.x - in.local.x, half.y - in.local.y, in.local.x + half.x)
          / max(w, vec4<f32>(1e-4));
        var side = 0u;
        var best = e.x;
        if (e.y < best) { best = e.y; side = 1u; }
        if (e.z < best) { best = e.z; side = 2u; }
        if (e.w < best) { side = 3u; }
        if ((q.k.z & (1u << side)) != 0u) {
          rgb = q.uv.rgb;
          a = q.uv.a * ring;
        }
      }
    }
    case 5u: {
      let s = sd_round_box(in.local - q.p.xy, half + vec2<f32>(q.p.w), grown(q.r, q.p.w));
      let body = edge(sd_round_box(in.local, half, q.r), scale);
      a *= blurred(s, q.p.z, scale) * (1.0 - body);
    }
    case 6u: {
      let hole_half = half - vec2<f32>(q.p.w);
      var hole = 0.0;
      if (hole_half.x > 0.0 && hole_half.y > 0.0) {
        hole = blurred(sd_round_box(in.local - q.p.xy, hole_half, grown(q.r, -q.p.w)), q.p.z, scale);
      }
      a *= edge(sd_round_box(in.local, half, q.r), scale) * (1.0 - hole);
    }
    case 7u: {
      // A texture's region, nine-sliced: its colour times the quad's (the
      // ink tints a mask; a picture's is white).
      let size = vec2<f32>(textureDimensions(page));
      let region = (q.uv.zw - q.uv.xy) * size;
      let tile = (q.k.z & 2u) != 0u;
      let fill = (q.k.z & 1u) != 0u;
      let at = in.local + half;
      let x = slice_axis(at.x, 2.0 * half.x, q.r.w, q.r.y, q.p.w, q.p.y, region.x, tile);
      let y = slice_axis(at.y, 2.0 * half.y, q.r.x, q.r.z, q.p.x, q.p.z, region.y, tile);
      // Each axis's slice scale over the fragment's step in the box. A
      // stretched middle is uniform along its stretch (the art is made so),
      // so it filters by the other axis's scale: a squeezed strip would
      // otherwise take a mip that smears the thin edge across it.
      var kx = x.z;
      var ky = y.z;
      if (!tile && x.y > 0.5) { kx = min(kx, ky); }
      if (!tile && y.y > 0.5) { ky = min(ky, x.z); }
      let k = vec2<f32>(kx, ky) / size;
      let s = textureSampleGrad(page, page_sampler, q.uv.xy + vec2<f32>(x.x, y.x) / size, local_dx * k, local_dy * k);
      rgb *= s.rgb;
      a *= s.a;
      if (!fill && x.y > 0.5 && y.y > 0.5) {
        a = 0.0;
      }
      if (q.w.x > 0.0) {
        // The radial wipe: shown from the start angle round to the amount
        // left, its two edges antialiased by their distance in pixels.
        let turn = 6.2831853;
        var t = (atan2(in.local.x, -in.local.y) - q.w.y) / turn;
        t = select(t, -t, q.w.z > 0.5);
        t = fract(t);
        let shown = 1.0 - q.w.x;
        let r = length(in.local) * scale * turn;
        a *= clamp(min(t, shown - t) * r + 0.5, 0.0, 1.0) * select(0.0, 1.0, shown > 0.0);
      }
    }
    case 10u: {
      // A group's filtered layer, or a backdrop's filtered capture
      // (premultiplied, sRGB-encoded), at its opacity; a backdrop cut to
      // its element's rounded box.
      if (m.a <= 0.0) {
        cut = true;
      } else {
        rgb = srgb_decode(m.rgb / m.a);
        a *= m.a;
        if ((q.k.z & 1u) != 0u) {
          a *= edge(sd_round_box(in.local, half, q.r), scale);
        }
      }
    }
    case 8u: {
      // A polyline's segment, round capped: painted only where it is the
      // nearest of itself and its neighbours (a tie to the lower one), so
      // a join draws once.
      let at = q.rect.xy + half + in.local;
      let flags = u32(q.p.y);
      let d = segment_distance(at, q.uv.xy, q.uv.zw);
      if ((flags & 1u) != 0u) {
        let dp = segment_distance(at, q.r.xy, q.uv.xy);
        if (dp < d || ((flags & 4u) != 0u && dp == d)) {
          cut = true;
        }
      }
      if ((flags & 2u) != 0u) {
        let dn = segment_distance(at, q.uv.zw, q.r.zw);
        if (dn < d || ((flags & 8u) != 0u && dn == d)) {
          cut = true;
        }
      }
      a *= edge(d - 0.5 * q.p.x, scale);
    }
    default: {
      a *= edge(sd_round_box(in.local, half, q.r), scale);
    }
  }
  // Its mask's alpha where it falls in the mask's box (CSS's mask-image:
  // drawn where the mask is opaque, not where it is transparent).
  if (!cut && q.w.w > 0.5) {
    let mi = u32(q.w.w) - 1u;
    let box = gradients[3u * mi + 2u];
    let mh = 0.5 * box.zw;
    a *= gradient_at(mi, in.at - box.xy - mh, mh).a;
  }
  // Its rounded clip, at the fragment's place in the clip's own space
  // (through the inverse of its transform), whatever the quad's kind or
  // transform: so an image or a glyph crossing a rounded corner is cut
  // round, antialiased, and a spotlight's shade spares its round hole.
  if (!cut && q.f.y != 0u) {
    let ri = q.f.y - 1u;
    let box = gradients[ri];
    let inv = gradients[ri + 2u];
    let t = gradients[ri + 3u];
    let p = mat2x2<f32>(inv.xy, inv.zw) * css + t.xy;
    let k = frame.output.z / sqrt(max(abs(inv.x * inv.w - inv.y * inv.z), 1e-12));
    var c = edge(sd_round_box(p - 0.5 * (box.xy + box.zw), 0.5 * (box.zw - box.xy), gradients[ri + 1u]), k);
    if (t.z > 0.5) {
      c = 1.0 - c;
    }
    a *= c;
  }
  if (a <= 0.0) {
    cut = true;
  }
  rgb = srgb_encode(rgb);
  // Its colour filters (CSS's brightness(), grayscale() and the rest, on
  // the encoded, unpremultiplied colour), composed into one matrix.
  if (!cut && q.f.x != 0u) {
    let fi = q.f.x - 1u;
    let c = vec4<f32>(rgb, a);
    let o = clamp(
      vec4<f32>(
        dot(gradients[fi], c),
        dot(gradients[fi + 1u], c),
        dot(gradients[fi + 2u], c),
        dot(gradients[fi + 3u], c),
      ) + gradients[fi + 4u],
      vec4<f32>(0.0),
      vec4<f32>(1.0),
    );
    rgb = o.rgb;
    a = o.a;
    if (a <= 0.0) {
      cut = true;
    }
  }
  if (cut) {
    discard;
  }
  if (dither) {
    // A pixel's worth of noise before the target's code values: a shallow
    // ramp shows no bands.
    let n = ign(in.position.xy);
    rgb += vec3<f32>((n - 0.5) * frame.extra.x);
  }
  // Blend mode adaptation (`q.f.z`: 0 normal, 1 add, 2 screen, 3 multiply).
  if (q.f.z == 2u) {
    return vec4<f32>(rgb * a, a);
  }
  if (q.f.z == 3u) {
    return vec4<f32>(mix(vec3<f32>(1.0), rgb, a), a);
  }
  return vec4<f32>(rgb, a);
}

// The pass's layer (bound as the page: premultiplied, sRGB-encoded) over an
// output the pass cannot blend in, an sRGB one with no plain view or a float
// one: a pixel the pass blended keeps its colour. fs_layer_over meets what
// is behind it (a copy, group 2) in encoded values, as an sRGB output's
// blend does; fs_layer, for a target the renderer cannot copy, in the
// output's light.
@vertex
fn vs_layer(@builtin(vertex_index) vi: u32) -> @builtin(position) vec4<f32> {
  return fullscreen_clip(vi);
}

@fragment
fn fs_layer(@builtin(position) at: vec4<f32>) -> @location(0) vec4<f32> {
  let c = textureLoad(page, vec2<i32>(at.xy), 0);
  if (c.a <= 0.0) {
    discard;
  }
  return vec4<f32>(srgb_decode(c.rgb / c.a), c.a);
}

@group(2) @binding(0) var behind: texture_2d<f32>;

// srgb_encode and srgb_decode extend past 0 and 1 and invert each other
// there, so a wide colour (a negative channel) or a highlight behind the UI
// comes back as it was.
@fragment
fn fs_layer_over(@builtin(position) at: vec4<f32>) -> @location(0) vec4<f32> {
  let p = vec2<i32>(at.xy);
  let c = textureLoad(page, p, 0);
  if (c.a <= 0.0) {
    discard;
  }
  let b = textureLoad(behind, p, 0);
  let rgb = c.rgb + srgb_encode(b.rgb) * (1.0 - c.a);
  return vec4<f32>(srgb_decode(rgb), c.a + b.a * (1.0 - c.a));
}
