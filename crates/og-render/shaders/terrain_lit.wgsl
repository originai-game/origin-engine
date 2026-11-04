// The terrain's lit, prepass and normal-map passes. Prepended by
// frame.wgsl, the light tables (clusters.rs), lighting.wgsl and terrain.wgsl.
//
// Holes (terrain.wgsl terrain_hole) are discarded in every pass.
//
// Shading: the normal from the terrain's normal map (built from the
// heights on the GPU, mipmapped), up to sixteen layers weighted by the
// control map's channels times each layer's slope and altitude rule, the
// four strongest at each pixel blended (all of them with four or fewer),
// optionally sharpened by the layers' heights (their albedo's alpha, each
// layer's own height contrast), each layer sampled by world xz, triplanar,
// or past its projectSlope biplanar, each projection hex-tiled when it
// asks for anti-tiling, with its macro variation, lit like a
// StandardMaterial (cascaded shadows, local lights, image-based light),
// caustics under the frame's water, then fog.

struct TerrainOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world: vec3<f32>,
  @location(1) cur_clip: vec4<f32>,
  @location(2) prev_clip: vec4<f32>,
  // The point in the terrain's space (its maps are laid out there).
  @location(3) local: vec3<f32>,
};

@vertex
fn vs(@builtin(vertex_index) vi: u32) -> TerrainOut {
  let local = terrain_local_vertex(vi);
  let p = terrain_world(local);
  var o: TerrainOut;
  o.clip = frame.view_proj * vec4<f32>(p, 1.0);
  o.world = p;
  o.local = local;
  o.cur_clip = terrain.view_proj * vec4<f32>(p, 1.0);
  o.prev_clip = terrain.prev_view_proj * vec4<f32>(p, 1.0);
  return o;
}

// The terrain's maps at a point of its space: its world normal and the
// control map's channels, from the finest resident tile.
struct TerrainPoint {
  normal: vec3<f32>,
  control: array<vec4<f32>, 4>,
};

fn terrain_point(local: vec3<f32>) -> TerrainPoint {
  let s = (local.xz - terrain_min()) / terrain_cell();
  let m = terrain_maps_at(s, dpdx(s), dpdy(s));
  var o: TerrainPoint;
  o.normal = normalize(terrain.normal * m.normal);
  o.control = m.control;
  return o;
}

fn layer_albedo(i: u32, uv: vec2<f32>, dx: vec2<f32>, dy: vec2<f32>) -> vec4<f32> {
  switch i {
    case 0u: { return textureSampleGrad(albedo0, layer_sampler, uv, dx, dy); }
    case 1u: { return textureSampleGrad(albedo1, layer_sampler, uv, dx, dy); }
    case 2u: { return textureSampleGrad(albedo2, layer_sampler, uv, dx, dy); }
    default: { return textureSampleGrad(albedo3, layer_sampler, uv, dx, dy); }
  }
}

fn layer_dimensions(i: u32) -> vec2<f32> {
  switch i {
    case 0u: { return vec2<f32>(textureDimensions(albedo0)); }
    case 1u: { return vec2<f32>(textureDimensions(albedo1)); }
    case 2u: { return vec2<f32>(textureDimensions(albedo2)); }
    default: { return vec2<f32>(textureDimensions(albedo3)); }
  }
}

fn layer_normal_map(i: u32, uv: vec2<f32>, dx: vec2<f32>, dy: vec2<f32>) -> vec3<f32> {
  var t: vec4<f32>;
  switch i {
    case 0u: { t = textureSampleGrad(normal0, layer_sampler, uv, dx, dy); }
    case 1u: { t = textureSampleGrad(normal1, layer_sampler, uv, dx, dy); }
    case 2u: { t = textureSampleGrad(normal2, layer_sampler, uv, dx, dy); }
    default: { t = textureSampleGrad(normal3, layer_sampler, uv, dx, dy); }
  }
  return t.xyz * 2.0 - 1.0;
}

// The weight of a rule over [lo, hi] faded over `fade` on both sides.
fn band(x: f32, lo: f32, hi: f32, fade: f32) -> f32 {
  let f = max(fade, 1e-4);
  return smoothstep(lo - f, lo, x) * (1.0 - smoothstep(hi, hi + f, x));
}

struct LayerSample {
  albedo: vec4<f32>,   // rgb colour, a: height
  normal: vec3<f32>,   // world normal with the layer's detail
};

// ── Projections ─────────────────────────────────────────────────────────
// A layer is sampled from above (planar: world xz), by three projections
// blended by the normal (`triplanar`), or past its `projectSlope` by the
// two projections facing the surface most (biplanar, after Quílez,
// "Biplanar mapping", 2020: two taps instead of three, and never the
// projection seen edge-on, whose texels smear into streaks on a steep
// face whatever the filtering), the planar and biplanar sampling faded
// into each other over 10° around that slope. Planes: 0 xz (seen along
// y), 1 zy (along x), 2 xy (along z).
struct Projection {
  planes: vec4<u32>,
  w: vec4<f32>,
  // Per entry: 0 the planar normal frame (about the surface normal),
  // 1 whiteout blending about the plane's axis.
  whiteout: vec4<u32>,
  count: u32,
};

// The plane seen along axis `a` (0 x, 1 y, 2 z).
fn axis_plane(a: u32) -> u32 {
  return select(select(2u, 0u, a == 1u), 1u, a == 0u);
}

fn layer_projection(l: TerrainLayerGpu, n: vec3<f32>) -> Projection {
  var o: Projection;
  o.planes = vec4<u32>(0u);
  o.w = vec4<f32>(0.0);
  o.whiteout = vec4<u32>(0u);
  o.count = 0u;
  var biplanar = 0.0;
  if (l.look.z > 0.0) {
    let angle = degrees(acos(clamp(n.y, -1.0, 1.0)));
    biplanar = smoothstep(l.look.z - 5.0, l.look.z + 5.0, angle);
  } else if (l.mapping.y > 0.5) {
    var w = pow(abs(n), vec3<f32>(4.0));
    w = w / max(w.x + w.y + w.z, 1e-5);
    o.planes = vec4<u32>(1u, 0u, 2u, 0u);
    o.w = vec4<f32>(w, 0.0);
    o.whiteout = vec4<u32>(1u, 1u, 1u, 0u);
    o.count = 3u;
    return o;
  }
  if (biplanar < 1.0) {
    o.w[0] = 1.0 - biplanar;
    o.count = 1u;
  }
  if (biplanar > 0.0) {
    let a = abs(n);
    let major = select(select(2u, 1u, a.y >= a.z), 0u, a.x >= a.y && a.x >= a.z);
    let o1 = (major + 1u) % 3u;
    let o2 = (major + 2u) % 3u;
    let median = select(o2, o1, a[o1] >= a[o2]);
    // Quílez's weights: each projection from where it starts to matter
    // (the normal's component past 1/√3), normalised.
    var bw = clamp((vec2<f32>(a[major], a[median]) - 0.5773) / (1.0 - 0.5773), vec2<f32>(0.0), vec2<f32>(1.0));
    bw = bw / max(bw.x + bw.y, 1e-5) * biplanar;
    o.planes[o.count] = axis_plane(major);
    o.w[o.count] = bw.x;
    o.whiteout[o.count] = 1u;
    o.count += 1u;
    if (bw.y > 0.0) {
      o.planes[o.count] = axis_plane(median);
      o.w[o.count] = bw.y;
      o.whiteout[o.count] = 1u;
      o.count += 1u;
    }
  }
  return o;
}

fn plane_of(v: vec3<f32>, plane: u32) -> vec2<f32> {
  if (plane == 1u) { return v.zy; }
  if (plane == 2u) { return v.xy; }
  return v.xz;
}

// ── Anti-tiling ─────────────────────────────────────────────────────────
// Hex-tiling (Mikkelsen, "Practical Real-Time Hex-Tiling", JCGT 11(2),
// 2022; ported from mmikk/hextile-demo's hextiling.h, MIT, licence in
// third_party/hextile/): the plane is cut into a lattice of hexagons, each
// a copy of the texture turned and moved by its own hash, every point
// blending the three hexagons around it by its barycentric weights to the
// seventh power, scaled towards the brighter tap so the seams do not go
// grey. Differences from the demo: the hash is pcg2d on integer cell ids
// (not sin) and the turn comes from it, and the lattice is placed from
// the world's zero by f64 values the CPU computes per frame
// (TerrainLayerGpu.hex, hex_base), so the hexagons stay where they are
// on the ground when the frame's origin moves.

const HEX_SCALE: f32 = 3.4641016; // 2√3: the demo's hexagons per repeat

// Jarzynski & Olano, "Hash Functions for GPU Rendering" (JCGT 2020): pcg2d.
fn pcg2d(v: vec2<u32>) -> vec2<u32> {
  var u = v * 1664525u + 1013904223u;
  u.x += u.y * 1664525u;
  u.y += u.x * 1664525u;
  u = u ^ (u >> vec2<u32>(16u));
  u.x += u.y * 1664525u;
  u.y += u.x * 1664525u;
  u = u ^ (u >> vec2<u32>(16u));
  return u;
}

// Four numbers in [0, 1) from a cell's ids.
fn hex_hash(v: vec2<i32>) -> vec4<f32> {
  let a = pcg2d(bitcast<vec2<u32>>(v));
  let b = pcg2d(a ^ vec2<u32>(0x9e3779b9u, 0x85ebca6bu));
  return vec4<f32>(vec4<u32>(a, b) >> vec4<u32>(8u)) / 16777216.0;
}

// A lattice vertex (skewed ids) as a point of uv.
fn hex_centre(v: vec2<f32>) -> vec2<f32> {
  return vec2<f32>(v.x + 0.5 * v.y, 0.8660254 * v.y) / HEX_SCALE;
}

fn turn2(v: vec2<f32>, c: vec2<f32>) -> vec2<f32> {
  return vec2<f32>(c.x * v.x - c.y * v.y, c.y * v.x + c.x * v.y);
}

// The taps one plane takes of a layer's maps: one (plain) or three
// (hex-tiling), with the uv at a quarter of the frequency (macro
// variation's distance blend).
struct PlaneTaps {
  uv: array<vec2<f32>, 3>,
  dx: array<vec2<f32>, 3>,
  dy: array<vec2<f32>, 3>,
  turn: array<vec2<f32>, 3>,  // each tap's turn: cos, sin
  w: vec3<f32>,               // the barycentric weights (hex), or (1, 0, 0)
  n: u32,
  coarse_uv: vec2<f32>,
  coarse_dx: vec2<f32>,
  coarse_dy: vec2<f32>,
};

struct HexTap {
  uv: vec2<f32>,
  dx: vec2<f32>,
  dy: vec2<f32>,
  turn: vec2<f32>,
};

// One hexagon's tap: `v` its vertex relative to the origin's cell,
// `base` that cell's ids, `u` the point's uv from the origin, `mf` the
// origin's offset from its cell's centre and `c` that centre's uv.
fn hex_tap(v: vec2<f32>, base: vec2<i32>, u: vec2<f32>, mf: vec2<f32>, c: vec2<f32>, strength: f32, gx: vec2<f32>, gy: vec2<f32>) -> HexTap {
  let h = hex_hash(vec2<i32>(v) + base);
  let angle = (h.x * 2.0 - 1.0) * PI * strength;
  let turn = vec2<f32>(cos(angle), sin(angle));
  let back = vec2<f32>(turn.x, -turn.y);
  let cen = hex_centre(v);
  var o: HexTap;
  o.uv = turn2(u + mf - cen, back) + c + cen + h.zw;
  o.dx = turn2(gx, back);
  o.dy = turn2(gy, back);
  o.turn = turn;
  return o;
}

fn plane_taps(l: TerrainLayerGpu, plane: u32, p: vec3<f32>, dpx: vec3<f32>, dpy: vec3<f32>) -> PlaneTaps {
  let k = l.mapping.x;
  let u = plane_of(p, plane) * k;
  let gx = plane_of(dpx, plane) * k;
  let gy = plane_of(dpy, plane) * k;
  var t: PlaneTaps;
  t.coarse_uv = u * 0.25 + plane_of(l.coarse.xyz, plane);
  t.coarse_dx = gx * 0.25;
  t.coarse_dy = gy * 0.25;
  if (l.look.x <= 0.0) {
    t.n = 1u;
    t.uv[0] = u + plane_of(l.offset.xyz, plane);
    t.dx[0] = gx;
    t.dy[0] = gy;
    t.turn[0] = vec2<f32>(1.0, 0.0);
    t.w = vec3<f32>(1.0, 0.0, 0.0);
    return t;
  }
  let hf = l.hex[plane];
  var base = l.hex_base[0].xy;
  if (plane == 1u) { base = l.hex_base[0].zw; }
  if (plane == 2u) { base = l.hex_base[1].xy; }
  // The demo's TriangleGrid, from the origin's cell.
  let st = u * HEX_SCALE;
  let sk = vec2<f32>(st.x - 0.57735027 * st.y, 1.15470054 * st.y) + hf.xy;
  let id = floor(sk);
  let f = sk - id;
  let z = 1.0 - f.x - f.y;
  let s = step(0.0, -z);
  let s2 = 2.0 * s - 1.0;
  t.w = max(vec3<f32>(-z * s2, s - f.y * s2, s - f.x * s2), vec3<f32>(0.0));
  t.n = 3u;
  let mf = hex_centre(hf.xy);
  let strength = clamp(l.look.x, 0.0, 1.0);
  let a = hex_tap(id + vec2<f32>(s, s), base, u, mf, hf.zw, strength, gx, gy);
  let b = hex_tap(id + vec2<f32>(s, 1.0 - s), base, u, mf, hf.zw, strength, gx, gy);
  let c = hex_tap(id + vec2<f32>(1.0 - s, s), base, u, mf, hf.zw, strength, gx, gy);
  t.uv = array<vec2<f32>, 3>(a.uv, b.uv, c.uv);
  t.dx = array<vec2<f32>, 3>(a.dx, b.dx, c.dx);
  t.dy = array<vec2<f32>, 3>(a.dy, b.dy, c.dy);
  t.turn = array<vec2<f32>, 3>(a.turn, b.turn, c.turn);
  return t;
}

// The hexagons' weights from their barycentric ones alone (no albedo to
// weigh by).
fn hex_weights(t: PlaneTaps) -> vec3<f32> {
  if (t.n == 1u) { return vec3<f32>(1.0, 0.0, 0.0); }
  let w = pow(t.w, vec3<f32>(7.0));
  return w / max(w.x + w.y + w.z, 1e-12);
}

struct TapsAlbedo {
  albedo: vec4<f32>,
  w: vec3<f32>,  // the hexagons' final weights, which the normal taps reuse
};

// A plane's albedo (and height in alpha): its taps blended, then the
// quarter-frequency tap mixed in by `coarse`.
fn taps_albedo(i: u32, t: PlaneTaps, coarse: f32) -> TapsAlbedo {
  var o: TapsAlbedo;
  if (t.n == 1u) {
    o.albedo = layer_albedo(i, t.uv[0], t.dx[0], t.dy[0]);
    o.w = vec3<f32>(1.0, 0.0, 0.0);
  } else {
    let c0 = layer_albedo(i, t.uv[0], t.dx[0], t.dy[0]);
    let c1 = layer_albedo(i, t.uv[1], t.dx[1], t.dy[1]);
    let c2 = layer_albedo(i, t.uv[2], t.dx[2], t.dy[2]);
    let luma = vec3<f32>(0.299, 0.587, 0.114);
    let lum = vec3<f32>(dot(c0.rgb, luma), dot(c1.rgb, luma), dot(c2.rgb, luma));
    var w = mix(vec3<f32>(1.0), lum, 0.6) * pow(t.w, vec3<f32>(7.0));
    w = w / max(w.x + w.y + w.z, 1e-12);
    o.albedo = c0 * w.x + c1 * w.y + c2 * w.z;
    o.w = w;
  }
  if (coarse > 0.0) {
    let far = layer_albedo(i, t.coarse_uv, t.coarse_dx, t.coarse_dy);
    o.albedo = mix(o.albedo, far, 0.5 * coarse);
  }
  return o;
}

// A plane's tangent-space normal: its taps' slopes turned back with their
// hexagons and blended by `w`.
fn taps_normal(i: u32, t: PlaneTaps, w: vec3<f32>) -> vec3<f32> {
  if (t.n == 1u) { return layer_normal_map(i, t.uv[0], t.dx[0], t.dy[0]); }
  let m0 = layer_normal_map(i, t.uv[0], t.dx[0], t.dy[0]);
  let m1 = layer_normal_map(i, t.uv[1], t.dx[1], t.dy[1]);
  let m2 = layer_normal_map(i, t.uv[2], t.dx[2], t.dy[2]);
  let d = turn2(m0.xy, t.turn[0]) / max(m0.z, 0.05) * w.x
    + turn2(m1.xy, t.turn[1]) / max(m1.z, 0.05) * w.y
    + turn2(m2.xy, t.turn[2]) / max(m2.z, 0.05) * w.z;
  return normalize(vec3<f32>(d, 1.0));
}

// A plane's tangent-space normal `m` as a world normal about `n`: the
// planar frame (the normal map's x along world x, y along world z), or
// whiteout blending about the plane's axis.
fn plane_normal(whiteout: u32, plane: u32, m: vec3<f32>, n: vec3<f32>, strength: f32) -> vec3<f32> {
  if (whiteout == 0u) {
    let t = normalize(vec3<f32>(1.0, 0.0, 0.0) - n * n.x);
    let b = normalize(vec3<f32>(0.0, 0.0, 1.0) - n * n.z);
    let d = vec3<f32>(m.xy * strength, m.z);
    return t * d.x + b * d.y + n * d.z;
  }
  if (plane == 1u) {
    let t = vec3<f32>(m.xy * strength + n.zy, abs(m.z) * n.x);
    return t.zyx;
  }
  if (plane == 2u) {
    return vec3<f32>(m.xy * strength + n.xy, abs(m.z) * n.z);
  }
  let t = vec3<f32>(m.xy * strength + n.xz, abs(m.z) * n.y);
  return t.xzy;
}

// ── Macro variation ─────────────────────────────────────────────────────
// What keeps a field of one layer from reading as one texture from afar:
// a slow drift of brightness and hue over the ground (value noise over
// world xz, measured in the layer's repeats, so it is the same wherever
// the frame's origin is), and the texture at a quarter of its frequency
// blended in with distance (10 to 40 repeats away).

fn value_noise(x: vec2<f32>) -> f32 {
  let i = floor(x);
  let f = x - i;
  let u = f * f * (3.0 - 2.0 * f);
  let c = vec2<i32>(i);
  let a = hex_hash(c).x;
  let b = hex_hash(c + vec2<i32>(1, 0)).x;
  let d = hex_hash(c + vec2<i32>(0, 1)).x;
  let e = hex_hash(c + vec2<i32>(1, 1)).x;
  return mix(mix(a, b, u.x), mix(d, e, u.x), u.y);
}

fn layer_coarse(l: TerrainLayerGpu, p: vec3<f32>) -> f32 {
  if (l.look.w <= 0.0) { return 0.0; }
  let tile = 1.0 / max(l.mapping.x, 1e-6);
  return smoothstep(10.0 * tile, 40.0 * tile, distance(p, frame.view_pos.xyz)) * clamp(l.look.w, 0.0, 1.0);
}

fn macro_tint(l: TerrainLayerGpu, p: vec3<f32>) -> vec3<f32> {
  if (l.look.w <= 0.0) { return vec3<f32>(1.0); }
  let m = clamp(l.look.w, 0.0, 1.0);
  let at = (p.xz + frame.origin.xz) * l.mapping.x;
  let slow = value_noise(at / 16.0) * 0.65 + value_noise(at / 5.0 + vec2<f32>(17.3, 5.1)) * 0.35;
  let hue = value_noise(at / 40.0 + vec2<f32>(3.1, 41.7));
  let bright = 1.0 + m * 0.6 * (slow - 0.5);
  let warm = mix(vec3<f32>(0.92, 1.0, 1.07), vec3<f32>(1.07, 1.0, 0.88), hue);
  return bright * mix(vec3<f32>(1.0), warm, m);
}

// ── A layer at a point ──────────────────────────────────────────────────

// Its albedo and height alone (the prepass: no normal map is read).
fn sample_layer_albedo(i: u32, p: vec3<f32>, n: vec3<f32>, dpx: vec3<f32>, dpy: vec3<f32>) -> vec4<f32> {
  let l = terrain.layers[i];
  if ((u32(l.mapping.w) & 1u) == 0u) { return vec4<f32>(1.0); }
  let proj = layer_projection(l, n);
  let coarse = layer_coarse(l, p);
  var a = vec4<f32>(0.0);
  for (var k = 0u; k < proj.count; k++) {
    let t = plane_taps(l, proj.planes[k], p, dpx, dpy);
    a += taps_albedo(i, t, coarse).albedo * proj.w[k];
  }
  return a;
}

// Its albedo, height and detailed normal.
fn sample_layer(i: u32, p: vec3<f32>, n: vec3<f32>, dpx: vec3<f32>, dpy: vec3<f32>) -> LayerSample {
  let l = terrain.layers[i];
  let has_albedo = (u32(l.mapping.w) & 1u) != 0u;
  let has_normal = (u32(l.mapping.w) & 2u) != 0u;
  var s: LayerSample;
  s.albedo = vec4<f32>(1.0);
  s.normal = n;
  if (!has_albedo && !has_normal) { return s; }
  let proj = layer_projection(l, n);
  let coarse = layer_coarse(l, p);
  var albedo = vec4<f32>(0.0);
  var normal = vec3<f32>(0.0);
  for (var k = 0u; k < proj.count; k++) {
    let plane = proj.planes[k];
    let t = plane_taps(l, plane, p, dpx, dpy);
    var w = hex_weights(t);
    if (has_albedo) {
      let a = taps_albedo(i, t, coarse);
      albedo += a.albedo * proj.w[k];
      w = a.w;
    }
    if (has_normal) {
      normal += plane_normal(proj.whiteout[k], plane, taps_normal(i, t, w), n, l.mapping.z) * proj.w[k];
    }
  }
  if (has_albedo) { s.albedo = albedo; }
  if (has_normal) { s.normal = normalize(normal); }
  return s;
}

// The four layers a pixel blends: the strongest raw weights (control map
// times the slope and altitude rules), ties to the lower layer, and those
// weights; with four layers or fewer, every layer. Where nothing shows
// (no weight past the height blend's floor, or a sum near 0), the first
// layer alone (`fallback`).
struct LayerPick {
  ids: vec4<u32>,
  w: vec4<f32>,
  n: u32,
  fallback: bool,
};

fn pick_layers(p: vec3<f32>, n: vec3<f32>, c: array<vec4<f32>, 4>) -> LayerPick {
  let count = min(u32(terrain.origin.w), 16u);
  let angle = degrees(acos(clamp(n.y, -1.0, 1.0)));
  let altitude = p.y - terrain.origin.y;
  var pick: LayerPick;
  pick.ids = vec4<u32>(0u, 1u, 2u, 3u);
  pick.w = vec4<f32>(-1.0);
  var top = 0.0;
  var sum = 0.0;
  for (var i = 0u; i < count; i++) {
    let l = terrain.layers[i];
    let w = c[i / 4u][i % 4u] * band(angle, l.slope.x, l.slope.y, l.slope.z) * band(altitude, l.altitude.x, l.altitude.y, l.altitude.z);
    top = max(top, w);
    sum += w;
    // Insert below every weight at least as large (ties keep the lower layer first).
    var at = 4u;
    for (var k = 0u; k < 4u; k++) {
      if (at == 4u && w > pick.w[k]) { at = k; }
    }
    if (at < 4u) {
      for (var k = 3u; k > at; k--) {
        pick.w[k] = pick.w[k - 1u];
        pick.ids[k] = pick.ids[k - 1u];
      }
      pick.w[at] = w;
      pick.ids[at] = i;
    }
  }
  pick.n = min(count, 4u);
  pick.w = max(pick.w, vec4<f32>(0.0));
  let hb = terrain.params.x;
  pick.fallback = select((sum < 1e-4), (top <= 1e-3), (hb > 0.0));
  if (pick.fallback) {
    pick.ids = vec4<u32>(0u, 1u, 2u, 3u);
    pick.w = vec4<f32>(1.0, 0.0, 0.0, 0.0);
    pick.n = min(count, 1u);
  }
  return pick;
}

// The picked layers' weights (summing to 1), sharpened by their heights:
// each layer keeps what its weight plus its height (times the terrain's
// height blend) reaches within its window of the highest, the window
// 0.15 narrowed by the layer's height contrast (to a tenth at 1).
fn blend_weights(pick: LayerPick, heights_of: vec4<f32>) -> vec4<f32> {
  var w = pick.w;
  let hb = terrain.params.x;
  if (hb > 0.0 && !pick.fallback) {
    var ha = vec4<f32>(-1e9);
    var top = -1e9;
    for (var k = 0u; k < 4u; k++) {
      if (k < pick.n && w[k] > 1e-3) {
        ha[k] = w[k] + heights_of[k] * hb;
        top = max(top, ha[k]);
      }
    }
    for (var k = 0u; k < 4u; k++) {
      let contrast = clamp(terrain.layers[pick.ids[k]].look.y, 0.0, 1.0);
      w[k] = max(ha[k] - (top - 0.15 * (1.0 - 0.9 * contrast)), 0.0);
    }
  }
  let sum = w.x + w.y + w.z + w.w;
  if (sum < 1e-4) { return vec4<f32>(1.0, 0.0, 0.0, 0.0); }
  return w / sum;
}

// The blended layers at a fragment: albedo, shading normal, roughness.
struct TerrainLook {
  albedo: vec3<f32>,
  normal: vec3<f32>,
  roughness: f32,
};

// The picked layers' roughness under weights `w` (the lit pass and the
// prepass both: their roughness is one function of the same weights).
fn blended_roughness(pick: LayerPick, w: vec4<f32>) -> f32 {
  if (u32(terrain.origin.w) == 0u) { return 0.9; }
  var roughness = 0.0;
  for (var k = 0u; k < 4u; k++) {
    if (k < pick.n && w[k] > 0.0) {
      roughness += terrain.layers[pick.ids[k]].color.w * w[k];
    }
  }
  return roughness;
}

fn terrain_look(in: TerrainOut, point: TerrainPoint, dpx: vec3<f32>, dpy: vec3<f32>) -> TerrainLook {
  let p = in.world;
  let n_geo = point.normal;
  let count = u32(terrain.origin.w);
  let pick = pick_layers(p, n_geo, point.control);
  // The picked layers' colours and normals (a layer of weight 0 skips its maps).
  var samples: array<LayerSample, 4>;
  var heights_of = vec4<f32>(1.0);
  for (var k = 0u; k < 4u; k++) {
    if (k < pick.n && (pick.w[k] > 0.0 || k == 0u)) {
      samples[k] = sample_layer(pick.ids[k], p, n_geo, dpx, dpy);
      heights_of[k] = samples[k].albedo.a;
    }
  }
  let w = blend_weights(pick, heights_of);
  var albedo = vec3<f32>(0.0);
  var normal = vec3<f32>(0.0);
  for (var k = 0u; k < 4u; k++) {
    if (k < pick.n && w[k] > 0.0) {
      let l = terrain.layers[pick.ids[k]];
      albedo += samples[k].albedo.rgb * l.color.rgb * macro_tint(l, p) * w[k];
      normal += samples[k].normal * w[k];
    }
  }
  if (count == 0u) {
    albedo = vec3<f32>(0.5);
    normal = n_geo;
  }
  return TerrainLook(albedo, normal, blended_roughness(pick, w));
}

// The roughness `terrain_look` gives a fragment, from the layers'
// heights alone: the prepass reads no normal map and blends no colour.
fn terrain_roughness(in: TerrainOut, point: TerrainPoint, dpx: vec3<f32>, dpy: vec3<f32>) -> f32 {
  let p = in.world;
  let n_geo = point.normal;
  let pick = pick_layers(p, n_geo, point.control);
  var heights_of = vec4<f32>(1.0);
  if (terrain.params.x > 0.0 && !pick.fallback) {
    for (var k = 0u; k < 4u; k++) {
      if (k < pick.n && (pick.w[k] > 0.0 || k == 0u)) {
        heights_of[k] = sample_layer_albedo(pick.ids[k], p, n_geo, dpx, dpy).a;
      }
    }
  }
  return blended_roughness(pick, blend_weights(pick, heights_of));
}

// The toon ramp's three bands for the terrain (pbr.wgsl toon_ramp's built-in
// one), each edge smoothed over a pixel of t.
fn terrain_band(t: f32) -> f32 {
  let e = max(fwidth(t), 1e-4);
  return 0.22 + 0.38 * smoothstep(0.08 - e, 0.08 + e, t) + 0.4 * smoothstep(0.45 - e, 0.45 + e, t);
}

// The share of layer `layer` at a point (world `p`, normal `n`, control
// channels `c`) over every layer's weight, as og.Grass places its blades
// (grass_generate.wgsl grass_layer_weight).
fn grass_layer_share(p: vec3<f32>, n: vec3<f32>, c: array<vec4<f32>, 4>, layer: u32) -> f32 {
  let count = min(u32(terrain.origin.w), 16u);
  let angle = degrees(acos(clamp(n.y, -1.0, 1.0)));
  let altitude = p.y - terrain.origin.y;
  var sum = 0.0;
  var mine = 0.0;
  for (var i = 0u; i < count; i++) {
    let l = terrain.layers[i];
    let w = c[i / 4u][i % 4u] * band(angle, l.slope.x, l.slope.y, l.slope.z) * band(altitude, l.altitude.x, l.altitude.y, l.altitude.z);
    sum += w;
    if (i == layer) { mine = w; }
  }
  if (sum < 1e-4) { return select(0.0, 1.0, layer == 0u); }
  return mine / sum;
}

// og.Grass's colour on the ground under and past its blades (grass.rs,
// after bevy_grass's far.wesl): where its layer shows, the ground takes the
// field's average colour from `fade.x` on, whole from `fade.y`, so the
// gaps between blades and the ground past them read as the same field
// and there is no seam where the blades thin out to nothing; per clump
// and dry drift as the blades are, paler where a gust presses it down.
fn grass_far(albedo: vec3<f32>, p: vec3<f32>, local: vec3<f32>, n: vec3<f32>, c: array<vec4<f32>, 4>) -> vec3<f32> {
  let g = terrain.grass;
  if (g.color.w < 0.0) { return albedo; }
  let amount = smoothstep(g.fade.x, g.fade.y, distance(p, frame.view_pos.xyz)) * g.fade.w * grass_layer_share(p, n, c, u32(g.color.w));
  if (amount <= 0.0) { return albedo; }
  let at = p.xz + frame.origin.xz;
  let sheen = grass_gust_sheen(g.wind, at, g.fade.z);
  let clump = grass_clump(local.xz - terrain_min(), g.clump.x, u32(g.clump.w));
  let v = g.clump.y;
  var col = g.color.rgb * mix(1.0 - 0.35 * v, 1.0 + 0.25 * v, clump.x);
  // The dry drifts as the ground's share of dry clumps, blended (a blade
  // is dry or not; the ground under many shows their mix), a little
  // more or less per clump.
  let dry = clamp(grass_dry_share(at, g.clump.z) * mix(0.7, 1.3, clump.y), 0.0, 1.0);
  col = mix(col, g.dry.rgb * 0.8, dry * 0.75);
  return mix(albedo, col * sheen, amount);
}

// Six repeating colours, shared by the layer and mip diagnostic legends.
fn terrain_debug_color(i: u32) -> vec3<f32> {
  let colors = array<vec3<f32>, 6>(
    vec3<f32>(0.1, 1.0, 0.1), vec3<f32>(1.0, 1.0, 0.1),
    vec3<f32>(1.0, 0.2, 0.1), vec3<f32>(0.1, 0.8, 1.0),
    vec3<f32>(0.6, 0.2, 1.0), vec3<f32>(1.0, 0.5, 0.1));
  return colors[i % 6u];
}

fn terrain_debug(in: TerrainOut, point: TerrainPoint) -> vec3<f32> {
  let vertical = dpdy(in.world);
  if (frame.light_color.w > 9.5) {
    let pick = pick_layers(in.world, point.normal, point.control);
    let layer = terrain.layers[pick.ids.x];
    if (frame.light_color.w > 10.5) {
      if ((u32(layer.mapping.w) & 1u) == 0u) { return debug_unsupported(in.clip.xy).rgb; }
      let dy = vertical * layer.mapping.x;
      let size = layer_dimensions(pick.ids.x);
      let density = vec3<f32>(length(dy.zy * size), length(dy.xz * size), length(dy.xy * size));
      // Linear RGB = d/(1+d): black 0, half intensity 1 texel per
      // screen pixel, approaching white as the footprint grows.
      return density / (vec3<f32>(1.0) + density);
    }
    if (layer.mapping.y < 0.5) { return vec3<f32>(0.0, 1.0, 0.0); }
    let weights = pow(abs(point.normal), vec3<f32>(4.0));
    return weights / max(dot(weights, vec3<f32>(1.0)), 1e-5);
  }
  let samples = (in.local.xz - terrain_min()) / terrain_cell();
  let gx = dpdx(samples);
  let gy = dpdy(samples);
  let cell = (in.local.xz - terrain_min()) / (level.camera.z * level.center.z);
  let edge = abs(fract(cell - 0.5) - 0.5) / max(fwidth(cell), vec2<f32>(1e-5));
  if (frame.light_color.w > 8.5) {
    let resident = resident_from(0u, samples);
    let footprint = max(length(gx), length(gy)) / f32(1u << resident.lvl);
    let mip = max(log2(max(footprint, 1.0)), 0.0);
    return terrain_debug_color(u32(mip)) * (1.0 - 0.5 * fract(mip));
  }
  if (frame.light_color.w > 7.5) {
    let pick = pick_layers(in.world, point.normal, point.control);
    if (pick.fallback) { return vec3<f32>(1.0, 0.0, 1.0); }
    let share = pick.w.x / max(dot(pick.w, vec4<f32>(1.0)), 1e-5);
    return terrain_debug_color(pick.ids.x) * (0.25 + 0.75 * share);
  }
  var weights = pow(abs(point.normal), vec3<f32>(4.0));
  weights /= max(dot(weights, vec3<f32>(1.0)), 1e-5);
  let c = (in.local.xz - terrain_min()) / level.camera.z;
  let d = max(abs(c.x - level.camera.x), abs(c.y - level.camera.y)) / level.center.z;
  let seam = select(0.0, clamp((d - level.grid.y) / max(level.grid.z, 1.0), 0.0, 1.0), level.grid.z > 0.0);
  return mix(weights * mix(0.15, 1.0, clamp(min(edge.x, edge.y), 0.0, 1.0)), vec3<f32>(1.0), seam * 0.7);
}

@fragment
fn fs(in: TerrainOut) -> @location(0) vec4<f32> {
  if (terrain_hole_at(in.local)) { discard; }
  let p = in.world;
  let dpx = dpdx(p);
  let dpy = dpdy(p);
  let point = terrain_point(in.local);
  if (frame.light_color.w > 6.5) {
    return vec4<f32>(terrain_debug(in, point), 1.0);
  }
  let n_geo = point.normal;
  let look = terrain_look(in, point, dpx, dpy);
  let albedo = grass_far(look.albedo, p, in.local, n_geo, point.control);
  let normal = look.normal;
  let roughness = look.roughness;
  var n = normalize(normal);
  var s: Surface;
  s.albedo = albedo;
  s.metallic = 0.0;
  s.roughness = clamp(roughness, 0.04, 1.0);
  s.f0 = vec3<f32>(0.04);
  let weathered = weather_surface(s, n, n_geo, p);
  s = weathered.s;
  n = weathered.n;
  if (frame.light_color.w > 1.5 && frame.light_color.w < 2.5) {
    return debug_unsupported(in.clip.xy);
  }
  if (frame.light_color.w > 2.5 && frame.light_color.w < 3.5) {
    return vec4<f32>(s.albedo, 1.0);
  }
  let v = select(normalize(frame.view_pos.xyz - p), -frame.view_forward.xyz, frame.view_pos.w > 0.5);

  let l = normalize(frame.light_dir.xyz);
  let n_dot_l = max(dot(n, l), 0.0);
  var shadow = 1.0;
  if (frame.light_dir.w > 0.5 && n_dot_l > 0.0) { shadow = cascade_shadow(p, n_geo, n_dot_l); }
  var color = brdf(s, n, v, l) * frame.light_color.rgb * shadow;
  // og.MaterialStyle's shadow tint: the sunlight a cast shadow takes moves towards the tint, as on a lit material.
  var tinted = (1.0 - shadow) * n_dot_l;
  if (frame.outline_style.z > 0.5) {
    // og.MaterialStyle's toon: the ground in the built-in ramp's three bands
    // (as a lit material without its own gradientMap), matte (no hard
    // highlight), each band's edge smoothed over a pixel so it does not crawl.
    let band = terrain_band(n_dot_l * shadow);
    color = s.albedo / PI * band * frame.light_color.rgb;
    // The tint takes what the ramp lost to the shadow, once, from the final
    // shadow factor: inside the dark band a deeper shadow (two overlapping,
    // or a soft one deepened) loses nothing more, so it gains no more tint
    // (else overlapping shadows came out lighter).
    tinted = terrain_band(n_dot_l) - band;
  }
  if (frame.shadow_tint.w > 0.0 && n_dot_l > 0.0) {
    var tint = frame.shadow_tint.rgb * frame.shadow_tint.w;
    // Under toon, never more than the diffuse light the ramp took (as pbr_fs.wgsl).
    if (frame.outline_style.z > 0.5) { tint = min(tint, vec3<f32>(1.0 / PI)); }
    color += tinted * tint * s.albedo * frame.light_color.rgb;
  }
  color += s.albedo * frame.light_color.rgb * n_dot_l * shadow * caustics(p);
  // Point, spot and rect lights: this fragment's cluster's, or every light
  // in a reflection probe's face (area.wgsl surface_local_lights).
  color += surface_local_lights(in.clip.xy, s, p, n, v);
  let n_dot_v = max(dot(n, v), 1e-4);
  let f_ibl = f_schlick_roughness(n_dot_v, s.f0, s.roughness);
  let kd_ibl = vec3<f32>(1.0) - f_ibl;
  // The environment, or the probe holding the tile blended with it (probes.rs),
  // at its intensity, plus the hemisphere light at its own.
  let irradiance = ambient_light(n, p) * frame.params.x + hemi_irradiance(n);
  let r = reflect(-v, n);
  let prefiltered = reflected_light(r, p, s.roughness * frame.params.y) * frame.params.x;
  let lut = textureSample(brdf_lut, ibl_sampler, vec2<f32>(n_dot_v, s.roughness)).rg;
  let diffuse_ibl = kd_ibl * irradiance * s.albedo;
  let specular_ibl = prefiltered * (f_ibl * lut.x + lut.y);
  let view = u32(frame.light_color.w);
  if (view == 4u) { return vec4<f32>(diffuse_ibl, 1.0); }
  if (view == 5u) { return vec4<f32>(specular_ibl, 1.0); }
  if (view == 6u) { return vec4<f32>(color, 1.0); }
  color += diffuse_ibl + specular_ibl;

  if (frame.light_color.w > 0.5 && frame.light_color.w < 1.5) {
    return vec4<f32>(cascade_tint(cascade_index(p)) * 4.0, 1.0);
  }
  return vec4<f32>(apply_fog(color, p), 1.0);
}

struct TerrainPrepass {
  @location(0) gbuffer: vec4<f32>,
  @location(1) depth: vec4<f32>,
  @location(2) outline: vec4<f32>,
};

@fragment
fn fs_prepass(in: TerrainOut) -> TerrainPrepass {
  if (terrain_hole_at(in.local)) { discard; }
  let point = terrain_point(in.local);
  let n = point.normal;
  // The layers' roughness as the lit pass shades it, for screen-space
  // reflections (a terrain is never metal): their weights alone.
  let roughness = terrain_roughness(in, point, dpdx(in.world), dpdy(in.world));
  let cur = in.cur_clip.xy / in.cur_clip.w;
  let prev = in.prev_clip.xy / max(in.prev_clip.w, 1e-6);
  var o: TerrainPrepass;
  o.gbuffer = vec4<f32>(oct_encode(n), (cur - prev) * vec2<f32>(0.5, -0.5));
  let reflectivity = reflectivity_bits(clamp(roughness, 0.04, 1.0), 0.0);
  o.depth = vec4<f32>(dot(in.world - frame.view_pos.xyz, frame.view_forward.xyz), reflectivity, 0.0, 1.0);
  o.outline = vec4<f32>(0.0);
  return o;
}
