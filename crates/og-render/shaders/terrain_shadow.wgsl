// The terrain in the shadow passes: depth from a cascade, or distance /
// range from a local light, into the R32Float target like shadow.wgsl.
// Prepended by terrain.wgsl (group 1) and shadow_pass.wgsl (group 0, the
// shadow pass's slot).

struct TerrainShadowOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world: vec3<f32>,
  @location(1) local: vec3<f32>,
};

@vertex
fn vs_shadow(@builtin(vertex_index) vi: u32) -> TerrainShadowOut {
  let local = terrain_local_vertex(vi);
  let p = terrain_world(local);
  var o: TerrainShadowOut;
  o.clip = shadow_clip(vec4<f32>(p, 1.0));
  o.world = p;
  o.local = local;
  return o;
}

@fragment
fn fs_shadow(in: TerrainShadowOut) -> @location(0) vec4<f32> {
  // Light falls through the holes.
  if (terrain_hole_at(in.local)) { discard; }
  let dx = dpdx(in.world);
  let dy = dpdy(in.world);
  if (shadow_pass.light.w > 0.0) {
    return vec4<f32>(local_distance(in.world, in.clip, dx, dy), 0.0, 0.0, 1.0);
  }
  return vec4<f32>(in.clip.z, 0.0, 0.0, 1.0);
}

// A terrain stamp's shadow (terrain_lit.wgsl's stamps): its mesh, never
// cut by the terrain's holes.
@group(2) @binding(0) var<uniform> stamp: TerrainStampUniform;

struct StampShadowOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world: vec3<f32>,
};

@vertex
fn vs_stamp_shadow(@location(0) position: vec3<f32>) -> StampShadowOut {
  let p = (stamp.model * vec4<f32>(position, 1.0)).xyz;
  var o: StampShadowOut;
  o.clip = shadow_clip(vec4<f32>(p, 1.0));
  o.world = p;
  return o;
}

@fragment
fn fs_stamp_shadow(in: StampShadowOut) -> @location(0) vec4<f32> {
  if (shadow_pass.light.w > 0.0) {
    return vec4<f32>(local_distance(in.world, in.clip, dpdx(in.world), dpdy(in.world)), 0.0, 0.0, 1.0);
  }
  return vec4<f32>(in.clip.z, 0.0, 0.0, 1.0);
}
