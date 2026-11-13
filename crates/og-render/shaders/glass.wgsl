// Glass (src/glass.rs): the frame under the overlay as a blur pyramid the
// overlay's glass panels sample. `fs_down` makes each level from the one
// above with the bloom chain's 13-tap downsample (bloom_chain.wgsl,
// appended), so a level is a smooth blur of the frame at its scale rather
// than a box, and the frame's blurs are one filter; the first level decodes
// an sRGB-encoded source into linear light. `fs_blit` copies the source into
// the output untouched (the frame the tonemap wrote there before glass
// needed it).
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var linear_sampler: sampler;
struct GlassParams {
  // x: 1 to decode sRGB from the source; zw: the source's size in pixels.
  // Not textureDimensions: a level is sampled through its view's base mip
  // level, and the size of the level is what the pass needs.
  params: vec4<f32>,
};
@group(0) @binding(2) var<uniform> glass: GlassParams;

fn decode(c: vec3<f32>) -> vec3<f32> {
  let lo = c / 12.92;
  let hi = pow((c + 0.055) / 1.055, vec3<f32>(2.4));
  return select(hi, lo, c <= vec3<f32>(0.04045));
}

// bloom_chain.wgsl's tap: the source's colour in linear light.
fn bloom_tap(uv: vec2<f32>) -> vec3<f32> {
  let c = textureSampleLevel(source, linear_sampler, uv, 0.0).rgb;
  if (glass.params.x > 0.5) {
    return decode(c);
  }
  return c;
}

fn bloom_source_size() -> vec2<f32> {
  return glass.params.zw;
}

@fragment
fn fs_down(in: VsOut) -> @location(0) vec4<f32> {
  return vec4<f32>(downsample13(in.uv), 1.0);
}

@fragment
fn fs_blit(in: VsOut) -> @location(0) vec4<f32> {
  return textureLoad(source, vec2<i32>(in.pos.xy), 0);
}
