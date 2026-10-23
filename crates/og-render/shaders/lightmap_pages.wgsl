// The lightmap pages of a lightmapped batch's scene (lightmap.rs), at group
// 3 beside the skinned palette's bindings (0 and 1), which no lightmapped
// pipeline binds (lightmapped meshes are never skinned). In the standard
// and physical lit modules, before their lightmapped entry points
// (lightmap.wgsl, lightmap_physical.wgsl).

@group(3) @binding(4) var lightmap_pages: texture_2d_array<f32>;
@group(3) @binding(5) var lightmap_sampler: sampler;

// The mesh's lightmap uv (0..1 on its page) and the page, per vertex.
struct LightmapIn {
  @location(9) lightmap: vec3<f32>,
};

// The baked texel at a lightmap coordinate (uv and page) as
// `baked_lightmap` takes it: irradiance over π, a 1.
fn lightmap_texel(l: vec3<f32>) -> vec4<f32> {
  let page = i32(l.z + 0.5);
  return vec4<f32>(textureSample(lightmap_pages, lightmap_sampler, l.xy, page).rgb, 1.0);
}
