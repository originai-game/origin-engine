// A full custom shader's lit and prepass entries (custom.rs), after
// custom_common.wgsl and the shader's code: the names and vertex buffers
// of the built-in lit shader's (`vs`, `vs_skinned`, `fs`, `vs_prepass`,
// `vs_prepass_skinned`, `fs_prepass`), so its pipelines are the hooked
// materials'. `CUSTOM_LIT` (prepended): lit by the frame's lights, or the
// colour as returned.

override EFFECTS: bool = true;

fn camera() -> Camera {
  var c: Camera;
  c.view_proj = frame.view_proj;
  c.position = frame.view_pos.xyz;
  c.forward = frame.view_forward.xyz;
  c.time = frame.time.x;
  return c;
}

fn project(world: vec3<f32>) -> vec4<f32> {
  return frame.view_proj * vec4<f32>(world, 1.0);
}

fn custom_view(p: vec3<f32>) -> vec3<f32> {
  return select(normalize(frame.view_pos.xyz - p), -frame.view_forward.xyz, frame.view_pos.w > 0.5);
}

@vertex
fn vs(v: CustomVertexIn, i: CustomInstanceIn) -> CustomOut {
  return custom_plain(v, i);
}

@vertex
fn vs_skinned(@builtin(vertex_index) vi: u32, v: CustomVertexIn, i: CustomSkinnedIn, s: CustomSkinIn) -> CustomOut {
  return custom_skinned(vi, v, i, s);
}

@fragment
fn fs(in: CustomOut) -> @location(0) vec4<f32> {
  let v = custom_view(in.world);
  let out = custom_surface(in, v);
  if (custom_cut(out)) { discard; }
  let mode = u32(material.flags.w);
  let alpha = select(1.0, out.color.a, (mode & 4u) != 0u);
  if (!CUSTOM_LIT) {
    return vec4<f32>(apply_fog(out.color.rgb + out.emissive, in.world), alpha);
  }
  var s: Surface;
  s.albedo = out.color.rgb;
  s.metallic = clamp(out.metallic, 0.0, 1.0);
  s.roughness = clamp(out.roughness, 0.04, 1.0);
  s.f0 = mix(vec3<f32>(0.04), s.albedo, s.metallic);
  s.coat = 0.0;
  s.coat_roughness = 1.0;
  var n = normalize(out.normal);
  if (dot(n, v) < 0.0) { n = -n; }
  s.coat_normal = n;
  let l = normalize(frame.light_dir.xyz);
  let n_dot_l = max(dot(n, l), 0.0);
  var shadow = 1.0;
  if (frame.light_dir.w > 0.5 && n_dot_l > 0.0) { shadow = cascade_shadow(in.world, n, n_dot_l); }
  var color = brdf(s, n, v, l) * frame.light_color.rgb * shadow;
  if (frame.params.z > 0.5) {
    let word = cluster_word(cluster_at(in.clip.xy, in.world));
    let first = word & 0xffffffu;
    let count = word >> 24u;
    // Rect lights through LTC with their shadows, points and spots with
    // their cookies (area.wgsl), as terrain, scatter and the 3D fluid.
    for (var k = 0u; k < count; k++) {
      color += surface_local_light(light_list[cluster_ref(first + k)], s, in.world, n, v);
    }
  }
  let n_dot_v = max(dot(n, v), 1e-4);
  let f_ibl = f_schlick_roughness(n_dot_v, s.f0, s.roughness);
  let kd_ibl = (vec3<f32>(1.0) - f_ibl) * (1.0 - s.metallic);
  // The environment, or the batch's reflection probe blended with it (probes.rs).
  let irradiance = ambient_light_level(n, in.world);
  let prefiltered = reflected_light(reflect(-v, n), in.world, s.roughness * frame.params.y);
  let lut = textureSampleLevel(brdf_lut, ibl_sampler, vec2<f32>(n_dot_v, s.roughness), 0.0).rg;
  color += (kd_ibl * irradiance * s.albedo + prefiltered * (f_ibl * lut.x + lut.y)) * frame.params.x * material.flags.z;
  color += out.emissive;
  return vec4<f32>(apply_fog(color, in.world), alpha);
}

// The depth prepass (pbr_prepass.wgsl's targets) through the vertex entry.
struct CustomPrepassOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
  @location(3) color: vec4<f32>,
  @location(4) local: vec3<f32>,
  @location(5) data: vec4<f32>,
  @location(6) cur_clip: vec4<f32>,
  @location(7) prev_clip: vec4<f32>,
};

struct CustomPrevIn {
  @location(11) p0: vec3<f32>,
  @location(12) p1: vec3<f32>,
  @location(13) p2: vec3<f32>,
  @location(14) p3: vec3<f32>,
};

fn custom_prepass(o: CustomOut, prev_world: vec3<f32>) -> CustomPrepassOut {
  var p: CustomPrepassOut;
  p.clip = o.clip;
  p.world = o.world;
  p.normal = o.normal;
  p.uv = o.uv;
  p.color = o.color;
  p.local = o.local;
  p.data = o.data;
  p.cur_clip = fx.view_proj_unjittered * vec4<f32>(o.world, 1.0);
  p.prev_clip = fx.prev_view_proj * vec4<f32>(prev_world, 1.0);
  return p;
}

@vertex
fn vs_prepass(v: CustomVertexIn, i: CustomInstanceIn, prev: CustomPrevIn) -> CustomPrepassOut {
  let o = custom_plain(v, i);
  // The vertex entry's displacement carried onto last frame's transform.
  let now = instance_matrix(i.m0, i.m1, i.m2, i.m3) * vec4<f32>(v.position, 1.0);
  let before = instance_matrix(prev.p0, prev.p1, prev.p2, prev.p3) * vec4<f32>(v.position, 1.0);
  return custom_prepass(o, o.world + before.xyz - now.xyz);
}

@vertex
fn vs_prepass_skinned(@builtin(vertex_index) vi: u32, v: CustomVertexIn, i: CustomSkinnedIn, s: CustomSkinIn) -> CustomPrepassOut {
  let o = custom_skinned(vi, v, i, s);
  return custom_prepass(o, o.world);
}

struct CustomPrepassTargets {
  @location(0) gbuffer: vec4<f32>,
  @location(1) depth: vec4<f32>,
  @location(2) outline: vec4<f32>,
};

@fragment
fn fs_prepass(in: CustomPrepassOut) -> CustomPrepassTargets {
  var o: CustomOut;
  o.clip = in.clip;
  o.world = in.world;
  o.normal = in.normal;
  o.uv = in.uv;
  o.color = in.color;
  o.local = in.local;
  o.data = in.data;
  let v = custom_view(in.world);
  let out = custom_surface(o, v);
  if (custom_cut(out)) { discard; }
  var n = normalize(out.normal);
  if (dot(n, v) < 0.0) { n = -n; }
  let cur = in.cur_clip.xy / in.cur_clip.w;
  let prev = in.prev_clip.xy / max(in.prev_clip.w, 1e-6);
  var t: CustomPrepassTargets;
  t.gbuffer = vec4<f32>(oct_encode(n), (cur - prev) * vec2<f32>(0.5, -0.5));
  t.depth = vec4<f32>(dot(in.world - frame.view_pos.xyz, frame.view_forward.xyz), 0.0, 0.0, 1.0);
  // No outline of its own (the scene's outline still lines it).
  t.outline = vec4<f32>(0.0);
  return t;
}
