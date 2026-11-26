// A decal's texture copied into its cells of the decals' atlas
// (shading/decal_atlas.rs): the viewport is the cells, so `uv` spans the
// whole source. The atlas is sRGB. A colour is stored encoded, as an
// artist's file is. A normal map's values are written as they are, as if
// they were linear light: the atlas's filter and its levels then average
// the normals themselves, and decals.wgsl reads them back unchanged.
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;

@fragment
fn fs_color(in: VsOut) -> @location(0) vec4<f32> {
  return textureSample(source, source_sampler, in.uv);
}

// A normal map loaded linear.
@fragment
fn fs_normal(in: VsOut) -> @location(0) vec4<f32> {
  return vec4<f32>(textureSample(source, source_sampler, in.uv).xyz, 1.0);
}

// A normal map loaded as sRGB: its read decoded it, so back to its stored
// values.
@fragment
fn fs_normal_srgb(in: VsOut) -> @location(0) vec4<f32> {
  let n = textureSample(source, source_sampler, in.uv).xyz;
  return vec4<f32>(srgb_encode(clamp(n, vec3<f32>(0.0), vec3<f32>(1.0))), 1.0);
}

// A two-channel normal map (BC5, EAC RG11): z from x and y.
@fragment
fn fs_normal_rg(in: VsOut) -> @location(0) vec4<f32> {
  let xy = textureSample(source, source_sampler, in.uv).xy * 2.0 - 1.0;
  return vec4<f32>(vec3<f32>(xy, sqrt(max(1.0 - dot(xy, xy), 0.0))) * 0.5 + 0.5, 1.0);
}
