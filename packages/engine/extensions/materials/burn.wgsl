// Burn (burn.ogt): two octaves of value noise against the progress; the
// edge band is charred (darkened) behind the flame and the flame colour in
// front, the hole beyond it.
fn material(m: MaterialInput, p: Params) -> vec4<f32> {
  let c = layer(m.uv);
  let s = max(p.scale, 1.0);
  let n = 0.7 * value_noise(m.pixel / s) + 0.3 * value_noise(m.pixel / (0.4 * s) + 11.0);
  let t = clamp(p.progress, 0.0, 1.0) * 1.04 - 0.02;
  let e = max(p.edge, 1e-3);
  let keep = smoothstep(t - 0.005, t + 0.005, n);
  let flame = 1.0 - smoothstep(t, t + e * 0.5, n);
  let charred = 1.0 - smoothstep(t + e * 0.5, t + e, n);
  let on = step(0.0001, p.progress);
  var rgb = c.rgb * (1.0 - 0.75 * charred * on);
  rgb = mix(rgb, p.glow, flame * on);
  return vec4<f32>(rgb, c.a * keep);
}
