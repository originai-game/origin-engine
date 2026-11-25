// Compute 256-bin trimmed luminance histogram auto-exposure
// (`post/exposure_hist.rs`): replaces the 3-pass fragment 64x64->8x8->1x1
// geometric mean with a full-screen 256-bin log2-luminance histogram that
// discards the darkest 10% and brightest 5% outliers so sky glare or dark
// letterbox borders never skew exposure. Runs within WebGPU Compat limits
// (workgroup <= 128, one rgba16float storage texture, one atomic storage
// buffer). Prepended by common.wgsl (LUMA).

struct HistParams {
  // x: 1 when lit image alpha is tone-mapped fraction (untonemapped sprites).
  flags: vec4<f32>,
};

@group(0) @binding(0) var<uniform> params: HistParams;
@group(0) @binding(1) var source: texture_2d<f32>;
@group(0) @binding(2) var<storage, read_write> histogram: array<atomic<u32>, 256>;
@group(0) @binding(3) var metered_out: texture_storage_2d<rgba16float, write>;

const MIN_LOG_LUM: f32 = -10.0;
const MAX_LOG_LUM: f32 = 12.0;
const LOG_RANGE: f32 = 22.0;
const INV_LOG_RANGE: f32 = 1.0 / 22.0;

var<workgroup> local_hist: array<atomic<u32>, 256>;
var<workgroup> avg_bins: array<u32, 256>;

fn accumulate_pixel(px: vec2<i32>, dims: vec2<i32>, untonemapped: bool) {
  if (px.x >= dims.x || px.y >= dims.y) { return; }
  let c = textureLoad(source, px, 0);
  let w = select(4u, u32(round(clamp(c.a, 0.0, 1.0) * 4.0)), untonemapped);
  if (w == 0u) { return; }
  let lum = dot(max(c.rgb, vec3<f32>(0.0)), LUMA);
  let t = clamp((log2(max(lum, 1e-5)) - MIN_LOG_LUM) * INV_LOG_RANGE, 0.0, 1.0);
  let bin = min(u32(t * 255.0 + 0.5), 255u);
  atomicAdd(&local_hist[bin], w);
}

// Each 16x8 workgroup (128 threads, WebGPU Compat max) processes a 32x16
// pixel tile (2x2 pixels per thread) into shared memory before merging once
// into the global 256-bin histogram.
@compute @workgroup_size(16, 8, 1)
fn cs_histogram(
  @builtin(workgroup_id) wg: vec3<u32>,
  @builtin(local_invocation_id) lid: vec3<u32>,
  @builtin(local_invocation_index) li: u32,
) {
  atomicStore(&local_hist[li], 0u);
  atomicStore(&local_hist[li + 128u], 0u);
  workgroupBarrier();

  let dims = vec2<i32>(textureDimensions(source));
  let base = vec2<i32>(wg.xy * vec2<u32>(32u, 16u) + lid.xy * 2u);
  let untonemapped = params.flags.x > 0.5;
  accumulate_pixel(base, dims, untonemapped);
  accumulate_pixel(base + vec2<i32>(1, 0), dims, untonemapped);
  accumulate_pixel(base + vec2<i32>(0, 1), dims, untonemapped);
  accumulate_pixel(base + vec2<i32>(1, 1), dims, untonemapped);
  workgroupBarrier();

  let c0 = atomicLoad(&local_hist[li]);
  if (c0 > 0u) { atomicAdd(&histogram[li], c0); }
  let c1 = atomicLoad(&local_hist[li + 128u]);
  if (c1 > 0u) { atomicAdd(&histogram[li + 128u], c1); }
}

// Single-workgroup reduction: trims darkest 10% and brightest 5% outliers,
// writes (avg_log2_luma, 1.0, 0.0, 1.0) matching `metered_log` in
// fullscreen.wgsl, and zeros `histogram` in place for the next frame.
@compute @workgroup_size(128, 1, 1)
fn cs_average(@builtin(local_invocation_index) li: u32) {
  let b0 = atomicExchange(&histogram[li], 0u);
  let b1 = atomicExchange(&histogram[li + 128u], 0u);
  avg_bins[li] = b0;
  avg_bins[li + 128u] = b1;
  workgroupBarrier();

  if (li != 0u) { return; }
  var total = 0u;
  for (var i = 0u; i < 256u; i++) {
    total += avg_bins[i];
  }
  if (total == 0u) {
    textureStore(metered_out, vec2<i32>(0, 0), vec4<f32>(MIN_LOG_LUM, 1.0, 0.0, 1.0));
    return;
  }
  let total_f = f32(total);
  let low_cut = total_f * 0.10;
  let high_cut = total_f * 0.95;
  var accum = 0.0;
  var weighted = 0.0;
  var weight_sum = 0.0;
  for (var i = 0u; i < 256u; i++) {
    let c = f32(avg_bins[i]);
    let next = accum + c;
    let lo = max(accum, low_cut);
    let hi = min(next, high_cut);
    let take = max(hi - lo, 0.0);
    if (take > 0.0) {
      let log_lum = MIN_LOG_LUM + (f32(i) / 255.0) * LOG_RANGE;
      weighted += take * log_lum;
      weight_sum += take;
    }
    accum = next;
  }
  let avg_log = select(MIN_LOG_LUM, weighted / max(weight_sum, 1e-6), weight_sum > 0.0);
  textureStore(metered_out, vec2<i32>(0, 0), vec4<f32>(avg_log, 1.0, 0.0, 1.0));
}
