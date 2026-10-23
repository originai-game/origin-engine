// A CRT: the image bent outwards from the centre (black past its edges),
// scanlines across the bent image, a shadow mask favouring red, green or
// blue by pixel column, and darker rounded corners. Linear HDR in and out.
fn post(f: PostInput, p: Params) -> vec4<f32> {
  let c = f.uv * 2.0 - 1.0;
  let uv = c * (1.0 + p.curve * dot(c, c)) * 0.5 + 0.5;
  if (any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0))) {
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
  }
  var color = previous(uv).rgb;
  let line = 0.5 + 0.5 * cos(uv.y * p.lines * 6.2831853);
  color *= 1.0 - p.scanlines * line;
  var mask = vec3<f32>(0.88);
  mask[u32(f.pixel.x) % 3u] = 1.12;
  color *= mask;
  let edge = uv * (1.0 - uv);
  color *= pow(clamp(edge.x * edge.y * 16.0, 0.0, 1.0), p.vignette * 0.5);
  return vec4<f32>(color, 1.0);
}
