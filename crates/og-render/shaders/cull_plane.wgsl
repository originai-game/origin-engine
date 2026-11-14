// The plane test every cull shares (the instances', the crowds', the
// clusters' and the scattered copies'): whether what is centred on `c`
// and reaches `reach` towards a plane lies inside it or across it.
// Inside is dot(plane.xyz, p) + plane.w >= 0; `reach` is in the units of
// the plane's normal (a caller whose normal is not a unit vector scales it
// by the normal's length).
fn plane_keeps(plane: vec4<f32>, c: vec3<f32>, reach: f32) -> bool {
  return dot(plane.xyz, c) + plane.w >= -reach;
}
