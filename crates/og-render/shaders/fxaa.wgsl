// FXAA (Timothy Lottes' FXAA 3.11, its quality path) over the tonemapped
// image: `Antialias::Fxaa`, and `Tier`/`Msaa4` where the HDR target cannot be
// multisampled. A pixel whose 4-neighbourhood's perceptual luma range is
// below the threshold is copied as it is; on an edge, the edge's direction
// and its two ends are searched (eight steps each way, widening), and the
// pixel is resampled across the edge by how far it sits from the nearer
// end, or by the sub-pixel blend of its 3x3 neighbourhood, whichever is
// larger. Reads the LDR target (sRGB-encoded 8 bits, sampled as linear).
// Prepended by fullscreen.wgsl.
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;

// FXAA 3.11's "high quality" settings.
const EDGE_THRESHOLD: f32 = 0.125;
const EDGE_THRESHOLD_MIN: f32 = 0.0312;
const SUBPIX: f32 = 0.75;
const SEARCH_STEPS: i32 = 8;

fn search_step(i: i32) -> f32 {
  switch i {
    case 0: { return 1.0; }
    case 1: { return 1.5; }
    case 2, 3, 4: { return 2.0; }
    case 5: { return 4.0; }
    default: { return 8.0; }
  }
}

// Perceptual luma of a linear colour (gamma 2.0, close to sRGB's curve).
fn fxaa_luma(c: vec3<f32>) -> f32 {
  return sqrt(max(dot(c, LUMA), 0.0));
}

fn luma_at(px: vec2<i32>, size: vec2<i32>) -> f32 {
  return fxaa_luma(textureLoad(source, clamp(px, vec2<i32>(0), size - 1), 0).rgb);
}

fn luma_uv(uv: vec2<f32>) -> f32 {
  return fxaa_luma(textureSampleLevel(source, source_sampler, uv, 0.0).rgb);
}

fn fxaa(pos: vec2<f32>) -> vec3<f32> {
  let size = vec2<i32>(textureDimensions(source));
  let rcp = 1.0 / vec2<f32>(size);
  let px = vec2<i32>(pos);
  let uv = (vec2<f32>(px) + 0.5) * rcp;
  let center = textureLoad(source, px, 0).rgb;
  let m = fxaa_luma(center);
  // y down: N is the row above.
  let n = luma_at(px + vec2<i32>(0, -1), size);
  let s = luma_at(px + vec2<i32>(0, 1), size);
  let e = luma_at(px + vec2<i32>(1, 0), size);
  let w = luma_at(px + vec2<i32>(-1, 0), size);
  let max_luma = max(max(max(n, s), max(e, w)), m);
  let min_luma = min(min(min(n, s), min(e, w)), m);
  let range = max_luma - min_luma;
  if (range < max(EDGE_THRESHOLD_MIN, max_luma * EDGE_THRESHOLD)) {
    return center;
  }
  let nw = luma_at(px + vec2<i32>(-1, -1), size);
  let ne = luma_at(px + vec2<i32>(1, -1), size);
  let sw = luma_at(px + vec2<i32>(-1, 1), size);
  let se = luma_at(px + vec2<i32>(1, 1), size);

  // Sub-pixel aliasing: how far the centre is from its 3x3 low-pass.
  let ns = n + s;
  let we = w + e;
  let average = ((ns + we) * 2.0 + (nw + ne + sw + se)) / 12.0;
  let subpix_c = clamp(abs(average - m) / range, 0.0, 1.0);
  let subpix_d = (-2.0 * subpix_c + 3.0) * subpix_c * subpix_c;
  let subpix = subpix_d * subpix_d * SUBPIX;

  // The edge's orientation: a horizontal edge has its gradient along y.
  let edge_h = abs(-2.0 * w + nw + sw) + abs(-2.0 * m + ns) * 2.0 + abs(-2.0 * e + ne + se);
  let edge_v = abs(-2.0 * n + nw + ne) + abs(-2.0 * m + we) * 2.0 + abs(-2.0 * s + sw + se);
  let horizontal = edge_h >= edge_v;
  var step_len = select(rcp.x, rcp.y, horizontal);
  var l1 = select(w, n, horizontal);
  var l2 = select(e, s, horizontal);
  let g1 = l1 - m;
  let g2 = l2 - m;
  let first = abs(g1) >= abs(g2);
  let gradient_scaled = 0.25 * max(abs(g1), abs(g2));
  var local_avg: f32;
  if (first) {
    step_len = -step_len;
    local_avg = 0.5 * (l1 + m);
  } else {
    local_avg = 0.5 * (l2 + m);
  }
  // Half a pixel across, onto the edge.
  var edge_uv = uv;
  if (horizontal) { edge_uv.y += step_len * 0.5; } else { edge_uv.x += step_len * 0.5; }
  let along = select(vec2<f32>(0.0, rcp.y), vec2<f32>(rcp.x, 0.0), horizontal);

  var uv1 = edge_uv - along;
  var uv2 = edge_uv + along;
  var end1 = luma_uv(uv1) - local_avg;
  var end2 = luma_uv(uv2) - local_avg;
  var done1 = abs(end1) >= gradient_scaled;
  var done2 = abs(end2) >= gradient_scaled;
  for (var i = 1; i < SEARCH_STEPS; i++) {
    if (done1 && done2) { break; }
    let q = search_step(i);
    if (!done1) {
      uv1 -= along * q;
      end1 = luma_uv(uv1) - local_avg;
      done1 = abs(end1) >= gradient_scaled;
    }
    if (!done2) {
      uv2 += along * q;
      end2 = luma_uv(uv2) - local_avg;
      done2 = abs(end2) >= gradient_scaled;
    }
  }
  let d1 = select(uv.y - uv1.y, uv.x - uv1.x, horizontal);
  let d2 = select(uv2.y - uv.y, uv2.x - uv.x, horizontal);
  let nearer_is_1 = d1 < d2;
  let dist = min(d1, d2);
  let span = d1 + d2;
  // Blend only when the nearer end's luma moves away from the centre's side.
  let centre_below = m - local_avg < 0.0;
  let good = select((end2 < 0.0) != centre_below, (end1 < 0.0) != centre_below, nearer_is_1);
  let edge_offset = select(0.0, 0.5 - dist / span, good);
  let offset = max(edge_offset, subpix);
  var final_uv = uv;
  if (horizontal) { final_uv.y += offset * step_len; } else { final_uv.x += offset * step_len; }
  return textureSampleLevel(source, source_sampler, final_uv, 0.0).rgb;
}

fn fxaa_srgb_encode(c: vec3<f32>) -> vec3<f32> {
  let lo = c * 12.92;
  let hi = 1.055 * pow(c, vec3<f32>(1.0 / 2.4)) - 0.055;
  return select(hi, lo, c <= vec3<f32>(0.0031308));
}

@fragment
fn fs(in: VsOut) -> @location(0) vec4<f32> {
  return vec4<f32>(fxaa(in.pos.xy), 1.0);
}

// A non-sRGB output (web canvases) gets the sRGB curve in the shader.
@fragment
fn fs_srgb_encode(in: VsOut) -> @location(0) vec4<f32> {
  return vec4<f32>(fxaa_srgb_encode(clamp(fxaa(in.pos.xy), vec3<f32>(0.0), vec3<f32>(1.0))), 1.0);
}
