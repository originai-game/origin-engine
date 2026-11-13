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
  // The sun's and the sky's light the field shows on average from here,
  // and the sun's through its blades (grass_common.wgsl grass_mean's sun,
  // sky and through), a far blade's.
  @location(6) mean: vec3<f32>,
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
  // Far off, a blade takes what its field shows on average and lights as
  // the ground under it (in `fs`; terrain_lit.wgsl grass_far), so the
  // thinning survivors match the ground they stand on rather than dot it
  // (Ghost of Tsushima's far blades take the terrain's normal).
  o.far = smoothstep(0.2, 0.8, distance(r.blade.root, grass.eye.xyz) / max(grass.variation.w, 1e-3));
  let kind = r.blade.kind * 2u;
  let to_eye = select(normalize(grass.eye.xyz - r.blade.root), -grass.forward.xyz, grass.eye.w > 0.5);
  let mean = grass_mean(grass.canopy.x, grass.canopy.y, grass.base_color.w, vec3<f32>(0.0, 1.0, 0.0), to_eye, grass.sun.xyz);
  let average = grass_mean_color(grass.kinds[kind].rgb, grass.kinds[kind + 1u].rgb, mean.t);
  o.mean = vec3<f32>(mean.sun, mean.sky, mean.through);
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
// the more the lower the sun.
fn grass_canopy(above: f32, l: vec3<f32>) -> f32 {
  let deep = max(grass.canopy.y - above, 0.0);
  let path = deep / max(l.y, 0.08);
  return exp(-grass.canopy.x * path);
}

@fragment
fn fs(in: GrassOut, @builtin(front_facing) front: bool) -> @location(0) vec4<f32> {
  let p = in.world;
  let view = u32(frame.light_color.w);
  if (view > 6u) { return debug_unsupported(in.clip.xy); }
  if (view == 1u) { return vec4<f32>(cascade_tint(cascade_index(p)) * 4.0, 1.0); }
  if (view == 2u) {
    let ao = textureSampleLevel(ssao_tex, fx_sampler, in.clip.xy * fx.screen.zw, 0.0).r;
    return vec4<f32>(vec3<f32>(select(1.0, ao, fx.ao.x > 0.5)), 1.0);
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
  // The decals over the blades (decals.wgsl): a blade takes what is
  // projected onto the ground it stands on, so a mark on a meadow covers
  // its grass; its own normal stays.
  let decaled = surface_decals(s, n0, vec3<f32>(0.0, 1.0, 0.0), in.clip.xy, p, dpdx(p), dpdy(p));
  // Rain, snow and dust on the blades (weather_surface.wgsl): wet blades
  // darken and gloss, snow lies on their upper sides, dust dulls them.
  s = weather_grass(decaled.s, n0, p, in.shading.z).s;
  let n_dot_l = max(dot(n, l), 0.0);
  var shadow = 1.0;
  if (frame.light_dir.w > 0.5 && n_dot_l > 0.0) {
    // The 2x2 PCF, not the tent of solid surfaces: a canopy's own
    // shadows are finer than a texel, and the tent averaged them away (a
    // flat, bright meadow: grass-near 0.021 rmse off its reference).
    shadow = cascade_shadow_lookup(p, n, n_dot_l, false);
  }
  // The field's own canopy, with or without the cascades; a far blade
  // under the field's mean, as the ground it stands on.
  let sunlit = shadow;
  shadow *= mix(grass_canopy(in.above, l), in.mean.x, in.far);
  // Lit from behind, the light comes through the blades in the way: the
  // cascades' thickness of them, already the canopy's depth along the ray
  // (so not dimmed by it a second time).
  if (frame.light_dir.w > 0.5 && n_dot_l > 0.0 && backlit) {
    shadow = max(shadow, grass_transmittance(p));
  }
  // The sun under the frame's material style (lighting.wgsl style_sun), as
  // the terrain's: the BRDF, or the toon bands, matte.
  let sun = style_sun(s, n, v, l, shadow, style_edge(n_dot_l * shadow));
  // The light through a backlit blade: dimmed by its translucency, tinted by its colour.
  let through = grass.base_color.w * min(s.albedo * 2.0, vec3<f32>(1.0));
  var own = sun;
  own.direct = sun.direct * select(vec3<f32>(1.0), through, backlit);
  // A far blade takes the field's mean glow for its own lobe below.
  own.direct += in.far * in.mean.z * pow(max(s.albedo, vec3<f32>(1e-4)), vec3<f32>(0.85)) * frame.light_color.rgb * sunlit;
  // Looking into the sun through a blade, the light it scatters forward
  // (lighting.wgsl leaf_forward, the shared leaf lobe): the glow along
  // backlit blades. Tinted by Beer-Lambert through the blade, its colour
  // raised to its thickness, so the thin tips pass a brighter, warmer
  // light than the roots (Ghost of Tsushima's and Breath of the Wild's
  // sunlit grass). Lit, not toon.
  // Far blades light as the ground, with no lobe of their own.
  if (backlit && !style_toon()) {
    let thickness = mix(1.0, 0.75, in.shading.z);
    let tint = pow(max(s.albedo, vec3<f32>(1e-4)), vec3<f32>(thickness));
    let back = max(-dot(n0, l), 0.0);
    let lobe = 1.5 * grass.base_color.w * back * leaf_forward(n0, v, l) * (1.0 - in.far);
    own.direct += lobe * tint * frame.light_color.rgb * shadow;
  }
  // The caustics (through a backlit blade, dimmed by its translucency),
  // the local lights and the light around the blade, as every lit
  // surface's (surface_shade.wgsl), along the blade's own side. Far
  // blades light as the ground (`far`), under the sky alike.
  let at = ShadeAt(p, n0, v, in.clip.xy, true, true, true);
  let under = n_dot_l * shadow * select(1.0, grass.base_color.w, backlit);
  let lit = surface_shade(s, at, own, under, decaled.emissive, 1.0, mix(in.shading.x, in.mean.y, in.far));
  if (view > 3u) { return vec4<f32>(shaded_debug(lit, s.albedo, view), 1.0); }
  return vec4<f32>(apply_fog(shaded_color(lit), p), 1.0);
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
  // No line of its own; a clump with the ground (id 0), so a line filter
  // draws no blade against it.
  o.outline = outline_bytes(vec3<f32>(0.0), 0.0, 0u, true);
  return o;
}
