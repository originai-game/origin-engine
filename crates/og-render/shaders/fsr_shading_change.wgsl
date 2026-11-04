// FSR 3.1's shading change pass (ffx_fsr3upscaler_shading_change.h; MIT,
// third_party/fidelityfx/LICENSE.txt): at half resolution, the largest of
// |difference x sign| over the pyramid's first three levels, read
// bilinearly at the jittered place. Prepended by fsr_common.wgsl.

@group(0) @binding(1) var spd_mips: texture_2d<f32>;
@group(0) @binding(2) var linear_clamp: sampler;
@group(0) @binding(3) var shading_change_out: texture_storage_2d<rgba8unorm, write>;

const MIP_LEVELS_TO_USE: i32 = 3;

@compute @workgroup_size(8, 8, 1)
fn cs(@builtin(global_invocation_id) id: vec3<u32>) {
  let p = vec2<i32>(id.xy);
  let size = shading_change_render_size();
  if (!is_on_screen(p, size)) { return; }
  let uv = (vec2<f32>(p) + 0.5) / vec2<f32>(size);
  let uv_j = uv + jitter() / vec2<f32>(render_size());
  let mip_uv = clamp_uv(uv_j, size, vec2<i32>(textureDimensions(spd_mips, 0)));
  // fScale = 1 + iShadingChangeMipStart / s_MipLevelsToUse (integer: 1).
  var change = 0.0;
  for (var m = SHADING_CHANGE_MIP_START; m < MIP_LEVELS_TO_USE; m++) {
    let s = textureSampleLevel(spd_mips, linear_clamp, mip_uv, f32(m)).xy;
    let v = abs(s.x * s.y);
    if (v > 0.0) {
      change = max(change, v);
    }
  }
  textureStore(shading_change_out, p, vec4<f32>(saturate(change), 0.0, 0.0, 0.0));
}
