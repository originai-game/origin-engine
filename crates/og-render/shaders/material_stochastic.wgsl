// Heitz & Neyret, HPG 2018; per-channel truncated transforms after
// Burley, JCGT 2019. Original implementation, no third-party source copied.
// Unrotated patches share offsets across every PBR map and keep tangent frames.
override OG_STOCHASTIC: bool = true;
struct StochasticLookup { entries: array<vec4<u32>, 768>, };
@group(1) @binding(10) var<uniform> stochastic_lookup: StochasticLookup;

struct StochasticGrid {
  a: vec2<f32>, b: vec2<f32>, c: vec2<f32>, weights: vec3<f32>,
};

fn stochastic_offset(cell: vec2<f32>) -> vec2<f32> {
  let c = vec2<i32>(cell);
  return vec2<f32>(lattice3(vec3<i32>(c, 137), 0u), lattice3(vec3<i32>(c, 139), 0u));
}

fn stochastic_grid(at: vec2<f32>) -> StochasticGrid {
  // Inverse of the equilateral lattice basis (1,0), (1/2,sqrt(3)/2).
  let q = vec2<f32>(at.x - at.y * 0.57735026919, at.y * 1.15470053838);
  let cell = floor(q);
  let f = fract(q);
  var grid: StochasticGrid;
  if (f.x + f.y <= 1.0) {
    grid.a = stochastic_offset(cell);
    grid.b = stochastic_offset(cell + vec2<f32>(1.0, 0.0));
    grid.c = stochastic_offset(cell + vec2<f32>(0.0, 1.0));
    grid.weights = vec3<f32>(1.0 - f.x - f.y, f.x, f.y);
  } else {
    grid.a = stochastic_offset(cell + vec2<f32>(1.0));
    grid.b = stochastic_offset(cell + vec2<f32>(0.0, 1.0));
    grid.c = stochastic_offset(cell + vec2<f32>(1.0, 0.0));
    grid.weights = vec3<f32>(f.x + f.y - 1.0, 1.0 - f.x, 1.0 - f.y);
  }
  // Burley's sharp weights reduce ghosting without seams at triangle edges.
  grid.weights = pow(grid.weights, vec3<f32>(7.0));
  grid.weights /= dot(grid.weights, vec3<f32>(1.0));
  return grid;
}

fn stochastic_lookup_at(slot: u32, channel: u32, x: f32, inverse: bool) -> f32 {
  let at = clamp(x, 0.0, 1.0) * 127.0;
  let lo = u32(floor(at));
  let hi = min(lo + 1u, 127u);
  let a = unpack2x16unorm(stochastic_lookup.entries[slot * 128u + lo][channel]);
  let b = unpack2x16unorm(stochastic_lookup.entries[slot * 128u + hi][channel]);
  return mix(select(a.x, a.y, inverse), select(b.x, b.y, inverse), fract(at));
}

fn stochastic_channel(a: f32, b: f32, c: f32, w: vec3<f32>, slot: u32, channel: u32) -> f32 {
  let ga = stochastic_lookup_at(slot, channel, a, false);
  let gb = stochastic_lookup_at(slot, channel, b, false);
  let gc = stochastic_lookup_at(slot, channel, c, false);
  let g = 0.5 + (dot(vec3<f32>(ga, gb, gc), w) - 0.5) * inverseSqrt(dot(w, w));
  return stochastic_lookup_at(slot, channel, g, true);
}

fn stochastic_blend(a: vec4<f32>, b: vec4<f32>, c: vec4<f32>, w: vec3<f32>, slot: u32) -> vec4<f32> {
  var result = a * w.x + b * w.y + c * w.z;
  if (slot == 3u) { return result; } // tangent-space normal, normalized by shading
  for (var channel = 0u; channel < 3u; channel++) {
    result[channel] = stochastic_channel(a[channel], b[channel], c[channel], w, slot, channel);
  }
  return result;
}

// Original images, not Gaussian images, supply the filtered samples. Fade to
// ordinary filtered texturing under minification rather than stretch a level-0
// histogram over the narrow distribution of a distant mip (colour drift).
fn stochastic_amount(size: vec2<u32>, gx: vec2<f32>, gy: vec2<f32>) -> f32 {
  if (!OG_STOCHASTIC) { return 0.0; }
  let footprint = max(length(gx * vec2<f32>(size)), length(gy * vec2<f32>(size)));
  return material.variation.z * (1.0 - smoothstep(1.0, 4.0, footprint));
}

fn base_texel(at: vec2<f32>, gx: vec2<f32>, gy: vec2<f32>) -> vec4<f32> {
  let ordinary = textureSampleGrad(base_color_tex, material_sampler, at, gx, gy);
  let amount = stochastic_amount(textureDimensions(base_color_tex), gx, gy);
  if (amount <= 0.0) { return ordinary; }
  let grid = stochastic_grid(at);
  let qa = at + grid.a;
  let qb = at + grid.b;
  let qc = at + grid.c;
  let a = textureSampleGrad(base_color_tex, material_sampler, qa, gx, gy);
  let b = textureSampleGrad(base_color_tex, material_sampler, qb, gx, gy);
  let c = textureSampleGrad(base_color_tex, material_sampler, qc, gx, gy);
  return mix(ordinary, stochastic_blend(a, b, c, grid.weights, 0u), amount);
}

fn normal_texel(at: vec2<f32>, gx: vec2<f32>, gy: vec2<f32>) -> vec4<f32> {
  let ordinary = textureSampleGrad(normal_tex, material_sampler, at, gx, gy);
  let amount = stochastic_amount(textureDimensions(normal_tex), gx, gy);
  if (amount <= 0.0) { return ordinary; }
  let grid = stochastic_grid(at);
  let qa = at + grid.a;
  let qb = at + grid.b;
  let qc = at + grid.c;
  let a = textureSampleGrad(normal_tex, material_sampler, qa, gx, gy);
  let b = textureSampleGrad(normal_tex, material_sampler, qb, gx, gy);
  let c = textureSampleGrad(normal_tex, material_sampler, qc, gx, gy);
  return mix(ordinary, stochastic_blend(a, b, c, grid.weights, 3u), amount);
}

fn stochastic_texel(t: texture_2d<f32>, at: vec2<f32>, gx: vec2<f32>, gy: vec2<f32>, slot: u32) -> vec4<f32> {
  let ordinary = textureSampleGrad(t, material_sampler, at, gx, gy);
  let amount = stochastic_amount(textureDimensions(t), gx, gy);
  if (amount <= 0.0) { return ordinary; }
  let grid = stochastic_grid(at);
  let a = textureSampleGrad(t, material_sampler, at + grid.a, gx, gy);
  let b = textureSampleGrad(t, material_sampler, at + grid.b, gx, gy);
  let c = textureSampleGrad(t, material_sampler, at + grid.c, gx, gy);
  return mix(ordinary, stochastic_blend(a, b, c, grid.weights, slot), amount);
}
