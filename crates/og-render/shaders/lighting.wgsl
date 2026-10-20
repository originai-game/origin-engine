// Lighting shared by the lit passes (pbr.wgsl, terrain.wgsl, water.wgsl):
// the GGX BRDF, the directional light's cascaded shadows and the local
// lights' shadows, read with explicit compares (the WebGL2-safe path).
// Prepended after frame.wgsl.

const PI: f32 = 3.14159265359;

fn d_ggx(n_dot_h: f32, a: f32) -> f32 {
  let a2 = a * a;
  let d = n_dot_h * n_dot_h * (a2 - 1.0) + 1.0;
  return a2 / (PI * d * d + 1e-7);
}

fn g_smith(n_dot_v: f32, n_dot_l: f32, roughness: f32) -> f32 {
  let r = roughness + 1.0;
  let k = (r * r) / 8.0;
  let gv = n_dot_v / (n_dot_v * (1.0 - k) + k);
  let gl = n_dot_l / (n_dot_l * (1.0 - k) + k);
  return gv * gl;
}

fn f_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
  return f0 + (1.0 - f0) * pow(1.0 - cos_theta, 5.0);
}

fn f_schlick_roughness(cos_theta: f32, f0: vec3<f32>, roughness: f32) -> vec3<f32> {
  return f0 + (max(vec3<f32>(1.0 - roughness), f0) - f0) * pow(1.0 - cos_theta, 5.0);
}

// Bilinear PCF over one layer of an R32Float array: the four texels around
// the sample compared explicitly (ref <= stored means lit) and blended by the
// sample's position between them, as a hardware comparison sampler with
// linear filtering would, so edges ramp over one texel instead of stepping.
// textureLoad and explicit compares give the same result on every backend
// (WebGL2 can neither filter a depth texture nor gather).
fn pcf(map: texture_2d_array<f32>, uv: vec2<f32>, layer: u32, ref_depth: f32) -> f32 {
  let size = vec2<i32>(textureDimensions(map));
  let p = uv * vec2<f32>(size) - 0.5;
  let base = vec2<i32>(floor(p));
  let f = fract(p);
  var lit: array<f32, 4>;
  for (var i = 0; i < 4; i++) {
    let at = clamp(base + vec2<i32>(i & 1, i >> 1u), vec2<i32>(0), size - 1);
    lit[i] = select(0.0, 1.0, ref_depth <= textureLoad(map, at, i32(layer), 0).r);
  }
  return mix(mix(lit[0], lit[1], f.x), mix(lit[2], lit[3], f.x), f.y);
}

// The cascade whose camera slice holds the point (by view depth, the same
// slices its casters were culled to); CASCADES beyond the last one.
fn cascade_index(p: vec3<f32>) -> u32 {
  let count = u32(frame.light_dir.w);
  let depth = dot(p - frame.view_pos.xyz, frame.view_forward.xyz);
  for (var c = 0u; c < count; c++) {
    if (depth <= frame.cascade_bias[c].z) { return c; }
  }
  return CASCADES;
}

// The sun's soft shadows (PCSS, `DirectionalLight::shadow_softness`): a
// blocker search over the widest penumbra the cascade allows, then 16
// bilinear PCF taps over the penumbra the blockers' mean depth gives
// (softness × the distance from blocker to receiver, in texels). A
// Vogel disc spreads the taps; a slope term keeps the wide kernel off a
// receiver tilted from the light. Every read is a textureLoad compare, as
// pcf's, so WebGL2 draws it too.
const PCSS_TAPS: i32 = 16;
const PCSS_MAX_TEXELS: f32 = 24.0;

fn vogel(i: i32, radius: f32) -> vec2<f32> {
  let r = sqrt((f32(i) + 0.5) / f32(PCSS_TAPS)) * radius;
  let a = f32(i) * 2.39996323;
  return vec2<f32>(cos(a), sin(a)) * r;
}

fn pcss(uv: vec2<f32>, c: u32, ref_depth: f32, bias: vec4<f32>, tan_l: f32) -> f32 {
  let size = vec2<f32>(textureDimensions(cascade_map));
  let isize = vec2<i32>(size);
  // World units per texel, and the cascade's depth range in world units.
  let texel = bias.x;
  let depth_range = texel / max(bias.y, 1e-12);
  let softness = frame.shadow.x;
  // How the receiver's light-space depth changes per texel along it.
  let slope = bias.y * min(tan_l, 4.0);
  let search = clamp(softness * ref_depth * depth_range / texel, 1.0, PCSS_MAX_TEXELS);
  var blockers = 0.0;
  var sum = 0.0;
  for (var i = 0; i < PCSS_TAPS; i++) {
    let o = vogel(i, search);
    let at = clamp(vec2<i32>(floor(uv * size + o)), vec2<i32>(0), isize - 1);
    let d = textureLoad(cascade_map, at, i32(c), 0).r;
    if (d < ref_depth - slope * length(o)) {
      blockers += 1.0;
      sum += d;
    }
  }
  if (blockers < 0.5) { return 1.0; }
  let blocker = sum / blockers;
  let penumbra = clamp(softness * (ref_depth - blocker) * depth_range / texel, 1.0, PCSS_MAX_TEXELS);
  var lit = 0.0;
  for (var i = 0; i < PCSS_TAPS; i++) {
    let o = vogel(i, penumbra);
    lit += pcf(cascade_map, uv + o / size, c, ref_depth - slope * length(o));
  }
  return lit / f32(PCSS_TAPS);
}

// One cascade's shadow at p (-1: beyond its depth, lit). The bias is in
// world units scaled by the cascade's texel size (a normal offset growing
// with the light's angle to the surface, plus a depth bias converted to the
// cascade's normalized depth), so it holds at any scene scale.
fn cascade_sample(c: u32, p: vec3<f32>, n: vec3<f32>, n_dot_l: f32) -> f32 {
  let bias = frame.cascade_bias[c];
  let sin_l = sqrt(max(1.0 - n_dot_l * n_dot_l, 0.0));
  let offset = p + n * bias.x * (0.25 + sin_l);
  let sp = frame.cascade_view_proj[c] * vec4<f32>(offset, 1.0);
  let uv = vec2<f32>(sp.x * 0.5 + 0.5, 0.5 - sp.y * 0.5);
  if (sp.z > 1.0) { return -1.0; }
  if (frame.shadow.x > 0.0) {
    return pcss(uv, c, sp.z - bias.y, bias, sin_l / max(n_dot_l, 0.05));
  }
  return pcf(cascade_map, uv, c, sp.z - bias.y);
}

// The cascades' shadow at p: across a cascade's blend band (cascade_bias.w
// to its far split, `DirectionalLight::cascade_blend`) it fades into the
// next one, which covers that band too. Shadows fade out over the last
// tenth of the shadow distance.
fn cascade_shadow(p: vec3<f32>, n: vec3<f32>, n_dot_l: f32) -> f32 {
  let c = cascade_index(p);
  if (c >= CASCADES) { return 1.0; }
  var shadow = cascade_sample(c, p, n, n_dot_l);
  if (shadow < 0.0) { return 1.0; }
  let count = u32(frame.light_dir.w);
  let depth = dot(p - frame.view_pos.xyz, frame.view_forward.xyz);
  let band = frame.cascade_bias[c].w;
  if (c + 1u < count && depth > band) {
    var next = cascade_sample(c + 1u, p, n, n_dot_l);
    if (next < 0.0) { next = 1.0; }
    shadow = mix(shadow, next, smoothstep(band, frame.cascade_bias[c].z, depth));
  }
  let last = frame.cascade_bias[count - 1u].z;
  return mix(shadow, 1.0, smoothstep(0.9 * last, last, depth));
}

// A point light's cube face (+x, -x, +y, -y, +z, -z) for a direction from it.
fn cube_face(d: vec3<f32>) -> u32 {
  let a = abs(d);
  if (a.x >= a.y && a.x >= a.z) { return select(1u, 0u, d.x > 0.0); }
  if (a.y >= a.z) { return select(3u, 2u, d.y > 0.0); }
  return select(5u, 4u, d.z > 0.0);
}

// Dual-paraboloid point shadows (shadows.rs): hemisphere 0 holds what lies
// below the light, 1 what lies above, each the paraboloid projection of
// its half of the sphere scaled into 0.9 of the disc (shadow_pass.wgsl
// draws them).
const PARABOLOID_SCALE: f32 = 0.9;

// Local shadows store distance / range; the fragment compares its own.
fn local_shadow(light: LocalLight, p: vec3<f32>, n: vec3<f32>) -> f32 {
  let slot = i32(light.direction_slot.w);
  if (slot < 0) { return 1.0; }
  let range = light.position_range.w;
  let to_frag = p - light.position_range.xyz;
  let dist = length(to_frag);
  // Normal offset grows with distance (texels widen with it).
  let offset = p + n * light.cone.z * dist;
  var layer = u32(slot);
  if (light.color_kind.w < 0.5 && frame.shadow.y > 0.5) {
    let to = offset - light.position_range.xyz;
    let below = to.y <= 0.0;
    let sigma = select(-1.0, 1.0, below);
    layer += select(1u, 0u, below);
    let l = vec3<f32>(to.x, sigma * to.z, -sigma * to.y);
    let d = max(length(l), 1e-6);
    let q = l / d;
    let ndc = q.xy * (PARABOLOID_SCALE / (1.0 + q.z));
    let uv = vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
    return pcf(local_shadow_map, uv, layer, d / range - light.cone.w);
  }
  if (light.color_kind.w < 0.5) { layer += cube_face(to_frag); }
  let clip = frame.local_view_proj[layer] * vec4<f32>(offset, 1.0);
  if (clip.w <= 0.0) { return 1.0; }
  let ndc = clip.xy / clip.w;
  let uv = vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
  if (light.color_kind.w > 0.5 && (any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0)))) { return 1.0; }
  let ref_depth = length(offset - light.position_range.xyz) / range - light.cone.w;
  return pcf(local_shadow_map, clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0)), layer, ref_depth);
}

struct Surface {
  albedo: vec3<f32>,
  metallic: f32,
  roughness: f32,
  f0: vec3<f32>,
  // Clearcoat: a clear specular layer over the base (0: none), its
  // roughness and its normal (the surface's before the normal map).
  coat: f32,
  coat_roughness: f32,
  coat_normal: vec3<f32>,
};

fn brdf(s: Surface, n: vec3<f32>, v: vec3<f32>, l: vec3<f32>) -> vec3<f32> {
  let h = normalize(v + l);
  let n_dot_l = max(dot(n, l), 0.0);
  let n_dot_v = max(dot(n, v), 1e-4);
  let n_dot_h = max(dot(n, h), 0.0);
  let h_dot_v = max(dot(h, v), 0.0);
  let a = s.roughness * s.roughness;
  let d = d_ggx(n_dot_h, a);
  let g = g_smith(n_dot_v, n_dot_l, s.roughness);
  let f = f_schlick(h_dot_v, s.f0);
  let spec = (d * g * f) / (4.0 * n_dot_v * n_dot_l + 1e-4);
  let kd = (vec3<f32>(1.0) - f) * (1.0 - s.metallic);
  let base = (kd * s.albedo / PI + spec) * n_dot_l;
  if (s.coat <= 0.0) { return base; }
  // The coat (Filament's clear coat): GGX with a fixed 4 % reflectance and
  // Kelemen's visibility; what it reflects never reaches the base.
  let nc = s.coat_normal;
  let c_dot_l = max(dot(nc, l), 0.0);
  let ac = s.coat_roughness * s.coat_roughness;
  let fc = f_schlick(h_dot_v, vec3<f32>(0.04)).x * s.coat;
  let dc = d_ggx(max(dot(nc, h), 0.0), ac);
  let vc = 0.25 / max(h_dot_v * h_dot_v, 1e-4);
  return base * (1.0 - fc) + vec3<f32>(dc * vc * fc * c_dot_l);
}

// Octahedral encoding of a unit vector into -1..1 (two channels).
fn oct_encode(n: vec3<f32>) -> vec2<f32> {
  let p = n.xy / (abs(n.x) + abs(n.y) + abs(n.z));
  if (n.z >= 0.0) { return p; }
  return (1.0 - abs(p.yx)) * select(vec2<f32>(-1.0), vec2<f32>(1.0), p >= vec2<f32>(0.0));
}

// Caustics under the frame's water: two scrolling cell patterns (bright
// where the waves focus light), fading in below the surface.
fn caustics(p: vec3<f32>) -> f32 {
  let c = frame.caustics;
  if (c.y <= 0.0) { return 0.0; }
  let area = frame.caustics_area;
  if (any(p.xz < area.xy) || any(p.xz > area.zw)) { return 0.0; }
  let depth = c.x - p.y;
  if (depth <= 0.0) { return 0.0; }
  let t = frame.time.x;
  let q = p.xz * c.z;
  let a = caustic_cells(q + vec2<f32>(t * 0.21, t * 0.13));
  let b = caustic_cells(q * 1.37 + vec2<f32>(-t * 0.17, t * 0.23) + 3.7);
  let fade = smoothstep(0.0, 0.4, depth) * exp(-depth * c.w);
  return pow(min(a, b), 3.0) * 4.0 * c.y * fade;
}

fn caustic_hash(p: vec2<f32>) -> vec2<f32> {
  let q = vec2<f32>(dot(p, vec2<f32>(127.1, 311.7)), dot(p, vec2<f32>(269.5, 183.3)));
  return fract(sin(q) * 43758.5453);
}

// 1 on the cell borders of a jittered grid, falling off towards the centres.
fn caustic_cells(p: vec2<f32>) -> f32 {
  let i = floor(p);
  let f = fract(p);
  var d1 = 8.0;
  var d2 = 8.0;
  for (var y = -1; y <= 1; y++) {
    for (var x = -1; x <= 1; x++) {
      let o = vec2<f32>(f32(x), f32(y));
      let r = o + caustic_hash(i + o) - f;
      let d = dot(r, r);
      if (d < d1) { d2 = d1; d1 = d; } else if (d < d2) { d2 = d; }
    }
  }
  return 1.0 - clamp((sqrt(d2) - sqrt(d1)) * 2.2, 0.0, 1.0);
}
