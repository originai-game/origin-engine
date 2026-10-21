// The physical lit shader (three.js MeshPhysicalMaterial, glTF
// KHR_materials_*), after pbr.wgsl in place of pbr_fs.wgsl and
// pbr_prepass.wgsl, with material_physical.wgsl's bindings: the standard
// shading plus clearcoat maps, sheen (Charlie distribution, Neubelt
// visibility), transmission through the copy of the opaque scene,
// iridescence (Belcour and Barla's thin film) and anisotropic GGX. Every
// lobe is behind a branch on its factor.

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
  iridescence: f32,
  iridescence_fresnel: vec3<f32>,
  anisotropy: f32,            // 0: isotropic
  aniso_t: vec3<f32>,         // the stretch's direction on the surface
  aniso_b: vec3<f32>,
  alpha_t: f32,               // GGX alpha along aniso_t (roughness² across it)
};

// What one light gives: the diffuse part (which transmission replaces)
// and the rest.
struct Shade {
  diffuse: vec3<f32>,
  specular: vec3<f32>,
};

fn max3(c: vec3<f32>) -> f32 {
  return max(c.x, max(c.y, c.z));
}

fn srgb_decode(c: vec3<f32>) -> vec3<f32> {
  return select(pow((c + 0.055) / 1.055, vec3<f32>(2.4)), c / 12.92, c <= vec3<f32>(0.04045));
}

// Sheen: the Charlie distribution and Neubelt's visibility (as three.js).
fn d_charlie(roughness: f32, n_dot_h: f32) -> f32 {
  let inv = 1.0 / (roughness * roughness);
  let sin2h = max(1.0 - n_dot_h * n_dot_h, 0.0078125);
  return (2.0 + inv) * pow(sin2h, inv * 0.5) / (2.0 * PI);
}

fn v_neubelt(n_dot_v: f32, n_dot_l: f32) -> f32 {
  return clamp(1.0 / (4.0 * (n_dot_l + n_dot_v - n_dot_l * n_dot_v)), 0.0, 1.0);
}

// The sheen's directional albedo under the image-based light (Estevez and
// Kulla's fit), in this shader's irradiance units (irradiance / pi).
fn sheen_ibl(n_dot_v: f32, r: f32) -> f32 {
  let r2 = r * r;
  let low = r < 0.25;
  let a = select(-8.48 * r2 + 14.3 * r - 9.95, -339.2 * r2 + 161.4 * r - 25.9, low);
  let b = select(1.97 * r2 - 3.27 * r + 0.72, 44.0 * r2 - 23.7 * r + 3.26, low);
  let dg = exp(a * n_dot_v + b) + select(0.1 * (r - 0.25), 0.0, low);
  return clamp(dg, 0.0, PI);
}

// Anisotropic GGX (as three.js and Filament): alpha_t along t, alpha_b across.
fn d_ggx_aniso(alpha_t: f32, alpha_b: f32, n_dot_h: f32, t_dot_h: f32, b_dot_h: f32) -> f32 {
  let a2 = alpha_t * alpha_b;
  let w = vec3<f32>(alpha_b * t_dot_h, alpha_t * b_dot_h, a2 * n_dot_h);
  let w2 = a2 / dot(w, w);
  return a2 * w2 * w2 / PI;
}

fn v_ggx_aniso(alpha_t: f32, alpha_b: f32, tv: f32, bv: f32, tl: f32, bl: f32, n_dot_v: f32, n_dot_l: f32) -> f32 {
  // (Not `gl`: GLSL reserves the gl_ prefix naga's renaming would give it.)
  let lambda_v = n_dot_l * length(vec3<f32>(alpha_t * tv, alpha_b * bv, n_dot_v));
  let lambda_l = n_dot_v * length(vec3<f32>(alpha_t * tl, alpha_b * bl, n_dot_l));
  return clamp(0.5 / (lambda_v + lambda_l), 0.0, 1.0);
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

// One light's shading: the GGX lobe (anisotropic, with the film's Fresnel),
// the diffuse base, the sheen over them and the coat over all.
fn physical_brdf(s: Surface, k: Lobes, n: vec3<f32>, v: vec3<f32>, l: vec3<f32>) -> Shade {
  let h = normalize(v + l);
  let n_dot_l = max(dot(n, l), 0.0);
  let n_dot_v = max(dot(n, v), 1e-4);
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
    let d = d_ggx(n_dot_h, s.roughness * s.roughness);
    let g = g_smith(n_dot_v, n_dot_l, s.roughness);
    spec = (d * g * f) / (4.0 * n_dot_v * n_dot_l + 1e-4);
  }
  var o: Shade;
  o.diffuse = (vec3<f32>(1.0) - f) * (1.0 - s.metallic) * s.albedo / PI * n_dot_l;
  o.specular = spec * n_dot_l;
  if (any(k.sheen_color > vec3<f32>(0.0))) {
    // The sheen layer takes its albedo from what it covers.
    let scale = 1.0 - 0.157 * max3(k.sheen_color);
    o.diffuse *= scale;
    o.specular = o.specular * scale + k.sheen_color * d_charlie(k.sheen_roughness, n_dot_h) * v_neubelt(n_dot_v, n_dot_l) * n_dot_l;
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
  if (atten <= 0.0 || dot(n, l) <= 0.0) { return o; }
  var radiance = light.color_kind.rgb * atten;
  if (receives) { radiance *= local_shadow(light, p, n); }
  let b = physical_brdf(s, k, n, v, l);
  o.diffuse = b.diffuse * radiance;
  o.specular = b.specular * radiance;
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
// has a full mip chain; WebGL2 cannot ask a texture for its levels).
fn scene_behind(uv: vec2<f32>, lod: f32) -> vec3<f32> {
  let size = vec2<f32>(textureDimensions(transmission_tex));
  let top = floor(log2(max(size.x, size.y)));
  let lo = min(floor(lod), top);
  let hi = min(lo + 1.0, top);
  return mix(bicubic_level(uv, lo), bicubic_level(uv, hi), fract(lod));
}

@fragment
fn fs(in: PhysicalOut) -> @location(0) vec4<f32> {
  let uv = stylised_uv(in.uv, in.affine);
  let base_sample = textureSample(base_color_tex, material_sampler, uv);
  var hook: Hook;
  hook.world = in.world_pos;
  hook.local = in.local_pos;
  hook.normal = in.normal;
  hook.uv = uv;
  hook.color = in.color;
  hook.time = frame.time.x;
  let params = shader_params();
  let base = surface_color(hook, params, look_tint(material.base_color * base_sample * in.color, in.look));
  var dissolve = vec4<f32>(0.0);
  if (EFFECTS) { dissolve = dissolve_at(uv, in.local_pos); }
  if ((material.emissive.w >= 0.0 && base.a < material.emissive.w) || dissolve.w > 0.5) {
    discard;
  }
  let mode = u32(material.flags.w);
  let alpha = select(1.0, base.a, (mode & 4u) != 0u);
  // The packed maps (physical.rs): white where a material has none.
  let maps = pack_sample(physical.layers.x, uv);      // metalness, roughness, ao, dissolve
  let coat_maps = pack_sample(physical.layers.y, uv); // clearcoat, its roughness, transmission, thickness
  let sheen_maps = pack_sample(physical.layers.z, uv); // sheen colour (sRGB), its roughness
  let film_maps = pack_sample(physical.layers.w, uv);  // the coat's normal, iridescence
  let aniso_maps = pack_sample(physical.layers2.x, uv); // anisotropy, the film's thickness
  let dpx = dpdx(in.world_pos);
  let dpy = dpdy(in.world_pos);
  var s: Surface;
  s.metallic = clamp(material.mr.x * maps.r, 0.0, 1.0);
  s.roughness = clamp(material.mr.y * maps.g, 0.04, 1.0);
  s.albedo = base.rgb;

  var n = normalize(in.normal);
  if ((mode & 1u) != 0u) {
    n = normalize(cross(dpx, dpy));
  }
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
  // The dielectric reflectance from the index of refraction.
  let ior = physical.transmission.z;
  let f0 = ior_to_f0(ior, 1.0);
  s.f0 = mix(vec3<f32>(f0), s.albedo, s.metallic);
  // The vertex tangent frame (pbr_fs.wgsl's), for the normal maps and the stretch.
  var t = in.tangent.xyz - n_vertex * dot(n_vertex, in.tangent.xyz);
  let framed = dot(t, t) > 1e-12;
  t = select(normalize(cross(select(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), abs(n_vertex.y) > 0.9), n_vertex)), normalize(t), framed);
  let bt = cross(n_vertex, t) * select(1.0, -1.0, in.tangent.w < 0.0);
  let tbn = mat3x3<f32>(t, bt, n_vertex);
  let normal_sample = textureSample(normal_tex, material_sampler, uv);
  if (material.flags.x > 0.5) {
    var map = normal_sample.xyz * 2.0 - 1.0;
    if (material.flags.x > 1.5) {
      map.z = sqrt(max(1.0 - dot(map.xy, map.xy), 0.0));
    }
    map = vec3<f32>(map.xy * material.mr.zw, map.z);
    if (framed) { n = normalize(tbn * map) * face; }
  }
  n = surface_normal(hook, params, n);
  s.coat = material.coat.x * coat_maps.r;
  s.coat_roughness = clamp(material.coat.y * coat_maps.g, 0.04, 1.0);
  s.coat_normal = n_vertex * face;
  if (physical.coat.z > 0.5 && framed) {
    var map = film_maps.xyz * 2.0 - 1.0;
    map = vec3<f32>(map.xy * physical.coat.xy, map.z);
    s.coat_normal = normalize(tbn * map) * face;
  }
  let n_dot_v = max(dot(n, v), 1e-4);

  var k: Lobes;
  k.sheen_color = physical.sheen.rgb * srgb_decode(sheen_maps.rgb);
  k.sheen_roughness = clamp(physical.sheen.w * sheen_maps.a, 0.07, 1.0);
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
  if (physical.anisotropy.x > 0.0) {
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
  }

  // The directional light.
  let l = normalize(frame.light_dir.xyz);
  let n_dot_l = max(dot(n, l), 0.0);
  let receives = look_receives(in.look);
  var shadow = 1.0;
  if (frame.light_dir.w > 0.5 && n_dot_l > 0.0 && receives) { shadow = cascade_shadow(in.world_pos, normalize(in.normal), n_dot_l); }
  let sun = physical_brdf(s, k, n, v, l);
  var diffuse = sun.diffuse * frame.light_color.rgb * shadow;
  var specular = sun.specular * frame.light_color.rgb * shadow;
  diffuse += s.albedo * frame.light_color.rgb * n_dot_l * shadow * caustics(in.world_pos);
  if (frame.shadow_tint.w > 0.0 && n_dot_l > 0.0 && (mode & 8u) == 0u) {
    diffuse += (1.0 - shadow) * frame.shadow_tint.rgb * frame.shadow_tint.w * s.albedo * frame.light_color.rgb * n_dot_l;
  }

  if (frame.params.z > 0.5) {
    let word = cluster_word(cluster_at(in.clip.xy, in.world_pos));
    let first = word & 0xffffffu;
    let count = word >> 24u;
    for (var c = 0u; c < count; c++) {
      let o = physical_local(light_list[cluster_ref(first + c)], s, k, in.world_pos, n, v, receives);
      diffuse += o.diffuse;
      specular += o.specular;
    }
  }

  // Image-based light (split sum): an anisotropic surface reflects along
  // its bent normal (as three.js).
  let f_ibl = f_schlick_roughness(n_dot_v, f0_ibl, s.roughness);
  let kd_ibl = (vec3<f32>(1.0) - f_ibl) * (1.0 - s.metallic);
  let irradiance = textureSample(irradiance_map, ibl_sampler, n).rgb;
  var bent = n;
  if (physical.anisotropy.x > 0.0) {
    let across = normalize(cross(cross(k.aniso_b, v), k.aniso_b));
    let w = 1.0 - k.anisotropy * (1.0 - s.roughness);
    bent = normalize(mix(across, n, w * w * w * w));
  }
  let r = reflect(-v, bent);
  let prefiltered = textureSampleLevel(prefiltered_map, ibl_sampler, r, s.roughness * frame.params.y).rgb;
  let lut = textureSample(brdf_lut, ibl_sampler, vec2<f32>(n_dot_v, s.roughness)).rg;
  let env_brdf = f_ibl * lut.x + lut.y;
  let ao_sample = maps.b;
  var ibl_scale = frame.params.x * material.flags.z;
  if (material.flags.y >= 0.0) {
    ibl_scale *= mix(1.0, ao_sample, material.flags.y);
  }
  if (EFFECTS && fx.ao.x > 0.5) {
    let ssao_sample = textureSampleLevel(ssao_tex, fx_sampler, in.clip.xy * fx.screen.zw, 0.0).r;
    ibl_scale *= mix(1.0, ssao_sample, fx.ao.y);
  }
  // The coat's reflection of the environment, and what it takes from
  // everything under it (the lights' share is taken per light).
  var coat_keep = 1.0;
  var coat_ibl = vec3<f32>(0.0);
  if (material.coat.x > 0.0) {
    let c_dot_v = max(dot(s.coat_normal, v), 1e-4);
    let fc = f_schlick(c_dot_v, vec3<f32>(0.04)).x * s.coat;
    let rc = reflect(-v, s.coat_normal);
    let coat_env = textureSampleLevel(prefiltered_map, ibl_sampler, rc, s.coat_roughness * frame.params.y).rgb;
    let coat_lut = textureSampleLevel(brdf_lut, ibl_sampler, vec2<f32>(c_dot_v, s.coat_roughness), 0.0).rg;
    coat_keep = 1.0 - fc;
    coat_ibl = coat_env * (0.04 * coat_lut.x + coat_lut.y) * s.coat;
  }
  var ibl_diffuse = kd_ibl * irradiance * s.albedo;
  var ibl_specular = prefiltered * env_brdf;
  // The sheen over the base (the lights' share is in physical_brdf).
  var sheen_keep = 1.0;
  if (any(k.sheen_color > vec3<f32>(0.0))) {
    sheen_keep = 1.0 - 0.157 * max3(k.sheen_color);
    ibl_diffuse *= sheen_keep;
    ibl_specular = ibl_specular * sheen_keep + irradiance * k.sheen_color * sheen_ibl(n_dot_v, k.sheen_roughness);
  }
  diffuse += ibl_diffuse * coat_keep * ibl_scale;
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

  color += surface_emissive(hook, params, material.emissive.rgb * textureSample(emissive_tex, material_sampler, uv).rgb);
  if (EFFECTS && fx.decal_info.x > 0.5) { color += decal_emissive; }
  if (EFFECTS && material.rim.w > 0.0) { color += dissolve.rgb; }
  if (EFFECTS && material.toon.y > 0.0) {
    color += material.rim.rgb * material.toon.y * pow(1.0 - n_dot_v, material.toon.z);
  }
  if (EFFECTS && material.toon.w > 0.0) {
    color = mix(color, material.flash.rgb, material.toon.w);
  }
  color = look_flash(color, in.look);
  if (frame.light_color.w > 1.5) {
    let ssao = textureSampleLevel(ssao_tex, fx_sampler, in.clip.xy * fx.screen.zw, 0.0).r;
    return vec4<f32>(vec3<f32>(select(1.0, ssao, fx.ao.x > 0.5)), 1.0);
  }
  if (frame.light_color.w > 0.5) {
    let c = cascade_index(in.world_pos);
    var tint = vec3<f32>(0.0);
    if (c == 0u) { tint = vec3<f32>(1.0, 0.0, 0.0); }
    if (c == 1u) { tint = vec3<f32>(0.0, 1.0, 0.0); }
    if (c == 2u) { tint = vec3<f32>(0.0, 0.0, 1.0); }
    return vec4<f32>(tint * 4.0, 1.0);
  }
  return vec4<f32>(apply_fog(color, in.world_pos), alpha);
}
