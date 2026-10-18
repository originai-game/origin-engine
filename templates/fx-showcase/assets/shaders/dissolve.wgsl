// Dissolve by noise: fragments whose noise is below `amount` get alpha 0
// (the material's alphaTest discards them); a glowing band of
// `edgeWidth` in `edgeColor` runs along the edge. value_noise3 is the
// engine's 3D value noise (0..1).
fn dissolve_value(h: Hook, p: Params) -> f32 {
  let q = h.local * p.scale;
  return 0.6 * value_noise3(q, 11u) + 0.4 * value_noise3(q * 2.3 + vec3<f32>(3.1, 7.7, 1.9), 29u);
}

fn surface_color(h: Hook, p: Params, c: vec4<f32>) -> vec4<f32> {
  return vec4<f32>(c.rgb, select(c.a, 0.0, dissolve_value(h, p) < p.amount));
}

fn surface_emissive(h: Hook, p: Params, e: vec3<f32>) -> vec3<f32> {
  let n = dissolve_value(h, p);
  let edge = 1.0 - clamp((n - p.amount) / max(p.edgeWidth, 1e-4), 0.0, 1.0);
  return e + p.edgeColor * edge * edge * step(0.001, p.amount);
}
