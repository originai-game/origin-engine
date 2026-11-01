fn material(m: MaterialInput, p: Params) -> vec4<f32> {
  let c = layer(m.uv);
  let l = dot(c.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
  let wave = 0.5 + 0.5 * sin(m.pixel.y * 0.35 - m.time * 4.0);
  let tone = mix(p.dark, p.light, clamp(l * 1.6, 0.0, 1.0));
  return vec4<f32>(tone * (1.0 - p.band + p.band * wave), c.a);
}
