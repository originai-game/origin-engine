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

// The first element dropped and the last repeated: the set's walk one
// step on, its head always at index 0 (AMD's walk indexes the sets by
// two counters; a dynamic index into a private array spills it to memory
// on Metal, so the walk moves the arrays instead, every index constant).
// A counter past the end reads the last element, as WGSL's clamped index
// did.
fn advance(s: array<f32, 5>) -> array<f32, 5> {
  return array<f32, 5>(s[1], s[2], s[3], s[4], s[4]);
}

// One step of ComputeMinimumDifference's walk (its loop body, written out
// five times by the caller); a step after one that found equal values
// does nothing, as AMD's loop breaks.
fn walk_step(s0: ptr<function, array<f32, 5>>, s1: ptr<function, array<f32, 5>>, min_diff: ptr<function, f32>) {
  if (*min_diff >= FP16_MAX) { return; }
  let va = (*s0)[0];
  let vb = (*s1)[0];
  var diff = va - vb;
  if (abs(diff) > FP16_MIN) {
    diff = sign(diff) * (1.0 - min_divided_by_max(va, vb));
    *min_diff = select(*min_diff, diff, abs(diff) < abs(*min_diff));
    // AMD advances a, then compares the new a with b.
    if ((*s0)[0] < (*s1)[0]) { *s0 = advance(*s0); }
    if ((*s0)[0] >= (*s1)[0]) { *s1 = advance(*s1); }
  } else {
    *min_diff = FP16_MAX;
  }
}

fn compute_minimum_difference(set0_in: array<f32, 5>, set1_in: array<f32, 5>) -> f32 {
  var s0 = set0_in;
  var s1 = set1_in;
  var min_diff = FP16_MAX - 1.0;
  sort_set(&s0);
  sort_set(&s1);
  let m = min(s0[SHADING_CHANGE_SET_SIZE - 1], s1[SHADING_CHANGE_SET_SIZE - 1]);
  if (m > FP32_MIN) {
    walk_step(&s0, &s1, &min_diff);
    walk_step(&s0, &s1, &min_diff);
    walk_step(&s0, &s1, &min_diff);
    walk_step(&s0, &s1, &min_diff);
    walk_step(&s0, &s1, &min_diff);
  }
  return min_diff * f32(min_diff < FP16_MAX - 1.0);
}

// The 5-tap set (centre, left, right, up, down) around `base` of a luma
// texture, times `scale`, each at least EPSILON. Written out: naga's
// guarded loops are not unrolled on Metal, and the set would sit in
// stack memory.
fn luma_set(t: texture_2d<f32>, base: vec2<i32>, size: vec2<i32>, scale: f32) -> array<f32, 5> {
  return array<f32, 5>(
    max(textureLoad(t, clamp_load(base, vec2<i32>(0, 0), size), 0).r * scale, EPSILON),
    max(textureLoad(t, clamp_load(base, vec2<i32>(-1, 0), size), 0).r * scale, EPSILON),
    max(textureLoad(t, clamp_load(base, vec2<i32>(1, 0), size), 0).r * scale, EPSILON),
    max(textureLoad(t, clamp_load(base, vec2<i32>(0, -1), size), 0).r * scale, EPSILON),
    max(textureLoad(t, clamp_load(base, vec2<i32>(0, 1), size), 0).r * scale, EPSILON));
}

fn compute_diff(uv: vec2<f32>, mv: vec2<f32>) -> f32 {
  let uv_j = uv + jitter() / vec2<f32>(render_size());
  let base = vec2<i32>(floor(uv_j * vec2<f32>(render_size())));
  let current = luma_set(current_luma_tex, base, render_size(), exposure());
  let prev_uv = uv + previous_jitter() / vec2<f32>(previous_render_size()) + mv;
  if (!is_uv_inside(prev_uv)) { return 0.0; }
  let prev_base = vec2<i32>(floor(prev_uv * vec2<f32>(previous_render_size())));
  let previous = luma_set(previous_luma_tex, prev_base, previous_render_size(), delta_pre_exposure() * exposure());
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

var<workgroup> level0: array<vec2<f32>, 256>;
var<workgroup> level1: array<vec2<f32>, 64>;

// A 16x16 workgroup covers 32x32 render pixels: each thread reduces its
// 2x2 render pixels to one level-0 texel, a quarter of the threads reduce
// 2x2 of those to level 1, and a sixteenth 2x2 of those to level 2. (One
// thread per 4x4 render pixels, 16 differences each in turn, left the GPU
// waiting on too few threads.)
@compute @workgroup_size(16, 16, 1)
fn cs(
  @builtin(workgroup_id) wg: vec3<u32>,
  @builtin(local_invocation_id) local: vec3<u32>,
  @builtin(local_invocation_index) li: u32,
) {
  let m0 = vec2<i32>(wg.xy * 16u + local.xy);
  let s0 = (load_source(m0 * 2) + load_source(m0 * 2 + vec2<i32>(1, 0))
    + load_source(m0 * 2 + vec2<i32>(0, 1)) + load_source(m0 * 2 + vec2<i32>(1, 1))) * 0.25;
  textureStore(mip0_out, m0, vec4<f32>(s0, 0.0, 0.0));
  level0[li] = s0;
  workgroupBarrier();
  let l1 = local.xy / 2u;
  if (all((local.xy & vec2<u32>(1u)) == vec2<u32>(0u))) {
    let s1 = (level0[li] + level0[li + 1u] + level0[li + 16u] + level0[li + 17u]) * 0.25;
    textureStore(mip1_out, m0 / 2, vec4<f32>(s1, 0.0, 0.0));
    level1[l1.y * 8u + l1.x] = s1;
  }
  workgroupBarrier();
  if (all((local.xy & vec2<u32>(3u)) == vec2<u32>(0u))) {
    let k = (l1.y * 8u + l1.x);
    let s2 = (level1[k] + level1[k + 1u] + level1[k + 8u] + level1[k + 9u]) * 0.25;
    textureStore(mip2_out, m0 / 4, vec4<f32>(s2, 0.0, 0.0));
  }
}
