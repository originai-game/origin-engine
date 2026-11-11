// The GGX specular visibility, one function for direct light (lighting.wgsl
// `brdf`, physical.wgsl `physical_brdf`) and for the split-sum table the
// image-based light reads (ibl.wgsl `fs_brdf_lut`), so the two agree.
// Prepended to lighting.wgsl and to ibl.wgsl.

// Height-correlated Smith visibility, V = G / (4 n.l n.v), for
// alpha = roughness squared (Heitz 2014, "Understanding the
// Masking-Shadowing Function in Microfacet-Based BRDFs"; Filament's
// V_SmithGGXCorrelated, Frostbite's). No roughness remap: Karis's
// k = (r + 1)^2 / 8 and the separable form it sat in lost up to three
// quarters of a rough surface's specular at grazing angles.
fn v_ggx(n_dot_v: f32, n_dot_l: f32, alpha: f32) -> f32 {
  let a2 = alpha * alpha;
  // (Not `gl`: GLSL reserves the gl_ prefix naga's renaming would give it.)
  let lambda_v = n_dot_l * sqrt(n_dot_v * n_dot_v * (1.0 - a2) + a2);
  let lambda_l = n_dot_v * sqrt(n_dot_l * n_dot_l * (1.0 - a2) + a2);
  return 0.5 / max(lambda_v + lambda_l, 1e-6);
}
