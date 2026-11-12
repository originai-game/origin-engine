// Lighting shared by the lit passes (pbr.wgsl, terrain.wgsl, water.wgsl):
// the GGX BRDF (its visibility is ggx.wgsl's), the directional light's
// cascaded shadows and the local lights' shadows, read with explicit
// compares. Prepended after frame.wgsl.

const PI: f32 = 3.14159265359;

fn d_ggx(n_dot_h: f32, a: f32) -> f32 {
  let a2 = a * a;
  let d = n_dot_h * n_dot_h * (a2 - 1.0) + 1.0;
  return a2 / (PI * d * d + 1e-7);
}

fn f_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
  return f0 + (1.0 - f0) * pow(saturate(1.0 - cos_theta), 5.0);
}

fn f_schlick_roughness(cos_theta: f32, f0: vec3<f32>, roughness: f32) -> vec3<f32> {
  return f0 + (max(vec3<f32>(1.0 - roughness), f0) - f0) * pow(saturate(1.0 - cos_theta), 5.0);
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
// take [`local_wide`]): two bilinear PCFs half a texel either side of uv
// per axis, summed, as one pass over the 3x3 texels around it with
// separable tent weights (1 - f, 1, f) / 2 per axis (Castaño's filter for
// The Witness at its smallest, as Unity's
// SampleShadow_ComputeSamples_Tent_3x3). Edges ramp over two texels and
// run straight instead of stepping texel by texel, while a caster a texel
// thin (a wire, a character's arm) keeps its shadow: the 4x4 tent's three
// PCFs a texel apart averaged those to a faint haze (Metal: smaa's wire
// grid, gltf-reference's figures). The receiver plane (`slope`, as pcf's)
// keeps the kernel off the surface's own depth.
fn pcf_tent(map: texture_2d_array<f32>, uv: vec2<f32>, layer: u32, ref_depth: f32, slope: vec2<f32>) -> f32 {
  let size = vec2<i32>(textureDimensions(map));
  let p = uv * vec2<f32>(size) - 1.0;
  let base = vec2<i32>(floor(p));
  let f = fract(p);
  let wx = array<f32, 3>(1.0 - f.x, 1.0, f.x);
  let wy = array<f32, 3>(1.0 - f.y, 1.0, f.y);
  var lit = 0.0;
  for (var j = 0; j < 3; j++) {
    var row = 0.0;
    for (var i = 0; i < 3; i++) {
      let at = clamp(base + vec2<i32>(i, j), vec2<i32>(0), size - 1);
      let r = ref_depth + dot(slope, vec2<f32>(f32(i), f32(j)) - 0.5 - f);
      row += wx[i] * select(0.0, 1.0, r <= textureLoad(map, at, i32(layer), 0).r);
    }
    lit += wy[j] * row;
  }
  return lit / 4.0;
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
// lit 2 mm from it). The cascades alone read it: a local layer holds its
// casters' midpoint ([`local_depth`]), where no receiver is in the map
// itself, closed or not. A closed floor of two coplanar quads facing apart
// (window-room.glb's) is a caster at its own depth and shadows itself
// under this rule; moving the cascades to midpoints too is a follow-up.
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
  let closed = shadow_closed && bias.w >= 0.0;
  let slope = select(bias.y * min(tan_l, 4.0), 0.0, closed);
  let reference = select(ref_depth, ref_depth + 1.25 * bias.y, closed);
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
  // The light's penumbra (its disc's radius in texels) convolved with the
  // hard sun's own filter, as a soft shadow is the hard one blurred by the
  // light: their variances add. pcf_tent averages two bilinear PCFs half
  // a texel either side per axis (variance 1/4 beyond the bilinear each
  // tap here already takes), the disc of radius r has r² / 4, so the disc
  // that carries both has r² + 1. A soft sun is then never sharper than
  // the hard one at contact (a texel there) and every edge is the light's
  // width on top of the map's filter, not the larger of the two.
  let disc = softness * (ref_depth - blocker) * depth_range / texel;
  let penumbra = min(sqrt(disc * disc + 1.0), PCSS_MAX_TEXELS);
  var lit = 0.0;
  for (var i = 0; i < PCSS_TAPS; i++) {
    let o = vogel(i, PCSS_TAPS, penumbra, turn);
    lit += pcf(cascade_map, uv + o / size, c, reference - slope * length(o), vec2<f32>(0.0));
  }
  lit /= f32(PCSS_TAPS);
  let fill = frame.shadow.z;
  if (fill <= 0.0) { return lit; }
  // The light a cover scatters (frame.shadow.z, at most 0.65 of the beam)
  // leaves the cloud near-diffuse, multiply scattered: it reaches the
  // shadow as far as the sky over the receiver is open, read in the map
  // over a 45-degree cone (the cosine lobe's median angle) round the light:
  // a disc as wide as the blockers stand above it, the share of its taps
  // nothing blocks. Under a pole or a plank most of it is open and the
  // shadow fills in; under a roof, a bridge or at a box's foot it stays
  // shade (a uniform fill lit the floor under a snowed-in shelter as the
  // open snow beside it, and floated the shelter).
  let cone = min((ref_depth - blocker) * depth_range / texel, 4.0 * PCSS_MAX_TEXELS);
  var open = 0.0;
  for (var i = 0; i < PCSS_TAPS; i++) {
    let o = vogel(i, PCSS_TAPS, cone, turn + 1.3);
    let at = clamp(vec2<i32>(floor(uv * size + o)), vec2<i32>(0), isize - 1);
    open += select(0.0, 1.0, textureLoad(cascade_map, at, i32(c), 0).r >= reference - slope * length(o));
  }
  return mix(lit, 1.0, fill * open / f32(PCSS_TAPS));
}

// One cascade's shadow at p (-1: beyond its depth, lit). The bias is in
// world units scaled by the cascade's texel size (a normal offset growing
// with the light's angle to the surface, plus a depth bias converted to the
// cascade's normalized depth), so it holds at any scene scale.
// `wide` filters with the tent (over the receiver plane for an open
// receiver; [`shadow_closed`]); the volumetric march takes the 2x2 PCF,
// as it averages many samples anyway.
// Whether a pipeline has the sun's soft shadows (pipeline_features.rs).
override OG_PCSS: bool = true;

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
  // Nested, so a pipeline without soft shadows folds the search away.
  if (OG_PCSS) {
    if (frame.shadow.x > 0.0) {
      return pcss(uv, c, sp.z - bias.y, bias, sin_l / max(n_dot_l, 0.05));
    }
  }
  if (!wide) { return pcf(cascade_map, uv, c, sp.z - bias.y, vec2<f32>(0.0)); }
  // A closed receiver: a quarter texel past its own depth, flat. Else
  // along the receiver plane, a texel along each tangent (orthographic:
  // exact). A fixed box (bias.w < 0) always compares towards the light
  // by the game's bias. One tent filter for both (one copy of its code).
  var depth = sp.z + 0.25 * bias.y;
  var slope = vec2<f32>(0.0);
  if (!shadow_closed || bias.w < 0.0) {
    let t = plane_tangents(n);
    let sb = frame.cascade_view_proj[c] * vec4<f32>(offset + t[0] * bias.x, 1.0);
    let sc = frame.cascade_view_proj[c] * vec4<f32>(offset + t[1] * bias.x, 1.0);
    let size = vec2<f32>(textureDimensions(cascade_map));
    slope = receiver_slope(
      vec3<f32>(uv, sp.z),
      vec3<f32>(sb.x * 0.5 + 0.5, 0.5 - sb.y * 0.5, sb.z),
      vec3<f32>(sc.x * 0.5 + 0.5, 0.5 - sc.y * 0.5, sc.z),
      size,
      n_dot_l,
    );
    depth = sp.z - bias.y;
  }
  return pcf_tent(cascade_map, uv, c, depth, slope);
}

// The cascades' shadow at p: across a cascade's blend band (cascade_bias.w
// to its far split, `DirectionalLight::cascade_blend`) it fades into the
// next one, which covers that band too. Shadows fade out over the last
// tenth of the shadow distance. The light a cloud cover or the weather has
// scattered (`DirectionalLight::diffusion`, frame.shadow.z) fills them in
// where the sky round the sun is open ([`pcss`]).
fn cascade_shadow(p: vec3<f32>, n: vec3<f32>, n_dot_l: f32) -> f32 {
  return cascade_shadow_lookup(p, n, n_dot_l, true);
}

fn cascade_shadow_lookup(p: vec3<f32>, n: vec3<f32>, n_dot_l: f32, wide: bool) -> f32 {
  let c = cascade_index(p);
  if (c >= CASCADES) { return 1.0; }
  let count = u32(frame.light_dir.w);
  let depth = dot(p - frame.view_pos.xyz, frame.view_forward.xyz);
  let band = frame.cascade_bias[c].w;
  // This cascade and, across its blend band, the next, in one loop: one
  // copy of the filter's code (DXC inlines every call).
  let taps = select(1u, 2u, c + 1u < count && depth > band);
  var shadow = 1.0;
  for (var k = 0u; k < taps; k++) {
    var s = cascade_sample(c + k, p, n, n_dot_l, wide);
    if (k == 0u) {
      if (s < 0.0) { return 1.0; }
      shadow = s;
    } else {
      if (s < 0.0) { s = 1.0; }
      shadow = mix(shadow, s, smoothstep(band, frame.cascade_bias[c].z, depth));
    }
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
// texel towards the light (`cone.w`). Every receiver is open here: a local
// layer holds its casters' midpoint ([`local_depth`]), inside a closed caster
// and at a zero-thickness one's plane, so a surface lies before the depth
// it is compared with and the receivers behind a caster past it. The flat,
// past-its-depth rule of a closed receiver (`shadow_closed`) is the
// cascades' alone.
fn local_bias(light: LocalLight) -> f32 {
  return 1.0 - light.cone.w;
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
      let d = local_depth(at, layer);
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

// A local layer's depth at a texel: the midpoint of its casters' far
// sides (red) and near sides (green), each the nearest of its kind
// (shadow_target.rs; Woo, "The Shadow Depth Map Revisited", Graphics Gems
// III, 1992). It lies inside a closed caster and at the plane of a
// zero-thickness one (a double-sided card, two coplanar quads facing
// apart), so no surface compares against itself. Where no far side was
// drawn (1: past the range) nothing is: a single-sided open surface (a
// plane, a wall) casts nothing, as it did when the layers held far sides
// alone, rather than shadowing the receivers it meets at a crease through
// the filter's taps on it. Where no near side was drawn, the far side is
// the depth.
fn local_depth(at: vec2<i32>, layer: u32) -> f32 {
  let v = textureLoad(local_shadow_map, at, i32(layer), 0);
  return select(0.5 * (v.r + v.g), v.r, v.r >= 1.0 || v.g >= 1.0);
}

// `pcf` over a local layer's depths ([`local_depth`]).
fn pcf_local(uv: vec2<f32>, layer: u32, ref_depth: f32) -> f32 {
  let size = vec2<i32>(textureDimensions(local_shadow_map));
  let p = uv * vec2<f32>(size) - 0.5;
  let base = vec2<i32>(floor(p));
  let f = fract(p);
  var lit: array<f32, 4>;
  for (var i = 0; i < 4; i++) {
    let at = clamp(base + vec2<i32>(i & 1, i >> 1u), vec2<i32>(0), size - 1);
    lit[i] = select(0.0, 1.0, ref_depth <= local_depth(at, layer));
  }
  return mix(mix(lit[0], lit[1], f.x), mix(lit[2], lit[3], f.x), f.y);
}

// One filter of a local layer: the 2x2 PCF (`wide` false) or
// [`local_wide`].
fn local_filter(uv: vec2<f32>, layer: u32, ref_depth: f32, slope: vec2<f32>, wide: bool) -> f32 {
  if (!wide) { return pcf_local(uv, layer, ref_depth); }
  return local_wide(uv, layer, ref_depth, slope);
}

// Both biases scale with the distance to the light, as its texels widen
// with it (`cone.z`: the normal offset per metre at a grazing light, `cone.w`:
// the depth bias as a fraction of the distance, [`local_bias`]), and the
// normal offset with the light's angle to the surface (a quarter of it
// facing the light), so a box standing on a floor casts its shadow from
// its base, not from a little way out. An open receiver is filtered over
// its plane ([`receiver_slope`]). `wide` filters with [`local_filter`]'s wide
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
  let paraboloid = light.color_kind.w < 0.5 && frame.shadow.y > 0.5;
  let k = local_bias(light);
  let plane = wide;
  // The receiver plane, about a texel along each tangent (a 90-degree
  // face's texel at this distance; the plane is what matters, not the step).
  let t = plane_tangents(n);
  let h = dist * 2.0 / size.x;
  if (paraboloid) {
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
  return s.albedo * (1.0 - s.metallic) * s.translucency * back * (3.0 * leaf_forward(n, v, l) + 0.16);
}

// The lobe's forward-scattered part alone, 0..1: strongest looking through
// the leaf (normal `n`, towards the eye `v`) into the light `l`.
fn leaf_forward(n: vec3f, v: vec3f, l: vec3f) -> f32 {
  return pow(max(dot(v, -normalize(l + n * 0.3)), 0.0), 6.0);
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
  let f = f_schlick(h_dot_v, s.f0);
  let spec = d * v_ggx(n_dot_v, n_dot_l, a) * f;
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

// The frame's material style (og.MaterialStyle; frame.toon,
// frame.outline_style, frame.shadow_tint): one shading rule for every lit
// surface, so a style changes the whole picture and not only its meshes.
// A mesh's material adds its own ramp, highlight and rim over the same
// bands (pbr.wgsl toon_ramp); the terrain, the grass, scattered copies,
// impostors and crowd cards, which carry no such maps, call style_sun,
// style_irradiance and style_local.

// The outline target's texel (Rgba8Unorm, read back as bytes by
// fxpost.wgsl `fs_outline`): the colour in 5:5:5 with the clump bit
// between green and blue (r the high byte, g the low), the width in
// eighths of a pixel (0: no line of its own), the object's id. A clump
// (a tree's crown, a scattered copy, the terrain) is lined only against
// the sky or another id, never between its own leaves or folds.
fn outline_bytes(color: vec3<f32>, width: f32, object: u32, clump: bool) -> vec4<f32> {
  let c = vec3<u32>(round(clamp(color, vec3<f32>(0.0), vec3<f32>(1.0)) * 31.0));
  let packed = (c.r << 11u) | (c.g << 6u) | (u32(clump) << 5u) | c.b;
  let w = u32(round(clamp(width, 0.0, 31.875) * 8.0));
  return vec4<f32>(f32(packed >> 8u), f32(packed & 255u), f32(w), f32(object)) / 255.0;
}

// A style line's width at the world point `p`: with the style's outline
// distance, nearer than it the line grows, up to four times its width (a
// ground a metre under the camera is not lined 30 px wide), farther it
// thins in proportion (never for an orthographic camera). Never under a
// pixel, where a far silhouette wants ink (inked-comic-3d's hills), unless
// `thin` (foliage): its line never grows past its width and fades out by
// coverage far off, so a wood is not filled with ink near or far.
fn style_line_width(width: f32, p: vec3<f32>, thin: bool) -> f32 {
  if (frame.outline_style.x <= 0.0 || frame.view_pos.w >= 0.5 || width <= 0.0) { return width; }
  let depth = max(dot(p - frame.view_pos.xyz, frame.view_forward.xyz), 0.01);
  let w = width * frame.outline_style.x / depth;
  if (thin) { return min(w, width); }
  return clamp(w, 1.0, max(min(31.0, width * 4.0), 1.0));
}

// A clump's id in the outline target, 1 to 255, from where it stands
// (pbr.wgsl object_id for an instance): neighbouring clumps share one in 255.
fn clump_id(at: vec3<f32>) -> u32 {
  var h = 2166136261u;
  h = (h ^ bitcast<u32>(at.x)) * 16777619u;
  h = (h ^ bitcast<u32>(at.y)) * 16777619u;
  h = (h ^ bitcast<u32>(at.z)) * 16777619u;
  h = h ^ (h >> 16u);
  return 1u + h % 255u;
}

// Whether the frame's style is toon.
fn style_toon() -> bool {
  return frame.outline_style.z > 0.5;
}

// A band's step at `edge`: 0 below, 1 above, over `aa` either side (hard at 0).
fn style_step(x: f32, edge: f32, aa: f32) -> f32 {
  if (aa <= 0.0) { return select(0.0, 1.0, x > edge); }
  return smoothstep(edge - aa, edge + aa, x);
}

// The built-in toon ramp: N·L (times shadow) to the light received, in
// three bands, each edge over `aa`.
fn style_bands(t: f32, aa: f32) -> f32 {
  return 0.22 + 0.38 * style_step(t, 0.08, aa) + 0.4 * style_step(t, 0.45, aa);
}

// A band edge's half width for a fragment whose ramp input is `t`: its
// change over a pixel times the style's softness in pixels, never under
// one pixel (a harder step crawls on blades and leaves). Screen
// derivatives: called in uniform control flow.
fn style_edge(t: f32) -> f32 {
  return clamp(fwidthFine(t), 1e-4, 0.25) * max(frame.toon.x, 1.0);
}

// The sun on a surface under the style, and what the surface keeps of its
// ambient light (the style's shadow darkness).
struct StyleSun {
  direct: vec3<f32>,
  keep: f32,
};

// `shadow`: the sun's visibility; `aa`: style_edge of N·L. Lit: the BRDF,
// shadowed. Toon: the built-in bands on the surface's colour, matte (a
// metal in its own colour). Then the style's shadow darkness and its
// shadow tint, which takes what the shadow took of the light, once.
fn style_sun(s: Surface, n: vec3<f32>, v: vec3<f32>, l: vec3<f32>, shadow: f32, aa: f32) -> StyleSun {
  return style_sun_of(brdf(s, n, v, l) * shadow, s, n, l, shadow, aa);
}

// style_sun for a surface with its own lit term, `sunlit`: the sun's light
// on it per unit of the sun's colour, already shadowed (an impostor's,
// summed over the surfaces of a texel).
fn style_sun_of(sunlit: vec3<f32>, s: Surface, n: vec3<f32>, l: vec3<f32>, shadow: f32, aa: f32) -> StyleSun {
  let n_dot_l = max(dot(n, l), 0.0);
  let toon = style_toon();
  var direct = sunlit;
  var tinted = (1.0 - shadow) * n_dot_l;
  var lit = smoothstep(0.0, 0.25, n_dot_l * shadow);
  if (toon) {
    let dark = style_bands(0.0, 0.0);
    let band = style_bands(n_dot_l * shadow, aa);
    direct = mix(s.albedo, s.f0, s.metallic) / PI * band;
    tinted = style_bands(n_dot_l, aa) - band;
    lit = clamp((band - dark) / (1.0 - dark), 0.0, 1.0);
  }
  var o: StyleSun;
  o.keep = 1.0 - frame.outline_style.y * (1.0 - lit);
  direct *= o.keep;
  if (frame.shadow_tint.w > 0.0 && n_dot_l > 0.0) {
    var tint = frame.shadow_tint.rgb * frame.shadow_tint.w;
    // Never more than the diffuse light the ramp took: a white tint fills the shadow, never outshines the lit side.
    if (toon) { tint = min(tint, vec3<f32>(1.0 / PI)); }
    direct += tinted * tint * s.albedo;
  }
  o.direct = direct * frame.light_color.rgb;
  return o;
}

// The diffuse ambient light on a surface under the style, from the
// surface's own `irradiance` along its normal: under toon mixed towards one
// flat colour, the light from above and below averaged (MToon's equalised
// ambient; frame.toon.y 0 flat, 1 as given), so the bands sit on flat
// colour. No screen derivatives.
fn style_irradiance(irradiance: vec3<f32>, p: vec3<f32>) -> vec3<f32> {
  if (!style_toon() || frame.toon.y >= 1.0) { return irradiance; }
  let up = vec3<f32>(0.0, 1.0, 0.0);
  let flat_env = 0.5 * (ambient_light_level(up, p) + ambient_light_level(-up, p));
  let flat_hemi = 0.5 * (frame.hemi_sky.rgb + frame.hemi_ground.rgb) * frame.hemi_sky.w;
  return mix(flat_env * frame.params.x + flat_hemi, irradiance, frame.toon.y);
}

// A point or spot light's light along `l` under the style: the BRDF, or
// under toon the bands, hard-edged.
fn style_local(s: Surface, n: vec3<f32>, v: vec3<f32>, l: vec3<f32>, shadow: f32) -> vec3<f32> {
  if (style_toon()) {
    return mix(s.albedo, s.f0, s.metallic) / PI * style_bands(max(dot(n, l), 0.0) * shadow, 0.0);
  }
  return brdf(s, n, v, l) * shadow;
}

// Octahedral encoding of a unit vector into -1..1 (two channels).
fn oct_encode(n: vec3<f32>) -> vec2<f32> {
  let p = n.xy / (abs(n.x) + abs(n.y) + abs(n.z));
  if (n.z >= 0.0) { return p; }
  return (1.0 - abs(p.yx)) * select(vec2<f32>(-1.0), vec2<f32>(1.0), p >= vec2<f32>(0.0));
}

// W2: a sea's surface curvature about the camera (water_caustics.wgsl), a uniform (the lit pipelines' storage
// buffers are spent): per texel its Hessian (xx, zz, xz) as three signed 10-bit words, each the square root of its
// share of CAUSTIC_H (finer near zero, where most of the sea's curvature lies).
struct CausticGrid { w: array<vec4<u32>, 1024> };
@group(0) @binding(12) var<uniform> caustic_grid: CausticGrid;
const CAUSTIC_H: f32 = 8.0;

fn caustic_word(q: u32) -> f32 {
  let s = (f32(q & 1023u) - 511.0) / 511.0;
  return sign(s) * s * s * CAUSTIC_H;
}

fn caustic_h(i: vec2<i32>, n: i32) -> vec3<f32> {
  let k = u32(clamp(i.y, 0, n - 1) * n + clamp(i.x, 0, n - 1));
  let w = caustic_grid.w[k / 4u][k % 4u];
  return vec3<f32>(caustic_word(w), caustic_word(w >> 10u), caustic_word(w >> 20u));
}

// The waves' own caustics at `p`, `depth` under a sea's surface (water_caustics.wgsl): the irradiance the surface's
// curvature focuses there, 1 / |det(I + d c H)| at the surface point the refracted sun came through, less 1 (0: as
// the flat surface lights it). Past the focus the rays cross and the pattern is chaos the grid cannot hold: the
// determinant's magnitude is held off zero by how far the receiver lies past the waves' focus.
fn wave_caustics(p: vec3<f32>, depth: f32) -> f32 {
  let g = frame.caustics_cells[0];
  let n = i32(g.w);
  let l = normalize(frame.light_dir.xyz);
  if (l.y <= 0.0) { return 0.0; }
  let t = refract(-l, vec3<f32>(0.0, 1.0, 0.0), 1.0 / 1.33);
  let s = p.xz - t.xz * (depth / max(-t.y, 0.05));
  let u = (s - g.xy) / g.z;
  let i = vec2<i32>(floor(u));
  let f = u - floor(u);
  let h = mix(mix(caustic_h(i, n), caustic_h(i + vec2<i32>(1, 0), n), f.x),
              mix(caustic_h(i + vec2<i32>(0, 1), n), caustic_h(i + vec2<i32>(1, 1), n), f.x), f.y);
  // Its edge fades out (the grid holds 25.6 m about the camera).
  let edge = max(abs(u.x / f32(n) - 0.5), abs(u.y / f32(n) - 0.5)) * 2.0;
  let keep = 1.0 - smoothstep(0.7, 1.0, edge);
  let k = depth * 0.248 / max(-t.y, 0.05);
  let det = (1.0 + k * h.x) * (1.0 + k * h.y) - k * k * h.z * h.z;
  let floor_det = 0.25 + 0.04 * depth;
  let e = 1.0 / max(abs(det), floor_det);
  return clamp(e - 1.0, -0.8, 3.0) * keep;
}

// Caustics under the frame's water: two scrolling cell patterns (bright
// where the waves focus light), fading in below the surface; under a sea
// (W2) its waves' own.
// Whether a pipeline has the caustics' code (pipeline_features.rs).
override OG_CAUSTICS: bool = true;

fn caustics(p: vec3<f32>) -> f32 {
  if (!OG_CAUSTICS) { return 0.0; }
  let c = frame.caustics;
  if (c.y <= 0.0) { return 0.0; }
  let area = frame.caustics_area;
  if (any(p.xz < area.xy) || any(p.xz > area.zw)) { return 0.0; }
  let depth = c.x - p.y;
  if (depth <= 0.0) { return 0.0; }
  if (frame.caustics_cells[1].w > 0.5) {
    return wave_caustics(p, depth) * c.y * smoothstep(0.0, 0.4, depth) * exp(-depth * c.w);
  }
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
