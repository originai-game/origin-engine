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
  return vec3<f32>(c.x + c.y - c.z, c.x + c.z, c.x - c.y - c.z);
}

fn catmull_rom_weights(f: vec2<f32>) -> array<vec2<f32>, 4> {
  return array<vec2<f32>, 4>(
    f * (-0.5 + f * (1.0 - 0.5 * f)),
    1.0 + f * f * (-2.5 + 1.5 * f),
    f * (0.5 + f * (2.0 - 1.5 * f)),
    f * f * (-0.5 + 0.5 * f));
}

// Catmull-Rom over the 4x4 texels around uv in 9 bilinear taps (the
// middle two of each row and column merged into one tap; Jimenez,
// "Filmic SMAA", 2016). Negative lobes can ring: the caller clamps.
fn history_catmull_rom(uv: vec2<f32>) -> vec3<f32> {
  let size = vec2<f32>(textureDimensions(history));
  let p = uv * size;
  let t1 = floor(p - 0.5) + 0.5;
  let f = p - t1;
  let weights = catmull_rom_weights(f);
  let w0 = weights[0];
  let w1 = weights[1];
  let w2 = weights[2];
  let w3 = weights[3];
  let w12 = w1 + w2;
  let t0 = (t1 - 1.0) / size;
  let t3 = (t1 + 2.0) / size;
  let t12 = (t1 + w2 / w12) / size;
  var c = vec3<f32>(0.0);
  c += sample_exact(history, linear_sampler, vec2<f32>(t0.x, t0.y)).rgb * (w0.x * w0.y);
  c += sample_exact(history, linear_sampler, vec2<f32>(t12.x, t0.y)).rgb * (w12.x * w0.y);
  c += sample_exact(history, linear_sampler, vec2<f32>(t3.x, t0.y)).rgb * (w3.x * w0.y);
  c += sample_exact(history, linear_sampler, vec2<f32>(t0.x, t12.y)).rgb * (w0.x * w12.y);
  c += sample_exact(history, linear_sampler, vec2<f32>(t12.x, t12.y)).rgb * (w12.x * w12.y);
  c += sample_exact(history, linear_sampler, vec2<f32>(t3.x, t12.y)).rgb * (w3.x * w12.y);
  c += sample_exact(history, linear_sampler, vec2<f32>(t0.x, t3.y)).rgb * (w0.x * w3.y);
  c += sample_exact(history, linear_sampler, vec2<f32>(t12.x, t3.y)).rgb * (w12.x * w3.y);
  c += sample_exact(history, linear_sampler, vec2<f32>(t3.x, t3.y)).rgb * (w3.x * w3.y);
  return max(c, vec3<f32>(0.0));
}

// Float32 roundoff in inverse/project matrix products, not a scene-depth
// threshold. Shared by whole-footprint visibility and individual taps.
const DEPTH_ROUNDOFF: f32 = 32.0 * 1.1920929e-7;

fn saved_depth(at: vec2<i32>) -> f32 {
  let size = vec2<i32>(textureDimensions(previous_depth));
  let q = clamp(at, vec2<i32>(0), size - 1);
  return textureSampleLevel(previous_depth, depth_sampler, (vec2<f32>(q) + 0.5) / vec2<f32>(size), 0.0).r;
}

// Interpolate only taps on the reprojected surface. The closest current
// foreground supplies its depth even when jitter misses a thin feature
// at the colour sample; accepting any closer history would also retain
// an occluder that just moved away from that foreground.
fn history_on_surface(uv: vec2<f32>, expected_uv: vec2<f32>, expected: f32,
                      catmull_rom: bool, current_sample: vec4<f32>) -> vec4<f32> {
  let size = vec2<f32>(textureDimensions(history));
  let scene_size = vec2<f32>(textureDimensions(previous_depth));
  let p = uv * size - 0.5;
  let base = vec2<i32>(floor(p));
  let f = fract(p);
  let linear_weights = array<vec2<f32>, 4>(vec2<f32>(0.0), 1.0 - f, f, vec2<f32>(0.0));
  var weights = linear_weights;
  if (catmull_rom) { weights = catmull_rom_weights(f); }
  var colour = vec3<f32>(0.0);
  var total = 0.0;
  var alpha = 0.0;
  var alpha_total = 0.0;
  for (var y = 0; y < 4; y++) {
    for (var x = 0; x < 4; x++) {
      let weight = weights[x].x * weights[y].y;
      if (weight == 0.0) { continue; }
      let q = clamp(base + vec2<i32>(x - 1, y - 1), vec2<i32>(0), vec2<i32>(size) - 1);
      let at = vec2<i32>(floor((vec2<f32>(q) + 0.5) / size * scene_size + taa.previous_jitter.xy));
      let d = saved_depth(at);
      // A slanted surface changes depth across the footprint. Bound that
      // change by its local slope and the actual distance from the
      // reprojected scene sample. Opposite one-sided slopes mark an edge
      // or extremum, not a plane spanning two different surfaces.
      let left = vec2<f32>(d - saved_depth(at - vec2<i32>(1, 0)), d - saved_depth(at - vec2<i32>(0, 1)));
      let right = vec2<f32>(saved_depth(at + vec2<i32>(1, 0)) - d, saved_depth(at + vec2<i32>(0, 1)) - d);
      let slope = select(vec2<f32>(0.0), min(abs(left), abs(right)), left * right > vec2<f32>(0.0));
      let distance = abs(vec2<f32>(at) + 0.5 - (expected_uv * scene_size + taa.previous_jitter.xy));
      if (abs(d - expected) > dot(slope, distance) + DEPTH_ROUNDOFF) { continue; }
      let c = textureLoad(history, q, 0);
      colour += c.rgb * weight;
      total += weight;
      let a = linear_weights[x].x * linear_weights[y].y;
      alpha += c.a * a;
      alpha_total += a;
    }
  }
  // Catmull-Rom's outer lobes alone cannot reconstruct this pixel.
  if (total <= 0.0 || alpha_total <= 0.0) { return current_sample; }
  return vec4<f32>(max(colour / total, vec3<f32>(0.0)), alpha / alpha_total);
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
  let cur = cur4.rgb;

  // Neighbourhood colour box (YCoCg), and the alpha's range.
  var lo = vec3<f32>(1e9);
  var hi = vec3<f32>(-1e9);
  var a_lo = 1e9;
  var a_hi = -1e9;
  // An output pixel can lie on foreground geometry while its nearest
  // jittered scene sample falls beside it. Reproject that pixel with the
  // closest surface in the footprint, not the background's motion. Keep
  // colour sampling and its confidence at the original scene sample.
  var motion_at = at;
  var closest_depth = 0.0;
  if (upscale) {
    closest_depth = textureSampleLevel(depth, depth_sampler, (vec2<f32>(at) + 0.5) / vec2<f32>(size), 0.0).r;
  }
  for (var y = -1; y <= 1; y++) {
    for (var x = -1; x <= 1; x++) {
      let q = clamp(at + vec2<i32>(x, y), vec2<i32>(0), size - 1);
      let t = textureLoad(current, q, 0);
      let c = to_ycocg(t.rgb);
      lo = min(lo, c);
      hi = max(hi, c);
      a_lo = min(a_lo, t.a);
      a_hi = max(a_hi, t.a);
      if (upscale) {
        let d = textureSampleLevel(depth, depth_sampler, (vec2<f32>(q) + 0.5) / vec2<f32>(size), 0.0).r;
        if (d > closest_depth) {
          closest_depth = d;
          motion_at = q;
        }
      }
    }
  }

  // Motion: the camera's through the depth (none for a still camera: the
  // CPU compared the view-projections), or the prepass's velocity where
  // the prepass drew this sample (objects and skinned meshes move it).
  let here = in.pos.xy / out_size;
  let reprojected = camera_reprojection(at, size);
  var motion = reprojected.xy;
  var history_depth = reprojected.z;
  // Whether this sample moves otherwise than the camera moves it (an
  // object or a skinned mesh on its motion vectors).
  var own_motion = false;
  if (taa.still.y > 0.5 && !is_sky_depth(textureLoad(view_depth_tex, at, 0).r)) {
    let v = textureLoad(velocity_tex, at, 0).zw;
    own_motion = any(abs(v - motion) * out_size > vec2<f32>(0.5));
    motion = v;
    if (own_motion) { history_depth = textureLoad(motion_depth, at, 0).r; }
  }
  // Test the camera-reprojected background before borrowing a nearby
  // foreground's motion. Moving surfaces are tested per tap below with
  // the previous depth their prepass wrote beside that motion.
  // A foreground surface that covered this point last frame invalidates
  // its history even when that colour lies inside the current box.
  // Test the colour sample before borrowing a neighbour's motion: a
  // moving foreground in the footprint must not disable visibility
  // rejection for the camera-reprojected background it just uncovered.
  if (!smaa && !own_motion && was_occluded(here - motion, reprojected.z)) {
    // Seed with the surface whose visibility was tested. The first-frame
    // bilinear upscale would mix neighbouring surfaces into this sample's
    // new history, which later frames retain as if it belonged here.
    return cur4;
  }
  if (upscale && any(motion_at != at)) {
    let foreground = camera_reprojection(motion_at, size);
    motion = foreground.xy;
    history_depth = foreground.z;
    own_motion = false;
    if (taa.still.y > 0.5 && !is_sky_depth(textureLoad(view_depth_tex, motion_at, 0).r)) {
      let v = textureLoad(velocity_tex, motion_at, 0).zw;
      own_motion = any(abs(v - motion) * out_size > vec2<f32>(0.5));
      motion = v;
      if (own_motion) { history_depth = textureLoad(motion_depth, motion_at, 0).r; }
    }
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
      return fresh(in);
    }
    if (upscale && !smaa) {
      let sample_uv = (vec2<f32>(motion_at) + 0.5 - jitter) / vec2<f32>(size);
      hist4 = history_on_surface(prev_uv, sample_uv - motion, history_depth, catmull_rom, cur4);
    } else if (catmull_rom) {
      hist4 = vec4<f32>(history_catmull_rom(prev_uv), sample_exact(history, linear_sampler, prev_uv).a);
    } else {
      hist4 = sample_exact(history, linear_sampler, prev_uv);
    }
  }
  // Under TAAU a sample counts as much as it is near this pixel's centre
  // (a Gaussian of about half an output pixel).
  let near = select(1.0, exp(-2.0 * dot(off, off)), upscale);
  // The clamp trusts the neighbourhood only as far as it saw this pixel:
  // under TAAU a detail thinner than a scene pixel (a pole) can fall
  // between this frame's samples, and a box of what lies beside it would
  // erase it from the history. Far from a sample the clamped history is
  // eased back towards the history itself; a sample near the centre comes
  // round every few frames of the jitter and clamps it fully, so a stale
  // colour lasts no longer than that. Where the sample moves on its own
  // motion vectors (objects, skinned meshes) the clamp stays whole: their
  // edges are where a ghost would trail.
  // Nor where something moving on its own lies near: the background it
  // has just uncovered holds its colour in the history, which only the
  // whole clamp removes at once (eased, it showed through on alternate
  // jitter phases as a checker in the moving sphere's wake).
  var relax = select(0.0, 0.9 * (1.0 - near), upscale && !own_motion);
  if (relax > 0.0 && taa.still.y > 0.5 && moving_nearby(at, size, out_size)) {
    relax = 0.0;
  }
  let hist = from_ycocg(mix(clamp(to_ycocg(hist4.rgb), lo, hi), to_ycocg(hist4.rgb), relax));
  let hist_a = mix(clamp(hist4.a, a_lo, a_hi), hist4.a, relax);

  // A history over twice as bright as the brightest sample about it is no
  // detail the jitter moves but something gone (a bolt's last frame, a
  // light switched off), so it is dropped rather than clamped. The clamp
  // kept it at the level of the brightest neighbour, and where that was a
  // thin bright line (rain under a vanished bolt) the box's top was the
  // line's, leaving a bright band round each line for several frames.
  // Rejection by the history's distance from the box: Pedersen, "Temporal
  // Reprojection Anti-Aliasing in INSIDE", GDC 2016.
  let top = hi.x;
  let gone = clamp((to_ycocg(hist4.rgb).x - top) / max(top, 1e-4) - 1.0, 0.0, 1.0);

  if (smaa) { return mix(cur4, vec4<f32>(hist, hist_a), 0.5 * (1.0 - gone)); }
  // Without OIT the binding is a 1x1 white placeholder: the coordinate is
  // clamped to it, never loaded out of bounds (which may answer zero and
  // drop the history). With OIT `at` is the scene pixel, oit.reveal's size.
  let reactive_size = vec2<i32>(textureDimensions(reactive_tex));
  let reactive = textureLoad(reactive_tex, min(at, reactive_size - 1), 0);
  // The tail's transmittance times the front layer's (oit.rs REVEAL_FORMAT).
  let reveal = clamp(reactive.r * reactive.g, 0.0, 1.0);
  let a = mix(1.0 - (1.0 - taa.params.x * near) * reveal, 1.0, gone);
  // Compute anti-flicker weights after neighbourhood clipping, using
  // compressed luminance. Raw HDR weights erase bright subpixel coverage
  // against dark history; compression bounds that bias to 2:1. Blend the
  // original HDR colours below, not compressed or display-mapped colours.
  // The moving-sun regression's max/min energy falls from 1.665 to 1.226
  // (TAA), 1.898 to 1.249 (TAAU); T2x stays 1.138. This trades strong
  // firefly suppression for stable HDR energy: keep specular/taa-sharp
  // coverage when changing it, rather than restoring raw-luma weights.
  // See Tardif, Temporal Antialiasing Starter Pack, Resolve:
  // https://alextardif.com/TAA.html. Formula here compresses luma directly.
  let lc = max(dot(cur, LUMA), 0.0);
  let lh = max(dot(hist, LUMA), 0.0);
  let wc = a / (1.0 + lc / (1.0 + lc));
  let wh = (1.0 - a) / (1.0 + lh / (1.0 + lh));
  return vec4<f32>((cur * wc + hist * wh) / (wc + wh), (cur4.a * wc + hist_a * wh) / (wc + wh));
}

// xy: camera motion (uv/frame); z: this surface's previous window depth.
fn camera_reprojection(q: vec2<i32>, size: vec2<i32>) -> vec3<f32> {
  // Scene pixel q shows the jittered frame's clip position at its centre.
  let uv = (vec2<f32>(q) + 0.5) / vec2<f32>(size);
  let d = textureSampleLevel(depth, depth_sampler, uv, 0.0).r;
  if (taa.still.x > 0.5) { return vec3<f32>(0.0, 0.0, d); }
  let w = taa.inv_view_proj * vec4<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, d, 1.0);
  let world = w.xyz / w.w;
  let cur_clip = taa.cur_view_proj * vec4<f32>(world, 1.0);
  let prev_clip = taa.prev_view_proj * vec4<f32>(world, 1.0);
  let motion = (cur_clip.xy / cur_clip.w - prev_clip.xy / prev_clip.w) * vec2<f32>(0.5, -0.5);
  return vec3<f32>(motion, prev_clip.z / prev_clip.w);
}

fn camera_motion(q: vec2<i32>, size: vec2<i32>) -> vec2<f32> {
  if (taa.still.x > 0.5) { return vec2<f32>(0.0); }
  return camera_reprojection(q, size).xy;
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
      if (is_sky_depth(textureLoad(view_depth_tex, q, 0).r)) { continue; }
      let own = textureLoad(velocity_tex, q, 0).zw - camera_motion(q, size);
      let moving = any(abs(own) * out_size > vec2<f32>(0.5));
      let reach = 2.0 + 8.0 * length(own * vec2<f32>(size));
      if (moving && f32(rings[r]) <= reach) { return true; }
    }
  }
  return false;
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
