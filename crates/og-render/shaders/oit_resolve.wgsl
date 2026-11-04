// Order-independent transparency's resolve (oit.rs): a full-screen pass
// over the opaque scene, before the additive effects, the sprites and the
// post chain (TAA reads the revealage as its reactive mask). After
// fullscreen.wgsl and the bindings oit.rs `resolve_source` declares:
// `oit_accum_at`, `oit_reveal_at` and `oit_front_at` (each sample's weighted
// sums and front layer), and
// `oit_exact`, a pixel's sorted nodes composited (oit_list.wgsl), or a
// negative alpha where the pixel has none to give (the weighted mode, or
// a pixel that overflowed its nodes). With MSAA it runs per sample.
//
// Weighted blending (McGuire and Bavoil 2013, eq. 10): the weighted mean
// of the fragments' colours, covering what the product of their
// transmittances does not let through.
fn oit_weighted_color(accum: vec4<f32>, reveal: f32) -> vec4<f32> {
  let alpha = 1.0 - clamp(reveal, 0.0, 1.0);
  // A saturated half float (infinity) stays finite; alpha is never
  // inferred from the weights.
  let sums = min(accum, vec4<f32>(65504.0));
  return vec4<f32>(sums.rgb / max(sums.a, 1e-5) * alpha, alpha);
}

// The weighted mode's front layer (oit_arena.wgsl `oit_hybrid`) over the
// weighted tail: the front's premultiplied sums (one fragment, or the
// average of coplanar ones) covering by one minus its transmittance.
fn oit_hybrid_color(accum: vec4<f32>, reveal: vec2<f32>, front: vec4<f32>) -> vec4<f32> {
  let tail = oit_weighted_color(accum, reveal.x);
  let front_alpha = 1.0 - clamp(reveal.y, 0.0, 1.0);
  let sums = min(front, vec4<f32>(65504.0));
  let near = vec4<f32>(sums.rgb / max(sums.a, 1e-5) * front_alpha, front_alpha);
  return near + tail * (1.0 - front_alpha);
}

// Premultiplied, blended One / OneMinusSrcAlpha over the scene.
fn oit_resolve(p: vec2<i32>, sample: u32) -> vec4<f32> {
  var c = oit_exact(p, sample);
  if (c.a < 0.0) {
    c = oit_hybrid_color(oit_accum_at(p, sample), oit_reveal_at(p, sample), oit_front_at(p, sample));
  }
  if (c.a <= 0.0) { discard; }
  return c;
}
