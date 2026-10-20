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
  coat: vec4<f32>,        // x: clearcoat (0 none), y: clearcoat roughness
  custom: array<vec4<f32>, 8>,  // a material shader's parameters (hooks.rs)
};

@group(1) @binding(0) var<uniform> material: MaterialFactors;
