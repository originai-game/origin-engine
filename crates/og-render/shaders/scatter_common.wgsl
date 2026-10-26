// Scattered copies over a terrain (scatter.rs): instances cached per chunk,
// each a world transform, faded out with the distance from the camera by
// a dither (never shrunk), swayed by the wind. Shared by the lit, prepass
// and shadow passes.

struct ScatterUniform {
  camera: vec4<f32>,          // xyz: the camera, w: the distance copies reach (m)
  fade: vec4<f32>,            // x: where the fade starts (m)
  view_proj: mat4x4<f32>,     // unjittered (prepass velocity)
  prev_view_proj: mat4x4<f32>,
  wind: vec4<f32>,            // xy: the wind's horizontal direction (x, z), z: the mean lean (0: still), w: the gust
  wind_phase: vec4<f32>,      // xy: the gust wave's two time phases this frame, zw: the previous frame's
  wind_shape: vec4<f32>,      // x: the wave's wavenumber (rad/m), y: the mesh's top above its origin (m)
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

// lowbias32 (scatter.rs `hash`).
fn scatter_hash(x: u32) -> u32 {
  var h = x;
  h ^= h >> 16u;
  h *= 0x7feb352du;
  h ^= h >> 15u;
  h *= 0x846ca68bu;
  return h ^ (h >> 16u);
}

// How the wind moves a vertex at `local` (mesh space) of the copy standing
// at `base` with uniform scale `scale`, at the time phases `phase`
// (scatter.rs, the module's Wind): downwind by the lean times its height
// times its height over the mesh's top, lowered to keep its distance from
// the root. The lean varies with a wave of two octaves travelling
// downwind, offset a little per copy by its position's bits.
fn scatter_sway(local: vec3<f32>, base: vec3<f32>, scale: f32, phase: vec2<f32>) -> vec3<f32> {
  let lean = scatter.wind.z;
  let top = scatter.wind_shape.y;
  if (lean <= 0.0 || top <= 0.0 || local.y <= 0.0) {
    return vec3<f32>(0.0);
  }
  let dir = scatter.wind.xy;
  let jitter = f32(scatter_hash(bitcast<u32>(base.x) ^ scatter_hash(bitcast<u32>(base.z))) >> 8u) / 16777216.0;
  let x = scatter.wind_shape.x * dot(base.xz, dir) + jitter;
  let wave = (sin(x - phase.x) + 0.5 * sin(2.3 * x + 1.7 - phase.y)) / 1.5;
  let h = local.y * scale;
  let d = lean * (1.0 + scatter.wind.w * wave) * h * min(local.y / top, 1.0);
  let drop = h - sqrt(max(h * h - d * d, 0.0));
  return vec3<f32>(dir.x * d, -drop, dir.y * d);
}
