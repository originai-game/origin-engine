// FSR 3.1's prepare-inputs pass (ffx_fsr3upscaler_prepare_inputs.h; MIT,
// third_party/fidelityfx/LICENSE.txt): per render pixel the nearest depth
// of its 3x3 neighbourhood and that sample's motion (the dilated depth and
// motion), the farthest depth in metres, the frame's luma, and the nearest
// depth scattered to where it lay in the previous frame (the reconstructed
// previous depth). The motion is the engine's: the camera's through the
// depth, or the prepass's velocity where it drew the pixel, as taa.wgsl
// reads them, negated to FSR's convention (previous uv minus current).
// The previous depth is scattered with atomicMax on the depth's inverted
// bits (u32 storage atomics are core WebGPU; float atomics are not), so 0
// is "nothing reached here", which reads as the far plane (AMD clears it
// to 1.0). Prepended by fsr_common.wgsl.

@group(0) @binding(1) var color_tex: texture_2d<f32>;
@group(0) @binding(2) var depth_tex: texture_2d<f32>;
@group(0) @binding(3) var velocity_tex: texture_2d<f32>;
@group(0) @binding(4) var view_depth_tex: texture_2d<f32>;
@group(0) @binding(5) var dilated_depth_out: texture_storage_2d<r32float, write>;
@group(0) @binding(6) var dilated_motion_out: texture_storage_2d<rg32float, write>;
@group(0) @binding(7) var farthest_depth_out: texture_storage_2d<r32float, write>;
@group(0) @binding(8) var current_luma_out: texture_storage_2d<r32float, write>;
@group(0) @binding(9) var<storage, read_write> reconstructed_depth: array<atomic<u32>>;

fn load_input_depth(p: vec2<i32>) -> f32 {
  return textureLoad(depth_tex, clamp(p, vec2<i32>(0), render_size() - 1), 0).r;
}

// The camera's motion (uv per frame, current minus previous) of scene
// pixel `q`'s world point through its depth; none for a still camera.
fn camera_motion(q: vec2<i32>) -> vec2<f32> {
  if (fsr.extra.y > 0.5) { return vec2<f32>(0.0); }
  let uv = (vec2<f32>(q) + 0.5) / vec2<f32>(render_size());
  let d = textureLoad(depth_tex, q, 0).r;
  let w = fsr.inv_view_proj * vec4<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, d, 1.0);
  let world = w.xyz / w.w;
  let cur_clip = fsr.cur_view_proj * vec4<f32>(world, 1.0);
  let prev_clip = fsr.prev_view_proj * vec4<f32>(world, 1.0);
  return (cur_clip.xy / cur_clip.w - prev_clip.xy / prev_clip.w) * vec2<f32>(0.5, -0.5);
}

// LoadInputMotionVector at render resolution (the engine's motion vectors
// are the render's: FSR's LOW_RESOLUTION_MOTION_VECTORS), in FSR's sign.
fn load_input_motion_vector(q: vec2<i32>) -> vec2<f32> {
  var motion = camera_motion(q);
  if (fsr.extra.z > 0.5) {
    if (textureLoad(view_depth_tex, q, 0).r < 1.0e8) {
      motion = textureLoad(velocity_tex, q, 0).zw;
    }
  }
  return -motion;
}

fn store_reconstructed_depth(p: vec2<i32>, depth: f32) {
  let i = u32(p.y) * u32(max_render_size().x) + u32(p.x);
  atomicMax(&reconstructed_depth[i], ~bitcast<u32>(depth));
}

fn scatter_depth(b: BilinearSamplingData, k: i32, offset: vec2<i32>, depth: f32) {
  if (b.weights[k] > RECONSTRUCTED_DEPTH_BILINEAR_WEIGHT_THRESHOLD) {
    let store = b.base + offset;
    if (is_on_screen(store, render_size())) {
      store_reconstructed_depth(store, depth);
    }
  }
}

fn reconstruct_prev_depth(p: vec2<i32>, depth: f32, mv_in: vec2<f32>) {
  let nearest_m = min(view_depth(depth), FP16_MAX);
  let threshold = reconstructed_depth_mv_px_threshold(nearest_m);
  // Discard small motion.
  let mv = mv_in * f32(get_4k_velocity(mv_in) > threshold);
  let uv = (vec2<f32>(p) + 0.5) / vec2<f32>(render_size());
  let b = get_bilinear_sampling_data(uv + mv, render_size());
  // The four bilinear taps written out (naga's guarded loops are not
  // unrolled on Metal).
  scatter_depth(b, 0, vec2<i32>(0, 0), depth);
  scatter_depth(b, 1, vec2<i32>(1, 0), depth);
  scatter_depth(b, 2, vec2<i32>(0, 1), depth);
  scatter_depth(b, 3, vec2<i32>(1, 1), depth);
}

struct Extents {
  nearest: f32,
  farthest: f32,
  nearest_coord: vec2<i32>,
};

// One neighbour of FindDepthExtents' walk: the nearest (smallest: the
// engine's depth is not reversed) of the samples on screen, and the
// farthest of those nearer than the centre as the walk finds them.
fn extents_tap(e: ptr<function, Extents>, p: vec2<i32>, offset: vec2<i32>) {
  let q = p + offset;
  if (is_on_screen(q, render_size())) {
    let d = load_input_depth(q);
    if (d < (*e).nearest) {
      (*e).farthest = max((*e).farthest, d);
      (*e).nearest_coord = q;
      (*e).nearest = d;
    }
  }
}

@compute @workgroup_size(8, 8, 1)
fn cs(@builtin(global_invocation_id) id: vec3<u32>) {
  let p = vec2<i32>(id.xy);
  if (!is_on_screen(p, render_size())) { return; }

  // FindDepthExtents, in AMD's order of the 3x3.
  let centre = load_input_depth(p);
  var e = Extents(centre, centre, p);
  extents_tap(&e, p, vec2<i32>(1, 0));
  extents_tap(&e, p, vec2<i32>(0, 1));
  extents_tap(&e, p, vec2<i32>(0, -1));
  extents_tap(&e, p, vec2<i32>(-1, 0));
  extents_tap(&e, p, vec2<i32>(-1, 1));
  extents_tap(&e, p, vec2<i32>(1, 1));
  extents_tap(&e, p, vec2<i32>(-1, -1));
  extents_tap(&e, p, vec2<i32>(1, -1));
  let nearest = e.nearest;
  let farthest = e.farthest;
  let nearest_coord = e.nearest_coord;

  let mv = load_input_motion_vector(nearest_coord);
  reconstruct_prev_depth(p, nearest, mv);

  textureStore(dilated_motion_out, p, vec4<f32>(mv, 0.0, 0.0));
  textureStore(dilated_depth_out, p, vec4<f32>(nearest, 0.0, 0.0, 0.0));
  let farthest_m = min(view_depth(farthest), FP16_MAX);
  textureStore(farthest_depth_out, p, vec4<f32>(farthest_m, 0.0, 0.0, 0.0));
  let rgb = max(vec3<f32>(0.0), textureLoad(color_tex, p, 0).rgb);
  textureStore(current_luma_out, p, vec4<f32>(rgb_to_luma(rgb), 0.0, 0.0, 0.0));
}
