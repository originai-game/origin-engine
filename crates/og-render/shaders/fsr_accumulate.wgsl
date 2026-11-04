// FSR 3.1's accumulate pass (ffx_fsr3upscaler_accumulate.h, _upsample.h,
// _reproject.h; MIT, third_party/fidelityfx/LICENSE.txt), with HDR input:
// per output pixel the history reprojected with a deringed Lanczos-2
// (AMD's reference kernel) and its lock, the current frame upsampled with
// a Lanczos-2 slice over the 3x3 samples nearest (its jitter walks the
// slices over the frames), the history rectified into the samples' variance
// box unless locks or luma instability vouch for it, and the two blended
// by their weights in a reversibly tonemapped YCoCg. Writes the internal
// history (rgb, lock in a) and the output; the output's alpha (the
// engine's tone-mapped fraction, post.wgsl `tonemapped`) is the current
// frame's, blended with the previous output's along the motion as the
// colour is and clamped to the current samples' range, as taa.wgsl does.
// Prepended by fsr_common.wgsl.

@group(0) @binding(1) var dilated_reactive_tex: texture_2d<f32>;
@group(0) @binding(2) var dilated_motion_tex: texture_2d<f32>;
@group(0) @binding(3) var history_tex: texture_2d<f32>;
@group(0) @binding(4) var farthest_depth_mip1_tex: texture_2d<f32>;
@group(0) @binding(5) var color_tex: texture_2d<f32>;
@group(0) @binding(6) var luma_instability_tex: texture_2d<f32>;
@group(0) @binding(7) var<storage, read> frame_info: vec4<f32>;
@group(0) @binding(8) var<storage, read_write> new_locks: array<f32>;
@group(0) @binding(9) var linear_clamp: sampler;
@group(0) @binding(10) var previous_output_tex: texture_2d<f32>;
@group(0) @binding(11) var history_out: texture_storage_2d<rgba16float, write>;
@group(0) @binding(12) var output: texture_storage_2d<rgba16float, write>;

fn exposure() -> f32 { return exposure_of(frame_info); }

struct Params {
  hr_pos: vec2<i32>,
  hr_uv: vec2<f32>,
  lr_uv_jittered: vec2<f32>,
  lr_uv_hw: vec2<f32>,
  motion: vec2<f32>,
  reprojected_hr_uv: vec2<f32>,
  velocity_4k: f32,
  disocclusion: f32,
  reactive: f32,
  shading_change: f32,
  accumulation: f32,
  luma_instability: f32,
  farthest_depth_m: f32,
  is_existing_sample: bool,
  is_new_sample: bool,
};

struct Box {
  center: vec3<f32>,
  vec: vec3<f32>,
  aabb_min: vec3<f32>,
  aabb_max: vec3<f32>,
  center_weight: f32,
};

struct Data {
  clipping_box: Box,
  upsampled_color: vec3<f32>,
  upsampled_weight: f32,
  history_color: vec3<f32>,
  history_weight: f32,
  lock: f32,
  lock_contribution: f32,
};

// RectificationBoxAddSample, on the pass data's box (a pointer to the
// whole data: naga's SPIR-V writer cannot pass a pointer to a member of a
// pointed-to struct).
fn box_add(initial: bool, d: ptr<function, Data>, c: vec3<f32>, w: f32) {
  if (initial) {
    (*d).clipping_box.aabb_min = c;
    (*d).clipping_box.aabb_max = c;
    (*d).clipping_box.center = c * w;
    (*d).clipping_box.vec = c * c * w;
    (*d).clipping_box.center_weight = w;
  } else {
    (*d).clipping_box.aabb_min = min((*d).clipping_box.aabb_min, c);
    (*d).clipping_box.aabb_max = max((*d).clipping_box.aabb_max, c);
    (*d).clipping_box.center += c * w;
    (*d).clipping_box.vec += c * c * w;
    (*d).clipping_box.center_weight += w;
  }
}

fn box_variance(d: ptr<function, Data>) {
  var b = (*d).clipping_box;
  let w = select(1.0, b.center_weight, abs(b.center_weight) > FP32_MIN);
  b.center_weight = w;
  b.center /= w;
  b.vec /= w;
  b.vec = sqrt(abs(b.vec - b.center * b.center));
  (*d).clipping_box = b;
}

fn load_history(p: vec2<i32>) -> vec4<f32> {
  return textureLoad(history_tex, p, 0);
}

// ClampCoord of ffx_fsr3upscaler_sample.h.
fn clamp_coord(p: vec2<i32>, offset: vec2<i32>, size: vec2<i32>) -> vec2<i32> {
  return clamp(p + offset, vec2<i32>(1), size - 2);
}

fn lanczos2_4(c0: vec4<f32>, c1: vec4<f32>, c2: vec4<f32>, c3: vec4<f32>, t: f32) -> vec4<f32> {
  let w0 = lanczos2(-1.0 - t);
  let w1 = lanczos2(-0.0 - t);
  let w2 = lanczos2(1.0 - t);
  let w3 = lanczos2(2.0 - t);
  return (w0 * c0 + w1 * c1 + w2 * c2 + w3 * c3) / (w0 + w1 + w2 + w3);
}

// HistorySample: DeclareCustomTextureSample with the reference Lanczos2
// over FetchHistorySamples' 4x4, deringed to the middle 2x2.
fn history_sample(uv: vec2<f32>, size: vec2<i32>) -> vec4<f32> {
  var px = uv * vec2<f32>(size) - 0.5;
  let f = fract(px);
  px = clamp(px, vec2<f32>(0.0), vec2<f32>(size - 1));
  let ip = vec2<i32>(floor(px));
  var rows: array<vec4<f32>, 4>;
  var c11: vec4<f32>;
  var c21: vec4<f32>;
  var c12: vec4<f32>;
  var c22: vec4<f32>;
  for (var y = 0; y < 4; y++) {
    let c0 = load_history(clamp_coord(ip, vec2<i32>(-1, y - 1), size));
    let c1 = load_history(clamp_coord(ip, vec2<i32>(0, y - 1), size));
    let c2 = load_history(clamp_coord(ip, vec2<i32>(1, y - 1), size));
    let c3 = load_history(clamp_coord(ip, vec2<i32>(2, y - 1), size));
    if (y == 1) { c11 = c1; c21 = c2; }
    if (y == 2) { c12 = c1; c22 = c2; }
    rows[y] = lanczos2_4(c0, c1, c2, c3, f.x);
  }
  let c = lanczos2_4(rows[0], rows[1], rows[2], rows[3], f.y);
  let lo = min(min(c11, c21), min(c12, c22));
  let hi = max(max(c11, c21), max(c12, c22));
  return clamp(c, lo, hi);
}

fn prepared_color(p: vec2<i32>) -> vec3<f32> {
  let rgb = max(vec3<f32>(0.0), textureLoad(color_tex, p, 0).rgb) * exposure();
  return rgb_to_ycocg(rgb);
}

fn init_params(hr: vec2<i32>) -> Params {
  var p: Params;
  p.hr_pos = hr;
  p.hr_uv = (vec2<f32>(hr) + 0.5) / vec2<f32>(upscale_size());
  p.lr_uv_jittered = p.hr_uv + jitter() / vec2<f32>(render_size());
  p.lr_uv_hw = clamp_uv(p.lr_uv_jittered, render_size(), max_render_size());
  // GetMotionVector with low-resolution motion vectors.
  p.motion = textureLoad(dilated_motion_tex, vec2<i32>(p.hr_uv * vec2<f32>(render_size())), 0).xy;
  p.velocity_4k = get_4k_velocity(p.motion);
  p.reprojected_hr_uv = p.hr_uv + p.motion;
  p.is_existing_sample = is_uv_inside(p.reprojected_hr_uv);
  p.luma_instability = bilinear_r(luma_instability_tex, clamp_uv(p.hr_uv, render_size(), max_render_size()));
  let farthest_uv = clamp_uv(p.lr_uv_jittered, render_size() / 2, vec2<i32>(textureDimensions(farthest_depth_mip1_tex)));
  p.farthest_depth_m = textureSampleLevel(farthest_depth_mip1_tex, linear_clamp, farthest_uv, 0.0).r;
  p.is_new_sample = !p.is_existing_sample | (frame_index() == 0.0);
  let masks = textureSampleLevel(dilated_reactive_tex, linear_clamp, p.lr_uv_hw, 0.0);
  p.reactive = saturate(masks[REACTIVE]);
  p.disocclusion = saturate(masks[DISOCCLUSION]);
  p.shading_change = saturate(masks[SHADING_CHANGE]);
  p.accumulation = saturate(masks[ACCUMULATION]);
  p.accumulation *= f32(round(p.accumulation * 100.0) > 1.0);
  return p;
}

fn reproject_history_color(p: Params, d: ptr<function, Data>) {
  let h = history_sample(p.reprojected_hr_uv, previous_upscale_size());
  (*d).history_color = rgb_to_ycocg(h.rgb * delta_pre_exposure() * exposure());
  (*d).lock = h.w;
}

fn update_lock_status(p: Params, d: ptr<function, Data>) {
  (*d).lock *= f32(!p.is_new_sample);
  let decrease = max(saturate(p.shading_change), max(p.reactive, p.disocclusion));
  (*d).lock = max(0.0, (*d).lock - decrease * LOCK_MAX);
  (*d).lock_contribution = saturate(saturate((*d).lock - LOCK_THRESHOLD) * (LOCK_MAX - LOCK_THRESHOLD));
  let i = u32(p.hr_pos.y) * u32(upscale_size().x) + u32(p.hr_pos.x);
  let new_lock = new_locks[i] * (1.0 - max(p.shading_change * 0.0, p.reactive));
  (*d).lock = max(0.0, min((*d).lock + new_lock, LOCK_MAX));
  let lifetime_decrease = (0.1 / jitter_sequence_length()) * (1.0 - decrease);
  (*d).lock = max(0.0, (*d).lock - lifetime_decrease);
  // Kill a lock whose place leaves the screen next frame.
  (*d).lock *= f32(is_uv_inside(p.hr_uv - p.motion));
}

fn compute_base_accumulation_weight(p: Params, d: ptr<function, Data>) {
  var base = p.accumulation;
  base = min(base, mix(base, 0.15, saturate(max(0.0, (p.velocity_4k * velocity_factor()) / 0.5))));
  (*d).history_weight = base;
}

fn compute_upsampled_color_and_weight(p: Params, d: ptr<function, Data>) {
  let dst = vec2<f32>(p.hr_pos) + 0.5;
  let src_output = dst * downscale_factor();
  let src_input = vec2<i32>(floor(src_output));
  let src_unjittered = vec2<f32>(src_input) + 0.5 - jitter();
  let base_offset = src_unjittered - src_output;
  let flip_col = src_unjittered.x > src_output.x;
  let flip_row = src_unjittered.y > src_output.y;
  let offset_tl = vec2<i32>(select(-1, -2, flip_col), select(-1, -2, flip_row));
  let initial = p.accumulation == 0.0;

  var samples: array<vec3<f32>, 9>;
  for (var row = 0; row < 3; row++) {
    for (var col = 0; col < 3; col++) {
      let cr = vec2<i32>(select(col, 3 - col, flip_col), select(row, 3 - row, flip_row));
      let q = clamp_load(src_input + offset_tl + cr, vec2<i32>(0), render_size());
      var s = prepared_color(q);
      if (initial) {
        s = rgb_to_ycocg(tonemap(ycocg_to_rgb(s)));
      }
      samples[row * 3 + col] = s;
    }
  }

  // ComputeMaxKernelWeight.
  let kernel_bias_max = min(1.99, 1.0 + (1.0 / downscale_factor().x - 1.0));
  let kernel_bias_min = max(1.0, (1.0 + kernel_bias_max) * 0.3);
  let kernel_bias_weight = min(1.0 - p.disocclusion * 0.5, min(1.0 - p.shading_change, saturate((*d).history_weight * 5.0)));
  let kernel_bias = mix(kernel_bias_min, kernel_bias_max, kernel_bias_weight);

  for (var row = 0; row < 3; row++) {
    for (var col = 0; col < 3; col++) {
      let cr = vec2<i32>(select(col, 3 - col, flip_col), select(row, 3 - row, flip_row));
      let offset = vec2<f32>(offset_tl) + vec2<f32>(cr);
      let sample_offset = base_offset + offset;
      let src = src_input + offset_tl + cr;
      let on_screen = f32(is_on_screen(src, render_size()));
      let s = samples[row * 3 + col];
      if (!initial) {
        // GetUpsampleLanczosWeight, LANCZOS_TYPE_APPROXIMATE.
        let biased = sample_offset * kernel_bias;
        let w = on_screen * lanczos2_approx_sq(dot(biased, biased));
        (*d).upsampled_color += s * w;
        (*d).upsampled_weight += w;
      }
      let box_weight = exp(-2.3 * dot(sample_offset, sample_offset)) * on_screen;
      box_add((row == 0) & (col == 0), d, s, box_weight);
    }
  }
  box_variance(d);

  (*d).upsampled_weight *= f32((*d).upsampled_weight > EPSILON);
  if ((*d).upsampled_weight > EPSILON) {
    (*d).upsampled_color /= (*d).upsampled_weight;
    (*d).upsampled_weight *= AVERAGE_LANCZOS_WEIGHT_PER_FRAME;
    // Deringing.
    (*d).upsampled_color = clamp((*d).upsampled_color, (*d).clipping_box.aabb_min, (*d).clipping_box.aabb_max);
  }
  if (initial) {
    (*d).upsampled_color = rgb_to_ycocg(inverse_tonemap(ycocg_to_rgb((*d).clipping_box.center)));
    (*d).upsampled_weight = 1.0;
    (*d).history_weight = 0.0;
  }
}

fn rectify_history(p: Params, d: ptr<function, Data>) {
  let velocity_factor_ = saturate(p.velocity_4k / 20.0);
  let distance_factor = saturate(0.75 - p.farthest_depth_m / 20.0);
  let accumulation_factor = 1.0 - p.accumulation;
  let reactive_factor = pow(p.reactive, 1.0 / 2.0);
  let t = max(velocity_factor_, max(distance_factor, max(accumulation_factor, max(reactive_factor, p.shading_change))));
  let scale = mix(3.0, 1.0, t);
  let scaled = (*d).clipping_box.vec * vec3<f32>(1.7, 1.0, 1.0) * scale;
  let clamped_scaled = max(scaled, vec3<f32>(1.193e-7));
  let transformed = ((*d).history_color - (*d).clipping_box.center) / clamped_scaled;
  if (length(transformed) > 1.0) {
    let clamped = normalize(transformed) * scaled + (*d).clipping_box.center;
    let contribution = max(p.luma_instability, (*d).lock_contribution) * p.accumulation * (1.0 - p.disocclusion);
    (*d).history_color = mix(clamped, (*d).history_color, saturate(contribution));
  }
}

// Returns the blend's weight of the current frame.
fn accumulate(d: ptr<function, Data>) -> f32 {
  (*d).history_weight *= f32((*d).history_weight > FP16_MIN);
  (*d).history_weight = max(EPSILON, (*d).history_weight + (*d).upsampled_weight);
  (*d).upsampled_color = rgb_to_ycocg(tonemap(ycocg_to_rgb((*d).upsampled_color)));
  (*d).history_color = rgb_to_ycocg(tonemap(ycocg_to_rgb((*d).history_color)));
  let alpha = saturate((*d).upsampled_weight / (*d).history_weight);
  (*d).history_color = ycocg_to_rgb(mix((*d).history_color, (*d).upsampled_color, alpha));
  (*d).history_color = inverse_tonemap((*d).history_color);
  return alpha;
}

@compute @workgroup_size(8, 8, 1)
fn cs(@builtin(global_invocation_id) id: vec3<u32>) {
  let hr = vec2<i32>(id.xy);
  if (!is_on_screen(hr, upscale_size())) { return; }
  let p = init_params(hr);
  var d: Data;
  d.history_weight = 1.0;
  if (p.is_existing_sample & !p.is_new_sample) {
    reproject_history_color(p, &d);
  }
  update_lock_status(p, &d);
  compute_base_accumulation_weight(p, &d);
  compute_upsampled_color_and_weight(p, &d);
  rectify_history(p, &d);
  let alpha = accumulate(&d);
  let color = max(d.history_color / exposure(), vec3<f32>(0.0));
  textureStore(history_out, hr, vec4<f32>(color, d.lock));

  // The engine's alpha: the current frame's (bilinear at the jittered
  // place), with the previous output's along the motion.
  let size = vec2<f32>(render_size());
  let a_cur = textureSampleLevel(color_tex, linear_clamp, p.lr_uv_jittered, 0.0).a;
  let c = clamp(vec2<i32>(floor(p.lr_uv_jittered * size - 0.5)), vec2<i32>(0), render_size() - 1);
  let c2 = min(c + 1, render_size() - 1);
  let a0 = textureLoad(color_tex, c, 0).a;
  let a1 = textureLoad(color_tex, vec2<i32>(c2.x, c.y), 0).a;
  let a2 = textureLoad(color_tex, vec2<i32>(c.x, c2.y), 0).a;
  let a3 = textureLoad(color_tex, c2, 0).a;
  var a = a_cur;
  if (p.is_existing_sample & !p.is_new_sample) {
    let a_hist = clamp(textureSampleLevel(previous_output_tex, linear_clamp, p.reprojected_hr_uv, 0.0).a,
      min(min(a0, a1), min(a2, a3)), max(max(a0, a1), max(a2, a3)));
    a = mix(a_hist, a_cur, alpha);
  }
  textureStore(output, hr, vec4<f32>(color, a));
  new_locks[u32(hr.y) * u32(upscale_size().x) + u32(hr.x)] = 0.0;
}
