// Burn (burn.ogt). Two octaves of value noise against the progress give the
// front. Its distance in CSS px comes from the noise's gradient, measured
// by finite difference, so the rim keeps the same width wherever the noise
// is steep or flat. Going from the paper into the hole, the rim grades from
// charred brown through orange ember to white-hot at the edge, and a glow
// bleeds a few px into the hole. Everything is clipped to the element's own
// alpha (the card's rounded shape, as drawn), so neither the flame nor the
// glow ever draws outside it.
fn burn_noise(px: vec2<f32>, s: f32) -> f32 {
  return 0.7 * value_noise(px / s) + 0.3 * value_noise(px / (0.4 * s) + 11.0);
}

fn material(m: MaterialInput, p: Params) -> vec4<f32> {
  let c = layer(m.uv);
  let s = max(p.scale, 1.0);
  let n = burn_noise(m.pixel, s);
  let gx = burn_noise(m.pixel + vec2<f32>(1.0, 0.0), s) - n;
  let gy = burn_noise(m.pixel + vec2<f32>(0.0, 1.0), s) - n;
  let t = clamp(p.progress, 0.0, 1.0) * 1.04 - 0.02;
  // Signed distance to the front, CSS px: above 0 on the paper, below in the hole.
  let d = (n - t) / max(length(vec2<f32>(gx, gy)), 1e-4);
  let w = max(p.edge, 0.5);
  let on = step(0.0001, p.progress);
  // The paper's rim, from the edge (0) to unburnt (1) across w px.
  let k = clamp(d / w, 0.0, 1.0);
  let ember = p.glow;
  let hot = mix(ember, vec3<f32>(1.0, 0.96, 0.82) * max(max(ember.r, ember.g), 1.0), 0.8);
  let char = vec3<f32>(0.07, 0.03, 0.012);
  var rim = mix(hot, ember, smoothstep(0.0, 0.3, k));
  rim = mix(rim, char, smoothstep(0.3, 0.65, k));
  // Past the char, the paper scorches: its own colour darkened, then whole.
  let paper = c.rgb * mix(0.3, 1.0, smoothstep(0.65, 1.0, k));
  let rgb_paper = mix(rim, paper, smoothstep(0.55, 0.75, k));
  // The glow in the hole: opaque white-hot at the front (so its edge has no
  // dark seam), cooling to the ember and fading out over half the rim's width.
  let out = clamp(-d / (0.5 * w), 0.0, 1.0);
  let glow_rgb = mix(hot, ember, smoothstep(0.0, 0.5, out));
  let glow_a = 1.0 - smoothstep(0.0, 1.0, sqrt(out));
  let in_paper = step(0.0, d);
  let rgb = mix(glow_rgb, rgb_paper, in_paper);
  let a = c.a * mix(glow_a, 1.0, in_paper);
  if (p.progress <= 0.0) {
    return c;
  }
  return vec4<f32>(mix(c.rgb, rgb, on), a);
}
