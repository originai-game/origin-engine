// Foil (foil.ogt). Two interfering band sets at crossing angles give the
// moving interference of a foil stamp; the sheen follows the art's own
// lightness so dark ink stays ink.
fn material(m: MaterialInput, p: Params) -> vec4<f32> {
  let c = layer(m.uv);
  let s = max(p.scale, 1.0);
  let t = m.time * p.speed;
  let a = sin((m.pixel.x * 0.8 + m.pixel.y) / s * 6.2831853 + t * 6.2831853);
  let b = sin((m.pixel.x - m.pixel.y * 0.6) / (s * 1.7) * 6.2831853 - t * 4.0);
  // Narrow bright streaks where the two sets agree, little between them.
  let band = pow(0.5 + 0.25 * (a + b), 6.0);
  let light = dot(c.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
  let sheen = p.tint * band * (0.1 + 0.9 * sqrt(light));
  let rgb = mix(c.rgb, c.rgb * (0.9 + 0.2 * p.tint) + sheen, clamp(p.amount, 0.0, 1.0));
  return vec4<f32>(rgb, c.a);
}
