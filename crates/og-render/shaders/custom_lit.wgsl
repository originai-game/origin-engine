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
  c.origin = frame.origin.xyz;
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
  return custom_fragment(in);
}

// The order-independent passes' entries (pbr_fs.wgsl `fs_oit`).
@fragment
fn fs_oit(in: CustomOut) -> OitOutput {
  return oit_weighted(custom_fragment(in), in.clip.z);
}

@fragment
fn fs_oit_list(in: CustomOut, @builtin(sample_mask) mask: u32) -> OitOutput {
  let c = custom_fragment(in);
  oit_store(in.clip, c, mask, in.oit_key);
  return oit_weighted(c, in.clip.z);
}

@fragment
fn fs_oit_front(in: CustomOut) -> OitOutput {
  let c = custom_fragment(in);
  return oit_hybrid(c, in.clip);
}

@fragment
fn fs_oit_mark(in: CustomOut) {
  let c = custom_fragment(in);
  oit_front_mark(in.clip, c.a);
}

fn custom_fragment(in: CustomOut) -> vec4<f32> {
  // Before the shader's code, whose branches derivatives may not follow.
  let dpx = dpdx(in.world);
  let dpy = dpdy(in.world);
  let v = custom_view(in.world);
  let out = custom_surface(in, v);
  if (custom_cut(out)) { discard; }
  let mode = u32(material.flags.w);
  let alpha = select(1.0, out.color.a, (mode & 4u) != 0u);
  if (frame.light_color.w > 2.5 && frame.light_color.w < 3.5) { return vec4<f32>(out.color.rgb, alpha); }
  let view = u32(frame.light_color.w + 0.5);
  // An unlit shader has no light to show apart; the cascades', the
  // occlusion's and the terrain's views are other shaders'.
  if (view > 0u && (!CUSTOM_LIT || view < 3u || view > 6u)) { return debug_unsupported(in.clip.xy); }
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
  // The decals over it, then the rain, snow and dust on it, as on every
  // lit mesh (decals.wgsl, weather_surface.wgsl; pbr_fs.wgsl): they land
  // by the mesh's own normal, and the material's weathering scales the
  // weather.
  var n_geo = normalize(in.normal);
  if (dot(n_geo, v) < 0.0) { n_geo = -n_geo; }
  var emissive = out.emissive;
  if (EFFECTS) {
    let decaled = surface_decals(s, n, n_geo, in.clip.xy, in.world, dpx, dpy);
    s = decaled.s;
    n = decaled.n;
    emissive += decaled.emissive;
  }
  let weathered = weather_surface_at(s, n, n_geo, in.world, 0.5, material.uv[3].w, WEATHER_STANDS);
  s = weathered.s;
  n = weathered.n;
  s.coat_normal = n;
  let l = normalize(frame.light_dir.xyz);
  let n_dot_l = max(dot(n, l), 0.0);
  var shadow = 1.0;
  if (frame.light_dir.w > 0.5 && n_dot_l > 0.0) { shadow = cascade_shadow(in.world, n, n_dot_l); }
  let sun = StyleSun(brdf(s, n, v, l) * frame.light_color.rgb * shadow, 1.0);
  // The caustics, the local lights and the light around the surface, as
  // every lit surface's (surface_shade.wgsl), without the frame's material
  // style: the shader's colour is its own. Blended, it lies over the
  // surface the screen's reflections were traced for.
  let at = ShadeAt(in.world, n, v, in.clip.xy, EFFECTS, !oit_pass && (mode & 4u) == 0u, false);
  let lit = surface_shade(s, at, sun, n_dot_l * shadow, emissive, material.flags.z, 1.0);
  if (view > 3u) { return vec4<f32>(shaded_debug(lit, s.albedo, view), alpha); }
  return vec4<f32>(apply_fog(shaded_color(lit), in.world), alpha);
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
  // The reflectivity screen-space reflections read (pbr_prepass.wgsl): a
  // lit shader's gloss and metalness as its lit pass shades them, wet and
  // rippled where the weather lies; none for an unlit shader.
  var reflectivity = 0.0;
  if (CUSTOM_LIT) {
    var n_geo = normalize(in.normal);
    if (dot(n_geo, v) < 0.0) { n_geo = -n_geo; }
    let g = weather_gloss(clamp(out.roughness, 0.04, 1.0), clamp(out.metallic, 0.0, 1.0), n, n_geo, in.world, 0.5, material.uv[3].w, WEATHER_STANDS);
    n = g.n;
    reflectivity = reflectivity_bits(g.s.roughness, g.s.metallic);
  }
  let cur = in.cur_clip.xy / in.cur_clip.w;
  let prev = in.prev_clip.xy / max(in.prev_clip.w, 1e-6);
  var t: CustomPrepassTargets;
  t.gbuffer = vec4<f32>(oct_encode(n), (cur - prev) * vec2<f32>(0.5, -0.5));
  t.depth = vec4<f32>(dot(in.world - frame.view_pos.xyz, frame.view_forward.xyz), reflectivity, 0.0, 1.0);
  // No outline of its own (the scene's outline still lines it).
  t.outline = vec4<f32>(0.0);
  return t;
}
