// Crowd impostor cards' lit and prepass passes (crowd_card_common.wgsl).
// Prepended as impostor.wgsl is, after it. A card writes its surface's
// depth (the atlas's), not its own plane's, and is lit as a mesh member:
// its part's colour (its crowd's material) times the atlas's detail and
// the member's tint, the sun through the shadow cascades (looked up past
// its sphere, as a model impostor's: the card takes no shadow of its own),
// the image-based and local lights, fog and the member's flash. Its edge
// is alpha to coverage under MSAA, the cut without.

struct CardOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world: vec3<f32>,
  @location(1) local: vec3<f32>,
  @location(2) @interpolate(flat) cols: vec2<f32>,
  @location(3) @interpolate(flat) rows: vec2<f32>,
  // x: the second column's weight, y: the second frame's, z: the sphere's
  // radius in the world
  @location(4) @interpolate(flat) blend: vec3<f32>,
  @location(5) @interpolate(flat) toward: vec3<f32>,
  // The member's turn (its columns, normalised) and its sphere's centre.
  @location(6) @interpolate(flat) m0: vec3<f32>,
  @location(7) @interpolate(flat) m1: vec3<f32>,
  @location(8) @interpolate(flat) m2: vec3<f32>,
  @location(9) @interpolate(flat) centre: vec3<f32>,
  @location(10) @interpolate(flat) look: vec4<f32>,
  @location(11) cur_clip: vec4<f32>,
  @location(12) prev_clip: vec4<f32>,
};

@vertex
fn vs_crowd_card(@builtin(vertex_index) vertex: u32, i: CardIn) -> CardOut {
  let model = instance_matrix(i.m0, i.m1, i.m2, i.m3);
  let card = card_make(model, vertex, frame.view_pos.xyz, false, i.look.x);
  var o: CardOut;
  o.clip = mul_point(frame.view_proj, card.world);
  o.world = card.world.xyz;
  o.local = card.local;
  o.cols = card.cols;
  o.rows = card.rows;
  o.blend = vec3<f32>(card.col_w, card.row_w, card.extent);
  o.toward = card.toward;
  o.m0 = normalize(i.m0);
  o.m1 = normalize(i.m1);
  o.m2 = normalize(i.m2);
  o.centre = mul_point(model, vec4<f32>(scatter.impostor.xyz, 1.0)).xyz;
  o.look = i.look;
  // The camera's motion alone: a far member walks well under a pixel a frame.
  o.cur_clip = mul_point(scatter.view_proj, card.world);
  o.prev_clip = mul_point(scatter.prev_view_proj, card.world);
  return o;
}

// The pixel's surface point: the card's moved along its facing by the depth.
fn card_surface(in: CardOut, s: CardSample) -> vec3<f32> {
  return in.world + in.toward * s.depth * in.blend.z;
}

fn card_depth(world: vec3<f32>) -> f32 {
  let c = mul_point(frame.view_proj, vec4<f32>(world, 1.0));
  return clamp(c.z / c.w, 0.0, 1.0);
}

struct CardLit {
  @location(0) color: vec4<f32>,
  @builtin(frag_depth) depth: f32,
};

@fragment
fn fs_crowd_card(in: CardOut) -> CardLit {
  card_set_level(in.local);
  let s = card_sample(in.local, in.cols, in.blend.x, in.rows, in.blend.y);
  let a2c = scatter.impostor_grid.z > 0.5;
  let coverage = clamp((s.coverage - 0.5) / max(fwidth(s.coverage), 1e-4) + 0.5, 0.0, 1.0);
  if (select(s.coverage < 0.5, coverage <= 0.0, a2c)) {
    discard;
  }
  let world = card_surface(in, s);
  var o: CardLit;
  o.depth = card_depth(world);
  let part = card_part(s.part);
  var albedo = part.rgb * s.detail;
  if (in.look.w > 0.0) { albedo *= vec3<f32>(1.0) - card_rgb(in.look.w); }
  if (frame.light_color.w > 0.5) {
    o.color = debug_unsupported(in.clip.xy);
    if (frame.light_color.w > 2.5 && frame.light_color.w < 3.5) { o.color = vec4<f32>(albedo, 1.0); }
    return o;
  }
  let m3 = mat3x3<f32>(in.m0, in.m1, in.m2);
  var surface: Surface;
  surface.metallic = 0.0;
  surface.roughness = clamp(part.a, 0.04, 1.0);
  surface.albedo = albedo;
  surface.f0 = vec3<f32>(0.04);
  let v = select(normalize(frame.view_pos.xyz - world), -frame.view_forward.xyz, frame.view_pos.w > 0.5);
  var n = normalize(m3 * s.normal);
  if (dot(n, v) < 0.0) { n = normalize(n - v * dot(n, v) * 1.01); }
  let l = normalize(frame.light_dir.xyz);
  let n_dot_l = max(dot(n, l), 0.0);
  // The cascades looked up past the member's sphere along the light, as
  // a model impostor's (impostor.wgsl): other members and the scene shadow
  // it, its own card does not.
  let rel = world - in.centre;
  let b = dot(rel, l);
  let leave = -b + sqrt(max(b * b - dot(rel, rel) + in.blend.z * in.blend.z, 0.0));
  var shadow = 1.0;
  if (frame.light_dir.w > 0.5 && in.look.x >= 0.0 && n_dot_l > 0.0) {
    shadow = cascade_shadow(world + l * (max(leave, 0.0) + 0.5 * in.blend.z), n, max(n_dot_l, 0.05));
  }
  var color = brdf(surface, n, v, l) * frame.light_color.rgb * shadow;
  color += surface.albedo * frame.light_color.rgb * n_dot_l * shadow * caustics(world);
  color += surface_local_lights(in.clip.xy, surface, world, n, v);
  let n_dot_v = max(dot(n, v), 1e-4);
  let f_ibl = f_schlick_roughness(n_dot_v, surface.f0, surface.roughness);
  let kd_ibl = (vec3<f32>(1.0) - f_ibl) * (1.0 - surface.metallic);
  let irradiance = ambient_light(n, world) * frame.params.x + hemi_irradiance(n);
  let r = reflect(-v, n);
  let prefiltered = reflected_light(r, world, surface.roughness * frame.params.y) * frame.params.x;
  let lut = textureSample(brdf_lut, ibl_sampler, vec2<f32>(n_dot_v, surface.roughness)).rg;
  color += (kd_ibl * irradiance * surface.albedo + prefiltered * (surface.f0 * lut.x + lut.y)) * material.flags.z;  // Karis split sum (pbr_fs.wgsl)
  if (in.look.y > 0.0) { color = mix(color, card_rgb(in.look.z), in.look.y); }
  o.color = vec4<f32>(apply_fog(color, world), select(1.0, coverage, a2c));
  return o;
}

struct CardPrepass {
  @location(0) gbuffer: vec4<f32>,
  @location(1) depth: vec4<f32>,
  @location(2) outline: vec4<f32>,
  @builtin(frag_depth) frag_depth: f32,
};

@fragment
fn fs_crowd_card_prepass(in: CardOut) -> CardPrepass {
  card_set_level(in.local);
  let s = card_sample(in.local, in.cols, in.blend.x, in.rows, in.blend.y);
  if (s.coverage < 0.5) {
    discard;
  }
  let world = card_surface(in, s);
  let m3 = mat3x3<f32>(in.m0, in.m1, in.m2);
  let v = select(normalize(frame.view_pos.xyz - world), -frame.view_forward.xyz, frame.view_pos.w > 0.5);
  var n = normalize(m3 * s.normal);
  if (dot(n, v) < 0.0) { n = normalize(n - v * dot(n, v) * 1.01); }
  let cur = in.cur_clip.xy / in.cur_clip.w;
  let prev = in.prev_clip.xy / max(in.prev_clip.w, 1e-6);
  var o: CardPrepass;
  o.gbuffer = vec4<f32>(oct_encode(n), (cur - prev) * vec2<f32>(0.5, -0.5));
  let part = card_part(s.part);
  o.depth = vec4<f32>(dot(world - frame.view_pos.xyz, frame.view_forward.xyz), reflectivity_bits(clamp(part.a, 0.04, 1.0), 0.0), 0.0, 1.0);
  o.outline = vec4<f32>(0.0);
  o.frag_depth = card_depth(world);
  return o;
}
