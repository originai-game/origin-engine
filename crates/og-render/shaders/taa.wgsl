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
// The order-independent transparency's revealage (oit.rs): how much of
// the opaque scene its blended surfaces let through. They have no depth
// or motion of their own, so under them the history is trusted less: its
// weight scaled by the revealage (a reactive mask; white without OIT).
@group(0) @binding(8) var reactive_tex: texture_2d<f32>;
// The previous frame's scene depth. Unlike colour clipping, visibility
// can distinguish an uncovered surface from the occluder in its history.
@group(0) @binding(9) var previous_depth: texture_2d<f32>;
// The previous clip depth of the same surface as the prepass's XY motion.
@group(0) @binding(10) var motion_depth: texture_2d<f32>;

fn to_ycocg(c: vec3<f32>) -> vec3<f32> {
  return vec3<f32>(
    0.25 * c.r + 0.5 * c.g + 0.25 * c.b,
    0.5 * c.r - 0.5 * c.b,
    -0.25 * c.r + 0.5 * c.g - 0.25 * c.b);
}

fn from_ycocg(c: vec3<f32>) -> vec3<f32> {
  return max(vec3<f32>(c.x + c.y - c.z, c.x + c.z, c.x - c.y - c.z), vec3<f32>(0.0));
}

// Reversible Karis tonemap in YCoCg so HDR fireflies do not inflate the
// neighbourhood variance box during temporal clipping.
fn tonemap_ycocg(c: vec3<f32>) -> vec3<f32> {
  let yc = to_ycocg(max(c, vec3<f32>(0.0)));
  return yc / (1.0 + max(yc.x, 0.0));
}

fn untonemap_ycocg(yc: vec3<f32>) -> vec3<f32> {
  let inv = 1.0 / max(1.0 - yc.x, 1e-4);
  return from_ycocg(yc * inv);
}

// Ray-AABB clip from `hist` towards `center` onto `[lo, hi]` (Pedersen,
// "Temporal Reprojection Anti-Aliasing in INSIDE", GDC 2016), avoiding the
// corner-sliding hue shift of axis-aligned clamp.
fn clip_aabb(lo: vec3<f32>, hi: vec3<f32>, center: vec3<f32>, hist: vec3<f32>) -> vec3<f32> {
  let p = 0.5 * (hi + lo);
  let e = 0.5 * (hi - lo) + vec3<f32>(1e-5);
  let v = hist - p;
  let u = abs(v / e);
  let m = max(u.x, max(u.y, u.z));
  if (m > 1.0) {
    return p + v / m;
  }
  return hist;
}

// 5-tap Catmull-Rom over the 4x4 texels around `uv` (Jimenez, "Filmic SMAA",
// 2016): drops the four outer corners (weight <= 1/64) and renormalises the
// five cross bilinear taps, returning RGB and alpha together.
fn history_catmull_rom(uv: vec2<f32>) -> vec4<f32> {
  let size = vec2<f32>(textureDimensions(history));
  let p = uv * size;
  let t1 = floor(p - 0.5) + 0.5;
  let f = p - t1;
  let f2 = f * f;
  let f3 = f2 * f;
  let w0 = f * (-0.5 + f * (1.0 - 0.5 * f));
  let w1 = 1.0 + f2 * (-2.5 + 1.5 * f);
  let w2 = f * (0.5 + f * (2.0 - 1.5 * f));
  let w3 = f2 * (-0.5 + 0.5 * f);
  let w12 = w1 + w2;
  let t0 = (t1 - 1.0) / size;
  let t3 = (t1 + 2.0) / size;
  let t12 = (t1 + w2 / w12) / size;
  let wt = w12.x * w0.y;
  let wl = w0.x * w12.y;
  let wc = w12.x * w12.y;
  let wr = w3.x * w12.y;
  let wb = w12.x * w3.y;
  var c = sample_exact(history, linear_sampler, vec2<f32>(t12.x, t0.y)) * wt;
  c += sample_exact(history, linear_sampler, vec2<f32>(t0.x, t12.y)) * wl;
  c += sample_exact(history, linear_sampler, vec2<f32>(t12.x, t12.y)) * wc;
  c += sample_exact(history, linear_sampler, vec2<f32>(t3.x, t12.y)) * wr;
  c += sample_exact(history, linear_sampler, vec2<f32>(t12.x, t3.y)) * wb;
  let norm = 1.0 / (wt + wl + wc + wr + wb);
  c *= norm;
  return vec4<f32>(max(c.rgb, vec3<f32>(0.0)), clamp(c.a, 0.0, 1.0));
}

// Float32 roundoff in inverse/project matrix products, not a scene-depth
// threshold.
const DEPTH_ROUNDOFF: f32 = 32.0 * 1.1920929e-7;

fn saved_depth(at: vec2<i32>) -> f32 {
  let size = vec2<i32>(textureDimensions(previous_depth));
  let q = clamp(at, vec2<i32>(0), size - 1);
  return textureSampleLevel(previous_depth, depth_sampler, (vec2<f32>(q) + 0.5) / vec2<f32>(size), 0.0).r;
}

// Blackman-Harris spatial window weight for squared distance `d2` (in scene
// pixels squared, support radius 1.5): smooth anti-aliased spatial fallback
// on disocclusion under TAAU without ringing.
fn blackman_harris(d2: f32) -> f32 {
  return exp(-2.29 * d2);
}

// A new history takes only the current frame, with the same jittered
// bilinear upscale as the full resolve. Its own entry point lets the
// first frame compile without reprojection or neighbourhood rejection.
fn fresh(in: VsOut) -> vec4<f32> {
  if (taa.upscale.z > 0.5) {
    let size = taa.params.yz;
    let s = in.pos.xy / taa.upscale.xy * size;
    return sample_exact(current, linear_sampler, (s + taa.still.zw) / size);
  }
  return textureLoad(current, vec2<i32>(in.pos.xy), 0);
}

@fragment
fn fs_fresh(in: VsOut) -> @location(0) vec4<f32> {
  return fresh(in);
}

@fragment
fn fs(in: VsOut) -> @location(0) vec4<f32> {
  return resolve(in, false, false);
}

@fragment
fn fs_catmull_rom(in: VsOut) -> @location(0) vec4<f32> {
  return resolve(in, true, false);
}

// SMAA T2x's two-frame mean (Jimenez et al., MIT:
// third_party/smaa/LICENSE.txt), using the engine's existing reprojection
// and neighbourhood rejection instead of storing velocity in opacity.
// Both inputs are raw SMAA frames; the result never feeds the history.
@fragment
fn fs_smaa(in: VsOut) -> @location(0) vec4<f32> {
  return resolve(in, false, true);
}

@fragment
fn fs_smaa_srgb(in: VsOut) -> @location(0) vec4<f32> {
  let c = resolve(in, false, true);
  return vec4<f32>(srgb_encode(c.rgb), c.a);
}

fn resolve(in: VsOut, catmull_rom: bool, smaa: bool) -> vec4<f32> {
  if (taa.params.x >= 1.0) { return fresh(in); }
  let size = vec2<i32>(taa.params.yz);
  let inv_size = 1.0 / vec2<f32>(size);
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
    off = (s - (vec2<f32>(at) + 0.5 - jitter)) * out_size * inv_size;
  }
  let cur4 = textureLoad(current, at, 0);
  let cur = cur4.rgb;
  let center_depth = textureSampleLevel(depth, depth_sampler, (vec2<f32>(at) + 0.5) * inv_size, 0.0).r;

  // Single 3x3 sweep computes:
  // 1. Reversible-tonemapped YCoCg moments (m1, m2) and min/max bounds for
  //    variance clipping (Salvi 2016) without HDR firefly inflation.
  // 2. Closest-depth texel (`motion_at`) for universal 3x3 velocity dilation
  //    in both 1x TAA and TAAU (reversed-Z: largest depth is closest).
  // 3. Depth-aware Blackman-Harris spatial reconstruction for crisp
  //    disocclusion fallback at zero extra texture fetch cost.
  var lo = vec3<f32>(1e9);
  var hi = vec3<f32>(-1e9);
  var m1 = vec3<f32>(0.0);
  var m2 = vec3<f32>(0.0);
  var a_lo = 1e9;
  var a_hi = -1e9;
  var motion_at = at;
  var closest_depth = center_depth;
  var spatial = vec4<f32>(0.0);
  var spatial_w = 0.0;
  let bh_center = s + jitter;
  for (var y = -1; y <= 1; y++) {
    for (var x = -1; x <= 1; x++) {
      let q = clamp(at + vec2<i32>(x, y), vec2<i32>(0), size - 1);
      let t = textureLoad(current, q, 0);
      let c = tonemap_ycocg(t.rgb);
      lo = min(lo, c);
      hi = max(hi, c);
      m1 += c;
      m2 += c * c;
      a_lo = min(a_lo, t.a);
      a_hi = max(a_hi, t.a);
      let d = select(
        textureSampleLevel(depth, depth_sampler, (vec2<f32>(q) + 0.5) * inv_size, 0.0).r,
        center_depth,
        x == 0 && y == 0,
      );
      if (d > closest_depth) {
        closest_depth = d;
        motion_at = q;
      }
      // Same-surface Blackman-Harris weight for disocclusion reconstruction.
      let delta = vec2<f32>(q) + 0.5 - bh_center;
      let d_diff = abs(d - center_depth);
      let surface_w = select(0.0, 1.0, d_diff <= max(center_depth * 0.05, DEPTH_ROUNDOFF * 4.0));
      let bw = blackman_harris(dot(delta, delta)) * select(surface_w, 1.0, x == 0 && y == 0);
      spatial += t * bw;
      spatial_w += bw;
    }
  }
  let fallback = select(cur4, spatial / max(spatial_w, 1e-5), upscale && spatial_w > 0.0);

  // Motion: camera reprojection through depth, or prepass velocity where the
  // prepass drew a moving/skinned surface. Check `at` first for visibility
  // disocclusion before dilating to the closest foreground's velocity.
  let here = in.pos.xy / out_size;
  let reprojected = camera_reprojection(at, size, center_depth);
  var motion = reprojected.xy;
  var own_motion = false;
  if (taa.still.y > 0.5 && !is_sky_depth(textureLoad(view_depth_tex, at, 0).r)) {
    let v = textureLoad(velocity_tex, at, 0).zw;
    own_motion = any(abs(v - motion) * out_size > vec2<f32>(0.5));
    motion = v;
  }
  let depth_valid = taa.previous_jitter.z > 0.5;
  if (!smaa && !own_motion && depth_valid && was_occluded(here - motion, reprojected.z)) {
    return fallback;
  }
  if (any(motion_at != at)) {
    let foreground = camera_reprojection(motion_at, size, closest_depth);
    var fg_motion = foreground.xy;
    var fg_own = false;
    if (taa.still.y > 0.5 && !is_sky_depth(textureLoad(view_depth_tex, motion_at, 0).r)) {
      let v = textureLoad(velocity_tex, motion_at, 0).zw;
      fg_own = any(abs(v - fg_motion) * out_size > vec2<f32>(0.5));
      fg_motion = v;
    }
    if (upscale || fg_own || any(abs(fg_motion - motion) * out_size > vec2<f32>(0.25))) {
      motion = fg_motion;
      own_motion = own_motion || fg_own;
    }
  }

  var hist4: vec4<f32>;
  let motion_px = abs(motion) * out_size;
  if (all(motion_px < vec2<f32>(1e-3))) {
    hist4 = textureLoad(history, px, 0);
  } else {
    let prev_uv = here - motion;
    if (any(prev_uv < vec2<f32>(0.0)) || any(prev_uv > vec2<f32>(1.0))) {
      return fallback;
    }
    if (catmull_rom) {
      hist4 = history_catmull_rom(prev_uv);
    } else {
      hist4 = sample_exact(history, linear_sampler, prev_uv);
    }
  }

  // Under TAAU a sample counts as much as it is near this pixel's centre
  // (a Gaussian of about half an output pixel).
  let near = select(1.0, exp(-2.0 * dot(off, off)), upscale);
  // Variance-guided YCoCg clipping box intersected with min/max bounds:
  // tighter (gamma = 1.0) under motion to suppress ghosting, slightly wider
  // (gamma = 1.25) when still to preserve subpixel detail.
  let mean = m1 * (1.0 / 9.0);
  let sigma = sqrt(max(m2 * (1.0 / 9.0) - mean * mean, vec3<f32>(0.0)));
  let speed = length(motion_px);
  let gamma = select(mix(1.25, 0.95, clamp(speed * 0.25, 0.0, 1.0)), 0.95, own_motion);
  let box_min = max(lo, mean - gamma * sigma);
  let box_max = min(hi, mean + gamma * sigma);

  let hist_tm = tonemap_ycocg(hist4.rgb);
  let clipped_tm = clip_aabb(box_min, box_max, mean, hist_tm);
  // Under static TAAU, ease clipping far from this frame's jitter sample so
  // subpixel thin geometry survives between jitter hits; disable relaxation
  // whenever the pixel or dilated foreground moves.
  var relax = select(0.0, 0.85 * (1.0 - near) * (1.0 - clamp(speed * 0.5, 0.0, 1.0)), upscale && !own_motion);
  // Background a moving object uncovered over the jitter cycle shows
  // through on alternate jitter phases as a checker in its wake unless
  // clipped: no relaxation near anything moving on its own.
  if (relax > 0.0 && taa.still.y > 0.5 && moving_nearby(at, size, out_size)) {
    relax = 0.0;
  }
  let hist = untonemap_ycocg(mix(clipped_tm, hist_tm, relax));
  let hist_a = mix(clamp(hist4.a, a_lo, a_hi), hist4.a, relax);

  // Drop stale bright transients (vanished flash/lightning/light off) whose
  // history luminance exceeds 2x the brightest current neighbour.
  let top = untonemap_ycocg(hi).r * 0.25 + untonemap_ycocg(hi).g * 0.5 + untonemap_ycocg(hi).b * 0.25;
  let gone = clamp((to_ycocg(hist4.rgb).x - top) / max(top, 1e-4) - 1.0, 0.0, 1.0);

  if (smaa) { return mix(cur4, vec4<f32>(hist, hist_a), 0.5 * (1.0 - gone)); }
  let reactive_size = vec2<i32>(textureDimensions(reactive_tex));
  let reactive = textureLoad(reactive_tex, min(at, reactive_size - 1), 0);
  let reveal = clamp(reactive.r * reactive.g, 0.0, 1.0);
  let a = mix(1.0 - (1.0 - taa.params.x * near) * reveal, 1.0, gone);
  // Compressed-luma anti-flicker weights (Tardif): preserve HDR specular
  // energy while keeping bright subpixel flicker bounded to 2:1.
  let lc = max(dot(cur, LUMA), 0.0);
  let lh = max(dot(hist, LUMA), 0.0);
  let wc = a / (1.0 + lc / (1.0 + lc));
  let wh = (1.0 - a) / (1.0 + lh / (1.0 + lh));
  return vec4<f32>((cur * wc + hist * wh) / (wc + wh), (cur4.a * wc + hist_a * wh) / (wc + wh));
}

// xy: camera motion (uv/frame); z: this surface's previous window depth.
fn camera_reprojection(q: vec2<i32>, size: vec2<i32>, d: f32) -> vec3<f32> {
  if (taa.still.x > 0.5) { return vec3<f32>(0.0, 0.0, d); }
  let uv = (vec2<f32>(q) + 0.5) / vec2<f32>(size);
  let w = taa.inv_view_proj * vec4<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, d, 1.0);
  let world = w.xyz / w.w;
  let cur_clip = taa.cur_view_proj * vec4<f32>(world, 1.0);
  let prev_clip = taa.prev_view_proj * vec4<f32>(world, 1.0);
  let motion = (cur_clip.xy / cur_clip.w - prev_clip.xy / prev_clip.w) * vec2<f32>(0.5, -0.5);
  return vec3<f32>(motion, prev_clip.z / prev_clip.w);
}

// Whether a sample moving on its own motion vectors (an object, a skinned
// mesh) lies near scene pixel `at`: within reach of the background it
// uncovered over the last jitter cycle (eight frames of its motion, and
// two pixels more), searched in rings of 1, 3, 7 and 15 scene pixels.
fn moving_nearby(at: vec2<i32>, size: vec2<i32>, out_size: vec2<f32>) -> bool {
  let dirs = array<vec2<i32>, 8>(
    vec2<i32>(1, 0), vec2<i32>(-1, 0), vec2<i32>(0, 1), vec2<i32>(0, -1),
    vec2<i32>(1, 1), vec2<i32>(-1, 1), vec2<i32>(1, -1), vec2<i32>(-1, -1));
  let rings = array<i32, 4>(1, 3, 7, 15);
  for (var r = 0; r < 4; r++) {
    for (var k = 0; k < 8; k++) {
      let q = clamp(at + dirs[k] * rings[r], vec2<i32>(0), size - 1);
      let d = textureLoad(view_depth_tex, q, 0).r;
      if (is_sky_depth(d)) { continue; }
      let own = textureLoad(velocity_tex, q, 0).zw - camera_reprojection(q, size, d).xy;
      let moving = any(abs(own) * out_size > vec2<f32>(0.5));
      let reach = 2.0 + 8.0 * length(own * vec2<f32>(size));
      if (moving && f32(rings[r]) <= reach) { return true; }
    }
  }
  return false;
}

fn was_occluded(uv: vec2<f32>, expected: f32) -> bool {
  let size = vec2<i32>(textureDimensions(previous_depth));
  // Colour history is unjittered; the saved scene depth is not. The
  // farthest of the four footprint taps is conservative at thin edges:
  // reject only when every tap had a closer surface, not merely one.
  let at = vec2<i32>(floor(uv * vec2<f32>(size) + taa.previous_jitter.xy - 0.5));
  var farthest = 1.0;
  for (var y = 0; y < 2; y++) {
    for (var x = 0; x < 2; x++) {
      let d = saved_depth(at + vec2<i32>(x, y));
      farthest = min(farthest, d);
    }
  }
  return expected < farthest - DEPTH_ROUNDOFF;
}

