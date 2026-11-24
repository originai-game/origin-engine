// Dedicated instanced EffectMesh draw (effect_mesh.rs): renders slash arcs,
// shockwave rings, shields and portals via hardware instancing without faking
// particle emitters or duplicating per-mesh uniforms. Supports full OIT,
// billboard orientation, soft depth against scene_depth, hue rotation, and
// EffectMaterial hooks.
// Prepended by FRAME_WGSL, OIT_WGSL, ARENA_WGSL, COMMON_WGSL, EFFECT_WGSL,
// and hooks_source(hook).

@group(1) @binding(0) var sprite_tex: texture_2d<f32>;
@group(1) @binding(1) var sprite_sampler: sampler;
@group(2) @binding(1) var<uniform> view: ParticleView;
@group(2) @binding(2) var scene_depth: texture_2d<f32>;

struct EffectMeshInst {
  @location(0) col0: vec4<f32>,
  @location(1) col1: vec4<f32>,
  @location(2) col2: vec4<f32>,
  @location(3) col3: vec4<f32>,
  @location(4) color: vec4<f32>,
  // x: life (0..1), y: hue turn (rad), z: blend id (0 add, 1 alpha, 2 sub, 3 mul), w: soft enabled (0 or 1)
  @location(9) life_hue: vec4<f32>,
  // x: hashed seed, y: flags (bit 0: billboard)
  @location(10) seed_flags: vec4<u32>,
};

struct EffectMeshVert {
  @location(5) position: vec3<f32>,
  @location(6) normal: vec3<f32>,
  @location(7) color: vec4<f32>,
  @location(8) uv: vec2<f32>,
};

struct EffectMeshOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) uv: vec2<f32>,
  @location(1) color: vec4<f32>,
  @location(2) depth: f32,
  @location(3) world: vec3<f32>,
  @location(4) normal: vec3<f32>,
  @location(5) @interpolate(flat, either) oit_key: vec2<u32>,
  @location(6) @interpolate(flat, either) fx_a: vec4<f32>,
  @location(7) @interpolate(flat, either) fx_b: vec4<f32>,
  @location(8) @interpolate(flat, either) fx_c1: vec4<f32>,
  @location(9) @interpolate(flat, either) fx_c2: vec4<f32>,
  // x: hue, y: blend id, z: soft enabled, w: unused
  @location(10) @interpolate(flat, either) fx_meta: vec4<f32>,
};

@vertex
fn vs_effect_mesh(inst: EffectMeshInst, m: EffectMeshVert) -> EffectMeshOut {
  var o: EffectMeshOut;
  let xform = mat4x4<f32>(inst.col0, inst.col1, inst.col2, inst.col3);
  var world = (xform * vec4<f32>(m.position, 1.0)).xyz;
  var nrm = (xform * vec4<f32>(m.normal, 0.0)).xyz;
  if ((inst.seed_flags.y & 1u) != 0u) {
    let origin = xform[3].xyz;
    let l = world - origin;
    let b = mat3x3<f32>(view.cam_right.xyz, view.cam_up.xyz, -view.cam_forward.xyz);
    world = origin + b * l;
    nrm = b * nrm;
  }
  let n_len = length(nrm);
  if (n_len > 1e-8) {
    nrm = nrm / n_len;
  } else {
    nrm = -view.cam_forward.xyz;
  }
  o.clip = frame.view_proj * vec4<f32>(world, 1.0);
  o.uv = m.uv;
  o.color = inst.color * m.color;
  o.depth = dot(world - view.cam_pos.xyz, view.cam_forward.xyz);
  o.world = world;
  o.normal = nrm;
  let life = clamp(inst.life_hue.x, 0.0, 1.0);
  let seed = inst.seed_flags.x;
  let random = hash_unit(seed);
  o.oit_key = oit_key4(
    vec4<f32>(xform[3].xyz, life),
    vec3<f32>(f32(seed & 0xffffu), inst.life_hue.y, 2.0),
  );
  if (effect_on()) {
    let d = effect_driven(life, u32(random * 16777216.0) ^ (seed * 2246822519u));
    o.fx_a = d.a;
    o.fx_b = d.b;
    o.fx_c1 = d.c1;
    o.fx_c2 = d.c2;
  }
  o.fx_meta = vec4<f32>(inst.life_hue.y, inst.life_hue.z, inst.life_hue.w, 0.0);
  return o;
}

fn em_soft(in: EffectMeshOut) -> f32 {
  let dist = effect.fresnel.w;
  if (in.fx_meta.z < 0.5 || dist <= 0.0) { return 1.0; }
  let at = min(vec2<i32>(in.clip.xy), vec2<i32>(textureDimensions(scene_depth)) - vec2<i32>(1));
  let scene = textureLoad(scene_depth, at, 0).r;
  return clamp((scene - in.depth) / dist, 0.0, 1.0);
}

fn em_blend_out(c: vec4<f32>, blend_id: f32) -> vec4<f32> {
  if (blend_id < 2.5) { return c; }
  return vec4<f32>(mix(vec3<f32>(1.0), c.rgb, clamp(c.a, 0.0, 1.0)), c.a);
}

fn effect_mesh_fragment(in: EffectMeshOut) -> vec4<f32> {
  var c = in.color;
  if (effect_on()) {
    var f: EffectFragment;
    f.uv = in.uv;
    f.shape_uv = in.uv;
    f.color = in.color;
    f.shape = 1.0;
    f.profile = 1.0;
    f.textured = 0.0;
    f.world = in.world;
    f.normal = normalize(in.normal);
    var d: EffectDriven;
    d.a = in.fx_a;
    d.b = in.fx_b;
    d.c1 = in.fx_c1;
    d.c2 = in.fx_c2;
    f.driven = d;
    c = effect_shade(f, sprite_tex);
  }
  if (in.fx_meta.x != 0.0) {
    c = vec4<f32>(hue_rotate(c.rgb, in.fx_meta.x), c.a);
  }
  c.a *= em_soft(in);
  c = fx_fog(c, in.world, in.clip.xy, in.fx_meta.y > 0.5 && in.fx_meta.y < 1.5);
  if (c.a <= 0.0) { discard; }
  if (frame.light_color.w > 0.5) { return debug_unsupported(in.clip.xy); }
  return em_blend_out(c, in.fx_meta.y);
}

@fragment
fn fs_effect_mesh(in: EffectMeshOut) -> @location(0) vec4<f32> {
  return effect_mesh_fragment(in);
}

@fragment
fn fs_effect_mesh_oit(in: EffectMeshOut) -> OitOutput {
  return oit_weighted(effect_mesh_fragment(in), in.clip.z);
}

@fragment
fn fs_effect_mesh_oit_list(in: EffectMeshOut, @builtin(sample_mask) mask: u32) -> OitOutput {
  let c = effect_mesh_fragment(in);
  oit_store(in.clip, c, mask, in.oit_key);
  return oit_weighted(c, in.clip.z);
}

@fragment
fn fs_effect_mesh_oit_front(in: EffectMeshOut) -> OitOutput {
  let c = effect_mesh_fragment(in);
  return oit_hybrid(c, in.clip);
}

@fragment
fn fs_effect_mesh_oit_mark(in: EffectMeshOut) {
  let c = effect_mesh_fragment(in);
  oit_front_mark(in.clip, c.a);
}
