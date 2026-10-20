// The lean lit path, after pbr_prepass.wgsl: the entry points a plain
// material draws through (no texture, no effect, no outline, no material
// shader, opaque, not physical, no alpha cut; `GpuMaterial::lean` in
// lib.rs). Its vertex stages read only the attributes they use from the
// shared vertex buffer and hand on the fewest varyings: the world
// position, normal and colour, 10 floats against the full VertexOut's 23
// (the prepass's: position and normal, and its two motion clips, 14
// against 23). Positions come from pbr.wgsl's `lit_place`; the fragment
// stages shade with pbr_fs.wgsl's and pbr_prepass.wgsl's code, without
// their discard (a plain material never takes it) unless the fragment
// takes screen derivatives (`DISCARD`). A plain material's pixels are the
// full pipeline's, byte for byte. What the lean stages leave out only
// feeds what a plain material lacks: the uv (its maps are the 1x1 white
// defaults, still sampled, at uv 0, so the shading code is the full
// path's), the local position (the dissolve, the hooks), the tangent
// (normal maps) and the affine uv (retro 3D's texturing; its snapped clip
// position is `lit_place`'s).

struct LeanVertexIn {
  @location(0) position: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(3) color: vec4<f32>,
};

struct LeanOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world_pos: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) color: vec4<f32>,
};

@vertex
fn vs_lean(v: LeanVertexIn, i: InstanceIn) -> LeanOut {
  let p = lit_place(mat4x4<f32>(i.m0, i.m1, i.m2, i.m3), v.position, v.normal, vec2<f32>(0.0), v.color);
  var o: LeanOut;
  o.clip = p.clip;
  o.world_pos = p.world;
  o.normal = p.normal;
  o.color = v.color;
  return o;
}

@fragment
fn fs_lean(in: LeanOut) -> @location(0) vec4<f32> {
  var full: VertexOut;
  full.clip = in.clip;
  full.world_pos = in.world_pos;
  full.normal = in.normal;
  full.uv = vec2<f32>(0.0);
  full.color = in.color;
  full.local_pos = vec3<f32>(0.0);
  full.tangent = vec4<f32>(0.0);
  full.affine = vec3<f32>(0.0);
  return lit_fragment(full);
}

struct LeanPrepassOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world_pos: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) cur_clip: vec4<f32>,
  @location(3) prev_clip: vec4<f32>,
};

@vertex
fn vs_prepass_lean(@location(0) position: vec3<f32>, @location(1) normal: vec3<f32>, i: InstanceIn, prev: PrevIn) -> LeanPrepassOut {
  let p = lit_place(mat4x4<f32>(i.m0, i.m1, i.m2, i.m3), position, normal, vec2<f32>(0.0), vec4<f32>(1.0));
  var o: LeanPrepassOut;
  o.clip = p.clip;
  o.world_pos = p.world;
  o.normal = p.normal;
  o.cur_clip = fx.view_proj_unjittered * vec4<f32>(p.world, 1.0);
  o.prev_clip = fx.prev_view_proj * (mat4x4<f32>(prev.p0, prev.p1, prev.p2, prev.p3) * vec4<f32>(position, 1.0));
  return o;
}

@fragment
fn fs_prepass_lean(in: LeanPrepassOut) -> PrepassTargets {
  var full: PrepassOut;
  full.clip = in.clip;
  full.world_pos = in.world_pos;
  full.normal = in.normal;
  full.uv = vec2<f32>(0.0);
  full.color = vec4<f32>(1.0);
  full.local_pos = vec3<f32>(0.0);
  full.cur_clip = in.cur_clip;
  full.prev_clip = in.prev_clip;
  return prepass_targets(full);
}
