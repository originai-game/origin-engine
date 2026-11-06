// Holo (holo.ogt): the hue walks a diagonal plus a slow noise warp, and
// takes the art's lightness, so the art keeps its forms under the colour.
fn holo_hue(h: f32) -> vec3<f32> {
  let k = vec3<f32>(0.0, 2.0943951, 4.1887902);
  return 0.5 + 0.5 * cos(6.2831853 * h + k);
}

fn material(m: MaterialInput, p: Params) -> vec4<f32> {
  let c = layer(m.uv);
  let s = max(p.scale, 1.0);
  let warp = value_noise(m.pixel / (0.6 * s)) * 0.35;
  let h = fract((m.pixel.x + m.pixel.y) / s + warp + m.time * p.speed);
  let light = dot(c.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
  let rainbow = mix(vec3<f32>(1.0), holo_hue(h), clamp(p.saturation, 0.0, 1.0));
  let rgb = mix(c.rgb, rainbow * (0.15 + 1.1 * light), clamp(p.amount, 0.0, 1.0));
  return vec4<f32>(rgb, c.a);
}
