// The standard lit fragment shader, after pbr.wgsl. The physical
// pipelines use physical.wgsl's instead.

// False in the lean pipelines (pbr_lean.wgsl) whose fragments take no
// screen derivative: a plain material has no alpha cut-off and no
// dissolve, so it never discards, and a shader without a discard keeps
// the early depth test (Apple's hidden surface removal among them): the
// same pixels, each shaded once. Where derivatives are taken (flat
// shading, the frame's decals) the discard stays even though it is never
// taken: on Metal a shader with a discard computes screen derivatives
// differently from one without (flat-shaded normals moved by up to
// 122/255, decal texels by 5/255 in og-gfx runs), and the lean pixels
// must be the full path's.
override DISCARD: bool = true;

@fragment
fn fs(in: VertexOut) -> @location(0) vec4<f32> {
  return lit_fragment(in);
}

// The lit shading of a fragment (`fs`, and `fs_lean`).
fn lit_fragment(in: VertexOut) -> vec4<f32> {
  let uv = stylised_uv(in.uv, in.affine);
  let base_sample = textureSample(base_color_tex, material_sampler, uv);
  // A material shader's hooks (identities with the built-in hooks).
  var hook: Hook;
  hook.world = in.world_pos;
  hook.local = in.local_pos;
  hook.normal = in.normal;
  hook.uv = uv;
  hook.color = in.color;
  hook.time = frame.time.x;
  let params = shader_params();
  let base = surface_color(hook, params, look_tint(material.base_color * base_sample * in.color, in.look));
  var dissolve = vec4<f32>(0.0);
  if (EFFECTS) { dissolve = dissolve_at(uv, in.local_pos); }
  // One discard for the alpha cut-off and the dissolve (see dissolve_at);
  // none in most lean pipelines (DISCARD).
  if (DISCARD && ((material.emissive.w >= 0.0 && base.a < material.emissive.w) || dissolve.w > 0.5)) {
    discard;
  }
  let dissolve_edge = dissolve.rgb;
  let mode = u32(material.flags.w);
  // Blended materials keep their alpha; opaque ones write 1.
  let alpha = select(1.0, base.a, (mode & 4u) != 0u);
  if ((mode & 2u) != 0u) {
    // Unlit (MeshBasicMaterial): the colour plus the emissive term.
    let e = surface_emissive(hook, params, material.emissive.rgb * textureSample(emissive_tex, material_sampler, uv).rgb);
    var unlit = base.rgb + e + dissolve_edge;
    if (EFFECTS && material.toon.w > 0.0) { unlit = mix(unlit, material.flash.rgb, material.toon.w); }
    unlit = look_flash(unlit, in.look);
    return vec4<f32>(apply_fog(unlit, in.world_pos), alpha);
  }
  let metalness_sample = textureSample(metalness_tex, material_sampler, uv);
  let roughness_sample = textureSample(roughness_tex, material_sampler, uv);
  let dpx = dpdx(in.world_pos);
  let dpy = dpdy(in.world_pos);
  var s: Surface;
  s.metallic = clamp(material.mr.x * metalness_sample.b, 0.0, 1.0);
  s.roughness = clamp(material.mr.y * roughness_sample.g, 0.04, 1.0);
  s.albedo = base.rgb;

  var n = normalize(in.normal);
  if ((mode & 1u) != 0u) {
    // Flat shading: the face normal from the position derivatives.
    n = normalize(cross(dpx, dpy));
  }
  // Perspective: towards the eye; orthographic: against the view direction.
  let v = select(normalize(frame.view_pos.xyz - in.world_pos), -frame.view_forward.xyz, frame.view_pos.w > 0.5);
  let n_vertex = n;
  let face = select(1.0, -1.0, dot(n, v) < 0.0);
  n = n * face;
  var decal_emissive = vec3<f32>(0.0);
  if (EFFECTS && fx.decal_info.x > 0.5) {
    let d = apply_decals(in.world_pos, n, s.albedo, s.roughness, dpx, dpy);
    s.albedo = d.albedo;
    s.roughness = clamp(d.roughness, 0.04, 1.0);
    decal_emissive = d.emissive;
  }
  s.f0 = mix(vec3<f32>(0.04), s.albedo, s.metallic);
  let normal_sample = textureSample(normal_tex, material_sampler, uv);
  if (material.flags.x > 0.5) {
    var map = normal_sample.xyz * 2.0 - 1.0;
    if (material.flags.x > 1.5) {
      // Two-channel (BC5, EAC RG11): z from x and y.
      map.z = sqrt(max(1.0 - dot(map.xy, map.xy), 0.0));
    }
    map = vec3<f32>(map.xy * material.mr.zw, map.z);
    // The vertex tangent frame (glTF's: the file's tangents, the cook's
    // MikkTSpace ones, or generated at upload; scene.rs Vertex::tangent).
    var t = in.tangent.xyz - n_vertex * dot(n_vertex, in.tangent.xyz);
    if (dot(t, t) > 1e-12) {
      t = normalize(t);
      let b = cross(n_vertex, t) * select(1.0, -1.0, in.tangent.w < 0.0);
      // A back face flips the whole frame with its normal.
      n = normalize(mat3x3<f32>(t, b, n_vertex) * map) * face;
    }
  }
  n = surface_normal(hook, params, n);
  s.coat = material.coat.x;
  s.coat_roughness = clamp(material.coat.y, 0.04, 1.0);
  s.coat_normal = n_vertex * face;

  // The directional light.
  let l = normalize(frame.light_dir.xyz);
  let n_dot_l = max(dot(n, l), 0.0);
  var shadow = 1.0;
  if (frame.light_dir.w > 0.5 && n_dot_l > 0.0) { shadow = cascade_shadow(in.world_pos, normalize(in.normal), n_dot_l); }
  var color: vec3<f32>;
  if (EFFECTS && material.toon.x > 0.5) {
    color = toon_light(s, n, v, l, shadow) * frame.light_color.rgb;
  } else {
    color = brdf(s, n, v, l) * frame.light_color.rgb * shadow;
  }
  // Caustics where the surface lies under the frame's water (0 without).
  color += s.albedo * frame.light_color.rgb * n_dot_l * shadow * caustics(in.world_pos);
  // The material style's shadow tint: shadowed sunlight moves towards a
  // colour instead of black (the cool or warm shadow of illustration).
  if (frame.shadow_tint.w > 0.0 && n_dot_l > 0.0 && (mode & 8u) == 0u) {
    color += (1.0 - shadow) * frame.shadow_tint.rgb * frame.shadow_tint.w * s.albedo * frame.light_color.rgb * n_dot_l;
  }

  // Point and spot lights: those of this fragment's cluster, in scene order.
  if (frame.params.z > 0.5) {
    let word = cluster_word(cluster_at(in.clip.xy, in.world_pos));
    let first = word & 0xffffffu;
    let count = word >> 24u;
    for (var k = 0u; k < count; k++) {
      color += local_light(light_list[cluster_ref(first + k)], s, in.world_pos, n, v);
    }
  }

  // Image-based light (split sum).
  let n_dot_v = max(dot(n, v), 1e-4);
  let f_ibl = f_schlick_roughness(n_dot_v, s.f0, s.roughness);
  let kd_ibl = (vec3<f32>(1.0) - f_ibl) * (1.0 - s.metallic);
  let irradiance = textureSample(irradiance_map, ibl_sampler, n).rgb;
  let diffuse_ibl = irradiance * s.albedo;
  let r = reflect(-v, n);
  let mip = s.roughness * frame.params.y;
  let prefiltered = textureSampleLevel(prefiltered_map, ibl_sampler, r, mip).rgb;
  let lut = textureSample(brdf_lut, ibl_sampler, vec2<f32>(n_dot_v, s.roughness)).rg;
  let specular_ibl = prefiltered * (f_ibl * lut.x + lut.y);
  let ao_sample = textureSample(ao_tex, material_sampler, uv).r;
  var ibl_scale = frame.params.x * material.flags.z;
  if (material.flags.y >= 0.0) {
    // Ambient occlusion darkens only the image-based light.
    ibl_scale *= mix(1.0, ao_sample, material.flags.y);
  }
  if (EFFECTS && fx.ao.x > 0.5) {
    // Screen-space ambient occlusion, likewise.
    let ssao_sample = textureSampleLevel(ssao_tex, fx_sampler, in.clip.xy * fx.screen.zw, 0.0).r;
    ibl_scale *= mix(1.0, ssao_sample, fx.ao.y);
  }
  var ibl = kd_ibl * diffuse_ibl + specular_ibl;
  // Branch on the material itself, not on `s`: Tint's uniformity analysis
  // takes all of `s` as varying once a member comes from a texture, and the
  // LUT is read without derivatives (it has one level).
  if (material.coat.x > 0.0) {
    // The coat's reflection of the environment, and what it takes from the base.
    let c_dot_v = max(dot(s.coat_normal, v), 1e-4);
    let fc = f_schlick(c_dot_v, vec3<f32>(0.04)).x * s.coat;
    let rc = reflect(-v, s.coat_normal);
    let coat_env = textureSampleLevel(prefiltered_map, ibl_sampler, rc, s.coat_roughness * frame.params.y).rgb;
    let coat_lut = textureSampleLevel(brdf_lut, ibl_sampler, vec2<f32>(c_dot_v, s.coat_roughness), 0.0).rg;
    ibl = ibl * (1.0 - fc) + coat_env * (0.04 * coat_lut.x + coat_lut.y) * s.coat;
  }
  color += ibl * ibl_scale;

  color += surface_emissive(hook, params, material.emissive.rgb * textureSample(emissive_tex, material_sampler, uv).rgb);
  if (EFFECTS && fx.decal_info.x > 0.5) { color += decal_emissive; }
  if (EFFECTS && material.rim.w > 0.0) { color += dissolve_edge; }
  if (EFFECTS && material.toon.y > 0.0) {
    // Rim (fresnel) light at grazing angles.
    color += material.rim.rgb * material.toon.y * pow(1.0 - n_dot_v, material.toon.z);
  }
  if (EFFECTS && material.toon.w > 0.0) {
    color = mix(color, material.flash.rgb, material.toon.w);
  }
  color = look_flash(color, in.look);

  if (frame.light_color.w > 1.5) {
    // Debug: the ambient occlusion term.
    let ssao = textureSampleLevel(ssao_tex, fx_sampler, in.clip.xy * fx.screen.zw, 0.0).r;
    return vec4<f32>(vec3<f32>(select(1.0, ssao, fx.ao.x > 0.5)), 1.0);
  }
  if (frame.light_color.w > 0.5) {
    // Debug: the cascade covering this pixel.
    let c = cascade_index(in.world_pos);
    var tint = vec3<f32>(0.0);
    if (c == 0u) { tint = vec3<f32>(1.0, 0.0, 0.0); }
    if (c == 1u) { tint = vec3<f32>(0.0, 1.0, 0.0); }
    if (c == 2u) { tint = vec3<f32>(0.0, 0.0, 1.0); }
    return vec4<f32>(tint * 4.0, 1.0);
  }
  return vec4<f32>(apply_fog(color, in.world_pos), alpha);
}

