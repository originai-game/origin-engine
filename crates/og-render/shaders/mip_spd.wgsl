// Compute LDS Single-Pass Downsample (SPD) prefiltered mip generator
// (`gpu/mips.rs`): collapses up to 4 mip levels per dispatch into a single
// ComputePass using a 13-tap energy-preserving prefilter kernel for mip +1
// and workgroup LDS 2x2 reductions for mips +2, +3, +4 (WebGPU Compat safe:
// 8x8 workgroup, 4 write-only rgba16float storage textures).

@group(0) @binding(0) var src_tex: texture_2d<f32>;
@group(0) @binding(1) var linear_sampler: sampler;
@group(0) @binding(2) var dst0: texture_storage_2d<rgba16float, write>;
@group(0) @binding(3) var dst1: texture_storage_2d<rgba16float, write>;
@group(0) @binding(4) var dst2: texture_storage_2d<rgba16float, write>;
@group(0) @binding(5) var dst3: texture_storage_2d<rgba16float, write>;
// x: number of mip levels written by this dispatch (1..=4).
@group(0) @binding(6) var<uniform> params: vec4<u32>;

var<workgroup> spd_tile: array<vec4<f32>, 64>;

fn mip_tap(uv: vec2<f32>) -> vec4<f32> {
  return textureSampleLevel(src_tex, linear_sampler, uv, 0.0);
}

// 13-tap Jimenez energy-preserving downsample: prevents rough-glass refraction
// shimmering and fireflies compared to a 2x2 box filter.
fn downsample13_rgba(uv: vec2<f32>) -> vec4<f32> {
  let t = 1.0 / vec2<f32>(textureDimensions(src_tex));
  let e = mip_tap(uv);
  let a = mip_tap(uv + t * vec2<f32>(-2.0, -2.0));
  let b = mip_tap(uv + t * vec2<f32>(0.0, -2.0));
  let c = mip_tap(uv + t * vec2<f32>(2.0, -2.0));
  let d = mip_tap(uv + t * vec2<f32>(-2.0, 0.0));
  let f = mip_tap(uv + t * vec2<f32>(2.0, 0.0));
  let g = mip_tap(uv + t * vec2<f32>(-2.0, 2.0));
  let h = mip_tap(uv + t * vec2<f32>(0.0, 2.0));
  let i = mip_tap(uv + t * vec2<f32>(2.0, 2.0));
  let j = mip_tap(uv + t * vec2<f32>(-1.0, -1.0));
  let k = mip_tap(uv + t * vec2<f32>(1.0, -1.0));
  let l = mip_tap(uv + t * vec2<f32>(-1.0, 1.0));
  let m = mip_tap(uv + t * vec2<f32>(1.0, 1.0));
  return e * 0.125
    + (a + c + g + i) * 0.03125
    + (b + d + f + h) * 0.0625
    + (j + k + l + m) * 0.125;
}

@compute @workgroup_size(8, 8, 1)
fn cs_mip_spd(
  @builtin(workgroup_id) wg: vec3<u32>,
  @builtin(local_invocation_id) lid: vec3<u32>,
) {
  let levels = params.x;
  let px = wg.xy * 8u + lid.xy;
  let d0_size = textureDimensions(dst0);
  let uv = (vec2<f32>(min(px, d0_size - 1u)) + 0.5) / vec2<f32>(d0_size);
  let m0 = downsample13_rgba(uv);
  if (all(px < d0_size)) {
    textureStore(dst0, px, m0);
  }
  if (levels <= 1u) {
    return;
  }

  let idx = lid.y * 8u + lid.x;
  spd_tile[idx] = m0;
  workgroupBarrier();

  var m1 = vec4<f32>(0.0);
  if (all(lid.xy % 2u == vec2<u32>(0u))) {
    m1 = 0.25 * (
      spd_tile[idx] + spd_tile[idx + 1u] +
      spd_tile[idx + 8u] + spd_tile[idx + 9u]);
    let at1 = wg.xy * 4u + lid.xy / 2u;
    if (all(at1 < textureDimensions(dst1))) {
      textureStore(dst1, at1, m1);
    }
  }
  if (levels <= 2u) {
    return;
  }
  workgroupBarrier();
  if (all(lid.xy % 2u == vec2<u32>(0u))) {
    spd_tile[idx] = m1;
  }
  workgroupBarrier();

  var m2 = vec4<f32>(0.0);
  if (all(lid.xy % 4u == vec2<u32>(0u))) {
    m2 = 0.25 * (
      spd_tile[idx] + spd_tile[idx + 2u] +
      spd_tile[idx + 16u] + spd_tile[idx + 18u]);
    let at2 = wg.xy * 2u + lid.xy / 4u;
    if (all(at2 < textureDimensions(dst2))) {
      textureStore(dst2, at2, m2);
    }
  }
  if (levels <= 3u) {
    return;
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
    if (all(wg.xy < textureDimensions(dst3))) {
      textureStore(dst3, wg.xy, m3);
    }
  }
}
