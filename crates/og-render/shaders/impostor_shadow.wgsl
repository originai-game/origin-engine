// Impostors in the shadow passes (impostor.rs): the card faces the light
// (a cascade's direction, or a local light's position) and cuts where the
// blended frames' coverage does, sampled as the lit pass samples them. Prepended as scatter_shadow.wgsl is, then
// impostor_common.wgsl.

struct ImpostorShadowOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world: vec3<f32>,
  @location(1) uv01: vec4<f32>,
  @location(2) uv2: vec2<f32>,
  @location(3) @interpolate(flat) frames: vec3<u32>,
  @location(4) @interpolate(flat) weights: vec3<f32>,
  @location(5) @interpolate(flat) fade: vec2<f32>,
  @location(6) @interpolate(flat) par01: vec4<f32>,
  @location(7) @interpolate(flat) par2: vec2<f32>,
  // Towards the light from the copy's centre (the frames' depth axis) and
  // the model's sphere radius in the world.
  @location(8) @interpolate(flat) toward: vec3<f32>,
  @location(9) @interpolate(flat) extent: f32,
};

@vertex
fn vs_impostor_shadow(@builtin(vertex_index) vertex: u32, i: ScatterInstance) -> ImpostorShadowOut {
  let model = instance_matrix(i.m0, i.m1, i.m2, i.m3);
  let local = shadow_pass.light.w > 0.0;
  let toward = select(scatter.impostor_light.xyz, shadow_pass.light.xyz, local);
  let card = imp_card(model, vertex, toward, !local);
  let centre = mul_point(model, vec4<f32>(scatter.impostor.xyz, 1.0)).xyz;
  var o: ImpostorShadowOut;
  o.fade = imp_fade(i.m3);
  o.clip = select(SCATTER_GONE, shadow_project(card.world, mul_point(shadow_pass.view_proj, card.world)), o.fade.y > o.fade.x);
  o.world = card.world.xyz;
  o.uv01 = card.uv01;
  o.uv2 = card.uv2;
  o.par01 = card.par01;
  o.par2 = card.par2;
  o.frames = card.frames;
  o.weights = card.weights;
  o.toward = normalize(select(toward, toward - centre, local) + vec3<f32>(0.0, 1e-9, 0.0));
  o.extent = card.extent;
  return o;
}

struct ImpostorShadowFrag {
  @location(0) value: vec4<f32>,
  @builtin(frag_depth) depth: f32,
};

// The card stores the depth of the surface it stands for, not its own
// plane's through the model's centre: otherwise every point the lit card
// shows behind that plane (a palm's drooping fronds) shadows itself.
@fragment
fn fs_impostor_shadow(in: ImpostorShadowOut) -> ImpostorShadowFrag {
  imp_set_level(in.uv01.xy);
  let dx = dpdx(in.world);
  let dy = dpdy(in.world);
  let uvs = imp_uvs(in.uv01, in.uv2, in.par01, in.par2, in.frames);
  let cover = imp_coverage(uvs.uv01, uvs.uv2, in.frames, in.weights);
  let surface = in.world + in.toward * imp_blended_depth(cover) * in.extent;
  if (cover.alpha < imp_cutoff() || imp_dropped(in.clip.xy, in.fade.x, in.fade.y)) {
    discard;
  }
  var o: ImpostorShadowFrag;
  if (shadow_pass.paraboloid.x > 0.5) {
    o.depth = clamp(length(surface - shadow_pass.light.xyz) / shadow_pass.light.w, 0.0, 1.0);
  } else {
    let c = mul_point(shadow_pass.view_proj, vec4<f32>(surface, 1.0));
    o.depth = clamp(c.z / c.w, 0.0, 1.0);
  }
  if (shadow_pass.light.w > 0.0) {
    o.value = vec4<f32>(local_distance(surface, in.clip, dx, dy), 0.0, 0.0, 1.0);
  } else {
    o.value = vec4<f32>(o.depth, 0.0, 0.0, 1.0);
  }
  return o;
}
