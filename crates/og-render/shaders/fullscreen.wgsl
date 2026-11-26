// The fullscreen triangle every post pass draws. Prepended to the post shaders.
struct VsOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };

@vertex
fn vs(@builtin(vertex_index) i: u32) -> VsOut {
  var o: VsOut;
  o.pos = fullscreen_clip(i);
  o.uv = vec2<f32>(o.pos.x * 0.5 + 0.5, 0.5 - o.pos.y * 0.5);
  return o;
}

// A linear read of `t` at `uv`, exact where `uv` is one of its texel
// centres: there the filter's answer is that texel, so it is loaded. A
// centre is rarely representable ((x + 0.5) / width), and a filter rounds
// it onto its weight grid as it likes: PowerVR (Pixel 10) leaks 1/256 of
// the neighbour, so a 1:1 read through a linear sample was not a copy
// (tests/evidence/compat/android/filter-precision). A read between
// centres (a scaled or offset one) is filtered as before.
fn sample_exact(t: texture_2d<f32>, s: sampler, uv: vec2<f32>) -> vec4<f32> {
  let size = vec2<f32>(textureDimensions(t));
  let texel = uv * size - 0.5;
  let centre = round(texel);
  if (all(abs(texel - centre) < vec2<f32>(1.0 / 1024.0)) && all(centre >= vec2<f32>(0.0)) && all(centre < size)) {
    return textureLoad(t, vec2<i32>(centre), 0);
  }
  return textureSampleLevel(t, s, uv, 0.0);
}

// What a scene's luminance of 1 (a white surface in full sun, at the
// physical daylight's scale) is on the scale of metered_key's curve. 20
// puts the key at 0.18 where a daylit landscape's log mean lies, a
// sixteenth (log2 -4; the alpine valley's views meter -3.8 to -4.3 on the
// L4), so such a view is exposed as a fixed key of 0.18 exposes it. It is
// a calibration of the curve to the frames this engine draws, not the
// scene's luminance in cd/m² (a sunlit white is some 30000 of those, where
// the curve is flat: every daylit view at a key near 0.7).
const KEY_SCALE: f32 = 20.0;

// The key the meter gives a scene of mean log2 luminance `avg`: the display
// luminance its mean is exposed to. Krawczyk, Myszkowski and Seidel,
// "Perceptual Effects in Real-time Tone Mapping" (SCCG 2005), equation 11,
// key = 1.03 - 2 / (2 + log10(Y + 1)), Y the adapting luminance: a bright
// view keeps a high key and a dim one a low key, as an eye that adapts only
// part of the way sees them, from 0.03 in the dark to 0.18 at a sixteenth
// and 0.43 at a white in full sun.
//
// A fixed key of 0.18 brought every view's mean to middle grey: it could not
// tell a white under less light from a grey under more, so a cloud deck from
// above or a snowfield came out grey, as bright under a sun of 1 as of 20,
// and a night as bright as a day.
fn metered_key(avg: f32) -> f32 {
  return 1.03 - 2.0 / (2.0 + log(exp2(avg) * KEY_SCALE + 1.0) / log(10.0));
}

// The exposure EV the meter asks for: the scene's mean log2 luminance `avg`
// brought to its key (metered_key), moved by `compensation` EV, within
// `lo`..`hi`.
fn metered_ev(avg: f32, compensation: f32, lo: f32, hi: f32) -> f32 {
  return clamp(log2(metered_key(avg)) - avg + compensation, lo, hi);
}

// The mean log2 luminance a meter texel holds (luminance.wgsl): its
// weighted sum over its weight.
fn metered_log(t: vec4<f32>) -> f32 {
  return t.r / max(t.g, 1e-6);
}

// The exposure multiplier of a pass's Post rows: `e` is post.exposure (x:
// manual EV, y: 0 manual, 1 metered, 2 adapted, zw: the EV range),
// `compensation` its EV compensation, `luminance` the 1x1 meter (metered:
// the mean log2 luminance; adapted: the adapted EV, exposure.wgsl).
fn exposure_from(e: vec4<f32>, compensation: f32, luminance: texture_2d<f32>) -> f32 {
  let l = textureLoad(luminance, vec2<i32>(0), 0);
  if (e.y > 1.5) { return exp2(l.r); }
  if (e.y > 0.5) { return exp2(metered_ev(metered_log(l), compensation, e.z, e.w)); }
  return exp2(e.x);
}
