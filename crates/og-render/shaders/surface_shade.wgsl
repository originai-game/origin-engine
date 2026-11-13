// The light every lit surface takes after its own sun term: one gather of
// the light around it (the environment, the reflection probe, the light
// volumes, the probe GI, the screen's reflections and occlusion) and one
// sum of it with the caustics and the local lights. A surface fills a
// `Surface` (lighting.wgsl) and a `ShadeAt`, lights itself by the sun as it
// must (its own shadow lookup, its own lobes), and hands both here; none
// keeps a copy of this tail. After lighting.wgsl, area.wgsl, the light
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

// The light around a surface, before what the surface keeps of it.
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
  var o: Ambient;
  let n_dot_v = clamp(dot(at.n, at.v), 1e-4, 1.0);
  o.kd = (vec3<f32>(1.0) - f_schlick_roughness(n_dot_v, s.f0, s.roughness)) * (1.0 - s.metallic);
  o.occlusion = ScreenOcclusion(1.0, 1.0, vec3<f32>(0.0));
  var ambient_n = at.n;
  if (at.screen) {
    o.occlusion = screen_occlusion(at.frag);
    ambient_n = ambient_normal(at.frag, at.n);
  }
  // The probe GI's one gather, for the diffuse light along the ambient
  // normal and the reflections' occlusion.
  let gi = gi_sample_at(at.p, ambient_n);
  // The baked lightmap or the light volumes where there are, at the
  // environment's intensity; the hemisphere light adds at its own.
  var irradiance = ambient_irradiance(ambient_light_from(ambient_n, at.p, gi), at.p, at.n) * frame.params.x + hemi_irradiance(at.n);
  if (at.styled) { irradiance = style_irradiance(irradiance, at.p); }
  o.diffuse = irradiance * s.albedo;
  // three.js's getIBLRadiance (r170): the reflection bent towards the
  // normal by roughness squared before the prefiltered map is read (a
  // rough surface reflects around its normal, not the mirror direction).
  let r = normalize(mix(reflect(-at.v, at.n), at.n, s.roughness * s.roughness));
  var prefiltered = gi_reflected_from(volume_reflected(reflected_light(r, at.p, s.roughness * frame.params.y), at.p, r, s.roughness), r, at.p, gi) * frame.params.x;
  if (at.opaque) { prefiltered = ssr_reflected(prefiltered, at.p, at.frag); }
  // Split sum as Karis (and three.js): F0 scales the LUT, whose bias term
  // already holds the Fresnel rise at grazing angles.
  let lut = textureSampleLevel(brdf_lut, ibl_sampler, vec2<f32>(n_dot_v, s.roughness), 0.0).rg;
  o.specular = prefiltered * (s.f0 * lut.x + lut.y);
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

// `s` at `at` under `sun` (its sun term: lighting.wgsl style_sun, or the
// surface's own with `keep` 1), with `sunlit` its n.l times its shadow
// (the caustics' share of the sun), `emissive` what it emits, `intensity`
// the material's share of the light around it (envMapIntensity) and
// `keep` what its own occlusion leaves of that light (an AO map, a baked
// sky occlusion).
fn surface_shade(s: Surface, at: ShadeAt, sun: StyleSun, sunlit: f32, emissive: vec3<f32>, intensity: f32, keep: f32) -> Shaded {
  var o: Shaded;
  o.direct = sun.direct;
  // Caustics where the surface lies under the frame's water (0 without).
  o.direct += s.albedo * frame.light_color.rgb * sunlit * caustics(at.p);
  // Point, spot and rect lights: this fragment's cluster's, or every light
  // in a reflection probe's face.
  o.direct += local_lights_on(at.frag, s, at.p, at.n, at.v, at.styled);
  let a = surface_ambient(s, at);
  // The style's shadow darkness takes its share of the light around the
  // surface as of the sun's.
  let kept = keep * sun.keep;
  let scale = intensity * kept * a.occlusion.ambient;
  o.direct *= a.occlusion.all;
  o.diffuse = a.kd * (a.diffuse * scale + s.albedo * a.occlusion.bounce * kept);
  o.specular = a.specular * scale;
  o.emissive = emissive * a.occlusion.all;
  return o;
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
