// Light that spreads under a surface before it leaves (physical.rs
// `subsurface`): skin, wax, marble, jade, milk. A forward renderer has no
// image of the diffuse light to blur, so the light's falloff across the
// surface is blurred instead, by the surface's own curvature
// (pre-integrated subsurface scattering: Penner and Borshukov, GPU Pro 2,
// 2011), in closed form where the paper bakes a table: the lit pipelines
// have no texture slot for one.
//
// The profile is Christensen and Burley's normalised diffusion (2015): of
// the light that enters at a point, a quarter leaves within a lobe of
// scale d and three quarters within one of 3 d, d the scatter distance of
// a colour channel. Each lobe is taken as a Gaussian of the same second
// moment. On a surface that curves with radius 1 / curvature, a Gaussian of
// that width blurs the clamped cosine over the sphere, which scales its
// band l (Ramamoorthi and Hanrahan, 2001) by exp(-l (l + 1) t), t half the
// squared width in radians.

// The clamped cosine max(x, 0) blurred over the sphere by a Gaussian of
// variance 2 t: its bands 0, 1, 2 and 4 each by its own factor, and what
// is left of it (the bands from 6 up, its kink) by band 6's. t 0: max(x, 0);
// a large t: the sphere's mean, 1/4. Its mean over x is 1/4 at every t:
// the light is moved, never made or lost.
fn scattered_cosine(x: f32, t: f32) -> f32 {
  let x2 = x * x;
  let p2 = 1.5 * x2 - 0.5;
  let p4 = 4.375 * x2 * x2 - 3.75 * x2 + 0.375;
  let bands = vec3<f32>(0.5 * x, 0.3125 * p2, -0.09375 * p4);
  let rest = max(x, 0.0) - 0.25 - bands.x - bands.y - bands.z;
  let kept = exp(-t * vec4<f32>(2.0, 6.0, 20.0, 42.0));
  return max(0.25 + dot(vec4<f32>(bands, rest), kept), 0.0);
}

// A light's falloff on a surface of `curvature` (1 / m) whose light
// scatters `radius` (m, per channel) under it, in place of max(x, 0):
// `x` the cosine between the normal and the light.
fn subsurface_cosine(x: f32, curvature: f32, radius: vec3<f32>) -> vec3<f32> {
  let width = radius * curvature;
  let t = 0.5 * width * width;
  let near = vec3<f32>(scattered_cosine(x, t.r), scattered_cosine(x, t.g), scattered_cosine(x, t.b));
  let far = vec3<f32>(scattered_cosine(x, 9.0 * t.r), scattered_cosine(x, 9.0 * t.g), scattered_cosine(x, 9.0 * t.b));
  return 0.25 * near + 0.75 * far;
}

// How much of the light that enters one side of a sheet `thickness` (m)
// thick leaves the other: the profile's share past that distance.
fn subsurface_through(thickness: f32, radius: vec3<f32>) -> vec3<f32> {
  let d = max(radius, vec3<f32>(1e-6));
  return 0.25 * exp(-thickness / d) + 0.75 * exp(-thickness / (3.0 * d));
}

// How much of the light arriving behind a surface its far side faces (the
// light that comes through it): none on the lit side, most straight
// through (Jimenez et al., 2010).
fn subsurface_back(n: vec3<f32>, l: vec3<f32>) -> f32 {
  return clamp(0.3 - dot(n, l), 0.0, 1.0);
}
