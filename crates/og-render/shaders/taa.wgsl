// Temporal antialiasing and temporal upscaling (TAAU): the jittered frame
// blended into the history. The history is reprojected through the depth
// buffer with the unjittered matrices (camera motion), or along the
// prepass's velocity where the prepass drew the pixel (moving and skinned
// objects: `still.y`), rejected off the frame, clamped to the current 3x3
// neighbourhood's colour box, and blended with luminance weights so bright
// pixels do not flicker. Under TAAU (`upscale.z`) the scene is drawn
// smaller than the output and this pass writes the output-sized history:
// each output pixel takes the scene sample nearest its centre this frame
// (the jitter walks those samples over it), weighted by how near it lies,
// so the history converges to the full resolution over the jitter's
// phases. The history is read with one bilinear tap (fs) or a 9-tap
// Catmull-Rom (fs_catmull_rom, `TaaHistory::CatmullRom`), which keeps it
// sharp under motion; fs_sharpen is the optional sharpening after it. The
// alpha (the tone-mapped fraction of a pixel where untonemapped sprites
// lowered it, post.wgsl `tonemapped`) is blended with the colour's
// weights, the history's clamped to the neighbourhood's range as the
// colour is, so it follows a moving sprite without a trail. Prepended by
// fullscreen.wgsl.
struct Taa {
  // current frame (jittered) clip -> world
  inv_view_proj: mat4x4<f32>,
  // world -> current clip without jitter, and the previous frame's
  cur_view_proj: mat4x4<f32>,
  prev_view_proj: mat4x4<f32>,
  params: vec4<f32>,  // x: current frame weight (1 = no history), yz: the scene's size, w: sharpening (0..1)
  still: vec4<f32>,   // x: 1 when the camera is where the history's frame had it (decided on the CPU), y: 1 with the prepass's velocity bound, zw: the jitter (scene px, y down)
  upscale: vec4<f32>, // xy: the history's size (the output's under TAAU), z: 1 under TAAU
};
@group(0) @binding(0) var current: texture_2d<f32>;
@group(0) @binding(1) var history: texture_2d<f32>;
// The depth buffer (Depth32Float) read as an unfilterable float texture:
// GLSL cannot textureLoad a texture_depth_2d.
@group(0) @binding(2) var depth: texture_2d<f32>;
@group(0) @binding(3) var linear_sampler: sampler;
@group(0) @binding(4) var<uniform> taa: Taa;
// The depth is read through this nearest sampler at the pixel's centre,
// which reads the texel textureLoad reads.
@group(0) @binding(5) var depth_sampler: sampler;
// The prepass's gbuffer (zw: velocity, uv per frame) and view depth (r:
// far where the prepass drew nothing); placeholders without a prepass.
@group(0) @binding(6) var velocity_tex: texture_2d<f32>;
@group(0) @binding(7) var view_depth_tex: texture_2d<f32>;

fn to_ycocg(c: vec3<f32>) -> vec3<f32> {
  return vec3<f32>(
    0.25 * c.r + 0.5 * c.g + 0.25 * c.b,
    0.5 * c.r - 0.5 * c.b,
    -0.25 * c.r + 0.5 * c.g - 0.25 * c.b);
}

fn from_ycocg(c: vec3<f32>) -> vec3<f32> {
  return vec3<f32>(c.x + c.y - c.z, c.x + c.z, c.x - c.y - c.z);
}

// Catmull-Rom over the 4x4 texels around uv in 9 bilinear taps (the
// middle two of each row and column merged into one tap; Jimenez,
// "Filmic SMAA", 2016). Negative lobes can ring: the caller clamps.
fn history_catmull_rom(uv: vec2<f32>) -> vec3<f32> {
  let size = vec2<f32>(textureDimensions(history));
  let p = uv * size;
  let t1 = floor(p - 0.5) + 0.5;
  let f = p - t1;
  let w0 = f * (-0.5 + f * (1.0 - 0.5 * f));
  let w1 = 1.0 + f * f * (-2.5 + 1.5 * f);
  let w2 = f * (0.5 + f * (2.0 - 1.5 * f));
  let w3 = f * f * (-0.5 + 0.5 * f);
  let w12 = w1 + w2;
  let t0 = (t1 - 1.0) / size;
  let t3 = (t1 + 2.0) / size;
  let t12 = (t1 + w2 / w12) / size;
  var c = vec3<f32>(0.0);
  c += textureSampleLevel(history, linear_sampler, vec2<f32>(t0.x, t0.y), 0.0).rgb * (w0.x * w0.y);
  c += textureSampleLevel(history, linear_sampler, vec2<f32>(t12.x, t0.y), 0.0).rgb * (w12.x * w0.y);
  c += textureSampleLevel(history, linear_sampler, vec2<f32>(t3.x, t0.y), 0.0).rgb * (w3.x * w0.y);
  c += textureSampleLevel(history, linear_sampler, vec2<f32>(t0.x, t12.y), 0.0).rgb * (w0.x * w12.y);
  c += textureSampleLevel(history, linear_sampler, vec2<f32>(t12.x, t12.y), 0.0).rgb * (w12.x * w12.y);
  c += textureSampleLevel(history, linear_sampler, vec2<f32>(t3.x, t12.y), 0.0).rgb * (w3.x * w12.y);
  c += textureSampleLevel(history, linear_sampler, vec2<f32>(t0.x, t3.y), 0.0).rgb * (w0.x * w3.y);
  c += textureSampleLevel(history, linear_sampler, vec2<f32>(t12.x, t3.y), 0.0).rgb * (w12.x * w3.y);
  c += textureSampleLevel(history, linear_sampler, vec2<f32>(t3.x, t3.y), 0.0).rgb * (w3.x * w3.y);
  return max(c, vec3<f32>(0.0));
}

@fragment
fn fs(in: VsOut) -> @location(0) vec4<f32> {
  return resolve(in, false);
}

@fragment
fn fs_catmull_rom(in: VsOut) -> @location(0) vec4<f32> {
  return resolve(in, true);
}

fn resolve(in: VsOut, catmull_rom: bool) -> vec4<f32> {
  let size = vec2<i32>(taa.params.yz);
  let out_size = taa.upscale.xy;
  let upscale = taa.upscale.z > 0.5;
  let jitter = taa.still.zw;
  let px = vec2<i32>(in.pos.xy);
  // This pixel's place in the (unjittered) image, in scene pixels, and the
  // scene sample nearest it this frame: scene pixel k shows the image at
  // k + 0.5 - jitter. `off` is how far that sample lies, in output pixels.
  var at = px;
  var off = vec2<f32>(0.0);
  var s = in.pos.xy;
  if (upscale) {
    s = in.pos.xy / out_size * vec2<f32>(size);
    at = clamp(vec2<i32>(floor(s + jitter)), vec2<i32>(0), size - 1);
    off = (s - (vec2<f32>(at) + 0.5 - jitter)) * out_size / vec2<f32>(size);
  }
  let cur4 = textureLoad(current, at, 0);
  // The current frame where there is no history to blend with: under TAAU
  // filtered to the output (bilinear at the jittered place).
  var fresh = cur4;
  if (upscale) {
    fresh = textureSampleLevel(current, linear_sampler, (s + jitter) / vec2<f32>(size), 0.0);
  }
  let cur = cur4.rgb;
  if (taa.params.x >= 1.0) { return fresh; }

  // Neighbourhood colour box (YCoCg), and the alpha's range.
  var lo = vec3<f32>(1e9);
  var hi = vec3<f32>(-1e9);
  var a_lo = 1e9;
  var a_hi = -1e9;
  for (var y = -1; y <= 1; y++) {
    for (var x = -1; x <= 1; x++) {
      let q = clamp(at + vec2<i32>(x, y), vec2<i32>(0), size - 1);
      let t = textureLoad(current, q, 0);
      let c = to_ycocg(t.rgb);
      lo = min(lo, c);
      hi = max(hi, c);
      a_lo = min(a_lo, t.a);
      a_hi = max(a_hi, t.a);
    }
  }

  // Motion: the camera's through the depth (none for a still camera: the
  // CPU compared the view-projections), or the prepass's velocity where
  // the prepass drew this sample (objects and skinned meshes move it).
  let here = in.pos.xy / out_size;
  var motion = vec2<f32>(0.0);
  if (taa.still.x < 0.5) {
    // This pixel's world point, seen by the previous frame, from its
    // window position (exact on every backend).
    let uv_j = (s + select(vec2<f32>(0.0), jitter, upscale)) / vec2<f32>(size);
    let d = textureSampleLevel(depth, depth_sampler, (vec2<f32>(at) + 0.5) / vec2<f32>(size), 0.0).r;
    let ndc = vec2<f32>(uv_j.x * 2.0 - 1.0, 1.0 - uv_j.y * 2.0);
    let w = taa.inv_view_proj * vec4<f32>(ndc, d, 1.0);
    let world = w.xyz / w.w;
    let cur_clip = taa.cur_view_proj * vec4<f32>(world, 1.0);
    let prev_clip = taa.prev_view_proj * vec4<f32>(world, 1.0);
    motion = (cur_clip.xy / cur_clip.w - prev_clip.xy / prev_clip.w) * vec2<f32>(0.5, -0.5);
  }
  if (taa.still.y > 0.5 && textureLoad(view_depth_tex, at, 0).r < 1.0e8) {
    motion = textureLoad(velocity_tex, at, 0).zw;
  }
  var hist4: vec4<f32>;
  if (all(abs(motion) * out_size < vec2<f32>(1e-3))) {
    // Nothing moved under this pixel: its own history texel, unfiltered,
    // so nothing about the sampler or a backend's coordinates can move
    // the history from frame to frame.
    hist4 = textureLoad(history, px, 0);
  } else {
    let prev_uv = here - motion;
    if (any(prev_uv < vec2<f32>(0.0)) || any(prev_uv > vec2<f32>(1.0))) {
      // Disoccluded off the frame: no history.
      return fresh;
    }
    if (catmull_rom) {
      hist4 = vec4<f32>(history_catmull_rom(prev_uv), textureSampleLevel(history, linear_sampler, prev_uv, 0.0).a);
    } else {
      hist4 = textureSampleLevel(history, linear_sampler, prev_uv, 0.0);
    }
  }
  let hist = from_ycocg(clamp(to_ycocg(hist4.rgb), lo, hi));
  let hist_a = clamp(hist4.a, a_lo, a_hi);

  // Under TAAU a sample counts as much as it is near this pixel's centre
  // (a Gaussian of about half an output pixel).
  let a = taa.params.x * select(1.0, exp(-2.0 * dot(off, off)), upscale);
  let wc = a / (1.0 + dot(cur, LUMA));
  let wh = (1.0 - a) / (1.0 + dot(hist, LUMA));
  return vec4<f32>((cur * wc + hist * wh) / (wc + wh), (cur4.a * wc + hist_a * wh) / (wc + wh));
}

// The sharpening after TAA (`RenderDesc::taa_sharpen`, `current` bound to
// what TAA just wrote): an unsharp mask over the 4 neighbours, its
// strength params.w, clamped to their range so edges never overshoot. It
// writes its own target; the history keeps the unsharpened image.
@fragment
fn fs_sharpen(in: VsOut) -> @location(0) vec4<f32> {
  let size = vec2<i32>(taa.upscale.xy);
  let px = vec2<i32>(in.pos.xy);
  let c4 = textureLoad(current, px, 0);
  let c = c4.rgb;
  let n = textureLoad(current, clamp(px + vec2<i32>(0, -1), vec2<i32>(0), size - 1), 0).rgb;
  let s = textureLoad(current, clamp(px + vec2<i32>(0, 1), vec2<i32>(0), size - 1), 0).rgb;
  let e = textureLoad(current, clamp(px + vec2<i32>(1, 0), vec2<i32>(0), size - 1), 0).rgb;
  let w = textureLoad(current, clamp(px + vec2<i32>(-1, 0), vec2<i32>(0), size - 1), 0).rgb;
  let lo = min(c, min(min(n, s), min(e, w)));
  let hi = max(c, max(max(n, s), max(e, w)));
  let sharp = c + (c - (n + s + e + w) * 0.25) * taa.params.w;
  return vec4<f32>(clamp(sharp, lo, hi), c4.a);
}
