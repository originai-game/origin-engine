// The scattered copies' lit and prepass passes. Prepended by frame.wgsl,
// the light tables (clusters.rs), noise.wgsl, material.wgsl (group 1: the copies' StandardMaterial),
// lighting.wgsl and scatter_common.wgsl (group 2).
//
// Shading: the material's colour (factor, texture and the mesh's vertex
// colours), alpha cut-off, roughness and metalness, emissive, unlit and
// flat modes, lit by the directional light with its cascades, the local
// lights, the image-based light and caustics under water, then fog. A leaf
// material is lit as on a model: its normals domed about the mesh's canopy
// (domeNormals; scatter.canopy, the mesh's foliage), the sun let through
// by its translucency. Normal
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
  // How much of the copy draws as its mesh (scatter_near; scatter_dropped).
  @location(4) @interpolate(flat) fade: f32,
  @location(5) cur_clip: vec4<f32>,
  @location(6) prev_clip: vec4<f32>,
  @location(7) local: vec3<f32>,
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
  let size = scatter_scale(i.m3);
  let model = scatter_shrunk(instance_matrix(i.m0, i.m1, i.m2, i.m3), size);
  var o: ScatterOut;
  o.fade = scatter_near(i.m3);
  let scale = length(i.m1) * size;
  let sway = scatter_sway(v.position, i.m3, scale, false);
  var world = model * vec4<f32>(v.position, 1.0) + vec4<f32>(sway, 0.0);
  // A leaf's canopy dome (the material's domeNormals, coat.w), as a model's
  // leaves take it (meshlet.wgsl vs_meshlet). The vertex stands for its
  // leaf's anchor: a copy carries no foliage stream, and a leaf is small
  // beside its canopy.
  let dome = foliage_normal(v.normal, v.position, scatter.canopy.xyz, material.coat.w * scatter.canopy.w);
  o.normal = normalize((model * vec4<f32>(dome, 0.0)).xyz);
  var h: Hook;
  h.world = world.xyz;
  h.local = v.position;
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
  o.local = v.position;
  o.uv = v.uv;
  o.color = v.color;
  // The camera and the wind move the copies on screen: where the vertex
  // was swayed to the previous frame.
  o.cur_clip = mul_point(scatter.view_proj, world);
  let prev_sway = scatter_sway(v.position, i.m3, scale, true);
  h.world = (model * vec4<f32>(v.position, 1.0)).xyz + prev_sway;
  h.time = frame.time.w;
  h.wind = frame.foliage_wind[1][0];
  let previous = h.world + vertex_offset(h, previous_shader_params());
  o.prev_clip = mul_point(scatter.prev_view_proj, vec4<f32>(previous, 1.0));
  return o;
}

// The lit pass's effect group (`Fx` in pbr.wgsl, its head): the screen-space AO, which pbr_fs applies to the
// image-based light and, by `ao.z`, the direct light. Without it grass and flowers missed the contact darkening
// three.js' post AO puts on everything (grass tufts came out grey where the original's are dark green).
struct ScatterFx {
  view_proj_unjittered: mat4x4<f32>,
  prev_view_proj: mat4x4<f32>,
  screen: vec4<f32>,
  ao: vec4<f32>,
};
@group(3) @binding(0) var<uniform> scatter_fx: ScatterFx;
@group(3) @binding(1) var scatter_ssao: texture_2d<f32>;
@group(3) @binding(3) var scatter_fx_sampler: sampler;

@fragment
fn fs(in: ScatterOut, @builtin(front_facing) front: bool) -> @location(0) vec4<f32> {
  let h = scatter_hook(in, front);
  let base = surface_color(h, shader_params(), material.base_color * textureSample(base_color_tex, material_sampler, material_uv(UV_MAP, in.uv)) * in.color);
  let dpx = dpdxFine(in.world);
  let dpy = dpdyFine(in.world);
  // Alpha to coverage (mode 16, an alpha-tested material, as pbr_fs.wgsl): alpha past the cut-off over one pixel's change
  // of it is the share of the MSAA samples covered (three.js' alphaToCoverage on grass and foliage cards).
  let mode = u32(material.flags.w);
  let a2c = (mode & 16u) != 0u && material.emissive.w >= 0.0;
  let coverage = clamp((base.a - material.emissive.w) / max(fwidthFine(base.a), 1e-4) + 0.5, 0.0, 1.0);
  // One discard for the cut-off and the fade (Metal ignored a second one).
  if ((material.emissive.w >= 0.0 && select(base.a < material.emissive.w, coverage <= 0.0, a2c)) || scatter_dropped(in.clip.xy, in.fade)) {
    discard;
  }
  let out_a = select(1.0, coverage, a2c);
  if (frame.light_color.w > 2.5 && frame.light_color.w < 3.5) { return vec4<f32>(base.rgb, out_a); }
  if (frame.light_color.w > 1.5 && frame.light_color.w < 2.5) {
    let ao = textureSampleLevel(scatter_ssao, scatter_fx_sampler, in.clip.xy * scatter_fx.screen.zw, 0.0).r;
    return vec4<f32>(vec3<f32>(select(1.0, ao, scatter_fx.ao.x > 0.5)), out_a);
  }
  if (frame.light_color.w > 0.5 && (frame.light_color.w < 3.5 || frame.light_color.w > 6.5)) { return debug_unsupported(in.clip.xy); }
  let emissive = surface_emissive(h, shader_params(), material.emissive.rgb * textureSample(emissive_tex, material_sampler, material_uv(UV_EMISSIVE, in.uv)).rgb);
  if ((mode & 2u) != 0u) {
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
    n = normalize(cross(dpx, dpy));
  }
  let v = select(normalize(frame.view_pos.xyz - in.world), -frame.view_forward.xyz, frame.view_pos.w > 0.5);
  if (dot(n, v) < 0.0) { n = -n; }
  n = normalize(surface_normal(h, shader_params(), n));
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
  var color = brdf(s, n, v, l) * frame.light_color.rgb * shadow;
  color += s.albedo * frame.light_color.rgb * n_dot_l * shadow * caustics(in.world);
  // Point, spot and rect lights: this fragment's cluster's, or every light
  // in a reflection probe's face (area.wgsl surface_local_lights).
  color += surface_local_lights(in.clip.xy, s, in.world, n, v);
  var ssao_scale = 1.0;
  var ao_all = 1.0;  // also scales the emitted light (pbr_fs.wgsl)
  // Only with PostEffects.aoDirect above 0: without it the copies keep their
  // look of before the AO reached them.
  if (scatter_fx.ao.x > 0.5 && scatter_fx.ao.z > 0.0) {
    let ssao = textureSampleLevel(scatter_ssao, scatter_fx_sampler, in.clip.xy * scatter_fx.screen.zw, 0.0).r;
    ssao_scale = mix(mix(1.0, ssao, scatter_fx.ao.y), mix(0.35, 1.0, ssao), scatter_fx.ao.z);
    ao_all = mix(1.0, mix(0.35, 1.0, ssao), scatter_fx.ao.z);
    color *= ao_all;
    // Debug (DebugView::AmbientOcclusion): the ambient occlusion term, as pbr_fs.wgsl.
    if (frame.light_color.w > 1.5 && frame.light_color.w < 2.5) { return vec4<f32>(vec3<f32>(ssao), 1.0); }
  }
  let n_dot_v = max(dot(n, v), 1e-4);
  let f_ibl = f_schlick_roughness(n_dot_v, s.f0, s.roughness);
  let kd_ibl = (vec3<f32>(1.0) - f_ibl) * (1.0 - s.metallic);
  // The environment, or the probe holding the chunk blended with it (probes.rs),
  // at its intensity, plus the hemisphere light at its own.
  let irradiance = ambient_light(n, in.world) * frame.params.x + hemi_irradiance(n);
  let r = reflect(-v, n);
  // With the screen-space reflections over it (frame.wgsl ssr_reflected),
  // unless blended: then it lies over the surface they were traced for.
  var prefiltered = reflected_light(r, in.world, s.roughness * frame.params.y) * frame.params.x;
  if ((u32(material.flags.w) & 4u) == 0u) { prefiltered = ssr_reflected(prefiltered, in.world, in.clip.xy); }
  let lut = textureSample(brdf_lut, ibl_sampler, vec2<f32>(n_dot_v, s.roughness)).rg;
  let dbg_direct = color;  // the direct light, for the debug views below (fork s68)
  var ibl_scale = material.flags.z * ssao_scale;
  if (material.flags.y >= 0.0) {
    ibl_scale *= mix(1.0, textureSample(ao_tex, material_sampler, material_uv(UV_AO, in.uv)).r, material.flags.y);
  }
  color += (kd_ibl * irradiance * s.albedo + prefiltered * (s.f0 * lut.x + lut.y)) * ibl_scale;  // Karis split sum (pbr_fs.wgsl)
  color += emissive * ao_all;
  if (frame.light_color.w > 2.5) {
    // Debug (DebugView::lit_code, as pbr_fs.wgsl): 3 albedo, 4 diffuse image-based light, 5 specular image-based
    // light (what the split sum above added), 6 direct light; the copies showed their whole lit colour in these views
    // (fork s68).
    let m = u32(frame.light_color.w + 0.5);
    let dbg_diffuse = kd_ibl * irradiance * s.albedo * ibl_scale;
    if (m == 3u) { return vec4<f32>(s.albedo, out_a); }
    if (m == 4u) { return vec4<f32>(dbg_diffuse, out_a); }
    if (m == 5u) { return vec4<f32>(color - dbg_direct - dbg_diffuse - emissive * ao_all, out_a); }
    return vec4<f32>(dbg_direct, out_a);
  }
  return vec4<f32>(apply_fog(color, in.world), out_a);
}

struct ScatterPrepass {
  @location(0) gbuffer: vec4<f32>,
  @location(1) depth: vec4<f32>,
  @location(2) outline: vec4<f32>,
};

@fragment
fn fs_prepass(in: ScatterOut, @builtin(front_facing) front: bool) -> ScatterPrepass {
  let base = surface_color(scatter_hook(in, front), shader_params(), material.base_color * textureSample(base_color_tex, material_sampler, material_uv(UV_MAP, in.uv)) * in.color);
  let flat_n = normalize(cross(dpdxFine(in.world), dpdyFine(in.world)));
  if ((material.emissive.w >= 0.0 && base.a < material.emissive.w) || scatter_dropped(in.clip.xy, in.fade)) {
    discard;
  }
  var n = normalize(in.normal);
  if ((u32(material.flags.w) & 1u) != 0u) { n = flat_n; }
  let v = select(normalize(frame.view_pos.xyz - in.world), -frame.view_forward.xyz, frame.view_pos.w > 0.5);
  if (dot(n, v) < 0.0) { n = -n; }
  let cur = in.cur_clip.xy / in.cur_clip.w;
  let prev = in.prev_clip.xy / max(in.prev_clip.w, 1e-6);
  var o: ScatterPrepass;
  o.gbuffer = vec4<f32>(oct_encode(n), (cur - prev) * vec2<f32>(0.5, -0.5));
  // The reflectivity screen-space reflections read, as the lit pass shades
  // the copy (none unlit).
  let metalness = material.mr.x * textureSample(metalness_tex, material_sampler, material_uv(UV_METALNESS, in.uv)).b;
  let roughness = clamp(material.mr.y * textureSample(roughness_tex, material_sampler, material_uv(UV_METALNESS, in.uv)).g, 0.04, 1.0);
  let reflectivity = select(reflectivity_bits(roughness, metalness), 0.0, (u32(material.flags.w) & 2u) != 0u);
  o.depth = vec4<f32>(dot(in.world - frame.view_pos.xyz, frame.view_forward.xyz), reflectivity, 0.0, 1.0);
  o.outline = vec4<f32>(0.0);
  return o;
}
