// Colour, one copy for every shader (LIBRARY_WGSL, after common.wgsl): the
// sRGB transfer curve both ways and the luminance of linear Rec.709 light.
// Callers clamp where they need 0..1; below the knee the curve is linear,
// so a negative channel encodes and decodes to a negative one.

// Linear light to the sRGB curve (IEC 61966-2-1).
fn srgb_encode(c: vec3<f32>) -> vec3<f32> {
  let lo = c * 12.92;
  let hi = 1.055 * pow(max(c, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.4)) - 0.055;
  return select(hi, lo, c <= vec3<f32>(0.0031308));
}

// The sRGB curve to linear light: srgb_encode's inverse.
fn srgb_decode(c: vec3<f32>) -> vec3<f32> {
  let lo = c / 12.92;
  let hi = pow(max((c + 0.055) / 1.055, vec3<f32>(0.0)), vec3<f32>(2.4));
  return select(hi, lo, c <= vec3<f32>(0.04045));
}

// Rec.709 luminance weights of linear light.
const LUMA: vec3<f32> = vec3<f32>(0.2126, 0.7152, 0.0722);

// The luminance of linear light `c`.
fn luma(c: vec3<f32>) -> f32 {
  return dot(c, LUMA);
}
