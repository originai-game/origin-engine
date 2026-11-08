// The StandardMaterial bindings (group 1) and the dissolve, shared by the
// lit, prepass and shadow shaders. Prepended after noise.wgsl.

// `MaterialUniform` in material.rs (a StandardMaterial).
struct MaterialFactors {
  base_color: vec4<f32>,  // rgb: colour, a: opacity
  mr: vec4<f32>,          // x: metalness, y: roughness, zw: normal scale
  emissive: vec4<f32>,    // rgb: emissive x intensity, w: alpha cutoff (<0 = none)
  flags: vec4<f32>,       // x: normal map (0 none, 1 rgb, 2 two-channel), y: ao intensity (<0 no map), z: env intensity, w: mode bits (1 flat, 2 unlit, 4 blended)
  toon: vec4<f32>,        // x: toon (0 off, 1 gradient map, 2 built-in bands), y: rim intensity, z: rim power, w: flash amount
  rim: vec4<f32>,         // rgb: rim colour, w: dissolve amount
  flash: vec4<f32>,       // rgb: flash colour, w: dissolve edge width
  dissolve: vec4<f32>,    // rgb: dissolve edge colour, w: noise scale (<0: dissolve map)
  outline: vec4<f32>,     // rgb: outline colour, w: outline width (px)
  coat: vec4<f32>,        // x: clearcoat (0 none), y: clearcoat roughness, zw: leaf translucency/dome normals
  uv: array<vec4<f32>, 10>,  // per slot (UV_MAP and on): [a, b, c, d], [tx, ty, 0, 0]; uv[1].zw holds base/normal array layers, uv[3].z paintDetail, uv[3].w weathering
  custom: array<vec4<f32>, 8>,  // a material shader's parameters (hooks.rs)
  previous_custom: array<vec4<f32>, 8>,  // the parameters the frame before `stamp.x` drew, not the last simulation tick
  stamp: vec4<u32>,  // x: the drawn frame `custom` was written in (frame.wgsl `params_moved`), yzw unused
};

@group(1) @binding(0) var<uniform> material: MaterialFactors;

// The texture slots' uv transforms (KHR_texture_transform, three.js
// Texture.offset/repeat/rotation): StandardMaterial::uv_transforms' order.
const UV_MAP: u32 = 0u;
const UV_METALNESS: u32 = 1u;  // metalness and roughness, and a physical material's packed maps
const UV_NORMAL: u32 = 2u;
const UV_EMISSIVE: u32 = 3u;
const UV_AO: u32 = 4u;

// `uv` as texture slot `slot` samples it.
fn material_uv(slot: u32, uv: vec2<f32>) -> vec2<f32> {
  let m = material.uv[slot * 2u];
  let t = material.uv[slot * 2u + 1u];
  return vec2<f32>(m.x * uv.x + m.y * uv.y, m.z * uv.x + m.w * uv.y) + t.xy;
}
