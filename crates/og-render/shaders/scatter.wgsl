// The scattered copies' lit and prepass passes. Prepended by frame.wgsl,
// the light tables (clusters.rs), noise.wgsl, material.wgsl (group 1: the copies' StandardMaterial),
// lighting.wgsl and scatter_common.wgsl (group 2).
//
// Shading: the material's colour (factor, texture and the mesh's vertex
// colours), alpha cut-off, roughness and metalness, emissive, unlit and
// flat modes, lit by the directional light with its cascades, the local
// lights, the image-based light and caustics under water, then fog. Normal
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
  h.eye = frame.view_pos.xyz;
  h.front = select(-1.0, 1.0, front);
  hook_front = h.front;
  return h;
}

@vertex
fn vs(v: ScatterVertex, i: ScatterInstance) -> ScatterOut {
  let model = instance_matrix(i.m0, i.m1, i.m2, i.m3);
  var o: ScatterOut;
  o.fade = scatter_fade(i.m3);
  var world = model * vec4<f32>(v.position, 1.0);
  o.normal = normalize((model * vec4<f32>(v.normal, 0.0)).xyz);
  var h: Hook;
  h.world = world.xyz;
  h.local = v.position;
  h.normal = o.normal;
  h.uv = v.uv;
  h.color = v.color;
  h.time = frame.time.x;
  h.eye = frame.view_pos.xyz;
  h.front = 1.0;
  world = vec4<f32>(world.xyz + vertex_offset(h, shader_params()), world.w);
  o.clip = select(SCATTER_GONE, mul_point(frame.view_proj, world), o.fade > 0.0);
  o.world = world.xyz;
  o.local = v.position;
  o.uv = v.uv;
  o.color = v.color;
  // The copies stand still: only the camera moves them on screen.
  o.cur_clip = mul_point(scatter.view_proj, world);
  o.prev_clip = mul_point(scatter.prev_view_proj, world);
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
  let base = surface_color(h, shader_params(), material.base_color * textureSample(base_color_tex, material_sampler, in.uv) * in.color);
  let dpx = dpdx(in.world);
  let dpy = dpdy(in.world);
  // Alpha to coverage (mode 16, an alpha-tested material, as pbr_fs.wgsl): alpha past the cut-off over one pixel's change
  // of it is the share of the MSAA samples covered (three.js' alphaToCoverage on grass and foliage cards).
  let mode = u32(material.flags.w);
  let a2c = (mode & 16u) != 0u && material.emissive.w >= 0.0;
  let coverage = clamp((base.a - material.emissive.w) / max(fwidth(base.a), 1e-4) + 0.5, 0.0, 1.0);
  // One discard for the cut-off and the fade (Metal ignored a second one).
  if ((material.emissive.w >= 0.0 && select(base.a < material.emissive.w, coverage <= 0.0, a2c)) || scatter_dropped(in.clip.xy, in.fade)) {
    discard;
  }
  let out_a = select(1.0, coverage, a2c);
  let emissive = surface_emissive(h, shader_params(), material.emissive.rgb * textureSample(emissive_tex, material_sampler, in.uv).rgb);
  if ((mode & 2u) != 0u) {
    return vec4<f32>(apply_fog(base.rgb + emissive, in.world), out_a);
  }
  var s: Surface;
  s.metallic = clamp(material.mr.x * textureSample(metalness_tex, material_sampler, in.uv).b, 0.0, 1.0);
  s.roughness = clamp(material.mr.y * textureSample(roughness_tex, material_sampler, in.uv).g, 0.04, 1.0);
  s.albedo = base.rgb;
  s.f0 = mix(vec3<f32>(0.04), s.albedo, s.metallic);
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
  if (frame.light_dir.w > 0.5 && n_dot_l > 0.0) { shadow = cascade_shadow(in.world, n, n_dot_l); }
  var color = brdf(s, n, v, l) * frame.light_color.rgb * shadow;
  color += s.albedo * frame.light_color.rgb * n_dot_l * shadow * caustics(in.world);
  // Point, spot and rect lights: this fragment's cluster's, or every light
  // in a reflection probe's face (area.wgsl surface_local_lights).
  color += surface_local_lights(in.clip.xy, s, in.world, n, v);
  var ssao_scale = 1.0;
  var ao_all = 1.0;  // also scales the emitted light (pbr_fs.wgsl)
  if (scatter_fx.ao.x > 0.5) {
    let ssao = textureSampleLevel(scatter_ssao, scatter_fx_sampler, in.clip.xy * scatter_fx.screen.zw, 0.0).r;
    ssao_scale = mix(mix(1.0, ssao, scatter_fx.ao.y), mix(0.35, 1.0, ssao), scatter_fx.ao.z);
    ao_all = mix(1.0, mix(0.35, 1.0, ssao), scatter_fx.ao.z);
    color *= ao_all;
    // Debug (OG_DEBUG_VIEW=2): the ambient occlusion term, as pbr_fs.wgsl.
    if (frame.light_color.w > 1.5 && frame.light_color.w < 2.5) { return vec4<f32>(vec3<f32>(ssao), 1.0); }
  }
  let n_dot_v = max(dot(n, v), 1e-4);
  let f_ibl = f_schlick_roughness(n_dot_v, s.f0, s.roughness);
  let kd_ibl = (vec3<f32>(1.0) - f_ibl) * (1.0 - s.metallic);
  // The environment, or the probe holding the chunk blended with it (probes.rs),
  // plus the hemisphere light.
  let irradiance = ambient_light(n, in.world) + hemi_irradiance(n);
  let r = reflect(-v, n);
  let prefiltered = reflected_light(r, in.world, s.roughness * frame.params.y);
  let lut = textureSample(brdf_lut, ibl_sampler, vec2<f32>(n_dot_v, s.roughness)).rg;
  var ibl_scale = frame.params.x * material.flags.z * ssao_scale;
  if (material.flags.y >= 0.0) {
    ibl_scale *= mix(1.0, textureSample(ao_tex, material_sampler, in.uv).r, material.flags.y);
  }
  color += (kd_ibl * irradiance * s.albedo + prefiltered * (s.f0 * lut.x + lut.y)) * ibl_scale;
  color += emissive * ao_all;
  return vec4<f32>(apply_fog(color, in.world), out_a);
}

struct ScatterPrepass {
  @location(0) gbuffer: vec4<f32>,
  @location(1) depth: vec4<f32>,
  @location(2) outline: vec4<f32>,
};

@fragment
fn fs_prepass(in: ScatterOut, @builtin(front_facing) front: bool) -> ScatterPrepass {
  let base = surface_color(scatter_hook(in, front), shader_params(), material.base_color * textureSample(base_color_tex, material_sampler, in.uv) * in.color);
  let flat_n = normalize(cross(dpdx(in.world), dpdy(in.world)));
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
  let metalness = material.mr.x * textureSample(metalness_tex, material_sampler, in.uv).b;
  let roughness = clamp(material.mr.y * textureSample(roughness_tex, material_sampler, in.uv).g, 0.04, 1.0);
  let reflectivity = select(reflectivity_bits(roughness, metalness), 0.0, (u32(material.flags.w) & 2u) != 0u);
  o.depth = vec4<f32>(dot(in.world - frame.view_pos.xyz, frame.view_forward.xyz), reflectivity, 0.0, 1.0);
  o.outline = vec4<f32>(0.0);
  return o;
}
