// Half-resolution soft particle overdraw governance (particles/half_res.rs):
// - `fs_downsample`: reduces full-resolution linear view depth (`lindepth`)
//   into a half-resolution 2x2 (max_depth, min_depth) pyramid. Soft particles
//   shade against `max_depth` so silhouettes keep background smoke behind thin
//   occluders.
// - `fs_upsample`: composites the half-resolution premultiplied-alpha +
//   transmittance particle buffer (`fx.half`) back onto the full-resolution
//   HDR target using depth-aware bilateral weights (nearest-depth weighting),
//   eliminating foreground bleeding and haloing around depth edges.

struct VsOut {
  @builtin(position) pos: vec4<f32>,
};

@vertex
fn vs(@builtin(vertex_index) i: u32) -> VsOut {
  let x = f32((i << 1u) & 2u) * 2.0 - 1.0;
  let y = 1.0 - f32(i & 2u) * 2.0;
  return VsOut(vec4<f32>(x, y, 0.0, 1.0));
}

@group(0) @binding(0) var full_depth: texture_2d<f32>;
@group(0) @binding(1) var fx_half: texture_2d<f32>;
@group(0) @binding(2) var fx_half_depth: texture_2d<f32>;

@fragment
fn fs_downsample(in: VsOut) -> @location(0) vec4<f32> {
  let sz = vec2<i32>(textureDimensions(full_depth));
  let mx = max(sz - vec2<i32>(1), vec2<i32>(0));
  let p = vec2<i32>(in.pos.xy) * 2;
  let d00 = textureLoad(full_depth, min(p, mx), 0).r;
  let d10 = textureLoad(full_depth, min(p + vec2<i32>(1, 0), mx), 0).r;
  let d01 = textureLoad(full_depth, min(p + vec2<i32>(0, 1), mx), 0).r;
  let d11 = textureLoad(full_depth, min(p + vec2<i32>(1, 1), mx), 0).r;
  let d_max = max(max(d00, d10), max(d01, d11));
  let d_min = min(min(d00, d10), min(d01, d11));
  return vec4<f32>(d_max, d_min, 0.0, 0.0);
}

fn sample_bilateral(pos_xy: vec2<f32>) -> vec4<f32> {
  let full_sz = vec2<i32>(textureDimensions(full_depth));
  let fp = min(vec2<i32>(pos_xy), max(full_sz - vec2<i32>(1), vec2<i32>(0)));
  let d_full = textureLoad(full_depth, fp, 0).r;

  let half_sz = vec2<i32>(textureDimensions(fx_half));
  let half_mx = max(half_sz - vec2<i32>(1), vec2<i32>(0));
  let hc = pos_xy * 0.5 - vec2<f32>(0.5);
  let b = vec2<i32>(floor(hc));
  let f = fract(hc);

  let p00 = clamp(b, vec2<i32>(0), half_mx);
  let p10 = clamp(b + vec2<i32>(1, 0), vec2<i32>(0), half_mx);
  let p01 = clamp(b + vec2<i32>(0, 1), vec2<i32>(0), half_mx);
  let p11 = clamp(b + vec2<i32>(1, 1), vec2<i32>(0), half_mx);

  let c00 = textureLoad(fx_half, p00, 0);
  let c10 = textureLoad(fx_half, p10, 0);
  let c01 = textureLoad(fx_half, p01, 0);
  let c11 = textureLoad(fx_half, p11, 0);

  let z00 = textureLoad(fx_half_depth, p00, 0).rg;
  let z10 = textureLoad(fx_half_depth, p10, 0).rg;
  let z01 = textureLoad(fx_half_depth, p01, 0).rg;
  let z11 = textureLoad(fx_half_depth, p11, 0).rg;

  let e00 = min(abs(d_full - z00.r) / max(min(d_full, z00.r), 0.25), 100.0);
  let e10 = min(abs(d_full - z10.r) / max(min(d_full, z10.r), 0.25), 100.0);
  let e01 = min(abs(d_full - z01.r) / max(min(d_full, z01.r), 0.25), 100.0);
  let e11 = min(abs(d_full - z11.r) / max(min(d_full, z11.r), 0.25), 100.0);

  // An edge cell (z.r > z.g * 1.15) shaded particles against d_max (background);
  // if d_full is closer to d_min (foreground occluder), penalise background bleed.
  let fg00 = z00.r > z00.g * 1.15 && abs(d_full - z00.g) < abs(d_full - z00.r);
  let fg10 = z10.r > z10.g * 1.15 && abs(d_full - z10.g) < abs(d_full - z10.r);
  let fg01 = z01.r > z01.g * 1.15 && abs(d_full - z01.g) < abs(d_full - z01.r);
  let fg11 = z11.r > z11.g * 1.15 && abs(d_full - z11.g) < abs(d_full - z11.r);

  let k00 = 1.0 / (1e-4 + e00 * e00 * 64.0) * select(1.0, 0.02, fg00);
  let k10 = 1.0 / (1e-4 + e10 * e10 * 64.0) * select(1.0, 0.02, fg10);
  let k01 = 1.0 / (1e-4 + e01 * e01 * 64.0) * select(1.0, 0.02, fg01);
  let k11 = 1.0 / (1e-4 + e11 * e11 * 64.0) * select(1.0, 0.02, fg11);

  let w00 = (1.0 - f.x) * (1.0 - f.y) * k00;
  let w10 = f.x * (1.0 - f.y) * k10;
  let w01 = (1.0 - f.x) * f.y * k01;
  let w11 = f.x * f.y * k11;

  let wsum = max(w00 + w10 + w01 + w11, 1e-6);
  let out = (c00 * w00 + c10 * w10 + c01 * w01 + c11 * w11) / wsum;
  // When every neighbour is an edge cell whose background depth is far behind
  // this foreground pixel, blend toward transparent (0,0,0,1) instead of
  // re-normalising background smoke to 100% opacity.
  let fg_all = fg00 && fg10 && fg01 && fg11;
  if (fg_all) {
    let min_e = min(min(e00, e10), min(e01, e11));
    let keep = clamp(1.0 - smoothstep(0.1, 0.5, min_e), 0.0, 1.0);
    return mix(vec4<f32>(0.0, 0.0, 0.0, 1.0), out, keep);
  }
  return out;
}

@fragment
fn fs_upsample(in: VsOut) -> @location(0) vec4<f32> {
  return sample_bilateral(in.pos.xy);
}

// Stamps half-res soft FX coverage into an Rg8Unorm reactive revealage target
// (`oit.reveal` or `fx.reactive`) with multiplicative blending so TAA and
// FSR 3.1 reduce history trust over fast additive fireballs and soft clouds.
@fragment
fn fs_reactive(in: VsOut) -> @location(0) vec4<f32> {
  let c = sample_bilateral(in.pos.xy);
  let peak = max(c.r, max(c.g, c.b));
  let react = clamp(max(1.0 - c.a, peak / (1.0 + peak)), 0.0, 0.85);
  return vec4<f32>(1.0, 1.0 - react, 0.0, 1.0);
}
