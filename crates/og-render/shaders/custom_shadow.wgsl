// A full custom shader's shadow casters (custom.rs), after
// custom_common.wgsl and the shader's code, under shadow_pass.wgsl: the
// hooked casters' entry names (`vs_hooked`, `vs_skinned_hooked`,
// `fs_hooked`), so every cascade, spot, cube-face and paraboloid pass
// draws the mesh where its vertex entry put it and drops what its
// fragment entry cuts.

fn camera() -> Camera {
  var c: Camera;
  c.view_proj = shadow_pass.view_proj;
  c.position = shadow_pass.light.xyz;
  c.forward = vec3<f32>(0.0);
  c.time = shadow_pass.time.x;
  return c;
}

fn project(world: vec3<f32>) -> vec4<f32> {
  return shadow_clip(vec4<f32>(world, 1.0));
}

@vertex
fn vs_hooked(v: CustomVertexIn, i: CustomInstanceIn) -> CustomOut {
  return custom_plain(v, i);
}

@vertex
fn vs_skinned_hooked(@builtin(vertex_index) vi: u32, v: CustomVertexIn, i: CustomSkinnedIn, s: CustomSkinIn) -> CustomOut {
  return custom_skinned(vi, v, i, s);
}

@fragment
fn fs_hooked(in: CustomOut) -> @location(0) vec4<f32> {
  let dx = dpdx(in.world);
  let dy = dpdy(in.world);
  let cut = custom_cut(custom_surface(in, vec3<f32>(0.0)));
  if (cut) { discard; }
  if (shadow_pass.light.w > 0.0) {
    return vec4<f32>(local_distance(in.world, in.clip, dx, dy), 0.0, 0.0, 1.0);
  }
  return vec4<f32>(in.clip.z, 0.0, 0.0, 1.0);
}
