// The surface's display transfer, after linear scene/UI composition
// (post/display.rs): the frame, linear sRGB relative to SDR white, limited
// to the Display P3 gamut and the headroom, then written in the surface's
// space (scRGB, extended sRGB, or Display P3 at either range).
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
// x: scale (scRGB's 80-nit unit), y: 1 to sRGB-encode, z: headroom, w: 1 for P3 primaries.
@group(0) @binding(2) var<uniform> output: vec4f;

// Linear sRGB to linear Display P3 and back (CSS Color 4 §10, D65).
const SRGB_TO_P3 = mat3x3<f32>(
  vec3<f32>(0.8224621, 0.0331941, 0.0170827),
  vec3<f32>(0.1775380, 0.9668058, 0.0723974),
  vec3<f32>(0.0000000, 0.0000000, 0.9105199));
const P3_TO_SRGB = mat3x3<f32>(
  vec3<f32>(1.2249401, -0.0420569, -0.0196376),
  vec3<f32>(-0.2249404, 1.0420571, -0.0786361),
  vec3<f32>(0.0000000, 0.0000000, 1.0982735));

@fragment
fn fs(v: VsOut) -> @location(0) vec4f {
  let p3 = clamp(SRGB_TO_P3 * sample_exact(source, source_sampler, v.uv).rgb, vec3f(0.0), vec3f(output.z));
  var color = select(P3_TO_SRGB * p3, p3, output.w > 0.5) * output.x;
  if (output.y > 0.5) {
    // Extended sRGB continues the curve below zero, mirrored.
    color = sign(color) * srgb_encode(abs(color));
  }
  return vec4f(color, 1.0);
}
