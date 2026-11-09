// What the UI's blends share (src/ui/pass.rs), after ui_common.wgsl and
// ui_blend_modes.wgsl: the group 2 a shape or a layer's composite reads
// what is under it from, and the blend over it.

struct BackdropParams {
  // x: 1 when `behind` holds sRGB-encoded values; y: 1 when it holds
  // premultiplied colour with its alpha (a layer's target: a group's
  // backdrop starts transparent); w: the pyramid's levels.
  params: vec4<f32>,
};
// The blur pyramid of what is under a backdrop (src/glass.rs), and a copy
// of the target as drawn so far (the blends that read it).
@group(2) @binding(0) var pyramid_tex: texture_2d<f32>;
@group(2) @binding(1) var behind_tex: texture_2d<f32>;
@group(2) @binding(2) var backdrop_sampler: sampler;
@group(2) @binding(3) var<uniform> backdrop: BackdropParams;

// Straight linear colour `rgb` at alpha `a` blended by `mode` (UiBlend's
// code, src/ui/build.rs) over what is under device position `position`,
// as the target's alpha blending lays it: Compositing and Blending 1 §5.2,
// Cs' = (1 - ab) Cs + ab B(Cb, Cs), so over a group's transparent
// backdrop the colour is itself. Blends run on sRGB-encoded values, as
// CSS does, and decode back to the linear light drawn here; plus-lighter
// (3) adds in linear light.
fn blend_over(rgb: vec3<f32>, a: f32, mode: u32, position: vec2<f32>) -> vec4<f32> {
  if (mode == 0u) {
    return finish_color(rgb, a);
  }
  let size = vec2<i32>(textureDimensions(behind_tex));
  let at = clamp(vec2<i32>(position), vec2<i32>(0), size - 1);
  let t = textureLoad(behind_tex, at, 0);
  var b = t.rgb;
  var ab = 1.0;
  if (backdrop.params.y > 0.5) {
    ab = clamp(t.a, 0.0, 1.0);
    b = t.rgb / max(t.a, 1e-6);
  }
  if (backdrop.params.x > 0.5) {
    b = decode(b);
  }
  if (mode == 3u) {
    return finish_color(b * ab + rgb, a);
  }
  let src = encode(clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0)));
  let bb = encode(clamp(b, vec3<f32>(0.0), vec3<f32>(1.0)));
  var r: vec3<f32>;
  switch mode {
    case 13u: { r = set_lum(set_sat(src, sat(bb)), lum(bb)); }
    case 14u: { r = set_lum(set_sat(bb, sat(src)), lum(bb)); }
    case 15u: { r = set_lum(src, lum(bb)); }
    case 16u: { r = set_lum(bb, lum(src)); }
    default: {
      r = vec3<f32>(blend_channel(bb.r, src.r, mode), blend_channel(bb.g, src.g, mode), blend_channel(bb.b, src.b, mode));
    }
  }
  r = mix(src, clamp(r, vec3<f32>(0.0), vec3<f32>(1.0)), ab);
  return finish_color(decode(r), a);
}
