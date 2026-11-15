// The GGX specular lobe, one copy for direct light (lighting.wgsl `brdf`,
// physical.wgsl `physical_brdf`), for the image-based light's prefilter
// and split-sum table (ibl.wgsl) and for screen-space reflections
// (ssr.wgsl), so they agree. Both functions take alpha = roughness
// squared. Prepended to lighting.wgsl, ibl.wgsl and ssr.wgsl.

// The GGX (Trowbridge-Reitz) distribution of microfacet normals at
// `n_dot_h`, for alpha = roughness squared.
fn d_ggx(n_dot_h: f32, alpha: f32) -> f32 {
  let a2 = alpha * alpha;
  let d = n_dot_h * n_dot_h * (a2 - 1.0) + 1.0;
  return a2 / (PI * d * d + 1e-7);
}

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
