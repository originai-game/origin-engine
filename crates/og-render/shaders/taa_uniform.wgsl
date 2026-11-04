// Shared by the temporal resolve and SMAA's subsample-weight selection.
struct Taa {
  // current frame (jittered) clip -> world
  inv_view_proj: mat4x4<f32>,
  // world -> current clip without jitter, and the previous frame's
  cur_view_proj: mat4x4<f32>,
  prev_view_proj: mat4x4<f32>,
  params: vec4<f32>,  // x: current frame weight (1 = no history), yz: the scene's size, w: sharpening (0..1)
  still: vec4<f32>,   // x: 1 when the camera is where the history's frame had it (decided on the CPU), y: 1 with the prepass's velocity bound, zw: the jitter (scene px, y down)
  upscale: vec4<f32>, // xy: the history's size (the output's under TAAU), z: 1 under TAAU, w: SMAA area layer (0 for 1x, 1/2 for T2x)
};
