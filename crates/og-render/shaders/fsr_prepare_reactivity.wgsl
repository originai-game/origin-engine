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
// composition mask. The same pass keeps the luma history and its
// instability (ffx_fsr3upscaler_luma_instability.h), which AMD runs as a
// pass of its own reading these masks back bilinearly at the jittered
// place; here it takes the pixel's own, a sub-pixel apart, and saves the
// pass and its reads. Prepended by fullscreen.wgsl and fsr_common.wgsl.

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
@group(0) @binding(13) var luma_history_tex: texture_2d<f32>;
@group(0) @binding(14) var luma_history_out: texture_storage_2d<rgba16float, write>;
@group(0) @binding(15) var luma_instability_out: texture_storage_2d<r32float, write>;

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

struct Disocclusion {
  sum: f32,
  weight_sum: f32,
  potential: bool,
};

// One bilinear tap of ComputeDisocclusions; a tap after one whose previous
// depth lay behind does nothing, as AMD's loop stops.
fn disocclusion_tap(o: ptr<function, Disocclusion>, b: BilinearSamplingData, k: i32, offset: vec2<i32>, current_depth_view: f32) {
  if (!(*o).potential) { return; }
  let q = clamp_load(b.base, offset, render_size());
  if (is_on_screen(q, render_size())) {
    let w = b.weights[k];
    if (w > RECONSTRUCTED_DEPTH_BILINEAR_WEIGHT_THRESHOLD) {
      let prev_nearest_view = view_depth(load_reconstructed_prev_depth(q));
      let depth_difference = current_depth_view - prev_nearest_view;
      (*o).potential = depth_difference > FP32_MIN;
      if ((*o).potential) {
        let half_viewport_width = length(vec2<f32>(render_size()) * 0.5);
        let depth_threshold = max(current_depth_view, prev_nearest_view);
        let ksep = 1.37e-05;
        let required_separation = ksep * half_viewport_width * depth_threshold;
        (*o).sum += saturate(required_separation / depth_difference) * w;
        (*o).weight_sum += w;
      }
    }
  }
}

// ComputeDisocclusions, its four taps written out (naga's guarded loops
// are not unrolled on Metal).
fn compute_disocclusions(uv: vec2<f32>, mv_in: vec2<f32>, current_depth_view: f32) -> f32 {
  let nearest_m = min(current_depth_view, FP16_MAX);
  let threshold = reconstructed_depth_mv_px_threshold(nearest_m);
  let mv = mv_in * f32(get_4k_velocity(mv_in) > threshold);
  let b = get_bilinear_sampling_data(uv + mv, render_size());
  var o = Disocclusion(0.0, 0.0, true);
  disocclusion_tap(&o, b, 0, vec2<i32>(0, 0), current_depth_view);
  disocclusion_tap(&o, b, 1, vec2<i32>(1, 0), current_depth_view);
  disocclusion_tap(&o, b, 2, vec2<i32>(0, 1), current_depth_view);
  disocclusion_tap(&o, b, 3, vec2<i32>(1, 1), current_depth_view);
  if (o.potential & (o.weight_sum > 0.0)) {
    return saturate(1.0 - o.sum / o.weight_sum);
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

fn reactive_at(p: vec2<i32>, x: i32, y: i32) -> f32 {
  return load_reactive_mask(clamp_load(p, vec2<i32>(x, y), render_size()));
}

fn dilate_reactive_masks(p: vec2<i32>) -> f32 {
  return max(max(max(reactive_at(p, -1, -1), reactive_at(p, 0, -1)), max(reactive_at(p, 1, -1), reactive_at(p, -1, 0))),
    max(max(reactive_at(p, 0, 0), reactive_at(p, 1, 0)), max(max(reactive_at(p, -1, 1), reactive_at(p, 0, 1)), reactive_at(p, 1, 1))));
}

struct Ridge {
  pattern: u32,
  dissimilar_min: f32,
  dissimilar_max: f32,
};

// One neighbour (bit `bit`) of ComputeThinFeatureConfidence's pattern:
// similar to the nucleus, or one of the dissimilar luma's extents.
fn ridge_tap(r: ptr<function, Ridge>, v: f32, nucleus: f32, range: f32, bit: u32) {
  if (abs(v - nucleus) / range < 0.9) {
    (*r).pattern |= 1u << bit;
  } else {
    (*r).dissimilar_min = min((*r).dissimilar_min, v);
    (*r).dissimilar_max = max((*r).dissimilar_max, v);
  }
}

fn luma_at(p: vec2<i32>, x: i32, y: i32) -> f32 {
  return textureLoad(current_luma_tex, clamp_load(p, vec2<i32>(x, y), render_size()), 0).r * exposure();
}

// ComputeThinFeatureConfidence over the 3x3 (1 2 3 / 4 0 5 / 6 7 8),
// written out: naga's guarded loops are not unrolled on Metal, and its
// nine samples sat in stack memory.
fn compute_thin_feature_confidence(p: vec2<i32>) -> f32 {
  let s0 = luma_at(p, 0, 0);
  let s1 = luma_at(p, -1, -1);
  let s2 = luma_at(p, 0, -1);
  let s3 = luma_at(p, 1, -1);
  let s4 = luma_at(p, -1, 0);
  let s5 = luma_at(p, 1, 0);
  let s6 = luma_at(p, -1, 1);
  let s7 = luma_at(p, 0, 1);
  let s8 = luma_at(p, 1, 1);
  let luma_min = min(min(min(s0, s1), min(s2, s3)), min(min(s4, s5), min(min(s6, s7), s8)));
  let luma_max = max(max(max(s0, s1), max(s2, s3)), max(max(s4, s5), max(max(s6, s7), s8)));
  // AMD divides by the range unguarded: with every sample equal it is
  // 0 / 0, no sample is similar, and the nucleus is no ridge, so 0.
  let range = luma_max - luma_min;
  if (range <= 0.0) { return 0.0; }
  var r = Ridge(1u, FP32_MAX, 0.0);
  ridge_tap(&r, s1, s0, range, 1u);
  ridge_tap(&r, s2, s0, range, 2u);
  ridge_tap(&r, s3, s0, range, 3u);
  ridge_tap(&r, s4, s0, range, 4u);
  ridge_tap(&r, s5, s0, range, 5u);
  ridge_tap(&r, s6, s0, range, 6u);
  ridge_tap(&r, s7, s0, range, 7u);
  ridge_tap(&r, s8, s0, range, 8u);
  if (!((s0 > r.dissimilar_max) | (s0 < r.dissimilar_min))) { return 0.0; }
  // The four corner patterns that are not thin features.
  let ul = (1u << 1u) | (1u << 2u) | (1u << 4u) | 1u;
  let ur = (1u << 2u) | (1u << 3u) | (1u << 5u) | 1u;
  let ll = (1u << 4u) | (1u << 6u) | (1u << 7u) | 1u;
  let lr = (1u << 5u) | (1u << 7u) | (1u << 8u) | 1u;
  let m = r.pattern;
  if (((m & ul) == ul) | ((m & ur) == ur) | ((m & ll) == ll) | ((m & lr) == lr)) { return 0.0; }
  return 1.0 - luma_min / luma_max;
}

fn update_accumulation(p: vec2<i32>, uv: vec2<f32>, mv: vec2<f32>, disocclusion: f32, shading_change: f32) -> f32 {
  let reprojected = uv + mv;
  var accumulation = 0.0;
  // After a reset nothing has accumulated (AMD clears the texture).
  if (is_uv_inside(reprojected) & (frame_index() > 0.0)) {
    let hw = clamp_uv(reprojected, previous_render_size(), max_render_size());
    accumulation = saturate(sample_exact(accumulation_tex, linear_clamp, hw).r);
  }
  accumulation = mix(accumulation, 0.0, shading_change);
  accumulation = mix(accumulation, min(min_disocclusion_accumulation(), accumulation), disocclusion);
  accumulation *= f32(round(accumulation * 100.0) > 1.0);
  textureStore(accumulation_out, p, vec4<f32>(saturate(accumulation + accumulation_added_per_frame()), 0.0, 0.0, 0.0));
  return accumulation;
}

// One earlier frame of ComputeLumaInstabilityFactor's three (written out:
// naga's guarded loops are not unrolled on Metal): its similarity counts
// where the luma moved the same way from it as from the last frame.
fn similar_frame(max_similarity: f32, current: f32, diff0: f32, earlier: f32) -> f32 {
  if (sign(diff0) == sign(current - earlier)) {
    return max(max_similarity, min_divided_by_max(current, earlier));
  }
  return max_similarity;
}

// The luma instability: each render pixel keeps its luma over the last
// four frames, and where a well-accumulated pixel's luma returns to an
// earlier frame's (flickering rather than changing) it is unstable, so the
// accumulation trusts its history over the clamp. `masks` are the pixel's
// dilated reactive, disocclusion, shading change and accumulation.
fn update_luma_instability(p: vec2<i32>, uv: vec2<f32>, mv: vec2<f32>, masks: vec4<f32>) {
  var instability = 0.0;
  var history = vec4<f32>(0.0);
  let uv_cur_j = uv + jitter() / vec2<f32>(render_size());
  let uv_prev_j = uv + previous_jitter() / vec2<f32>(previous_render_size());
  let reprojected = uv_prev_j + mv;
  if (is_uv_inside(reprojected) & (saturate(masks[ACCUMULATION]) > 0.9)) {
    let current = bilinear_r(current_luma_tex, clamp_uv(uv_cur_j, render_size(), max_render_size())) * exposure();
    let reprojected_hw = clamp_uv(reprojected, previous_render_size(), max_render_size());
    history = textureSampleLevel(luma_history_tex, linear_clamp, reprojected_hw, 0.0) * delta_pre_exposure() * exposure();
    // ComputeLumaInstabilityFactor (the farthest depth it is handed is
    // unused upstream too).
    var unstable = 0.0;
    let diff0 = current - history[0];
    let similarity0 = min_divided_by_max_or(current, history[0], 1.0);
    var max_similarity = similarity0;
    if (similarity0 < 1.0) {
      max_similarity = similar_frame(max_similarity, current, diff0, history.y);
      max_similarity = similar_frame(max_similarity, current, diff0, history.z);
      max_similarity = similar_frame(max_similarity, current, diff0, history.w);
      unstable = f32(max_similarity > similarity0);
    }
    history = vec4<f32>(current, history.x, history.y, history.z) / exposure();
    instability = unstable * f32(history.w != 0.0);
    let velocity_weight = 1.0 - saturate(get_4k_velocity(mv) / 20.0);
    instability *= velocity_weight * (1.0 - saturate(masks[DISOCCLUSION])) * (1.0 - saturate(masks[REACTIVE]))
      * (1.0 - saturate(masks[SHADING_CHANGE]));
  }
  textureStore(luma_history_out, p, history);
  textureStore(luma_instability_out, p, vec4<f32>(instability, 0.0, 0.0, 0.0));
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

  let masks = vec4<f32>(final_reactiveness, disocclusion, shading_change, accumulation);
  textureStore(dilated_reactive_out, p, masks);
  update_luma_instability(p, uv, mv, masks);

  let lock_strength = compute_thin_feature_confidence(p);
  if (lock_strength > 1.0 / 100.0) {
    let hr = compute_hr_pos_from_lr_pos(p);
    if (is_on_screen(hr, upscale_size())) {
      new_locks[u32(hr.y) * u32(upscale_size().x) + u32(hr.x)] = lock_strength;
    }
  }
}
