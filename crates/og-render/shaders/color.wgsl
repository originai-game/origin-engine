// Colour, one copy for every shader (LIBRARY_WGSL, after common.wgsl): the
// sRGB transfer curve both ways, the luminance of linear Rec.709 light, a
// hue rotation, Oklab both ways, Display P3's primaries both ways and CSS
// gamut mapping into sRGB or Display P3.
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

// Linear sRGB to linear Display P3 and back (CSS Color 4 §10, D65).
const SRGB_TO_P3 = mat3x3<f32>(
  vec3<f32>(0.8224621, 0.0331941, 0.0170827),
  vec3<f32>(0.1775380, 0.9668058, 0.0723974),
  vec3<f32>(0.0000000, 0.0000000, 0.9105199));
const P3_TO_SRGB = mat3x3<f32>(
  vec3<f32>(1.2249401, -0.0420569, -0.0196376),
  vec3<f32>(-0.2249404, 1.0420571, -0.0786361),
  vec3<f32>(0.0000000, 0.0000000, 1.0982735));

// Linear sRGB `c` in the gamut's own linear space: itself, or Display P3.
fn gamut_space(c: vec3<f32>, p3: bool) -> vec3<f32> {
  if (p3) {
    return SRGB_TO_P3 * c;
  }
  return c;
}

// Whether linear sRGB `c` is inside sRGB, or Display P3 when `p3`.
fn gamut_inside(c: vec3<f32>, p3: bool) -> bool {
  let v = gamut_space(c, p3);
  return all(v >= vec3<f32>(-1e-5)) && all(v <= vec3<f32>(1.0 + 1e-5));
}

// Linear sRGB `c` clipped to sRGB, or Display P3 when `p3`, as linear sRGB.
fn gamut_clip(c: vec3<f32>, p3: bool) -> vec3<f32> {
  let v = clamp(gamut_space(c, p3), vec3<f32>(0.0), vec3<f32>(1.0));
  if (p3) {
    return P3_TO_SRGB * v;
  }
  return v;
}

// Oklab `lab` as linear sRGB inside sRGB, or Display P3 when `p3`, by CSS
// Color 4 §13.2: chroma is lowered at the colour's lightness and hue until
// clipping what is left moves it less than a just-noticeable difference
// (deltaEOK 0.02), so a ramp leaving the gamut keeps its lightness and hue
// and shows no crease where a clip would turn its hue. A colour inside is
// itself; one at or past white or black is white or black. The same
// search as og-css's gamut_map, on the GPU per pixel.
fn gamut_map(lab: vec3<f32>, p3: bool) -> vec3<f32> {
  let rgb = oklab_decode(lab);
  if (gamut_inside(rgb, p3)) {
    return rgb;
  }
  if (lab.x >= 1.0) {
    return vec3<f32>(1.0);
  }
  if (lab.x <= 0.0) {
    return vec3<f32>(0.0);
  }
  let jnd = 0.02;
  let chroma = length(lab.yz);
  let hue = lab.yz / max(chroma, 1e-9);
  var clipped = gamut_clip(rgb, p3);
  if (distance(oklab_encode(clipped), lab) < jnd) {
    return clipped;
  }
  var lo = 0.0;
  var hi = chroma;
  var lo_inside = true;
  // Bisection to 1e-4 of chroma: 16 halvings cover any chroma below 6.
  for (var i = 0; i < 16; i++) {
    if (hi - lo <= 1e-4) {
      break;
    }
    let c = 0.5 * (lo + hi);
    let current = vec3<f32>(lab.x, hue * c);
    let cur = oklab_decode(current);
    if (lo_inside && gamut_inside(cur, p3)) {
      lo = c;
      continue;
    }
    clipped = gamut_clip(cur, p3);
    let e = distance(oklab_encode(clipped), current);
    if (e < jnd) {
      if (jnd - e < 1e-4) {
        break;
      }
      lo_inside = false;
      lo = c;
    } else {
      hi = c;
    }
  }
  return clipped;
}
