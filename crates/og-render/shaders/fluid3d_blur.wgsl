// The 3D fluid's depth smoothing (fluid3d.wgsl): the narrow-range filter
// of Truong and Yuksel (I3D 2018) over the spheres' view depth, as
// matsuoka-601/Splash runs it (MIT, Copyright (c) 2025 matsuoka-601;
// revision 3df5d621e6f1, `render/narrowRangeFilter.wgsl`; see
// third_party/webgpu-ocean/). fluid3d.rs runs it four times one-dimensional
// (x, y, x, y) and twice two-dimensional over a small square.
//
// Its radius is the fluid's drawn particle radius times its smoothing as
// seen at that depth. A tap nearer than the range (another sheet in front)
// drops itself and its mirror; a tap farther than it, or empty, counts as
// a little behind the centre, so free surfaces and silhouettes round off
// instead of keeping the spheres' flat faces; the range widens with every
// tap taken, so one sloping surface is smoothed whole.

struct BlurUniform {
  dir: vec4<f32>,      // xy: the direction (one pixel), z: 1 orthographic, w: the largest radius (px)
  params: vec4<f32>,   // x: pixels per metre (at view depth 1 in perspective), y: the radius (m), z: the drawn particle radius (m), w: 1 two-dimensional
  screen: vec4<f32>,   // w, h, 1/w, 1/h
};

@group(0) @binding(0) var<uniform> blur: BlurUniform;
@group(0) @binding(1) var src: texture_2d<f32>;

const NONE: f32 = 1.0e8;
// The two-dimensional passes' half width, in pixels (upstream's).
const SQUARE: i32 = 2;

@vertex
fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  return fullscreen_clip(i);
}

fn load(px: vec2<i32>) -> f32 {
  let size = vec2<i32>(blur.screen.xy);
  return textureLoad(src, clamp(px, vec2<i32>(0), size - vec2<i32>(1)), 0).r;
}

// The narrow range: the bounds a tap must fall in, and what a farther tap counts as.
struct Range {
  low: f32,
  high: f32,
};

// One tap against the range: its depth and whether it counts, the range
// widened by it when it falls inside.
fn take(s: f32, range: ptr<function, Range>, threshold: f32, behind: f32) -> vec2<f32> {
  if (s < (*range).low) { return vec2<f32>(0.0, 0.0); }
  if (s > (*range).high) { return vec2<f32>(behind, 1.0); }
  (*range).low = min((*range).low, s - threshold);
  (*range).high = max((*range).high, s + threshold);
  return vec2<f32>(s, 1.0);
}

@fragment
fn fs(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
  let px = vec2<i32>(pos.xy);
  let d = load(px);
  if (d >= NONE) { return vec4<f32>(d, 0.0, 0.0, 0.0); }
  var radius = blur.params.x * blur.params.y;
  if (blur.dir.z < 0.5) { radius /= max(d, 1e-3); }
  let size = i32(ceil(clamp(radius, 0.0, blur.dir.w)));
  if (size < 1) { return vec4<f32>(d, 0.0, 0.0, 0.0); }
  let sigma = f32(size) * 0.5;
  let inv = 1.0 / (2.0 * sigma * sigma);
  // Upstream's constants, in its particle radius: a range of ten, and a
  // tap beyond it counted three behind.
  let r = max(blur.params.z, 1e-4);
  let threshold = 10.0 * r;
  let behind = d + 3.0 * r;
  var sum = d;
  var wsum = 1.0;
  if (blur.params.w < 0.5) {
    let step = vec2<i32>(blur.dir.xy);
    var lo = Range(d - threshold, d + threshold);
    var hi = Range(d - threshold, d + threshold);
    for (var i = 1; i <= size; i++) {
      let g = exp(-f32(i * i) * inv);
      let a = take(load(px - step * i), &lo, threshold, behind);
      let b = take(load(px + step * i), &hi, threshold, behind);
      // A tap in front on either side drops the pair, keeping it symmetric.
      let w = g * a.y * b.y;
      sum += (a.x + b.x) * w;
      wsum += 2.0 * w;
    }
  } else {
    var range = Range(d - threshold, d + threshold);
    for (var k = 1; k <= SQUARE; k++) {
      for (var i = 0; i < 2 * k; i++) {
        let g = exp((-f32(k * k) + f32((k - i) * (k - i))) * inv);
        let a = take(load(px - vec2<i32>(k, k - i)), &range, threshold, behind);
        let b = take(load(px + vec2<i32>(k, k - i)), &range, threshold, behind);
        let c = take(load(px - vec2<i32>(k - i, k)), &range, threshold, behind);
        let e = take(load(px + vec2<i32>(k - i, k)), &range, threshold, behind);
        let w0 = g * a.y * b.y;
        let w1 = g * c.y * e.y;
        sum += (a.x + b.x) * w0 + (c.x + e.x) * w1;
        wsum += 2.0 * (w0 + w1);
      }
    }
  }
  return vec4<f32>(sum / wsum, 0.0, 0.0, 0.0);
}
