// FSR 3.1's prepare-reactivity pass (ffx_fsr3upscaler_prepare_reactivity.h;
// MIT, third_party/fidelityfx/LICENSE.txt): per render pixel the
// disocclusion (this depth against the reconstructed previous depth), the
// shading change (with the dilated reactive mask), the motion divergence,
// the accumulation (how many frames of history it holds, as a fraction
// stepped a third a frame), and new locks on thin features, scattered to
// the output pixel the sample lands in. The reactive mask is the order
// independent transparency's: one minus its revealage (oit.rs), at most
// `fsr.misc.y` (AMD's guidance: reactive values near 1 are for what has no
// history at all); none without OIT. There is no transparency and
// composition mask. Prepended by fsr_common.wgsl.

@group(0) @binding(1) var dilated_motion_tex: texture_2d<f32>;
@group(0) @binding(2) var dilated_depth_tex: texture_2d<f32>;
@group(0) @binding(3) var<storage, read> reconstructed_depth: array<u32>;
@group(0) @binding(4) var reveal_tex: texture_2d<f32>;
@group(0) @binding(5) var shading_change_tex: texture_2d<f32>;
@group(0) @binding(6) var accumulation_tex: texture_2d<f32>;
@group(0) @binding(7) var current_luma_tex: texture_2d<f32>;
@group(0) @binding(8) var<storage, read> frame_info: vec4<f32>;
@group(0) @binding(9) var linear_clamp: sampler;
@group(0) @binding(10) var dilated_reactive_out: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(11) var accumulation_out: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(12) var<storage, read_write> new_locks: array<f32>;

fn exposure() -> f32 { return exposure_of(frame_info); }

fn load_reconstructed_prev_depth(p: vec2<i32>) -> f32 {
  let v = reconstructed_depth[u32(p.y) * u32(max_render_size().x) + u32(p.x)];
  return select(1.0, bitcast<f32>(~v), v != 0u);
}

// A load as HLSL's: zero outside the texture.
fn load_dilated_depth(p: vec2<i32>) -> f32 {
  if (!is_on_screen(p, max_render_size())) { return 0.0; }
  return textureLoad(dilated_depth_tex, p, 0).r;
}

fn load_dilated_motion(p: vec2<i32>) -> vec2<f32> {
  if (!is_on_screen(p, max_render_size())) { return vec2<f32>(0.0); }
  return textureLoad(dilated_motion_tex, p, 0).xy;
}

fn load_reactive_mask(p: vec2<i32>) -> f32 {
  // Without OIT a 1x1 white placeholder (both revealages 1): no reaction.
  let size = vec2<i32>(textureDimensions(reveal_tex));
  let r = textureLoad(reveal_tex, min(p, size - 1), 0);
  let reveal = saturate(r.r * r.g);
  return min(1.0 - reveal, fsr.misc.y) * reactiveness_scale();
}

fn compute_disocclusions(uv: vec2<f32>, mv_in: vec2<f32>, current_depth_view: f32) -> f32 {
  let nearest_m = min(current_depth_view, FP16_MAX);
  let threshold = reconstructed_depth_mv_px_threshold(nearest_m);
  let mv = mv_in * f32(get_4k_velocity(mv_in) > threshold);
  let reprojected = uv + mv;
  let b = get_bilinear_sampling_data(reprojected, render_size());
  var offsets = BILINEAR_OFFSETS;
  var disocclusion = 0.0;
  var weight_sum = 0.0;
  var potential = true;
  for (var i = 0; i < 4; i++) {
    if (!potential) { break; }
    let q = clamp_load(b.base, offsets[i], render_size());
    if (is_on_screen(q, render_size())) {
      let w = b.weights[i];
      if (w > RECONSTRUCTED_DEPTH_BILINEAR_WEIGHT_THRESHOLD) {
        let prev_nearest_view = view_depth(load_reconstructed_prev_depth(q));
        let depth_difference = current_depth_view - prev_nearest_view;
        potential = potential & (depth_difference > FP32_MIN);
        if (potential) {
          let half_viewport_width = length(vec2<f32>(render_size()) * 0.5);
          let depth_threshold = max(current_depth_view, prev_nearest_view);
          let ksep = 1.37e-05;
          let required_separation = ksep * half_viewport_width * depth_threshold;
          disocclusion += saturate(required_separation / depth_difference) * w;
          weight_sum += w;
        }
      }
    }
  }
  if (potential & (weight_sum > 0.0)) {
    return saturate(1.0 - disocclusion / weight_sum);
  }
  return 0.0;
}

fn compute_motion_divergence(uv: vec2<f32>, mv: vec2<f32>, current_depth: f32) -> f32 {
  let reprojected_px = vec2<i32>((uv + mv) * vec2<f32>(render_size()));
  let reprojected_depth = load_dilated_depth(reprojected_px);
  let reprojected_mv = load_dilated_motion(reprojected_px);
  let reprojected_velocity = get_4k_velocity(reprojected_mv);
  let velocity = get_4k_velocity(mv);
  let nucleus_m = view_depth(reprojected_depth);
  let current_m = view_depth(current_depth);
  let distance_factor = min_divided_by_max(nucleus_m, current_m);
  let velocity_factor_ = saturate(velocity / 10.0);
  // saturate(x / 0) is saturate(inf) = 1 or saturate(NaN) = 0 in HLSL;
  // either way the velocity factor (0) zeroes it.
  if (velocity == 0.0) { return 0.0; }
  return (1.0 - saturate(reprojected_velocity / velocity)) * distance_factor * velocity_factor_;
}

fn dilate_reactive_masks(p: vec2<i32>) -> f32 {
  var m = 0.0;
  for (var y = -1; y <= 1; y++) {
    for (var x = -1; x <= 1; x++) {
      m = max(m, load_reactive_mask(clamp_load(p, vec2<i32>(x, y), render_size())));
    }
  }
  return m;
}

fn compute_thin_feature_confidence(p: vec2<i32>) -> f32 {
  // 1 2 3 / 4 0 5 / 6 7 8
  var offsets = array<vec2<i32>, 9>(
    vec2<i32>(0, 0), vec2<i32>(-1, -1), vec2<i32>(0, -1), vec2<i32>(1, -1), vec2<i32>(-1, 0),
    vec2<i32>(1, 0), vec2<i32>(-1, 1), vec2<i32>(0, 1), vec2<i32>(1, 1));
  var samples: array<f32, 9>;
  var luma_min = FP32_MAX;
  var luma_max = FP32_MIN;
  for (var i = 0; i < 9; i++) {
    let q = clamp_load(p, offsets[i], render_size());
    samples[i] = textureLoad(current_luma_tex, q, 0).r * exposure();
    luma_min = min(luma_min, samples[i]);
    luma_max = max(luma_max, samples[i]);
  }
  // AMD divides by the range unguarded: with every sample equal it is
  // 0 / 0, no sample is similar, and the nucleus is no ridge, so 0.
  let range = luma_max - luma_min;
  if (range <= 0.0) { return 0.0; }
  let threshold = 0.9;
  var dissimilar_min = FP32_MAX;
  var dissimilar_max = 0.0;
  var pattern = 1u;
  for (var i = 1; i < 9; i++) {
    let difference = abs(samples[i] - samples[0]) / range;
    if (difference < threshold) {
      pattern |= 1u << u32(i);
    } else {
      dissimilar_min = min(dissimilar_min, samples[i]);
      dissimilar_max = max(dissimilar_max, samples[i]);
    }
  }
  let ridge = (samples[0] > dissimilar_max) | (samples[0] < dissimilar_min);
  if (!ridge) { return 0.0; }
  var rejection = array<u32, 4>(
    (1u << 1u) | (1u << 2u) | (1u << 4u) | 1u,
    (1u << 2u) | (1u << 3u) | (1u << 5u) | 1u,
    (1u << 4u) | (1u << 6u) | (1u << 7u) | 1u,
    (1u << 5u) | (1u << 7u) | (1u << 8u) | 1u);
  for (var i = 0; i < 4; i++) {
    if ((pattern & rejection[i]) == rejection[i]) { return 0.0; }
  }
  return 1.0 - luma_min / luma_max;
}

fn update_accumulation(p: vec2<i32>, uv: vec2<f32>, mv: vec2<f32>, disocclusion: f32, shading_change: f32) -> f32 {
  let reprojected = uv + mv;
  var accumulation = 0.0;
  // After a reset nothing has accumulated (AMD clears the texture).
  if (is_uv_inside(reprojected) & (frame_index() > 0.0)) {
    let hw = clamp_uv(reprojected, previous_render_size(), max_render_size());
    accumulation = saturate(textureSampleLevel(accumulation_tex, linear_clamp, hw, 0.0).r);
  }
  accumulation = mix(accumulation, 0.0, shading_change);
  accumulation = mix(accumulation, min(min_disocclusion_accumulation(), accumulation), disocclusion);
  accumulation *= f32(round(accumulation * 100.0) > 1.0);
  textureStore(accumulation_out, p, vec4<f32>(saturate(accumulation + accumulation_added_per_frame()), 0.0, 0.0, 0.0));
  return accumulation;
}

@compute @workgroup_size(8, 8, 1)
fn cs(@builtin(global_invocation_id) id: vec3<u32>) {
  let p = vec2<i32>(id.xy);
  if (!is_on_screen(p, render_size())) { return; }
  let uv = (vec2<f32>(p) + 0.5) / vec2<f32>(render_size());
  let mv = textureLoad(dilated_motion_tex, p, 0).xy;
  let dilated_depth = textureLoad(dilated_depth_tex, p, 0).r;

  let disocclusion = compute_disocclusions(uv, mv, view_depth(dilated_depth));
  // The shading change is sampled at the unjittered place (the
  // accumulation pass jitters it back).
  let sc_uv = clamp_uv(uv - jitter() / vec2<f32>(render_size()), shading_change_render_size(), shading_change_max_render_size());
  let sampled_change = saturate(textureSampleLevel(shading_change_tex, linear_clamp, sc_uv, 0.0).r * shading_change_scale());
  let shading_change = max(dilate_reactive_masks(p), sampled_change);
  let motion_divergence = compute_motion_divergence(uv, mv, dilated_depth);
  let final_reactiveness = max(motion_divergence, 0.0);
  let accumulation = update_accumulation(p, uv, mv, disocclusion, shading_change);

  textureStore(dilated_reactive_out, p, vec4<f32>(final_reactiveness, disocclusion, shading_change, accumulation));

  let lock_strength = compute_thin_feature_confidence(p);
  if (lock_strength > 1.0 / 100.0) {
    let hr = compute_hr_pos_from_lr_pos(p);
    if (is_on_screen(hr, upscale_size())) {
      new_locks[u32(hr.y) * u32(upscale_size().x) + u32(hr.x)] = lock_strength;
    }
  }
}
