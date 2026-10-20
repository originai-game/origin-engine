// Scattered copies over a terrain (scatter.rs): instances cached per chunk,
// each a world transform, faded out with the distance from the camera by
// a dither (never shrunk). Shared by the lit, prepass and shadow passes.

struct ScatterUniform {
  camera: vec4<f32>,          // xyz: the camera, w: the distance copies reach (m)
  fade: vec4<f32>,            // x: where the fade starts (m)
  view_proj: mat4x4<f32>,     // unjittered (prepass velocity)
  prev_view_proj: mat4x4<f32>,
};
@group(2) @binding(0) var<uniform> scatter: ScatterUniform;

struct ScatterVertex {
  @location(0) position: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
  @location(3) color: vec4<f32>,
};

// A copy's transform: its columns' xyz (instance.wgsl).
struct ScatterInstance {
  @location(4) m0: vec3<f32>,
  @location(5) m1: vec3<f32>,
  @location(6) m2: vec3<f32>,
  @location(7) m3: vec3<f32>,
};

// How much of a copy standing at `at` shows: 1 inside the fade, 0 past the distance.
fn scatter_fade(at: vec3<f32>) -> f32 {
  let d = length(at.xz - scatter.camera.xz);
  if (d > scatter.camera.w) { return 0.0; }
  if (scatter.camera.w <= scatter.fade.x) { return 1.0; }
  return 1.0 - smoothstep(scatter.fade.x, scatter.camera.w, d);
}

// Whether a pixel of a copy faded to `fade` is dropped: an interleaved
// gradient noise threshold per pixel, so the copy thins out evenly.
fn scatter_dropped(pixel: vec2<f32>, fade: f32) -> bool {
  let n = fract(52.9829189 * fract(dot(pixel, vec2<f32>(0.06711056, 0.00583715))));
  return n >= fade;
}

// A clip position outside the view: the copy's triangles are culled.
const SCATTER_GONE = vec4<f32>(2.0, 2.0, 2.0, 1.0);
