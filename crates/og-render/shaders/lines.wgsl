// Lines and points (lines.rs, points.rs): one instance per segment or
// point, expanded to a quad in screen space so a width in pixels stays that
// wide at any distance (or, with a world width, shrinks with it), with one
// pixel of feathered edge. Drawn into the HDR target in the main pass (depth
// tested or not) and over the finished frame after post (debug draw and
// screen lines). Colours are linear; dashes measure the distance along the
// line (world units, or CSS pixels for screen lines).

struct LineFrame {
  view_proj: mat4x4<f32>,
  // xy: the target in device pixels, z: device pixels per CSS pixel,
  // w: 1 when the shader encodes sRGB (an output that is not sRGB).
  viewport: vec4<f32>,
  // x: the projection's [1][1] (pixels per world unit at depth 1, halved).
  proj: vec4<f32>,
};
@group(0) @binding(0) var<uniform> lf: LineFrame;

struct LineIn {
  @location(0) a: vec4<f32>,        // xyz: the start (world, or CSS px for screen lines), w: distance along the line there
  @location(1) b: vec4<f32>,        // the end
  @location(2) color_a: vec4<f32>,
  @location(3) color_b: vec4<f32>,
  @location(4) params: vec4<f32>,   // x: width (CSS px, or world units with flag 1), y: flags (1 world width, 2 point, 4 screen), z: dash, w: gap
};

struct LineOut {
  @builtin(position) position: vec4<f32>,
  @location(0) color: vec4<f32>,
  // xy: device pixels from the centre (across a line; both axes of a point), z: half the width in device pixels, w: 1 for a point
  @location(1) across: vec4<f32>,
  @location(2) dist: f32,
  @location(3) dash: vec2<f32>,
};

fn to_clip(p: vec3<f32>, screen: bool) -> vec4<f32> {
  if (screen) {
    let d = p.xy * lf.viewport.z;
    return vec4<f32>(d.x / lf.viewport.x * 2.0 - 1.0, 1.0 - d.y / lf.viewport.y * 2.0, 0.0, 1.0);
  }
  return lf.view_proj * vec4<f32>(p, 1.0);
}

@vertex
fn vs(@builtin(vertex_index) vi: u32, l: LineIn) -> LineOut {
  var corners = array<vec2<f32>, 6>(
    vec2<f32>(0.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0),
    vec2<f32>(0.0, -1.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0));
  let c = corners[vi];
  let flags = u32(l.params.y + 0.5);
  let screen = (flags & 4u) != 0u;
  let point = (flags & 2u) != 0u;
  var o: LineOut;
  var ca = to_clip(l.a.xyz, screen);
  var cb = to_clip(l.b.xyz, screen);
  var da = l.a.w;
  var db = l.b.w;
  // Cut the segment at the near plane (w > 0): a line through the camera
  // keeps its part in front.
  let near = 1e-4;
  if (ca.w < near && cb.w < near) {
    o.position = vec4<f32>(2.0, 2.0, 2.0, 1.0);
    return o;
  }
  if (ca.w < near) {
    let t = (near - ca.w) / (cb.w - ca.w);
    ca = mix(ca, cb, t);
    da = mix(da, db, t);
  } else if (cb.w < near) {
    let t = (near - cb.w) / (ca.w - cb.w);
    cb = mix(cb, ca, t);
    db = mix(db, da, t);
  }
  let half_view = lf.viewport.xy * 0.5;
  let sa = ca.xy / ca.w * half_view;
  let sb = cb.xy / cb.w * half_view;
  let clip = select(ca, cb, c.x > 0.5);
  var width = l.params.x * lf.viewport.z;
  if ((flags & 1u) != 0u) {
    width = l.params.x * lf.proj.x * half_view.y / max(clip.w, near);
  }
  // Half the width plus a pixel of feather.
  let half = 0.5 * width + 1.0;
  var at: vec2<f32>;
  if (point) {
    let offset = vec2<f32>(c.x * 2.0 - 1.0, c.y) * half;
    at = sa + offset;
    o.across = vec4<f32>(offset, 0.5 * width, 1.0);
  } else {
    var dir = sb - sa;
    if (dot(dir, dir) < 1e-8) {
      dir = vec2<f32>(1.0, 0.0);
    }
    let n = normalize(vec2<f32>(-dir.y, dir.x));
    at = mix(sa, sb, c.x) + n * c.y * half;
    o.across = vec4<f32>(c.y * half, 0.0, 0.5 * width, 0.0);
  }
  o.position = vec4<f32>(at / half_view * clip.w, clip.z, clip.w);
  o.color = mix(l.color_a, l.color_b, c.x);
  o.dist = mix(da, db, c.x);
  o.dash = l.params.zw;
  return o;
}

fn encode(c: vec3<f32>) -> vec3<f32> {
  let x = clamp(c, vec3<f32>(0.0), vec3<f32>(1.0));
  let lo = x * 12.92;
  let hi = 1.055 * pow(x, vec3<f32>(1.0 / 2.4)) - 0.055;
  return select(hi, lo, x <= vec3<f32>(0.0031308));
}

@fragment
fn fs(in: LineOut) -> @location(0) vec4<f32> {
  if (in.dash.y > 0.0) {
    let period = in.dash.x + in.dash.y;
    if (in.dist - floor(in.dist / period) * period > in.dash.x) {
      discard;
    }
  }
  var d = abs(in.across.x);
  if (in.across.w > 0.5) {
    d = max(abs(in.across.x), abs(in.across.y));
  }
  var c = in.color;
  c.a *= clamp(in.across.z + 0.5 - d, 0.0, 1.0);
  if (c.a <= 0.0) {
    discard;
  }
  if (lf.viewport.w > 0.5) {
    return vec4<f32>(encode(c.rgb), c.a);
  }
  return c;
}
