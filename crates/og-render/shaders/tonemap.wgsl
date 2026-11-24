// The tone curves, one copy for the post pass (post.wgsl) and a view's
// resolve (views_color.wgsl): exposed linear radiance in, display-referred
// linear sRGB in 0..1 out (SDR), or 0..headroom on an HDR output
// (`tone_curve_hdr`).

// ACES, Narkowicz's fit.
fn aces(x: vec3<f32>) -> vec3<f32> {
  let a = 2.51; let b = 0.03; let c = 2.43; let d = 0.59; let e = 0.14;
  return clamp((x * (a * x + b)) / (x * (c * x + d) + e), vec3<f32>(0.0), vec3<f32>(1.0));
}

// AgX, base look: the fit Blender 4, Filament and three.js's AgXToneMapping
// use. Linear sRGB in and out; the curve works in an inset Rec.2020 log
// space from -12.47 to +4.03 EV around middle grey.
fn agx_contrast(x: vec3<f32>) -> vec3<f32> {
  let x2 = x * x;
  let x4 = x2 * x2;
  return 15.5 * x4 * x2 - 40.14 * x4 * x + 31.96 * x4 - 6.868 * x2 * x + 0.4298 * x2 + 0.1191 * x - 0.00232;
}

fn agx_ev(x: vec3<f32>, max_ev: f32) -> vec3<f32> {
  let srgb_to_2020 = mat3x3<f32>(
    vec3<f32>(0.6274, 0.0691, 0.0164),
    vec3<f32>(0.3293, 0.9195, 0.0880),
    vec3<f32>(0.0433, 0.0113, 0.8956));
  let rec2020_to_srgb = mat3x3<f32>(
    vec3<f32>(1.6605, -0.1246, -0.0182),
    vec3<f32>(-0.5876, 1.1329, -0.1006),
    vec3<f32>(-0.0728, -0.0083, 1.1187));
  let inset = mat3x3<f32>(
    vec3<f32>(0.856627153315983, 0.137318972929847, 0.11189821299995),
    vec3<f32>(0.0951212405381588, 0.761241990602591, 0.0767994186031903),
    vec3<f32>(0.0482516061458583, 0.101439036467562, 0.811302368396859));
  let outset = mat3x3<f32>(
    vec3<f32>(1.1271005818144368, -0.1413297634984383, -0.14132976349843826),
    vec3<f32>(-0.11060664309660323, 1.157823702216272, -0.11060664309660294),
    vec3<f32>(-0.016493938717834573, -0.016493938717834257, 1.2519364065950405));
  let min_ev = -12.47393;
  var c = inset * (srgb_to_2020 * x);
  c = clamp((log2(max(c, vec3<f32>(1e-10))) - min_ev) / (max_ev - min_ev), vec3<f32>(0.0), vec3<f32>(1.0));
  c = outset * agx_contrast(c);
  c = pow(max(c, vec3<f32>(0.0)), vec3<f32>(2.2));
  return clamp(rec2020_to_srgb * c, vec3<f32>(0.0), vec3<f32>(1.0));
}

fn agx(x: vec3<f32>) -> vec3<f32> {
  return agx_ev(x, 4.026069);
}

// Khronos PBR Neutral, as three.js's NeutralToneMapping: linear sRGB in
// and out. The 0.04 of a dielectric's reflection comes off (less near
// black); a colour whose peak is then under 0.76 passes through, a
// brighter one is scaled so the peak rolls off towards 1 and is mixed
// towards white by how far it rolled.
fn neutral(color: vec3<f32>) -> vec3<f32> {
  let start = 0.76;
  let x = min(color.r, min(color.g, color.b));
  let offset = select(0.04, x - 6.25 * x * x, x < 0.08);
  var c = color - offset;
  let peak = max(c.r, max(c.g, c.b));
  if (peak < start) { return clamp(c, vec3<f32>(0.0), vec3<f32>(1.0)); }
  let d = 1.0 - start;
  let new_peak = 1.0 - d * d / (peak + d - start);
  c *= new_peak / peak;
  let g = 1.0 - 1.0 / (0.15 * (peak - new_peak) + 1.0);
  return clamp(mix(c, vec3<f32>(new_peak), g), vec3<f32>(0.0), vec3<f32>(1.0));
}

// The curve `curve` picks (Tonemap: 0 ACES, 1 none, 2 AgX, 3 Khronos PBR
// Neutral) applied to `c`.
fn tone_curve(c: vec3<f32>, curve: f32) -> vec3<f32> {
  if (curve > 2.5) { return neutral(c); }
  if (curve > 1.5) { return agx(c); }
  if (curve > 0.5) { return clamp(c, vec3<f32>(0.0), vec3<f32>(1.0)); }
  return aces(c);
}

// Khronos PBR Neutral with its shoulder rolling off towards `top` instead
// of 1: below 0.76 it passes through as in SDR, so the HDR image matches
// the SDR one until the highlights, which keep rising to the headroom.
fn neutral_hdr(color: vec3<f32>, top: f32) -> vec3<f32> {
  let start = 0.76;
  let x = min(color.r, min(color.g, color.b));
  let offset = select(0.04, x - 6.25 * x * x, x < 0.08);
  var c = color - offset;
  let peak = max(c.r, max(c.g, c.b));
  if (peak < start) { return clamp(c, vec3<f32>(0.0), vec3<f32>(top)); }
  let d = top - start;
  let new_peak = top - d * d / (peak + d - start);
  c *= new_peak / peak;
  let g = 1.0 - 1.0 / (0.15 * (peak - new_peak) + 1.0);
  return clamp(mix(c, vec3<f32>(new_peak), g), vec3<f32>(0.0), vec3<f32>(top));
}

// `tone_curve` for an extended-range output (post/display.rs `curve`):
// hdr.x the headroom over SDR white the curves end at; ACES and AgX scaled
// by hdr.y and hdr.z so middle grey lands where SDR puts it, with AgX's
// log highlight ceiling expanded by log2(headroom).
fn tone_curve_hdr(c: vec3<f32>, curve: f32, hdr: vec4<f32>) -> vec3<f32> {
  if (curve > 2.5) { return neutral_hdr(c, hdr.x); }
  if (curve > 1.5) {
    let max_ev = 4.026069 + log2(max(hdr.x, 1.0));
    return agx_ev(c * hdr.z, max_ev) * hdr.x;
  }
  if (curve > 0.5) { return clamp(c, vec3<f32>(0.0), vec3<f32>(hdr.x)); }
  return aces(c * hdr.y) * hdr.x;
}
