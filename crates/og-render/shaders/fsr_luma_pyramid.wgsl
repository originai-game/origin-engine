// FSR 3.1's luma pyramid (ffx_fsr3upscaler_luma_pyramid.h; MIT,
// third_party/fidelityfx/LICENSE.txt) in two dispatches instead of SPD's
// one: core WebGPU has no device-wide coherence for SPD's last-workgroup
// step (its globallycoherent mip and atomic counter). `cs_reduce` writes
// the farthest depth's half-resolution level (SPD's mip 0, the average of
// each 2x2) and each workgroup's sums of log luma and luma; `cs_finish`
// (one workgroup) averages them over the render and updates the frame
// info as SpdStore's 1x1 level does: the log luma smoothed over time and
// the exposure derived from it. The averages are over the render's pixels
// exactly, where SPD's mip chain weighs a non-power-of-two edge's clamped
// texels twice. Prepended by fsr_common.wgsl.

@group(0) @binding(1) var current_luma_tex: texture_2d<f32>;
@group(0) @binding(2) var farthest_depth_tex: texture_2d<f32>;
@group(0) @binding(3) var farthest_depth_mip1_out: texture_storage_2d<rgba16float, write>;
@group(0) @binding(4) var<storage, read_write> luma_partials: array<vec2<f32>>;
@group(0) @binding(5) var<storage, read_write> frame_info: vec4<f32>;

var<workgroup> partial_log: array<f32, 256>;
var<workgroup> partial_luma: array<f32, 256>;

// Each thread reduces a 2x2 of render pixels to one half-resolution
// texel; a 16x16 workgroup covers 32x32 render pixels.
@compute @workgroup_size(16, 16, 1)
fn cs_reduce(
  @builtin(global_invocation_id) id: vec3<u32>,
  @builtin(local_invocation_index) li: u32,
  @builtin(workgroup_id) wg: vec3<u32>,
  @builtin(num_workgroups) groups: vec3<u32>,
) {
  let half_px = vec2<i32>(id.xy);
  let size = render_size();
  var log_sum = 0.0;
  var luma_sum = 0.0;
  var depth_sum = 0.0;
  for (var k = 0; k < 4; k++) {
    let q = half_px * 2 + vec2<i32>(k & 1, k >> 1u);
    // SpdLoadSourceImage's ClampLoad.
    let c = clamp(q, vec2<i32>(0), size - 1);
    let luma = textureLoad(current_luma_tex, c, 0).r;
    depth_sum += textureLoad(farthest_depth_tex, c, 0).r;
    if (all(q < size)) {
      log_sum += max(EPSILON, log(luma));
      luma_sum += luma;
    }
  }
  if (all(half_px < max(size / 2, vec2<i32>(1)))) {
    textureStore(farthest_depth_mip1_out, half_px, vec4<f32>(depth_sum * 0.25, 0.0, 0.0, 0.0));
  }
  partial_log[li] = log_sum;
  partial_luma[li] = luma_sum;
  workgroupBarrier();
  for (var s = 128u; s > 0u; s = s >> 1u) {
    if (li < s) {
      partial_log[li] += partial_log[li + s];
      partial_luma[li] += partial_luma[li + s];
    }
    workgroupBarrier();
  }
  if (li == 0u) {
    luma_partials[wg.y * groups.x + wg.x] = vec2<f32>(partial_log[0], partial_luma[0]);
  }
}

@compute @workgroup_size(256, 1, 1)
fn cs_finish(@builtin(local_invocation_index) li: u32) {
  let count = u32(fsr.misc.x);
  var log_sum = 0.0;
  var luma_sum = 0.0;
  for (var i = li; i < count; i += 256u) {
    log_sum += luma_partials[i].x;
    luma_sum += luma_partials[i].y;
  }
  partial_log[li] = log_sum;
  partial_luma[li] = luma_sum;
  workgroupBarrier();
  for (var s = 128u; s > 0u; s = s >> 1u) {
    if (li < s) {
      partial_log[li] += partial_log[li + s];
      partial_luma[li] += partial_luma[li + s];
    }
    workgroupBarrier();
  }
  if (li == 0u) {
    let pixels = f32(render_size().x * render_size().y);
    let scene_average_luma = partial_luma[0] / pixels;
    var log_luma = partial_log[0] / pixels;
    // A reset clears the frame info to (-1, 1, 0, 0) (fsr3upscalerDispatch).
    var info = select(frame_info, vec4<f32>(-1.0, 1.0, 0.0, 0.0), frame_index() == 0.0);
    let prev_log_luma = info.y;
    if (prev_log_luma < RESET_AUTO_EXPOSURE_AVERAGE_SMOOTHING) {
      log_luma = prev_log_luma + (log_luma - prev_log_luma) * (1.0 - exp(-delta_time()));
      log_luma = max(0.0, log_luma);
    }
    info.x = compute_auto_exposure_from_lavg(log_luma);
    info.y = log_luma;
    info.z = scene_average_luma;
    frame_info = info;
  }
}
