// Packs a physical material's maps into one layer (and mip) of its array
// texture (physical.rs): each output channel is one channel of one of four
// source textures, or a default. A fullscreen triangle per layer and
// level; the implicit derivatives pick each source's mip for the level.

struct Pack {
  pick: vec4<u32>,      // per output channel: source * 4 + channel, or 255 for `fallback`
  fallback: vec4<f32>,
  flags: vec4<u32>,     // x: channels stored sRGB-encoded (bit per channel), y: 1 to rebuild b as a normal's z from r and g
};
@group(0) @binding(0) var<uniform> pack: Pack;
@group(0) @binding(1) var t0: texture_2d<f32>;
@group(0) @binding(2) var t1: texture_2d<f32>;
@group(0) @binding(3) var t2: texture_2d<f32>;
@group(0) @binding(4) var t3: texture_2d<f32>;
@group(0) @binding(5) var pack_sampler: sampler;

struct VsOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };

@vertex
fn vs(@builtin(vertex_index) i: u32) -> VsOut {
  var o: VsOut;
  let x = f32(i32(i & 1u) * 4 - 1);
  let y = f32(i32(i & 2u) * 2 - 1);
  o.pos = vec4<f32>(x, y, 0.0, 1.0);
  o.uv = vec2<f32>(x * 0.5 + 0.5, 0.5 - y * 0.5);
  return o;
}

fn srgb_encode(c: f32) -> f32 {
  return select(1.055 * pow(c, 1.0 / 2.4) - 0.055, c * 12.92, c <= 0.0031308);
}

@fragment
fn fs(in: VsOut) -> @location(0) vec4<f32> {
  var s = array<vec4<f32>, 4>(
    textureSample(t0, pack_sampler, in.uv),
    textureSample(t1, pack_sampler, in.uv),
    textureSample(t2, pack_sampler, in.uv),
    textureSample(t3, pack_sampler, in.uv),
  );
  var o = pack.fallback;
  for (var c = 0u; c < 4u; c++) {
    let k = pack.pick[c];
    if (k < 16u) {
      var v = s[k >> 2u][k & 3u];
      if (((pack.flags.x >> c) & 1u) != 0u) { v = srgb_encode(clamp(v, 0.0, 1.0)); }
      o[c] = v;
    }
  }
  if (pack.flags.y != 0u) {
    // A two-channel normal map (BC5, EAC RG11): z from x and y.
    let xy = o.xy * 2.0 - 1.0;
    o.z = sqrt(max(1.0 - dot(xy, xy), 0.0)) * 0.5 + 0.5;
  }
  return o;
}
