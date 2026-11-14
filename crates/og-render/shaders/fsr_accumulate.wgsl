// FSR 3.1's accumulate pass (ffx_fsr3upscaler_accumulate.h, _upsample.h,
// _reproject.h; MIT, third_party/fidelityfx/LICENSE.txt), with HDR input:
// per output pixel the history reprojected with a deringed Lanczos-2
// (AMD's reference kernel) and its lock, the current frame upsampled with
// a Lanczos-2 slice over the 3x3 samples nearest (its jitter walks the
// slices over the frames), the history rectified into the samples' variance
// box unless locks or luma instability vouch for it, and the two blended
// by their weights in a reversibly tonemapped YCoCg. Writes the output,
// which is also next frame's history colour (the engine's TAA history),
// and the lock; the output's alpha (the
// engine's tone-mapped fraction, post.wgsl `tonemapped`) is the current
// frame's, blended with the previous output's along the motion as the
// colour is and clamped to the current samples' range, as taa.wgsl does.
// Prepended by fullscreen.wgsl and fsr_common.wgsl.

@group(0) @binding(1) var dilated_reactive_tex: texture_2d<f32>;
@group(0) @binding(2) var dilated_motion_tex: texture_2d<f32>;
@group(0) @binding(3) var lock_tex: texture_2d<f32>;
@group(0) @binding(4) var farthest_depth_mip1_tex: texture_2d<f32>;
@group(0) @binding(5) var color_tex: texture_2d<f32>;
@group(0) @binding(6) var luma_instability_tex: texture_2d<f32>;
@group(0) @binding(7) var<storage, read> frame_info: vec4<f32>;
@group(0) @binding(8) var<storage, read_write> new_locks: array<f32>;
@group(0) @binding(9) var linear_clamp: sampler;
@group(0) @binding(10) var previous_output_tex: texture_2d<f32>;
@group(0) @binding(11) var lock_out: texture_storage_2d<r32float, write>;
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
  // The lock the reactivity pass set here this frame (0 for none).
  new_lock: f32,
};

fn new_lock_index(hr: vec2<i32>) -> u32 {
  return u32(hr.y) * u32(upscale_size().x) + u32(hr.x);
}

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
  // The engine's alpha in the history (the previous output's).
  history_alpha: f32,
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
  return textureLoad(previous_output_tex, p, 0);
}

// The history at `p` (previous_output_tex) with its texel coordinates in
// `x` and `y`, `t` of the way from x.x to x.y (and y.x to y.y): one
// filtered fetch for two (or four) texels at their weights' ratio.
fn history_between(x: vec2<i32>, tx: f32, y: vec2<i32>, ty: f32) -> vec3<f32> {
  let at = vec2<f32>(mix(f32(x.x), f32(x.y), tx), mix(f32(y.x), f32(y.y), ty)) + 0.5;
  return textureSampleLevel(previous_output_tex, linear_clamp, at / vec2<f32>(previous_upscale_size()), 0.0).rgb;
}

// ClampCoord of ffx_fsr3upscaler_sample.h.
fn clamp_coord(p: vec2<i32>, offset: vec2<i32>, size: vec2<i32>) -> vec2<i32> {
  return clamp(p + offset, vec2<i32>(1), size - 2);
}

// The four Lanczos-2 weights of a 4-tap row at fraction `t` (taps at
// k - t for k = -1..2): lanczos2(x) = sin(pi x) sin(pi x / 2) / (pi^2 x^2 / 2),
// and over the four taps sin(pi (k - t)) is +-sin(pi t) and
// sin(pi (k - t) / 2) is +-cos(pi t / 2) or +-sin(pi t / 2), so three
// transcendentals make the row (lanczos2 eight times took sixteen sines).
fn lanczos2_weights(t: f32) -> vec4<f32> {
  let s = sin(PI * t);
  let a = sin(0.5 * PI * t);
  let c = cos(0.5 * PI * t);
  let x = vec4<f32>(-1.0 - t, -t, 1.0 - t, 2.0 - t);
  let w = vec4<f32>(-s * c, s * a, s * c, -s * a) / (0.5 * PI * PI * x * x);
  return select(w, vec4<f32>(1.0), abs(x) < vec4<f32>(EPSILON));
}

// HistorySample: DeclareCustomTextureSample with the reference Lanczos2
// over FetchHistorySamples' 4x4, deringed to the middle 2x2, read from the
// engine's previous output (which holds FSR's history colour, written
// once a frame). The kernel is separable, so its weights are made once
// per axis; the middle 2x2 is loaded (for the deringing box too), each
// edge's middle pair (two positive weights) is one filtered fetch at their
// ratio, and the corners (together at most 1/64 of the weight, at a half-texel offset
// on both axes) are left
// out and the sum renormalised: 8 fetches, not 16. The alpha (the
// engine's, not FSR's) is the middle 2x2's, bilinear.
fn history_sample(uv: vec2<f32>, size: vec2<i32>) -> vec4<f32> {
  var px = uv * vec2<f32>(size) - 0.5;
  let f = fract(px);
  px = clamp(px, vec2<f32>(0.0), vec2<f32>(size - 1));
  let ip = vec2<i32>(floor(px));
  let wx = lanczos2_weights(f.x);
  let wy = lanczos2_weights(f.y);
  let lo = clamp_coord(ip, vec2<i32>(-1), size);
  let mid0 = clamp_coord(ip, vec2<i32>(0), size);
  let mid1 = clamp_coord(ip, vec2<i32>(1), size);
  let hi = clamp_coord(ip, vec2<i32>(2), size);
  let h11 = load_history(mid0);
  let h21 = load_history(vec2<i32>(mid1.x, mid0.y));
  let h12 = load_history(vec2<i32>(mid0.x, mid1.y));
  let h22 = load_history(mid1);
  let c11 = h11.rgb;
  let c21 = h21.rgb;
  let c12 = h12.rgb;
  let c22 = h22.rgb;
  let mx = wx.y + wx.z;
  let my = wy.y + wy.z;
  let xs = vec2<i32>(mid0.x, mid1.x);
  let ys = vec2<i32>(mid0.y, mid1.y);
  let top = history_between(xs, wx.z / mx, vec2<i32>(lo.y), 0.0);
  let bottom = history_between(xs, wx.z / mx, vec2<i32>(hi.y), 0.0);
  let left = history_between(vec2<i32>(lo.x), 0.0, ys, wy.z / my);
  let right = history_between(vec2<i32>(hi.x), 0.0, ys, wy.z / my);
  let c = wy.y * (wx.y * c11 + wx.z * c21) + wy.z * (wx.y * c12 + wx.z * c22)
    + mx * (wy.x * top + wy.w * bottom)
    + my * (wx.x * left + wx.w * right);
  let total = mx * my + mx * (wy.x + wy.w) + my * (wx.x + wx.w);
  let a = mix(mix(h11.a, h21.a, f.x), mix(h12.a, h22.a, f.x), f.y);
  return vec4<f32>(clamp(c / total, min(min(c11, c21), min(c12, c22)), max(max(c11, c21), max(c12, c22))), a);
}

// The current colour exposed, in YCoCg, with the engine's alpha beside it.
fn prepared_color(p: vec2<i32>) -> vec4<f32> {
  let c = textureLoad(color_tex, p, 0);
  return vec4<f32>(rgb_to_ycocg(max(vec3<f32>(0.0), c.rgb) * exposure()), c.a);
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
  p.luma_instability = textureSampleLevel(luma_instability_tex, linear_clamp, clamp_uv(p.hr_uv, render_size(), max_render_size()), 0.0).r;
  let farthest_uv = clamp_uv(p.lr_uv_jittered, render_size() / 2, vec2<i32>(textureDimensions(farthest_depth_mip1_tex)));
  p.farthest_depth_m = textureSampleLevel(farthest_depth_mip1_tex, linear_clamp, farthest_uv, 0.0).r;
  p.is_new_sample = !p.is_existing_sample | (frame_index() == 0.0);
  let masks = textureSampleLevel(dilated_reactive_tex, linear_clamp, p.lr_uv_hw, 0.0);
  p.reactive = saturate(masks[REACTIVE]);
  p.disocclusion = saturate(masks[DISOCCLUSION]);
  p.shading_change = saturate(masks[SHADING_CHANGE]);
  p.accumulation = saturate(masks[ACCUMULATION]);
  p.accumulation *= f32(round(p.accumulation * 100.0) > 1.0);
  p.new_lock = new_locks[new_lock_index(hr)];
  return p;
}

fn reproject_history_color(p: Params, d: ptr<function, Data>) {
  let h = history_sample(p.reprojected_hr_uv, previous_upscale_size());
  (*d).history_color = rgb_to_ycocg(h.rgb * delta_pre_exposure() * exposure());
  (*d).history_alpha = h.a;
  // The lock beside it, bilinear (AMD's rides in the history's alpha
  // through the same Lanczos and deringing, whose result lies in the
  // middle 2x2's range as a bilinear one does).
  (*d).lock = bilinear_r(lock_tex, p.reprojected_hr_uv);
}

fn update_lock_status(p: Params, d: ptr<function, Data>) {
  (*d).lock *= f32(!p.is_new_sample);
  let decrease = max(saturate(p.shading_change), max(p.reactive, p.disocclusion));
  (*d).lock = max(0.0, (*d).lock - decrease * LOCK_MAX);
  (*d).lock_contribution = saturate(saturate((*d).lock - LOCK_THRESHOLD) * (LOCK_MAX - LOCK_THRESHOLD));
  let new_lock = p.new_lock * (1.0 - max(p.shading_change * 0.0, p.reactive));
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

// The prepared current colour (exposed, YCoCg, with the alpha) under a
// workgroup's 8x8 output pixels: their 3x3 footprints, src_input - 1 to
// src_input + 1, span at most 10 render texels a side when the render is
// no larger than the output (the pass's condition), one more for
// rounding. Neighbouring output pixels share most of their nine taps, so
// each texel is loaded and prepared once per group rather than nine times
// per pixel; the alpha's 2x2 at the jittered place lies in the same 3x3.
const TILE: i32 = 11;
var<workgroup> tile: array<vec4<f32>, 121>;
var<private> tile_origin: vec2<i32>;

// Where the history is whole, the upsample's Lanczos-2 is twice as narrow
// as AMD's: its first zero half an output pixel from the sample (AMD's at
// one), so what the frames accumulate converges to a box over the output
// pixel, as a supersampled frame does, rather than to a Lanczos-2 a pixel
// wide that drops a thin feature's contrast (fsr::compare, thin static
// geometry: 1.1 to 1.8 dB nearer the reference). The narrowing eases back
// to AMD's kernel as the history weight falls (motion, disocclusion, a
// cut): there few frames are blended and a narrow kernel keeps each
// frame's aliasing (grass while panning lost 0.3 dB with it fixed). The
// kernel covers 1/n^2 of the area, so a frame's weight is n^2 times AMD's
// average, keeping its convergence rate.
const KERNEL_NARROWING: f32 = 2.0;

// What every one of the 3x3 taps of the upsample shares.
struct Taps {
  src_input: vec2<i32>,
  offset_tl: vec2<i32>,
  flip_col: bool,
  flip_row: bool,
  base_offset: vec2<f32>,
  initial: bool,
  kernel_bias: f32,
};

// One tap of ComputeUpsampledColorAndWeight: its sample (tonemapped for an
// initial one), its Lanczos weight and its place in the rectification box.
// The nine calls have constant (col, row): naga writes every WGSL loop as
// a guarded `while (true)` the Metal compiler does not unroll, so a 3x3
// loop over a private array of samples kept the array in stack memory.
fn upsample_tap(t: Taps, d: ptr<function, Data>, col: i32, row: i32) {
  let cr = vec2<i32>(select(col, 3 - col, t.flip_col), select(row, 3 - row, t.flip_row));
  let src = t.src_input + t.offset_tl + cr;
  let k = src - tile_origin;
  var s = tile[k.y * TILE + k.x].xyz;
  if (t.initial) {
    s = rgb_to_ycocg(tonemap(ycocg_to_rgb(s)));
  }
  let sample_offset = t.base_offset + vec2<f32>(t.offset_tl) + vec2<f32>(cr);
  let on_screen = f32(is_on_screen(src, render_size()));
  // The outer ring lies at least a render pixel off on one axis, where a
  // kernel biased by 2 or more (KERNEL_NARROWING) is zero: its weight is
  // not made.
  if (!t.initial & ((t.kernel_bias < 2.0) | ((col > 0) & (row > 0)))) {
    // GetUpsampleLanczosWeight, LANCZOS_TYPE_APPROXIMATE.
    let biased = sample_offset * t.kernel_bias;
    let w = on_screen * lanczos2_approx_sq(dot(biased, biased));
    (*d).upsampled_color += s * w;
    (*d).upsampled_weight += w;
  }
  let box_weight = exp(-2.3 * dot(sample_offset, sample_offset)) * on_screen;
  box_add((row == 0) & (col == 0), d, s, box_weight);
}

fn compute_upsampled_color_and_weight(p: Params, d: ptr<function, Data>) {
  let dst = vec2<f32>(p.hr_pos) + 0.5;
  let src_output = dst * downscale_factor();
  let src_input = vec2<i32>(floor(src_output));
  let src_unjittered = vec2<f32>(src_input) + 0.5 - jitter();
  let flip_col = src_unjittered.x > src_output.x;
  let flip_row = src_unjittered.y > src_output.y;

  // ComputeMaxKernelWeight.
  let kernel_bias_max = min(1.99, 1.0 + (1.0 / downscale_factor().x - 1.0));
  let kernel_bias_min = max(1.0, (1.0 + kernel_bias_max) * 0.3);
  let narrowing = mix(KERNEL_NARROWING, 1.0, 1.0 - saturate((*d).history_weight));
  let kernel_bias_weight = min(1.0 - p.disocclusion * 0.5, min(1.0 - p.shading_change, saturate((*d).history_weight * 5.0)));

  let t = Taps(
    src_input,
    vec2<i32>(select(-1, -2, flip_col), select(-1, -2, flip_row)),
    flip_col,
    flip_row,
    src_unjittered - src_output,
    p.accumulation == 0.0,
    mix(kernel_bias_min, kernel_bias_max, kernel_bias_weight) * narrowing,
  );
  upsample_tap(t, d, 0, 0);
  upsample_tap(t, d, 1, 0);
  upsample_tap(t, d, 2, 0);
  upsample_tap(t, d, 0, 1);
  upsample_tap(t, d, 1, 1);
  upsample_tap(t, d, 2, 1);
  upsample_tap(t, d, 0, 2);
  upsample_tap(t, d, 1, 2);
  upsample_tap(t, d, 2, 2);
  box_variance(d);

  (*d).upsampled_weight *= f32((*d).upsampled_weight > EPSILON);
  if ((*d).upsampled_weight > EPSILON) {
    (*d).upsampled_color /= (*d).upsampled_weight;
    (*d).upsampled_weight *= AVERAGE_LANCZOS_WEIGHT_PER_FRAME * narrowing * narrowing;
    // Deringing.
    (*d).upsampled_color = clamp((*d).upsampled_color, (*d).clipping_box.aabb_min, (*d).clipping_box.aabb_max);
  }
  if (p.accumulation == 0.0) {
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
fn cs(@builtin(global_invocation_id) id: vec3<u32>, @builtin(workgroup_id) wg: vec3<u32>, @builtin(local_invocation_index) li: u32) {
  // The group's 3x3 footprints, prepared once into the tile.
  let first = vec2<i32>(wg.xy) * 8;
  tile_origin = vec2<i32>(floor((vec2<f32>(first) + 0.5) * downscale_factor())) - 1;
  let i0 = i32(li);
  let i1 = i32(li) + 64;
  tile[i0] = prepared_color(clamp_load(tile_origin, vec2<i32>(i0 % TILE, i0 / TILE), render_size()));
  if (i1 < TILE * TILE) {
    tile[i1] = prepared_color(clamp_load(tile_origin, vec2<i32>(i1 % TILE, i1 / TILE), render_size()));
  }
  workgroupBarrier();

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
  textureStore(lock_out, hr, vec4<f32>(d.lock, 0.0, 0.0, 0.0));

  // The engine's alpha: the current frame's, bilinear at the jittered
  // place from the tile (its clamped texels are a clamp-to-edge
  // sampler's), with the previous output's along the motion clamped to
  // the range of the same four samples.
  let q = p.lr_uv_jittered * vec2<f32>(render_size()) - 0.5;
  let k = vec2<i32>(floor(q)) - tile_origin;
  let fq = fract(q);
  let a0 = tile[k.y * TILE + k.x].w;
  let a1 = tile[k.y * TILE + k.x + 1].w;
  let a2 = tile[(k.y + 1) * TILE + k.x].w;
  let a3 = tile[(k.y + 1) * TILE + k.x + 1].w;
  var a = mix(mix(a0, a1, fq.x), mix(a2, a3, fq.x), fq.y);
  if (p.is_existing_sample & !p.is_new_sample) {
    let a_hist = clamp(d.history_alpha, min(min(a0, a1), min(a2, a3)), max(max(a0, a1), max(a2, a3)));
    a = mix(a_hist, a, alpha);
  }
  textureStore(output, hr, vec4<f32>(color, a));
  // Cleared where the reactivity pass set one (few pixels): no store
  // for the rest.
  if (p.new_lock != 0.0) {
    new_locks[new_lock_index(hr)] = 0.0;
  }
}
