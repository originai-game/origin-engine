// An eye (physical.rs `irisRadius`): the iris is a disc under the cornea,
// a clear dome over it filled with the aqueous humour, so it is seen
// refracted: it lies deeper than the surface that is drawn, shifts
// against the white of the eye as the eye turns, and stays in view from
// the side where a painted iris would thin to a sliver (Jimenez, Next
// Generation Character Rendering, GDC 2013). The cornea's own reflection
// is the material's clear coat.

// Where the base colour is read for the surface at `uv`: inside the iris
// (within `radius` of the centre of the uv square) the view ray is bent
// into the cornea (`ior`) and followed down to the iris, which lies
// `depth` radii under the cornea's top and meets it at the rim; outside
// it, `uv`. `t` and `bt` are the tangent frame (the bitangent up the
// image), `n` the surface's normal towards the eye `v`.
fn iris_uv(uv: vec2<f32>, t: vec3<f32>, bt: vec3<f32>, n: vec3<f32>, v: vec3<f32>, ior: f32, radius: f32, depth: f32) -> vec2<f32> {
  let c = (uv - vec2<f32>(0.5)) / radius;
  let r2 = dot(c, c);
  if (r2 >= 1.0) { return uv; }
  let ray = refract(-v, n, 1.0 / ior);
  // The cornea's height over the iris here: a dome, none at the rim.
  let height = radius * depth * (1.0 - r2);
  return uv + vec2<f32>(dot(ray, t), -dot(ray, bt)) / max(-dot(ray, n), 0.2) * height;
}
