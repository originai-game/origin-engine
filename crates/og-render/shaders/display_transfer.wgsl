// The surface's display transfer, after linear scene/UI composition
// (post/display.rs): the frame, linear sRGB relative to SDR white, limited
// to the Display P3 gamut and the headroom, then written in the surface's
// space (scRGB, extended sRGB, or Display P3 at either range).
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
// x: scale (scRGB's 80-nit unit), y: 1 to sRGB-encode, z: headroom, w: 1 for P3 primaries.
@group(0) @binding(2) var<uniform> output: vec4f;

@fragment
fn fs(v: VsOut) -> @location(0) vec4f {
  var p3 = SRGB_TO_P3 * sample_exact(source, source_sampler, v.uv).rgb;
  // Luminance-preserving P3 gamut compression before headroom limiting:
  // desaturate toward P3 luma instead of clipping negative channels, which twists hue.
  let y = max(dot(p3, vec3f(0.22897, 0.69174, 0.07929)), 0.0);
  let lo = min(p3.r, min(p3.g, p3.b));
  if (lo < 0.0) {
    let t = y / max(y - lo, 1e-5);
    p3 = mix(vec3f(y), p3, t);
  }
  let hi = max(p3.r, max(p3.g, p3.b));
  if (hi > output.z) {
    p3 *= output.z / max(hi, 1e-5);
  }
  p3 = clamp(p3, vec3f(0.0), vec3f(output.z));
  var color = select(P3_TO_SRGB * p3, p3, output.w > 0.5) * output.x;
  if (output.y > 0.5) {
    // Extended sRGB continues the curve below zero, mirrored.
    color = sign(color) * srgb_encode(abs(color));
    if (output.z <= 1.01) {
      let dither = (ign(v.pos.xy) + ign(v.pos.xy + vec2f(17.0, 31.0)) - 1.0) / 255.0;
      color += vec3f(dither);
    }
  }
  return vec4f(color, 1.0);
}
