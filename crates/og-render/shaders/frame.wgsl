// Per-frame bindings shared by the passes that draw into the HDR target
// (pbr.wgsl, sky.wgsl, sprite.wgsl): the camera, the directional light and
// its cascades, the local lights, the shadow maps and the image-based
// lighting. The renderer prepends this file to those shaders; the Rust side
// is `FrameUniform` in frame.rs.

const CASCADES: u32 = 3u;
const MAX_LIGHTS: u32 = 8u;
const LOCAL_SHADOW_LAYERS: u32 = 12u;

struct LocalLight {
  position_range: vec4<f32>,  // xyz: position, w: range
  color_kind: vec4<f32>,      // rgb: color * intensity, w: 0 point, 1 spot
  direction_slot: vec4<f32>,  // xyz: spot axis (away from the light), w: shadow slot (-1: none)
  cone: vec4<f32>,            // x: cos outer, y: 1 / (cos inner - cos outer), z: normal offset per metre, w: depth bias
};

struct Frame {
  view_proj: mat4x4<f32>,     // jittered when TAA is on
  inv_view_proj: mat4x4<f32>,
  view_pos: vec4<f32>,        // xyz: camera position, w: 1 orthographic, 0 perspective
  light_dir: vec4<f32>,       // xyz: towards the light, w: cascades in use (0: no shadows)
  light_color: vec4<f32>,     // rgb: radiance, w: debug view (1: cascades)
  params: vec4<f32>,          // x: ibl intensity, y: prefiltered mips - 1, z: local lights, w: 1 / cascade map size
  view_forward: vec4<f32>,    // xyz: camera forward, w: 1 / local shadow map size
  cascade_view_proj: array<mat4x4<f32>, 3>,
  cascade_bias: array<vec4<f32>, 3>,    // x: normal offset (world), y: depth bias (normalized), z: slice far (view depth), w: unused
  lights: array<LocalLight, 8>,
  local_view_proj: array<mat4x4<f32>, 12>,
};

@group(0) @binding(0) var<uniform> frame: Frame;
// Shadow depths stored in R32Float colour targets and read with textureLoad:
// WebGL2 can neither filter nor textureLod a depth texture.
@group(0) @binding(1) var cascade_map: texture_2d_array<f32>;
@group(0) @binding(2) var local_shadow_map: texture_2d_array<f32>;
@group(0) @binding(3) var irradiance_map: texture_cube<f32>;
@group(0) @binding(4) var prefiltered_map: texture_cube<f32>;
@group(0) @binding(5) var brdf_lut: texture_2d<f32>;
@group(0) @binding(6) var ibl_sampler: sampler;
@group(0) @binding(7) var sky_map: texture_cube<f32>;
