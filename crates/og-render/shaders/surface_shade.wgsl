// The light every lit surface takes after its own sun term: one gather of
// the light around it (the environment, the reflection probe, the light
// volumes, the probe GI, the screen's reflections and occlusion) and one
// sum of it with the caustics and the local lights. A surface fills a
// `Surface` (lighting.wgsl) and a `ShadeAt`, lights itself by the sun as it
// must (its own shadow lookup, its own lobes), and hands both here; none
// keeps a copy of this tail. Three entries, each built on the one before:
// surface_gather (a material with lobes of its own: the physical shader),
// surface_ambient and surface_sum (a material that changes the light
// around it before the sum: a mesh's toon ambient and coat), surface_shade
// (everything else); surface_matte for a body that reflects nothing;
// surface_irradiance, surface_reflected and local_lights_on for a surface
// with a reflection model of its own (the water). After lighting.wgsl, area.wgsl, the light
// volumes' hook (lightmap.rs `probe_wgsl`) and screen_ao.wgsl or its stub.

// Where and how a surface is shaded.
struct ShadeAt {
  // The world point, the shading normal and the unit vector to the eye.
  p: vec3<f32>,
  n: vec3<f32>,
  v: vec3<f32>,
  // The fragment's pixel.
  frag: vec2<f32>,
  // Whether it reads the screen's occlusion and bent normal (the AO
  // texture is the camera's opaque surfaces').
  screen: bool,
  // Whether it is the opaque surface the camera drew at the pixel, which
  // the screen-space reflections were traced for.
  opaque: bool,
  // Whether the frame's material style applies to it.
  styled: bool,
};

// What a surface gathers of the light around it, before its own response.
struct Gathered {
  // The diffuse light arriving along its ambient normal.
  irradiance: vec3<f32>,
  // The light arriving along its reflection, prefiltered to its roughness.
  prefiltered: vec3<f32>,
  // The split sum's scale and bias (the BRDF table at n.v and roughness).
  lut: vec2<f32>,
  // The screen's occlusion at the pixel.
  occlusion: ScreenOcclusion,
};

// The static lights' far field (static_far.wgsl) is in a pipeline only
// while a frame draws it (pipeline_features.rs `STATIC_FAR`): a lit
// pipeline that merely carries its gather costs registers, so with the
// reach at 0 none does.
override OG_STATIC_FAR: bool = true;

// The diffuse light arriving at a surface along `ambient_n`, with the
// probe GI's gather `gi` there.
fn surface_irradiance(at: ShadeAt, ambient_n: vec3<f32>, gi: GiSample) -> vec3<f32> {
  // The baked lightmap or the light volumes where there are; sky env is
  // already scaled by frame.params.x inside ambient_light_from, and the
  // hemisphere light adds at its own intensity.
  var irradiance = ambient_irradiance(ambient_light_from(ambient_n, at.p, gi), at.p, at.n) + hemi_irradiance(at.n);
  // The static lights' far field, here alone, so the lit shaders carry
  // one copy of its gather.
  if (OG_STATIC_FAR) { irradiance += static_far_irradiance(at.p, at.n); }
  if (at.styled) { return style_irradiance(irradiance, at.p); }
  return irradiance;
}

// `prefiltered` (already scaled by frame.params.x) as a surface of
// `roughness` at `at` receives it: relit by the light volumes and
// occluded by the probe GI's gather `gi` where there are. After
// surface_irradiance at the same point (the light volumes' weight is
// taken there).
fn surface_reflected(prefiltered: vec3<f32>, at: ShadeAt, r: vec3<f32>, roughness: f32, gi: GiSample) -> vec3<f32> {
  return gi_reflected_from(volume_reflected(prefiltered, at.p, r, roughness), r, at.p, gi);
}

// The gather for a surface of `roughness` that reflects around `around`
// (its normal; an anisotropic surface's bent normal; a leaf's own face),
// its Fresnel taken at `facet` (its normal; a leaf's own face).
fn surface_gather(at: ShadeAt, around: vec3<f32>, facet: vec3<f32>, roughness: f32) -> Gathered {
  var o: Gathered;
  o.occlusion = ScreenOcclusion(1.0, 1.0, vec3<f32>(0.0));
  var ambient_n = at.n;
  if (at.screen) {
    o.occlusion = screen_occlusion(at.frag);
    ambient_n = ambient_normal(at.frag, at.n);
  }
  // The probe GI's one gather, for the diffuse light along the ambient
  // normal and the reflections' occlusion.
  let gi = gi_sample_at(at.p, ambient_n);
  o.irradiance = surface_irradiance(at, ambient_n, gi);
  // three.js's getIBLRadiance (r170): the reflection bent towards the
  // normal by roughness squared before the prefiltered map is read (a
  // rough surface reflects around its normal, not the mirror direction;
  // a back-facing leaf card otherwise mirrors the bright sky behind it).
  let r = normalize(mix(reflect(-at.v, around), around, roughness * roughness));
  o.prefiltered = surface_reflected(reflected_light(r, at.p, roughness * frame.params.y) * frame.params.x, at, r, roughness, gi);
  // The screen-space reflections over it, for the surface they were traced for.
  if (at.opaque) { o.prefiltered = ssr_reflected(o.prefiltered, at.p, at.frag); }
  o.lut = textureSampleLevel(brdf_lut, ibl_sampler, vec2<f32>(clamp(dot(facet, at.v), 1e-4, 1.0), roughness), 0.0).rg;
  return o;
}

// The light around a surface as it reflects it, before what the surface
// keeps of it.
struct Ambient {
  // The share of the diffuse light the surface takes: what its Fresnel
  // reflection and its metal leave.
  kd: vec3<f32>,
  // The diffuse light off its albedo (before `kd`).
  diffuse: vec3<f32>,
  // The reflection, split sum.
  specular: vec3<f32>,
  // The screen's occlusion at the pixel.
  occlusion: ScreenOcclusion,
};

// Lagarde & de Rousiers 2014 specular occlusion from diffuse AO, N.V and roughness.
fn specular_occlusion(n_dot_v: f32, ao: f32, roughness: f32) -> f32 {
  return clamp(pow(n_dot_v + ao, exp2(-16.0 * roughness - 1.0)) - 1.0 + ao, 0.0, 1.0);
}

// Lagarde & de Rousiers 2014 horizon occlusion: attenuates specular IBL when
// a normal-mapped reflection ray dips below the geometric surface plane.
fn specular_horizon_occlusion(r: vec3<f32>, geo_n: vec3<f32>) -> f32 {
  let h = clamp(1.0 + 1.3 * dot(r, geo_n), 0.0, 1.0);
  return h * h;
}

// Multi-scattering GGX IBL directional reflectance (Fdez-Aguera 2019 / Turquin 2019)
// using the existing single-scattering split-sum BRDF LUT (scale, bias).
fn multi_scatter_ibl(f0: vec3<f32>, lut: vec2<f32>) -> vec3<f32> {
  let ess = lut.x + lut.y;
  let ems = max(1.0 - ess, 0.0);
  let f_avg = f0 + (vec3<f32>(1.0) - f0) / 21.0;
  let f_single = f0 * lut.x + lut.y;
  let f_ms = f_single * f_avg / max(vec3<f32>(1.0) - f_avg * ems, vec3<f32>(1e-4));
  return f_single + f_ms * ems;
}

// `facet`: the face that reflects, its normal for every surface but a
// leaf's, whose light around it arrives along its canopy's normal
// (foliage.wgsl `foliage_normal`) while its own face mirrors the
// sky.
fn surface_ambient(s: Surface, at: ShadeAt, facet: vec3<f32>) -> Ambient {
  let g = surface_gather(at, facet, facet, s.roughness);
  var o: Ambient;
  let k_spec = multi_scatter_ibl(s.f0, g.lut);
  o.kd = max(vec3<f32>(1.0) - k_spec, vec3<f32>(0.0)) * (1.0 - s.metallic);
  o.diffuse = g.irradiance * s.albedo;
  o.specular = g.prefiltered * k_spec;
  // Apply Lagarde specular occlusion from screen AO while preserving
  // linear AO for diffuse irradiance in surface_sum.
  let n_dot_v = clamp(dot(facet, at.v), 1e-4, 1.0);
  let spec_ao = specular_occlusion(n_dot_v, g.occlusion.ambient, s.roughness);
  o.specular *= spec_ao / max(g.occlusion.ambient, 1e-4);
  o.occlusion = g.occlusion;
  return o;
}

// What a crown's leaves pass on, forwards, of the light they take: half
// their single scattering (their colour reflected and let through), and
// none for a surface that is not a translucent leaf (its neighbours'
// colour is not its own).
fn crown_forward(s: Surface) -> vec3<f32> {
  if (s.translucency <= 0.0) { return vec3<f32>(0.0); }
  return 0.5 * min(s.albedo * (1.0 + s.translucency), vec3<f32>(1.0));
}

// What reaches a point behind `hidden` of a beam through a crown whose
// leaves pass on `forward` of it: the light scattered forwards keeps
// going, its extinction cut by `forward` (the delta scaling of Joseph,
// Wiscombe and Weinman 1976), so it falls off as `forward` to the power
// of the leaves crossed (a canopy's recollision, Smolander and Stenberg
// 2005), and what is passed is what crosses the hidden share.
fn crown_passed(hidden: f32, forward: vec3<f32>) -> vec3<f32> {
  return pow(vec3<f32>(max(1.0 - hidden, 1e-4)), vec3<f32>(1.0) - forward);
}

// The light around a surface inside its tree's crown, which hides
// `crown` of the sky (foliage.wgsl `meshlet_crown`, Hegeman et al.
// 2006): the diffuse light is what comes through the gaps and what the
// leaves pass on (`crown_passed`), in their colour, darker and greener
// each leaf deeper; its reflection of the sky is occluded as Lagarde and
// de Rousiers derive it from that visibility ("Moving Frostbite to PBR",
// 2014), so a leaf deep in the crown keeps its hue instead of mirroring
// the sky.
fn crown_ambient(a: Ambient, s: Surface, n_dot_v: f32, crown: f32) -> Ambient {
  if (crown <= 0.0) { return a; }
  let seen = 1.0 - clamp(crown, 0.0, 1.0);
  var o = a;
  o.diffuse *= crown_passed(crown, crown_forward(s));
  o.specular *= specular_occlusion(n_dot_v, seen, s.roughness);
  return o;
}

// The sun a surface inside its tree's crown sees: at most what the
// crown's leaves let through on its path (`crown`: foliage.wgsl
// `meshlet_crown`'s y), below `shadow` the shadow map's. The map holds
// the leaves as cards at its texel size, filtered and biased, and lets
// through the sun a crown of them hides from its depths (Hegeman et al.
// 2006), so a crown's inside is lit like its outside; the map's own where
// it is darker (another tree, a wall).
fn crown_sun(shadow: f32, crown: f32) -> f32 {
  return select(shadow, min(shadow, 1.0 - crown), crown > 0.0);
}

// The sun a leaf's crown took from it (`crown`, as `crown_sun`'s) is not
// all lost: its leaves pass some on (`crown_passed`; Boulanger, Pattanaik
// and Bouatouch, "Rendering trees with indirect lighting in real time",
// EGSR 2009), in their colour, less the deeper the leaf, so a leaf in its
// crown's shade keeps its hue and the crown its depth. Only of the sun
// that reaches the crown: where the map (`map_shadow`) is darker than the
// crown alone, something else took the rest. Zero for a surface that is
// not a translucent leaf.
fn crown_passed_sun(s: Surface, map_shadow: f32, crown: f32) -> vec3<f32> {
  if (s.translucency <= 0.0 || crown <= 0.0 || frame.light_dir.w <= 0.5) { return vec3<f32>(0.0); }
  let through = 1.0 - crown;
  let reach = clamp(map_shadow / max(through, 1e-3), 0.0, 1.0);
  let passed = crown_passed(crown, crown_forward(s)) - vec3<f32>(through);
  return frame.light_color.rgb * reach * max(passed, vec3<f32>(0.0)) * s.albedo * (1.0 - s.metallic) / PI;
}

// A seasonal leaf's colour: its own (`own`, the material's base colour
// times the vertex colour) with its autumn colour in its place by
// `season`'s a (foliage.wgsl `season_leaf_color`).
fn season_mix(own: vec4<f32>, season: vec4<f32>) -> vec4<f32> {
  if (season.a <= 0.0) { return own; }
  return vec4<f32>(mix(own.rgb, season.rgb, season.a), own.a);
}

// A shaded surface's light in the parts the debug views show
// (DebugView::lit_code); their sum is its colour before the fog.
struct Shaded {
  // The sun, the caustics and the local lights.
  direct: vec3<f32>,
  // The diffuse and the specular light from around it.
  diffuse: vec3<f32>,
  specular: vec3<f32>,
  // What it emits.
  emissive: vec3<f32>,
};

// `direct` (the sun, the caustics and the local lights on the surface) and
// `a` (the light around it) summed: `emissive` what it emits, `intensity`
// the material's share of the light around it (envMapIntensity) and `kept`
// what its own occlusion and the style's shadow darkness leave of that
// light (an AO map, a baked sky occlusion).
fn surface_sum(s: Surface, direct: vec3<f32>, a: Ambient, emissive: vec3<f32>, intensity: f32, kept: f32) -> Shaded {
  var o: Shaded;
  let scale = intensity * kept * a.occlusion.ambient;
  o.direct = direct * a.occlusion.all;
  o.diffuse = a.kd * (a.diffuse * scale + s.albedo * a.occlusion.bounce * kept);
  o.specular = a.specular * scale;
  o.emissive = emissive;
  return o;
}

// The direct light on `s` at `at`: `sun` (its sun term), the caustics at
// `sunlit` (its n.l times its shadow, their share of the sun) and the
// local lights.
fn surface_direct(s: Surface, at: ShadeAt, sun: vec3<f32>, sunlit: f32) -> vec3<f32> {
  var direct = sun;
  // Caustics where the surface lies under the frame's water (0 without).
  direct += s.albedo * frame.light_color.rgb * sunlit * caustics(at.p);
  // Point, spot and rect lights: this fragment's cluster's, or every light
  // in a reflection probe's face.
  direct += local_lights_on(at.frag, s, at.p, at.n, at.v, at.styled);
  return direct;
}

// `s` at `at` under `sun` (its sun term: lighting.wgsl style_sun, or the
// surface's own with `keep` 1), with `sunlit` as surface_direct's and
// `emissive`, `intensity` and `keep` as surface_sum's; the style's shadow
// darkness takes its share of the light around the surface as of the sun's.
fn surface_shade(s: Surface, at: ShadeAt, sun: StyleSun, sunlit: f32, emissive: vec3<f32>, intensity: f32, keep: f32) -> Shaded {
  let direct = surface_direct(s, at, sun.direct, sunlit);
  return surface_sum(s, direct, surface_ambient(s, at, at.n), emissive, intensity, keep * sun.keep);
}

// A matte body at `at` under `sun` (a lit particle, a puff of smoke): the
// direct light and the diffuse light around it off its albedo, with no
// reflection to gather and no screen term (it lies over the opaque
// surfaces those were made for).
fn surface_matte(s: Surface, at: ShadeAt, sun: vec3<f32>, sunlit: f32) -> vec3<f32> {
  let irradiance = surface_irradiance(at, at.n, gi_sample_at(at.p, at.n));
  return surface_direct(s, at, sun, sunlit) + irradiance * s.albedo;
}

fn shaded_color(o: Shaded) -> vec3<f32> {
  return o.direct + o.diffuse + o.specular + o.emissive;
}

// The lit debug views of a shaded surface (DebugView::lit_code `m`): 3
// albedo, 4 diffuse image-based light, 5 specular image-based light, 6
// direct light.
fn shaded_debug(o: Shaded, albedo: vec3<f32>, m: u32) -> vec3<f32> {
  if (m == 3u) { return albedo; }
  if (m == 4u) { return o.diffuse; }
  if (m == 5u) { return o.specular; }
  return o.direct;
}
