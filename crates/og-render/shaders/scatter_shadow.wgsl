// The scattered copies in the shadow passes: depth from a cascade, or
// distance / range from a local light, like shadow.wgsl, with the copies'
// fade dithered so their shadows thin out with them. Prepended by
// noise.wgsl, material.wgsl (group 1: the alpha cut-off),
// scatter_common.wgsl (group 2) and shadow_pass.wgsl (group 0, the shadow
// pass's slot).

struct ScatterShadowOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world: vec3<f32>,
  @location(1) uv: vec2<f32>,
  @location(2) @interpolate(flat) fade: f32,
};

@vertex
fn vs_shadow(v: ScatterVertex, i: ScatterInstance) -> ScatterShadowOut {
  let model = instance_matrix(i.m0, i.m1, i.m2, i.m3);
  let world = model * vec4<f32>(v.position, 1.0);
  var o: ScatterShadowOut;
  o.fade = scatter_fade(i.m3);
  o.clip = select(SCATTER_GONE, shadow_project(world, mul_point(shadow_pass.view_proj, world)), o.fade > 0.0);
  o.world = world.xyz;
  o.uv = v.uv;
  return o;
}

@fragment
fn fs_shadow(in: ScatterShadowOut) -> @location(0) vec4<f32> {
  let dx = dpdx(in.world);
  let dy = dpdy(in.world);
  let a = material.base_color.a * textureSample(base_color_tex, material_sampler, in.uv).a;
  if ((material.emissive.w >= 0.0 && a < material.emissive.w) || scatter_dropped(in.clip.xy, in.fade)) {
    discard;
  }
  if (shadow_pass.light.w > 0.0) {
    return vec4<f32>(local_distance(in.world, in.clip, dx, dy), 0.0, 0.0, 1.0);
  }
  return vec4<f32>(in.clip.z, 0.0, 0.0, 1.0);
}
