// The screen-space AO on lit surfaces (the effect group's `ssao_tex`,
// fx.wgsl) and the local lights' contact shadows in the same texture.
// A module without the effect group has `SCREEN_AO_STUB` instead
// (og-render-data's shader_capability.rs).

// The direction a lit fragment at `clip` reads its diffuse ambient light
// along: GTAO's bent normal where the AO texture holds it beside the
// probe GI (fx.ao.w 2: fxpost.wgsl `fs_gtao_bent`, blurred, so it is
// normalised again), the least occluded direction around `n`, which the
// probes then light; `n` itself otherwise (and in a pipeline without the
// probe GI's code, OG_GI: the bent normals are made only for it).
fn ambient_normal(clip: vec2<f32>, n: vec3<f32>) -> vec3<f32> {
  if (!OG_GI || fx.ao.x < 0.5 || fx.ao.w < 1.5) { return n; }
  let b = textureSampleLevel(ssao_tex, fx_sampler, clip * fx.screen.zw, 0.0).gba;
  let l = length(b);
  if (l < 0.25) { return n; }
  // Never past the surface's horizon: a bent normal blurred across a
  // crease may point below it.
  let bn = b / l;
  return normalize(bn + n * max(0.05 - dot(bn, n), 0.0));
}

// `screen_ao(p)` for material shaders: this frame's screen-space AO where
// world point `p` lands on the screen, 1 without AO. An unlit material is
// not darkened by the AO on its own; three.js's post AO (FogShader)
// darkened every pixel, unlit water included, and a shader can do the same.
fn screen_ao(p: vec3<f32>) -> f32 {
  if (fx.ao.x < 0.5 || oit_pass) { return 1.0; }
  let c = fx.view_proj_unjittered * vec4<f32>(p, 1.0);
  let ndc = c.xy / max(c.w, 1e-4);
  return textureSampleLevel(ssao_tex, fx_sampler, vec2<f32>(0.5 + 0.5 * ndc.x, 0.5 - 0.5 * ndc.y), 0.0).r;
}

// What the screen's occlusion keeps of a lit fragment's light.
struct ScreenOcclusion {
  // Of its image-based light.
  ambient: f32,
  // Of its whole colour, the emitted light too (PostEffects.aoDirect:
  // three.js-era post AO multiplies everything).
  all: f32,
  // Screen-space GI's bounce (ssgi.rs: the AO texture's gba with `ao.w`
  // 1), the light a diffuse surface receives from what the screen shows
  // around it.
  bounce: vec3<f32>,
};

// The screen's occlusion of the lit fragment at pixel `frag`: none without
// the AO texture, and for a blended surface in the order-independent
// passes (the texture holds the opaque surface's behind it). `ao.z` moves
// the image-based light to the floor the whole colour takes (0.35).
fn screen_occlusion(frag: vec2<f32>) -> ScreenOcclusion {
  var o = ScreenOcclusion(1.0, 1.0, vec3<f32>(0.0));
  if (fx.ao.x < 0.5 || oit_pass) { return o; }
  let texel = textureSampleLevel(ssao_tex, fx_sampler, frag * fx.screen.zw, 0.0);
  if (fx.ao.w > 0.5 && fx.ao.w < 1.5) { o.bounce = texel.gba; }
  o.ambient = mix(mix(1.0, texel.r, fx.ao.y), mix(0.35, 1.0, texel.r), fx.ao.z);
  o.all = mix(1.0, mix(0.35, 1.0, texel.r), fx.ao.z);
  return o;
}

// A local light's screen-space contact shadow at world point `p`
// (contact.wgsl `cs_contact`, its mask in the effect group's AO texture's
// g, b or a): a contact light's shadow slot carries a quarter, a half or
// three quarters past its layer (render.rs). 1 for every other light, in a
// reflection probe's face (no clusters: the mask is the camera's) and
// where the AO texture is the white placeholder (views, reflections), and
// for a blended surface in the order-independent passes (oit.wgsl
// `oit_pass`), which the opaque surface's mask behind it would darken.
fn contact_shadow(light: LocalLight, p: vec3<f32>) -> f32 {
  let w = light.direction_slot.w;
  let c = i32(round(fract(max(w, 0.0)) * 4.0)) - 1;
  if (c < 0 || frame.cluster.x <= 0.0 || oit_pass) { return 1.0; }
  let clip = fx.view_proj_unjittered * vec4<f32>(p, 1.0);
  let ndc = clip.xy / max(clip.w, 1e-4);
  let m = textureSampleLevel(ssao_tex, fx_sampler, vec2<f32>(0.5 + 0.5 * ndc.x, 0.5 - 0.5 * ndc.y), 0.0);
  return select(select(m.a, m.b, c == 1), m.g, c == 0);
}
