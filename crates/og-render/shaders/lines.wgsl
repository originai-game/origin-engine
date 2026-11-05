// Lines and points (lines.rs, points.rs): one instance per segment or
// point, expanded to a quad in screen space so a width in pixels stays that
// wide at any distance (or, with a world width, shrinks with it), with one
// pixel of feathered edge. Drawn into the HDR target in the main pass (depth
// tested or not) and over the finished frame after post (debug draw and
// screen lines). Colours are linear; dashes measure the distance along the
// line (world units, or CSS pixels for screen lines).
//
// A wireframe's edges (lines.rs `WireInstance`, `vs_wire` and `fs_wire`)
// draw with their material's standard group (material.wgsl, prepended) and
// take the colour the surface shows along the edge: its colour times its
// map and the vertex colours, plus its emissive light times its emissive
// map, sampled at the edge's uvs; or one line colour (wireframeColor).

// The standard material group's maps (material_maps.wgsl's bindings).
@group(1) @binding(1) var base_color_tex: texture_2d<f32>;
@group(1) @binding(5) var emissive_tex: texture_2d<f32>;
@group(1) @binding(7) var material_sampler: sampler;

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
  // A wire's: xy the surface's uv there, z 1 to take the surface's colour.
  @location(4) surface: vec3<f32>,
};

fn to_clip(p: vec3<f32>, screen: bool) -> vec4<f32> {
  if (screen) {
    let d = p.xy * lf.viewport.z;
    return vec4<f32>(d.x / lf.viewport.x * 2.0 - 1.0, 1.0 - d.y / lf.viewport.y * 2.0, 0.0, 1.0);
  }
  return lf.view_proj * vec4<f32>(p, 1.0);
}

// The quad's corner `vi`: x 0 at the start, 1 at the end; y across.
fn corner(vi: u32) -> vec2<f32> {
  var corners = array<vec2<f32>, 6>(
    vec2<f32>(0.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0),
    vec2<f32>(0.0, -1.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0));
  return corners[vi];
}

@vertex
fn vs(@builtin(vertex_index) vi: u32, l: LineIn) -> LineOut {
  return expand(vi, l);
}

// A wireframe edge: a line with the surface's uv at each end (xy, zw) and
// whether it takes the surface's colour (`l.params.z` 1; a wire is never
// dashed).
@vertex
fn vs_wire(@builtin(vertex_index) vi: u32, l: LineIn, @location(5) uv: vec4<f32>) -> LineOut {
  var plain = l;
  plain.params.z = 0.0;
  var o = expand(vi, plain);
  o.surface = vec3<f32>(mix(uv.xy, uv.zw, corner(vi).x), l.params.z);
  return o;
}

fn expand(vi: u32, l: LineIn) -> LineOut {
  let c = corner(vi);
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
  o.surface = vec3<f32>(0.0);
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
  return shade(in, in.color);
}

@fragment
fn fs_wire(in: LineOut) -> @location(0) vec4<f32> {
  // Sampled before any branch: the derivatives run along the line.
  let base = textureSample(base_color_tex, material_sampler, material_uv(UV_MAP, in.surface.xy));
  let glow = textureSample(emissive_tex, material_sampler, material_uv(UV_EMISSIVE, in.surface.xy)).rgb;
  var c = in.color;
  if (in.surface.z > 0.5) {
    c = vec4<f32>(
      material.base_color.rgb * base.rgb * c.rgb + material.emissive.rgb * glow,
      material.base_color.a * base.a * c.a);
  }
  return shade(in, c);
}

// The line's coverage, dashes and encoding over its colour `color`.
fn shade(in: LineOut, color: vec4<f32>) -> vec4<f32> {
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
  var c = color;
  c.a *= clamp(in.across.z + 0.5 - d, 0.0, 1.0);
  if (c.a <= 0.0) {
    discard;
  }
  if (lf.viewport.w > 0.5) {
    return vec4<f32>(encode(c.rgb), c.a);
  }
  return c;
}
