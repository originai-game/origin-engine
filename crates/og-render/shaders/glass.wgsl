// Glass (src/glass.rs): the frame under the overlay as a blur pyramid the
// overlay's glass panels sample. `fs_down` makes each level from the one
// above with a 13-tap filter (the centre and two rings of four, weights
// 4 : 1 : 1: the dual-filter downsample), so a level is a smooth blur of the
// frame at its scale rather than a box; the first level decodes an
// sRGB-encoded source into linear light. `fs_blit` copies the source into
// the output untouched (the frame the tonemap wrote there before glass
// needed it).
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
struct GlassParams {
  // x: 1 to decode sRGB from the source.
  params: vec4<f32>,
};
@group(0) @binding(2) var<uniform> glass: GlassParams;

struct VsOut {
  @builtin(position) pos: vec4<f32>,
  @location(0) uv: vec2<f32>,
};

@vertex
fn vs(@builtin(vertex_index) i: u32) -> VsOut {
  var o: VsOut;
  let x = f32(i32(i & 1u) * 4 - 1);
  let y = f32(i32(i & 2u) * 2 - 1);
  o.pos = vec4<f32>(x, y, 0.0, 1.0);
  o.uv = vec2<f32>(x * 0.5 + 0.5, 0.5 - y * 0.5);
  return o;
}

fn decode(c: vec3<f32>) -> vec3<f32> {
  let lo = c / 12.92;
  let hi = pow((c + 0.055) / 1.055, vec3<f32>(2.4));
  return select(hi, lo, c <= vec3<f32>(0.04045));
}

fn tap(uv: vec2<f32>) -> vec3<f32> {
  let c = textureSampleLevel(source, source_sampler, uv, 0.0).rgb;
  if (glass.params.x > 0.5) {
    return decode(c);
  }
  return c;
}

@fragment
fn fs_down(in: VsOut) -> @location(0) vec4<f32> {
  let h = 1.0 / vec2<f32>(textureDimensions(source, 0));
  var c = tap(in.uv) * 4.0;
  c += tap(in.uv + vec2<f32>(-h.x, -h.y));
  c += tap(in.uv + vec2<f32>(h.x, -h.y));
  c += tap(in.uv + vec2<f32>(-h.x, h.y));
  c += tap(in.uv + vec2<f32>(h.x, h.y));
  let f = 2.0 * h;
  c += tap(in.uv + vec2<f32>(-f.x, 0.0));
  c += tap(in.uv + vec2<f32>(f.x, 0.0));
  c += tap(in.uv + vec2<f32>(0.0, -f.y));
  c += tap(in.uv + vec2<f32>(0.0, f.y));
  return vec4<f32>(c / 12.0, 1.0);
}

@fragment
fn fs_blit(in: VsOut) -> @location(0) vec4<f32> {
  return textureLoad(source, vec2<i32>(in.pos.xy), 0);
}
