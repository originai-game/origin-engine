// The fullscreen triangle every post pass draws. Prepended to the post shaders.
struct VsOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };

@vertex
fn vs(@builtin(vertex_index) i: u32) -> VsOut {
  var o: VsOut;
  o.pos = fullscreen_clip(i);
  o.uv = vec2<f32>(o.pos.x * 0.5 + 0.5, 0.5 - o.pos.y * 0.5);
  return o;
}

const LUMA: vec3<f32> = vec3<f32>(0.2126, 0.7152, 0.0722);

// A linear read of `t` at `uv`, exact where `uv` is one of its texel
// centres: there the filter's answer is that texel, so it is loaded. A
// centre is rarely representable ((x + 0.5) / width), and a filter rounds
// it onto its weight grid as it likes: PowerVR (Pixel 10) leaks 1/256 of
// the neighbour, so a 1:1 read through a linear sample was not a copy
// (tests/evidence/compat/android/filter-precision). A read between
// centres (a scaled or offset one) is filtered as before.
fn sample_exact(t: texture_2d<f32>, s: sampler, uv: vec2<f32>) -> vec4<f32> {
  let size = vec2<f32>(textureDimensions(t));
  let texel = uv * size - 0.5;
  let centre = round(texel);
  if (all(abs(texel - centre) < vec2<f32>(1.0 / 1024.0)) && all(centre >= vec2<f32>(0.0)) && all(centre < size)) {
    return textureLoad(t, vec2<i32>(centre), 0);
  }
  return textureSampleLevel(t, s, uv, 0.0);
}
