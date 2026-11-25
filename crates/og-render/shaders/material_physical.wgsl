// The physical lit pipelines' material group (group 1), after
// material.wgsl in place of material_maps.wgsl: the base colour, normal,
// emissive and gradient textures as there, every other map packed into
// the layers of one array texture by physical.rs, and the physical
// factors. Five textures, so the skinned physical pipeline stays within
// the sixteen a stage has under the downlevel limits (frame 5, material 5,
// effects 3, skin 1).

// `PhysicalUniform` in physical.rs.
struct PhysicalFactors {
  coat: vec4<f32>,          // xy: clearcoat normal scale, z: 1 with a clearcoat normal map, w: 1 with an anisotropy map
  sheen: vec4<f32>,         // rgb: sheen colour x sheen, w: sheen roughness
  transmission: vec4<f32>,  // x: transmission, y: thickness, z: ior, w: attenuation distance (0: none)
  attenuation: vec4<f32>,   // rgb: attenuation colour, w: chromatic dispersion (0: none)
  iridescence: vec4<f32>,   // x: iridescence, y: the film's ior, zw: its thickness range (nm)
  anisotropy: vec4<f32>,    // x: strength, yz: direction (cos and sin of the rotation), w: how much of the surface is hair
  layers: vec4<f32>,        // pack layers (-1: none): metalness/roughness/ao/dissolve, clearcoat/clearcoat roughness/transmission/thickness, sheen colour/roughness, coat normal/iridescence
  layers2: vec4<f32>,       // x: pack layer of anisotropy/film thickness, y: the iris's radius (uv; 0: no eye), z: its depth under the cornea (of its radius)
  subsurface: vec4<f32>,    // rgb: the scatter distance per channel (m), w: how much of the diffuse light scatters (0: none)
  specular: vec4<f32>,      // rgb: dielectric specular colour, w: dielectric specular intensity
};

@group(1) @binding(1) var base_color_tex: texture_2d<f32>;
@group(1) @binding(2) var normal_tex: texture_2d<f32>;
@group(1) @binding(3) var emissive_tex: texture_2d<f32>;
@group(1) @binding(4) var material_sampler: sampler;
@group(1) @binding(5) var gradient_tex: texture_2d<f32>;
@group(1) @binding(6) var material_pack: texture_2d_array<f32>;
@group(1) @binding(7) var<uniform> physical: PhysicalFactors;

// One layer of the pack at `uv`, white where the material has none. Explicit
// gradients let unused layers skip the texture fetch while remaining in
// uniform control flow for derivatives.
fn pack_sample(layer: f32, uv: vec2<f32>, gx: vec2<f32>, gy: vec2<f32>) -> vec4<f32> {
  if (layer < 0.0) { return vec4<f32>(1.0); }
  let texel = textureSampleGrad(material_pack, material_sampler, uv, i32(layer), gx, gy);
  let amount = stochastic_amount(textureDimensions(material_pack), gx, gy);
  if (amount <= 0.0) { return texel; }
  let grid = stochastic_grid(uv);
  let a = textureSampleGrad(material_pack, material_sampler, uv + grid.a, i32(layer), gx, gy);
  let b = textureSampleGrad(material_pack, material_sampler, uv + grid.b, i32(layer), gx, gy);
  let c = textureSampleGrad(material_pack, material_sampler, uv + grid.c, i32(layer), gx, gy);
  var blended = a * grid.weights.x + b * grid.weights.y + c * grid.weights.z;
  if (layer == physical.layers.x) {
    blended.r = stochastic_channel(a.r, b.r, c.r, grid.weights, 1u, 2u);
    blended.g = stochastic_channel(a.g, b.g, c.g, grid.weights, 2u, 1u);
    blended.b = stochastic_channel(a.b, b.b, c.b, grid.weights, 5u, 0u);
    blended.a = texel.a; // the independent dissolve mask is not randomized
  }
  return mix(texel, blended, amount);
}

// The dissolve noise: the map's red channel (the first layer's alpha), or
// value noise over the object-space position.
fn dissolve_noise(uv: vec2<f32>, local_pos: vec3<f32>) -> f32 {
  if (material.dissolve.w < 0.0) {
    return textureSampleLevel(material_pack, material_sampler, uv, i32(max(physical.layers.x, 0.0)), 0.0).a;
  }
  let p = local_pos * material.dissolve.w;
  return 0.6 * value_noise3(p, 17u) + 0.4 * value_noise3(p * 2.7 + vec3<f32>(5.2, 1.3, 7.7), 91u);
}
