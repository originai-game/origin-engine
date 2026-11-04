// FSR 3.1's luma instability pass (ffx_fsr3upscaler_luma_instability.h;
// MIT, third_party/fidelityfx/LICENSE.txt): each render pixel keeps its
// luma over the last four frames, and where a well-accumulated pixel's
// luma returns to an earlier frame's (flickering rather than changing) it
// is unstable: the accumulation then trusts its history over the clamp.
// Prepended by fsr_common.wgsl.

@group(0) @binding(1) var dilated_motion_tex: texture_2d<f32>;
@group(0) @binding(2) var dilated_reactive_tex: texture_2d<f32>;
@group(0) @binding(3) var current_luma_tex: texture_2d<f32>;
@group(0) @binding(4) var luma_history_tex: texture_2d<f32>;
@group(0) @binding(5) var<storage, read> frame_info: vec4<f32>;
@group(0) @binding(6) var linear_clamp: sampler;
@group(0) @binding(7) var luma_history_out: texture_storage_2d<rgba16float, write>;
@group(0) @binding(8) var luma_instability_out: texture_storage_2d<r32float, write>;

fn exposure() -> f32 { return exposure_of(frame_info); }

@compute @workgroup_size(8, 8, 1)
fn cs(@builtin(global_invocation_id) id: vec3<u32>) {
  let p = vec2<i32>(id.xy);
  if (!is_on_screen(p, render_size())) { return; }
  var instability = 0.0;
  var history = vec4<f32>(0.0);

  let mv = textureLoad(dilated_motion_tex, p, 0).xy;
  let uv = (vec2<f32>(p) + 0.5) / vec2<f32>(render_size());
  let uv_cur_j = uv + jitter() / vec2<f32>(render_size());
  let uv_prev_j = uv + previous_jitter() / vec2<f32>(previous_render_size());
  let reprojected = uv_prev_j + mv;

  if (is_uv_inside(reprojected)) {
    let reactive_uv = clamp_uv(uv_cur_j, render_size(), max_render_size());
    let masks = textureSampleLevel(dilated_reactive_tex, linear_clamp, reactive_uv, 0.0);
    let reactive = saturate(masks[REACTIVE]);
    let disocclusion = saturate(masks[DISOCCLUSION]);
    let shading_change = saturate(masks[SHADING_CHANGE]);
    let accumulation = saturate(masks[ACCUMULATION]);
    if (accumulation > 0.9) {
      let current = bilinear_r(current_luma_tex, reactive_uv) * exposure();
      let reprojected_hw = clamp_uv(reprojected, previous_render_size(), max_render_size());
      history = textureSampleLevel(luma_history_tex, linear_clamp, reprojected_hw, 0.0) * delta_pre_exposure() * exposure();

      // ComputeLumaInstabilityFactor (the farthest depth it is handed is
      // unused upstream too).
      var unstable = 0.0;
      let diff0 = current - history[0];
      let similarity0 = min_divided_by_max_or(current, history[0], 1.0);
      var max_similarity = similarity0;
      if (similarity0 < 1.0) {
        for (var i = 1; i <= 3; i++) {
          let diff1 = current - history[i];
          let similarity1 = min_divided_by_max(current, history[i]);
          if (sign(diff0) == sign(diff1)) {
            max_similarity = max(max_similarity, similarity1);
          }
        }
        unstable = f32(max_similarity > similarity0);
      }
      history = vec4<f32>(current, history.x, history.y, history.z) / exposure();
      instability = unstable * f32(history.w != 0.0);

      let velocity_weight = 1.0 - saturate(get_4k_velocity(mv) / 20.0);
      instability *= velocity_weight * (1.0 - disocclusion) * (1.0 - reactive) * (1.0 - shading_change);
    }
  }
  textureStore(luma_history_out, p, history);
  textureStore(luma_instability_out, p, vec4<f32>(instability, 0.0, 0.0, 0.0));
}
