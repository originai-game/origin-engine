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
  // How far into the field's distance the blade stands (0 near, 1 where
  // it lights as the ground under it does).
  @location(4) far: f32,
  // How high above its root this point of the blade stands (m).
  @location(5) above: f32,
};

// What the prepass takes of a blade: its lit draw needs no motion, and
// the prepass no colour, so each builds only its own.
struct GrassPrepassIn {
  @builtin(position) clip: vec4<f32>,
  @location(0) world: vec3<f32>,
  @location(1) normal: vec3<f32>,
  // How much sheen shows here (GrassOut's shading.y).
  @location(2) sheen: f32,
  @location(3) cur_clip: vec4<f32>,
  @location(4) prev_clip: vec4<f32>,
};

// How much of the field's sheen shows at `t` along a blade.
fn grass_sheen(t: f32) -> f32 {
  return grass.tip_color.w * smoothstep(0.4, 0.8, t);
}

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
  // Far off, a blade takes its kind's average colour and lights as the
  // ground under it (in `fs`), so the thinning survivors match the ground
  // they stand on rather than dot it (Ghost of Tsushima's far blades
  // take the terrain's normal).
  o.far = smoothstep(0.2, 0.8, distance(r.blade.root, grass.eye.xyz) / max(grass.variation.w, 1e-3));
  let kind = r.blade.kind * 2u;
  let average = grass_far_color(grass.kinds[kind].rgb, grass.kinds[kind + 1u].rgb);
  o.color = mix(grass_color(r.blade, v.t), average, 0.6 * o.far) * grass_gust_sheen(grass.wind, r.blade.root.xz + grass.wind.time.zw, GRASS_GUST_SHEEN);
  o.shading = vec3<f32>(grass_occlusion(max(v.world.y - r.blade.root.y, 0.0)), grass_sheen(v.t), v.t);
  o.above = max(v.world.y - r.blade.root.y, 0.0);
  return o;
}

@vertex
fn vs_prepass(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> GrassPrepassIn {
  let r = grass_ref(vi, ii);
  let v = grass_vertex(r, grass_eye(false));
  var o: GrassPrepassIn;
  o.clip = frame.view_proj * vec4<f32>(v.world, 1.0);
  o.world = v.world;
  o.normal = v.normal;
  o.sheen = grass_sheen(v.t);
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

// The sunlight that reaches `p` through what the first cascades hold
// between it and the sun (Jimenez et al., "Real-Time Realistic Skin
// Translucency", 2010: the thickness from the shadow map's depth to the
// point's, and Beer-Lambert through it): a backlit blade behind others
// still glows through them, fading with the canopy it is under. Thin
// next to a blade (its own face), 0 behind a hill.
fn grass_transmittance(p: vec3<f32>) -> f32 {
  let c = cascade_index(p);
  if (c >= CASCADES) { return 1.0; }
  let m = frame.cascade_view_proj[c];
  let sp = m * vec4<f32>(p, 1.0);
  let uv = vec2<f32>(sp.x * 0.5 + 0.5, 0.5 - sp.y * 0.5);
  if (sp.z > 1.0 || any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0))) { return 1.0; }
  let size = vec2<i32>(textureDimensions(cascade_map));
  let at = clamp(vec2<i32>(uv * vec2<f32>(size)), vec2<i32>(0), size - 1);
  let occluder = textureLoad(cascade_map, at, i32(c), 0).r;
  // The map's depth per metre along the light.
  let per_metre = length(vec3<f32>(m[0].z, m[1].z, m[2].z));
  let thickness = max(sp.z - occluder, 0.0) / max(per_metre, 1e-9);
  return exp(-thickness * GRASS_EXTINCTION);
}

// How fast light dims through a canopy of blades (per metre).
const GRASS_EXTINCTION: f32 = 1.6;

// The sunlight left at a point `above` metres over its root after
// crossing the field's own canopy towards the sun `l` (G5): the canopy a
// slab from the ground to grass.canopy.y, as thick along the ray as the
// point is deep in it over the sun's height, its extinction the blades'
// density times their width (Beer-Lambert through a turbid layer: Monsi
// and Saeki's law of light in a plant canopy, 1953, the leaf area along
// the ray standing for their leaf area index). Self-shadowing the
// cascades are too coarse to draw: dense grass darkens towards its roots,
// the more the lower the sun. Far blades light as the ground (`far`).
fn grass_canopy(above: f32, l: vec3<f32>, far: f32) -> f32 {
  let deep = max(grass.canopy.y - above, 0.0);
  let path = deep / max(l.y, 0.08);
  return mix(exp(-grass.canopy.x * path), 1.0, far);
}

// The toon ramp's three bands (terrain_lit.wgsl terrain_band).
fn grass_band(t: f32) -> f32 {
  let e = max(fwidthFine(t), 1e-4);
  return 0.22 + 0.38 * smoothstep(0.08 - e, 0.08 + e, t) + 0.4 * smoothstep(0.45 - e, 0.45 + e, t);
}

@fragment
fn fs(in: GrassOut, @builtin(front_facing) front: bool) -> @location(0) vec4<f32> {
  let p = in.world;
  let view = u32(frame.light_color.w);
  if (view > 6u) { return debug_unsupported(in.clip.xy); }
  if (view == 1u) { return vec4<f32>(cascade_tint(cascade_index(p)) * 4.0, 1.0); }
  if (view == 2u) {
    let ao = textureSampleLevel(grass_ssao, grass_fx_sampler, in.clip.xy * grass_fx.screen.zw, 0.0).r;
    return vec4<f32>(vec3<f32>(select(1.0, ao, grass_fx.ao.x > 0.5)), 1.0);
  }
  if (view == 3u) { return vec4<f32>(in.color, 1.0); }
  // Two-sided: the back face is lit with the flipped normal.
  let n0 = normalize(mix(normalize(in.normal) * select(-1.0, 1.0, front), vec3<f32>(0.0, 1.0, 0.0), 0.85 * in.far));
  let v = select(normalize(frame.view_pos.xyz - p), -frame.view_forward.xyz, frame.view_pos.w > 0.5);
  let l = normalize(frame.light_dir.xyz);
  let backlit = dot(n0, l) < 0.0;
  let n = select(n0, -n0, backlit);
  // Specular anti-aliasing: a blade narrower than a pixel cannot show
  // its gloss as a highlight (it sparkles); its gloss spreads instead
  // (rougher, dimmer), as Ghost of Tsushima's grass does far off.
  let depth = select(dot(p - frame.view_pos.xyz, frame.view_forward.xyz), 1.0, frame.view_pos.w > 0.5);
  let resolved = clamp(grass.shape.y / max(depth * grass.forward.w, 1e-6), 0.0, 1.0);
  let sheen = in.shading.y * resolved;
  var s: Surface;
  s.albedo = in.color;
  s.metallic = 0.0;
  s.roughness = mix(0.95, clamp(grass.dry_color.w * (1.0 - 0.45 * sheen), 0.04, 1.0), resolved);
  let reflectance = (0.25 + 0.45 * sheen) * mix(0.5, 1.0, resolved);
  s.f0 = vec3<f32>(0.16 * reflectance * reflectance);
  // Rain, snow and dust on the blades (weather_surface.wgsl): wet blades
  // darken and gloss, snow lies on their upper sides, dust dulls them.
  s = weather_grass(s, n0, p, in.shading.z).s;
  let n_dot_l = max(dot(n, l), 0.0);
  var shadow = 1.0;
  if (frame.light_dir.w > 0.5 && n_dot_l > 0.0) {
    // The 2x2 PCF, not the tent of solid surfaces: a canopy's own
    // shadows are finer than a texel, and the tent averaged them away (a
    // flat, bright meadow: grass-near 0.021 rmse off its reference).
    shadow = cascade_shadow_lookup(p, n, n_dot_l, false);
  }
  // The field's own canopy, with or without the cascades.
  shadow *= grass_canopy(in.above, l, in.far);
  // Lit from behind, the light comes through the blades in the way: the
  // cascades' thickness of them, already the canopy's depth along the ray
  // (so not dimmed by it a second time).
  if (frame.light_dir.w > 0.5 && n_dot_l > 0.0 && backlit) {
    shadow = max(shadow, grass_transmittance(p));
  }
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
  // Looking into the sun through a blade, the light it scatters forward
  // (lighting.wgsl leaf_forward, the shared leaf lobe): the glow along
  // backlit blades. Tinted by Beer-Lambert through the blade, its colour
  // raised to its thickness, so the thin tips pass a brighter, warmer
  // light than the roots (Ghost of Tsushima's and Breath of the Wild's
  // sunlit grass). Lit, not toon.
  // Far blades light as the ground, with no lobe of their own.
  if (backlit && frame.outline_style.z < 0.5) {
    let thickness = mix(1.0, 0.75, in.shading.z);
    let tint = pow(max(s.albedo, vec3<f32>(1e-4)), vec3<f32>(thickness));
    let back = max(-dot(n0, l), 0.0);
    let lobe = 1.5 * grass.base_color.w * back * leaf_forward(n0, v, l) * (1.0 - in.far);
    color += lobe * tint * frame.light_color.rgb * shadow;
  }
  color += surface_local_lights(in.clip.xy, s, p, n0, v);
  // Far blades light as the ground (`far`), under the sky alike.
  var occlusion = mix(in.shading.x, 1.0, in.far);
  if (grass_fx.ao.x > 0.5) {
    let ssao = textureSampleLevel(grass_ssao, grass_fx_sampler, in.clip.xy * grass_fx.screen.zw, 0.0).r;
    occlusion *= mix(1.0, ssao, grass_fx.ao.y);
    color *= mix(1.0, mix(0.35, 1.0, ssao), grass_fx.ao.z);
  }
  let n_dot_v = max(dot(n0, v), 1e-4);
  let f_ibl = f_schlick_roughness(n_dot_v, s.f0, s.roughness);
  let irradiance = ambient_light(n0, p) * frame.params.x + hemi_irradiance(n0);
  let r = reflect(-v, n0);
  // With the screen-space reflections over it (frame.wgsl ssr_reflected).
  let prefiltered = ssr_reflected(reflected_light(r, p, s.roughness * frame.params.y) * frame.params.x, p, in.clip.xy);
  let lut = textureSample(brdf_lut, ibl_sampler, vec2<f32>(n_dot_v, s.roughness)).rg;
  if (view == 4u) { return vec4<f32>((vec3<f32>(1.0) - f_ibl) * irradiance * s.albedo * occlusion, 1.0); }
  if (view == 5u) { return vec4<f32>(prefiltered * (s.f0 * lut.x + lut.y) * occlusion, 1.0); }
  if (view == 6u) { return vec4<f32>(color, 1.0); }
  color += ((vec3<f32>(1.0) - f_ibl) * irradiance * s.albedo + prefiltered * (s.f0 * lut.x + lut.y)) * occlusion;  // Karis split sum (pbr_fs.wgsl)
  return vec4<f32>(apply_fog(color, p), 1.0);
}

struct GrassPrepass {
  @location(0) gbuffer: vec4<f32>,
  @location(1) depth: vec4<f32>,
  @location(2) outline: vec4<f32>,
};

@fragment
fn fs_prepass(in: GrassPrepassIn, @builtin(front_facing) front: bool) -> GrassPrepass {
  let n = normalize(in.normal) * select(-1.0, 1.0, front);
  let cur = in.cur_clip.xy / in.cur_clip.w;
  let prev = in.prev_clip.xy / max(in.prev_clip.w, 1e-6);
  var o: GrassPrepass;
  o.gbuffer = vec4<f32>(oct_encode(n), (cur - prev) * vec2<f32>(0.5, -0.5));
  let roughness = clamp(grass.dry_color.w * (1.0 - 0.45 * in.sheen), 0.04, 1.0);
  o.depth = vec4<f32>(dot(in.world - frame.view_pos.xyz, frame.view_forward.xyz), reflectivity_bits(roughness, 0.0), 0.0, 1.0);
  o.outline = vec4<f32>(0.0);
  return o;
}
