// The standard material's textures (group 1 after material.wgsl's
// uniform) and its dissolve noise; the physical pipelines bind
// material_physical.wgsl instead.
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

