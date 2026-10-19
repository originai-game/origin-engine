// A ripple travelling up a body: each vertex moves along its normal by a
// wave of `bulge` metres up the object's height, and the whole mesh by
// `offset`; bands of `stripe` are painted up it. On the skinned walker it
// runs before the pose is drawn, and its shadow and depth follow.
fn vertex_offset(h: Hook, p: Params) -> vec3<f32> {
  let wave = sin(h.local.y * 9.0 - h.time * 4.0) * p.bulge;
  return h.normal * wave + p.offset;
}

fn surface_color(h: Hook, p: Params, c: vec4<f32>) -> vec4<f32> {
  let band = step(0.5, fract(h.local.y * 2.5 - h.time * 0.5));
  return vec4<f32>(mix(c.rgb, p.stripe, band * 0.7), c.a);
}
