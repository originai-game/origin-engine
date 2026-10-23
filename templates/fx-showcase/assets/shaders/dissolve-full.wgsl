// A full custom shader: the vertex entry places the mesh (standard_vertex,
// with a slow sway so the shadows follow the vertex stage too), the
// fragment entry starts from the standard surface (the material's colour,
// maps and roughness) and cuts it by noise: alpha 0 is discarded in the
// colour pass, the prepass and every shadow map. A glowing band runs along
// the cut. noise3 is the contract's 3D value noise (0..1).
fn vertex(v: VertexInput, model: mat4x4<f32>, p: Params) -> VertexOutput {
  var o = standard_vertex(v, model);
  o.world.x += 0.05 * sin(camera().time * 1.5 + v.position.y * 2.0);
  return o;
}

fn fragment(f: FragmentInput, p: Params) -> SurfaceOutput {
  var s = standard_surface(f);
  let n = 0.65 * noise3(f.local * p.scale) + 0.35 * noise3(f.local * p.scale * 2.7 + vec3<f32>(5.1, 1.3, 8.2));
  let edge = 1.0 - clamp((n - p.amount) / max(p.edge, 1e-4), 0.0, 1.0);
  s.color.a = select(1.0, 0.0, n < p.amount);
  s.emissive += p.glow.rgb * edge * edge * step(0.001, p.amount);
  return s;
}
