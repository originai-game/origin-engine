// The UI's paint pass (src/ui/paint.rs): og-ui's paint arena drawn with one
// pipeline per target format, every variant a branch on the instance's kind,
// so nothing is compiled after boot. Each instance is a box in CSS pixels
// (origin top left, y down) with its clip: a rounded box in a colour or a
// gradient, an MSDF glyph of the bound font page, or a built-in icon's
// stroked segments. Colours are linear and straight; an sRGB view encodes in
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
  // x: a box's radius, or a glyph's weight (CSS px).
  p: vec4<f32>,
  // x: the kind (0 solid, 1 gradient, 2 glyph, 3 icon), y and z its
  // gradient, or its first segment and their count.
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
  // A box grows by a device pixel for its antialiased edge; a glyph's quad
  // already holds its field's reach.
  let grow = select(1.0 / max(frame.output.z, 1e-3), 0.0, q.k.x == 2u);
  let half = 0.5 * q.rect.zw;
  let local = (c - vec2<f32>(0.5)) * (q.rect.zw + vec2<f32>(2.0 * grow));
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

// A box of half size `half` rounded by `r`: its signed distance.
fn sd_round_box(p: vec2<f32>, half: vec2<f32>, r: f32) -> f32 {
  let q = abs(p) - half + vec2<f32>(r);
  return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - r;
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
      let d = sd_round_box(in.local, half, min(q.p.x, min(half.x, half.y)));
      a *= clamp(0.5 - d * scale, 0.0, 1.0);
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
    default: {
      let d = sd_round_box(in.local, half, min(q.p.x, min(half.x, half.y)));
      a *= clamp(0.5 - d * scale, 0.0, 1.0);
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
