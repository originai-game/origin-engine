// Forward+ PBR (metallic-roughness): the directional light with 3 cascaded
// shadow maps, the point and spot lights of the fragment's cluster (the
// first two shadowed; clusters.rs), split-sum IBL. Linear HDR output; the
// post passes tonemap. Prepended by frame.wgsl, the light tables
// (clusters.rs), noise.wgsl and material.wgsl; skin_*.wgsl supplies
// `joint_matrix` for the skinned entry points.
//
// Effects (each behind a branch, so a material or frame without it shades
// exactly as before): projected decals before lighting, screen-space
// ambient occlusion on the image-based light, toon bands, rim light, the
// dissolve (discard plus a glowing edge) and the hit flash. `vs_prepass` /
// `fs_prepass` write the depth-normal prepass: world normal and
// screen-space velocity, view depth, and the material's outline.

// `FxUniform` in fx.rs: per-frame effect inputs of the lit passes.
struct DecalGpu {
  inv: mat4x4<f32>,       // world -> decal box (-0.5..0.5)
  axis: vec4<f32>,        // xyz: the box's +z in world (surfaces facing it are covered), w: cos of the cut-off angle
  uv_rect: vec4<f32>,     // u0, v0, u1, v1 in the atlas
  color: vec4<f32>,       // linear rgba
  emissive: vec4<f32>,    // rgb: emitted light, w: roughness (<0 keep)
};
struct Fx {
  view_proj_unjittered: mat4x4<f32>,
  prev_view_proj: mat4x4<f32>,
  screen: vec4<f32>,      // w, h, 1/w, 1/h
  ao: vec4<f32>,          // x: 1 when the AO texture is valid, y: intensity
  decal_info: vec4<f32>,  // x: decals, y: 1 with an atlas, z: cos fade width
  reserved: vec4<f32>,
  decals: array<DecalGpu, 32>,
};
@group(2) @binding(0) var<uniform> fx: Fx;
@group(2) @binding(1) var ssao_tex: texture_2d<f32>;
@group(2) @binding(2) var decal_atlas: texture_2d<f32>;
@group(2) @binding(3) var fx_sampler: sampler;

struct VertexIn {
  @location(0) position: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
  @location(3) color: vec4<f32>,
  // xyz along +u, w the bitangent's sign (0: none).
  @location(15) tangent: vec4<f32>,
};

struct InstanceIn {
  @location(4) m0: vec4<f32>,
  @location(5) m1: vec4<f32>,
  @location(6) m2: vec4<f32>,
  @location(7) m3: vec4<f32>,
};

// A skinned instance: its transform and where its joints start in the palette.
struct SkinnedInstanceIn {
  @location(4) m0: vec4<f32>,
  @location(5) m1: vec4<f32>,
  @location(6) m2: vec4<f32>,
  @location(7) m3: vec4<f32>,
  @location(8) palette: vec4<f32>,  // x: first joint, y: joints, z: first morph delta, w: morph targets
  @location(11) morph0: vec4<f32>,  // morph weights 0..3
  @location(12) morph1: vec4<f32>,  // morph weights 4..7
};

struct SkinIn {
  @location(9) joints: vec4<u32>,
  @location(10) weights: vec4<f32>,
};

// The previous frame's transform of the instance (motion vectors).
struct PrevIn {
  @location(11) p0: vec4<f32>,
  @location(12) p1: vec4<f32>,
  @location(13) p2: vec4<f32>,
  @location(14) p3: vec4<f32>,
};

struct VertexOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world_pos: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
  @location(3) color: vec4<f32>,
  @location(4) local_pos: vec3<f32>,
  // The world tangent and the bitangent's sign (w 0: none).
  @location(5) tangent: vec4<f32>,
};

struct PrepassOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world_pos: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
  @location(3) color: vec4<f32>,
  @location(4) local_pos: vec3<f32>,
  @location(5) cur_clip: vec4<f32>,
  @location(6) prev_clip: vec4<f32>,
};

// A normal's world direction: through the inverse transpose of the model's
// upper 3x3 (its cofactor matrix, flipped for a mirroring transform), so a
// non-uniform scale bends normals the right way.
fn world_normal(model: mat4x4<f32>, n: vec3<f32>) -> vec3<f32> {
  let a = model[0].xyz;
  let b = model[1].xyz;
  let c = model[2].xyz;
  let cofactor = mat3x3<f32>(cross(b, c), cross(c, a), cross(a, b));
  let mirrored = dot(a, cross(b, c)) < 0.0;
  return normalize(cofactor * n) * select(1.0, -1.0, mirrored);
}

fn lit_vertex(model: mat4x4<f32>, v: VertexIn) -> VertexOut {
  var world = model * vec4<f32>(v.position, 1.0);
  let n = world_normal(model, v.normal);
  // A material shader's vertex offset (none with the built-in hooks).
  var h: Hook;
  h.world = world.xyz;
  h.local = v.position;
  h.normal = n;
  h.uv = v.uv;
  h.color = v.color;
  h.time = frame.time.x;
  world = vec4<f32>(world.xyz + vertex_offset(h, shader_params()), world.w);
  var o: VertexOut;
  o.clip = frame.view_proj * world;
  o.world_pos = world.xyz;
  o.normal = n;
  o.uv = v.uv;
  o.color = v.color;
  o.local_pos = v.position;
  // A tangent follows the surface: through the model itself.
  let t = (model * vec4<f32>(v.tangent.xyz, 0.0)).xyz;
  o.tangent = vec4<f32>(select(vec3<f32>(0.0), normalize(t), dot(t, t) > 1e-20), v.tangent.w);
  return o;
}

// The weighted joint matrices of one vertex (identity without weights).
fn skin_matrix(s: SkinIn, palette: vec4<f32>) -> mat4x4<f32> {
  let total = s.weights.x + s.weights.y + s.weights.z + s.weights.w;
  if (total < 1e-6) {
    return mat4x4<f32>(vec4<f32>(1.0, 0.0, 0.0, 0.0), vec4<f32>(0.0, 1.0, 0.0, 0.0), vec4<f32>(0.0, 0.0, 1.0, 0.0), vec4<f32>(0.0, 0.0, 0.0, 1.0));
  }
  let w = s.weights / total;
  let base = u32(palette.x);
  let last = u32(max(palette.y, 1.0)) - 1u;
  return joint_matrix(base + min(s.joints.x, last)) * w.x
    + joint_matrix(base + min(s.joints.y, last)) * w.y
    + joint_matrix(base + min(s.joints.z, last)) * w.z
    + joint_matrix(base + min(s.joints.w, last)) * w.w;
}

@vertex
fn vs(v: VertexIn, i: InstanceIn) -> VertexOut {
  return lit_vertex(mat4x4<f32>(i.m0, i.m1, i.m2, i.m3), v);
}


struct Morphed {
  position: vec3<f32>,
  normal: vec3<f32>,
};

// Blendshapes: the weighted position and normal deltas of vertex `vi`
// (palette.z: the mesh's first delta, palette.w: its targets).
fn morphed(position: vec3<f32>, normal: vec3<f32>, vi: u32, palette: vec4<f32>, w0: vec4<f32>, w1: vec4<f32>) -> Morphed {
  var m: Morphed;
  m.position = position;
  m.normal = normal;
  let targets = u32(palette.w);
  if (targets == 0u) { return m; }
  let base = u32(palette.z) + vi * targets * 2u;
  var w = array<f32, 8>(w0.x, w0.y, w0.z, w0.w, w1.x, w1.y, w1.z, w1.w);
  for (var t = 0u; t < min(targets, 8u); t++) {
    if (w[t] != 0.0) {
      m.position += morph_delta(base + t * 2u).xyz * w[t];
      m.normal += morph_delta(base + t * 2u + 1u).xyz * w[t];
    }
  }
  return m;
}

fn morph_vertex(v: VertexIn, vi: u32, i: SkinnedInstanceIn) -> VertexIn {
  let m = morphed(v.position, v.normal, vi, i.palette, i.morph0, i.morph1);
  var o = v;
  o.position = m.position;
  o.normal = m.normal;
  return o;
}

@vertex
fn vs_skinned(@builtin(vertex_index) vi: u32, v: VertexIn, i: SkinnedInstanceIn, s: SkinIn) -> VertexOut {
  let model = mat4x4<f32>(i.m0, i.m1, i.m2, i.m3) * skin_matrix(s, i.palette);
  return lit_vertex(model, morph_vertex(v, vi, i));
}

fn prepass_vertex(model: mat4x4<f32>, prev: mat4x4<f32>, v: VertexIn) -> PrepassOut {
  let o = lit_vertex(model, v);
  var p: PrepassOut;
  p.clip = o.clip;
  p.world_pos = o.world_pos;
  p.normal = o.normal;
  p.uv = o.uv;
  p.color = o.color;
  p.local_pos = o.local_pos;
  p.cur_clip = fx.view_proj_unjittered * vec4<f32>(o.world_pos, 1.0);
  p.prev_clip = fx.prev_view_proj * (prev * vec4<f32>(v.position, 1.0));
  return p;
}

@vertex
fn vs_prepass(v: VertexIn, i: InstanceIn, prev: PrevIn) -> PrepassOut {
  return prepass_vertex(mat4x4<f32>(i.m0, i.m1, i.m2, i.m3), mat4x4<f32>(prev.p0, prev.p1, prev.p2, prev.p3), v);
}

@vertex
fn vs_prepass_skinned(@builtin(vertex_index) vi: u32, v: VertexIn, i: SkinnedInstanceIn, s: SkinIn) -> PrepassOut {
  let model = mat4x4<f32>(i.m0, i.m1, i.m2, i.m3) * skin_matrix(s, i.palette);
  return prepass_vertex(model, model, morph_vertex(v, vi, i));
}

const PI: f32 = 3.14159265359;

// False for materials and frames without effects: their code folds away.
override EFFECTS: bool = true;

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

// The bias is in world units scaled by the cascade's texel size (a normal
// offset growing with the light's angle to the surface, plus a depth bias
// converted to the cascade's normalized depth), so it holds at any scene
// scale. Shadows fade out over the last tenth of the shadow distance.
fn cascade_shadow(p: vec3<f32>, n: vec3<f32>, n_dot_l: f32) -> f32 {
  let c = cascade_index(p);
  if (c >= CASCADES) { return 1.0; }
  let bias = frame.cascade_bias[c];
  let sin_l = sqrt(max(1.0 - n_dot_l * n_dot_l, 0.0));
  let offset = p + n * bias.x * (0.25 + sin_l);
  let sp = frame.cascade_view_proj[c] * vec4<f32>(offset, 1.0);
  let uv = vec2<f32>(sp.x * 0.5 + 0.5, 0.5 - sp.y * 0.5);
  if (sp.z > 1.0) { return 1.0; }
  let shadow = pcf(cascade_map, uv, c, sp.z - bias.y);
  let last = frame.cascade_bias[u32(frame.light_dir.w) - 1u].z;
  let depth = dot(p - frame.view_pos.xyz, frame.view_forward.xyz);
  return mix(shadow, 1.0, smoothstep(0.9 * last, last, depth));
}

// A point light's cube face (+x, -x, +y, -y, +z, -z) for a direction from it.
fn cube_face(d: vec3<f32>) -> u32 {
  let a = abs(d);
  if (a.x >= a.y && a.x >= a.z) { return select(1u, 0u, d.x > 0.0); }
  if (a.y >= a.z) { return select(3u, 2u, d.y > 0.0); }
  return select(5u, 4u, d.z > 0.0);
}

// Local shadows store distance / range; the fragment compares its own.
fn local_shadow(light: LocalLight, p: vec3<f32>, n: vec3<f32>) -> f32 {
  let slot = i32(light.direction_slot.w);
  if (slot < 0) { return 1.0; }
  let range = light.position_range.w;
  let to_frag = p - light.position_range.xyz;
  let dist = length(to_frag);
  // Normal offset grows with distance (texels widen with it).
  let offset = p + n * light.cone.z * dist;
  var layer = u32(slot) * 6u;
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

// The toon ramp: N·L (times shadow) to the light received, from the
// gradient map's first row without filtering, or three built-in bands.
fn toon_ramp(x: f32) -> f32 {
  let t = clamp(x, 0.0, 1.0);
  if (material.toon.x < 1.5) {
    let w = i32(textureDimensions(gradient_tex).x);
    let i = clamp(i32(t * f32(w)), 0, w - 1);
    return textureLoad(gradient_tex, vec2<i32>(i, 0), 0).r;
  }
  return select(select(0.22, 0.6, t > 0.08), 1.0, t > 0.45);
}

// Cel-shaded light: the ramp on the diffuse, a hard specular highlight.
fn toon_light(s: Surface, n: vec3<f32>, v: vec3<f32>, l: vec3<f32>, shadow: f32) -> vec3<f32> {
  let n_dot_l = max(dot(n, l), 0.0);
  let band = toon_ramp(n_dot_l * shadow);
  let h = normalize(v + l);
  let shine = exp2(10.0 * (1.0 - s.roughness) + 1.0);
  let spec = smoothstep(0.48, 0.52, pow(max(dot(n, h), 0.0), shine) * shadow);
  let kd = (1.0 - s.metallic) * s.albedo / PI;
  return kd * band + s.f0 * spec * (1.0 - s.roughness);
}

fn local_light(light: LocalLight, s: Surface, p: vec3<f32>, n: vec3<f32>, v: vec3<f32>) -> vec3<f32> {
  let to_light = light.position_range.xyz - p;
  let d2 = max(dot(to_light, to_light), 1e-4);
  let range = light.position_range.w;
  let ratio = d2 / (range * range);
  // Inverse square, windowed to zero at the range (UE4's falloff).
  let window = clamp(1.0 - ratio * ratio, 0.0, 1.0);
  var atten = window * window / d2;
  let l = to_light * inverseSqrt(d2);
  if (light.color_kind.w > 0.5) {
    let cos_angle = dot(-l, light.direction_slot.xyz);
    let t = clamp((cos_angle - light.cone.x) * light.cone.y, 0.0, 1.0);
    atten *= t * t;
  }
  if (atten <= 0.0) { return vec3<f32>(0.0); }
  if (dot(n, l) <= 0.0) { return vec3<f32>(0.0); }
  let shadow = local_shadow(light, p, n);
  if (EFFECTS && material.toon.x > 0.5) {
    return toon_light(s, n, v, l, shadow) * light.color_kind.rgb * atten;
  }
  return brdf(s, n, v, l) * light.color_kind.rgb * atten * shadow;
}

struct DecalResult {
  albedo: vec3<f32>,
  roughness: f32,
  emissive: vec3<f32>,
};

// Projected decals: the boxes covering the point blend their image over
// the surface's albedo (and roughness) before it is lit.
fn apply_decals(p: vec3<f32>, n: vec3<f32>, albedo: vec3<f32>, roughness: f32, dpx: vec3<f32>, dpy: vec3<f32>) -> DecalResult {
  var r: DecalResult;
  r.albedo = albedo;
  r.roughness = roughness;
  r.emissive = vec3<f32>(0.0);
  let count = u32(fx.decal_info.x);
  for (var i = 0u; i < count; i++) {
    let d = fx.decals[i];
    let local = (d.inv * vec4<f32>(p, 1.0)).xyz;
    if (any(abs(local) > vec3<f32>(0.5))) { continue; }
    let facing = dot(n, d.axis.xyz);
    let fade = smoothstep(d.axis.w, d.axis.w + fx.decal_info.z, facing) * (1.0 - smoothstep(0.35, 0.5, abs(local.z)));
    if (fade <= 0.0) { continue; }
    let t = vec2<f32>(local.x + 0.5, 0.5 - local.y);
    let uv = mix(d.uv_rect.xy, d.uv_rect.zw, t);
    var texel = vec4<f32>(1.0);
    if (fx.decal_info.y > 0.5) {
      let span = d.uv_rect.zw - d.uv_rect.xy;
      let gx = (d.inv * vec4<f32>(dpx, 0.0)).xy * vec2<f32>(1.0, -1.0) * span;
      let gy = (d.inv * vec4<f32>(dpy, 0.0)).xy * vec2<f32>(1.0, -1.0) * span;
      texel = textureSampleGrad(decal_atlas, fx_sampler, uv, gx, gy);
    }
    let c = d.color * texel;
    let a = clamp(c.a * fade, 0.0, 1.0);
    r.albedo = mix(r.albedo, c.rgb, a);
    if (d.emissive.w >= 0.0) { r.roughness = mix(r.roughness, d.emissive.w, a); }
    r.emissive += d.emissive.rgb * texel.rgb * a;
  }
  return r;
}

@fragment
fn fs(in: VertexOut) -> @location(0) vec4<f32> {
  let base_sample = textureSample(base_color_tex, material_sampler, in.uv);
  // A material shader's hooks (identities with the built-in hooks).
  var hook: Hook;
  hook.world = in.world_pos;
  hook.local = in.local_pos;
  hook.normal = in.normal;
  hook.uv = in.uv;
  hook.color = in.color;
  hook.time = frame.time.x;
  let params = shader_params();
  let base = surface_color(hook, params, material.base_color * base_sample * in.color);
  var dissolve = vec4<f32>(0.0);
  if (EFFECTS) { dissolve = dissolve_at(in.uv, in.local_pos); }
  // One discard for the alpha cut-off and the dissolve (see dissolve_at).
  if ((material.emissive.w >= 0.0 && base.a < material.emissive.w) || dissolve.w > 0.5) {
    discard;
  }
  let dissolve_edge = dissolve.rgb;
  let mode = u32(material.flags.w);
  // Blended materials keep their alpha; opaque ones write 1.
  let alpha = select(1.0, base.a, (mode & 4u) != 0u);
  if ((mode & 2u) != 0u) {
    // Unlit (MeshBasicMaterial): the colour plus the emissive term.
    let e = surface_emissive(hook, params, material.emissive.rgb * textureSample(emissive_tex, material_sampler, in.uv).rgb);
    var unlit = base.rgb + e + dissolve_edge;
    if (EFFECTS && material.toon.w > 0.0) { unlit = mix(unlit, material.flash.rgb, material.toon.w); }
    return vec4<f32>(apply_fog(unlit, in.world_pos), alpha);
  }
  let metalness_sample = textureSample(metalness_tex, material_sampler, in.uv);
  let roughness_sample = textureSample(roughness_tex, material_sampler, in.uv);
  let dpx = dpdx(in.world_pos);
  let dpy = dpdy(in.world_pos);
  var s: Surface;
  s.metallic = clamp(material.mr.x * metalness_sample.b, 0.0, 1.0);
  s.roughness = clamp(material.mr.y * roughness_sample.g, 0.04, 1.0);
  s.albedo = base.rgb;

  var n = normalize(in.normal);
  if ((mode & 1u) != 0u) {
    // Flat shading: the face normal from the position derivatives.
    n = normalize(cross(dpx, dpy));
  }
  // Perspective: towards the eye; orthographic: against the view direction.
  let v = select(normalize(frame.view_pos.xyz - in.world_pos), -frame.view_forward.xyz, frame.view_pos.w > 0.5);
  let n_vertex = n;
  let face = select(1.0, -1.0, dot(n, v) < 0.0);
  n = n * face;
  var decal_emissive = vec3<f32>(0.0);
  if (EFFECTS && fx.decal_info.x > 0.5) {
    let d = apply_decals(in.world_pos, n, s.albedo, s.roughness, dpx, dpy);
    s.albedo = d.albedo;
    s.roughness = clamp(d.roughness, 0.04, 1.0);
    decal_emissive = d.emissive;
  }
  s.f0 = mix(vec3<f32>(0.04), s.albedo, s.metallic);
  let normal_sample = textureSample(normal_tex, material_sampler, in.uv);
  if (material.flags.x > 0.5) {
    var map = normal_sample.xyz * 2.0 - 1.0;
    if (material.flags.x > 1.5) {
      // Two-channel (BC5, EAC RG11): z from x and y.
      map.z = sqrt(max(1.0 - dot(map.xy, map.xy), 0.0));
    }
    map = vec3<f32>(map.xy * material.mr.zw, map.z);
    // The vertex tangent frame (glTF's: the file's tangents, the cook's
    // MikkTSpace ones, or generated at upload; scene.rs Vertex::tangent).
    var t = in.tangent.xyz - n_vertex * dot(n_vertex, in.tangent.xyz);
    if (dot(t, t) > 1e-12) {
      t = normalize(t);
      let b = cross(n_vertex, t) * select(1.0, -1.0, in.tangent.w < 0.0);
      // A back face flips the whole frame with its normal.
      n = normalize(mat3x3<f32>(t, b, n_vertex) * map) * face;
    }
  }
  n = surface_normal(hook, params, n);
  s.coat = material.coat.x;
  s.coat_roughness = clamp(material.coat.y, 0.04, 1.0);
  s.coat_normal = n_vertex * face;

  // The directional light.
  let l = normalize(frame.light_dir.xyz);
  let n_dot_l = max(dot(n, l), 0.0);
  var shadow = 1.0;
  if (frame.light_dir.w > 0.5 && n_dot_l > 0.0) { shadow = cascade_shadow(in.world_pos, normalize(in.normal), n_dot_l); }
  var color: vec3<f32>;
  if (EFFECTS && material.toon.x > 0.5) {
    color = toon_light(s, n, v, l, shadow) * frame.light_color.rgb;
  } else {
    color = brdf(s, n, v, l) * frame.light_color.rgb * shadow;
  }

  // Point and spot lights: those of this fragment's cluster, in scene order.
  if (frame.params.z > 0.5) {
    let word = cluster_word(cluster_at(in.clip.xy, in.world_pos));
    let first = word & 0xffffffu;
    let count = word >> 24u;
    for (var k = 0u; k < count; k++) {
      color += local_light(light_list[cluster_ref(first + k)], s, in.world_pos, n, v);
    }
  }

  // Image-based light (split sum).
  let n_dot_v = max(dot(n, v), 1e-4);
  let f_ibl = f_schlick_roughness(n_dot_v, s.f0, s.roughness);
  let kd_ibl = (vec3<f32>(1.0) - f_ibl) * (1.0 - s.metallic);
  let irradiance = textureSample(irradiance_map, ibl_sampler, n).rgb;
  let diffuse_ibl = irradiance * s.albedo;
  let r = reflect(-v, n);
  let mip = s.roughness * frame.params.y;
  let prefiltered = textureSampleLevel(prefiltered_map, ibl_sampler, r, mip).rgb;
  let lut = textureSample(brdf_lut, ibl_sampler, vec2<f32>(n_dot_v, s.roughness)).rg;
  let specular_ibl = prefiltered * (f_ibl * lut.x + lut.y);
  let ao_sample = textureSample(ao_tex, material_sampler, in.uv).r;
  var ibl_scale = frame.params.x * material.flags.z;
  if (material.flags.y >= 0.0) {
    // Ambient occlusion darkens only the image-based light.
    ibl_scale *= mix(1.0, ao_sample, material.flags.y);
  }
  if (EFFECTS && fx.ao.x > 0.5) {
    // Screen-space ambient occlusion, likewise.
    let ssao_sample = textureSampleLevel(ssao_tex, fx_sampler, in.clip.xy * fx.screen.zw, 0.0).r;
    ibl_scale *= mix(1.0, ssao_sample, fx.ao.y);
  }
  var ibl = kd_ibl * diffuse_ibl + specular_ibl;
  if (s.coat > 0.0) {
    // The coat's reflection of the environment, and what it takes from the base.
    let c_dot_v = max(dot(s.coat_normal, v), 1e-4);
    let fc = f_schlick(c_dot_v, vec3<f32>(0.04)).x * s.coat;
    let rc = reflect(-v, s.coat_normal);
    let coat_env = textureSampleLevel(prefiltered_map, ibl_sampler, rc, s.coat_roughness * frame.params.y).rgb;
    let coat_lut = textureSample(brdf_lut, ibl_sampler, vec2<f32>(c_dot_v, s.coat_roughness)).rg;
    ibl = ibl * (1.0 - fc) + coat_env * (0.04 * coat_lut.x + coat_lut.y) * s.coat;
  }
  color += ibl * ibl_scale;

  color += surface_emissive(hook, params, material.emissive.rgb * textureSample(emissive_tex, material_sampler, in.uv).rgb);
  if (EFFECTS && fx.decal_info.x > 0.5) { color += decal_emissive; }
  if (EFFECTS && material.rim.w > 0.0) { color += dissolve_edge; }
  if (EFFECTS && material.toon.y > 0.0) {
    // Rim (fresnel) light at grazing angles.
    color += material.rim.rgb * material.toon.y * pow(1.0 - n_dot_v, material.toon.z);
  }
  if (EFFECTS && material.toon.w > 0.0) {
    color = mix(color, material.flash.rgb, material.toon.w);
  }

  if (frame.light_color.w > 1.5) {
    // Debug: the ambient occlusion term.
    let ssao = textureSampleLevel(ssao_tex, fx_sampler, in.clip.xy * fx.screen.zw, 0.0).r;
    return vec4<f32>(vec3<f32>(select(1.0, ssao, fx.ao.x > 0.5)), 1.0);
  }
  if (frame.light_color.w > 0.5) {
    // Debug: the cascade covering this pixel.
    let c = cascade_index(in.world_pos);
    var tint = vec3<f32>(0.0);
    if (c == 0u) { tint = vec3<f32>(1.0, 0.0, 0.0); }
    if (c == 1u) { tint = vec3<f32>(0.0, 1.0, 0.0); }
    if (c == 2u) { tint = vec3<f32>(0.0, 0.0, 1.0); }
    return vec4<f32>(tint * 4.0, 1.0);
  }
  return vec4<f32>(apply_fog(color, in.world_pos), alpha);
}

// Octahedral encoding of a unit vector into -1..1 (two channels).
fn oct_encode(n: vec3<f32>) -> vec2<f32> {
  let p = n.xy / (abs(n.x) + abs(n.y) + abs(n.z));
  if (n.z >= 0.0) { return p; }
  return (1.0 - abs(p.yx)) * select(vec2<f32>(-1.0), vec2<f32>(1.0), p >= vec2<f32>(0.0));
}

struct PrepassTargets {
  // xy: world normal (octahedral), zw: screen-space velocity (uv per frame).
  @location(0) gbuffer: vec4<f32>,
  // View depth along the camera's forward axis (world units).
  @location(1) depth: vec4<f32>,
  // rgb: outline colour, a: outline width / 32 (0: the scene's outline).
  @location(2) outline: vec4<f32>,
};

@fragment
fn fs_prepass(in: PrepassOut) -> PrepassTargets {
  let base_sample = textureSample(base_color_tex, material_sampler, in.uv);
  // A material shader's colour (and alpha cut) as in the lit pass.
  var hook: Hook;
  hook.world = in.world_pos;
  hook.local = in.local_pos;
  hook.normal = in.normal;
  hook.uv = in.uv;
  hook.color = in.color;
  hook.time = frame.time.x;
  let base = surface_color(hook, shader_params(), material.base_color * base_sample * in.color);
  let dissolved = dissolve_at(in.uv, in.local_pos).w > 0.5;
  if ((material.emissive.w >= 0.0 && base.a < material.emissive.w) || dissolved) {
    discard;
  }
  var n = normalize(in.normal);
  if ((u32(material.flags.w) & 1u) != 0u) {
    n = normalize(cross(dpdx(in.world_pos), dpdy(in.world_pos)));
  }
  let v = select(normalize(frame.view_pos.xyz - in.world_pos), -frame.view_forward.xyz, frame.view_pos.w > 0.5);
  if (dot(n, v) < 0.0) { n = -n; }
  let cur = in.cur_clip.xy / in.cur_clip.w;
  let prev = in.prev_clip.xy / max(in.prev_clip.w, 1e-6);
  let velocity = (cur - prev) * vec2<f32>(0.5, -0.5);
  var o: PrepassTargets;
  o.gbuffer = vec4<f32>(oct_encode(n), velocity);
  o.depth = vec4<f32>(dot(in.world_pos - frame.view_pos.xyz, frame.view_forward.xyz), 0.0, 0.0, 1.0);
  o.outline = vec4<f32>(clamp(material.outline.rgb, vec3<f32>(0.0), vec3<f32>(1.0)), material.outline.w / 32.0);
  return o;
}
