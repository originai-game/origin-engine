// Impostors in the shadow passes (impostor.rs): the card faces the light
// (a cascade's direction, or a local light's position) and cuts where the
// blended frames' coverage does, sampled as the lit pass samples them. Prepended as scatter_shadow.wgsl is, then
// impostor_common.wgsl.

struct ImpostorShadowOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world: vec3<f32>,
  @location(1) uv01: vec4<f32>,
  @location(2) uv2: vec2<f32>,
  @location(3) @interpolate(flat, either) frames: vec3<u32>,
  @location(4) @interpolate(flat, either) weights: vec3<f32>,
  @location(5) @interpolate(flat, either) fade_extent: vec2<f32>,
  @location(6) @interpolate(flat, either) par01: vec4<f32>,
  @location(7) @interpolate(flat, either) par2: vec2<f32>,
  // Towards the light from the copy's centre (the frames' depth axis) and
  // the copy's root position for seasonal leaf drop.
  @location(8) @interpolate(flat, either) toward: vec3<f32>,
  @location(9) @interpolate(flat, either) root_pos: vec3<f32>,
  // imp_depths of the card's point (pd linear over the card).
  @location(10) pd: vec3<f32>,
  @location(11) @interpolate(flat, either) vd: vec3<f32>,
  // Crown-local offset from model centre and local light direction for seasonal cut.
  @location(12) local_pos: vec3<f32>,
  @location(13) @interpolate(flat, either) local_toward: vec3<f32>,
};

@vertex
fn vs_impostor_shadow(@builtin(vertex_index) vertex: u32, copy: ScatterInstance) -> ImpostorShadowOut {
  let i = scatter_placed(copy);
  let size = imp_scale(i.m3);
  // Kept invertible when gone (the card is culled then).
  let model = scatter_shrunk(instance_matrix(i.m0, i.m1, i.m2, i.m3), max(size, 1e-4));
  let local = shadow_pass.light.w > 0.0;
  let toward = select(scatter.impostor_light.xyz, shadow_pass.light.xyz, local);
  let card = imp_card(model, vertex, toward, !local);
  let centre = mul_point(model, vec4<f32>(scatter.impostor.xyz, 1.0)).xyz;
  var o: ImpostorShadowOut;
  let fade = imp_hidden(i);
  o.fade_extent = vec2<f32>(fade, card.extent);
  let world = card.world + vec4<f32>(imp_sway(card, i, size, false), 0.0);
  o.clip = select(SCATTER_GONE, shadow_project(world, mul_point(shadow_pass.view_proj, world)), size > 0.0 && abs(fade) < 1.0);
  o.world = world.xyz;
  o.uv01 = card.uv01;
  o.uv2 = card.uv2;
  o.par01 = card.par01;
  o.par2 = card.par2;
  o.frames = card.frames;
  o.weights = card.weights;
  // The card's own view of the light (imp_view), in the world.
  let m3 = mat3x3<f32>(normalize(model[0].xyz), normalize(model[1].xyz), normalize(model[2].xyz));
  let to_light = normalize(select(toward, toward - centre, local) + vec3<f32>(0.0, 1e-9, 0.0));
  let seen = transpose(m3) * to_light;
  o.toward = to_light + m3 * (imp_view(seen) - seen);
  o.root_pos = i.m3.xyz;
  o.pd = card.pd;
  o.vd = card.vd;
  o.local_pos = transpose(m3) * (card.world.xyz - centre);
  o.local_toward = transpose(m3) * o.toward;
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
fn fs_impostor_shadow(in: ImpostorShadowOut, @builtin(front_facing) front: bool) -> ImpostorShadowFrag {
  imp_set_level(in.uv01.xy);
  let dx = dpdxFine(in.world);
  let dy = dpdyFine(in.world);
  var dd: ImpDepths;
  dd.pd = in.pd;
  dd.vd = in.vd;
  // A card read at a coarse level of its atlases (a far cascade's, where it
  // covers a few texels) takes its frames where its point projects: their
  // parallax moves the sample by less than the level's texel. A finer one
  // takes each frame's parallax step (imp_parallax).
  var uvs = ImpUvs(in.uv01, in.uv2, in.weights);
  if (imp_level < IMP_MARCH_LEVEL) {
    uvs = imp_uvs(in.uv01, in.uv2, in.par01, in.par2, in.frames, dd, in.weights, false);
  }
  let cover = imp_coverage(uvs.uv01, uvs.uv2, in.frames, uvs.weights);
  let step = imp_blended_step(cover, dd) * in.fade_extent.y;
  let surface = in.world + in.toward * step;
  var leaf_cut = false;
  if (scatter.card_leaves[0].y > 0.0 && scatter.leaf_year.fall.w >= 0.5 && scatter.impostor_grid.w >= 0.5) {
    let total = max(cover.wa.x + cover.wa.y + cover.wa.z, 1e-5);
    let translucency = (
      textureSampleLevel(roughness_tex, material_sampler, cover.a0, imp_level).g * cover.wa.x
      + textureSampleLevel(roughness_tex, material_sampler, cover.a1, imp_level).g * cover.wa.y
      + textureSampleLevel(roughness_tex, material_sampler, cover.a2, imp_level).g * cover.wa.z
    ) / total;
    if (translucency > 0.0) {
      let p = in.local_pos + in.local_toward * step;
      let leaf = crown_cell_leaf(scatter.card_leaves[0], p, in.root_pos + scatter_origin(), scatter.leaf_year);
      leaf_cut = dither_cut(1.0 - leaf.leaf.kept, in.clip.xy, shadow_dither_turn());
    }
  }
  let cut = cover.alpha < imp_cutoff() || dither_cut(in.fade_extent.x, in.clip.xy, shadow_dither_turn()) || leaf_cut;
  let c = shadow_card_cut(surface, mul_point(shadow_pass.view_proj, vec4<f32>(surface, 1.0)), in.clip, dx, dy, front, cut);
  var o: ImpostorShadowFrag;
  o.depth = c.depth;
  o.value = c.value;
  return o;
}
