// Colour, one copy for every shader (LIBRARY_WGSL, after common.wgsl): the
// sRGB transfer curve both ways, the luminance of linear Rec.709 light, a
// hue rotation and Oklab both ways.
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

// `rgb` with its hue turned by `a` radians: Rodrigues' rotation about the
// grey axis, so grey stays grey and the sum of the channels is kept.
fn hue_rotate(rgb: vec3<f32>, a: f32) -> vec3<f32> {
  let k = vec3<f32>(0.57735);
  let c = cos(a);
  return rgb * c + cross(k, rgb) * sin(a) + k * dot(k, rgb) * (1.0 - c);
}

// Linear light to Oklab (Ottosson 2020, "A perceptual color space for image
// processing"): lightness, then the green-red and blue-yellow axes. The
// cube root keeps its sign, as the reference's cbrtf does, so a colour
// outside the gamut (a negative channel) maps to Oklab and back to itself.
fn oklab_encode(c: vec3<f32>) -> vec3<f32> {
  let lms = vec3<f32>(
    0.4122214708 * c.r + 0.5363325363 * c.g + 0.0514459929 * c.b,
    0.2119034982 * c.r + 0.6806995451 * c.g + 0.1073969566 * c.b,
    0.0883024619 * c.r + 0.2817188376 * c.g + 0.6299787005 * c.b);
  let r = sign(lms) * pow(abs(lms), vec3<f32>(1.0 / 3.0));
  return vec3<f32>(
    0.2104542553 * r.x + 0.7936177850 * r.y - 0.0040720468 * r.z,
    1.9779984951 * r.x - 2.4285922050 * r.y + 0.4505937099 * r.z,
    0.0259040371 * r.x + 0.7827717662 * r.y - 0.8086757660 * r.z);
}

// Oklab to linear light: oklab_encode's inverse.
fn oklab_decode(lab: vec3<f32>) -> vec3<f32> {
  let r = vec3<f32>(
    lab.x + 0.3963377774 * lab.y + 0.2158037573 * lab.z,
    lab.x - 0.1055613458 * lab.y - 0.0638541728 * lab.z,
    lab.x - 0.0894841775 * lab.y - 1.2914855480 * lab.z);
  let lms = r * r * r;
  return vec3<f32>(
    4.0767416621 * lms.x - 3.3077115913 * lms.y + 0.2309699292 * lms.z,
    -1.2684380046 * lms.x + 2.6097574011 * lms.y - 0.3413193965 * lms.z,
    -0.0041960863 * lms.x - 0.7034186147 * lms.y + 1.7076147010 * lms.z);
}
