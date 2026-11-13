// Screen-space contact shadows for the shadowed local lights (frame.rs
// `contact_lights`), ported from Bend Studio's screen-space shadows
// (Graham Aldridge, SIGGRAPH 2023; bend_sss_gpu.h, Copyright 2023 Sony
// Interactive Entertainment, Apache-2.0, third_party/bend-sss): one
// 64-thread workgroup per 64-pixel run of a line towards the light
// (`ComputeWavefrontExtents`), each thread reading the two depths of its
// pixel and of the pixel 64 further along, shared through workgroup
// memory, then every thread walking the next 60 of them
// (`WriteScreenSpaceShadow`). The dispatches that tile the screen in such
// runs come from contact.rs (`BuildDispatchList`), one list per light.
// FarDepthValue 0, NearDepthValue above it (z_sign -1), the depth 1 / view
// depth or an orthographic camera's affine reversed depth (contact.rs
// `light_coordinate`), BilinearSamplingOffsetMode false, IgnoreEdgePixels
// true, UsePrecisionOffset false; two changes: a sample counts as an edge
// also against the previous sample along the line (the shadow's rim
// speckled where a line grazed a caster's silhouette), and a pixel facing
// away from the light, or in the sky, is lit without walking
// (`EarlyOutPixel`). The runs of neighbouring lines converge towards the
// light, so a pixel near it lies on several (up to some hundred threads
// of a list write one pixel); each writes a key, `atomicMax`, so the line
// passing nearest the pixel's centre gives it its shadow whatever order
// the GPU runs them in (Bend's plain store kept whichever wrote last, and
// a frame differed from the same frame drawn again). Prepended by
// fxpost_uniform.wgsl.

// Bend's per-dispatch parameters (contact.rs `Dispatch`).
struct ContactDispatch {
  // Bend's LightCoordinate: xy the light's pixel coordinate, z its depth,
  // w +1 with the light in front of the camera, -1 behind.
  light: vec4<f32>,
  // Bend's WaveOffset: the dispatch's first run, in pixels from the light.
  wave_offset: vec2<i32>,
  // The light's row of `fxp.contact`.
  index: u32,
  // The frame's tag, 1 to 255 (contact.rs `ContactPasses::prepare`):
  // the top byte of every key, so a key of this frame outranks every
  // earlier one and the masks need no clearing between frames.
  frame: u32,
};

@group(0) @binding(0) var<uniform> fxp: FxPost;
@group(0) @binding(1) var<uniform> dispatch: ContactDispatch;
@group(0) @binding(2) var tex_depth: texture_2d<f32>;    // the prepass view depth
@group(0) @binding(3) var tex_gbuffer: texture_2d<f32>;  // oct normal, velocity
// The lights' masks, one screen of keys per light by its row (contact.rs
// `mask_buffer`): the frame's tag in the top byte, then how near the
// writing line passes to the pixel's centre (16 bits), then the mask.
@group(0) @binding(4) var<storage, read_write> masks: array<atomic<u32>>;
// The compose (`cs_contact_compose`): the AO and the lights' masks into
// the AO texture (post.rs `CONTACT_FORMAT`).
@group(0) @binding(5) var tex_ao: texture_2d<f32>;
@group(0) @binding(6) var<storage, read> mask_keys: array<u32>;
@group(0) @binding(9) var composed: texture_storage_2d<rgba8unorm, write>;

const WAVE_SIZE: i32 = 64;
// Bend's READ_COUNT for up to 126 samples (SAMPLE_COUNT / WAVE_SIZE + 2).
const READ_COUNT: i32 = 2;
const CONTACT_HARD_SAMPLES: i32 = 4;
const CONTACT_FADE_SAMPLES: i32 = 8;
// A sample on a depth edge: never a hit.
const EDGE_SKIP: f32 = 1e20;

// The prepass's view depth as Bend's depth: 1 / depth with a perspective
// camera (reversed-Z device depth to a scale, which Bend's fractions of it
// cancel), and an orthographic camera's affine reversed depth; 0 off the
// screen (Bend's border colour, the far plane).
fn contact_depth(px: vec2<i32>, size: vec2<i32>) -> f32 {
  if (any(px < vec2<i32>(0)) || any(px >= size)) { return 0.0; }
  let d = textureLoad(tex_depth, px, 0).r;
  if (fxp.cam.y > 0.5) { return max(1.0 - d / fxp.cam.x, 0.0); }
  return 1.0 / max(d, 1e-6);
}

// The view-space point at `uv` whose view depth is `d`.
fn view_pos(uv: vec2<f32>, d: f32) -> vec3<f32> {
  return view_point(fxp.inv_proj, uv, d);
}

// Whether the pixel at `px` takes no light from the contact light: off
// the screen, in the sky, or facing away from it (Bend's EarlyOutPixel).
fn unlit(px: vec2<i32>, size: vec2<i32>) -> bool {
  if (any(px < vec2<i32>(0)) || any(px >= size)) { return true; }
  let d = textureLoad(tex_depth, px, 0).r;
  if (d >= fxp.cam.x * 0.999) { return true; }
  let p = view_pos((vec2<f32>(px) + 0.5) / vec2<f32>(size), d);
  let n = normalize((fxp.view * vec4<f32>(oct_decode(textureLoad(tex_gbuffer, px, 0).xy), 0.0)).xyz);
  return dot(fxp.contact[dispatch.index].xyz - p, n) <= 0.0;
}

// The mask `value` at `px` from a thread whose line passes `near` (0 to
// 65535) to the pixel's centre: kept unless a line nearer it, or as near
// with a lighter value, wrote this frame. The value rounds to 8 bits as
// the rgba8unorm mask did.
fn put(px: vec2<i32>, size: vec2<i32>, near: u32, value: f32) {
  if (any(px < vec2<i32>(0)) || any(px >= size)) { return; }
  let key = (dispatch.frame << 24u) | (near << 8u) | u32(round(saturate(value) * 255.0));
  let at = (dispatch.index * u32(size.y) + u32(px.y)) * u32(size.x) + u32(px.x);
  atomicMax(&masks[at], key);
}

// Bend's DepthData: the depths along the run, perspective-corrected so
// every line's rays are parallel; and the raw depths, for the edge test
// against the previous sample.
var<workgroup> depth_data: array<f32, 128>;
var<workgroup> raw_depth: array<f32, 128>;

@compute @workgroup_size(64)
fn cs_contact(@builtin(workgroup_id) group_id: vec3<u32>, @builtin(local_invocation_index) tid: u32) {
  let size = vec2<i32>(textureDimensions(tex_depth));
  let params = fxp.contact[3];
  let thickness = params.x;
  let bilinear_threshold = params.y;
  let contrast = params.z;
  // At most the depths read past the thread's own (every thread's first
  // sample is the next pixel's).
  let samples = i32(params.w);
  let light = dispatch.light;
  let direction = -light.w;
  // ComputeWavefrontExtents: the run's first pixel, its distance from the
  // light along the major axis, and the step to the pixel 64 further.
  var xy = vec2<i32>(group_id.yz) * WAVE_SIZE + dispatch.wave_offset;
  let light_xy = floor(light.xy) + 0.5;
  let light_fraction = light.xy - light_xy;
  let reverse = light.w > 0.0;
  let sign_xy = sign(xy);
  let horizontal = abs(xy.x + sign_xy.y) < abs(xy.y - sign_xy.x);
  let axis = select(vec2<i32>(0, -sign_xy.x), vec2<i32>(sign_xy.y, 0), horizontal);
  xy = axis * i32(group_id.x) + xy;
  let xy_f = vec2<f32>(xy);
  let x_major = abs(xy_f.x) > abs(xy_f.y);
  let major_axis = select(xy_f.y, xy_f.x, x_major);
  let major_start = abs(major_axis);
  let major_end = abs(major_axis) - f32(WAVE_SIZE);
  var ma_light_frac = select(light_fraction.y, light_fraction.x, x_major);
  ma_light_frac = select(ma_light_frac, -ma_light_frac, major_axis > 0.0);
  let start_xy = xy_f + light_xy;
  // The innermost run interpolates to a pixel centre, so the rounding to
  // pixels skips none.
  let end_xy = mix(light.xy, start_xy, (major_end + ma_light_frac) / (major_start + ma_light_frac));
  let xy_delta = start_xy - end_xy;
  let thread_step = f32(tid ^ select(u32(WAVE_SIZE - 1), 0u, reverse));
  var pixel_xy = mix(start_xy, end_xy, thread_step / f32(WAVE_SIZE));
  let pixel_distance = major_start - thread_step + ma_light_frac;
  let write_xy = vec2<i32>(floor(pixel_xy));
  // How near the thread's point on its line lies to its pixel's centre.
  let off = pixel_xy - (floor(pixel_xy) + 0.5);
  let near = u32(round(saturate(1.0 - 2.0 * dot(off, off)) * 65535.0));

  // The two depths of each of the thread's pixels: its own and the one
  // bilinearly towards the line on the minor axis, which shadows from the
  // nearer of them unless they differ by an edge.
  var sampling_depth: array<f32, READ_COUNT>;
  var shadowing_depth: array<f32, READ_COUNT>;
  var sample_distance: array<f32, READ_COUNT>;
  for (var i = 0; i < READ_COUNT; i++) {
    let read_xy = vec2<i32>(floor(pixel_xy));
    let minor = select(pixel_xy.x, pixel_xy.y, x_major);
    let bias = select(-1, 1, fract(minor) - 0.5 > 0.0);
    let offset = select(vec2<i32>(bias, 0), vec2<i32>(0, bias), x_major);
    let dx = contact_depth(read_xy, size);
    let dy = contact_depth(read_xy + offset, size);
    let edge = abs(dx - dy) > dx * bilinear_threshold;
    sampling_depth[i] = dx;
    shadowing_depth[i] = select(dx - abs(dx - dy), EDGE_SKIP, edge);
    sample_distance[i] = pixel_distance + f32(WAVE_SIZE * i) * direction;
    pixel_xy += xy_delta * direction;
  }
  for (var i = 0; i < READ_COUNT; i++) {
    raw_depth[i * WAVE_SIZE + i32(tid)] = sampling_depth[i];
  }
  workgroupBarrier();
  for (var i = 0; i < READ_COUNT; i++) {
    let idx = i * WAVE_SIZE + i32(tid);
    // The sample one pixel back along the line (towards the receivers):
    // where a line grazes a caster's silhouette, successive samples step
    // through its depth faster than the slab, and testing them one by one
    // speckles the shadow's rim.
    let dx = sampling_depth[i];
    let prev = select(raw_depth[max(idx - 1, 0)], dx, idx == 0);
    var shadowing = shadowing_depth[i];
    if (abs(dx - prev) > dx * bilinear_threshold) { shadowing = EDGE_SKIP; }
    var stored = (shadowing - light.z) / sample_distance[i];
    // Past the light's own pixel (the extended reads near it): never a hit.
    if (i != 0 && sample_distance[i] <= 0.0) { stored = 1e10; }
    depth_data[idx] = stored;
  }
  workgroupBarrier();

  // No run reaches the light's own pixel (the innermost ends one pixel
  // short of it, contact.rs `dispatch_list_covers_the_screen`): the
  // innermost run's thread nearest it writes it lit.
  if (major_start <= f32(WAVE_SIZE) && thread_step == f32(WAVE_SIZE - 1)) {
    put(vec2<i32>(floor(light.xy)), size, 65535u, 1.0);
  }
  // A pixel within one of the light's: lit (Bend's +direction bias makes
  // it lit or shadowed whole).
  if (sample_distance[0] < 1.0 || unlit(write_xy, size)) {
    put(write_xy, size, near, 1.0);
    return;
  }
  let start_depth = sampling_depth[0];
  // WriteScreenSpaceShadow's window: the depth scaled so a slab of
  // `thickness` is 1, offset by one slab towards the camera.
  let depth_scale = min(sample_distance[0] + direction, 1.0 / thickness) * sample_distance[0] / start_depth;
  let start = (start_depth - light.z) / sample_distance[0] * depth_scale + 1.0;
  let first = i32(tid) + 1;
  var hard = 1.0;
  var shadow = vec4<f32>(1.0);
  for (var k = 0; k < CONTACT_HARD_SAMPLES; k++) {
    hard = min(hard, abs(start - depth_data[first + k] * depth_scale));
  }
  for (var k = CONTACT_HARD_SAMPLES; k < samples - CONTACT_FADE_SAMPLES; k++) {
    shadow[k & 3] = min(shadow[k & 3], abs(start - depth_data[first + k] * depth_scale));
  }
  for (var k = samples - CONTACT_FADE_SAMPLES; k < samples; k++) {
    let fade = f32(k + 1 - (samples - CONTACT_FADE_SAMPLES)) / f32(CONTACT_FADE_SAMPLES + 1) * 0.75;
    shadow[k & 3] = min(shadow[k & 3], abs(start - depth_data[first + k] * depth_scale) + fade);
  }
  shadow = saturate(shadow * contrast + (1.0 - contrast));
  hard = saturate(hard * contrast + (1.0 - contrast));
  put(write_xy, size, near, min(hard, dot(shadow, vec4<f32>(0.25))));
}

// The AO texture with the contact shadows: the AO (`tex_ao`, white
// without the AO passes) in r, the drawn lights' masks in gba.
@compute @workgroup_size(8, 8)
fn cs_contact_compose(@builtin(global_invocation_id) id: vec3<u32>) {
  let size = textureDimensions(composed);
  if (any(id.xy >= size)) { return; }
  let px = vec2<i32>(id.xy);
  let ao_size = vec2<i32>(textureDimensions(tex_ao));
  let ao = textureLoad(tex_ao, min(px, ao_size - 1), 0).r;
  var lit = vec3<f32>(1.0);
  let screen = size.x * size.y;
  let at = id.y * size.x + id.x;
  for (var i = 0u; i < 3u; i++) {
    // A pixel no run wrote this frame lies past the light's reach
    // (contact.rs `light_bounds`): lit.
    let key = mask_keys[i * screen + at];
    let tag = u32(fxp.contact[i].w);
    if (tag != 0u && key >> 24u == tag) { lit[i] = f32(key & 255u) / 255.0; }
  }
  textureStore(composed, px, vec4<f32>(ao, lit));
}
