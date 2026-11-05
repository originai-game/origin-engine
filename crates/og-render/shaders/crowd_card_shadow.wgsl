// Crowd impostor cards in the shadow passes (crowd_card_common.wgsl): the
// card faces the light around the vertical (a cascade's direction, or a
// local light's position) and cuts where the atlas's coverage does,
// storing the depth of the surface it stands for. Prepended as
// impostor_shadow.wgsl is, after it.

struct CardShadowOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world: vec3<f32>,
  @location(1) local: vec3<f32>,
  @location(2) @interpolate(flat) cols: vec2<f32>,
  @location(3) @interpolate(flat) rows: vec2<f32>,
  // x: the second column's weight, y: the second frame's, z: the sphere's
  // radius in the world
  @location(4) @interpolate(flat) blend: vec3<f32>,
  @location(5) @interpolate(flat) toward: vec3<f32>,
};

@vertex
fn vs_crowd_card_shadow(@builtin(vertex_index) vertex: u32, i: CardIn) -> CardShadowOut {
  let model = instance_matrix(i.m0, i.m1, i.m2, i.m3);
  let local = shadow_pass.light.w > 0.0;
  let toward = select(scatter.impostor_light.xyz, shadow_pass.light.xyz, local);
  let card = card_make(model, vertex, toward, !local, i.look.x);
  var o: CardShadowOut;
  o.clip = shadow_project(card.world, mul_point(shadow_pass.view_proj, card.world));
  o.world = card.world.xyz;
  o.local = card.local;
  o.cols = card.cols;
  o.rows = card.rows;
  o.blend = vec3<f32>(card.col_w, card.row_w, card.extent);
  o.toward = card.toward;
  return o;
}

struct CardShadowFrag {
  @location(0) value: vec4<f32>,
  @builtin(frag_depth) depth: f32,
};

@fragment
fn fs_crowd_card_shadow(in: CardShadowOut) -> CardShadowFrag {
  card_set_level(in.local);
  let dx = dpdxFine(in.world);
  let dy = dpdyFine(in.world);
  let s = card_sample(in.local, in.cols, in.blend.x, in.rows, in.blend.y);
  if (s.coverage < 0.5) {
    discard;
  }
  let surface = in.world + in.toward * s.depth * in.blend.z;
  var o: CardShadowFrag;
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
