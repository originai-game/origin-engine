// Impostors in the shadow passes (impostor.rs): the card faces the light
// (a cascade's direction, or a local light's position) and cuts where the
// blended frames' coverage does. Prepended as scatter_shadow.wgsl is, then
// impostor_common.wgsl.

struct ImpostorShadowOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world: vec3<f32>,
  @location(1) uv01: vec4<f32>,
  @location(2) uv2: vec2<f32>,
  @location(3) @interpolate(flat) frames: vec3<u32>,
  @location(4) @interpolate(flat) weights: vec3<f32>,
  @location(5) @interpolate(flat) fade: f32,
};

@vertex
fn vs_impostor_shadow(@builtin(vertex_index) vertex: u32, i: ScatterInstance) -> ImpostorShadowOut {
  let model = instance_matrix(i.m0, i.m1, i.m2, i.m3);
  let local = shadow_pass.light.w > 0.0;
  let toward = select(scatter.impostor_light.xyz, shadow_pass.light.xyz, local);
  let card = imp_card(model, vertex, toward, !local);
  var o: ImpostorShadowOut;
  let near = select(1.0, 1.0 - scatter_near(i.m3), scatter.impostor_grid.y > 0.0);
  o.fade = scatter_fade(i.m3) * near;
  o.clip = select(SCATTER_GONE, shadow_project(card.world, mul_point(shadow_pass.view_proj, card.world)), o.fade > 0.0);
  o.world = card.world.xyz;
  o.uv01 = card.uv01;
  o.uv2 = card.uv2;
  o.frames = card.frames;
  o.weights = card.weights;
  return o;
}

@fragment
fn fs_impostor_shadow(in: ImpostorShadowOut) -> @location(0) vec4<f32> {
  let dx = dpdx(in.world);
  let dy = dpdy(in.world);
  let cover = imp_coverage(in.uv01, in.uv2, in.frames, in.weights);
  if (cover.alpha < imp_cutoff() || scatter_dropped(in.clip.xy, in.fade)) {
    discard;
  }
  if (shadow_pass.light.w > 0.0) {
    return vec4<f32>(local_distance(in.world, in.clip, dx, dy), 0.0, 0.0, 1.0);
  }
  return vec4<f32>(in.clip.z, 0.0, 0.0, 1.0);
}
