// Tier::Full subgroup + WG=256 FSR 3.1 luma pyramid (`post/fsr.rs`,
// `Caps::SUBGROUP`): replaces 8 shared-memory tree-reduction barriers with a
// single barrier using `subgroupAdd` and doubles tile area (32x32 render
// pixels per 16x16 workgroup). Prepended by `enable subgroups;`,
// `fullscreen.wgsl`, and `fsr_common.wgsl`.

@group(0) @binding(1) var current_luma_tex: texture_2d<f32>;
@group(0) @binding(2) var farthest_depth_tex: texture_2d<f32>;
@group(0) @binding(3) var farthest_depth_mip1_out: texture_storage_2d<rgba16float, write>;
@group(0) @binding(4) var<storage, read_write> luma_partials: array<vec2<f32>>;
@group(0) @binding(5) var<storage, read_write> frame_info: vec4<f32>;

const LANES: u32 = 256u;
var<workgroup> partial_log: array<f32, 64>;
var<workgroup> partial_luma: array<f32, 64>;

fn sg_reduce_luma(li: u32, sg_size: u32, sg_lane: u32, log_in: f32, luma_in: f32) -> vec2<f32> {
  let sg_log = subgroupAdd(log_in);
  let sg_luma = subgroupAdd(luma_in);
  let sg_id = li / sg_size;
  let num_sgs = (LANES + sg_size - 1u) / sg_size;
  if (sg_lane == 0u) {
    partial_log[sg_id] = sg_log;
    partial_luma[sg_id] = sg_luma;
  }
  workgroupBarrier();
  var out = vec2<f32>(0.0);
  if (sg_id == 0u) {
    var acc_log = 0.0;
    var acc_luma = 0.0;
    for (var base = 0u; base < num_sgs; base += sg_size) {
      let idx = base + sg_lane;
      let v_log = select(0.0, partial_log[idx], idx < num_sgs);
      let v_luma = select(0.0, partial_luma[idx], idx < num_sgs);
      acc_log += subgroupAdd(v_log);
      acc_luma += subgroupAdd(v_luma);
    }
    out = vec2<f32>(acc_log, acc_luma);
  }
  return out;
}

@compute @workgroup_size(LANES, 1, 1)
fn cs_reduce(
  @builtin(local_invocation_index) li: u32,
  @builtin(workgroup_id) wg: vec3<u32>,
  @builtin(num_workgroups) groups: vec3<u32>,
  @builtin(subgroup_size) sg_size: u32,
  @builtin(subgroup_invocation_id) sg_lane: u32,
) {
  let half_px = vec2<i32>(i32(wg.x * 16u + (li & 15u)), i32(wg.y * 16u + (li >> 4u)));
  let size = render_size();
  var log_sum = 0.0;
  var luma_sum = 0.0;
  var depth_sum = 0.0;
  for (var k = 0; k < 4; k++) {
    let q = half_px * 2 + vec2<i32>(k & 1, k >> 1u);
    let c = clamp(q, vec2<i32>(0), size - 1);
    let luma = textureLoad(current_luma_tex, c, 0).r;
    depth_sum += textureLoad(farthest_depth_tex, c, 0).r;
    if (all(q < size)) {
      log_sum += log(max(EPSILON, luma));
      luma_sum += luma;
    }
  }
  if (all(half_px < max(size / 2, vec2<i32>(1)))) {
    textureStore(farthest_depth_mip1_out, half_px, vec4<f32>(depth_sum * 0.25, 0.0, 0.0, 0.0));
  }
  let reduced = sg_reduce_luma(li, sg_size, sg_lane, log_sum, luma_sum);
  if (li == 0u) {
    luma_partials[wg.y * groups.x + wg.x] = reduced;
  }
}

@compute @workgroup_size(LANES, 1, 1)
fn cs_finish(
  @builtin(local_invocation_index) li: u32,
  @builtin(subgroup_size) sg_size: u32,
  @builtin(subgroup_invocation_id) sg_lane: u32,
) {
  let count = u32(fsr.misc.x);
  var log_sum = 0.0;
  var luma_sum = 0.0;
  for (var i = li; i < count; i += LANES) {
    log_sum += luma_partials[i].x;
    luma_sum += luma_partials[i].y;
  }
  let reduced = sg_reduce_luma(li, sg_size, sg_lane, log_sum, luma_sum);
  if (li == 0u) {
    let pixels = max(f32(render_size().x * render_size().y), 1.0);
    let scene_average_luma = reduced.y / pixels;
    var log_luma = reduced.x / pixels;
    var info = select(
      frame_info,
      vec4<f32>(-1.0, RESET_AUTO_EXPOSURE_AVERAGE_SMOOTHING, 0.0, 0.0),
      frame_index() == 0.0,
    );
    let prev_log_luma = info.y;
    if (prev_log_luma < RESET_AUTO_EXPOSURE_AVERAGE_SMOOTHING) {
      log_luma = prev_log_luma + (log_luma - prev_log_luma) * (1.0 - exp(-delta_time()));
    }
    info.x = compute_auto_exposure_from_lavg(log_luma);
    info.y = log_luma;
    info.z = scene_average_luma;
    frame_info = info;
  }
}
