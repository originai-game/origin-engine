// Compute LDS Single-Pass Downsample (SPD) Bloom (`post/bloom_spd.rs`):
// replaces up to 11 fragment bloom render passes with a single 4-level LDS
// prefilter+downsample dispatch (+ optional tiny tail dispatch for levels 4..5)
// and a single compute pass for the 9-tap tent upsample chain. Prepended by
// fullscreen.wgsl (for `exposure_from`, `luma`) and bloom_chain.wgsl.

struct Post {
  exposure: vec4<f32>,
  bloom: vec4<f32>,
  mode: vec4<f32>,
  grade: vec4<f32>,
  vignette: vec4<f32>,
  grain: vec4<f32>,
  flash: vec4<f32>,
  extra: vec4<f32>,
  unreal: vec4<f32>,
  hdr: vec4<f32>,
  flare: vec4<f32>,
};

@group(0) @binding(0) var<uniform> post: Post;
@group(0) @binding(1) var linear_sampler: sampler;
@group(0) @binding(2) var source: texture_2d<f32>;
@group(0) @binding(3) var aux_tex: texture_2d<f32>;     // luminance (prefilter) or lower mip (upsample)
@group(0) @binding(4) var glow_tex: texture_2d<f32>;    // 2D glow (prefilter)
@group(0) @binding(5) var level0: texture_storage_2d<rgba16float, write>;
@group(0) @binding(6) var level1: texture_storage_2d<rgba16float, write>;
@group(0) @binding(7) var level2: texture_storage_2d<rgba16float, write>;
@group(0) @binding(8) var level3: texture_storage_2d<rgba16float, write>;

var<workgroup> spd_tile: array<vec3<f32>, 64>;

fn bloom_source_size() -> vec2<f32> {
  return vec2<f32>(textureDimensions(source));
}

fn bloom_tap(uv: vec2<f32>) -> vec3<f32> {
  let s = textureSampleLevel(source, linear_sampler, uv, 0.0);
  if (post.extra.y > 0.5) { return s.rgb * clamp(s.a, 0.0, 1.0); }
  return s.rgb;
}

fn store_l0(at: vec2<u32>, c: vec3<f32>) {
  if (all(at < textureDimensions(level0))) { textureStore(level0, at, vec4<f32>(c, 1.0)); }
}
fn store_l1(at: vec2<u32>, c: vec3<f32>) {
  if (all(at < textureDimensions(level1))) { textureStore(level1, at, vec4<f32>(c, 1.0)); }
}
fn store_l2(at: vec2<u32>, c: vec3<f32>) {
  if (all(at < textureDimensions(level2))) { textureStore(level2, at, vec4<f32>(c, 1.0)); }
}
fn store_l3(at: vec2<u32>, c: vec3<f32>) {
  if (all(at < textureDimensions(level3))) { textureStore(level3, at, vec4<f32>(c, 1.0)); }
}

// Prefilter (13-tap Karis average + soft threshold + optional 2D glow) into
// level 0 (half res), then reduce levels 1, 2, 3 in workgroup memory in one
// dispatch (4 storage textures, WebGPU Compat safe).
@compute @workgroup_size(8, 8, 1)
fn cs_bloom_spd(
  @builtin(workgroup_id) wg: vec3<u32>,
  @builtin(local_invocation_id) lid: vec3<u32>,
) {
  let px = wg.xy * 8u + lid.xy;
  let l0_size = textureDimensions(level0);
  let uv = (vec2<f32>(min(px, l0_size - 1u)) + 0.5) / vec2<f32>(l0_size);
  let exp_gain = exposure_from(post.exposure, post.bloom.w, aux_tex);
  var out = vec3<f32>(0.0);
  if (post.mode.z > 0.5) {
    let c = downsample13_karis(uv, exp_gain) * exp_gain;
    let brightness = max(c.r, max(c.g, c.b));
    let knee = max(post.bloom.y, 1e-4);
    var soft = clamp(brightness - post.bloom.x + knee, 0.0, 2.0 * knee);
    soft = soft * soft / (4.0 * knee);
    let contribution = max(soft, brightness - post.bloom.x) / max(brightness, 1e-4);
    out = c * contribution;
  }
  if (post.mode.w > 0.0) {
    out += textureSampleLevel(glow_tex, linear_sampler, uv, 0.0).rgb * exp_gain * post.mode.w;
  }
  store_l0(px, out);

  let idx = lid.y * 8u + lid.x;
  spd_tile[idx] = out;
  workgroupBarrier();

  var m1 = vec3<f32>(0.0);
  if (all(lid.xy % 2u == vec2<u32>(0u))) {
    m1 = 0.25 * (
      spd_tile[idx] + spd_tile[idx + 1u] +
      spd_tile[idx + 8u] + spd_tile[idx + 9u]);
    store_l1(wg.xy * 4u + lid.xy / 2u, m1);
  }
  workgroupBarrier();
  if (all(lid.xy % 2u == vec2<u32>(0u))) {
    spd_tile[idx] = m1;
  }
  workgroupBarrier();

  var m2 = vec3<f32>(0.0);
  if (all(lid.xy % 4u == vec2<u32>(0u))) {
    m2 = 0.25 * (
      spd_tile[idx] + spd_tile[idx + 2u] +
      spd_tile[idx + 16u] + spd_tile[idx + 18u]);
    store_l2(wg.xy * 2u + lid.xy / 4u, m2);
  }
  workgroupBarrier();
  if (all(lid.xy % 4u == vec2<u32>(0u))) {
    spd_tile[idx] = m2;
  }
  workgroupBarrier();

  if (lid.x == 0u && lid.y == 0u) {
    let m3 = 0.25 * (
      spd_tile[0] + spd_tile[4] +
      spd_tile[32] + spd_tile[36]);
    store_l3(wg.xy, m3);
  }
}

// Tail downsample for mip levels 4 and 5 (when bloom_levels > 4): reads level 3
// via 13-tap Jimenez filter into level0 (= mip 4) and reduces 2x2 in LDS into
// level1 (= mip 5).
@compute @workgroup_size(8, 8, 1)
fn cs_bloom_spd_tail(
  @builtin(workgroup_id) wg: vec3<u32>,
  @builtin(local_invocation_id) lid: vec3<u32>,
) {
  let px = wg.xy * 8u + lid.xy;
  let dst_size = textureDimensions(level0);
  let uv = (vec2<f32>(min(px, dst_size - 1u)) + 0.5) / vec2<f32>(dst_size);
  // 13-tap Jimenez downsample without untonemapped alpha masking (already applied at mip 0).
  let t = 1.0 / vec2<f32>(textureDimensions(source));
  let s = textureSampleLevel(source, linear_sampler, uv, 0.0).rgb;
  let a = textureSampleLevel(source, linear_sampler, uv + t * vec2<f32>(-2.0, -2.0), 0.0).rgb;
  let b = textureSampleLevel(source, linear_sampler, uv + t * vec2<f32>(0.0, -2.0), 0.0).rgb;
  let c = textureSampleLevel(source, linear_sampler, uv + t * vec2<f32>(2.0, -2.0), 0.0).rgb;
  let d = textureSampleLevel(source, linear_sampler, uv + t * vec2<f32>(-2.0, 0.0), 0.0).rgb;
  let f = textureSampleLevel(source, linear_sampler, uv + t * vec2<f32>(2.0, 0.0), 0.0).rgb;
  let g = textureSampleLevel(source, linear_sampler, uv + t * vec2<f32>(-2.0, 2.0), 0.0).rgb;
  let h = textureSampleLevel(source, linear_sampler, uv + t * vec2<f32>(0.0, 2.0), 0.0).rgb;
  let i = textureSampleLevel(source, linear_sampler, uv + t * vec2<f32>(2.0, 2.0), 0.0).rgb;
  let j = textureSampleLevel(source, linear_sampler, uv + t * vec2<f32>(-1.0, -1.0), 0.0).rgb;
  let k = textureSampleLevel(source, linear_sampler, uv + t * vec2<f32>(1.0, -1.0), 0.0).rgb;
  let l = textureSampleLevel(source, linear_sampler, uv + t * vec2<f32>(-1.0, 1.0), 0.0).rgb;
  let m = textureSampleLevel(source, linear_sampler, uv + t * vec2<f32>(1.0, 1.0), 0.0).rgb;
  let m4 = s * 0.125 + (a + c + g + i) * 0.03125 + (b + d + f + h) * 0.0625 + (j + k + l + m) * 0.125;
  store_l0(px, m4);

  let idx = lid.y * 8u + lid.x;
  spd_tile[idx] = m4;
  workgroupBarrier();
  if (all(lid.xy % 2u == vec2<u32>(0u))) {
    let m5 = 0.25 * (
      spd_tile[idx] + spd_tile[idx + 1u] +
      spd_tile[idx + 8u] + spd_tile[idx + 9u]);
    store_l1(wg.xy * 4u + lid.xy / 2u, m5);
  }
}

// 9-tap tent upsample of `aux_tex` (lower mip) added to `source` (current down
// mip) and written to `level0` (current up mip).
@compute @workgroup_size(8, 8, 1)
fn cs_bloom_up(@builtin(global_invocation_id) gid: vec3<u32>) {
  let dst_size = textureDimensions(level0);
  if (gid.x >= dst_size.x || gid.y >= dst_size.y) { return; }
  let uv = (vec2<f32>(gid.xy) + 0.5) / vec2<f32>(dst_size);
  let t = post.mode.y / vec2<f32>(textureDimensions(aux_tex));
  var tent = textureSampleLevel(aux_tex, linear_sampler, uv, 0.0).rgb * 4.0;
  tent += textureSampleLevel(aux_tex, linear_sampler, uv + t * vec2<f32>(-1.0, 0.0), 0.0).rgb * 2.0;
  tent += textureSampleLevel(aux_tex, linear_sampler, uv + t * vec2<f32>(1.0, 0.0), 0.0).rgb * 2.0;
  tent += textureSampleLevel(aux_tex, linear_sampler, uv + t * vec2<f32>(0.0, -1.0), 0.0).rgb * 2.0;
  tent += textureSampleLevel(aux_tex, linear_sampler, uv + t * vec2<f32>(0.0, 1.0), 0.0).rgb * 2.0;
  tent += textureSampleLevel(aux_tex, linear_sampler, uv + t * vec2<f32>(-1.0, -1.0), 0.0).rgb;
  tent += textureSampleLevel(aux_tex, linear_sampler, uv + t * vec2<f32>(1.0, -1.0), 0.0).rgb;
  tent += textureSampleLevel(aux_tex, linear_sampler, uv + t * vec2<f32>(-1.0, 1.0), 0.0).rgb;
  tent += textureSampleLevel(aux_tex, linear_sampler, uv + t * vec2<f32>(1.0, 1.0), 0.0).rgb;
  let base = textureLoad(source, vec2<i32>(gid.xy), 0).rgb;
  textureStore(level0, gid.xy, vec4<f32>(base + tent * (1.0 / 16.0), 1.0));
}
