// The sheen lobe (MeshPhysicalMaterial.sheen, KHR_materials_sheen): cloth,
// velvet and fuzz as the retroreflection of fibres standing off the
// surface. Estevez and Kulla, "Production Friendly Microfacet Sheen BRDF"
// (Sony Pictures Imageworks, 2017): the "Charlie" distribution and its
// fitted shadowing term with the terminator softened, which glTF's sheen
// extension and Filament use. Its directional albedo, integrated by
// ibl.wgsl `fs_brdf_lut` into the BRDF table's blue channel, scales the
// layers under the sheen so that what the fibres reflect is taken from
// them (energy conservation, as Filament and Imageworks), and lights the
// sheen under the image-based light. Shared by ibl.wgsl and physical.wgsl;
// PI comes from either.

// The Charlie distribution: sin(theta_h)^(1/alpha) normalised.
// `alpha` is the sheen's roughness squared (three.js, Filament).
fn d_charlie(alpha: f32, n_dot_h: f32) -> f32 {
  let inv = 1.0 / max(alpha, 1e-3);
  let sin2h = max(1.0 - n_dot_h * n_dot_h, 0.0078125);
  return (2.0 + inv) * pow(sin2h, inv * 0.5) / (2.0 * PI);
}

// Estevez and Kulla's fit of the distribution's lambda, L(x) = a / (1 + b
// x^c) + d x + e, its five coefficients interpolated between alpha 0 and 1
// by (1 - alpha)^2 (their table 1).
fn sheen_l(alpha: f32, x: f32) -> f32 {
  let t = (1.0 - alpha) * (1.0 - alpha);
  let a = mix(21.5473, 25.3245, t);
  let b = mix(3.82987, 3.32435, t);
  let c = mix(0.19823, 0.16801, t);
  let d = mix(-1.97760, -1.27393, t);
  let e = mix(-4.32054, -4.85967, t);
  return a / (1.0 + b * pow(max(x, 0.0), c)) + d * x + e;
}

// Lambda at cos(theta): the fit is for theta up to 90 degrees from the
// grazing side; past cos 0.5 it is mirrored about it (their eq. 3).
fn sheen_lambda(alpha: f32, cos_theta: f32) -> f32 {
  if (cos_theta < 0.5) { return exp(sheen_l(alpha, cos_theta)); }
  return exp(2.0 * sheen_l(alpha, 0.5) - sheen_l(alpha, 1.0 - cos_theta));
}

// The sheen's visibility, G / (4 n.l n.v), with the terminator softened
// (lambda of the light raised to 1 + 2 (1 - n.l)^8, their eq. 4) so a
// sphere of velvet does not end in a hard line where the light leaves it.
fn v_charlie(alpha: f32, n_dot_v: f32, n_dot_l: f32) -> f32 {
  let lv = sheen_lambda(alpha, n_dot_v);
  let k = 1.0 - n_dot_l;
  let ll = pow(sheen_lambda(alpha, n_dot_l), 1.0 + 2.0 * k * k * k * k * k * k * k * k);
  return 1.0 / ((1.0 + lv + ll) * 4.0 * max(n_dot_v * n_dot_l, 1e-4));
}
