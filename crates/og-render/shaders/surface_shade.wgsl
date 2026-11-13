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
// (everything else); surface_matte for a body that reflects nothing. After lighting.wgsl, area.wgsl, the light
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

// The diffuse light arriving at a surface along `ambient_n`, with the
// probe GI's gather `gi` there.
fn surface_irradiance(at: ShadeAt, ambient_n: vec3<f32>, gi: GiSample) -> vec3<f32> {
  // The baked lightmap or the light volumes where there are, at the
  // environment's intensity; the hemisphere light adds at its own.
  let irradiance = ambient_irradiance(ambient_light_from(ambient_n, at.p, gi), at.p, at.n) * frame.params.x + hemi_irradiance(at.n);
  if (at.styled) { return style_irradiance(irradiance, at.p); }
  return irradiance;
}

// The gather for a surface of `roughness` that reflects around `around`
// (its normal; an anisotropic surface's bent normal).
fn surface_gather(at: ShadeAt, around: vec3<f32>, roughness: f32) -> Gathered {
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
  // Relit by the light volumes and occluded by the probe GI where there are.
  o.prefiltered = gi_reflected_from(volume_reflected(reflected_light(r, at.p, roughness * frame.params.y), at.p, r, roughness), r, at.p, gi) * frame.params.x;
  // The screen-space reflections over it, for the surface they were traced for.
  if (at.opaque) { o.prefiltered = ssr_reflected(o.prefiltered, at.p, at.frag); }
  o.lut = textureSampleLevel(brdf_lut, ibl_sampler, vec2<f32>(clamp(dot(at.n, at.v), 1e-4, 1.0), roughness), 0.0).rg;
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

fn surface_ambient(s: Surface, at: ShadeAt) -> Ambient {
  let g = surface_gather(at, at.n, s.roughness);
  var o: Ambient;
  o.kd = (vec3<f32>(1.0) - f_schlick_roughness(clamp(dot(at.n, at.v), 1e-4, 1.0), s.f0, s.roughness)) * (1.0 - s.metallic);
  o.diffuse = g.irradiance * s.albedo;
  // Split sum as Karis (and three.js): F0 scales the LUT, whose bias term
  // already holds the Fresnel rise at grazing angles.
  o.specular = g.prefiltered * (s.f0 * g.lut.x + g.lut.y);
  o.occlusion = g.occlusion;
  return o;
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
  o.emissive = emissive * a.occlusion.all;
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
  return surface_sum(s, direct, surface_ambient(s, at), emissive, intensity, keep * sun.keep);
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
