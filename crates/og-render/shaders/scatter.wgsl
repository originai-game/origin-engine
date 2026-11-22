// The scattered copies' lit and prepass passes. Prepended by frame.wgsl,
// the light tables (clusters.rs), noise.wgsl, material.wgsl (group 1: the copies' StandardMaterial),
// lighting.wgsl and scatter_common.wgsl (group 2).
//
// Shading: the material's colour (factor, texture and the mesh's vertex
// colours), alpha cut-off, roughness and metalness, emissive, unlit and
// flat modes, lit by the directional light with its cascades, the local
// lights, the image-based light and caustics under water, then fog. A
// mesh with foliage (its stream, `vs_foliage`) draws as the meshlet path
// draws a model's tree (foliage.wgsl): its leaves grown, turned and
// dropped through the year, bent about their anchors, branches and trunk
// by the wind, domed about its canopy (domeNormals) and shaded by its own
// crown, the sun let through by their translucency. Normal
// maps and the material effects (toon, rim, dissolve) are not drawn on
// scattered copies. A material shader's hooks run as on meshes: the copy's
// vertex_offset in the vertex stage, surface_color (before the cut-off),
// surface_normal and surface_emissive in the fragment stages.

struct ScatterOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
  @location(3) color: vec4<f32>,
  // How much of the copy draws as its mesh (scatter_near; scatter_hidden).
  @location(4) @interpolate(flat, either) fade: f32,
  // The copy's id in the outline target (clump_id); beside the other
  // scalar, so no vector is packed for it (impostor.wgsl).
  @location(8) @interpolate(flat, either) object: u32,
  @location(5) cur_clip: vec4<f32>,
  @location(6) prev_clip: vec4<f32>,
  @location(7) local: vec3<f32>,
  // A seasonal leaf's autumn colour (foliage.wgsl `season_leaf_color`; 0
  // without foliage) and what its crown hides (`meshlet_crown`).
  @location(9) season: vec4<f32>,
  @location(10) crown: vec2<f32>,
};

fn scatter_hook(in: ScatterOut, front: bool) -> Hook {
  var h: Hook;
  h.world = in.world;
  h.local = in.local;
  h.normal = in.normal;
  h.uv = in.uv;
  h.color = in.color;
  h.time = frame.time.x;
  h.origin = frame.origin.xyz;
  h.eye = frame.view_pos.xyz;
  h.front = select(-1.0, 1.0, front);
  h.wind = frame.foliage_wind[0][0];
  hook_front = h.front;
  return h;
}

@vertex
fn vs(v: ScatterVertex, i: ScatterInstance) -> ScatterOut {
  return scatter_vertex(v, i, SCATTER_RIGID, false);
}

// A foliage mesh's copy (pipelines.rs FOLIAGE_KIND).
@vertex
fn vs_foliage(v: ScatterVertex, i: ScatterInstance, f: ScatterFoliage) -> ScatterOut {
  return scatter_vertex(v, i, f, true);
}

fn scatter_vertex(v: ScatterVertex, i: ScatterInstance, f: ScatterFoliage, foliage: bool) -> ScatterOut {
  let size = scatter_scale(i.m3);
  let model = scatter_shrunk(instance_matrix(i.m0, i.m1, i.m2, i.m3), size);
  var o: ScatterOut;
  o.fade = scatter_near(i.m3);
  o.object = clump_id(i.m3);
  let scale = length(i.m1) * size;
  // A leaf in its year: drawn in to its anchor while in bud or fallen.
  var rest = v.position;
  var leaf = MeshletLeaf(SeasonLeaf(1.0, 0.0, 0.0), 0.0);
  if (foliage) {
    leaf = scatter_leaf(f, model);
    rest = meshlet_grown(rest, f.leaf, leaf.leaf.kept);
  }
  var sway: vec3<f32>;
  var prev_sway: vec3<f32>;
  if (foliage) {
    sway = scatter_foliage_sway(rest, f, model, i.m3, scale, foliage_wind_unpack(frame.foliage_wind[0]), false);
    prev_sway = scatter_foliage_sway(rest, f, model, i.m3, scale, foliage_wind_unpack(frame.foliage_wind[1]), true);
  } else {
    sway = scatter_sway(rest, i.m3, scale, false);
    prev_sway = scatter_sway(rest, i.m3, scale, true);
  }
  var world = model * vec4<f32>(rest, 1.0) + vec4<f32>(sway, 0.0);
  // A leaf's canopy dome (the material's domeNormals, coat.w) about its
  // anchor, as a model's leaves take it (meshlet.wgsl vs_meshlet).
  let dome = foliage_normal(v.normal, f.leaf.xyz, scatter.foliage[0].xyz, material.coat.w * f32(foliage));
  o.normal = normalize((model * vec4<f32>(dome, 0.0)).xyz);
  // Its autumn colour, which the fragment puts in place of its own, and
  // the light its crown hides (meshlet.wgsl vs_meshlet).
  if (foliage) {
    o.season = meshlet_leaf_color(scatter.foliage[2], scatter.foliage[3], leaf);
    let axis = meshlet_crown_axis(v.normal, f.leaf, scatter.foliage[0], scatter.foliage[1]);
    let sun = meshlet_model_dir(model, frame.light_dir.xyz);
    o.crown = meshlet_crown(rest, axis, sun, scatter.foliage[0], scatter.foliage[4]);
  }
  var h: Hook;
  h.world = world.xyz;
  h.local = rest;
  h.normal = o.normal;
  h.uv = v.uv;
  h.color = v.color;
  h.time = frame.time.x;
  h.origin = frame.origin.xyz;
  h.eye = frame.view_pos.xyz;
  h.front = 1.0;
  h.wind = frame.foliage_wind[0][0];
  world = vec4<f32>(world.xyz + vertex_offset(h, shader_params()), world.w);
  o.clip = select(SCATTER_GONE, mul_point(frame.view_proj, world), size > 0.0 && o.fade > 0.0);
  o.world = world.xyz;
  o.local = rest;
  o.uv = v.uv;
  o.color = v.color;
  // The camera and the wind move the copies on screen: where the vertex
  // was swayed to the previous frame.
  o.cur_clip = mul_point(scatter.view_proj, world);
  h.world = (model * vec4<f32>(rest, 1.0)).xyz + prev_sway;
  h.time = frame.time.w;
  h.wind = frame.foliage_wind[1][0];
  let previous = h.world + vertex_offset(h, previous_shader_params());
  o.prev_clip = mul_point(scatter.prev_view_proj, vec4<f32>(previous, 1.0));
  return o;
}

// How high a point lies in the material's relief for the weather, as a
// mesh's (pbr_fs.wgsl weather_cavity): from its AO map, 0.5 without one.
fn scatter_cavity(uv: vec2<f32>) -> f32 {
  if (material.flags.y < 0.0) { return 0.5; }
  return 0.2 + 0.6 * textureSample(ao_tex, material_sampler, material_uv(UV_AO, uv)).r;
}

@fragment
fn fs(in: ScatterOut, @builtin(front_facing) front: bool) -> @location(0) vec4<f32> {
  let h = scatter_hook(in, front);
  let base = surface_color(h, shader_params(), season_mix(material.base_color * in.color, in.season) * textureSample(base_color_tex, material_sampler, material_uv(UV_MAP, in.uv)));
  let dpx = dpdxFine(in.world);
  let dpy = dpdyFine(in.world);
  let dlx = dpdxFine(in.local);
  let dly = dpdyFine(in.local);
  // Alpha to coverage (mode 16, an alpha-tested material, as pbr_fs.wgsl): alpha past the cut-off over one pixel's change
  // of it is the share of the MSAA samples covered (three.js' alphaToCoverage on grass and foliage cards).
  let mode = u32(material.flags.w);
  let a2c = (mode & 16u) != 0u && material.emissive.w >= 0.0;
  let coverage = clamp((base.a - material.emissive.w) / max(fwidthFine(base.a), 1e-4) + 0.5, 0.0, 1.0);
  // One discard for the cut-off and the fade (Metal ignored a second one).
  if ((material.emissive.w >= 0.0 && select(base.a < material.emissive.w, coverage <= 0.0, a2c)) || dither_cut(scatter_hidden(in.fade), in.clip.xy, frame.drawn.w)) {
    discard;
  }
  return scatter_lit(in, h, base, select(1.0, coverage, a2c), dpx, dpy, dlx, dly);
}

// The lit pass over the prepass's own depth, tested for equality
// (pipelines.rs EQUAL_KIND): the prepass kept only the fragments past the
// cut-off and the fade, so this one discards nothing and writes alpha 1,
// and the GPU rejects every hidden fragment before shading it.
@fragment
fn fs_equal(in: ScatterOut, @builtin(front_facing) front: bool) -> @location(0) vec4<f32> {
  let h = scatter_hook(in, front);
  let base = surface_color(h, shader_params(), season_mix(material.base_color * in.color, in.season) * textureSample(base_color_tex, material_sampler, material_uv(UV_MAP, in.uv)));
  return scatter_lit(in, h, base, 1.0, dpdxFine(in.world), dpdyFine(in.world), dpdxFine(in.local), dpdyFine(in.local));
}

// A kept fragment's colour with alpha `out_a`; `dpx`..`dly` its world and
// local positions' derivatives, taken where control flow was uniform.
fn scatter_lit(in: ScatterOut, h: Hook, base: vec4<f32>, out_a: f32, dpx: vec3<f32>, dpy: vec3<f32>, dlx: vec3<f32>, dly: vec3<f32>) -> vec4<f32> {
  let mode = u32(material.flags.w);
  if (frame.light_color.w > 1.5 && frame.light_color.w < 2.5) {
    let ao = textureSampleLevel(ssao_tex, fx_sampler, in.clip.xy * fx.screen.zw, 0.0).r;
    return vec4<f32>(vec3<f32>(select(1.0, ao, fx.ao.x > 0.5)), out_a);
  }
  if (frame.light_color.w > 0.5 && (frame.light_color.w < 1.5 || frame.light_color.w > 6.5)) { return debug_unsupported(in.clip.xy); }
  let emissive = surface_emissive(h, shader_params(), material.emissive.rgb * textureSample(emissive_tex, material_sampler, material_uv(UV_EMISSIVE, in.uv)).rgb);
  if ((mode & 2u) != 0u) {
    if (frame.light_color.w > 2.5 && frame.light_color.w < 3.5) { return vec4<f32>(base.rgb, out_a); }
    return vec4<f32>(apply_fog(base.rgb + emissive, in.world), out_a);
  }
  var s: Surface;
  s.metallic = clamp(material.mr.x * textureSample(metalness_tex, material_sampler, material_uv(UV_METALNESS, in.uv)).b, 0.0, 1.0);
  s.roughness = clamp(material.mr.y * textureSample(roughness_tex, material_sampler, material_uv(UV_METALNESS, in.uv)).g, 0.04, 1.0);
  s.albedo = base.rgb;
  s.f0 = mix(vec3<f32>(0.04), s.albedo, s.metallic);
  // A leaf lets the sun through (brdf's leaf_transmission), as pbr_fs.wgsl.
  s.translucency = material.coat.z;
  var n = normalize(in.normal);
  if ((mode & 1u) != 0u) {
    n = -normalize(cross(dpx, dpy)) * h.front;
  }
  let v = toward_eye(in.world);
  n *= material_face(h.front);
  let n_geo = n;
  n = normalize(surface_normal(h, shader_params(), n));
  // The decals over the copy, then the rain, snow and dust on it, as on a
  // mesh (decals.wgsl, weather_surface.wgsl; pbr_fs.wgsl).
  let decaled = surface_decals(s, n, n_geo, in.clip.xy, in.world, dpx, dpy);
  let weathered = weather_surface_at(decaled.s, decaled.n, n_geo, in.world, scatter_cavity(in.uv), material.uv[3].w, WEATHER_STANDS);
  s = weathered.s;
  n = weathered.n;
  let styled = (mode & 8u) == 0u;
  var made = StyledSurface(s, n, 0.0, vec3<f32>(1.0));
  if (styled) {
    made = styled_surface(s, n, in.local, dpx, dpy, dlx, dly);
    s = made.s;
    n = made.n;
  }
  if (frame.light_color.w > 2.5 && frame.light_color.w < 3.5) { return vec4<f32>(s.albedo, out_a); }
  let l = normalize(frame.light_dir.xyz);
  let n_dot_l = max(dot(n, l), 0.0);
  var shadow = 1.0;
  // The 2x2 PCF, as main drew every scattered copy: copies are mostly
  // blades and cards, thinner than a texel, and neither closed (they would
  // stop shadowing themselves) nor smooth enough for the tent, which
  // averaged their own shadows away (probe-outdoor's meadow went flat).
  if (frame.light_dir.w > 0.5 && (n_dot_l > 0.0 || s.translucency > 0.0)) {
    shadow = cascade_shadow_lookup(in.world, leaf_shadow_normal(s, n, l), abs(dot(n, l)), false);
  }
  // Inside its crown, what the crown's leaves let through (crown_sun).
  let map_shadow = shadow;
  shadow = crown_sun(shadow, in.crown.y);
  // The sun under the frame's material style (lighting.wgsl style_sun: its
  // toon bands, shadow darkness and tint), as every lit surface's, unless
  // the material is unstyled (mode 8, as pbr_fs.wgsl).
  let band_aa = style_edge(n_dot_l * shadow);
  var sun: StyleSun;
  sun.direct = brdf(s, n, v, l) * frame.light_color.rgb * shadow;
  sun.keep = 1.0;
  if (styled) {
    sun = style_sun(s, n, v, l, shadow, band_aa);
    sun.direct += styled_wrap(made, l, shadow);
  }
  sun.direct += crown_passed_sun(s, map_shadow, in.crown.y);
  // What the material's AO map keeps of the light around the copy.
  var keep = 1.0;
  if (material.flags.y >= 0.0) {
    keep = mix(1.0, textureSample(ao_tex, material_sampler, material_uv(UV_AO, in.uv)).r, material.flags.y);
  }
  // The caustics, the local lights and the light around the copy, as every
  // lit surface's (surface_shade.wgsl); the screen-space reflections unless
  // blended: then it lies over the surface they were traced for.
  let at = ShadeAt(in.world, n, v, in.clip.xy, true, (mode & 4u) == 0u, styled);
  // The light around a leaf inside its crown (crown_ambient), as a model's.
  let direct = surface_direct(s, at, sun.direct, n_dot_l * shadow);
  let around = crown_ambient(surface_ambient(s, at, n), s, clamp(dot(n_geo, v), 1e-4, 1.0), in.crown.x);
  let lit = surface_sum(s, direct, around, emissive + decaled.emissive, material.flags.z, keep * sun.keep);
  if (frame.light_color.w > 2.5) {
    return vec4<f32>(shaded_debug(lit, s.albedo, u32(frame.light_color.w + 0.5)), out_a);
  }
  return vec4<f32>(apply_fog(shaded_color(lit), in.world), out_a);
}

struct ScatterPrepass {
  @location(0) gbuffer: vec4<f32>,
  @location(1) depth: vec4<f32>,
  @location(2) outline: vec4<f32>,
};

@fragment
fn fs_prepass(in: ScatterOut, @builtin(front_facing) front: bool) -> ScatterPrepass {
  let base = surface_color(scatter_hook(in, front), shader_params(), season_mix(material.base_color * in.color, in.season) * textureSample(base_color_tex, material_sampler, material_uv(UV_MAP, in.uv)));
  let flat_n = normalize(cross(dpdxFine(in.world), dpdyFine(in.world)));
  if ((material.emissive.w >= 0.0 && base.a < material.emissive.w) || dither_cut(scatter_hidden(in.fade), in.clip.xy, frame.drawn.w)) {
    discard;
  }
  var n = normalize(in.normal);
  let face = select(-1.0, 1.0, front);
  if ((u32(material.flags.w) & 1u) != 0u) { n = -flat_n * face; }
  n *= material_face(face);
  let cur = in.cur_clip.xy / in.cur_clip.w;
  let prev = in.prev_clip.xy / max(in.prev_clip.w, 1e-6);
  var o: ScatterPrepass;
  o.gbuffer = vec4<f32>(oct_encode(n), (cur - prev) * vec2<f32>(0.5, -0.5));
  // The reflectivity screen-space reflections read, as the lit pass shades
  // the copy (none unlit).
  var metalness = material.mr.x * textureSample(metalness_tex, material_sampler, material_uv(UV_METALNESS, in.uv)).b;
  var roughness = clamp(material.mr.y * textureSample(roughness_tex, material_sampler, material_uv(UV_METALNESS, in.uv)).g, 0.04, 1.0);
  // Wet copies and the puddles on them as the lit pass shades them
  // (weather_surface.wgsl weather_gloss), so reflections see them.
  let unlit = (u32(material.flags.w) & 2u) != 0u;
  if (!unlit) {
    let wet = wet_decal_gloss(in.clip.xy, in.world, n, n, roughness);
    let g = weather_gloss(wet.w, metalness, wet.xyz, n, in.world, scatter_cavity(in.uv), material.uv[3].w, WEATHER_STANDS);
    roughness = g.s.roughness;
    metalness = g.s.metallic;
    o.gbuffer = vec4<f32>(oct_encode(g.n), (cur - prev) * vec2<f32>(0.5, -0.5));
  }
  let reflectivity = select(reflectivity_bits(roughness, metalness), 0.0, unlit);
  o.depth = vec4<f32>(dot(in.world - frame.view_pos.xyz, frame.view_forward.xyz), reflectivity, 0.0, 1.0);
  // The material's outline (the style's, material.rs), as a clump: round
  // the copy, never between its leaves; an unstyled material's (mode bit
  // 8) keeps its width at any distance.
  var width = material.outline.w;
  if ((u32(material.flags.w) & 8u) == 0u) {
    width = style_line_width(width, in.world, true);
  }
  o.outline = outline_bytes(material.outline.rgb, width, in.object, true);
  return o;
}
