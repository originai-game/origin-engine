// The physical lit pipelines' material group (group 1), after
// material.wgsl in place of material_maps.wgsl: the base colour, normal,
// emissive and gradient textures as there, every other map packed into
// the layers of one array texture by physical.rs, and the physical
// factors. Five textures, so the skinned physical pipeline stays within
// the sixteen a WebGL2 pipeline has (frame 5, material 5, effects 3,
// skin 1).

// `PhysicalUniform` in physical.rs.
struct PhysicalFactors {
  coat: vec4<f32>,          // xy: clearcoat normal scale, z: 1 with a clearcoat normal map, w: 1 with an anisotropy map
  sheen: vec4<f32>,         // rgb: sheen colour x sheen, w: sheen roughness
  transmission: vec4<f32>,  // x: transmission, y: thickness, z: ior, w: attenuation distance (0: none)
  attenuation: vec4<f32>,   // rgb: attenuation colour
  iridescence: vec4<f32>,   // x: iridescence, y: the film's ior, zw: its thickness range (nm)
  anisotropy: vec4<f32>,    // x: strength, yz: direction (cos and sin of the rotation)
  layers: vec4<f32>,        // pack layers (-1: none): metalness/roughness/ao/dissolve, clearcoat/clearcoat roughness/transmission/thickness, sheen colour/roughness, coat normal/iridescence
  layers2: vec4<f32>,       // x: pack layer of anisotropy/film thickness
};

@group(1) @binding(1) var base_color_tex: texture_2d<f32>;
@group(1) @binding(2) var normal_tex: texture_2d<f32>;
@group(1) @binding(3) var emissive_tex: texture_2d<f32>;
@group(1) @binding(4) var material_sampler: sampler;
@group(1) @binding(5) var gradient_tex: texture_2d<f32>;
@group(1) @binding(6) var material_pack: texture_2d_array<f32>;
@group(1) @binding(7) var<uniform> physical: PhysicalFactors;

// One layer of the pack at `uv`, white where the material has none. The
// pack is always sampled and the result selected: a sample after a
// branch on an argument is not in uniform control flow.
fn pack_sample(layer: f32, uv: vec2<f32>) -> vec4<f32> {
  let texel = textureSample(material_pack, material_sampler, uv, i32(max(layer, 0.0)));
  return select(texel, vec4<f32>(1.0), layer < 0.0);
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

