// The UI's paint pass (src/ui/paint.rs): og-ui's paint arena drawn with one
// pipeline per target format, every variant a branch on the instance's kind,
// so nothing is compiled after boot. Each instance is a box in CSS pixels
// (origin top left, y down) with its clip: a box rounded per corner in a
// colour or a gradient, a border ring, an outer or inset Gaussian box
// shadow, an MSDF glyph of the bound font page, or a built-in icon's stroked
// segments. Colours are linear and straight; an sRGB view encodes in
// hardware, any other output here.

struct PaintFrame {
  // xy: the target in device pixels; z: device pixels per CSS pixel; w: 1
  // when the shader encodes sRGB.
  output: vec4<f32>,
  // x: one code value of the output, encoded (a gradient's dither).
  extra: vec4<f32>,
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
  // 6 inset shadow), y and z its gradient, or its first segment and their
  // count.
  k: vec4<u32>,
};
@group(0) @binding(1) var<storage, read> quads: array<PaintGpu>;
// x: the angle (radians, CSS: 0 up, clockwise), y: its first stop, z: stops.
@group(0) @binding(2) var<storage, read> gradients: array<vec4<f32>>;
// Two entries a stop: its colour (linear, straight), then its offset in x.
@group(0) @binding(3) var<storage, read> stops: array<vec4<f32>>;
// The built-in icons' segments, a b in their 24-unit space.
@group(0) @binding(4) var<storage, read> segments: array<vec4<f32>>;

@group(1) @binding(0) var page: texture_2d<f32>;
@group(1) @binding(1) var page_sampler: sampler;
// x: the distance range in atlas pixels, yz: the atlas size.
@group(1) @binding(2) var<uniform> page_params: vec4<f32>;

struct PaintOut {
  @builtin(position) position: vec4<f32>,
  // The fragment's place in the box, CSS px from its centre.
  @location(0) local: vec2<f32>,
  @location(1) uv: vec2<f32>,
  @location(2) @interpolate(flat) index: u32,
};

@vertex
fn vs_paint(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> PaintOut {
  var corners = array<vec2<f32>, 6>(
    vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0),
    vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0));
  let q = quads[ii];
  let c = corners[vi];
  // What the quad covers around the box's centre: a box and a device pixel
  // for its antialiased edge; a glyph's quad, which holds its field's reach;
  // a shadow's moved, grown box and its blur's reach (three deviations).
  let px = 1.0 / max(frame.output.z, 1e-3);
  let half = 0.5 * q.rect.zw;
  var lo = -half - vec2<f32>(px);
  var hi = half + vec2<f32>(px);
  if (q.k.x == 2u) {
    lo = -half;
    hi = half;
  } else if (q.k.x == 5u) {
    let reach = max(half + vec2<f32>(q.p.w + 1.5 * q.p.z + px), vec2<f32>(0.0));
    lo = q.p.xy - reach;
    hi = q.p.xy + reach;
  }
  let local = mix(lo, hi, c);
  let css = q.rect.xy + half + local;
  let device = css * frame.output.z;
  var o: PaintOut;
  o.position = vec4<f32>(device.x / frame.output.x * 2.0 - 1.0, 1.0 - device.y / frame.output.y * 2.0, 0.0, 1.0);
  o.local = local;
  o.uv = vec2<f32>(mix(q.uv.x, q.uv.z, c.x), mix(q.uv.y, q.uv.w, c.y));
  o.index = ii;
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

// Gradient `gi` at `p` in a box of half size `half`: CSS's linear
// gradient, mixed in premultiplied sRGB as CSS mixes it by default.
fn gradient_at(gi: u32, p: vec2<f32>, half: vec2<f32>) -> vec4<f32> {
  let g = gradients[gi];
  let dir = vec2<f32>(sin(g.x), -cos(g.x));
  let len = abs(2.0 * half.x * dir.x) + abs(2.0 * half.y * dir.y);
  let t = dot(p, dir) / max(len, 1e-4) + 0.5;
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

@fragment
fn fs_paint(in: PaintOut) -> @location(0) vec4<f32> {
  // Sampled before any branch: derivatives need uniform control flow.
  let m = textureSample(page, page_sampler, in.uv);
  let screen_tex = vec2<f32>(1.0) / fwidthFine(in.uv);
  let q = quads[in.index];
  let css = in.position.xy / frame.output.z;
  if (css.x < q.clip.x || css.y < q.clip.y || css.x >= q.clip.z || css.y >= q.clip.w) {
    discard;
  }
  let scale = frame.output.z;
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
    case 2u: {
      let unit_range = vec2<f32>(page_params.x) / max(page_params.yz, vec2<f32>(1.0));
      let px_range = max(0.5 * dot(unit_range, screen_tex), 1.0);
      let weight = min(q.p.x * scale, max(0.5 * px_range - 0.75, 0.0));
      let d = px_range * (median(m.r, m.g, m.b) - 0.5) + weight;
      if (page_params.x <= 0.0) {
        // A colour page: its images, at the text's alpha.
        rgb = m.rgb;
        a *= m.a;
      } else {
        a *= clamp(d + 0.5, 0.0, 1.0);
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
      a *= edge(sd_round_box(in.local, half, q.r), scale) * (1.0 - inner);
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
    default: {
      a *= edge(sd_round_box(in.local, half, q.r), scale);
    }
  }
  if (a <= 0.0) {
    discard;
  }
  if (frame.output.w > 0.5) {
    rgb = srgb_encode(rgb);
    if (dither) {
      // A pixel's worth of noise before the output's code values: a
      // shallow ramp shows no bands.
      let n = fract(52.9829189 * fract(dot(in.position.xy, vec2<f32>(0.06711056, 0.00583715))));
      rgb += vec3<f32>((n - 0.5) * frame.extra.x);
    }
  }
  return vec4<f32>(rgb, a);
}
