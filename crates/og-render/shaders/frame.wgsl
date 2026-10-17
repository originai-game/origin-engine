// Per-frame bindings shared by the passes that draw into the HDR target
// (pbr.wgsl, sky.wgsl): the camera and light uniform, the shadow map and the
// image-based lighting. The renderer prepends this file to those shaders.

struct Camera {
  view_proj: mat4x4<f32>,
  shadow_view_proj: mat4x4<f32>,
  view_pos: vec4<f32>,
  light_dir: vec4<f32>,   // xyz: direction towards the light, w: unused
  light_color: vec4<f32>, // rgb: radiance, w: shadow texel size
  params: vec4<f32>,      // x: ibl intensity, y: prefiltered mip count - 1, z: shadows on, w: unused
  inv_view_proj: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> camera: Camera;
// Shadow depth stored in an R32Float colour target and read with textureLoad:
// WebGL2 can neither filter nor textureLod a depth texture.
@group(0) @binding(1) var shadow_map: texture_2d<f32>;
@group(0) @binding(2) var shadow_sampler: sampler;
@group(0) @binding(3) var irradiance_map: texture_cube<f32>;
@group(0) @binding(4) var prefiltered_map: texture_cube<f32>;
@group(0) @binding(5) var brdf_lut: texture_2d<f32>;
@group(0) @binding(6) var ibl_sampler: sampler;
@group(0) @binding(7) var sky_map: texture_cube<f32>;

