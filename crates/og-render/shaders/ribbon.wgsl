// Trails, ribbons and beams: camera-facing strips blended into the HDR
// target after the opaque scene (depth tested, no depth write). u runs
// along the strip, v across it. Untextured strips get a soft edge across v.
// Prepended by frame.wgsl.
//
// `vs_strip` turns the strip points (ribbons_gpu.rs; a trail's written by
// trails.wgsl) into two triangles a segment facing the bound camera, so a
// view's camera draws the same points facing its own eye. `vs` draws
// vertices built on the CPU: the 2D water's columns, and every strip where
// the device has no compute (ribbons.rs `Trails::emit`).

@group(1) @binding(0) var ribbon_tex: texture_2d<f32>;
@group(1) @binding(1) var ribbon_sampler: sampler;

struct RibbonIn {
  @location(0) position_v: vec4<f32>,
  @location(1) color: vec4<f32>,
  @location(2) extra: vec4<f32>,  // x: u, y: 1 when textured
};

struct RibbonOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) uv: vec2<f32>,
  @location(1) color: vec4<f32>,
  @location(2) textured: f32,
  @location(3) world: vec3<f32>,
  // oit.wgsl `oit_key`: the strip's segment (its provoking vertex's).
  @location(4) @interpolate(flat) oit_key: vec2<u32>,
};

@vertex
fn vs(r: RibbonIn) -> RibbonOut {
  var o: RibbonOut;
  o.clip = frame.view_proj * vec4<f32>(r.position_v.xyz, 1.0);
  o.uv = vec2<f32>(r.extra.x, r.position_v.w);
  o.color = r.color;
  o.textured = r.extra.y;
  o.world = r.position_v.xyz;
  o.oit_key = oit_key4(r.position_v, r.color.rgb);
  return o;
}

struct StripPoint {
  position: vec3<f32>,
  width: f32,
  color: vec4<f32>,
  u: f32,
  flags: u32,          // 1: has a point before, 2: after, 4: textured
  pad0: u32,
  pad1: u32,
};

@group(2) @binding(0) var<storage, read> strip_points: array<StripPoint>;

fn strip_dir(v: vec3<f32>, fallback: vec3<f32>) -> vec3<f32> {
  let l = length(v);
  if (l > 0.0) {
    return v / l;
  }
  return fallback;
}

// Vertex 6 i + k: corner k of segment i (point i to point i + 1), in the
// order `Trails::emit` lays them: edges a (v 0) and b (v 1) of the points,
// a0 b0 b1, a0 b1 a1. A point without a next draws nothing.
@vertex
fn vs_strip(@builtin(vertex_index) vi: u32) -> RibbonOut {
  var o: RibbonOut;
  let seg = vi / 6u;
  let k = vi % 6u;
  if ((strip_points[seg].flags & 2u) == 0u) {
    o.clip = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    o.uv = vec2<f32>(0.0);
    o.color = vec4<f32>(0.0);
    o.textured = 0.0;
    o.world = vec3<f32>(0.0);
    return o;
  }
  let far = k == 2u || k == 4u || k == 5u;
  let plus = k == 1u || k == 2u || k == 4u;
  let i = select(seg, seg + 1u, far);
  let p = strip_points[i];
  let prev = strip_points[select(i, i - 1u, (p.flags & 1u) != 0u)].position;
  let next = strip_points[select(i, i + 1u, (p.flags & 2u) != 0u)].position;
  let tangent = strip_dir(next - prev, vec3<f32>(1.0, 0.0, 0.0));
  let forward = frame.view_forward.xyz;
  var to_eye = -forward;
  if (frame.view_pos.w < 0.5) {
    to_eye = strip_dir(frame.view_pos.xyz - p.position, -forward);
  }
  let side = strip_dir(cross(tangent, to_eye), vec3<f32>(0.0, 1.0, 0.0)) * (p.width * 0.5);
  let start = strip_points[seg];
  o.oit_key = oit_key4(vec4<f32>(start.position, start.u), vec3<f32>(f32(k / 3u), start.width, 0.0));
  let world = p.position + select(-side, side, plus);
  o.clip = frame.view_proj * vec4<f32>(world, 1.0);
  o.uv = vec2<f32>(p.u, select(0.0, 1.0, plus));
  o.color = p.color;
  o.textured = select(0.0, 1.0, (p.flags & 4u) != 0u);
  o.world = world;
  return o;
}

@fragment
fn fs(in: RibbonOut) -> @location(0) vec4<f32> {
  return ribbon_fragment(in);
}

// An alpha strip in the main camera's order-independent passes (oit.rs).
@fragment
fn fs_oit(in: RibbonOut) -> OitOutput {
  return oit_weighted(ribbon_fragment(in), in.clip.z);
}

@fragment
fn fs_oit_list(in: RibbonOut, @builtin(sample_mask) mask: u32) -> OitOutput {
  let c = ribbon_fragment(in);
  oit_store(in.clip, c, mask, in.oit_key);
  return oit_weighted(c, in.clip.z);
}

@fragment
fn fs_oit_front(in: RibbonOut) -> OitOutput {
  let c = ribbon_fragment(in);
  return oit_hybrid(c, in.clip);
}

@fragment
fn fs_oit_mark(in: RibbonOut) {
  let c = ribbon_fragment(in);
  oit_front_mark(in.clip, c.a);
}

fn ribbon_fragment(in: RibbonOut) -> vec4<f32> {
  let tex = textureSample(ribbon_tex, ribbon_sampler, in.uv);
  var c = in.color;
  if (in.textured > 0.5) {
    c *= tex;
  } else {
    let e = 1.0 - abs(in.uv.y * 2.0 - 1.0);
    c.a *= e * e * (3.0 - 2.0 * e);
  }
  c.a *= fog_keep(in.world);
  if (c.a <= 0.0) { discard; }
  if (frame.light_color.w > 0.5) { return debug_unsupported(in.clip.xy); }
  return c;
}
