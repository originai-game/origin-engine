// Lightmapped static surfaces (lightmap.rs): the standard lit shading of
// pbr_fs.wgsl with the ambient light read from the baked lightmap pages
// instead of the environment's irradiance (`baked_lightmap`, which
// ambient_irradiance prefers). Appended to the standard lit module after
// lightmap_pages.wgsl, so the material shaders' modules (the same source
// with their hooks) and the array-map module (pbr_array.wgsl's
// `fs_lightmap_array`) have these entry points too; the physical module
// has lightmap_physical.wgsl's.

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

// A lean batch's compact vertices (pbr_lean.wgsl `vs_lean_compact`: no uv,
// no tangent; a plain material reads neither) with its lightmap.
@vertex
fn vs_lightmap_compact(v: CompactVertexIn, i: InstanceIn, k: InstanceFxIn, m: LightmapIn) -> LightmappedOut {
  let n = oct_decode(v.normal);
  let p = lit_place(instance_matrix(i.m0, i.m1, i.m2, i.m3), v.position.xyz, n, vec2<f32>(0.0), v.color);
  var l: LightmappedOut;
  l.clip = p.clip;
  l.world_pos = p.world;
  l.normal = p.normal;
  l.uv = vec2<f32>(0.0);
  l.color = v.color;
  l.local_pos = vec3<f32>(0.0);
  l.tangent = vec4<f32>(0.0);
  l.affine = vec3<f32>(0.0);
  l.lightmap = m.lightmap;
  l.look = k.look;
  return l;
}

// The lit shading of a lightmapped fragment.
fn lightmapped_fragment(in: LightmappedOut) -> vec4<f32> {
  baked_lightmap = lightmap_texel(in.lightmap);
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

@fragment
fn fs_lightmap(in: LightmappedOut) -> @location(0) vec4<f32> {
  return lightmapped_fragment(in);
}
