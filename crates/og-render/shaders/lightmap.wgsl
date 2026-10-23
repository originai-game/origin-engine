// Lightmapped static surfaces (lightmap.rs): the standard lit shading of
// pbr_fs.wgsl with the ambient light read from the baked lightmap pages
// instead of the environment's irradiance (`baked_lightmap`, which
// ambient_irradiance prefers). Appended to the standard lit module only;
// the lightmap pages sit at group 3 beside the skinned palette's bindings
// (0 and 1), which no lightmapped pipeline binds (lightmapped meshes are
// never skinned).

@group(3) @binding(4) var lightmap_pages: texture_2d_array<f32>;
@group(3) @binding(5) var lightmap_sampler: sampler;

// The mesh's lightmap uv (0..1 on its page) and the page, per vertex.
struct LightmapIn {
  @location(9) lightmap: vec3<f32>,
};

struct LightmappedOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world_pos: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
  @location(3) color: vec4<f32>,
  @location(4) local_pos: vec3<f32>,
  @location(5) tangent: vec4<f32>,
  @location(6) affine: vec3<f32>,
  @location(7) lightmap: vec3<f32>,
  @location(8) @interpolate(flat) look: vec4<f32>,
};

@vertex
fn vs_lightmap(v: VertexIn, i: InstanceIn, k: InstanceFxIn, m: LightmapIn) -> LightmappedOut {
  let o = lit_vertex(instance_matrix(i.m0, i.m1, i.m2, i.m3), v);
  var l: LightmappedOut;
  l.clip = o.clip;
  l.world_pos = o.world_pos;
  l.normal = o.normal;
  l.uv = o.uv;
  l.color = o.color;
  l.local_pos = o.local_pos;
  l.tangent = o.tangent;
  l.affine = o.affine;
  l.lightmap = m.lightmap;
  l.look = k.look;
  return l;
}

@fragment
fn fs_lightmap(in: LightmappedOut) -> @location(0) vec4<f32> {
  let page = i32(in.lightmap.z + 0.5);
  baked_lightmap = vec4<f32>(textureSample(lightmap_pages, lightmap_sampler, in.lightmap.xy, page).rgb, 1.0);
  var o: VertexOut;
  o.clip = in.clip;
  o.world_pos = in.world_pos;
  o.normal = in.normal;
  o.uv = in.uv;
  o.color = in.color;
  o.local_pos = in.local_pos;
  o.tangent = in.tangent;
  o.affine = in.affine;
  o.look = in.look;
  return lit_fragment(o);
}
