// The UI pass's common code (src/ui/mod.rs, docs/ui.md §3.3): the frame
// uniform and the tables every UI and overlay module reads (clip shapes,
// gradients, polyline points), the signed distance functions of the
// shapes, the shadow integrals, OKLab gradients and the instance's vertex
// stage. Lengths are CSS pixels; a shape's local space is centred on the
// instance, y down. A signed distance is negative inside.

struct UiFrame {
  // x, y: the target in device pixels; z: device pixels per CSS pixel;
  // w: 1 when the shader encodes sRGB (a non-sRGB target).
  output: vec4<f32>,
  // xy: the target's top left in CSS px (a layer's rectangle; 0 for the
  // output); z: the UI clock in seconds.
  origin: vec4<f32>,
};
@group(0) @binding(0) var<uniform> frame: UiFrame;

// A clip's shape (UiClip): what uses its clip rectangle is also cut to it.
struct UiClipShape {
  center_half: vec4<f32>,
  // The screen to the local space, a 2x2 matrix (x' = a x + c y, y' = b x + d y).
  inverse: vec4<f32>,
  radii: vec4<f32>,
  geo: vec4<f32>,
  // x: geometry kind, y: corner kind, z: squircle exponent, w: the parent
  // shape's index + 1 (0: none).
  kinds: vec4<f32>,
};
@group(0) @binding(1) var<storage, read> clip_shapes: array<UiClipShape>;

struct UiGradient {
  // x: kind (0 linear, 1 radial, 2 conic, 3 diamond), y: stops, z: spread
  // (0 pad, 1 repeat, 2 reflect), w: 1 to dither.
  head: vec4<f32>,
  // from xy, to xy: fractions of the box.
  geom: vec4<f32>,
  // An absolute box (x, y, width, height in CSS px) when its width is above
  // 0 (text); otherwise the shape's.
  box: vec4<f32>,
  offsets: array<vec4<f32>, 2>,
  // OKLab premultiplied by alpha, and alpha.
  colors: array<vec4<f32>, 8>,
};
@group(0) @binding(2) var<storage, read> gradients: array<UiGradient>;

// Polyline points: xy local, z the length along the line up to the point.
@group(0) @binding(3) var<storage, read> points: array<vec4<f32>>;

// The frame's view depth (the prepass's `lindepth`: r is the depth along
// the camera's forward axis, world units; far without a prepass), for
// occluders (UiOccluder) and mass elements (ui_mass.wgsl).
@group(0) @binding(5) var scene_depth: texture_2d<f32>;

const PI: f32 = 3.14159265;
const TAU: f32 = 6.28318531;

// Geometry kinds, corner kinds, caps and joins (src/ui/build.rs).
const GEO_RECT: u32 = 0u;
const GEO_ELLIPSE: u32 = 1u;
const GEO_ARC: u32 = 2u;
const GEO_POLYGON: u32 = 3u;
const GEO_POLYLINE: u32 = 4u;
// A clip record that is a depth test against the scene (UiOccluder).
const GEO_OCCLUDE: u32 = 5u;
// A vector path (src/ui/path.rs).
const GEO_PATH: u32 = 6u;
const CORNER_ROUND: u32 = 0u;
const CORNER_SQUIRCLE: u32 = 1u;
const CORNER_BEVEL: u32 = 2u;
const CORNER_NOTCH: u32 = 3u;
// CSS corners (UiCorner::Css): elliptical radii and a superellipse per corner.
const CORNER_CSS: u32 = 4u;
const CAP_BUTT: u32 = 0u;
const CAP_ROUND: u32 = 1u;
const CAP_SQUARE: u32 = 2u;
const JOIN_MITER: u32 = 0u;
const JOIN_ROUND: u32 = 1u;
const JOIN_BEVEL: u32 = 2u;
// Modes.
const MODE_FILL: u32 = 0u;
const MODE_STROKE: u32 = 1u;
const MODE_SHADOW: u32 = 2u;
const MODE_INNER_SHADOW: u32 = 3u;
const MODE_BACKDROP: u32 = 4u;
// Paints.
const PAINT_SOLID: u32 = 0u;
const PAINT_GRADIENT: u32 = 1u;
const PAINT_IMAGE: u32 = 2u;
const PAINT_LAYER: u32 = 3u;
// Blends.
const BLEND_NORMAL: u32 = 0u;
const BLEND_MULTIPLY: u32 = 1u;
const BLEND_SCREEN: u32 = 2u;
const BLEND_ADD: u32 = 3u;
const BLEND_OVERLAY: u32 = 4u;
const BLEND_SOFT_LIGHT: u32 = 5u;
const BLEND_COLOR_DODGE: u32 = 6u;

fn encode(c: vec3<f32>) -> vec3<f32> {
  let x = clamp(c, vec3<f32>(0.0), vec3<f32>(1.0));
  let lo = x * 12.92;
  let hi = 1.055 * pow(x, vec3<f32>(1.0 / 2.4)) - 0.055;
  return select(hi, lo, x <= vec3<f32>(0.0031308));
}

fn decode(c: vec3<f32>) -> vec3<f32> {
  let lo = c / 12.92;
  let hi = pow((c + 0.055) / 1.055, vec3<f32>(2.4));
  return select(hi, lo, c <= vec3<f32>(0.04045));
}

// A mask's fraction `m` restated for linear-light blending of the layer's
// colour `rgb`: the alpha drawing the luminance CSS's sRGB blend draws
// with `m` over the colour's opposite (sRGB 1 − its own: white under a
// dark scrim, black under light text), the ground a fade shows against.
// Dark content is close to exact over any ground; mid grey has no
// contrast to fade and keeps `m`. Applying `m` directly fades in linear
// light, where a dark scrim barely darkens until the mask is nearly
// whole and light text stays bright until its very edge: either fade
// reads as a line, not a fade.
fn mask_weight(m: f32, rgb: vec3<f32>) -> f32 {
  let lc = dot(rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
  let cs = encode(vec3<f32>(lc)).x;
  let gs = 1.0 - cs;
  let lg = decode(vec3<f32>(gs)).x;
  if (abs(lc - lg) < 1e-3) {
    return m;
  }
  let drawn = decode(vec3<f32>(mix(gs, cs, m))).x;
  return clamp((drawn - lg) / (lc - lg), 0.0, 1.0);
}

// A text fill's coverage for its ink `rgb`: never less than `fill`,
// raised toward the coverage CSS's sRGB blend shows for dark ink
// (mask_weight). Linear-light coverage thins dark strokes on a light
// ground (a 14 px label on a light primary reads pale grey, though its
// ink is near black) and fattens light ones on a dark ground; only the
// first is a loss of legibility, so light ink keeps its coverage.
fn ink_cover(fill: f32, rgb: vec3<f32>) -> f32 {
  return max(fill, mask_weight(fill, rgb));
}

// The target's encoding of a straight-alpha linear colour.
fn finish_color(rgb: vec3<f32>, a: f32) -> vec4<f32> {
  if (frame.output.w > 0.5) {
    return vec4<f32>(encode(rgb), a);
  }
  return vec4<f32>(rgb, a);
}

// The CSS position of a fragment's device position in this target.
fn css_of(device: vec2<f32>) -> vec2<f32> {
  return device / frame.output.z + frame.origin.xy;
}

fn rotate2(v: vec2<f32>, a: f32) -> vec2<f32> {
  let c = cos(a);
  let s = sin(a);
  return vec2<f32>(c * v.x - s * v.y, s * v.x + c * v.y);
}

fn mat_apply(m: vec4<f32>, v: vec2<f32>) -> vec2<f32> {
  return vec2<f32>(m.x * v.x + m.z * v.y, m.y * v.x + m.w * v.y);
}

// Interleaved gradient noise, 0..1.
fn ign(pixel: vec2<f32>) -> f32 {
  return fract(52.9829189 * fract(dot(pixel, vec2<f32>(0.06711056, 0.00583715))));
}

// ── signed distances ─────────────────────────────────────────────────────

fn sd_box(p: vec2<f32>, b: vec2<f32>) -> f32 {
  let q = abs(p) - b;
  return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0);
}

// The radius of the corner `p` is nearest (radii: tl, tr, br, bl; y down).
fn corner_radius(p: vec2<f32>, radii: vec4<f32>) -> f32 {
  let left = select(radii.w, radii.x, p.y < 0.0);
  let right = select(radii.z, radii.y, p.y < 0.0);
  return select(right, left, p.x < 0.0);
}

// Two 12-bit fractions packed in one float (src/ui/build.rs `pack12`).
fn unpack12(v: f32) -> vec2<f32> {
  let hi = floor(v / 4096.0);
  return vec2<f32>(hi, v - hi * 4096.0) / 4095.0;
}

// CSS corners (CSS Backgrounds 3 §5, CSS Borders 4 corner-shape): the
// corner `p` is nearest, an ellipse quadrant of radii `rx`, `ry` (`ext`
// packs ry as fractions of the half height) shaped as a superellipse of
// parameter K (`ext.w` packs it, 6 bits a corner): K ≥ 0 convex, exponent
// 2^K about the corner's inner centre (0 bevel, 1 round, 2 squircle);
// K < 0 concave, exponent 2^-K about the box's corner (−1 scoop, −∞ notch);
// +∞ square.
fn sd_css_corner(p: vec2<f32>, half: vec2<f32>, radii: vec4<f32>, ext: vec4<f32>) -> f32 {
  let ab = unpack12(ext.y);
  let cd = unpack12(ext.z);
  let ry4 = vec4<f32>(ab.x, ab.y, cd.x, cd.y) * half.y;
  let codes = u32(ext.w);
  // tl, tr, br, bl: 6 bits each, tl lowest.
  let idx = select(select(2u, 3u, p.x < 0.0), select(1u, 0u, p.x < 0.0), p.y < 0.0);
  let code = (codes >> (6u * idx)) & 63u;
  let r = vec2<f32>(clamp(corner_radius(p, radii), 0.0, half.x), clamp(corner_radius(p, ry4), 0.0, half.y));
  let a = abs(p);
  let box = max(a.x - half.x, a.y - half.y);
  let outside = sd_box(p, half);
  if (code == 63u || r.x <= 0.0 || r.y <= 0.0) {
    return outside;
  }
  if (code == 0u) {
    // Notch: the corner's r.x × r.y rectangle taken out.
    let u = half - a;
    return max(outside, min(r.x - u.x, r.y - u.y));
  }
  let k = (f32(code) - 32.0) / 4.0;
  let n = exp2(abs(k));
  if (k >= 0.0) {
    let q = (a - half + r) / r;
    if (q.x > 0.0 && q.y > 0.0) {
      let s = pow(pow(q.x, n) + pow(q.y, n), 1.0 / n);
      let g = vec2<f32>(pow(q.x, n - 1.0), pow(q.y, n - 1.0)) / max(pow(s, n - 1.0), 1e-6) / r;
      return (s - 1.0) / max(length(g), 1e-6);
    }
    return select(box, outside, box > 0.0);
  }
  let u = max(half - a, vec2<f32>(0.0)) / r;
  if (u.x < 1.0 && u.y < 1.0) {
    let s = pow(pow(u.x, n) + pow(u.y, n), 1.0 / n);
    let g = vec2<f32>(pow(u.x, n - 1.0), pow(u.y, n - 1.0)) / max(pow(s, n - 1.0), 1e-6) / r;
    return max(outside, (1.0 - s) / max(length(g), 1e-6));
  }
  return outside;
}

fn sd_rect(p: vec2<f32>, half: vec2<f32>, radii: vec4<f32>, corner: u32, n: f32, ext: vec4<f32>) -> f32 {
  if (corner == CORNER_CSS) {
    return sd_css_corner(p, half, radii, ext);
  }
  let r = clamp(corner_radius(p, radii), 0.0, min(half.x, half.y));
  switch corner {
    case 1u: {
      // Squircle: the corner a superellipse |x|^n + |y|^n = r^n, its
      // distance divided by the norm's gradient (an exact edge at any n).
      let q = abs(p) - half + vec2<f32>(r);
      if (q.x > 0.0 && q.y > 0.0 && r > 0.0) {
        let e = max(n, 1.0);
        let u = q / r;
        let s = pow(pow(u.x, e) + pow(u.y, e), 1.0 / e);
        let g = vec2<f32>(pow(u.x, e - 1.0), pow(u.y, e - 1.0)) / max(pow(s, e - 1.0), 1e-6);
        return (s - 1.0) * r / max(length(g), 1e-4);
      }
      return max(q.x, q.y) - r;
    }
    case 2u: {
      // Bevel: the box cut by a 45° line `r` from the corner along each edge.
      let b = sd_box(p, half);
      let a = abs(p);
      let h = (a.x + a.y - (half.x + half.y - r)) * 0.70710678;
      return max(b, h);
    }
    case 3u: {
      // Notch: a quarter circle of `r` around the corner taken out.
      let b = sd_box(p, half);
      let c = r - length(abs(p) - half);
      return max(b, select(c, -1e9, r <= 0.0));
    }
    default: {
      let q = abs(p) - half + vec2<f32>(r);
      return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - r;
    }
  }
}

// An ellipse's distance (Quilez's first-order approximation, exact on the
// edge's normal to first order: what antialiasing needs).
fn sd_ellipse(p: vec2<f32>, r: vec2<f32>) -> f32 {
  let rr = max(r, vec2<f32>(1e-4));
  let k0 = length(p / rr);
  let k1 = length(p / (rr * rr));
  if (k1 < 1e-6) {
    return -min(rr.x, rr.y);
  }
  return k0 * (k0 - 1.0) / k1;
}

// A ring segment on the circle of radius `outer`: `start` and `sweep`
// radians clockwise on screen, `thickness` inwards.
fn sd_arc(p: vec2<f32>, outer: f32, start: f32, sweep: f32, thickness: f32, cap: u32) -> f32 {
  let t = clamp(thickness, 0.0, outer);
  let rm = outer - 0.5 * t;
  let dr = abs(length(p) - rm) - 0.5 * t;
  if (abs(sweep) >= TAU - 1e-4) {
    return dr;
  }
  // Turned so the arc is symmetric about +x.
  let q = rotate2(p, -(start + 0.5 * sweep));
  let h = 0.5 * abs(sweep);
  let a = atan2(q.y, q.x);
  if (abs(a) <= h) {
    return dr;
  }
  let side = select(-1.0, 1.0, a >= 0.0);
  let u = vec2<f32>(cos(h), side * sin(h));
  if (cap == CAP_ROUND) {
    return length(q - rm * u) - 0.5 * t;
  }
  let w = vec2<f32>(-sin(h), side * cos(h));
  var beyond = dot(q, w);
  if (cap == CAP_SQUARE) {
    beyond -= 0.5 * t;
  }
  let radial = dot(q, u) - rm;
  let b = vec2<f32>(abs(radial) - 0.5 * t, beyond);
  return length(max(b, vec2<f32>(0.0))) + min(max(b.x, b.y), 0.0);
}

// A regular polygon or star of radius `outer` (geo: sides, inner ratio,
// corner rounding, rotation). Returns the distance; `fold` gets the
// position along the outline for dashes.
fn polygon_frame(p: vec2<f32>, outer: f32, geo: vec4<f32>) -> vec4<f32> {
  let n = max(round(geo.x), 3.0);
  let an = PI / n;
  let rounding = clamp(geo.z, 0.0, outer * 0.5);
  // The first vertex at the top, turned by the rotation.
  var a = atan2(p.y, p.x) + 0.5 * PI - geo.w;
  a = a - TAU * floor(a / TAU);
  let half_sector = floor(a / an);
  var f = a - half_sector * an;
  let mirrored = (u32(half_sector) & 1u) == 1u;
  if (mirrored) {
    f = an - f;
  }
  let len = length(p);
  let q = len * vec2<f32>(cos(f), sin(f));
  let rr = outer - rounding;
  let k = clamp(geo.y, 0.0, cos(an));
  let v0 = vec2<f32>(rr, 0.0);
  let v1 = rr * k * vec2<f32>(cos(an), sin(an));
  let e = v1 - v0;
  let t = clamp(dot(q - v0, e) / max(dot(e, e), 1e-8), 0.0, 1.0);
  let dist = length(q - v0 - e * t);
  let c = e.x * (q.y - v0.y) - e.y * (q.x - v0.x);
  let c0 = e.x * (-v0.y) - e.y * (-v0.x);
  let s = select(1.0, -1.0, c * c0 > 0.0);
  let edge = length(e);
  let along = select(half_sector + t, half_sector + 1.0 - t, mirrored) * edge;
  return vec4<f32>(s * dist - rounding, along, edge * 2.0 * n, 0.0);
}

// The distance to a polyline of instance points `first`..`first + count`
// and the length along it at the nearest segment (x: distance, y: along).
fn sd_polyline(p: vec2<f32>, first: u32, count: u32, closed: bool, width: f32, cap: u32, join: u32, limit: f32) -> vec2<f32> {
  let hw = 0.5 * width;
  var best = 1e9;
  var along = 0.0;
  if (count < 2u) {
    return vec2<f32>(best, 0.0);
  }
  let segs = select(count - 1u, count, closed);
  for (var i = 0u; i < segs; i++) {
    let a4 = points[first + i];
    let b4 = points[first + (i + 1u) % count];
    let a = a4.xy;
    let b = b4.xy;
    let ab = b - a;
    let len = max(length(ab), 1e-5);
    let dir = ab / len;
    let nrm = vec2<f32>(-dir.y, dir.x);
    let u = dot(p - a, dir);
    let v = dot(p - a, nrm);
    let open_start = !closed && i == 0u;
    let open_end = !closed && i == segs - 1u;
    // The end types: 0 butt, 1 round, 2 square, 3 a sharp joint (cut below).
    var e0 = 0u;
    var e1 = 0u;
    if (open_start) { e0 = cap; } else if (join == JOIN_ROUND) { e0 = 1u; } else { e0 = 3u; }
    if (open_end) { e1 = cap; } else if (join == JOIN_ROUND) { e1 = 1u; } else { e1 = 3u; }
    let ext0 = select(select(0.0, hw, e0 == 2u), hw * (max(limit, 1.0) + 1.0), e0 == 3u);
    let ext1 = select(select(0.0, hw, e1 == 2u), hw * (max(limit, 1.0) + 1.0), e1 == 3u);
    var d: f32;
    if (e0 == 1u && u < 0.0) {
      d = length(vec2<f32>(u, v)) - hw;
    } else if (e1 == 1u && u > len) {
      d = length(vec2<f32>(u - len, v)) - hw;
    } else {
      let lo = select(-ext0, 0.0, e0 == 1u);
      let hi = select(len + ext1, len, e1 == 1u);
      let alongd = max(lo - u, u - hi);
      let bq = vec2<f32>(alongd, abs(v) - hw);
      d = length(max(bq, vec2<f32>(0.0))) + min(max(bq.x, bq.y), 0.0);
    }
    // Sharp joints: cut at the bisector, and past the miter limit (or at
    // the bevel's line) on the outer side.
    if (e0 == 3u) {
      let pi = select(i - 1u, count - 1u, i == 0u);
      let pdir = normalize(a - points[first + pi].xy + vec2<f32>(1e-9, 0.0));
      d = max(d, joint_cut(p, a, pdir, dir, hw, join, limit, false));
    }
    if (e1 == 3u) {
      let ndir = normalize(points[first + (i + 2u) % count].xy - b + vec2<f32>(1e-9, 0.0));
      d = max(d, joint_cut(p, b, dir, ndir, hw, join, limit, true));
    }
    if (d < best) {
      best = d;
      along = a4.z + clamp(u, 0.0, len);
    }
  }
  return vec2<f32>(best, along);
}

// A filled path's signed distance (geo.z 2): to the closed polygon
// through instance points `first`..`first + count`, negative inside by the
// even-odd rule, and the length along its outline at the nearest edge.
fn sd_filled_path(p: vec2<f32>, first: u32, count: u32) -> vec2<f32> {
  var best = 1e9;
  var along = 0.0;
  var inside = false;
  if (count < 3u) {
    return vec2<f32>(best, 0.0);
  }
  for (var i = 0u; i < count; i++) {
    let a4 = points[first + i];
    let a = a4.xy;
    let b = points[first + (i + 1u) % count].xy;
    let ab = b - a;
    let h = clamp(dot(p - a, ab) / max(dot(ab, ab), 1e-10), 0.0, 1.0);
    let d = length(p - a - ab * h);
    if (d < best) {
      best = d;
      along = a4.z + h * length(ab);
    }
    if ((a.y > p.y) != (b.y > p.y) && p.x < (b.x - a.x) * (p.y - a.y) / (b.y - a.y) + a.x) {
      inside = !inside;
    }
  }
  return vec2<f32>(select(best, -best, inside), along);
}

// The distance from the origin to the quadratic Bézier `a b c` (relative
// to the sample) and the parameter of its nearest point: Inigo Quilez's
// closed form (MIT, third_party/vector-paths/), the nearest of the cubic's
// real roots.
fn dist_quad(a: vec2<f32>, b: vec2<f32>, c: vec2<f32>) -> vec2<f32> {
  let qa = b - a;
  let qb = a - 2.0 * b + c;
  let qc = qa * 2.0;
  let qd = a;
  let kk = 1.0 / dot(qb, qb);
  let kx = kk * dot(qa, qb);
  let ky = kk * (2.0 * dot(qa, qa) + dot(qd, qb)) / 3.0;
  let kz = kk * dot(qd, qa);
  let pp = ky - kx * kx;
  let p3 = pp * pp * pp;
  let q = kx * (2.0 * kx * kx - 3.0 * ky) + kz;
  var h = q * q + 4.0 * p3;
  var res: f32;
  var tt: f32;
  if (h >= 0.0) {
    h = sqrt(h);
    let x = (vec2<f32>(h, -h) - q) * 0.5;
    let uv = sign(x) * pow(abs(x), vec2<f32>(1.0 / 3.0));
    tt = clamp(uv.x + uv.y - kx, 0.0, 1.0);
    let e = qd + (qc + qb * tt) * tt;
    res = dot(e, e);
  } else {
    let z = sqrt(-pp);
    let v = acos(clamp(q / (pp * z * 2.0), -1.0, 1.0)) / 3.0;
    let m = cos(v);
    let n = sin(v) * 1.732050808;
    let t = clamp(vec2<f32>(m + m, -n - m) * z - kx, vec2<f32>(0.0), vec2<f32>(1.0));
    let e0 = qd + (qc + qb * t.x) * t.x;
    let e1 = qd + (qc + qb * t.y) * t.y;
    res = dot(e0, e0);
    tt = t.x;
    if (dot(e1, e1) < res) {
      res = dot(e1, e1);
      tt = t.y;
    }
  }
  // The ends, against the roots' precision on nearly flat curves.
  if (dot(a, a) < res) {
    res = dot(a, a);
    tt = 0.0;
  }
  if (dot(c, c) < res) {
    res = dot(c, c);
    tt = 1.0;
  }
  return vec2<f32>(sqrt(res), tt);
}

// The signed crossings of the ray from the origin to +x with the
// quadratic `a b c`: Slug's root eligibility (MIT or Apache-2.0,
// third_party/vector-paths/), which counts a crossing at a shared
// endpoint once and a tangent touch never.
fn quad_crossings(a: vec2<f32>, b: vec2<f32>, c: vec2<f32>) -> i32 {
  let shift = select(0u, 2u, a.y > 0.0) + select(0u, 4u, b.y > 0.0) + select(0u, 8u, c.y > 0.0);
  let code = (0x2E74u >> shift) & 3u;
  if (code == 0u) {
    return 0;
  }
  let ay = a.y - 2.0 * b.y + c.y;
  let by = a.y - b.y;
  let ax = a.x - 2.0 * b.x + c.x;
  let bx = a.x - b.x;
  var t1: f32;
  var t2: f32;
  if (abs(ay) < 1.0 / 65536.0) {
    t1 = a.y * 0.5 / by;
    t2 = t1;
  } else {
    let d = sqrt(max(by * by - ay * a.y, 0.0));
    t1 = (by - d) / ay;
    t2 = (by + d) / ay;
  }
  var w = 0;
  if ((code & 1u) != 0u && (ax * t1 - bx * 2.0) * t1 + a.x > 0.0) {
    w += 1;
  }
  if (code > 1u && (ax * t2 - bx * 2.0) * t2 + a.x > 0.0) {
    w -= 1;
  }
  return w;
}

// A path's curve: its points relative to a sample, and whether it is a line.
struct PathCurve {
  a: vec2<f32>,
  b: vec2<f32>,
  c: vec2<f32>,
  line: bool,
};

// The band list (its first record, its length) local height `y` is in.
fn path_band(y: f32, geo: vec4<f32>) -> vec2<u32> {
  let nb = u32(geo.z);
  let header = u32(geo.y);
  let top = points[header + 1u].x;
  let band = u32(clamp(floor((y - top) / max(geo.w, 1e-6)), 0.0, f32(nb - 1u)));
  let head = points[header + 2u + band];
  return vec2<u32>(u32(head.x), u32(head.y));
}

// Curve `k` of the band list at `start`, relative to `p`.
fn path_curve(geo: vec4<f32>, start: u32, k: u32, p: vec2<f32>) -> PathCurve {
  let ci = u32(points[start + k / 4u][k % 4u]);
  let r0 = points[u32(geo.x) + 2u * ci];
  let r1 = points[u32(geo.x) + 2u * ci + 1u];
  return PathCurve(r0.xy - p, r0.zw - p, r1.xy - p, r1.z > 0.5);
}

// The winding number at local `p`: a ray to +x through its band.
fn path_winding(p: vec2<f32>, geo: vec4<f32>) -> i32 {
  let band = path_band(p.y, geo);
  var w = 0;
  for (var k = 0u; k < band.y; k++) {
    let q = path_curve(geo, band.x, k, p);
    w += quad_crossings(q.a, q.b, q.c);
  }
  return w;
}

fn path_inside(w: i32, rule: f32) -> bool {
  if (rule > 0.5) {
    return (w & 1) != 0;
  }
  return w != 0;
}

// A path's signed distances (src/ui/path.rs `write`; `sd2` there mirrors
// it): x the fill's, to the curves within the margin where inside meets
// outside, so a line crossing the inside of a nonzero fill is no edge; y
// the line's, to every curve, what strokes follow. Each is signed by the
// winding number of a ray to +x and the fill rule. A curve's sides are
// judged at its nearest point: the winding just on the sample's side of it
// (a second ray from there) and that plus the curve's direction across it.
// geo: the curves' first record, the header's, the bands' count, a band's
// height; `rule` 1 for even-odd.
fn sd_path(p: vec2<f32>, geo: vec4<f32>, rule: f32) -> vec2<f32> {
  if (geo.z < 0.5) {
    return vec2<f32>(1e9);
  }
  let margin = points[u32(geo.y) + 1u].y;
  let band = path_band(p.y, geo);
  let here = path_inside(path_winding(p, geo), rule);
  var fill = 1e9;
  var line = 1e9;
  for (var k = 0u; k < band.y; k++) {
    let q = path_curve(geo, band.x, k, p);
    var d: f32;
    var t: f32;
    if (q.line) {
      let ab = q.c - q.a;
      t = clamp(-dot(q.a, ab) / max(dot(ab, ab), 1e-12), 0.0, 1.0);
      d = length(q.a + ab * t);
    } else {
      let dt = dist_quad(q.a, q.b, q.c);
      d = dt.x;
      t = dt.y;
    }
    line = min(line, d);
    if (d >= fill || d > margin) {
      continue;
    }
    let u = 1.0 - t;
    let at = q.a * (u * u) + q.b * (2.0 * u * t) + q.c * (t * t);
    let tangent = (q.b - q.a) * u + (q.c - q.b) * t;
    let cr = tangent.x * -at.y - tangent.y * -at.x;
    var s = 0;
    if (abs(cr) > 1e-9) {
      s = select(-1, 1, cr > 0.0);
    }
    // Just on the sample's side of the curve's nearest point.
    let near = at * (1.0 - min(0.01 / max(d, 0.01), 1.0));
    let w = path_winding(p + near, geo);
    if (s == 0 || path_inside(w, rule) != path_inside(w + s, rule)) {
      fill = d;
    }
  }
  let sg = select(1.0, -1.0, here);
  return vec2<f32>(sg * fill, sg * line);
}

// The cut of a segment at a sharp joint `j` between directions `d0` and
// `d1`: the bisector (`end`: the segment ends at `j`, else starts there)
// and the miter limit or bevel line on the outer side of the turn.
fn joint_cut(p: vec2<f32>, j: vec2<f32>, d0: vec2<f32>, d1: vec2<f32>, hw: f32, join: u32, limit: f32, end: bool) -> f32 {
  let t = d0 + d1;
  if (dot(t, t) < 1e-8) {
    // A full turn back: a butt end.
    let dd = select(-dot(p - j, d1), dot(p - j, d0), end);
    return dd;
  }
  let m = normalize(t);
  // Each segment reaches a little past the bisector, so the two overlap
  // there instead of each fading to half coverage on it (a seam at every
  // joint of a flattened curve); the miter's own cut below still bounds
  // the outer corner.
  let overlap = min(0.5 * hw, 1.0);
  let bis = select(-dot(p - j, m), dot(p - j, m), end) - overlap;
  let n0 = vec2<f32>(-d0.y, d0.x);
  let n1 = vec2<f32>(-d1.y, d1.x);
  let nm = n0 + n1;
  if (dot(nm, nm) < 1e-8) {
    return bis;
  }
  let turn = d0.x * d1.y - d0.y * d1.x;
  let o = -sign(turn) * normalize(nm);
  let cos_phi = max(dot(n0, normalize(nm)), 1e-4);
  var reach = hw / cos_phi;
  if (join == JOIN_BEVEL) {
    reach = hw * cos_phi;
  } else if (reach > limit * hw) {
    reach = hw * cos_phi;
  }
  return max(bis, dot(p - j, o) - reach);
}

// A filled area's distance: rectangle, ellipse, arc or polygon.
fn sd_area(p: vec2<f32>, kind: u32, corner: u32, cap: u32, half: vec2<f32>, radii: vec4<f32>, geo: vec4<f32>, n: f32) -> f32 {
  switch kind {
    case 1u: { return sd_ellipse(p, half); }
    case 2u: { return sd_arc(p, min(half.x, half.y), geo.x, geo.y, geo.z, cap); }
    case 3u: { return polygon_frame(p, min(half.x, half.y), geo).x; }
    // A rectangle's `geo` is its corners' extension (geo2).
    default: { return sd_rect(p, half, radii, corner, n, geo); }
  }
}

// The length along a closed outline at the nearest point to `p`, for
// dashes: x along, y the outline's whole length (0: no dashes there).
fn outline_along(p: vec2<f32>, kind: u32, cap: u32, half: vec2<f32>, radii: vec4<f32>, geo: vec4<f32>) -> vec2<f32> {
  switch kind {
    case 1u: {
      // Ramanujan's perimeter, spread by the eccentric angle from the top.
      let a = half.x;
      let b = half.y;
      let h = (a - b) * (a - b) / max((a + b) * (a + b), 1e-6);
      let per = PI * (a + b) * (1.0 + 3.0 * h / (10.0 + sqrt(max(4.0 - 3.0 * h, 0.0))));
      var t = atan2(p.y * a, p.x * b) + 0.5 * PI;
      t = t - TAU * floor(t / TAU);
      return vec2<f32>(t / TAU * per, per);
    }
    case 2u: {
      let outer = min(half.x, half.y);
      let rm = outer - 0.5 * clamp(geo.z, 0.0, outer);
      var t = atan2(p.y, p.x) - geo.x;
      t = t - TAU * floor(t / TAU);
      // Past the end of a partial arc: before its start.
      if (abs(geo.y) < TAU && t > abs(geo.y)) {
        let over = t - abs(geo.y);
        if (over > 0.5 * (TAU - abs(geo.y))) {
          t = t - TAU;
        }
      }
      return vec2<f32>(t * rm, abs(geo.y) * rm);
    }
    case 3u: {
      let f = polygon_frame(p, min(half.x, half.y), geo);
      return f.yz;
    }
    default: {
      return rect_along(p, half, radii);
    }
  }
}

// A rounded rectangle's outline from the end of the top left corner,
// clockwise: the nearest of its four edges and four corner arcs.
fn rect_along(p: vec2<f32>, half: vec2<f32>, radii4: vec4<f32>) -> vec2<f32> {
  let m = min(half.x, half.y);
  let r = clamp(radii4, vec4<f32>(0.0), vec4<f32>(m));
  let top = 2.0 * half.x - r.x - r.y;
  let right = 2.0 * half.y - r.y - r.z;
  let bottom = 2.0 * half.x - r.z - r.w;
  let left = 2.0 * half.y - r.w - r.x;
  let q = 0.5 * PI;
  let total = top + right + bottom + left + q * (r.x + r.y + r.z + r.w);
  var best = 1e9;
  var at = 0.0;
  var s = 0.0;
  // Top edge, left to right.
  {
    let t = clamp(p.x - (-half.x + r.x), 0.0, top);
    let d = length(p - vec2<f32>(-half.x + r.x + t, -half.y));
    if (d < best) { best = d; at = s + t; }
    s += top;
  }
  // Top right corner.
  {
    let c = vec2<f32>(half.x - r.y, -half.y + r.y);
    let a = clamp(atan2(p.y - c.y, p.x - c.x) + q, 0.0, q);
    let d = abs(length(p - c) - r.y);
    if (d < best && p.x >= c.x && p.y <= c.y) { best = d; at = s + a * r.y; }
    s += q * r.y;
  }
  // Right edge, top to bottom.
  {
    let t = clamp(p.y - (-half.y + r.y), 0.0, right);
    let d = length(p - vec2<f32>(half.x, -half.y + r.y + t));
    if (d < best) { best = d; at = s + t; }
    s += right;
  }
  // Bottom right corner.
  {
    let c = vec2<f32>(half.x - r.z, half.y - r.z);
    let a = clamp(atan2(p.y - c.y, p.x - c.x), 0.0, q);
    let d = abs(length(p - c) - r.z);
    if (d < best && p.x >= c.x && p.y >= c.y) { best = d; at = s + a * r.z; }
    s += q * r.z;
  }
  // Bottom edge, right to left.
  {
    let t = clamp((half.x - r.z) - p.x, 0.0, bottom);
    let d = length(p - vec2<f32>(half.x - r.z - t, half.y));
    if (d < best) { best = d; at = s + t; }
    s += bottom;
  }
  // Bottom left corner.
  {
    let c = vec2<f32>(-half.x + r.w, half.y - r.w);
    let a = clamp(atan2(p.y - c.y, p.x - c.x) - q, 0.0, q);
    let d = abs(length(p - c) - r.w);
    if (d < best && p.x <= c.x && p.y >= c.y) { best = d; at = s + a * r.w; }
    s += q * r.w;
  }
  // Left edge, bottom to top.
  {
    let t = clamp((half.y - r.w) - p.y, 0.0, left);
    let d = length(p - vec2<f32>(-half.x, half.y - r.w - t));
    if (d < best) { best = d; at = s + t; }
    s += left;
  }
  // Top left corner.
  {
    let c = vec2<f32>(-half.x + r.x, -half.y + r.x);
    var a = atan2(p.y - c.y, p.x - c.x);
    a = clamp(select(a, a + TAU, a < 0.0) - PI, 0.0, q);
    let d = abs(length(p - c) - r.x);
    if (d < best && p.x <= c.x && p.y <= c.y) { best = d; at = s + a * r.x; }
  }
  return vec2<f32>(at, total);
}

// ── clips ────────────────────────────────────────────────────────────────

// The coverage of clip shape `head` (index + 1, 0: none) and its parents at
// CSS position `css`.
fn clip_coverage(head: u32, css: vec2<f32>) -> f32 {
  var k = head;
  var c = 1.0;
  for (var i = 0; i < 8 && k != 0u; i++) {
    let s = clip_shapes[k - 1u];
    if (u32(s.kinds.x) == GEO_OCCLUDE) {
      // center_half: the point and the overlay's CSS size; geo: depth,
      // alpha behind the scene, bias, softness.
      let v = scene_visibility(s.center_half.xy, s.center_half.zw, s.geo.x, s.geo.z, s.geo.w);
      c *= mix(s.geo.y, 1.0, v);
      k = u32(s.kinds.w);
      continue;
    }
    let local = mat_apply(s.inverse, css - s.center_half.xy);
    let det = abs(s.inverse.x * s.inverse.w - s.inverse.y * s.inverse.z);
    let px = sqrt(max(det, 1e-8)) / frame.output.z;
    let kinds = vec4<u32>(s.kinds);
    var d: f32;
    if (kinds.x == 6u) {
      // A path (geo its bands, kinds.z its fill rule): the fill's distance.
      d = sd_path(local, s.geo, s.kinds.z).x;
    } else {
      d = sd_area(local, kinds.x, kinds.y, CAP_BUTT, s.center_half.zw, s.radii, s.geo, s.kinds.z);
    }
    c *= clamp(0.5 - d / max(px, 1e-6), 0.0, 1.0);
    k = kinds.w;
  }
  return c;
}

// How much of the scene around `css` (CSS px of an overlay `size` CSS px)
// is behind a point `depth` world units along the camera's forward axis:
// five taps two CSS px apart, each 1 where the scene is no nearer than the
// depth less `bias`, fading to 0 over `soft` world units nearer. A point
// off the frame is visible.
fn scene_visibility(css: vec2<f32>, size: vec2<f32>, depth: f32, bias: f32, soft: f32) -> f32 {
  let dims = vec2<f32>(textureDimensions(scene_depth));
  let k = dims / max(size, vec2<f32>(1.0));
  let p = css * k;
  if (p.x < 0.0 || p.y < 0.0 || p.x >= dims.x || p.y >= dims.y) {
    return 1.0;
  }
  var taps = array<vec2<f32>, 5>(
    vec2<f32>(0.0, 0.0), vec2<f32>(2.0, 0.0), vec2<f32>(-2.0, 0.0),
    vec2<f32>(0.0, 2.0), vec2<f32>(0.0, -2.0));
  var v = 0.0;
  for (var i = 0; i < 5; i++) {
    let q = clamp(p + taps[i] * k, vec2<f32>(0.0), dims - vec2<f32>(1.0));
    let d = textureLoad(scene_depth, vec2<i32>(q), 0).r;
    v += clamp((d - (depth - bias)) / max(soft, 1e-3) + 1.0, 0.0, 1.0);
  }
  return v / 5.0;
}

// ── shadows ──────────────────────────────────────────────────────────────

fn erf2(x: vec2<f32>) -> vec2<f32> {
  let s = sign(x);
  let a = abs(x);
  var y = 1.0 + (0.278393 + (0.230389 + 0.078108 * (a * a)) * a) * a;
  y *= y;
  return s - s / (y * y);
}

fn erf1(x: f32) -> f32 {
  return erf2(vec2<f32>(x, 0.0)).x;
}

fn gaussian(x: f32, sigma: f32) -> f32 {
  return exp(-(x * x) / (2.0 * sigma * sigma)) / (2.50662827 * sigma);
}

fn box_shadow_x(x: f32, y: f32, sigma: f32, corner: f32, half: vec2<f32>) -> f32 {
  let delta = min(half.y - corner - abs(y), 0.0);
  let curved = half.x - corner + sqrt(max(0.0, corner * corner - delta * delta));
  let integral = 0.5 + 0.5 * erf2((x + vec2<f32>(-curved, curved)) * (0.70710678 / sigma));
  return integral.y - integral.x;
}

// A rounded box of `half` and `corner` blurred by a Gaussian of `sigma`,
// at `p` (Evan Wallace's closed form across x, four samples across y).
fn rounded_box_shadow(p: vec2<f32>, half: vec2<f32>, corner: f32, sigma: f32) -> f32 {
  let low = p.y - half.y;
  let high = p.y + half.y;
  let start = clamp(-3.0 * sigma, low, high);
  let end = clamp(3.0 * sigma, low, high);
  let step = (end - start) / 4.0;
  var y = start + step * 0.5;
  var value = 0.0;
  for (var i = 0; i < 4; i++) {
    value += box_shadow_x(p.x, p.y - y, sigma, corner, half) * gaussian(y, sigma) * step;
    y += step;
  }
  return value;
}

// ── gradients ────────────────────────────────────────────────────────────

fn oklab_to_linear(lab: vec3<f32>) -> vec3<f32> {
  let l_ = lab.x + 0.3963377774 * lab.y + 0.2158037573 * lab.z;
  let m_ = lab.x - 0.1055613458 * lab.y - 0.0638541728 * lab.z;
  let s_ = lab.x - 0.0894841775 * lab.y - 1.2914855480 * lab.z;
  let l = l_ * l_ * l_;
  let m = m_ * m_ * m_;
  let s = s_ * s_ * s_;
  return vec3<f32>(
    4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
    -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
    -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s);
}

fn stop_offset(g: UiGradient, i: u32) -> f32 {
  return g.offsets[i / 4u][i % 4u];
}

// Gradient `gi` at `f` (fractions of a box of `size` CSS px), straight
// alpha, linear; `pixel` the device pixel, for the dither.
fn gradient_color(gi: u32, f: vec2<f32>, size: vec2<f32>, pixel: vec2<f32>) -> vec4<f32> {
  let g = gradients[gi];
  let kind = u32(g.head.x);
  let count = u32(g.head.y);
  let v = (f - g.geom.xy) * size;
  var t = 0.0;
  switch kind {
    case 1u: {
      t = length(v / max(abs(g.geom.zw * size), vec2<f32>(1e-4)));
    }
    case 2u: {
      var a = atan2(v.y, v.x) - g.geom.z;
      t = (a - TAU * floor(a / TAU)) / TAU;
    }
    case 3u: {
      let r = max(abs(g.geom.zw * size), vec2<f32>(1e-4));
      t = abs(v.x) / r.x + abs(v.y) / r.y;
    }
    default: {
      let d = (g.geom.zw - g.geom.xy) * size;
      t = dot(v, d) / max(dot(d, d), 1e-8);
    }
  }
  let spread = u32(g.head.z);
  if (spread == 1u) {
    t = fract(t);
  } else if (spread == 2u) {
    t = 1.0 - abs(fract(t * 0.5) * 2.0 - 1.0);
  }
  var c = g.colors[0];
  if (count > 1u && t > stop_offset(g, 0u)) {
    c = g.colors[count - 1u];
    for (var i = 1u; i < count; i++) {
      let o1 = stop_offset(g, i);
      if (t <= o1) {
        let o0 = stop_offset(g, i - 1u);
        let k = clamp((t - o0) / max(o1 - o0, 1e-6), 0.0, 1.0);
        c = mix(g.colors[i - 1u], g.colors[i], k);
        break;
      }
    }
  }
  var rgb = oklab_to_linear(c.xyz / max(c.w, 1e-6));
  rgb = max(rgb, vec3<f32>(0.0));
  if (g.head.w > 0.5) {
    // Half a code value of noise in the encoded output, so 8 bits show no bands.
    let n = (ign(pixel) - 0.5) / 255.0;
    rgb = decode(clamp(encode(rgb) + vec3<f32>(n), vec3<f32>(0.0), vec3<f32>(1.0)));
  }
  return vec4<f32>(rgb, c.w);
}

// ── the instance ─────────────────────────────────────────────────────────

// One shape instance (src/ui/build.rs `UiGpu`).
struct UiInstance {
  center_half: vec4<f32>,
  // The local space to the screen, 2x2 (x' = a x + c y, y' = b x + d y).
  xform: vec4<f32>,
  // Rectangle: radii (tl, tr, br, bl); arc: start, sweep, thickness;
  // polygon: sides, inner, rounding, rotation; polyline: first, count,
  // closed. Vector: its view box.
  geo: vec4<f32>,
  // x: squircle exponent; CSS corners: y, z the vertical radii (pack12),
  // w each corner's shape; polyline: y width, z miter limit.
  geo2: vec4<f32>,
  // Stroke: width, offset of its centre from the outline, dash, gap.
  // Shadow: blur, spread, offset x, y. Backdrop: blur, saturation,
  // brightness, the hairline's width.
  mode: vec4<f32>,
  // Stroke: x dash offset, y dash cap. Backdrop: the hairline's colour.
  mode2: vec4<f32>,
  // Solid colour, image tint, backdrop tint.
  color: vec4<f32>,
  // Image: uv rect. Backdrop: its first shadow's offset xy, blur, spread.
  paint: vec4<f32>,
  // Image: focal point xy, its size in pixels zw. Pattern: tile wh,
  // rotation. Backdrop: the first shadow's colour.
  paint2: vec4<f32>,
  // Nine-slice: source border (px). Pattern: offset xy, scroll xy.
  // Backdrop: the second shadow.
  paint3: vec4<f32>,
  // Nine-slice: border drawn (CSS px). Backdrop: the second shadow's colour.
  paint4: vec4<f32>,
  // The clip rectangle x0, y0, x1, y1 (CSS px).
  clip: vec4<f32>,
  // x: geometry | corner << 4 | cap << 8 | join << 12; y: paint | fit << 4
  // | edges << 8 | centre << 12; z: mode | blend << 4; w: clip shape + 1.
  kinds: vec4<u32>,
  // x: opacity, y: gradient index (a vector's gradient base), z: a
  // backdrop's flags (1 solid, 2 mixed in sRGB-encoded values), w: layer.
  extra: vec4<f32>,
};
@group(0) @binding(4) var<storage, read> instances: array<UiInstance>;

struct UiOut {
  @builtin(position) position: vec4<f32>,
  @location(0) local: vec2<f32>,
  @location(1) @interpolate(flat) instance: u32,
  // A vector's paint (its gradient's index past the instance's base).
  @location(2) @interpolate(flat) vpaint: u32,
  // A vector's edge coverage, 1 inside, 0 at the fringe's outer side.
  @location(3) aa: f32,
};

fn device_position(css: vec2<f32>) -> vec4<f32> {
  let device = (css - frame.origin.xy) * frame.output.z;
  return vec4<f32>(device.x / frame.output.x * 2.0 - 1.0, 1.0 - device.y / frame.output.y * 2.0, 0.0, 1.0);
}

// The quad around a shape: its half extent grown by what its mode draws
// outside it and by antialiasing.
@vertex
fn vs_shape(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> UiOut {
  var corners = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0),
    vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, 1.0), vec2<f32>(-1.0, 1.0));
  let s = instances[ii];
  let mode = s.kinds.z & 15u;
  let det = abs(s.xform.x * s.xform.w - s.xform.y * s.xform.z);
  // Two device pixels in local units, for the antialiased edge.
  let aa = 2.0 / (frame.output.z * sqrt(max(det, 1e-8)));
  var grow = aa;
  if (mode == MODE_STROKE) {
    grow += max(s.mode.x * 0.5 + s.mode.y, 0.0);
  } else if (mode == MODE_SHADOW) {
    grow += 1.5 * max(s.mode.x, 0.0) + max(s.mode.y, 0.0) + max(abs(s.mode.z), abs(s.mode.w));
  } else if (mode == MODE_BACKDROP) {
    // The glass's shadows, outside it (none: the quad as it was).
    grow += max(glass_shadow_reach(s.paint, s.paint2.a), glass_shadow_reach(s.paint3, s.paint4.a));
  }
  var local = corners[vi] * (s.center_half.zw + vec2<f32>(grow));
  if ((s.kinds.x & 15u) == GEO_PATH) {
    // A path's quad is its curves' box (its header), not the paint's.
    if (s.geo.z < 0.5) {
      local = vec2<f32>(0.0);
    } else {
      let b = points[u32(s.geo.y)];
      local = mix(b.xy - vec2<f32>(grow), b.zw + vec2<f32>(grow), corners[vi] * 0.5 + 0.5);
    }
  }
  var o: UiOut;
  o.position = device_position(s.center_half.xy + mat_apply(s.xform, local));
  o.local = local;
  o.instance = ii;
  o.vpaint = 0u;
  o.aa = 1.0;
  return o;
}

// How far past the shape a glass shadow (offset xy, blur, spread) of
// alpha `a` reaches; 0 for a clear one.
fn glass_shadow_reach(sh: vec4<f32>, a: f32) -> f32 {
  if (a <= 0.0) {
    return 0.0;
  }
  return max(1.5 * max(sh.z, 0.0) + sh.w + max(abs(sh.x), abs(sh.y)), 0.0);
}

// Local units per device pixel at this fragment (call in uniform control flow).
fn local_px(local: vec2<f32>) -> f32 {
  return max(0.5 * (length(dpdxFine(local)) + length(dpdyFine(local))), 1e-6);
}

// The shape's distance at `p` (x) and the length along a polyline there
// (y); a path's y is its line's distance (sd_path).
fn shape_distance(s: UiInstance, p: vec2<f32>) -> vec2<f32> {
  let kind = s.kinds.x & 15u;
  let corner = (s.kinds.x >> 4u) & 15u;
  let cap = (s.kinds.x >> 8u) & 15u;
  let join = (s.kinds.x >> 12u) & 15u;
  if (kind == GEO_POLYLINE && s.geo.z > 1.5) {
    return sd_filled_path(p, u32(s.geo.x), u32(s.geo.y));
  }
  if (kind == GEO_PATH) {
    return sd_path(p, s.geo, s.geo2.x);
  }
  if (kind == GEO_POLYLINE) {
    return sd_polyline(p, u32(s.geo.x), u32(s.geo.y), s.geo.z > 0.5, s.geo2.y, cap, join, s.geo2.z);
  }
  return vec2<f32>(sd_area(p, kind, corner, cap, s.center_half.zw, s.geo, select(s.geo, s.geo2, kind == GEO_RECT), s.geo2.x), 0.0);
}

// What the instance's mode covers at local `p`, 0..1 (`px`: local units
// per device pixel).
fn mode_coverage(s: UiInstance, p: vec2<f32>, px: f32) -> f32 {
  let mode = s.kinds.z & 15u;
  let kind = s.kinds.x & 15u;
  if (mode == MODE_SHADOW || mode == MODE_INNER_SHADOW) {
    let blur = max(s.mode.x, 0.0);
    let inner = mode == MODE_INNER_SHADOW;
    let spread = select(s.mode.y, -s.mode.y, inner);
    let q = p - s.mode.zw;
    let sigma = max(0.5 * blur, 0.5 * px);
    var shadow: f32;
    let corner = (s.kinds.x >> 4u) & 15u;
    if (kind == GEO_RECT && corner == CORNER_ROUND) {
      let half = max(s.center_half.zw + vec2<f32>(spread), vec2<f32>(0.0));
      let r = clamp(corner_radius(q, s.geo) + spread, 0.0, min(half.x, half.y));
      shadow = rounded_box_shadow(q, half, r, sigma);
    } else {
      let d = shape_distance(s, q).x - spread;
      shadow = 0.5 - 0.5 * erf1(d * 0.70710678 / sigma);
    }
    if (!inner) {
      return clamp(shadow, 0.0, 1.0);
    }
    let inside = clamp(0.5 - shape_distance(s, p).x / px, 0.0, 1.0);
    return inside * clamp(1.0 - shadow, 0.0, 1.0);
  }
  let sd = shape_distance(s, p);
  let cap = (s.kinds.x >> 8u) & 15u;
  // A filled path is an area: strokes outline it, as any other shape's.
  let filled = kind == GEO_POLYLINE && s.geo.z > 1.5;
  if (mode == MODE_STROKE && (kind != GEO_POLYLINE || filled)) {
    // A band of the width around the outline, its centre moved by the alignment.
    let hw = 0.5 * max(s.mode.x, 0.0);
    // A path's strokes follow every curve (y), self-crossings included.
    let band = abs(select(sd.x, sd.y, kind == GEO_PATH) - s.mode.y);
    var d = band - hw;
    if (s.mode.z > 0.0 && filled) {
      d = dashed(d, band, hw, sd.y, s.mode.z, s.mode.w, s.mode2.x, u32(s.mode2.y), 0.0);
    } else if (s.mode.z > 0.0 && kind != GEO_PATH) {
      let a = outline_along(p, kind, cap, s.center_half.zw, s.geo, s.geo);
      // Every outline is closed but a partial arc's.
      let open = kind == GEO_ARC && abs(s.geo.y) < TAU;
      d = dashed(d, band, hw, a.x, s.mode.z, s.mode.w, s.mode2.x, u32(s.mode2.y), select(a.y, 0.0, open));
    }
    return clamp(0.5 - d / px, 0.0, 1.0);
  }
  if (kind == GEO_POLYLINE && !filled && s.mode.z > 0.0) {
    let hw = 0.5 * s.geo2.y;
    let d = dashed(sd.x, sd.x + hw, hw, sd.y, s.mode.z, s.mode.w, s.mode2.x, u32(s.mode2.y), 0.0);
    return clamp(0.5 - d / px, 0.0, 1.0);
  }
  return clamp(0.5 - sd.x / px, 0.0, 1.0);
}

// A stroke's distance `d` (`band` the distance from its centre line, `hw`
// half its width) cut into dashes along the outline at `along`.
// On a closed outline of length `len` (0: open) the dashes are cut to
// [0, len], as SVG's are, and the last dash and the first are also measured
// across the outline's start, so a cap that crosses it draws whole on both
// sides when the length is no whole number of periods.
fn dashed(d: f32, band: f32, hw: f32, along: f32, dash: f32, gap: f32, offset: f32, cap: u32, len: f32) -> f32 {
  let period = max(dash + max(gap, 0.0), 1e-3);
  // Along the outline from the nearest dash (either side, so a cap before a
  // dash's start is not lost to the wrap); negative inside it.
  var m = along + offset - 0.5 * dash;
  m = m - period * floor(m / period + 0.5);
  var ad = abs(m) - 0.5 * dash;
  if (len > 0.0) {
    // The nearest dash cut to the outline, then the last dash (starting
    // before len) and the first (ending after 0), here and one lap away.
    let s = along - m - 0.5 * dash;
    ad = dash_span(along, s, s + dash, len);
    let sl = (ceil((len + offset) / period) - 1.0) * period - offset;
    let sf = (floor((offset - dash) / period) + 1.0) * period - offset;
    ad = min(ad, min(dash_span(along, sl, sl + dash, len), dash_span(along + len, sl, sl + dash, len)));
    ad = min(ad, min(dash_span(along, sf, sf + dash, len), dash_span(along - len, sf, sf + dash, len)));
  }
  // Caps reach half the width past the dash's ends, as SVG's do.
  if (cap == CAP_ROUND) {
    let dd = ad;
    let b = max(band, 0.0);
    return length(max(vec2<f32>(dd, b), vec2<f32>(0.0))) + min(max(dd, b), 0.0) - hw;
  }
  let ext = select(0.0, hw, cap == CAP_SQUARE);
  let dd = ad - ext;
  return max(d, dd);
}

// The distance along an outline of length `len` from `x` to the dash
// [s, e] cut to [0, len] (negative inside it; far when nothing is left).
fn dash_span(x: f32, s: f32, e: f32, len: f32) -> f32 {
  let a = max(s, 0.0);
  let b = min(e, len);
  if (a > b) {
    return 1e9;
  }
  return max(a - x, x - b);
}

// The box fraction of a local position.
fn box_fraction(s: UiInstance, p: vec2<f32>) -> vec2<f32> {
  return (p + s.center_half.zw) / max(2.0 * s.center_half.zw, vec2<f32>(1e-4));
}

// Gradient `gi` for a shape at local `p` (or over its own absolute box),
// at device pixel `pixel`.
fn shape_gradient(gi: u32, s: UiInstance, p: vec2<f32>, pixel: vec2<f32>) -> vec4<f32> {
  let g = gradients[gi];
  if (g.box.z > 0.0) {
    let css = css_of(pixel);
    return gradient_color(gi, (css - g.box.xy) / g.box.zw, g.box.zw, pixel);
  }
  return gradient_color(gi, box_fraction(s, p), 2.0 * s.center_half.zw, pixel);
}

// A solid or gradient paint at local `p`, straight alpha.
fn flat_paint(s: UiInstance, p: vec2<f32>, pixel: vec2<f32>) -> vec4<f32> {
  if ((s.kinds.y & 15u) == PAINT_GRADIENT) {
    let c = shape_gradient(u32(s.extra.y), s, p, pixel);
    return vec4<f32>(c.rgb, c.a * s.color.a);
  }
  return s.color;
}

// The coverage of the clip rectangle and clip shape at device pixel `pixel`.
fn instance_clip(s: UiInstance, pixel: vec2<f32>) -> f32 {
  let css = css_of(pixel);
  if (css.x < s.clip.x || css.y < s.clip.y || css.x >= s.clip.z || css.y >= s.clip.w) {
    return 0.0;
  }
  return clip_coverage(s.kinds.w, css);
}
