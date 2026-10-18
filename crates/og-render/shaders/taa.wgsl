// Temporal antialiasing (WebGPU tier): the jittered frame blended into the
// history. The history is reprojected through the depth buffer with the
// unjittered matrices (camera motion only; moving objects rely on the clamp),
// clamped to the current 3x3 neighbourhood's colour box, and blended with
// luminance weights so bright pixels do not flicker. Prepended by fullscreen.wgsl.
struct Taa {
  // current frame (jittered) clip -> world
  inv_view_proj: mat4x4<f32>,
  // world -> current clip without jitter, and the previous frame's
  cur_view_proj: mat4x4<f32>,
  prev_view_proj: mat4x4<f32>,
  params: vec4<f32>,  // x: current frame weight (1 = no history), yz: target size
};
@group(0) @binding(0) var current: texture_2d<f32>;
@group(0) @binding(1) var history: texture_2d<f32>;
// The depth buffer (Depth32Float) read as an unfilterable float texture:
// GLSL cannot textureLoad a texture_depth_2d.
@group(0) @binding(2) var depth: texture_2d<f32>;
@group(0) @binding(3) var linear_sampler: sampler;
@group(0) @binding(4) var<uniform> taa: Taa;

fn to_ycocg(c: vec3<f32>) -> vec3<f32> {
  return vec3<f32>(
    0.25 * c.r + 0.5 * c.g + 0.25 * c.b,
    0.5 * c.r - 0.5 * c.b,
    -0.25 * c.r + 0.5 * c.g - 0.25 * c.b);
}

fn from_ycocg(c: vec3<f32>) -> vec3<f32> {
  return vec3<f32>(c.x + c.y - c.z, c.x + c.z, c.x - c.y - c.z);
}

@fragment
fn fs(in: VsOut) -> @location(0) vec4<f32> {
  let size = vec2<i32>(taa.params.yz);
  let px = vec2<i32>(in.pos.xy);
  let cur = textureLoad(current, px, 0).rgb;
  if (taa.params.x >= 1.0) { return vec4<f32>(cur, 1.0); }

  // Neighbourhood colour box (YCoCg).
  var lo = vec3<f32>(1e9);
  var hi = vec3<f32>(-1e9);
  for (var y = -1; y <= 1; y++) {
    for (var x = -1; x <= 1; x++) {
      let at = clamp(px + vec2<i32>(x, y), vec2<i32>(0), size - 1);
      let c = to_ycocg(textureLoad(current, at, 0).rgb);
      lo = min(lo, c);
      hi = max(hi, c);
    }
  }

  // Reproject: this pixel's world point, seen by the previous frame.
  let d = textureLoad(depth, px, 0).r;
  let ndc = vec2<f32>(in.uv.x * 2.0 - 1.0, 1.0 - in.uv.y * 2.0);
  let w = taa.inv_view_proj * vec4<f32>(ndc, d, 1.0);
  let world = w.xyz / w.w;
  let cur_clip = taa.cur_view_proj * vec4<f32>(world, 1.0);
  let prev_clip = taa.prev_view_proj * vec4<f32>(world, 1.0);
  let motion = (cur_clip.xy / cur_clip.w - prev_clip.xy / prev_clip.w) * vec2<f32>(0.5, -0.5);
  let prev_uv = in.uv - motion;
  if (any(prev_uv < vec2<f32>(0.0)) || any(prev_uv > vec2<f32>(1.0))) {
    return vec4<f32>(cur, 1.0);
  }
  var hist = textureSampleLevel(history, linear_sampler, prev_uv, 0.0).rgb;
  hist = from_ycocg(clamp(to_ycocg(hist), lo, hi));

  let a = taa.params.x;
  let wc = a / (1.0 + dot(cur, LUMA));
  let wh = (1.0 - a) / (1.0 + dot(hist, LUMA));
  return vec4<f32>((cur * wc + hist * wh) / (wc + wh), 1.0);
}
