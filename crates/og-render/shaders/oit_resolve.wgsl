// Order-independent transparency's resolve (oit.rs): a full-screen pass
// over the opaque scene, before the additive effects, the sprites and the
// post chain (TAA reads the revealage as its reactive mask). After
// fullscreen.wgsl and the bindings oit.rs `resolve_source` declares:
// `oit_accum_at` and `oit_reveal_at` (each sample's weighted sums), and
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

// Premultiplied, blended One / OneMinusSrcAlpha over the scene.
fn oit_resolve(p: vec2<i32>, sample: u32) -> vec4<f32> {
  var c = oit_exact(p, sample);
  if (c.a < 0.0) {
    c = oit_weighted_color(oit_accum_at(p, sample), oit_reveal_at(p, sample));
  }
  if (c.a <= 0.0) { discard; }
  return c;
}
