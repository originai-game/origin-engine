// A view's sprite glow (views.rs): its glowing sprites drawn into a glow
// buffer at half its size, a box-filtered mip chain of it, and this pass
// adding the chain's levels, each spread by a 3x3 tent, onto the view's HDR image before its resolve,
// each level weighted by the glow's intensity, as the main frame's bloom
// chain adds its levels (post.rs) when the glow buffer feeds it.
struct GlowAdd {
  params: vec4<f32>,  // x: intensity, y: levels
};
@group(0) @binding(0) var glow: texture_2d<f32>;
@group(0) @binding(1) var glow_sampler: sampler;
@group(0) @binding(2) var<uniform> g: GlowAdd;

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

@fragment
fn fs(in: VsOut) -> @location(0) vec4<f32> {
  var c = vec3<f32>(0.0);
  let levels = u32(g.params.y);
  for (var l = 0u; l < levels; l++) {
    // A 3x3 tent over the level's texels, as the bloom chain's upsample
    // spreads each of its levels.
    let t = 1.0 / vec2<f32>(textureDimensions(glow, l));
    let lod = f32(l);
    var s = textureSampleLevel(glow, glow_sampler, in.uv, lod).rgb * 4.0;
    s += (textureSampleLevel(glow, glow_sampler, in.uv + vec2<f32>(t.x, 0.0), lod).rgb
      + textureSampleLevel(glow, glow_sampler, in.uv - vec2<f32>(t.x, 0.0), lod).rgb
      + textureSampleLevel(glow, glow_sampler, in.uv + vec2<f32>(0.0, t.y), lod).rgb
      + textureSampleLevel(glow, glow_sampler, in.uv - vec2<f32>(0.0, t.y), lod).rgb) * 2.0;
    s += textureSampleLevel(glow, glow_sampler, in.uv + t, lod).rgb
      + textureSampleLevel(glow, glow_sampler, in.uv - t, lod).rgb
      + textureSampleLevel(glow, glow_sampler, in.uv + vec2<f32>(t.x, -t.y), lod).rgb
      + textureSampleLevel(glow, glow_sampler, in.uv + vec2<f32>(-t.x, t.y), lod).rgb;
    c += s / 16.0;
  }
  return vec4<f32>(c * g.params.x, 0.0);
}
