// Rarity glow (rarity.ogt): the distance to the layer's transparent edge,
// estimated from eight taps of its alpha at the glow's reach, fades the
// tier's colour in from the edge; a rim at the very edge, a breath by the
// clock.
fn material(m: MaterialInput, p: Params) -> vec4<f32> {
  let c = layer(m.uv);
  let r = max(p.size, 1.0);
  let px = layer_texel();
  var inside = 0.0;
  var near = 0.0;
  for (var i = 0; i < 8; i = i + 1) {
    let a = f32(i) * 0.7853982;
    let d = vec2<f32>(cos(a), sin(a));
    inside = inside + layer(m.uv + d * px * r).a;
    near = near + layer(m.uv + d * px * 1.5).a;
  }
  // Toward 1 at the edge (taps outside the art or the rectangle), 0 deep inside.
  let edge_uv = min(m.uv, 1.0 - m.uv) * m.size;
  let border = 1.0 - smoothstep(0.0, r, min(edge_uv.x, edge_uv.y));
  let glow = max(1.0 - inside / 8.0, border);
  let rim = max(1.0 - near / 8.0, 1.0 - smoothstep(0.0, 1.5, min(edge_uv.x, edge_uv.y)));
  let breath = 0.75 + 0.25 * sin(m.time * p.pulse * 6.2831853);
  let k = clamp(p.strength * breath, 0.0, 2.0);
  let rgb = c.rgb + p.color * (glow * glow * 0.8 + rim * 0.6) * k;
  return vec4<f32>(rgb, c.a);
}
