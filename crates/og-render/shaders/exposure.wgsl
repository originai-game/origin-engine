// Temporal exposure adaptation (Exposure::Auto with adapt speeds). The
// metered target EV, as post.wgsl's exposure() computes it from the 1x1
// log2-luminance texel, is approached from the previous frame's EV by at
// most `up` EV per second upwards and `down` downwards over the frame's
// tick time (a value the renderer hands in: the tick owns time). A speed of
// 0 reaches the target at once in that direction. Prepended by fullscreen.wgsl.
struct Adapt {
  speed: vec4<f32>,  // x: seconds since the previous adapted frame, y: up EV/s, z: down EV/s, w: 1 snaps to the target
  meter: vec4<f32>,  // x: compensation, y: min ev, z: max ev
};
@group(0) @binding(0) var metered: texture_2d<f32>;
@group(0) @binding(1) var previous: texture_2d<f32>;
@group(0) @binding(2) var<uniform> adapt: Adapt;

@fragment
fn fs_adapt(in: VsOut) -> @location(0) vec4<f32> {
  let avg = textureLoad(metered, vec2<i32>(0), 0).r;
  let goal = clamp(log2(0.18) - avg + adapt.meter.x, adapt.meter.y, adapt.meter.z);
  if (adapt.speed.w > 0.5) { return vec4<f32>(goal, 0.0, 0.0, 1.0); }
  let prev = textureLoad(previous, vec2<i32>(0), 0).r;
  let d = goal - prev;
  var step = d;
  if (d > 0.0 && adapt.speed.y > 0.0) { step = min(d, adapt.speed.y * adapt.speed.x); }
  if (d < 0.0 && adapt.speed.z > 0.0) { step = max(d, -adapt.speed.z * adapt.speed.x); }
  return vec4<f32>(prev + step, 0.0, 0.0, 1.0);
}
