// Impostors' lit and prepass passes (impostor.rs). Prepended as
// scatter.wgsl is (frame.wgsl, the light tables, noise.wgsl,
// material.wgsl: group 1 the impostor's StandardMaterial, lighting.wgsl,
// scatter_common.wgsl: group 2) and impostor_common.wgsl.
//
// The three frames around the eye's direction are blended by weight and
// coverage: the albedo, the normal (each frame's turned out of its basis
// into the model's, then the world's) and the depth, which moves the lit
// point off the card towards the eye. Shaded as scatter.wgsl shades a copy,
// without a material shader's hooks, normal maps or effects.

struct ImpostorOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world: vec3<f32>,
  @location(1) uv01: vec4<f32>,
  @location(2) uv2: vec2<f32>,
  @location(3) @interpolate(flat) frames: vec3<u32>,
  @location(4) @interpolate(flat) weights: vec3<f32>,
  @location(5) @interpolate(flat) fade: f32,
  @location(6) cur_clip: vec4<f32>,
  @location(7) prev_clip: vec4<f32>,
  // The model's turn (its columns, normalised) and its sphere's radius in the world.
  @location(8) @interpolate(flat) m0: vec3<f32>,
  @location(9) @interpolate(flat) m1: vec3<f32>,
  @location(10) @interpolate(flat) m2: vec3<f32>,
  @location(11) @interpolate(flat) extent: f32,
};

// How much of a copy standing at `at` draws as its impostor: past the
// fade in of scatter_near (scatter_common.wgsl), all of a model's.
fn imp_fade(at: vec3<f32>) -> f32 {
  let near = select(1.0, 1.0 - scatter_near(at), scatter.impostor_grid.y > 0.0);
  return scatter_fade(at) * near;
}

@vertex
fn vs_impostor(@builtin(vertex_index) vertex: u32, i: ScatterInstance) -> ImpostorOut {
  let model = instance_matrix(i.m0, i.m1, i.m2, i.m3);
  let card = imp_card(model, vertex, frame.view_pos.xyz, false);
  var o: ImpostorOut;
  o.fade = imp_fade(i.m3);
  o.clip = select(SCATTER_GONE, mul_point(frame.view_proj, card.world), o.fade > 0.0);
  o.world = card.world.xyz;
  o.uv01 = card.uv01;
  o.uv2 = card.uv2;
  o.frames = card.frames;
  o.weights = card.weights;
  o.cur_clip = mul_point(scatter.view_proj, card.world);
  o.prev_clip = mul_point(scatter.prev_view_proj, card.world);
  o.m0 = normalize(i.m0);
  o.m1 = normalize(i.m1);
  o.m2 = normalize(i.m2);
  o.extent = card.extent;
  return o;
}

struct ImpSurface {
  albedo: vec3<f32>,
  normal: vec3<f32>,
  // The point lit: the card's moved along the eye by the blended depth.
  world: vec3<f32>,
};

// One frame's normal (its basis turned into the model's space) and depth.
fn imp_normal(f: u32, at: vec2<f32>) -> vec4<f32> {
  let s = textureSample(normal_tex, material_sampler, at);
  let d = imp_frame_dir(f);
  let r = imp_right(d);
  let u = cross(d, r);
  let n = s.rgb * 2.0 - 1.0;
  return vec4<f32>(r * n.x + u * n.y + d * n.z, s.a * 2.0 - 1.0);
}

fn imp_surface(in: ImpostorOut, c: ImpCoverage) -> ImpSurface {
  let total = max(c.wa.x + c.wa.y + c.wa.z, 1e-5);
  let rgb = textureSample(base_color_tex, material_sampler, c.a0).rgb * c.wa.x
    + textureSample(base_color_tex, material_sampler, c.a1).rgb * c.wa.y
    + textureSample(base_color_tex, material_sampler, c.a2).rgb * c.wa.z;
  let nd = imp_normal(in.frames.x, c.a0) * c.wa.x
    + imp_normal(in.frames.y, c.a1) * c.wa.y
    + imp_normal(in.frames.z, c.a2) * c.wa.z;
  var s: ImpSurface;
  s.albedo = material.base_color.rgb * rgb / total;
  let m3 = mat3x3<f32>(in.m0, in.m1, in.m2);
  s.normal = normalize(m3 * nd.xyz + vec3<f32>(0.0, 1e-6, 0.0));
  let v = select(normalize(frame.view_pos.xyz - in.world), -frame.view_forward.xyz, frame.view_pos.w > 0.5);
  s.world = in.world + v * (nd.w / total) * in.extent;
  return s;
}

@fragment
fn fs_impostor(in: ImpostorOut) -> @location(0) vec4<f32> {
  let cover = imp_coverage(in.uv01, in.uv2, in.frames, in.weights);
  let surface = imp_surface(in, cover);
  if (cover.alpha < imp_cutoff() || scatter_dropped(in.clip.xy, in.fade)) {
    discard;
  }
  let mode = u32(material.flags.w);
  let emissive = material.emissive.rgb;
  if ((mode & 2u) != 0u) {
    return vec4<f32>(apply_fog(surface.albedo + emissive, surface.world), 1.0);
  }
  var s: Surface;
  s.metallic = clamp(material.mr.x, 0.0, 1.0);
  s.roughness = clamp(material.mr.y, 0.04, 1.0);
  s.albedo = surface.albedo;
  s.f0 = mix(vec3<f32>(0.04), s.albedo, s.metallic);
  let v = select(normalize(frame.view_pos.xyz - surface.world), -frame.view_forward.xyz, frame.view_pos.w > 0.5);
  var n = surface.normal;
  if (dot(n, v) < 0.0) { n = normalize(n - v * dot(n, v) * 1.01); }
  let l = normalize(frame.light_dir.xyz);
  let n_dot_l = max(dot(n, l), 0.0);
  var shadow = 1.0;
  if (frame.light_dir.w > 0.5 && n_dot_l > 0.0) { shadow = cascade_shadow(surface.world, n, n_dot_l); }
  var color = brdf(s, n, v, l) * frame.light_color.rgb * shadow;
  color += s.albedo * frame.light_color.rgb * n_dot_l * shadow * caustics(surface.world);
  color += surface_local_lights(in.clip.xy, s, surface.world, n, v);
  let n_dot_v = max(dot(n, v), 1e-4);
  let f_ibl = f_schlick_roughness(n_dot_v, s.f0, s.roughness);
  let kd_ibl = (vec3<f32>(1.0) - f_ibl) * (1.0 - s.metallic);
  let irradiance = ambient_light(n, surface.world) * frame.params.x + hemi_irradiance(n);
  let r = reflect(-v, n);
  let prefiltered = reflected_light(r, surface.world, s.roughness * frame.params.y) * frame.params.x;
  let lut = textureSample(brdf_lut, ibl_sampler, vec2<f32>(n_dot_v, s.roughness)).rg;
  color += (kd_ibl * irradiance * s.albedo + prefiltered * (f_ibl * lut.x + lut.y)) * material.flags.z;
  color += emissive;
  return vec4<f32>(apply_fog(color, surface.world), 1.0);
}

struct ImpostorPrepass {
  @location(0) gbuffer: vec4<f32>,
  @location(1) depth: vec4<f32>,
  @location(2) outline: vec4<f32>,
};

@fragment
fn fs_impostor_prepass(in: ImpostorOut) -> ImpostorPrepass {
  let cover = imp_coverage(in.uv01, in.uv2, in.frames, in.weights);
  let surface = imp_surface(in, cover);
  if (cover.alpha < imp_cutoff() || scatter_dropped(in.clip.xy, in.fade)) {
    discard;
  }
  let v = select(normalize(frame.view_pos.xyz - surface.world), -frame.view_forward.xyz, frame.view_pos.w > 0.5);
  var n = surface.normal;
  if (dot(n, v) < 0.0) { n = normalize(n - v * dot(n, v) * 1.01); }
  let cur = in.cur_clip.xy / in.cur_clip.w;
  let prev = in.prev_clip.xy / max(in.prev_clip.w, 1e-6);
  var o: ImpostorPrepass;
  o.gbuffer = vec4<f32>(oct_encode(n), (cur - prev) * vec2<f32>(0.5, -0.5));
  let reflectivity = select(reflectivity_bits(clamp(material.mr.y, 0.04, 1.0), material.mr.x), 0.0, (u32(material.flags.w) & 2u) != 0u);
  o.depth = vec4<f32>(dot(surface.world - frame.view_pos.xyz, frame.view_forward.xyz), reflectivity, 0.0, 1.0);
  o.outline = vec4<f32>(0.0);
  return o;
}
