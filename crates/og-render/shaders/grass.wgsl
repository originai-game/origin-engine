// Grass's lit and prepass draws (grass.rs). Prepended by frame.wgsl
// (group 0), the light tables (clusters.rs), lighting.wgsl, area.wgsl,
// grass_common.wgsl and grass_blade.wgsl (group 1); the lit pass's effect
// group is group 2.
//
// Shading: the blade's colour lit like the terrain under it (the sun with
// its cascades, the local lights, the image-based and hemisphere light,
// caustics, fog), with og.MaterialStyle's toon bands and shadow tint as
// the terrain takes them, and:
// - translucency: a face turned from the sun is lit as its sunward side
//   (the same shadow), its direct light tinted and dimmed by the blade
//   it passed through (`translucency`), costing no second lookup;
// - sheen: the upper blade is glossier, so gusts show as moving sheen;
// - the root's occlusion on the ambient light and the screen-space AO.
//
// Ported from bevy_grass v0.20.3 (src/shaders/grass.wesl, look.wesl,
// grass_prepass.wesl; third_party/bevy-grass). MIT License, Copyright (c)
// 2026 Gediminas Mikelenas: the full notice is in grass_common.wgsl and
// third_party/bevy-grass/LICENSE.

struct GrassOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) color: vec3<f32>,
  // x: the root's occlusion, y: how much sheen shows here, z: t
  @location(3) shading: vec3<f32>,
  @location(4) cur_clip: vec4<f32>,
  @location(5) prev_clip: vec4<f32>,
};

fn grass_eye(previous: bool) -> GrassEye {
  var e: GrassEye;
  e.position = grass.eye.xyz;
  e.toward = -grass.forward.xyz;
  e.orthographic = grass.eye.w > 0.5;
  e.view_proj = frame.view_proj;
  e.pixel = grass.forward.w;
  e.previous = previous;
  return e;
}

@vertex
fn vs(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> GrassOut {
  let r = grass_ref(vi, ii);
  let v = grass_vertex(r, grass_eye(false));
  var o: GrassOut;
  o.clip = frame.view_proj * vec4<f32>(v.world, 1.0);
  o.world = v.world;
  o.normal = v.normal;
  o.color = grass_color(r.blade, v.t);
  o.shading = vec3<f32>(grass_occlusion(v.t), grass.tip_color.w * smoothstep(0.4, 0.8, v.t), v.t);
  o.cur_clip = grass.view_proj * vec4<f32>(v.world, 1.0);
  // Where the blade stood last frame: its last bend and flutter.
  let p = grass_vertex(r, grass_eye(true));
  o.prev_clip = grass.prev_view_proj * vec4<f32>(p.world, 1.0);
  return o;
}

// The lit pass's effect group (scatter.wgsl ScatterFx): the screen-space AO.
struct GrassFx {
  view_proj_unjittered: mat4x4<f32>,
  prev_view_proj: mat4x4<f32>,
  screen: vec4<f32>,
  ao: vec4<f32>,
};
@group(2) @binding(0) var<uniform> grass_fx: GrassFx;
@group(2) @binding(1) var grass_ssao: texture_2d<f32>;
@group(2) @binding(3) var grass_fx_sampler: sampler;

// The toon ramp's three bands (terrain_lit.wgsl terrain_band).
fn grass_band(t: f32) -> f32 {
  let e = max(fwidth(t), 1e-4);
  return 0.22 + 0.38 * smoothstep(0.08 - e, 0.08 + e, t) + 0.4 * smoothstep(0.45 - e, 0.45 + e, t);
}

@fragment
fn fs(in: GrassOut, @builtin(front_facing) front: bool) -> @location(0) vec4<f32> {
  let p = in.world;
  // Two-sided: the back face is lit with the flipped normal.
  let n0 = normalize(in.normal) * select(-1.0, 1.0, front);
  let v = select(normalize(frame.view_pos.xyz - p), -frame.view_forward.xyz, frame.view_pos.w > 0.5);
  let l = normalize(frame.light_dir.xyz);
  let backlit = dot(n0, l) < 0.0;
  let n = select(n0, -n0, backlit);
  let sheen = in.shading.y;
  var s: Surface;
  s.albedo = in.color;
  s.metallic = 0.0;
  s.roughness = clamp(grass.dry_color.w * (1.0 - 0.45 * sheen), 0.04, 1.0);
  let reflectance = 0.25 + 0.45 * sheen;
  s.f0 = vec3<f32>(0.16 * reflectance * reflectance);
  let n_dot_l = max(dot(n, l), 0.0);
  var shadow = 1.0;
  if (frame.light_dir.w > 0.5 && n_dot_l > 0.0) { shadow = cascade_shadow(p, n, n_dot_l); }
  var direct = brdf(s, n, v, l) * frame.light_color.rgb * shadow;
  var tinted = (1.0 - shadow) * n_dot_l;
  if (frame.outline_style.z > 0.5) {
    // og.MaterialStyle's toon: the terrain's bands, matte.
    let band = grass_band(n_dot_l * shadow);
    direct = s.albedo / PI * band * frame.light_color.rgb;
    tinted = grass_band(n_dot_l) - band;
  }
  if (frame.shadow_tint.w > 0.0 && n_dot_l > 0.0) {
    var tint = frame.shadow_tint.rgb * frame.shadow_tint.w;
    if (frame.outline_style.z > 0.5) { tint = min(tint, vec3<f32>(1.0 / PI)); }
    direct += tinted * tint * s.albedo * frame.light_color.rgb;
  }
  direct += s.albedo * frame.light_color.rgb * n_dot_l * shadow * caustics(p);
  // The light through a backlit blade: dimmed by its translucency, tinted by its colour.
  let through = grass.base_color.w * min(s.albedo * 2.0, vec3<f32>(1.0));
  var color = direct * select(vec3<f32>(1.0), through, backlit);
  color += surface_local_lights(in.clip.xy, s, p, n0, v);
  var occlusion = in.shading.x;
  if (grass_fx.ao.x > 0.5) {
    let ssao = textureSampleLevel(grass_ssao, grass_fx_sampler, in.clip.xy * grass_fx.screen.zw, 0.0).r;
    occlusion *= mix(1.0, ssao, grass_fx.ao.y);
    color *= mix(1.0, mix(0.35, 1.0, ssao), grass_fx.ao.z);
  }
  let n_dot_v = max(dot(n0, v), 1e-4);
  let f_ibl = f_schlick_roughness(n_dot_v, s.f0, s.roughness);
  let irradiance = ambient_light(n0, p) * frame.params.x + hemi_irradiance(n0);
  let r = reflect(-v, n0);
  let prefiltered = reflected_light(r, p, s.roughness * frame.params.y) * frame.params.x;
  let lut = textureSample(brdf_lut, ibl_sampler, vec2<f32>(n_dot_v, s.roughness)).rg;
  color += ((vec3<f32>(1.0) - f_ibl) * irradiance * s.albedo + prefiltered * (f_ibl * lut.x + lut.y)) * occlusion;
  return vec4<f32>(apply_fog(color, p), 1.0);
}

struct GrassPrepass {
  @location(0) gbuffer: vec4<f32>,
  @location(1) depth: vec4<f32>,
  @location(2) outline: vec4<f32>,
};

@fragment
fn fs_prepass(in: GrassOut, @builtin(front_facing) front: bool) -> GrassPrepass {
  let n = normalize(in.normal) * select(-1.0, 1.0, front);
  let cur = in.cur_clip.xy / in.cur_clip.w;
  let prev = in.prev_clip.xy / max(in.prev_clip.w, 1e-6);
  var o: GrassPrepass;
  o.gbuffer = vec4<f32>(oct_encode(n), (cur - prev) * vec2<f32>(0.5, -0.5));
  let roughness = clamp(grass.dry_color.w * (1.0 - 0.45 * in.shading.y), 0.04, 1.0);
  o.depth = vec4<f32>(dot(in.world - frame.view_pos.xyz, frame.view_forward.xyz), reflectivity_bits(roughness, 0.0), 0.0, 1.0);
  o.outline = vec4<f32>(0.0);
  return o;
}
