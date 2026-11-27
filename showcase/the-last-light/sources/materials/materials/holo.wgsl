// Holo (holo.ogt): fine iridescent stripes, many periods across the
// element, whose hue shifts with the position across it (the view angle a
// flat card cannot have) and the clock. They are laid on at low strength as
// a hue shift and a thin bright line, weighted towards the art, so the art
// keeps its own colour and the frame stays close to its base.
fn holo_hue(h: f32) -> vec3<f32> {
  let k = vec3<f32>(0.0, 2.0943951, 4.1887902);
  return 0.5 + 0.5 * cos(6.2831853 * h + k);
}

fn material(m: MaterialInput, p: Params) -> vec4<f32> {
  let c = layer(m.uv);
  let s = max(p.scale, 2.0);
  let light = dot(c.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
  let art = smoothstep(0.05, 0.5, light);
  let stripe = (m.pixel.x * 0.94 + m.pixel.y * 0.34) / s;
  let angle = 1.3 * (m.uv.x - m.uv.y) + m.time * p.speed;
  let h = fract(stripe * 0.5 + angle + 0.15 * value_noise(m.pixel / (6.0 * s)));
  let iri = mix(vec3<f32>(0.5), holo_hue(h), clamp(p.saturation, 0.0, 1.0));
  // A thin bright line once a period.
  let line = pow(0.5 + 0.5 * cos(6.2831853 * stripe), 8.0);
  let amt = clamp(p.amount, 0.0, 1.0) * (0.12 + 0.88 * art);
  let shift = (iri - vec3<f32>(dot(iri, vec3<f32>(0.3333)))) * 1.2;
  let rgb = c.rgb * (1.0 + shift * amt) + iri * line * amt * 0.2;
  return vec4<f32>(rgb, c.a);
}
