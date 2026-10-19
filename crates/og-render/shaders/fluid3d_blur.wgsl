// The 3D fluid's depth smoothing (fluid3d.wgsl): one direction of a
// separable bilateral filter over the spheres' view depth, its radius the
// fluid's drawn particle radius times its smoothing as seen at that depth,
// its depth range the same radius, so the spheres melt into one surface
// while separate sheets and the silhouette stay apart. Pixels with no
// fluid stay empty, and an empty tap counts as the pixel's own depth.

struct BlurUniform {
  dir: vec4<f32>,      // xy: the direction (one pixel), z: 1 orthographic, w: the largest radius (px)
  params: vec4<f32>,   // x: pixels per metre (at view depth 1 in perspective), y: the radius (m), z: the depth range (m)
  screen: vec4<f32>,   // w, h, 1/w, 1/h
};

@group(0) @binding(0) var<uniform> blur: BlurUniform;
@group(0) @binding(1) var src: texture_2d<f32>;

const NONE: f32 = 1.0e8;
const TAPS: i32 = 12;

@vertex
fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  let x = f32(i32(i & 1u) * 4 - 1);
  let y = f32(i32(i & 2u) * 2 - 1);
  return vec4<f32>(x, y, 0.0, 1.0);
}

@fragment
fn fs(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
  let px = vec2<i32>(pos.xy);
  let size = vec2<i32>(blur.screen.xy);
  let d = textureLoad(src, px, 0).r;
  if (d >= NONE) { return vec4<f32>(d, 0.0, 0.0, 0.0); }
  var radius = blur.params.x * blur.params.y;
  if (blur.dir.z < 0.5) { radius /= max(d, 1e-3); }
  radius = clamp(radius, 0.0, blur.dir.w);
  if (radius < 1.0) { return vec4<f32>(d, 0.0, 0.0, 0.0); }
  let step = radius / f32(TAPS);
  let sigma = radius * 0.5;
  let range = max(blur.params.z, 1e-4);
  var sum = 0.0;
  var weight = 0.0;
  for (var i = -TAPS; i <= TAPS; i++) {
    let off = f32(i) * step;
    let q = clamp(px + vec2<i32>(round(blur.dir.xy * off)), vec2<i32>(0), size - vec2<i32>(1));
    // An empty tap counts as the centre, so the silhouette stays put.
    var s = textureLoad(src, q, 0).r;
    if (s >= NONE) { s = d; }
    let dz = (s - d) / range;
    let w = exp(-off * off / (2.0 * sigma * sigma) - dz * dz);
    sum += s * w;
    weight += w;
  }
  return vec4<f32>(sum / max(weight, 1e-6), 0.0, 0.0, 0.0);
}
