// The physical lit shader (three.js MeshPhysicalMaterial, glTF
// KHR_materials_*), after pbr.wgsl in place of pbr_fs.wgsl and
// pbr_prepass.wgsl, with material_physical.wgsl's bindings: the standard
// shading plus clearcoat maps, sheen (Charlie distribution, Neubelt
// visibility), transmission through the copy of the opaque scene,
// iridescence (Belcour and Barla's thin film), anisotropic GGX, light
// that spreads under the surface (subsurface.wgsl), strands of hair
// (hair.wgsl) and an iris under its cornea (eye.wgsl). Every lobe is
// behind a branch on its factor.

// The opaque scene copied after its last opaque draw, with a box-filtered
// mip chain for rough refraction (the effects group of the physical
// pipelines: binding 4 after fx, the AO, the decal atlas and fx_sampler).
@group(2) @binding(4) var transmission_tex: texture_2d<f32>;

struct PhysicalOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world_pos: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
  @location(3) color: vec4<f32>,
  @location(4) local_pos: vec3<f32>,
  @location(5) tangent: vec4<f32>,
  // The model's scale along its axes (the thickness is in object space).
  @location(6) scale: vec3<f32>,
  @location(7) affine: vec3<f32>,
  // The instance's look (pbr.wgsl InstanceFxIn).
  @location(8) @interpolate(flat) look: vec4<f32>,
  // pbr.wgsl VertexOut's.
  @location(9) @interpolate(flat) oit_key: vec2<u32>,
};

fn physical_vertex(model: mat4x4<f32>, v: VertexIn) -> PhysicalOut {
  let o = lit_vertex(model, v);
  var p: PhysicalOut;
  p.clip = o.clip;
  p.world_pos = o.world_pos;
  p.normal = o.normal;
  p.uv = o.uv;
  p.color = o.color;
  p.local_pos = o.local_pos;
  p.tangent = o.tangent;
  p.affine = o.affine;
  p.look = o.look;
  p.oit_key = o.oit_key;
  p.scale = vec3<f32>(length(model[0].xyz), length(model[1].xyz), length(model[2].xyz));
  return p;
}

@vertex
fn vs_physical(v: VertexIn, i: InstanceIn, k: InstanceFxIn) -> PhysicalOut {
  var p = physical_vertex(instance_matrix(i.m0, i.m1, i.m2, i.m3), v);
  p.look = k.look;
  return p;
}

@vertex
fn vs_physical_skinned(@builtin(vertex_index) vi: u32, v: VertexIn, i: SkinnedInstanceIn, s: SkinIn) -> PhysicalOut {
  let model = instance_matrix(i.m0, i.m1, i.m2, i.m3) * skin_matrix(s, i.palette);
  var p = physical_vertex(model, morph_vertex(v, vi, i));
  p.look = i.look;
  return p;
}

// The lobes beyond Surface's, per fragment.
struct Lobes {
  sheen_color: vec3<f32>,     // 0: no sheen
  sheen_roughness: f32,
  sheen_albedo: f32,          // E(n.v) of the sheen (sheen_albedo)
  iridescence: f32,
  iridescence_fresnel: vec3<f32>,
  anisotropy: f32,            // 0: isotropic
  aniso_t: vec3<f32>,         // the stretch's direction on the surface
  aniso_b: vec3<f32>,
  alpha_t: f32,               // GGX alpha along aniso_t (roughness² across it)
  // Strands of hair along the surface (hair.wgsl).
  hair: f32,                  // how much of the surface is hair (0: none)
  strand: vec3<f32>,          // the strands' direction, towards their roots
  // Light that spreads under the surface (subsurface.wgsl).
  subsurface: f32,            // how much of the diffuse light does (0: none)
  scatter: vec3<f32>,         // its scatter distance per channel (m)
  curvature: f32,             // the surface's curvature here (1 / m)
  through: vec3<f32>,         // what comes through from the far side (0: nothing: no thickness)
  thickness: f32,             // how far off the far side is (m): 0 lets nothing through
};

// How far towards a light a subsurface material's shadow is read: light
// entering within the widest lobe of a point reaches it, so a point just
// past the terminator is not shadowed by its own surface.
fn subsurface_reach(k: Lobes) -> f32 {
  return select(0.0, 3.0 * max(k.scatter.r, max(k.scatter.g, k.scatter.b)), k.subsurface > 0.0);
}

// What comes through a subsurface material from a light behind it, of
// that light's radiance (its shadow read where it enters, by the caller).
fn subsurface_transmitted(s: Surface, k: Lobes, n: vec3<f32>, l: vec3<f32>) -> vec3<f32> {
  return k.subsurface * (1.0 - s.metallic) * s.albedo / PI * k.through * subsurface_back(n, l);
}

// What one light gives: the diffuse part (which transmission replaces),
// the rest, and what comes through the surface from a light behind it,
// which is shadowed where that light enters: `enters` towards the light
// (m), on the far side. (A closed caster's far side is what the shadow
// maps hold, lighting.wgsl `shadow_closed`: a body never shadows what
// comes through itself, so each lobe says how much does.)
struct Shade {
  diffuse: vec3<f32>,
  specular: vec3<f32>,
  through: vec3<f32>,
  enters: f32,
};

fn max3(c: vec3<f32>) -> f32 {
  return max(c.x, max(c.y, c.z));
}

fn srgb_decode(c: vec3<f32>) -> vec3<f32> {
  return select(pow((c + 0.055) / 1.055, vec3<f32>(2.4)), c / 12.92, c <= vec3<f32>(0.04045));
}

// The sheen's directional albedo E(cos, roughness) (sheen.wgsl), from the
// BRDF table's blue channel (ibl.wgsl fs_brdf_lut).
fn sheen_albedo(cos_theta: f32, roughness: f32) -> f32 {
  return textureSampleLevel(brdf_lut, ibl_sampler, vec2<f32>(max(cos_theta, 1e-3), roughness), 0.0).b;
}

// How much of the layers under the sheen one light reaches: what the
// fibres reflect towards the eye or from the light is taken from them,
// the lesser share kept (Imageworks' min{1 - E(o), 1 - E(i)}, glTF's
// sheen albedo scaling), so a white sheen over white cloth never shines
// brighter than white.
fn sheen_scale(k: Lobes, n_dot_l: f32) -> f32 {
  let c = max3(k.sheen_color);
  return min(1.0 - c * k.sheen_albedo, 1.0 - c * sheen_albedo(n_dot_l, k.sheen_roughness));
}

// Anisotropic GGX (as three.js and Filament): alpha_t along t, alpha_b across.
fn d_ggx_aniso(alpha_t: f32, alpha_b: f32, n_dot_h: f32, t_dot_h: f32, b_dot_h: f32) -> f32 {
  let a2 = alpha_t * alpha_b;
  let w = vec3<f32>(alpha_b * t_dot_h, alpha_t * b_dot_h, a2 * n_dot_h);
  let w2 = a2 / dot(w, w);
  return a2 * w2 * w2 / PI;
}

// With alpha_t = alpha_b it is ggx.wgsl's v_ggx, and like it is not held
// under 1 (three.js saturates it): the visibility passes 1 at grazing
// angles, where a cap would leave the lobe darker than the isotropic one
// it tends to as the anisotropy goes to 0.
fn v_ggx_aniso(alpha_t: f32, alpha_b: f32, tv: f32, bv: f32, tl: f32, bl: f32, n_dot_v: f32, n_dot_l: f32) -> f32 {
  // (Not `gl`: GLSL reserves the gl_ prefix naga's renaming would give it.)
  let lambda_v = n_dot_l * length(vec3<f32>(alpha_t * tv, alpha_b * bv, n_dot_v));
  let lambda_l = n_dot_v * length(vec3<f32>(alpha_t * tl, alpha_b * bl, n_dot_l));
  return 0.5 / max(lambda_v + lambda_l, 1e-6);
}

// Thin-film iridescence (Belcour and Barla 2017, as three.js): the
// Fresnel of a film of index `film_ior` and `thickness` nm over a base of
// reflectance `base_f0`, seen at `cos1`.
fn ior_to_f0(t: f32, i: f32) -> f32 {
  let r = (t - i) / (t + i);
  return r * r;
}

fn eval_sensitivity(opd: f32, shift: vec3<f32>) -> vec3<f32> {
  let phase = 2.0 * PI * opd * 1.0e-9;
  let val = vec3<f32>(5.4856e-13, 4.4201e-13, 5.2481e-13);
  let pos = vec3<f32>(1.6810e+06, 1.7953e+06, 2.2084e+06);
  let spread = vec3<f32>(4.3278e+09, 9.3046e+09, 6.6121e+09);
  var xyz = val * sqrt(2.0 * PI * spread) * cos(pos * phase + shift) * exp(-phase * phase * spread);
  xyz.x += 9.7470e-14 * sqrt(2.0 * PI * 4.5282e+09) * cos(2.2399e+06 * phase + shift.x) * exp(-4.5282e+09 * phase * phase);
  xyz /= 1.0685e-7;
  let to_rgb = mat3x3<f32>(
    vec3<f32>(3.2404542, -0.9692660, 0.0556434),
    vec3<f32>(-1.5371385, 1.8760108, -0.2040259),
    vec3<f32>(-0.4985314, 0.0415560, 1.0572252),
  );
  return to_rgb * xyz;
}

fn eval_iridescence(film_ior_in: f32, cos1: f32, thickness: f32, base_f0: vec3<f32>) -> vec3<f32> {
  let film_ior = mix(1.0, film_ior_in, smoothstep(0.0, 0.03, thickness));
  let sin2_sq = (1.0 / film_ior) * (1.0 / film_ior) * (1.0 - cos1 * cos1);
  let cos2_sq = 1.0 - sin2_sq;
  if (cos2_sq < 0.0) { return vec3<f32>(1.0); }
  let cos2 = sqrt(cos2_sq);
  let r0 = ior_to_f0(film_ior, 1.0);
  let r12 = r0 + (1.0 - r0) * pow(1.0 - cos1, 5.0);
  let t121 = 1.0 - r12;
  let phi12 = select(0.0, PI, film_ior < 1.0);
  let phi21 = PI - phi12;
  let sqrt_f0 = sqrt(clamp(base_f0, vec3<f32>(0.0), vec3<f32>(0.9999)));
  let base_ior = (vec3<f32>(1.0) + sqrt_f0) / (vec3<f32>(1.0) - sqrt_f0);
  let r1 = (base_ior - film_ior) / (base_ior + film_ior);
  let r1_sq = r1 * r1;
  let r23 = r1_sq + (vec3<f32>(1.0) - r1_sq) * pow(1.0 - cos2, 5.0);
  let phi23 = select(vec3<f32>(0.0), vec3<f32>(PI), base_ior < vec3<f32>(film_ior));
  let opd = 2.0 * film_ior * thickness * cos2;
  let phi = vec3<f32>(phi21) + phi23;
  let r123 = clamp(r12 * r23, vec3<f32>(1e-5), vec3<f32>(0.9999));
  let rr = sqrt(r123);
  let rs = t121 * t121 * r23 / (vec3<f32>(1.0) - r123);
  var i = r12 + rs;
  var cm = rs - t121;
  for (var m = 1; m <= 2; m++) {
    cm *= rr;
    i += cm * 2.0 * eval_sensitivity(f32(m) * opd, f32(m) * phi);
  }
  return max(i, vec3<f32>(0.0));
}

// The F0 a Schlick Fresnel would need to give `f` at `cos_theta`.
fn schlick_to_f0(f: vec3<f32>, cos_theta: f32) -> vec3<f32> {
  let x = clamp(1.0 - cos_theta, 0.0, 1.0);
  let x5 = clamp(x * x * x * x * x, 0.0, 0.9999);
  return (f - vec3<f32>(x5)) / (1.0 - x5);
}

// One light's shading: the GGX lobe (anisotropic, with the film's Fresnel)
// and the diffuse base, or hair's lobes in their place, the sheen over
// them and the coat over all.
fn physical_brdf(s: Surface, k: Lobes, n: vec3<f32>, v: vec3<f32>, l: vec3<f32>) -> Shade {
  let h = normalize(v + l);
  let n_dot_l = max(dot(n, l), 0.0);
  let n_dot_v = clamp(dot(n, v), 1e-4, 1.0);
  let n_dot_h = max(dot(n, h), 0.0);
  let h_dot_v = max(dot(h, v), 0.0);
  var f = f_schlick(h_dot_v, s.f0);
  if (k.iridescence > 0.0) { f = mix(f, k.iridescence_fresnel, k.iridescence); }
  var spec: vec3<f32>;
  if (k.anisotropy > 0.0) {
    let ab = s.roughness * s.roughness;
    let d = d_ggx_aniso(k.alpha_t, ab, n_dot_h, dot(k.aniso_t, h), dot(k.aniso_b, h));
    let vis = v_ggx_aniso(k.alpha_t, ab, dot(k.aniso_t, v), dot(k.aniso_b, v), dot(k.aniso_t, l), dot(k.aniso_b, l), n_dot_v, n_dot_l);
    spec = f * d * vis;
  } else {
    let a = s.roughness * s.roughness;
    spec = f * d_ggx(n_dot_h, a) * v_ggx(n_dot_v, n_dot_l, a);
  }
  var o: Shade;
  // The light's falloff: the cosine, spread past the terminator where the
  // light scatters under the surface.
  var falloff = vec3<f32>(n_dot_l);
  if (k.subsurface > 0.0) {
    falloff = mix(falloff, subsurface_cosine(dot(n, l), k.curvature, k.scatter), k.subsurface);
  }
  o.diffuse = (vec3<f32>(1.0) - f) * (1.0 - s.metallic) * s.albedo / PI * falloff + leaf_transmission(s, n, v, l);
  o.specular = spec * n_dot_l;
  o.through = vec3<f32>(0.0);
  o.enters = k.thickness + subsurface_reach(k);
  if (k.subsurface > 0.0 && k.thickness > 0.0) { o.through = subsurface_transmitted(s, k, n, l); }
  if (k.hair > 0.0) {
    let path = hair_path(k.thickness, k.curvature, shadow_closed, n, l);
    let hair = hair_lobes(s.albedo, s.roughness, k.strand, v, l, path.x);
    // The hair behind a strand shades it: its light fades past the
    // mesh's terminator.
    let lit = smoothstep(-0.4, 0.4, dot(n, l));
    o.diffuse = mix(o.diffuse, hair.scatter * lit, k.hair);
    o.specular = mix(o.specular, hair.front * lit, k.hair);
    if (k.thickness > 0.0) {
      o.through = mix(o.through, hair.through * path.y, k.hair);
      o.enters = max(o.enters, path.x);
    }
  }
  if (any(k.sheen_color > vec3<f32>(0.0))) {
    // The sheen layer takes its albedo from what it covers.
    let scale = sheen_scale(k, n_dot_l);
    let alpha = k.sheen_roughness * k.sheen_roughness;
    o.diffuse *= scale;
    o.specular = o.specular * scale + k.sheen_color * d_charlie(alpha, n_dot_h) * v_charlie(alpha, n_dot_v, n_dot_l) * n_dot_l;
  }
  if (s.coat > 0.0) {
    // The coat, as lighting.wgsl's brdf: what it reflects never reaches the base.
    let nc = s.coat_normal;
    let c_dot_l = max(dot(nc, l), 0.0);
    let ac = s.coat_roughness * s.coat_roughness;
    let fc = f_schlick(h_dot_v, vec3<f32>(0.04)).x * s.coat;
    let dc = d_ggx(max(dot(nc, h), 0.0), ac);
    let vc = 0.25 / max(h_dot_v * h_dot_v, 1e-4);
    o.diffuse *= 1.0 - fc;
    o.specular = o.specular * (1.0 - fc) + vec3<f32>(dc * vc * fc * c_dot_l);
  }
  return o;
}

// A point or spot light, as pbr.wgsl's local_light.
fn physical_local(light: LocalLight, s: Surface, k: Lobes, p: vec3<f32>, n: vec3<f32>, v: vec3<f32>, receives: bool) -> Shade {
  var o: Shade;
  o.diffuse = vec3<f32>(0.0);
  o.specular = vec3<f32>(0.0);
  if (light.color_kind.w < -0.5) {
    // A rect light (area.wgsl) on the base lobes.
    let a = area_shade(light, s, p, n, v);
    let shadow = select(1.0, rect_shadow(light, p, leaf_shadow_normal(s, n, light.position_range.xyz - p)), receives);
    o.diffuse = a.diffuse * shadow;
    o.specular = a.specular * shadow;
    return o;
  }
  let r = local_reach(light, max(s.translucency, max(k.subsurface, k.hair)), p, n);
  if (!r.lit) { return o; }
  var radiance = r.radiance;
  if (receives) {
    radiance *= local_shadow(light, p + r.l * subsurface_reach(k), leaf_shadow_normal(s, n, r.l));
    // The screen's contact shadow is the surface's own past the terminator.
    if (max(k.subsurface, k.hair) <= 0.0 || dot(n, r.l) > 0.0) { radiance *= contact_shadow(light, p); }
  }
  let b = physical_brdf(s, k, n, v, r.l);
  o.diffuse = b.diffuse * radiance;
  o.specular = b.specular * radiance;
  if (any(b.through > vec3<f32>(0.0))) {
    // The light that comes through from behind: shadowed where it enters.
    var open = 1.0;
    if (receives) { open = local_shadow(light, p + r.l * b.enters, r.l); }
    o.diffuse += b.through * r.radiance * open;
  }
  return o;
}

// Sigg and Hadwiger's bicubic B-spline filter in four bilinear taps (as
// three.js), on one level of the scene copy.
fn bicubic_level(uv: vec2<f32>, level: f32) -> vec3<f32> {
  let size = vec2<f32>(textureDimensions(transmission_tex, i32(level)));
  let p = uv * size + 0.5;
  let i = floor(p);
  let f = fract(p);
  let w0 = (f * (f * (-f + 3.0) - 3.0) + 1.0) / 6.0;
  let w1 = (f * f * (3.0 * f - 6.0) + 4.0) / 6.0;
  let w2 = (f * (f * (-3.0 * f + 3.0) + 3.0) + 1.0) / 6.0;
  let w3 = f * f * f / 6.0;
  let g0 = w0 + w1;
  let g1 = w2 + w3;
  let h0 = (i - 1.0 + w1 / g0 - 0.5) / size;
  let h1 = (i + 1.0 + w3 / g1 - 0.5) / size;
  let a = textureSampleLevel(transmission_tex, fx_sampler, vec2<f32>(h0.x, h0.y), level).rgb;
  let b = textureSampleLevel(transmission_tex, fx_sampler, vec2<f32>(h1.x, h0.y), level).rgb;
  let c = textureSampleLevel(transmission_tex, fx_sampler, vec2<f32>(h0.x, h1.y), level).rgb;
  let d = textureSampleLevel(transmission_tex, fx_sampler, vec2<f32>(h1.x, h1.y), level).rgb;
  return g0.y * (g0.x * a + g1.x * b) + g1.y * (g0.x * c + g1.x * d);
}

// The opaque scene behind the point, blurred by `lod` levels (the copy
// has a full mip chain).
fn scene_behind(uv: vec2<f32>, lod: f32) -> vec3<f32> {
  let size = vec2<f32>(textureDimensions(transmission_tex));
  let top = floor(log2(max(size.x, size.y)));
  let lo = min(floor(lod), top);
  let hi = min(lo + 1.0, top);
  return mix(bicubic_level(uv, lo), bicubic_level(uv, hi), fract(lod));
}

@fragment
fn fs(in: PhysicalOut, @builtin(front_facing) front_facing: bool) -> @location(0) vec4<f32> {
  hook_front = select(-1.0, 1.0, front_facing);
  return physical_fragment(in);
}

// The order-independent passes' entries (pbr_fs.wgsl `fs_oit`).
@fragment
fn fs_oit(in: PhysicalOut, @builtin(front_facing) front_facing: bool) -> OitOutput {
  hook_front = select(-1.0, 1.0, front_facing);
  oit_pass = true;
  return oit_weighted(physical_fragment(in), in.clip.z);
}

@fragment
fn fs_oit_list(in: PhysicalOut, @builtin(front_facing) front_facing: bool, @builtin(sample_mask) mask: u32) -> OitOutput {
  hook_front = select(-1.0, 1.0, front_facing);
  oit_pass = true;
  let c = physical_fragment(in);
  oit_store(in.clip, c, mask, in.oit_key);
  return oit_weighted(c, in.clip.z);
}

@fragment
fn fs_oit_front(in: PhysicalOut, @builtin(front_facing) front_facing: bool) -> OitOutput {
  hook_front = select(-1.0, 1.0, front_facing);
  oit_pass = true;
  let c = physical_fragment(in);
  return oit_hybrid(c, in.clip);
}

@fragment
fn fs_oit_mark(in: PhysicalOut, @builtin(front_facing) front_facing: bool) {
  hook_front = select(-1.0, 1.0, front_facing);
  oit_pass = true;
  let c = physical_fragment(in);
  oit_front_mark(in.clip, c.a);
}

// The physical shading of a fragment (`fs`, and lightmap_physical.wgsl's
// `fs_physical_lightmap`).
fn physical_fragment(in: PhysicalOut) -> vec4<f32> {
  // The surface's curvature (1 / m): how fast its normal turns across it
  // (Penner and Borshukov's), from the vertex normal, which no map bends.
  let curvature = length(fwidth(normalize(in.normal))) / max(length(fwidth(in.world_pos)), 1e-6);
  let uv = stylised_uv(in.uv, in.affine);
  let mode = u32(material.flags.w);
  let dpx = dpdxFine(in.world_pos);
  let dpy = dpdyFine(in.world_pos);
  var n = normalize(in.normal);
  if ((mode & 1u) != 0u) {
    n = normalize(cross(dpx, dpy));
  }
  let v = select(normalize(frame.view_pos.xyz - in.world_pos), -frame.view_forward.xyz, frame.view_pos.w > 0.5);
  let n_vertex = n;
  let face = select(1.0, -1.0, dot(n, v) < 0.0);
  n = n * face;
  // The vertex tangent frame (pbr_fs.wgsl's), for the normal maps, the stretch and the iris.
  var t = in.tangent.xyz - n_vertex * dot(n_vertex, in.tangent.xyz);
  let framed = dot(t, t) > 1e-12;
  t = select(normalize(cross(select(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), abs(n_vertex.y) > 0.9), n_vertex)), normalize(t), framed);
  let bt = cross(n_vertex, t) * select(1.0, -1.0, in.tangent.w < 0.0);
  let tbn = mat3x3<f32>(t, bt, n_vertex);
  // An eye: the base colour inside the iris lies under the cornea (eye.wgsl).
  var base_at = uv;
  if (physical.layers2.y > 0.0) {
    base_at = iris_uv(uv, t, bt, n_vertex * face, v, physical.transmission.z, physical.layers2.y, physical.layers2.z);
  }
  let base_uv = material_uv(UV_MAP, base_at);
  let base_sample = textureSample(base_color_tex, material_sampler, base_uv);
  var hook: Hook;
  hook.world = in.world_pos;
  hook.local = in.local_pos;
  hook.normal = in.normal;
  hook.uv = uv;
  hook.color = in.color;
  hook.time = frame.time.x;
  hook.origin = frame.origin.xyz;
  hook.eye = frame.view_pos.xyz;
  hook.front = hook_front;
  hook.wind = frame.foliage_wind[0][0];
  let params = shader_params();
  let base = surface_color(hook, params, look_tint(material.base_color * base_sample * in.color, in.look));
  var dissolve = vec4<f32>(0.0);
  if (EFFECTS) { dissolve = dissolve_at(uv, in.local_pos); }
  // A closed, single-sided surface (mode 32): its shadows compare past its depth (lighting.wgsl shadow_closed).
  shadow_closed = (mode & 32u) != 0u;
  // Alpha to coverage (mode 16): as pbr_fs.wgsl.
  let a2c = (mode & 16u) != 0u && material.emissive.w >= 0.0;
  let coverage = clamp((base.a - material.emissive.w) / max(fwidthFine(base.a), 1e-4) + 0.5, 0.0, 1.0);
  if ((material.emissive.w >= 0.0 && select(base.a < material.emissive.w, coverage <= 0.0, a2c)) || dissolve.w > 0.5) {
    discard;
  }
  let alpha = select(select(1.0, coverage, a2c), base.a, (mode & 4u) != 0u);
  // The packed maps (physical.rs): white where a material has none; all
  // at the metalness map's uv transform.
  let pack_uv = material_uv(UV_METALNESS, uv);
  let maps = pack_sample(physical.layers.x, pack_uv);      // metalness, roughness, ao, dissolve
  let coat_maps = pack_sample(physical.layers.y, pack_uv); // clearcoat, its roughness, transmission, thickness
  let sheen_maps = pack_sample(physical.layers.z, pack_uv); // sheen colour (sRGB), its roughness
  let film_maps = pack_sample(physical.layers.w, pack_uv);  // the coat's normal, iridescence
  let aniso_maps = pack_sample(physical.layers2.x, pack_uv); // anisotropy, the film's thickness
  var s: Surface;
  s.metallic = clamp(material.mr.x * maps.r, 0.0, 1.0);
  s.roughness = clamp(material.mr.y * maps.g, 0.04, 1.0);
  s.albedo = base.rgb;

  var decal_emissive = vec3<f32>(0.0);
  var decal_normal = vec4<f32>(0.0);
  if (EFFECTS && abs(fx.decal_info.x) > 0.5) {
    let d = apply_decals(in.clip.xy, in.world_pos, n, s.albedo, s.roughness, dpx, dpy);
    s.albedo = d.albedo;
    s.roughness = clamp(d.roughness, 0.04, 1.0);
    decal_emissive = d.emissive;
    decal_normal = d.normal;
  }
  // The dielectric reflectance from the index of refraction.
  let ior = physical.transmission.z;
  let f0 = ior_to_f0(ior, 1.0);
  s.f0 = mix(vec3<f32>(f0), s.albedo, s.metallic);
  let normal_uv = material_uv(UV_NORMAL, uv);
  let normal_sample = textureSample(normal_tex, material_sampler, normal_uv);
  if (material.flags.x > 0.5) {
    var map = normal_sample.xyz * 2.0 - 1.0;
    if (material.flags.x > 1.5) {
      map.z = sqrt(max(1.0 - dot(map.xy, map.xy), 0.0));
    }
    map = vec3<f32>(map.xy * material.mr.zw, map.z);
    if (framed) { n = normalize(tbn * map) * face; }
  }
  n = surface_normal(hook, params, n);
  // The decals' normal maps over the surface's own (apply_decals).
  if (decal_normal.w > 0.0) { n = normalize(mix(n, decal_normal.xyz, decal_normal.w)); }
  let weathered = weather_surface_at(s, n, normalize(n_vertex * face), in.world_pos, select(0.2 + 0.6 * maps.b, 0.5, material.flags.y < 0.0), material.uv[3].w, WEATHER_STANDS);
  s = weathered.s;
  n = weathered.n;
  s.coat = material.coat.x * coat_maps.r;
  s.coat_roughness = clamp(material.coat.y * coat_maps.g, 0.04, 1.0);
  s.coat_normal = n_vertex * face;
  s.translucency = material.coat.z;
  if (physical.coat.z > 0.5 && framed) {
    var map = film_maps.xyz * 2.0 - 1.0;
    map = vec3<f32>(map.xy * physical.coat.xy, map.z);
    s.coat_normal = normalize(tbn * map) * face;
  }
  let n_dot_v = clamp(dot(n, v), 1e-4, 1.0);

  var k: Lobes;
  k.sheen_color = physical.sheen.rgb * srgb_decode(sheen_maps.rgb);
  k.sheen_roughness = clamp(physical.sheen.w * sheen_maps.a, 0.07, 1.0);
  k.sheen_albedo = sheen_albedo(n_dot_v, k.sheen_roughness);
  k.iridescence = physical.iridescence.x * film_maps.a;
  k.iridescence_fresnel = vec3<f32>(0.0);
  var f0_ibl = s.f0;
  if (physical.iridescence.x > 0.0) {
    let thickness = mix(physical.iridescence.z, physical.iridescence.w, aniso_maps.a);
    k.iridescence_fresnel = eval_iridescence(physical.iridescence.y, n_dot_v, thickness, s.f0);
    f0_ibl = mix(s.f0, schlick_to_f0(k.iridescence_fresnel, n_dot_v), k.iridescence);
  }
  k.anisotropy = 0.0;
  k.alpha_t = s.roughness * s.roughness;
  k.aniso_t = t;
  k.aniso_b = bt;
  k.hair = physical.anisotropy.w;
  k.strand = bt;
  if (physical.anisotropy.x > 0.0 || k.hair > 0.0) {
    var dir = physical.anisotropy.yz;
    var strength = physical.anisotropy.x;
    if (physical.coat.w > 0.5) {
      // The map's direction in the tangent frame, turned by the rotation.
      let m = aniso_maps.rgb;
      let d = m.rg * 2.0 - 1.0;
      let dn = select(vec2<f32>(1.0, 0.0), normalize(d), dot(d, d) > 1e-8);
      dir = vec2<f32>(dir.x * dn.x - dir.y * dn.y, dir.y * dn.x + dir.x * dn.y);
      strength *= m.b;
    }
    k.anisotropy = clamp(strength, 0.0, 1.0);
    let ta = normalize(t * dir.x + bt * dir.y);
    k.aniso_t = normalize(ta - n * dot(n, ta));
    k.aniso_b = cross(n, k.aniso_t);
    k.alpha_t = mix(s.roughness * s.roughness, 1.0, k.anisotropy * k.anisotropy);
    // The strands run across the stretch, from root to tip the way v
    // does: their roots up the image, where the bitangent points.
    let root = bt * dir.x - t * dir.y;
    k.strand = normalize(root - n * dot(n, root));
  }

  // Light that spreads under the surface (subsurface.wgsl).
  k.subsurface = physical.subsurface.w;
  k.scatter = physical.subsurface.rgb;
  k.curvature = curvature;
  // (`thickness` is in the mesh's own units: by its scale, the mean of the three.)
  k.thickness = select(0.0, physical.transmission.y * coat_maps.a * dot(in.scale, vec3<f32>(1.0 / 3.0)), k.subsurface > 0.0 || k.hair > 0.0);
  k.through = select(vec3<f32>(0.0), subsurface_through(k.thickness, k.scatter), k.thickness > 0.0);

  // The directional light.
  let l = normalize(frame.light_dir.xyz);
  let n_dot_l = max(dot(n, l), 0.0);
  let receives = look_receives(in.look);
  let shadowed = frame.light_dir.w > 0.5 && receives;
  var shadow = 1.0;
  if (shadowed && (n_dot_l > 0.0 || s.translucency > 0.0 || k.subsurface > 0.0 || k.hair > 0.0)) {
    shadow = cascade_shadow(in.world_pos + l * subsurface_reach(k), leaf_shadow_normal(s, normalize(in.normal), l), abs(dot(n, l)));
  }
  let sun = physical_brdf(s, k, n, v, l);
  var diffuse = sun.diffuse * frame.light_color.rgb * shadow;
  if (any(sun.through > vec3<f32>(0.0))) {
    // The sun through it from behind: shadowed where it enters.
    var open = 1.0;
    if (shadowed) { open = cascade_shadow(in.world_pos + l * sun.enters, l, 1.0); }
    diffuse += sun.through * frame.light_color.rgb * open;
  }
  var specular = sun.specular * frame.light_color.rgb * shadow;
  diffuse += s.albedo * frame.light_color.rgb * n_dot_l * shadow * caustics(in.world_pos);
  // The material style's shadow darkness (pbr.wgsl shadow_keep).
  let keep = shadow_keep(n_dot_l * shadow, mode);
  diffuse *= keep;
  specular *= keep;
  if (frame.shadow_tint.w > 0.0 && n_dot_l > 0.0 && (mode & 8u) == 0u) {
    diffuse += (1.0 - shadow) * frame.shadow_tint.rgb * frame.shadow_tint.w * s.albedo * frame.light_color.rgb * n_dot_l;
  }

  // The cluster's lights, or every light in a reflection probe's capture
  // (probes.rs), which has no clusters (pbr_fs.wgsl).
  let lights = frame_lights(in.clip.xy, in.world_pos);
  for (var c = 0u; c < lights.y; c++) {
    let o = physical_local(frame_light(lights, c), s, k, in.world_pos, n, v, receives);
    diffuse += o.diffuse;
    specular += o.specular;
  }

  // Image-based light (split sum): an anisotropic surface reflects along
  // its bent normal (as three.js).
  let f_ibl = f_schlick_roughness(n_dot_v, f0_ibl, s.roughness);
  let kd_ibl = (vec3<f32>(1.0) - f_ibl) * (1.0 - s.metallic);
  // The light around it (surface_shade.wgsl): every lit surface's one
  // gather, around the bent normal. Screen-space reflections only in the
  // opaque main pass (a blended surface lies over the one they were
  // traced for).
  var bent = n;
  if (physical.anisotropy.x > 0.0) {
    let across = normalize(cross(cross(k.aniso_b, v), k.aniso_b));
    let w = 1.0 - k.anisotropy * (1.0 - s.roughness);
    bent = normalize(mix(across, n, w * w * w * w));
  }
  let at = ShadeAt(in.world_pos, n, v, in.clip.xy, EFFECTS, !oit_pass && (u32(material.flags.w) & 4u) == 0u, false);
  let around = surface_gather(at, bent, s.roughness);
  let irradiance = around.irradiance;
  let prefiltered = around.prefiltered;
  let env_brdf = f0_ibl * around.lut.x + around.lut.y;  // Karis split sum
  // What the material's AO map and the style's shadow darkness keep of
  // the light around it, then the screen's occlusion; `ao_direct` also
  // scales the emitted light.
  var kept = keep;
  if (material.flags.y >= 0.0) {
    kept *= mix(1.0, maps.b, material.flags.y);
  }
  let ibl_scale = material.flags.z * kept * around.occlusion.ambient;
  let bounce = around.occlusion.bounce * kept;
  let ao_direct = around.occlusion.all;
  diffuse *= ao_direct;
  specular *= ao_direct;
  // The coat's reflection of the environment, and what it takes from
  // everything under it (the lights' share is taken per light).
  var coat_keep = 1.0;
  var coat_ibl = vec3<f32>(0.0);
  if (material.coat.x > 0.0) {
    let c_dot_v = max(dot(s.coat_normal, v), 1e-4);
    let fc = f_schlick(c_dot_v, vec3<f32>(0.04)).x * s.coat;
    let rc = reflect(-v, s.coat_normal);
    let coat_env = reflected_light(rc, in.world_pos, s.coat_roughness * frame.params.y);
    let coat_lut = textureSampleLevel(brdf_lut, ibl_sampler, vec2<f32>(c_dot_v, s.coat_roughness), 0.0).rg;
    coat_keep = 1.0 - fc;
    coat_ibl = coat_env * (0.04 * coat_lut.x + coat_lut.y) * s.coat * frame.params.x;
  }
  var ibl_diffuse = kd_ibl * (irradiance * ibl_scale + bounce) * s.albedo;
  var ibl_specular = prefiltered * env_brdf;
  // The sheen over the base (the lights' share is in physical_brdf).
  var sheen_keep = 1.0;
  if (any(k.sheen_color > vec3<f32>(0.0))) {
    // The fibres scatter back and to the sides, never along the mirror
    // direction, so they see the environment's irradiance (the specular
    // reflection's prefiltered light drew a blurred sun on cotton), times
    // their directional albedo E.
    sheen_keep = 1.0 - max3(k.sheen_color) * k.sheen_albedo;
    ibl_diffuse *= sheen_keep;
    ibl_specular = ibl_specular * sheen_keep + irradiance * k.sheen_color * k.sheen_albedo;
  }
  diffuse += ibl_diffuse * coat_keep;
  specular += (ibl_specular * coat_keep + coat_ibl) * ibl_scale;

  // Transmission: the opaque scene behind, refracted through the volume
  // (`thickness` along the refracted ray, in object space), blurred by
  // the roughness and tinted by the base and the volume's absorption,
  // takes the diffuse light's place.
  if (physical.transmission.x > 0.0) {
    let amount = physical.transmission.x * coat_maps.b;
    let thickness = physical.transmission.y * coat_maps.a;
    let ray = normalize(refract(-v, n, 1.0 / ior)) * thickness * in.scale;
    let exit = frame.view_proj * vec4<f32>(in.world_pos + ray, 1.0);
    let uv = clamp(vec2<f32>(exit.x, -exit.y) / exit.w * 0.5 + 0.5, vec2<f32>(0.0), vec2<f32>(1.0));
    let width = f32(textureDimensions(transmission_tex).x);
    let lod = log2(width) * s.roughness * clamp(ior * 2.0 - 2.0, 0.0, 1.0);
    var absorbed = vec3<f32>(1.0);
    if (physical.transmission.w > 0.0) {
      let c = max(physical.attenuation.rgb, vec3<f32>(1e-4));
      absorbed = pow(c, vec3<f32>(length(ray) / physical.transmission.w));
    }
    let behind = scene_behind(uv, lod);
    let transmitted = (vec3<f32>(1.0) - env_brdf) * s.albedo * (1.0 - s.metallic) * absorbed * behind;
    diffuse = mix(diffuse, transmitted * sheen_keep * coat_keep, amount);
  }
  var color = diffuse + specular;

  color += surface_emissive(hook, params, material.emissive.rgb * textureSample(emissive_tex, material_sampler, material_uv(UV_EMISSIVE, uv)).rgb) * ao_direct;
  if (EFFECTS && abs(fx.decal_info.x) > 0.5) { color += decal_emissive; }
  if (EFFECTS && material.rim.w > 0.0) { color += dissolve.rgb; }
  if (EFFECTS && material.toon.y > 0.0) {
    color += material.rim.rgb * material.toon.y * pow(1.0 - n_dot_v, material.toon.z);
  }
  if (EFFECTS && material.toon.w > 0.0) {
    color = mix(color, material.flash.rgb, material.toon.w);
  }
  color = look_flash(color, in.look);
  if (frame.light_color.w > 2.5 && frame.light_color.w < 3.5) { return vec4<f32>(s.albedo, 1.0); }
  if (frame.light_color.w > 3.5) { return debug_unsupported(in.clip.xy); }
  if (frame.light_color.w > 1.5) {
    let ssao = textureSampleLevel(ssao_tex, fx_sampler, in.clip.xy * fx.screen.zw, 0.0).r;
    return vec4<f32>(vec3<f32>(select(1.0, ssao, fx.ao.x > 0.5)), 1.0);
  }
  if (frame.light_color.w > 0.5) {
    return vec4<f32>(cascade_tint(cascade_index(in.world_pos)) * 4.0, 1.0);
  }
  return vec4<f32>(apply_fog(color, in.world_pos), alpha);
}
