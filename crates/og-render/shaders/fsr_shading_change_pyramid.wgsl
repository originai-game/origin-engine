// FSR 3.1's shading change pyramid (ffx_fsr3upscaler_shading_change_pyramid.h;
// MIT, third_party/fidelityfx/LICENSE.txt): per render pixel the signed
// relative difference between this frame's luma and the previous frame's
// at its reprojected place (the smallest difference between the sorted
// 5-tap sets of each), averaged into three levels from half resolution
// down (SPD's mips 0 to 2, the ones the shading change pass reads; the
// deeper mips SPD also writes are read by nothing). The three levels lie
// within one workgroup's tile, so no device-wide step is needed. The
// levels hold (difference, sign) in rg. Prepended by fsr_common.wgsl.

@group(0) @binding(1) var dilated_motion_tex: texture_2d<f32>;
@group(0) @binding(2) var current_luma_tex: texture_2d<f32>;
@group(0) @binding(3) var previous_luma_tex: texture_2d<f32>;
@group(0) @binding(4) var<storage, read> frame_info: vec4<f32>;
@group(0) @binding(5) var mip0_out: texture_storage_2d<rgba16float, write>;
@group(0) @binding(6) var mip1_out: texture_storage_2d<rgba16float, write>;
@group(0) @binding(7) var mip2_out: texture_storage_2d<rgba16float, write>;

fn exposure() -> f32 { return exposure_of(frame_info); }

const SAMPLE_OFFSETS: array<vec2<i32>, 5> = array<vec2<i32>, 5>(
  vec2<i32>(0, 0), vec2<i32>(-1, 0), vec2<i32>(1, 0), vec2<i32>(0, -1), vec2<i32>(0, 1));

fn compare_swap(s: ptr<function, array<f32, 5>>, i: i32, j: i32) {
  let lo = min((*s)[i], (*s)[j]);
  (*s)[j] = max((*s)[i], (*s)[j]);
  (*s)[i] = lo;
}

fn sort_set(s: ptr<function, array<f32, 5>>) {
  compare_swap(s, 0, 3);
  compare_swap(s, 1, 4);
  compare_swap(s, 0, 2);
  compare_swap(s, 1, 3);
  compare_swap(s, 0, 1);
  compare_swap(s, 2, 4);
  compare_swap(s, 1, 2);
  compare_swap(s, 3, 4);
  compare_swap(s, 2, 3);
}

fn compute_minimum_difference(set0_in: array<f32, 5>, set1_in: array<f32, 5>) -> f32 {
  var s0 = set0_in;
  var s1 = set1_in;
  var min_diff = FP16_MAX - 1.0;
  var a = 0;
  var b = 0;
  sort_set(&s0);
  sort_set(&s1);
  let m = min(s0[SHADING_CHANGE_SET_SIZE - 1], s1[SHADING_CHANGE_SET_SIZE - 1]);
  if (m > FP32_MIN) {
    for (var i = 0; i < SHADING_CHANGE_SET_SIZE; i++) {
      if (min_diff >= FP16_MAX) { break; }
      // AMD's walk can step past the set's end; HLSL reads an
      // out-of-range local array element as undefined, WGSL clamps it.
      let va = s0[min(a, 4)];
      let vb = s1[min(b, 4)];
      var diff = va - vb;
      if (abs(diff) > FP16_MIN) {
        diff = sign(diff) * (1.0 - min_divided_by_max(va, vb));
        min_diff = select(min_diff, diff, abs(diff) < abs(min_diff));
        a += i32(s0[min(a, 4)] < s1[min(b, 4)]);
        b += i32(s0[min(a, 4)] >= s1[min(b, 4)]);
      } else {
        min_diff = FP16_MAX;
      }
    }
  }
  return min_diff * f32(min_diff < FP16_MAX - 1.0);
}

fn compute_diff(uv: vec2<f32>, mv: vec2<f32>) -> f32 {
  var offsets = SAMPLE_OFFSETS;
  var current: array<f32, 5>;
  let uv_j = uv + jitter() / vec2<f32>(render_size());
  let base = vec2<i32>(floor(uv_j * vec2<f32>(render_size())));
  for (var i = 0; i < SHADING_CHANGE_SET_SIZE; i++) {
    let q = clamp_load(base, offsets[i], render_size());
    current[i] = max(textureLoad(current_luma_tex, q, 0).r * exposure(), EPSILON);
  }
  let prev_uv = uv + previous_jitter() / vec2<f32>(previous_render_size()) + mv;
  if (!is_uv_inside(prev_uv)) { return 0.0; }
  var previous: array<f32, 5>;
  let prev_base = vec2<i32>(floor(prev_uv * vec2<f32>(previous_render_size())));
  for (var i = 0; i < SHADING_CHANGE_SET_SIZE; i++) {
    let q = clamp_load(prev_base, offsets[i], previous_render_size());
    previous[i] = max(textureLoad(previous_luma_tex, q, 0).r * delta_pre_exposure() * exposure(), EPSILON);
  }
  return compute_minimum_difference(current, previous);
}

// SpdLoadSourceImage: (difference, sign of it, 1).
fn load_source(p: vec2<i32>) -> vec2<f32> {
  let q = clamp(p, vec2<i32>(0), render_size() - 1);
  let mv = textureLoad(dilated_motion_tex, q, 0).xy;
  let uv = (vec2<f32>(q) + 0.5) / vec2<f32>(render_size());
  let d = compute_diff(uv, mv);
  return vec2<f32>(d, select(0.0, sign(d), d != 0.0));
}

var<workgroup> level1: array<vec2<f32>, 64>;

// Each thread reduces 4x4 render pixels to 2x2 texels of level 0 and one
// of level 1; an 8x8 workgroup covers 32x32 render pixels, and 16 of its
// threads reduce its 8x8 level-1 texels to 4x4 of level 2.
@compute @workgroup_size(8, 8, 1)
fn cs(
  @builtin(global_invocation_id) id: vec3<u32>,
  @builtin(local_invocation_id) local: vec3<u32>,
  @builtin(local_invocation_index) li: u32,
) {
  let t = vec2<i32>(id.xy);
  var sum1 = vec2<f32>(0.0);
  for (var k = 0; k < 4; k++) {
    let m0 = t * 2 + vec2<i32>(k & 1, k >> 1u);
    var sum0 = vec2<f32>(0.0);
    for (var j = 0; j < 4; j++) {
      sum0 += load_source(m0 * 2 + vec2<i32>(j & 1, j >> 1u));
    }
    sum0 *= 0.25;
    textureStore(mip0_out, m0, vec4<f32>(sum0, 0.0, 0.0));
    sum1 += sum0;
  }
  sum1 *= 0.25;
  textureStore(mip1_out, t, vec4<f32>(sum1, 0.0, 0.0));
  level1[li] = sum1;
  workgroupBarrier();
  if (all((local.xy & vec2<u32>(1u)) == vec2<u32>(0u))) {
    let s = level1[li] + level1[li + 1u] + level1[li + 8u] + level1[li + 9u];
    textureStore(mip2_out, t / 2, vec4<f32>(s * 0.25, 0.0, 0.0));
  }
}
