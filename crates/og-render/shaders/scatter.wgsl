// The scattered copies' lit and prepass passes. Prepended by frame.wgsl,
// the light tables (clusters.rs), noise.wgsl, material.wgsl (group 1: the copies' StandardMaterial),
// lighting.wgsl and scatter_common.wgsl (group 2).
//
// Shading: the material's colour (factor, texture and the mesh's vertex
// colours), alpha cut-off, roughness and metalness, emissive, unlit and
// flat modes, lit by the directional light with its cascades, the local
// lights, the image-based light and caustics under water, then fog. Normal
// maps and the material effects (toon, rim, dissolve) are not drawn on
// scattered copies.

struct ScatterOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
  @location(3) color: vec4<f32>,
  @location(4) @interpolate(flat) fade: f32,
  @location(5) cur_clip: vec4<f32>,
  @location(6) prev_clip: vec4<f32>,
};

@vertex
fn vs(v: ScatterVertex, i: ScatterInstance) -> ScatterOut {
  let model = instance_matrix(i.m0, i.m1, i.m2, i.m3);
  var o: ScatterOut;
  o.fade = scatter_fade(i.m3);
  let world = model * vec4<f32>(v.position, 1.0);
  o.clip = select(SCATTER_GONE, mul_point(frame.view_proj, world), o.fade > 0.0);
  o.world = world.xyz;
  o.normal = normalize((model * vec4<f32>(v.normal, 0.0)).xyz);
  o.uv = v.uv;
  o.color = v.color;
  // The copies stand still: only the camera moves them on screen.
  o.cur_clip = mul_point(scatter.view_proj, world);
  o.prev_clip = mul_point(scatter.prev_view_proj, world);
  return o;
}

fn scatter_local_light(light: LocalLight, s: Surface, p: vec3<f32>, n: vec3<f32>, v: vec3<f32>) -> vec3<f32> {
  let to_light = light.position_range.xyz - p;
  let d2 = max(dot(to_light, to_light), 1e-4);
  let range = light.position_range.w;
  let ratio = d2 / (range * range);
  let window = clamp(1.0 - ratio * ratio, 0.0, 1.0);
  var atten = window * window / d2;
  let l = to_light * inverseSqrt(d2);
  if (light.color_kind.w > 0.5) {
    let cos_angle = dot(-l, light.direction_slot.xyz);
    let t = clamp((cos_angle - light.cone.x) * light.cone.y, 0.0, 1.0);
    atten *= t * t;
  }
  if (atten <= 0.0 || dot(n, l) <= 0.0) { return vec3<f32>(0.0); }
  return brdf(s, n, v, l) * light.color_kind.rgb * atten * local_shadow(light, p, n);
}

@fragment
fn fs(in: ScatterOut) -> @location(0) vec4<f32> {
  let base = material.base_color * textureSample(base_color_tex, material_sampler, in.uv) * in.color;
  let dpx = dpdx(in.world);
  let dpy = dpdy(in.world);
  // One discard for the cut-off and the fade (Metal ignored a second one).
  if ((material.emissive.w >= 0.0 && base.a < material.emissive.w) || scatter_dropped(in.clip.xy, in.fade)) {
    discard;
  }
  let mode = u32(material.flags.w);
  let emissive = material.emissive.rgb * textureSample(emissive_tex, material_sampler, in.uv).rgb;
  if ((mode & 2u) != 0u) {
    return vec4<f32>(apply_fog(base.rgb + emissive, in.world), 1.0);
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
  let l = normalize(frame.light_dir.xyz);
  let n_dot_l = max(dot(n, l), 0.0);
  var shadow = 1.0;
  if (frame.light_dir.w > 0.5 && n_dot_l > 0.0) { shadow = cascade_shadow(in.world, n, n_dot_l); }
  var color = brdf(s, n, v, l) * frame.light_color.rgb * shadow;
  color += s.albedo * frame.light_color.rgb * n_dot_l * shadow * caustics(in.world);
  // Point and spot lights: those of this fragment's cluster (clusters.rs).
  if (frame.params.z > 0.5) {
    let word = cluster_word(cluster_at(in.clip.xy, in.world));
    let first = word & 0xffffffu;
    let light_count = word >> 24u;
    for (var k = 0u; k < light_count; k++) {
      color += scatter_local_light(light_list[cluster_ref(first + k)], s, in.world, n, v);
    }
  }
  let n_dot_v = max(dot(n, v), 1e-4);
  let f_ibl = f_schlick_roughness(n_dot_v, s.f0, s.roughness);
  let kd_ibl = (vec3<f32>(1.0) - f_ibl) * (1.0 - s.metallic);
  // The environment, or the probe holding the chunk blended with it (probes.rs).
  let irradiance = ambient_light(n, in.world);
  let r = reflect(-v, n);
  let prefiltered = reflected_light(r, in.world, s.roughness * frame.params.y);
  let lut = textureSample(brdf_lut, ibl_sampler, vec2<f32>(n_dot_v, s.roughness)).rg;
  var ibl_scale = frame.params.x * material.flags.z;
  if (material.flags.y >= 0.0) {
    ibl_scale *= mix(1.0, textureSample(ao_tex, material_sampler, in.uv).r, material.flags.y);
  }
  color += (kd_ibl * irradiance * s.albedo + prefiltered * (f_ibl * lut.x + lut.y)) * ibl_scale;
  color += emissive;
  return vec4<f32>(apply_fog(color, in.world), 1.0);
}

struct ScatterPrepass {
  @location(0) gbuffer: vec4<f32>,
  @location(1) depth: vec4<f32>,
  @location(2) outline: vec4<f32>,
};

@fragment
fn fs_prepass(in: ScatterOut) -> ScatterPrepass {
  let base = material.base_color * textureSample(base_color_tex, material_sampler, in.uv) * in.color;
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
