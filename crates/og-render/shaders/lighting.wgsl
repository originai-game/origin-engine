// Lighting shared by the lit passes (pbr.wgsl, terrain.wgsl, water.wgsl):
// the GGX BRDF, the directional light's cascaded shadows and the local
// lights' shadows, read with explicit compares.
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
// textureLoad and explicit compares give the same result on every backend.
// `slope` is the receiver plane's depth change per texel
// ([`receiver_slope`]): each texel is compared against the receiver's own
// depth there, so a surface tilted from the light does not shadow itself
// across the kernel.
fn pcf(map: texture_2d_array<f32>, uv: vec2<f32>, layer: u32, ref_depth: f32, slope: vec2<f32>) -> f32 {
  let size = vec2<i32>(textureDimensions(map));
  let p = uv * vec2<f32>(size) - 0.5;
  let base = vec2<i32>(floor(p));
  let f = fract(p);
  var lit: array<f32, 4>;
  for (var i = 0; i < 4; i++) {
    let o = vec2<i32>(i & 1, i >> 1u);
    let at = clamp(base + o, vec2<i32>(0), size - 1);
    let r = ref_depth + dot(slope, vec2<f32>(o) - f);
    lit[i] = select(0.0, 1.0, r <= textureLoad(map, at, i32(layer), 0).r);
  }
  return mix(mix(lit[0], lit[1], f.x), mix(lit[2], lit[3], f.x), f.y);
}

// A wider PCF for the sun's shadows on lit surfaces (the local lights'
// take [`local_wide`]): the 3x3 bilinear PCFs one texel apart summed, as
// one pass over the 4x4 texels around uv with separable tent weights
// (1 - f, 1, 1, f) / 3 per axis (Castaño's filter for The Witness, as Unity's
// SampleShadow_ComputeSamples_Tent_5x5). Edges come out soft over about
// three texels and straight instead of stepping texel by texel; the
// receiver plane (`slope`, as pcf's) keeps the wider kernel off the
// surface's own depth.
fn pcf_tent(map: texture_2d_array<f32>, uv: vec2<f32>, layer: u32, ref_depth: f32, slope: vec2<f32>) -> f32 {
  let size = vec2<i32>(textureDimensions(map));
  let p = uv * vec2<f32>(size) - 0.5;
  let base = vec2<i32>(floor(p)) - 1;
  let f = fract(p);
  let wx = array<f32, 4>(1.0 - f.x, 1.0, 1.0, f.x);
  let wy = array<f32, 4>(1.0 - f.y, 1.0, 1.0, f.y);
  var lit = 0.0;
  for (var j = 0; j < 4; j++) {
    var row = 0.0;
    for (var i = 0; i < 4; i++) {
      let at = clamp(base + vec2<i32>(i, j), vec2<i32>(0), size - 1);
      let r = ref_depth + dot(slope, vec2<f32>(f32(i - 1), f32(j - 1)) - f);
      row += wx[i] * select(0.0, 1.0, r <= textureLoad(map, at, i32(layer), 0).r);
    }
    lit += wy[j] * row;
  }
  return lit / 9.0;
}

// Receiver-plane depth bias (Isidoro, "Shadow Mapping: GPU-based Tips and
// Techniques", GDC 2006): the receiver's depth change per shadow texel,
// from `a` (the sample's uv and reference depth) and `b`, `c` (two points
// a little way along the surface, projected the same way). Taken from the
// surface's plane rather than screen derivatives, which these lookups'
// control flow does not allow. Towards a grazing light the plane runs
// along the light's rays and the slope grows without bound: it is held to
// what a light at tan 4 (76 degrees) gives, as PCSS's slope term, and the
// constant biases cover the rest.
fn receiver_slope(a: vec3<f32>, b: vec3<f32>, c: vec3<f32>, size: vec2<f32>, n_dot_l: f32) -> vec2<f32> {
  let d1 = (b.xy - a.xy) * size;
  let d2 = (c.xy - a.xy) * size;
  let det = d1.x * d2.y - d1.y * d2.x;
  if (abs(det) < 1e-12) { return vec2<f32>(0.0); }
  let z1 = b.z - a.z;
  let z2 = c.z - a.z;
  let g = vec2<f32>(z1 * d2.y - z2 * d1.y, z2 * d1.x - z1 * d2.x) / det;
  let sin_l = sqrt(max(1.0 - n_dot_l * n_dot_l, 0.0));
  return g * min(1.0, 4.0 * n_dot_l / max(sin_l, 1e-6));
}

// Two directions along the plane of normal n.
fn plane_tangents(n: vec3<f32>) -> array<vec3<f32>, 2> {
  let k = select(vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(0.0, 1.0, 0.0), abs(n.x) > 0.9);
  let t = normalize(cross(n, k));
  return array<vec3<f32>, 2>(t, cross(n, t));
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

// The cascades debug view's colour of cascade `c`: red, green, blue,
// yellow; black beyond the last.
fn cascade_tint(c: u32) -> vec3<f32> {
  switch c {
    case 0u: { return vec3<f32>(1.0, 0.0, 0.0); }
    case 1u: { return vec3<f32>(0.0, 1.0, 0.0); }
    case 2u: { return vec3<f32>(0.0, 0.0, 1.0); }
    case 3u: { return vec3<f32>(1.0, 1.0, 0.0); }
    default: { return vec3<f32>(0.0); }
  }
}

// Whether the surface being lit is a closed, single-sided one
// (StandardMaterial side front or back: material mode bit 32), set by the
// standard and physical fragment stages; every other caller leaves it
// false. Front-sided casters store their far side (shadow_pipeline), so
// such a receiver is never in the map itself: what lies at its own depth
// there is a caster resting on it (a box's bottom face on the floor), and
// it is compared a quarter texel past its depth with no receiver plane,
// so that contact reads as shadow. The receiver plane and a depth bias
// towards the light are for a double-sided surface (a leaf, a cloth),
// which is in the map and would otherwise shadow itself; on a closed
// receiver they read a caster's coplanar base as lit and left a lit line
// along the far side of every box standing on a floor (a CPU model of
// local_shadows_touch_their_casters' scene under a spot: the floor 1 mm
// past the face 53 % lit under the tent with the plane, 24 % without;
// under 1 % this way with local_wide, the floor before its lit face 98 %
// lit 2 mm from it; a paraboloid's 5 % and 80 %, 92 % at 1 cm).
var<private> shadow_closed: bool = false;

// A turn in [0, 2π) for PCSS's spiral at `texel` (a position in shadow
// texels): interleaved gradient noise (Jimenez, "Next Generation Post
// Processing in Call of Duty: Advanced Warfare", SIGGRAPH 2014; Bevy turns
// its shadow spirals the same way), so neighbouring pixels turn their
// spirals apart, and under TAA (frame.shadow.w: 1 + the drawn frame of 16)
// every drawn frame again, so the history averages the 16 taps into the
// smooth disc they sample.
fn shadow_turn(texel: vec2<f32>) -> f32 {
  let k = max(frame.shadow.w - 1.0, 0.0);
  return 6.2831853 * fract(52.9829189 * fract(dot(texel + k * 5.588238, vec2<f32>(0.06711056, 0.00583715))));
}

// The sun's soft shadows (PCSS, `DirectionalLight::shadow_softness`, and
// the weather's: frame.shadow.x): a blocker search over the widest
// penumbra the cascade allows, then 16 bilinear PCF taps over the penumbra
// the blockers' mean depth gives (softness × the distance from blocker to
// receiver, in texels). A Vogel disc spreads the taps, turned per pixel
// and per drawn frame ([`shadow_turn`]): a fixed disc printed its spiral
// as rings of copies across a wide penumbra. A slope term keeps the wide
// kernel off a receiver tilted from the light (an open one; a closed one
// compares past its own depth, as [`shadow_closed`]). Every read is a
// textureLoad compare, as pcf's.
const PCSS_TAPS: i32 = 16;
const PCSS_MAX_TEXELS: f32 = 24.0;

fn vogel(i: i32, n: i32, radius: f32, turn: f32) -> vec2<f32> {
  let r = sqrt((f32(i) + 0.5) / f32(n)) * radius;
  let a = f32(i) * 2.39996323 + turn;
  return vec2<f32>(cos(a), sin(a)) * r;
}

fn pcss(uv: vec2<f32>, c: u32, ref_depth: f32, bias: vec4<f32>, tan_l: f32) -> f32 {
  let size = vec2<f32>(textureDimensions(cascade_map));
  let isize = vec2<i32>(size);
  // World units per texel, and the cascade's depth range in world units.
  let texel = bias.x;
  let depth_range = texel / max(bias.y, 1e-12);
  let softness = frame.shadow.x;
  // How the receiver's light-space depth changes per texel along it (an
  // open receiver's; a closed one is compared a quarter texel past its
  // depth instead, as shadow_closed says).
  let slope = select(bias.y * min(tan_l, 4.0), 0.0, shadow_closed);
  let reference = select(ref_depth, ref_depth + 1.25 * bias.y, shadow_closed);
  let turn = shadow_turn(uv * size);
  let search = clamp(softness * ref_depth * depth_range / texel, 1.0, PCSS_MAX_TEXELS);
  var blockers = 0.0;
  var sum = 0.0;
  for (var i = 0; i < PCSS_TAPS; i++) {
    let o = vogel(i, PCSS_TAPS, search, turn);
    let at = clamp(vec2<i32>(floor(uv * size + o)), vec2<i32>(0), isize - 1);
    let d = textureLoad(cascade_map, at, i32(c), 0).r;
    if (d < reference - slope * length(o)) {
      blockers += 1.0;
      sum += d;
    }
  }
  if (blockers < 0.5) { return 1.0; }
  let blocker = sum / blockers;
  let penumbra = clamp(softness * (ref_depth - blocker) * depth_range / texel, 1.0, PCSS_MAX_TEXELS);
  var lit = 0.0;
  for (var i = 0; i < PCSS_TAPS; i++) {
    let o = vogel(i, PCSS_TAPS, penumbra, turn);
    lit += pcf(cascade_map, uv + o / size, c, reference - slope * length(o), vec2<f32>(0.0));
  }
  return lit / f32(PCSS_TAPS);
}

// One cascade's shadow at p (-1: beyond its depth, lit). The bias is in
// world units scaled by the cascade's texel size (a normal offset growing
// with the light's angle to the surface, plus a depth bias converted to the
// cascade's normalized depth), so it holds at any scene scale.
// `wide` filters with the tent (over the receiver plane for an open
// receiver; [`shadow_closed`]); the volumetric march takes the 2x2 PCF,
// as it averages many samples anyway.
fn cascade_sample(c: u32, p: vec3<f32>, n: vec3<f32>, n_dot_l: f32, wide: bool) -> f32 {
  let bias = frame.cascade_bias[c];
  let sin_l = sqrt(max(1.0 - n_dot_l * n_dot_l, 0.0));
  let offset = p + n * bias.x * (0.25 + sin_l);
  let sp = frame.cascade_view_proj[c] * vec4<f32>(offset, 1.0);
  let uv = vec2<f32>(sp.x * 0.5 + 0.5, 0.5 - sp.y * 0.5);
  // Outside the map (a fixed shadow box, fit_box) reads as lit, as three.js's
  // inFrustum does; the PCF taps clamp to the edge texel and would otherwise
  // extend whatever caster sits on the border across everything beyond it.
  if (sp.z > 1.0 || any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0))) { return -1.0; }
  if (frame.shadow.x > 0.0) {
    return pcss(uv, c, sp.z - bias.y, bias, sin_l / max(n_dot_l, 0.05));
  }
  if (!wide) { return pcf(cascade_map, uv, c, sp.z - bias.y, vec2<f32>(0.0)); }
  // A closed receiver: a quarter texel past its own depth, flat.
  if (shadow_closed) { return pcf_tent(cascade_map, uv, c, sp.z + 0.25 * bias.y, vec2<f32>(0.0)); }
  // The receiver plane, a texel along each tangent (orthographic: exact).
  let t = plane_tangents(n);
  let sb = frame.cascade_view_proj[c] * vec4<f32>(offset + t[0] * bias.x, 1.0);
  let sc = frame.cascade_view_proj[c] * vec4<f32>(offset + t[1] * bias.x, 1.0);
  let size = vec2<f32>(textureDimensions(cascade_map));
  let slope = receiver_slope(
    vec3<f32>(uv, sp.z),
    vec3<f32>(sb.x * 0.5 + 0.5, 0.5 - sb.y * 0.5, sb.z),
    vec3<f32>(sc.x * 0.5 + 0.5, 0.5 - sc.y * 0.5, sc.z),
    size,
    n_dot_l,
  );
  return pcf_tent(cascade_map, uv, c, sp.z - bias.y, slope);
}

// The cascades' shadow at p: across a cascade's blend band (cascade_bias.w
// to its far split, `DirectionalLight::cascade_blend`) it fades into the
// next one, which covers that band too. Shadows fade out over the last
// tenth of the shadow distance, and by frame.shadow.z everywhere: the share
// of the sun's light a cloud cover or the weather has scattered across
// the sky (`DirectionalLight::diffusion`), which reaches the shadow too.
fn cascade_shadow(p: vec3<f32>, n: vec3<f32>, n_dot_l: f32) -> f32 {
  return cascade_shadow_lookup(p, n, n_dot_l, true);
}

fn cascade_shadow_lookup(p: vec3<f32>, n: vec3<f32>, n_dot_l: f32, wide: bool) -> f32 {
  let c = cascade_index(p);
  if (c >= CASCADES) { return 1.0; }
  var shadow = cascade_sample(c, p, n, n_dot_l, wide);
  if (shadow < 0.0) { return 1.0; }
  let count = u32(frame.light_dir.w);
  let depth = dot(p - frame.view_pos.xyz, frame.view_forward.xyz);
  let band = frame.cascade_bias[c].w;
  if (c + 1u < count && depth > band) {
    var next = cascade_sample(c + 1u, p, n, n_dot_l, wide);
    if (next < 0.0) { next = 1.0; }
    shadow = mix(shadow, next, smoothstep(band, frame.cascade_bias[c].z, depth));
  }
  let last = frame.cascade_bias[count - 1u].z;
  return mix(mix(shadow, 1.0, smoothstep(0.9 * last, last, depth)), 1.0, frame.shadow.z);
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

// A point on hemisphere `sigma`'s paraboloid (sigma 1 below the light):
// its uv and its distance / range scaled by `k` (the depth bias,
// [`local_bias`]).
fn paraboloid_uv_ref(light: LocalLight, q: vec3<f32>, sigma: f32, k: f32) -> vec3<f32> {
  let to = q - light.position_range.xyz;
  let l = vec3<f32>(to.x, sigma * to.z, -sigma * to.y);
  let d = max(length(l), 1e-6);
  let u = l / d;
  let ndc = vec2<f32>(-u.x, u.y) * (PARABOLOID_SCALE / (1.0 + u.z));
  return vec3<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5, d * k / light.position_range.w);
}

// A point seen by local shadow layer `layer` (a spot's or a cube face's):
// its uv and its distance / range scaled by `k` (w <= 0: behind the
// layer's view).
fn perspective_uv_ref(light: LocalLight, layer: u32, q: vec3<f32>, k: f32) -> vec4<f32> {
  let clip = frame.local_view_proj[layer] * vec4<f32>(q, 1.0);
  let ndc = clip.xy / clip.w;
  let r = length(q - light.position_range.xyz) * k / light.position_range.w;
  return vec4<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5, r, clip.w);
}

// The factor on a receiver's distance it is compared at: a tenth of a
// texel towards the light (`cone.w`), or, for a closed receiver
// ([`shadow_closed`]), a quarter of a texel past it.
fn local_bias(light: LocalLight) -> f32 {
  return select(1.0 - light.cone.w, 1.0 + 2.5 * light.cone.w, shadow_closed);
}

// Local shadows store distance / range; the fragment compares its own,
// filtered with [`local_wide`] (`local_shadow_lookup`'s `wide`).
fn local_shadow(light: LocalLight, p: vec3<f32>, n: vec3<f32>) -> f32 {
  return local_shadow_lookup(light, p, n, true);
}

// The local lights' filter for lit surfaces: contact hardening between
// two box-filtered bilinear PCFs over the same 8x8 texels, the tent's 3x3
// (its width at contact) and 7x7 (Castaño's separable weights, as
// pcf_tent's, widened: (1 - f, 1, ..., 1, f) / 7 per axis), mixed by how
// far behind the casters the receiver lies: the casters' mean distance
// among those texels (Fernando's PCSS blocker search, "Percentage-Closer
// Soft Shadows", 2005), all 7x7 at 30 % of their distance behind them. A
// shadow's edge far from its caster crosses the map's texels at a shallow
// angle, and the 3x3 printed their steps along it: a CPU model of the
// spot box's far edge (texels 2.45 cm there) put its 0.5 contour 0.53 cm
// rms (1.13 at most) off a straight line under the 3x3, 0.27 (0.61)
// under a 5x5, 0.14 (0.24) under the 7x7. Nearer contact the edge stays
// the tent's. The light has no size: the widening is in texels, an
// antialiasing filter of the map, not a penumbra. 64 reads a shadowed
// light per pixel, against the tent's 16.
fn local_wide(uv: vec2<f32>, layer: u32, ref_depth: f32, slope: vec2<f32>) -> f32 {
  let size = vec2<i32>(textureDimensions(local_shadow_map));
  let p = uv * vec2<f32>(size) - 0.5;
  let base = vec2<i32>(floor(p)) - 3;
  let f = fract(p);
  var wide = 0.0;
  var narrow = 0.0;
  var blockers = 0.0;
  var sum = 0.0;
  for (var j = 0; j < 8; j++) {
    let wy = select(select(1.0, f.y, j == 7), 1.0 - f.y, j == 0);
    let ny = select(0.0, select(select(1.0, f.y, j == 5), 1.0 - f.y, j == 2), j >= 2 && j <= 5);
    for (var i = 0; i < 8; i++) {
      let wx = select(select(1.0, f.x, i == 7), 1.0 - f.x, i == 0);
      let nx = select(0.0, select(select(1.0, f.x, i == 5), 1.0 - f.x, i == 2), i >= 2 && i <= 5);
      let at = clamp(base + vec2<i32>(i, j), vec2<i32>(0), size - 1);
      let d = textureLoad(local_shadow_map, at, i32(layer), 0).r;
      let r = ref_depth + dot(slope, vec2<f32>(f32(i - 3), f32(j - 3)) - f);
      let w = wx * wy;
      if (r <= d) {
        wide += w;
        narrow += nx * ny;
      } else {
        blockers += w;
        sum += w * d;
      }
    }
  }
  if (blockers <= 0.0) { return 1.0; }
  let blocker = max(sum / blockers, 1e-6);
  let t = clamp((ref_depth - blocker) / blocker / 0.3, 0.0, 1.0);
  return mix(narrow / 9.0, wide / 49.0, t);
}

// One filter of a local layer: the 2x2 PCF (`wide` false) or
// [`local_wide`].
fn local_filter(uv: vec2<f32>, layer: u32, ref_depth: f32, slope: vec2<f32>, wide: bool) -> f32 {
  if (!wide) { return pcf(local_shadow_map, uv, layer, ref_depth, vec2<f32>(0.0)); }
  return local_wide(uv, layer, ref_depth, slope);
}

// Both biases scale with the distance to the light, as its texels widen
// with it (`cone.z`: the normal offset per metre at a grazing light, `cone.w`:
// the depth bias as a fraction of the distance, [`local_bias`]), and the
// normal offset with the light's angle to the surface (a quarter of it
// facing the light), so a box standing on a floor casts its shadow from
// its base, not from a little way out. An open receiver is filtered over
// its plane ([`receiver_slope`]; a closed one is flat, as
// [`shadow_closed`] says). `wide` filters with [`local_filter`]'s wide
// kernels; the volumetric march takes the 2x2 PCF, as it averages many
// samples anyway.
fn local_shadow_lookup(light: LocalLight, p: vec3<f32>, n: vec3<f32>, wide: bool) -> f32 {
  let slot = i32(light.direction_slot.w);
  if (slot < 0) { return 1.0; }
  let range = light.position_range.w;
  let to_frag = p - light.position_range.xyz;
  let dist = length(to_frag);
  let n_dot_l = clamp(dot(n, -to_frag / max(dist, 1e-6)), 0.0, 1.0);
  let sin_l = sqrt(1.0 - n_dot_l * n_dot_l);
  let offset = p + n * light.cone.z * dist * (0.25 + sin_l);
  var layer = u32(slot);
  let size = vec2<f32>(textureDimensions(local_shadow_map));
  let k = local_bias(light);
  let plane = wide && !shadow_closed;
  // The receiver plane, about a texel along each tangent (a 90-degree
  // face's texel at this distance; the plane is what matters, not the step).
  let t = plane_tangents(n);
  let h = dist * 2.0 / size.x;
  if (light.color_kind.w < 0.5 && frame.shadow.y > 0.5) {
    let below = offset.y - light.position_range.y <= 0.0;
    let sigma = select(-1.0, 1.0, below);
    layer += select(1u, 0u, below);
    let a = paraboloid_uv_ref(light, offset, sigma, k);
    var slope = vec2<f32>(0.0);
    if (plane) {
      slope = receiver_slope(
        a,
        paraboloid_uv_ref(light, offset + t[0] * h, sigma, k),
        paraboloid_uv_ref(light, offset + t[1] * h, sigma, k),
        size,
        n_dot_l,
      );
    }
    return local_filter(a.xy, layer, a.z, slope, wide);
  }
  if (light.color_kind.w < 0.5) { layer += cube_face(to_frag); }
  let a = perspective_uv_ref(light, layer, offset, k);
  if (a.w <= 0.0) { return 1.0; }
  let uv = a.xy;
  if (light.color_kind.w > 0.5 && (any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0)))) { return 1.0; }
  let at = clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0));
  var slope = vec2<f32>(0.0);
  if (plane) {
    slope = receiver_slope(
      a.xyz,
      perspective_uv_ref(light, layer, offset + t[0] * h, k).xyz,
      perspective_uv_ref(light, layer, offset + t[1] * h, k).xyz,
      size,
      n_dot_l,
    );
  }
  return local_filter(at, layer, a.z, slope, wide);
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
  translucency: f32,
};

// SeedThree leaf-cards.js, Copyright (c) 2026 SkyeShark, MIT
// (third_party/vegetation/SeedThree.LICENSE): distorted back-light lobe.
// This is direct light, not emission: the caller applies its shadow/cookie.
fn leaf_transmission(s: Surface, n: vec3f, v: vec3f, l: vec3f) -> vec3f {
  if (s.translucency <= 0.0) { return vec3f(0.0); }
  let back = max(-dot(n, l), 0.0);
  let through = pow(max(dot(v, -normalize(l + n * 0.3)), 0.0), 6.0);
  return s.albedo * (1.0 - s.metallic) * s.translucency * back * (3.0 * through + 0.16);
}

// Bias a thin leaf towards the incident light on either side. Offsetting
// a backlit card away from the light makes it shadow its own transmission.
fn leaf_shadow_normal(s: Surface, n: vec3f, to_light: vec3f) -> vec3f {
  return select(n, -n, s.translucency > 0.0 && dot(n, to_light) < 0.0);
}

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
  let base = (kd * s.albedo / PI + spec) * n_dot_l + leaf_transmission(s, n, v, l);
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
  // `p` is from the frame's origin: each layer adds its cells there.
  let q = p.xz * c.z;
  let ca = frame.caustics_cells[0];
  let cb = frame.caustics_cells[1];
  let a = caustic_cells(q + ca.zw + vec2<f32>(t * 0.21, t * 0.13), ca.xy);
  let b = caustic_cells(q * 1.37 + cb.zw + vec2<f32>(-t * 0.17, t * 0.23) + 3.7, cb.xy);
  let fade = smoothstep(0.0, 0.4, depth) * exp(-depth * c.w);
  return pow(min(a, b), 3.0) * 4.0 * c.y * fade;
}

fn caustic_hash(p: vec2<f32>) -> vec2<f32> {
  let c = vec2<i32>(p);
  return vec2<f32>(lattice_hash(vec3<i32>(c, 1)), lattice_hash(vec3<i32>(c, 2)));
}

// 1 on the cell borders of a jittered grid, falling off towards the centres.
fn caustic_cells(p: vec2<f32>, whole: vec2<f32>) -> f32 {
  let i = floor(p) + whole;
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
