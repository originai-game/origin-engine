// AMD FidelityFX Super Resolution 3.1 upscaler, ported to WGSL from FSR
// 3.1.6 (AMD FSR SDK 2.4.0, GPUOpen-LibrariesAndSDKs/FidelityFX-SDK
// 7539ae8b, Kits/FidelityFX/upscalers/fsr3/include/gpu/fsr3upscaler).
// Copyright (C) 2026 Advanced Micro Devices, Inc. MIT:
// third_party/fidelityfx/LICENSE.txt. The constants and helpers of
// ffx_fsr3upscaler_common.h and ffx_fsr3upscaler_sample.h, shared by every
// fsr_*.wgsl pass (post/fsr.rs prepends this file). The port's deviations
// are listed in third_party/fidelityfx/README.md.

// `Fsr3UpscalerConstants` (ffx_fsr3upscaler_private.h) and the engine's
// reprojection (the camera's motion through the depth, as taa.wgsl).
struct Fsr {
  // Current frame clip (jittered) -> world.
  inv_view_proj: mat4x4<f32>,
  // World -> current and previous clip, unjittered.
  cur_view_proj: mat4x4<f32>,
  prev_view_proj: mat4x4<f32>,
  // xy: render size, zw: the previous frame's.
  render: vec4<i32>,
  // xy: upscale (output) size, zw: the previous frame's.
  upscale: vec4<i32>,
  // xy: the most the render size can be (the FSR-owned render-sized
  // textures' size), zw: the most the upscale size can be.
  max_size: vec4<i32>,
  // View depth (positive, metres) from device depth d:
  // -(x d + y) / (z d + w), rows 2 and 3 of the inverse projection.
  device_to_view: vec4<f32>,
  // xy: this frame's jitter (render pixels, y down), zw: the previous's.
  jitter: vec4<f32>,
  // x: delta time (s, 0..1), y: delta pre-exposure, z: frame index (0 on a
  // reset), w: jitter sequence length.
  frame: vec4<f32>,
  // x: velocity factor, y: reactiveness scale, z: shading change scale,
  // w: accumulation added per frame.
  tuning: vec4<f32>,
  // x: min disocclusion accumulation, y: 1 when the camera is still (no
  // reprojection on the GPU), z: 1 with the prepass's velocity bound, w:
  // RCAS sharpness (linear, exp2(-stops)).
  extra: vec4<f32>,
  // x: the luma pyramid's workgroups (luma_partials' length), y: the
  // OIT revealage's reactive cap, z: 1 when the OIT drew (its revealage is
  // the reactive mask), w: 0.
  misc: vec4<f32>,
};

@group(0) @binding(0) var<uniform> fsr: Fsr;

const FP16_MIN: f32 = 6.10e-05;
const FP16_MAX: f32 = 65504.0;
const EPSILON: f32 = FP16_MIN;
const TONEMAP_EPSILON: f32 = FP16_MIN;
const FP32_MAX: f32 = 3.4028234e38;
const FP32_MIN: f32 = 1.175494351e-38;
const RECONSTRUCTED_DEPTH_BILINEAR_WEIGHT_THRESHOLD: f32 = FP16_MIN * 10.0;
const UPSAMPLE_LANCZOS_WEIGHT_SCALE: f32 = 1.0 / 16.0;
const AVERAGE_LANCZOS_WEIGHT_PER_FRAME: f32 = 0.74 * UPSAMPLE_LANCZOS_WEIGHT_SCALE;
const LOCK_THRESHOLD: f32 = 1.0;
const LOCK_MAX: f32 = 2.0;
const SHADING_CHANGE_SET_SIZE: i32 = 5;
const SHADING_CHANGE_MIP_START: i32 = 0;
// The channels of the dilated reactive masks.
const REACTIVE: i32 = 0;
const DISOCCLUSION: i32 = 1;
const SHADING_CHANGE: i32 = 2;
const ACCUMULATION: i32 = 3;
// The frame info's channels.
const FRAME_INFO_EXPOSURE: i32 = 0;
const FRAME_INFO_LOG_LUMA: i32 = 1;
const FRAME_INFO_SCENE_AVERAGE_LUMA: i32 = 2;
const RESET_AUTO_EXPOSURE_AVERAGE_SMOOTHING: f32 = 1e4;
const PI: f32 = 3.141592653589793;

fn render_size() -> vec2<i32> { return fsr.render.xy; }
fn previous_render_size() -> vec2<i32> { return fsr.render.zw; }
fn upscale_size() -> vec2<i32> { return fsr.upscale.xy; }
fn previous_upscale_size() -> vec2<i32> { return fsr.upscale.zw; }
fn max_render_size() -> vec2<i32> { return fsr.max_size.xy; }
fn jitter() -> vec2<f32> { return fsr.jitter.xy; }
fn previous_jitter() -> vec2<f32> { return fsr.jitter.zw; }
fn delta_time() -> f32 { return fsr.frame.x; }
fn delta_pre_exposure() -> f32 { return fsr.frame.y; }
fn frame_index() -> f32 { return fsr.frame.z; }
fn jitter_sequence_length() -> f32 { return fsr.frame.w; }
fn velocity_factor() -> f32 { return fsr.tuning.x; }
fn reactiveness_scale() -> f32 { return fsr.tuning.y; }
fn shading_change_scale() -> f32 { return fsr.tuning.z; }
fn accumulation_added_per_frame() -> f32 { return fsr.tuning.w; }
fn min_disocclusion_accumulation() -> f32 { return fsr.extra.x; }
fn downscale_factor() -> vec2<f32> { return vec2<f32>(render_size()) / vec2<f32>(upscale_size()); }
fn shading_change_render_size() -> vec2<i32> { return vec2<i32>(vec2<f32>(render_size()) * 0.5); }
fn shading_change_max_render_size() -> vec2<i32> { return vec2<i32>(vec2<f32>(max_render_size()) * 0.5); }

// Exposure() of the callbacks: the frame info's, 1 below 2^-15.
fn exposure_of(info: vec4<f32>) -> f32 {
  let e = info.x;
  return select(e, 1.0, e < 0.000030517578);
}

fn reconstructed_depth_mv_px_threshold(nearest_depth_m: f32) -> f32 {
  return mix(0.25, 0.75, saturate(nearest_depth_m / 100.0));
}

fn get_4k_velocity(mv: vec2<f32>) -> f32 {
  return length(mv * vec2<f32>(3840.0, 2160.0));
}

fn min_divided_by_max_or(v0: f32, v1: f32, on_zero: f32) -> f32 {
  let m = max(v0, v1);
  return select(on_zero, min(v0, v1) / m, m != 0.0);
}

fn min_divided_by_max(v0: f32, v1: f32) -> f32 {
  return min_divided_by_max_or(v0, v1, 0.0);
}

fn ycocg_to_rgb(c: vec3<f32>) -> vec3<f32> {
  return vec3<f32>(c.x + c.y - c.z, c.x + c.z, c.x - c.y - c.z);
}

fn rgb_to_ycocg(c: vec3<f32>) -> vec3<f32> {
  return vec3<f32>(
    0.25 * c.r + 0.5 * c.g + 0.25 * c.b,
    0.5 * c.r - 0.5 * c.b,
    -0.25 * c.r + 0.5 * c.g - 0.25 * c.b);
}

fn rgb_to_luma(c: vec3<f32>) -> f32 {
  return dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn tonemap(c: vec3<f32>) -> vec3<f32> {
  return c / (max(max(0.0, c.r), max(c.g, c.b)) + 1.0);
}

fn inverse_tonemap(c: vec3<f32>) -> vec3<f32> {
  return c / max(TONEMAP_EPSILON, 1.0 - max(c.r, max(c.g, c.b)));
}

fn is_uv_inside(uv: vec2<f32>) -> bool {
  return all(uv >= vec2<f32>(0.0)) & all(uv <= vec2<f32>(1.0));
}

fn clamp_load(p: vec2<i32>, offset: vec2<i32>, size: vec2<i32>) -> vec2<i32> {
  return clamp(p + offset, vec2<i32>(0), size - 1);
}

// The uv of `uv` (over `texture_size`) in a resource of `resource_size`,
// kept half a texel inside the image.
fn clamp_uv(uv: vec2<f32>, texture_size: vec2<i32>, resource_size: vec2<i32>) -> vec2<f32> {
  let location = uv * vec2<f32>(texture_size);
  let clamped = max(vec2<f32>(0.5), min(location, vec2<f32>(texture_size) - 0.5));
  return clamped / vec2<f32>(resource_size);
}

fn is_on_screen(p: vec2<i32>, size: vec2<i32>) -> bool {
  return all(vec2<u32>(p) < vec2<u32>(size));
}

fn compute_auto_exposure_from_lavg(lavg_log: f32) -> f32 {
  let lavg = exp(lavg_log);
  let s = 100.0;
  let k = 12.5;
  let exposure_iso100 = log2((lavg * s) / k);
  let q = 0.65;
  let lmax = (78.0 / (q * s)) * pow(2.0, exposure_iso100);
  return 1.0 / lmax;
}

fn compute_hr_pos_from_lr_pos(lr: vec2<i32>) -> vec2<i32> {
  let src = vec2<f32>(lr) + 0.5 - jitter();
  let in_hr = (src / vec2<f32>(render_size())) * vec2<f32>(upscale_size());
  return vec2<i32>(floor(in_hr));
}

// GetViewSpaceDepth: the positive view depth of a device depth (metres:
// the engine's view space is in metres, ViewSpaceToMetersFactor 1). Far
// past FP16_MAX stays finite: callers clamp to it as AMD's do.
fn view_depth(device_depth: f32) -> f32 {
  let k = fsr.device_to_view;
  let w = k.z * device_depth + k.w;
  let z = -(k.x * device_depth + k.y);
  if (abs(w) < 1e-30) { return FP32_MAX; }
  return z / w;
}

struct BilinearSamplingData {
  base: vec2<i32>,
  weights: vec4<f32>,
};

const BILINEAR_OFFSETS: array<vec2<i32>, 4> = array<vec2<i32>, 4>(
  vec2<i32>(0, 0), vec2<i32>(1, 0), vec2<i32>(0, 1), vec2<i32>(1, 1));

fn get_bilinear_sampling_data(uv: vec2<f32>, size: vec2<i32>) -> BilinearSamplingData {
  var d: BilinearSamplingData;
  let px = uv * vec2<f32>(size) - 0.5;
  d.base = vec2<i32>(floor(px));
  let f = fract(px);
  d.weights = vec4<f32>(
    (1.0 - f.x) * (1.0 - f.y),
    f.x * (1.0 - f.y),
    (1.0 - f.x) * f.y,
    f.x * f.y);
  return d;
}

// A bilinear read of an unfilterable one-channel texture (r32float has no
// filtering in core WebGPU), as a linear clamp sampler reads it.
fn bilinear_r(t: texture_2d<f32>, uv: vec2<f32>) -> f32 {
  let size = vec2<i32>(textureDimensions(t));
  let px = uv * vec2<f32>(size) - 0.5;
  let base = vec2<i32>(floor(px));
  let f = fract(px);
  let hi = size - 1;
  let a = textureLoad(t, clamp(base, vec2<i32>(0), hi), 0).r;
  let b = textureLoad(t, clamp(base + vec2<i32>(1, 0), vec2<i32>(0), hi), 0).r;
  let c = textureLoad(t, clamp(base + vec2<i32>(0, 1), vec2<i32>(0), hi), 0).r;
  let d = textureLoad(t, clamp(base + vec2<i32>(1, 1), vec2<i32>(0), hi), 0).r;
  return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}

fn lanczos2_no_clamp(x: f32) -> f32 {
  if (abs(x) < EPSILON) { return 1.0; }
  return (sin(PI * x) / (PI * x)) * (sin(0.5 * PI * x) / (0.5 * PI * x));
}

fn lanczos2(x: f32) -> f32 {
  return lanczos2_no_clamp(min(abs(x), 2.0));
}

// FSR1's Lanczos approximation; the input is x * x, at most 4.
fn lanczos2_approx_sq(x2_in: f32) -> f32 {
  let x2 = min(x2_in, 4.0);
  let a = (2.0 / 5.0) * x2 - 1.0;
  let b = (1.0 / 4.0) * x2 - 1.0;
  return ((25.0 / 16.0) * a * a - (25.0 / 16.0 - 1.0)) * (b * b);
}
