// The colour vision filter (src/color_vision.rs): the finished frame, scene
// and overlay, copied from the glass source into the output through one of
// two 3x3 matrices in linear sRGB, the one on the side of `plane` the pixel
// lies on (a single-matrix filter repeats it). Same layout as glass.wgsl.
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
struct FilterParams {
  a0: vec4<f32>,
  a1: vec4<f32>,
  a2: vec4<f32>,
  b0: vec4<f32>,
  b1: vec4<f32>,
  b2: vec4<f32>,
  // xyz: the plane's normal; w: 1 when the source holds sRGB-encoded
  // values (decoded before the matrices, encoded after).
  plane: vec4<f32>,
};
@group(0) @binding(2) var<uniform> cvd: FilterParams;

fn decode(c: vec3<f32>) -> vec3<f32> {
  let lo = c / 12.92;
  let hi = pow((c + 0.055) / 1.055, vec3<f32>(2.4));
  return select(hi, lo, c <= vec3<f32>(0.04045));
}

fn encode(c: vec3<f32>) -> vec3<f32> {
  let lo = c * 12.92;
  let hi = 1.055 * pow(c, vec3<f32>(1.0 / 2.4)) - 0.055;
  return select(hi, lo, c <= vec3<f32>(0.0031308));
}

@fragment
fn fs_filter(in: VsOut) -> @location(0) vec4<f32> {
  let texel = textureLoad(source, vec2<i32>(in.pos.xy), 0);
  let coded = cvd.plane.w > 0.5;
  var rgb = texel.rgb;
  if (coded) {
    rgb = decode(rgb);
  }
  var o: vec3<f32>;
  if (dot(cvd.plane.xyz, rgb) >= 0.0) {
    o = vec3<f32>(dot(cvd.a0.xyz, rgb), dot(cvd.a1.xyz, rgb), dot(cvd.a2.xyz, rgb));
  } else {
    o = vec3<f32>(dot(cvd.b0.xyz, rgb), dot(cvd.b1.xyz, rgb), dot(cvd.b2.xyz, rgb));
  }
  o = max(o, vec3<f32>(0.0));
  if (coded) {
    o = encode(min(o, vec3<f32>(1.0)));
  }
  return vec4<f32>(o, texel.a);
}
