// Lightmapped static surfaces of physical materials (lightmap.rs): the
// physical lit shading of physical.wgsl with the ambient light read from
// the baked lightmap pages (`baked_lightmap`, which ambient_irradiance
// prefers). Appended to the physical lit module after lightmap_pages.wgsl
// (the material shaders' physical modules too).

struct PhysicalLightmappedOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world_pos: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
  @location(3) color: vec4<f32>,
  @location(4) local_pos: vec3<f32>,
  @location(5) tangent: vec4<f32>,
  @location(6) scale: vec3<f32>,
  @location(7) affine: vec3<f32>,
  @location(8) @interpolate(flat) look: vec4<f32>,
  @location(9) lightmap: vec3<f32>,
};

@vertex
fn vs_physical_lightmap(v: VertexIn, i: InstanceIn, k: InstanceFxIn, m: LightmapIn) -> PhysicalLightmappedOut {
  let p = physical_vertex(instance_matrix(i.m0, i.m1, i.m2, i.m3), v);
  var l: PhysicalLightmappedOut;
  l.clip = p.clip;
  l.world_pos = p.world_pos;
  l.normal = p.normal;
  l.uv = p.uv;
  l.color = p.color;
  l.local_pos = p.local_pos;
  l.tangent = p.tangent;
  l.scale = p.scale;
  l.affine = p.affine;
  l.look = k.look;
  l.lightmap = m.lightmap;
  return l;
}

@fragment
fn fs_physical_lightmap(in: PhysicalLightmappedOut, @builtin(front_facing) front_facing: bool) -> @location(0) vec4<f32> {
  hook_front = select(-1.0, 1.0, front_facing);
  baked_lightmap = lightmap_texel(in.lightmap);
  var p: PhysicalOut;
  p.clip = in.clip;
  p.world_pos = in.world_pos;
  p.normal = in.normal;
  p.uv = in.uv;
  p.color = in.color;
  p.local_pos = in.local_pos;
  p.tangent = in.tangent;
  p.scale = in.scale;
  p.affine = in.affine;
  p.look = in.look;
  return physical_fragment(p);
}
