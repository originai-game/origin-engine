// A view's sprite glow (views.rs): its glowing sprites drawn into level 0
// of a glow chain at half its size, then the main frame's bloom chain
// over it (bloom_chain.wgsl, appended): 13-tap downsamples level by level,
// tent upsamples added back, and level 0 added onto the view's HDR image
// times the glow's intensity, as the tonemap adds the frame's (post.wgsl).
// Each pass binds one level as `source`, with a uniform of its own.
struct GlowChain {
  // x: intensity, y: upsample radius in texels, zw: the source level's size
  // in pixels (bloom_chain.wgsl's `bloom_source_size`).
  params: vec4<f32>,
};
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var linear_sampler: sampler;
@group(0) @binding(2) var<uniform> g: GlowChain;

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

// A view keeps no untonemapped sprites (every run is tone mapped there),
// so a tap is the level's colour, as the frame's without them.
fn bloom_tap(uv: vec2<f32>) -> vec3<f32> {
  return textureSampleLevel(source, linear_sampler, uv, 0.0).rgb;
}

// Not textureDimensions: the level is read through a view whose base mip
// level is that level (bloom_chain.wgsl).
fn bloom_source_size() -> vec2<f32> {
  return g.params.zw;
}

@fragment
fn fs_down(in: VsOut) -> @location(0) vec4<f32> {
  return vec4<f32>(downsample13(in.uv), 1.0);
}

@fragment
fn fs_up(in: VsOut) -> @location(0) vec4<f32> {
  return vec4<f32>(bloom_tent(in.uv, g.params.y), 1.0);
}

// Level 0 onto the view's HDR image.
@fragment
fn fs(in: VsOut) -> @location(0) vec4<f32> {
  return vec4<f32>(textureSampleLevel(source, linear_sampler, in.uv, 0.0).rgb * g.params.x, 0.0);
}
