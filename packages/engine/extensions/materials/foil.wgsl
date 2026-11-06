// Foil (foil.ogt): one broad, soft sheen band that sweeps across the
// element with the clock and the game's tilt, over a fine metal grain and
// sparse glints. The grain shows mostly where the art is bright, and the
// glints catch light only under the band, so dark ink stays ink and the
// frame stays close to its base.
fn material(m: MaterialInput, p: Params) -> vec4<f32> {
  let c = layer(m.uv);
  let light = dot(c.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
  let art = smoothstep(0.05, 0.5, light);
  // The band: a Gaussian across a diagonal, its centre swept from corner to
  // corner and back (sweep.x cycles a second), offset by the tilt (sweep.y).
  let q = dot(m.uv - 0.5, vec2<f32>(0.7071, 0.7071));
  let centre = 0.6 * sin(m.time * p.sweep.x * 6.2831853) + p.sweep.y;
  let band = exp(-pow((q - centre * 0.75) / 0.2, 2.0));
  // Fine grain: two octaves of value noise at `grain` CSS px, low contrast.
  let g = max(p.grain, 0.5);
  let grain = 0.6 * value_noise(m.pixel / g) + 0.4 * value_noise(m.pixel / (0.5 * g) + 7.0) - 0.5;
  // Sparse glints, one cell in a few dozen, lit under the band.
  let cell = m.pixel / (2.0 * g);
  let r = length(fract(cell) - 0.5);
  let glint = pow(hash12(floor(cell)), 120.0) * (1.0 - smoothstep(0.1, 0.45, r));
  let amt = clamp(p.amount, 0.0, 1.0);
  var rgb = c.rgb * (1.0 + grain * 0.22 * amt * (0.3 + 0.7 * art));
  rgb = rgb + p.tint * band * amt * (0.18 + 0.55 * art);
  rgb = rgb + p.tint * glint * band * amt * 1.5 * art;
  return vec4<f32>(rgb, c.a);
}
