// Area lights and light cookies (area.rs), after lighting.wgsl in the lit
// shaders (pbr.wgsl's local_light, physical.wgsl's physical_local).
//
// A rect light rides in the light tables as a LocalLight with color_kind.w
// -1 (one-sided) or -2 (two-sided): position_range its centre and range,
// direction_slot.xyz and cone.xyz its half-width and half-height vectors
// (no shadow slot). It is shaded with linearly transformed cosines (Heitz,
// Dupuy, Hill and Neubelt 2016; the fitted GGX tables of their ltc_code,
// resampled to 32 x 32 and held in a 16 KiB uniform so WebGL2's lit
// pipelines keep their texture slots: frame binding 11).
//
// Cookies are packed 8:8:8 in exact integers in layers of the local shadow
// array past the shadow layers (read with textureLoad, as the shadows): a
// spot's layer in color_kind.w - 2 (w >= 2), a point's six faces from
// -cone.y - 1 (cone.y < 0).

@group(0) @binding(11) var<uniform> ltc_table: array<vec4<u32>, 1024>;

struct LtcSample {
  m: vec4<f32>,       // the inverse transform's a, b, c, d (three.js ltc_1)
  fresnel: vec2<f32>, // the GGX norm and fresnel terms (ltc_2)
};

// An IEEE half (no infinities in the table) decoded by hand:
// unpack2x16float needs a capability some backends lack.
fn ltc_half(h: u32) -> f32 {
  let e = (h >> 10u) & 31u;
  let m = f32(h & 1023u);
  let v = select((1.0 + m / 1024.0) * exp2(f32(e) - 15.0), m * exp2(-24.0), e == 0u);
  return select(v, -v, (h & 0x8000u) != 0u);
}

fn ltc_pair(w: u32) -> vec2<f32> {
  return vec2<f32>(ltc_half(w & 0xffffu), ltc_half(w >> 16u));
}

fn ltc_texel(x: u32, y: u32) -> LtcSample {
  let t = ltc_table[y * 32u + x];
  var s: LtcSample;
  s.m = vec4<f32>(ltc_pair(t.x), ltc_pair(t.y));
  s.fresnel = ltc_pair(t.z);
  return s;
}

// The tables at (roughness, sqrt(1 - n·v)), bilinear.
fn ltc_lookup(uv: vec2<f32>) -> LtcSample {
  let g = clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0)) * 31.0;
  let i = min(vec2<u32>(floor(g)), vec2<u32>(30u));
  let f = g - vec2<f32>(i);
  let a = ltc_texel(i.x, i.y);
  let b = ltc_texel(i.x + 1u, i.y);
  let c = ltc_texel(i.x, i.y + 1u);
  let d = ltc_texel(i.x + 1u, i.y + 1u);
  var s: LtcSample;
  s.m = mix(mix(a.m, b.m, f.x), mix(c.m, d.m, f.x), f.y);
  s.fresnel = mix(mix(a.fresnel, b.fresnel, f.x), mix(c.fresnel, d.fresnel, f.x), f.y);
  return s;
}

fn ltc_edge(v1: vec3<f32>, v2: vec3<f32>) -> vec3<f32> {
  let x = dot(v1, v2);
  let y = abs(x);
  let a = 0.8543985 + (0.4965155 + 0.0145206 * y) * y;
  let b = 3.4175940 + (4.1616724 + y) * y;
  let v = a / b;
  let theta = select(0.5 * inverseSqrt(max(1.0 - x * x, 1e-7)) - v, v, x > 0.0);
  return cross(v1, v2) * theta;
}

// The rectangle's form factor through the cosine distribution m_inv
// transforms (identity: the diffuse lobe), seen from p with normal n.
fn ltc_evaluate(n: vec3<f32>, v: vec3<f32>, p: vec3<f32>, m_inv: mat3x3<f32>, rect: array<vec3<f32>, 4>) -> f32 {
  var t1 = v - n * dot(v, n);
  if (dot(t1, t1) < 1e-10) {
    t1 = select(vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(0.0, 1.0, 0.0), abs(n.x) > 0.9);
    t1 = t1 - n * dot(t1, n);
  }
  t1 = normalize(t1);
  let t2 = -cross(n, t1);
  let m = m_inv * transpose(mat3x3<f32>(t1, t2, n));
  var c = rect;
  for (var i = 0u; i < 4u; i++) {
    c[i] = normalize(m * (c[i] - p));
  }
  var f = ltc_edge(c[0], c[1]);
  f += ltc_edge(c[1], c[2]);
  f += ltc_edge(c[2], c[3]);
  f += ltc_edge(c[3], c[0]);
  let l = length(f);
  return max((l * l + f.z) / (l + 1.0), 0.0);
}

// A rect light's diffuse and specular radiance towards v (three.js's
// RE_Direct_RectArea_Physical), faded to zero at its range.
struct AreaShade {
  diffuse: vec3<f32>,
  specular: vec3<f32>,
};

fn area_shade(light: LocalLight, s: Surface, p: vec3<f32>, n: vec3<f32>, v: vec3<f32>) -> AreaShade {
  var o: AreaShade;
  o.diffuse = vec3<f32>(0.0);
  o.specular = vec3<f32>(0.0);
  let center = light.position_range.xyz;
  var hw = light.direction_slot.xyz;
  let hh = light.cone.xyz;
  let to_p = p - center;
  let d2 = max(dot(to_p, to_p), 1e-4);
  let range = light.position_range.w;
  let ratio = d2 / (range * range);
  let window = clamp(1.0 - ratio * ratio, 0.0, 1.0);
  if (window <= 0.0) { return o; }
  // Emits along -(hw × hh); the back face too when two-sided (reversed
  // winding).
  if (dot(to_p, cross(hw, hh)) >= 0.0) {
    if (light.color_kind.w > -1.5) { return o; }
    hw = -hw;
  }
  let rect = array<vec3<f32>, 4>(center + hw - hh, center - hw - hh, center - hw + hh, center + hw + hh);
  let n_dot_v = clamp(dot(n, v), 1e-4, 1.0);
  let t = ltc_lookup(vec2<f32>(s.roughness, sqrt(1.0 - n_dot_v)));
  let m_inv = mat3x3<f32>(vec3<f32>(t.m.x, 0.0, t.m.y), vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(t.m.z, 0.0, t.m.w));
  let fresnel = s.f0 * t.fresnel.x + (vec3<f32>(1.0) - s.f0) * t.fresnel.y;
  let identity = mat3x3<f32>(vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(0.0, 0.0, 1.0));
  let radiance = light.color_kind.rgb * window * window;
  o.specular = radiance * fresnel * ltc_evaluate(n, v, p, m_inv, rect);
  o.diffuse = radiance * s.albedo * (1.0 - s.metallic) * ltc_evaluate(n, v, p, identity, rect);
  return o;
}

fn area_light(light: LocalLight, s: Surface, p: vec3<f32>, n: vec3<f32>, v: vec3<f32>) -> vec3<f32> {
  let o = area_shade(light, s, p, n, v);
  return (o.diffuse + o.specular) * rect_shadow(light, p, n);
}

// tan(RECT_SHADOW_FOV / 2): the half-width of a rect shadow view at a
// distance of one metre (area.rs; 150°).
const RECT_SHADOW_TAN: f32 = 3.7320508;

// The tap at i of n on a unit disc (a golden-angle spiral turned by
// `turn` radians).
fn rect_shadow_tap(i: u32, n: u32, turn: f32) -> vec2<f32> {
  let r = sqrt((f32(i) + 0.5) / f32(n));
  let a = f32(i) * 2.3999632 + turn;
  return vec2<f32>(cos(a), sin(a)) * r;
}

// A turn in [0, 2π) hashed from a world position's bits (PCG): each
// pixel's spiral turned its own way, so the few taps over a wide penumbra
// read as fine grain instead of copies of the caster.
fn rect_shadow_turn(p: vec3<f32>) -> f32 {
  var h = bitcast<u32>(p.x) ^ (bitcast<u32>(p.y) * 0x9e3779b9u) ^ (bitcast<u32>(p.z) * 0x85ebca6bu);
  h = h * 747796405u + 2891336453u;
  h = ((h >> ((h >> 28u) + 4u)) ^ h) * 277803737u;
  h = (h >> 22u) ^ h;
  return f32(h >> 8u) * (6.2831853 / 16777216.0);
}

// How much of a shadowed rect light reaches p (normal n): 1 without a
// shadow slot (direction_slot.w < 0). Its layer (the back's next when
// two-sided) is a perspective view from the centre storing distance /
// range; read as PCSS: 16 taps find the mean blocker distance within the
// light's footprint, then 24 bilinear taps filter over the penumbra the
// rectangle's size casts, size × (receiver - blocker) / blocker (taps
// over a disc whose radius is half of it).
fn rect_shadow(light: LocalLight, p: vec3<f32>, n: vec3<f32>) -> f32 {
  let slot = i32(light.direction_slot.w);
  if (slot < 0) { return 1.0; }
  let center = light.position_range.xyz;
  let range = light.position_range.w;
  let hw = light.direction_slot.xyz;
  let hh = light.cone.xyz;
  let to = p - center;
  var layer = u32(slot);
  if (dot(to, cross(hw, hh)) > 0.0) {
    // Behind its face: lit only from the back of a two-sided panel.
    if (light.color_kind.w > -1.5) { return 1.0; }
    layer += 1u;
  }
  let dist = length(to);
  // Normal offset: about two texels of the view at this distance.
  let texels = f32(textureDimensions(local_shadow_map).x);
  let offset = p + n * (4.0 * RECT_SHADOW_TAN / texels) * dist;
  let clip = frame.local_view_proj[layer] * vec4<f32>(offset, 1.0);
  if (clip.w <= 0.0) { return 1.0; }
  let ndc = clip.xy / clip.w;
  let uv = vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
  if (any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0))) { return 1.0; }
  let receiver = length(offset - center);
  let ref_depth = receiver / range - 0.002;
  // The rectangle's half-width (the taps' disc is a radius), in the
  // view's uv at the receiver.
  let size = max(length(hw), length(hh));
  let per_metre = 1.0 / (2.0 * receiver * RECT_SHADOW_TAN);
  let texel = 1.0 / texels;
  let search = clamp(size * per_metre, 2.0 * texel, 48.0 * texel);
  let size_i = vec2<i32>(textureDimensions(local_shadow_map));
  let turn = rect_shadow_turn(p);
  var blockers = 0.0;
  var count = 0.0;
  for (var i = 0u; i < 16u; i++) {
    let q = clamp(uv + rect_shadow_tap(i, 16u, turn) * search, vec2<f32>(0.0), vec2<f32>(1.0));
    let at = clamp(vec2<i32>(q * vec2<f32>(size_i)), vec2<i32>(0), size_i - 1);
    let d = textureLoad(local_shadow_map, at, i32(layer), 0).r;
    if (d < ref_depth) {
      blockers += d;
      count += 1.0;
    }
  }
  if (count < 0.5) { return 1.0; }
  let blocker = max(blockers / count * range, 1e-3);
  let penumbra = clamp(size * (receiver - blocker) / blocker * per_metre, texel, 48.0 * texel);
  var lit = 0.0;
  for (var i = 0u; i < 24u; i++) {
    let q = clamp(uv + rect_shadow_tap(i, 24u, turn) * penumbra, vec2<f32>(0.0), vec2<f32>(1.0));
    lit += pcf(local_shadow_map, q, layer, ref_depth);
  }
  return lit / 24.0;
}

fn cookie_texel(layer: u32, at: vec2<i32>) -> vec3<f32> {
  let u = u32(textureLoad(local_shadow_map, at, i32(layer), 0).r);
  return vec3<f32>(f32((u >> 16u) & 255u), f32((u >> 8u) & 255u), f32(u & 255u)) / 255.0;
}

fn cookie_sample(layer: u32, uv: vec2<f32>) -> vec3<f32> {
  let size = vec2<i32>(textureDimensions(local_shadow_map));
  let q = clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0)) * vec2<f32>(size) - 0.5;
  let base = vec2<i32>(floor(q));
  let f = q - floor(q);
  let a = cookie_texel(layer, clamp(base, vec2<i32>(0), size - 1));
  let b = cookie_texel(layer, clamp(base + vec2<i32>(1, 0), vec2<i32>(0), size - 1));
  let c = cookie_texel(layer, clamp(base + vec2<i32>(0, 1), vec2<i32>(0), size - 1));
  let d = cookie_texel(layer, clamp(base + vec2<i32>(1, 1), vec2<i32>(0), size - 1));
  return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}

// What a light's cookie lets through towards d (from the light to the
// surface): white for a light without one.
fn light_cookie(light: LocalLight, d: vec3<f32>) -> vec3<f32> {
  let kind = light.color_kind.w;
  if (kind > 1.5) {
    let layer = u32(kind - 2.0 + 0.5);
    let axis = light.direction_slot.xyz;
    let up0 = select(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(0.0, 0.0, 1.0), abs(axis.y) > 0.99);
    let right = normalize(cross(axis, up0));
    let up = cross(right, axis);
    let z = dot(d, axis);
    if (z <= 1e-4) { return vec3<f32>(0.0); }
    let c = light.cone.x;
    let tan_outer = sqrt(max(1.0 - c * c, 0.0)) / max(c, 1e-4);
    let xy = vec2<f32>(dot(d, right), dot(d, up)) / (z * tan_outer);
    return cookie_sample(layer, vec2<f32>(xy.x * 0.5 + 0.5, 0.5 - xy.y * 0.5));
  }
  if (kind > -0.5 && kind < 0.5 && light.cone.y < 0.0) {
    let base = u32(-light.cone.y - 1.0 + 0.5);
    let face = cube_face(d);
    let a = abs(d);
    var sc = 0.0;
    var tc = 0.0;
    var ma = 1.0;
    switch face {
      case 0u: { sc = -d.z; tc = -d.y; ma = a.x; }
      case 1u: { sc = d.z; tc = -d.y; ma = a.x; }
      case 2u: { sc = d.x; tc = d.z; ma = a.y; }
      case 3u: { sc = d.x; tc = -d.z; ma = a.y; }
      case 4u: { sc = d.x; tc = -d.y; ma = a.z; }
      default: { sc = -d.x; tc = -d.y; ma = a.z; }
    }
    let uv = vec2<f32>(sc, tc) / max(ma, 1e-6) * 0.5 + 0.5;
    return cookie_sample(base + face, uv);
  }
  return vec3<f32>(1.0);
}

// A point, spot or rect light's light on a surface for the lit shaders
// that draw no material effects (terrain, scattered copies, the 3D fluid):
// a rect light through linearly transformed cosines (area_light), a point
// or spot by its falloff, cone, shadow and cookie, as pbr.wgsl's
// local_light without the toon and receive switches.
fn surface_local_light(light: LocalLight, s: Surface, p: vec3<f32>, n: vec3<f32>, v: vec3<f32>) -> vec3<f32> {
  if (light.color_kind.w < -0.5) { return area_light(light, s, p, n, v); }
  let to_light = light.position_range.xyz - p;
  let d2 = max(dot(to_light, to_light), 1e-4);
  let range = light.position_range.w;
  let ratio = d2 / (range * range);
  let window = clamp(1.0 - ratio * ratio, 0.0, 1.0);
  var atten = window * window / d2;
  let l = to_light * inverseSqrt(d2);
  if (light.color_kind.w > 0.5) {
    let cos_angle = dot(-l, light.direction_slot.xyz);
    let t = clamp((cos_angle - light.cone.x) * light.cone.y, 0.0, 1.0);
    atten *= t * t;
  }
  if (atten <= 0.0 || dot(n, l) <= 0.0) { return vec3<f32>(0.0); }
  let radiance = light.color_kind.rgb * light_cookie(light, -l);
  return brdf(s, n, v, l) * radiance * atten * local_shadow(light, p, n);
}

// Every point, spot and rect light on a surface for those shaders: the
// lights of the fragment's cluster (`frag`, its pixel; clusters.rs), in
// scene order, or, in a reflection probe's face, whose frame has no
// clusters (probes.rs), every light of the frame, as pbr_fs.wgsl's
// every-light branch. A face read the main camera's first cluster before.
fn surface_local_lights(frag: vec2<f32>, s: Surface, p: vec3<f32>, n: vec3<f32>, v: vec3<f32>) -> vec3<f32> {
  var color = vec3<f32>(0.0);
  if (frame.params.z < 0.5) { return color; }
  if (frame.cluster.x > 0.0) {
    let word = cluster_word(cluster_at(frag, p));
    let first = word & 0xffffffu;
    let count = word >> 24u;
    for (var k = 0u; k < count; k++) {
      color += surface_local_light(light_list[cluster_ref(first + k)], s, p, n, v);
    }
  } else {
    for (var k = 0u; k < u32(frame.params.z); k++) {
      color += surface_local_light(light_list[k], s, p, n, v);
    }
  }
  return color;
}
