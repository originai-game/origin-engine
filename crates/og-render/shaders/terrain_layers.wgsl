// The terrain's layers, shared by its lit and shadow modules (after
// terrain.wgsl): each layer sampled at a point (its projections and
// hex-tiling, terrain_layers' layer_albedo reading its texture or its
// array layer), and which layers a point blends and by how much. Nothing
// here reads the pass's own group 0, so the lit and shadow passes take
// the same layers at the same point (the drawn displacement,
// terrain_vertex.wgsl, is the same in both).

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

// xyz: the tangent-space normal, w: the map's alpha (its cell's id,
// `cell_tone`).
fn layer_normal_map(i: u32, uv: vec2<f32>, dx: vec2<f32>, dy: vec2<f32>) -> vec4<f32> {
  var t: vec4<f32>;
  switch i {
    case 0u: { t = textureSampleGrad(normal0, layer_sampler, uv, dx, dy); }
    case 1u: { t = textureSampleGrad(normal1, layer_sampler, uv, dx, dy); }
    case 2u: { t = textureSampleGrad(normal2, layer_sampler, uv, dx, dy); }
    default: { t = textureSampleGrad(normal3, layer_sampler, uv, dx, dy); }
  }
  return vec4<f32>(t.xyz * 2.0 - 1.0, t.w);
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
  seed: vec3<f32>,            // each tap's own number in [0, 1) (0 without hex-tiling)
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
  seed: f32,
};

// One hexagon's tap: `v` its vertex relative to the origin's cell,
// `base` that cell's ids, `u` the point's uv from the origin, `mf` the
// origin's offset from its cell's centre and `c` that centre's uv.
fn hex_tap(v: vec2<f32>, base: vec2<i32>, u: vec2<f32>, mf: vec2<f32>, c: vec2<f32>, strength: f32, gx: vec2<f32>, gy: vec2<f32>) -> HexTap {
  let h = hex_hash(vec2<i32>(v) + base);
  let angle = (h.x * 2.0 - 1.0) * 3.14159265359 * strength;
  let turn = vec2<f32>(cos(angle), sin(angle));
  let back = vec2<f32>(turn.x, -turn.y);
  let cen = hex_centre(v);
  var o: HexTap;
  o.uv = turn2(u + mf - cen, back) + c + cen + h.zw;
  o.dx = turn2(gx, back);
  o.dy = turn2(gy, back);
  o.turn = turn;
  o.seed = h.y;
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
    t.seed = vec3<f32>(0.0);
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
  // The three hexagons in one loop: one copy of the tap's code (DXC
  // inlines every call, so three calls compiled three copies).
  let cells = array<vec2<f32>, 3>(vec2<f32>(s, s), vec2<f32>(s, 1.0 - s), vec2<f32>(1.0 - s, s));
  for (var j = 0u; j < 3u; j++) {
    let h = hex_tap(id + cells[j], base, u, mf, hf.zw, strength, gx, gy);
    t.uv[j] = h.uv;
    t.dx[j] = h.dx;
    t.dy[j] = h.dy;
    t.turn[j] = h.turn;
    t.seed[j] = h.seed;
  }
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

// Layer `i`'s height (its albedo's alpha) at world point `p` of normal
// `n`, over the footprint `dpx`, `dpy` (its projections and hexagons as
// the lit pass samples them, without the distant quarter-frequency
// blend, which depends on the eye); -1 without an albedo map.
fn layer_height(i: u32, p: vec3<f32>, n: vec3<f32>, dpx: vec3<f32>, dpy: vec3<f32>) -> f32 {
  let l = terrain.layers[i];
  if ((u32(l.mapping.w) & 1u) == 0u) { return -1.0; }
  let proj = layer_projection(l, n);
  var a = 0.0;
  for (var k = 0u; k < proj.count; k++) {
    let t = plane_taps(l, proj.planes[k], p, dpx, dpy);
    a += taps_albedo(i, t, 0.0).albedo.a * proj.w[k];
  }
  return a;
}

// A cell's tone (`TerrainLayer::cell_variation` `s`): its id `c` (the
// normal map's alpha) turned into a brightness and a warmth, each a
// whole number of periods over 0 to 1, so a map without ids (opaque, 1)
// or a cell given 0 keeps its colour, and a texel filtered between two
// cells takes a tone between theirs (an id hashed into a tone flickered
// along every joint the filter blended). Per-cell variation, as
// stochastic tile shaders give each tile of a sampled layout its own
// tone (Substance's tile sampler and Unreal's per-instance random, keyed
// by the tile's id texture): a field of setts, flags or tiles laid in
// one texture reads as stones of different quarries.
fn cell_tone(c: f32, s: f32) -> vec3<f32> {
  let a = 6.2831853 * c;
  let bright = 1.0 + s * 0.25 * sin(3.0 * a);
  let warm = s * 0.2 * sin(5.0 * a);
  return bright * vec3<f32>(1.0 + warm, 1.0 + 0.3 * warm, 1.0 - warm);
}

struct TapsNormal {
  normal: vec3<f32>,  // tangent space
  tone: vec3<f32>,    // the taps' cells' tones (`cell_tone`), blended
};

// A plane's tangent-space normal: its taps' slopes turned back with their
// hexagons and blended by `w`, and its cells' tones, each hexagon's id
// moved by its own number (so the copies of one cell differ).
fn taps_normal(i: u32, t: PlaneTaps, w: vec3<f32>) -> TapsNormal {
  let s = clamp(terrain.layers[i].coarse.w, 0.0, 1.0);
  var o: TapsNormal;
  if (t.n == 1u) {
    let m = layer_normal_map(i, t.uv[0], t.dx[0], t.dy[0]);
    o.normal = m.xyz;
    o.tone = cell_tone(m.w, s);
    return o;
  }
  let m0 = layer_normal_map(i, t.uv[0], t.dx[0], t.dy[0]);
  let m1 = layer_normal_map(i, t.uv[1], t.dx[1], t.dy[1]);
  let m2 = layer_normal_map(i, t.uv[2], t.dx[2], t.dy[2]);
  let d = turn2(m0.xy, t.turn[0]) / max(m0.z, 0.05) * w.x
    + turn2(m1.xy, t.turn[1]) / max(m1.z, 0.05) * w.y
    + turn2(m2.xy, t.turn[2]) / max(m2.z, 0.05) * w.z;
  o.normal = normalize(vec3<f32>(d, 1.0));
  o.tone = cell_tone(m0.w + t.seed.x, select(s, 0.0, m0.w == 1.0)) * w.x
    + cell_tone(m1.w + t.seed.y, select(s, 0.0, m1.w == 1.0)) * w.y
    + cell_tone(m2.w + t.seed.z, select(s, 0.0, m2.w == 1.0)) * w.z;
  return o;
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

