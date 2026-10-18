// Wind-swayed grass: each blade bends along `wind` (direction times
// strength in metres), more towards its tip (local y from `base` at the
// root to `base + height`), with gusts that travel across the field; the
// tips are tinted lighter.
fn vertex_offset(h: Hook, p: Params) -> vec3<f32> {
  let t = clamp((h.local.y - p.base) / max(p.height, 1e-3), 0.0, 1.0);
  let phase = dot(h.world.xz, vec2<f32>(0.7, 0.4)) * p.waveScale - h.time * p.speed;
  let gust = 0.65 + 0.35 * sin(phase) + 0.15 * sin(phase * 2.7 + 1.3);
  let bend = t * t * gust;
  return vec3<f32>(p.wind.x * bend, -0.25 * length(p.wind) * bend * t, p.wind.y * bend);
}

fn surface_color(h: Hook, p: Params, c: vec4<f32>) -> vec4<f32> {
  let t = clamp((h.local.y - p.base) / max(p.height, 1e-3), 0.0, 1.0);
  return vec4<f32>(mix(c.rgb * 0.55, p.tip, t * t), c.a);
}
