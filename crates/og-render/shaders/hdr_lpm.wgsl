// AMD FidelityFX LPM, Rec.709 full-precision specialization.
// Copyright (c) 2017-2019 Advanced Micro Devices, Inc. MIT.
// Full notice and source revision: third_party/lpm/.
// No primaries conversion, saturation/contrast/shoulder powers are one.
// Curve anchors are calibrated on the CPU; output is relative to paper white.
fn hdr_lpm(input: vec3f, settings: vec4f) -> vec3f {
  let LUMA = vec3f(0.2126, 0.7152, 0.0722);
  let color = max(input, vec3f(0.0));
  if (settings.x == 1.0) { return min(color, vec3f(1.0)); }
  let largest = max(color.r, max(color.g, color.b));
  if (largest == 0.0) { return vec3f(0.0); }
  let ratio = color / largest;
  let original_luma = dot(color, LUMA);
  let luma = clamp(original_luma / (original_luma * settings.y + settings.z), 0.0, 1.0);
  let ratio_scale = clamp(luma / dot(ratio, LUMA), 0.0, 1.0);
  var mapped = clamp(ratio * ratio_scale, vec3f(0.0), vec3f(1.0));
  let capacity = (1.0 - mapped) * vec3f(1.0, 0.5, 1.0 / 32.0);
  var add = clamp(luma - dot(mapped, LUMA), 0.0, 1.0);
  if (add > 0.0) {
    mapped = clamp(mapped + capacity * (add / dot(capacity, LUMA)), vec3f(0.0), vec3f(1.0));
    add = clamp(luma - dot(mapped, LUMA), 0.0, 1.0);
    mapped = clamp(mapped + add / LUMA, vec3f(0.0), vec3f(1.0));
  }
  return mapped * settings.x;
}
