// The terrain in the shadow passes: depth from a cascade, or distance /
// range from a local light, into the R32Float target like shadow.wgsl.
// Prepended by terrain.wgsl (group 1); group 0 is the shadow pass's slot.

struct ShadowPass {
  view_proj: mat4x4<f32>,
  light: vec4<f32>,  // xyz: light position, w: range (0: directional, write depth)
};
@group(0) @binding(0) var<uniform> shadow_pass: ShadowPass;

struct TerrainShadowOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world: vec3<f32>,
};

@vertex
fn vs_shadow(@builtin(vertex_index) vi: u32) -> TerrainShadowOut {
  let p = terrain_vertex(vi);
  var o: TerrainShadowOut;
  o.clip = shadow_pass.view_proj * vec4<f32>(p, 1.0);
  o.world = p;
  return o;
}

@fragment
fn fs_shadow(in: TerrainShadowOut) -> @location(0) vec4<f32> {
  if (shadow_pass.light.w > 0.0) {
    return vec4<f32>(length(in.world - shadow_pass.light.xyz) / shadow_pass.light.w, 0.0, 0.0, 1.0);
  }
  return vec4<f32>(in.clip.z, 0.0, 0.0, 1.0);
}
