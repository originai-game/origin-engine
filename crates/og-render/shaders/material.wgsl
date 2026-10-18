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
};

@group(1) @binding(0) var<uniform> material: MaterialFactors;
@group(1) @binding(1) var base_color_tex: texture_2d<f32>;
@group(1) @binding(2) var metalness_tex: texture_2d<f32>;
@group(1) @binding(3) var roughness_tex: texture_2d<f32>;
@group(1) @binding(4) var normal_tex: texture_2d<f32>;
@group(1) @binding(5) var emissive_tex: texture_2d<f32>;
@group(1) @binding(6) var ao_tex: texture_2d<f32>;
@group(1) @binding(7) var material_sampler: sampler;
@group(1) @binding(8) var gradient_tex: texture_2d<f32>;
@group(1) @binding(9) var dissolve_tex: texture_2d<f32>;

// The dissolve noise of a fragment: the map's red channel by UV, or value
// noise over the object-space position.
fn dissolve_noise(uv: vec2<f32>, local_pos: vec3<f32>) -> f32 {
  if (material.dissolve.w < 0.0) { return textureSampleLevel(dissolve_tex, material_sampler, uv, 0.0).r; }
  let p = local_pos * material.dissolve.w;
  return 0.6 * value_noise3(p, 17u) + 0.4 * value_noise3(p * 2.7 + vec3<f32>(5.2, 1.3, 7.7), 91u);
}

// Discards dissolved fragments; returns the glowing edge's emission.
fn apply_dissolve(uv: vec2<f32>, local_pos: vec3<f32>) -> vec3<f32> {
  let amount = material.rim.w;
  if (amount <= 0.0) { return vec3<f32>(0.0); }
  let noise = dissolve_noise(uv, local_pos);
  if (noise < amount) { discard; }
  let width = max(material.flash.w, 1e-4);
  let edge = 1.0 - clamp((noise - amount) / width, 0.0, 1.0);
  return material.dissolve.rgb * edge * edge;
}

