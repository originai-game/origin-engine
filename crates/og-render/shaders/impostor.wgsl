// Impostors' lit and prepass passes (impostor.rs). Prepended as
// scatter.wgsl is (frame.wgsl, the light tables, noise.wgsl,
// material.wgsl: group 1 the impostor's StandardMaterial, lighting.wgsl,
// scatter_common.wgsl: group 2) and impostor_common.wgsl.
//
// The three frames around the eye's direction are each sampled where they
// see the eye's ray through the pixel meet the surface (one parallax step
// on their depth), and blended by weight and coverage: the albedo, the
// normal (each frame's turned out of its basis into the model's, then the
// world's) and the depth, which places the lit point and the depth
// written on the surface rather than the card, so ground in front of the
// model's centre does not cut it. Shaded as scatter.wgsl shades a copy,
// without a material shader's hooks, normal maps or effects. Its edge is
// alpha to coverage, as a cut-out copy's.

struct ImpostorOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world: vec3<f32>,
  @location(1) uv01: vec4<f32>,
  @location(2) uv2: vec2<f32>,
  @location(3) @interpolate(flat) frames: vec3<u32>,
  @location(4) @interpolate(flat) weights: vec3<f32>,
  // (lo, hi) of imp_dropped.
  @location(5) @interpolate(flat) fade: vec2<f32>,
  @location(6) cur_clip: vec4<f32>,
  @location(7) prev_clip: vec4<f32>,
  // The model's turn (its columns, normalised) and its sphere's radius in the world.
  @location(8) @interpolate(flat) m0: vec3<f32>,
  @location(9) @interpolate(flat) m1: vec3<f32>,
  @location(10) @interpolate(flat) m2: vec3<f32>,
  @location(11) @interpolate(flat) extent: f32,
  // The model's sphere's centre in the world.
  @location(12) @interpolate(flat) centre: vec3<f32>,
};

@vertex
fn vs_impostor(@builtin(vertex_index) vertex: u32, i: ScatterInstance) -> ImpostorOut {
  let size = imp_scale(i.m3);
  // Kept invertible when gone (the card is culled then).
  let model = scatter_shrunk(instance_matrix(i.m0, i.m1, i.m2, i.m3), max(size, 1e-4));
  let card = imp_card(model, vertex, frame.view_pos.xyz, false);
  var o: ImpostorOut;
  o.fade = imp_fade(i.m3);
  o.clip = select(SCATTER_GONE, mul_point(frame.view_proj, card.world), size > 0.0 && o.fade.y > o.fade.x);
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
  o.centre = mul_point(model, vec4<f32>(scatter.impostor.xyz, 1.0)).xyz;
  return o;
}

struct ImpSurface {
  albedo: vec3<f32>,
  normal: vec3<f32>,
  // The point lit: the card's moved along the eye by the blended depth.
  world: vec3<f32>,
  coverage: ImpCoverage,
};

// One frame's normal (its basis turned into the model's space) and depth.
fn imp_normal(f: u32, at: vec2<f32>) -> vec4<f32> {
  let s = textureSampleLevel(normal_tex, material_sampler, at, imp_level);
  let d = imp_frame_dir(f);
  let r = imp_right(d);
  let u = cross(d, r);
  let n = s.rgb * 2.0 - 1.0;
  return vec4<f32>(r * n.x + u * n.y + d * n.z, s.a * 2.0 - 1.0);
}

fn imp_surface(in: ImpostorOut) -> ImpSurface {
  // The card's point and the eye in the model's space (as imp_card has
  // them; the turn's columns are normalised, so its transpose undoes it).
  let m3 = mat3x3<f32>(in.m0, in.m1, in.m2);
  // The pixel's own eye ray, not the card centre's (the card's vertex
  // stage faces it to the centre): at a few metres they part by several
  // degrees, which tilted every sample's normal towards the rim.
  let toward = select(normalize(frame.view_pos.xyz - in.world), -frame.view_forward.xyz, frame.view_pos.w > 0.5);
  let ray = transpose(m3) * toward;
  let dd = imp_depths(transpose(m3) * (in.world - in.centre) / in.extent, ray, in.frames);
  let par01 = vec4<f32>(imp_frame_par(ray, in.frames.x), imp_frame_par(ray, in.frames.y));
  let uvs = imp_uvs(in.uv01, in.uv2, par01, imp_frame_par(ray, in.frames.z), in.frames, dd.pd);
  let c = imp_coverage(uvs.uv01, uvs.uv2, in.frames, in.weights);
  let total = max(c.wa.x + c.wa.y + c.wa.z, 1e-5);
  let rgb = textureSampleLevel(base_color_tex, material_sampler, c.a0, imp_level).rgb * c.wa.x
    + textureSampleLevel(base_color_tex, material_sampler, c.a1, imp_level).rgb * c.wa.y
    + textureSampleLevel(base_color_tex, material_sampler, c.a2, imp_level).rgb * c.wa.z;
  let nd = imp_normal(in.frames.x, c.a0) * c.wa.x
    + imp_normal(in.frames.y, c.a1) * c.wa.y
    + imp_normal(in.frames.z, c.a2) * c.wa.z;
  var s: ImpSurface;
  s.coverage = c;
  s.albedo = material.base_color.rgb * rgb / total;
  s.normal = normalize(m3 * nd.xyz + vec3<f32>(0.0, 1e-6, 0.0));
  s.world = in.world + toward * imp_blended_step(c, dd) * in.extent;
  return s;
}

// The surface's depth in the view (written, so the card stands where the
// model's surface does).
fn imp_depth(world: vec3<f32>) -> f32 {
  let c = mul_point(frame.view_proj, vec4<f32>(world, 1.0));
  return clamp(c.z / c.w, 0.0, 1.0);
}

struct ImpostorLit {
  @location(0) color: vec4<f32>,
  @builtin(frag_depth) depth: f32,
};

@fragment
fn fs_impostor(in: ImpostorOut) -> ImpostorLit {
  imp_set_level(in.uv01.xy);
  let surface = imp_surface(in);
  let alpha = surface.coverage.alpha;
  // Alpha to coverage under MSAA: the blended coverage past the cut over
  // one pixel's change of it is the share of the samples covered. Without
  // MSAA (the pipeline then has none) the edge is the cut, as a cut-out
  // copy's is (scatter.wgsl).
  let a2c = scatter.impostor_grid.z > 0.5;
  let coverage = clamp((alpha - imp_cutoff()) / max(fwidth(alpha), 1e-4) + 0.5, 0.0, 1.0);
  if (select(alpha < imp_cutoff(), coverage <= 0.0, a2c) || imp_dropped(in.clip.xy, in.fade.x, in.fade.y)) {
    discard;
  }
  var o: ImpostorLit;
  o.depth = imp_depth(surface.world);
  if (frame.light_color.w > 0.5) {
    o.color = debug_unsupported(in.clip.xy);
    if (frame.light_color.w > 2.5 && frame.light_color.w < 3.5) { o.color = vec4<f32>(surface.albedo, 1.0); }
    return o;
  }
  let mode = u32(material.flags.w);
  let emissive = material.emissive.rgb;
  if ((mode & 2u) != 0u) {
    o.color = vec4<f32>(apply_fog(surface.albedo + emissive, surface.world), select(1.0, coverage, a2c));
    return o;
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
  // Looked up half a radius past where the light's ray leaves the model's
  // sphere: the card takes no shadow of its own. The lit surface and the
  // shadow card's (impostor_shadow.wgsl) are each rebuilt from blended
  // frames and differ by the frames' steps, most near the light's
  // terminator, where any fixed bias left a stepped band; and a model
  // that fills its sphere (a ball) casts from the sphere itself, so the
  // exit point alone is its own lit surface. Other casters still shadow
  // it, and its own shadow on the rest of the scene is the surface's.
  let rel = surface.world - in.centre;
  let b = dot(rel, l);
  let leave = -b + sqrt(max(b * b - dot(rel, rel) + in.extent * in.extent, 0.0));
  if (frame.light_dir.w > 0.5 && n_dot_l > 0.0) { shadow = cascade_shadow(surface.world + l * (max(leave, 0.0) + 0.5 * in.extent), n, n_dot_l); }
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
  o.color = vec4<f32>(apply_fog(color, surface.world), select(1.0, coverage, a2c));
  return o;
}

struct ImpostorPrepass {
  @location(0) gbuffer: vec4<f32>,
  @location(1) depth: vec4<f32>,
  @location(2) outline: vec4<f32>,
  @builtin(frag_depth) frag_depth: f32,
};

@fragment
fn fs_impostor_prepass(in: ImpostorOut) -> ImpostorPrepass {
  imp_set_level(in.uv01.xy);
  let surface = imp_surface(in);
  if (surface.coverage.alpha < imp_cutoff() || imp_dropped(in.clip.xy, in.fade.x, in.fade.y)) {
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
  o.frag_depth = imp_depth(surface.world);
  return o;
}
