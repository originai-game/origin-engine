// The stylisation passes over the tonemapped image (stylise.rs, the IDL's
// Stylise): after the tonemap, at the logical resolution, each a fullscreen
// pass from one LDR target into the next (or the output, encoding to sRGB
// in the shader where the output is not an sRGB format):
//
//   paint     painterly smoothing (a Kuwahara filter, its sectors turned
//             along the image's edges by `anisotropy`), pigment pooling at
//             the edges, colour bleeding, a canvas laid over the frame or,
//             in surface or object space, on the surfaces;
//   lines     cross-hatching from the tone (and, with `hatchingDepth`, the
//             prepass's contours), halftone dots on a rotated grid (both
//             over the frame or, in surface or object space, on the
//             surfaces), ink lines on luminance edges and, with
//             `inkDepth`, on the depth-normal prepass's silhouettes and
//             creases;
//   quantize  posterised channels and/or the nearest palette colour, mixed
//             by an ordered (Bayer), blue-noise or interleaved-gradient
//             dither;
//   upscale   the logical frame scaled up by a whole number with nearest
//             filtering, centred in the output with the background round it.
//
// Prepended by fullscreen.wgsl. Every pass binds the same layout.
struct Stylise {
  screen: vec4<f32>,          // logical w, h, 1/w, 1/h
  output: vec4<f32>,          // output w, h, the whole-number upscale, 0
  offset: vec4<f32>,          // xy: where the scaled frame starts in the output, z: 1 with a canvas, w: palette colours
  background: vec4<f32>,      // the display colour round the frame (linear)
  quantize: vec4<f32>,        // x: tones per channel (0: none), y: dither (0 none, 2/4/8 Bayer, -1 interleaved gradient, -2 blue noise), z: strength in steps, w: cell size
  halftone: vec4<f32>,        // x: strength, y: pitch, z: angle (rad)
  halftone_color: vec4<f32>,
  hatching: vec4<f32>,        // x: strength, y: spacing, z: levels, w: angle (rad)
  hatching_color: vec4<f32>,
  paint: vec4<f32>,           // x: radius, y: anisotropy, z: edge darkening, w: bleed
  canvas: vec4<f32>,          // x: strength, y: texels per logical pixel
  ink: vec4<f32>,             // x: 1 on, y: threshold, z: width, w: depth edges' strength (0: luminance only)
  ink_color: vec4<f32>,
  ink_depth: vec4<f32>,       // x: the camera's far plane (sky), y: relative depth step, z: normal turn (1 - cos), w: hatching from depth edges (0: off)
  inv_proj: mat4x4<f32>,      // view space from clip space
  view_rotation: mat4x4<f32>, // world from view space, no translation
  eye: vec4<f32>,             // xyz: the camera from the surface origin (a 1024 m grid point)
  surface: vec4<f32>,         // where the canvas (x), hatching (z) and halftone (w) lie: 0 screen, 1 world, 2 object space; y: canvas texels per metre
  canvas_origin: vec4<f32>,   // the canvas uv at the surface origin: x along x, y along y, z along z as u, w along z as v
};
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var clamp_sampler: sampler;
@group(0) @binding(2) var<uniform> st: Stylise;
@group(0) @binding(3) var palette: texture_2d<f32>;   // 256 x 1 sRGB colours, linear when sampled
@group(0) @binding(4) var canvas: texture_2d<f32>;
@group(0) @binding(5) var repeat_sampler: sampler;
@group(0) @binding(6) var blue_noise: texture_2d<f32>;  // 64 x 64 thresholds (blue_noise.rs), R8
@group(0) @binding(7) var pre_depth: texture_2d<f32>;   // the prepass view depth (R32Float); far without a prepass
@group(0) @binding(8) var pre_normal: texture_2d<f32>;  // the prepass's oct-encoded world normal (xy)
@group(0) @binding(9) var pre_object: texture_2d<u32>;  // the object pass (stylise_object.wgsl): position bits, packed normal; 0 without

fn srgb_encode(c: vec3<f32>) -> vec3<f32> {
  let lo = c * 12.92;
  let hi = 1.055 * pow(max(c, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.4)) - 0.055;
  return select(hi, lo, c <= vec3<f32>(0.0031308));
}

fn srgb_decode(c: vec3<f32>) -> vec3<f32> {
  let lo = c / 12.92;
  let hi = pow((c + 0.055) / 1.055, vec3<f32>(2.4));
  return select(hi, lo, c <= vec3<f32>(0.04045));
}

fn load(px: vec2<i32>) -> vec3<f32> {
  let size = vec2<i32>(textureDimensions(source));
  return textureLoad(source, clamp(px, vec2<i32>(0), size - 1), 0).rgb;
}

fn luma(c: vec3<f32>) -> f32 {
  return dot(c, LUMA);
}

// The tone as displayed (sRGB-encoded luminance), which the print looks
// quantise and stroke.
fn tone(c: vec3<f32>) -> f32 {
  return luma(srgb_encode(c));
}

// --- surfaces -------------------------------------------------------------

fn view_depth(px: vec2<i32>) -> f32 {
  let size = vec2<i32>(textureDimensions(pre_depth));
  return textureLoad(pre_depth, clamp(px, vec2<i32>(0), size - 1), 0).r;
}

fn oct_decode(e: vec2<f32>) -> vec3<f32> {
  var n = vec3<f32>(e, 1.0 - abs(e.x) - abs(e.y));
  if (n.z < 0.0) {
    n = vec3<f32>((1.0 - abs(n.yx)) * select(vec2<f32>(-1.0), vec2<f32>(1.0), n.xy >= vec2<f32>(0.0)), n.z);
  }
  return normalize(n);
}

fn normal_at(px: vec2<i32>) -> vec3<f32> {
  let size = vec2<i32>(textureDimensions(pre_normal));
  return oct_decode(textureLoad(pre_normal, clamp(px, vec2<i32>(0), size - 1), 0).xy);
}

// The point at view depth `d` under the logical pixel position `p`, from
// the surface origin (fxpost.wgsl view_pos, turned into the world).
fn surface_pos(p: vec2<f32>, d: f32) -> vec3<f32> {
  let uv = p * st.screen.zw;
  let ndc = vec2<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0);
  let a = st.inv_proj * vec4<f32>(ndc, 0.0, 1.0);
  let b = st.inv_proj * vec4<f32>(ndc, 1.0, 1.0);
  let p0 = a.xyz / a.w;
  let p1 = b.xyz / b.w;
  let t = (d + p0.z) / (p0.z - p1.z);
  return st.eye.xyz + (st.view_rotation * vec4<f32>(mix(p0, p1, t), 0.0)).xyz;
}

// How the surface moves under one logical pixel step along screen axis `e`
// from `p` (at depth `d`, point `at`): xyz the step towards +e in metres,
// taken to the neighbour on that axis whose depth is nearer `d` (w: +1 or
// -1, which side), each neighbour rebuilt at its own depth, so a
// silhouette on one side does not count and a plane receding along the
// axis gives its foreshortened step.
fn surface_step(p: vec2<f32>, d: f32, at: vec3<f32>, e: vec2<f32>) -> vec4<f32> {
  let dp = view_depth(vec2<i32>(p + e));
  let dm = view_depth(vec2<i32>(p - e));
  let plus = abs(dp - d) <= abs(dm - d);
  let side = select(-1.0, 1.0, plus);
  let q = surface_pos(p + e * side, select(dm, dp, plus));
  return vec4<f32>((q - at) * side, side);
}

// `v` no longer than `most`.
fn at_most(v: vec3<f32>, most: f32) -> vec3<f32> {
  return v * min(1.0, most / max(length(v), 1e-12));
}

// The largest stretch of a pixel's footprint over its width across the
// ray: a floor seen at a grazing angle is cut off there, so its far rows
// keep the coarsest octave rather than an unbounded one.
const FOOTPRINT_STRETCH: f32 = 32.0;

// What the patterns in surface space read at a pixel: its point from the
// surface origin, its world normal, how wide one logical pixel's footprint
// is there (metres) and how the point moves per pixel across and down.
//
// The footprint is measured along the depth gradient: the longer of the
// steps to the neighbours across and down, each at its own depth (a mip
// level's footprint), never less than the pixel's width across the ray
// nor more than FOOTPRINT_STRETCH times it. It picks the octave, so the
// pattern never runs finer than its pixel spacing in any direction:
// measured across the pixel alone, a long receding floor got an octave
// too fine along its depth, whose sub-pixel dots and strokes aliased into
// faint bands as the octaves faded (owed row 55). The steps `dx` and `dy`
// give the marks' edges and widths in the screen's own directions, so the
// coarser octave stays crisp across the floor instead of smeared by the
// footprint along it. `ok` is false on the sky, which keeps the screen
// pattern.
struct Surf {
  p: vec3<f32>,
  n: vec3<f32>,
  pixel: f32,
  dx: vec3<f32>,
  dy: vec3<f32>,
  // Which neighbours dx and dy were taken to (+1 or -1 along x and y).
  side: vec2<f32>,
  ok: bool,
  // p and n are in the instance's own space (object_at).
  obj: bool,
};

fn surface_at(p: vec2<f32>) -> Surf {
  let px = vec2<i32>(p);
  let d = view_depth(px);
  var s: Surf;
  s.ok = d > 0.0 && d < st.ink_depth.x * 0.999;
  s.p = surface_pos(p, d);
  let across = max(length(surface_pos(p + vec2<f32>(1.0, 0.0), d) - s.p), 1e-5);
  let sx = surface_step(p, d, s.p, vec2<f32>(1.0, 0.0));
  let sy = surface_step(p, d, s.p, vec2<f32>(0.0, 1.0));
  s.pixel = clamp(max(length(sx.xyz), length(sy.xyz)), across, across * FOOTPRINT_STRETCH);
  s.dx = at_most(sx.xyz, s.pixel);
  s.dy = at_most(sy.xyz, s.pixel);
  s.side = vec2<f32>(sx.w, sy.w);
  s.n = normal_at(px);
  s.obj = false;
  return s;
}

// The object pass's texel at `p`: xyz the point's bits, w its normal
// with bit 0 set (0: no instance there).
fn object_texel(p: vec2<f32>) -> vec4<u32> {
  let size = vec2<i32>(textureDimensions(pre_object));
  return textureLoad(pre_object, clamp(vec2<i32>(p), vec2<i32>(0), size - 1), 0);
}

// The object-space step to the neighbour `side` along `e` (towards +e),
// where that neighbour is on the same instance: its length within a
// factor of two of the world step's (an instance keeps metres along its
// axes). Else the world step's length along both of the plane's axes is
// all that is known: (0, 0, 0), and object_at falls back to an isotropic
// footprint.
fn object_step(p: vec2<f32>, at: vec3<f32>, e: vec2<f32>, side: f32, world: vec3<f32>) -> vec3<f32> {
  let o = object_texel(p + e * side);
  if ((o.w & 1u) == 0u) { return vec3<f32>(0.0); }
  let v = (bitcast<vec3<f32>>(o.xyz) - at) * side;
  let l = length(v);
  let w = length(world);
  if (l < w * 0.5 || l > w * 2.0) { return vec3<f32>(0.0); }
  return v;
}

// The surface in object space: the point and normal the object pass wrote
// for the instance covering the pixel (metres along its axes, so the
// world footprint `pixel` holds), or `world` where none does (terrain,
// scattered copies, a material shader's moved vertices). The steps are
// the instance's own between neighbouring texels on it; where a neighbour
// is off it (an edge), the footprint's square: `pixel` along every axis.
fn object_at(p: vec2<f32>, world: Surf) -> Surf {
  let o = object_texel(p);
  if ((o.w & 1u) == 0u || !world.ok) { return world; }
  var s = world;
  s.p = bitcast<vec3<f32>>(o.xyz);
  let e = vec2<f32>(f32((o.w >> 1u) & 0x7fffu), f32((o.w >> 16u) & 0x7fffu)) / 32767.0;
  s.n = oct_decode(e * 2.0 - 1.0);
  s.obj = true;
  s.dx = object_step(p, s.p, vec2<f32>(1.0, 0.0), world.side.x, world.dx);
  s.dy = object_step(p, s.p, vec2<f32>(0.0, 1.0), world.side.y, world.dy);
  if (dot(s.dx, s.dx) == 0.0 || dot(s.dy, s.dy) == 0.0) {
    s.dx = vec3<f32>(0.0);
    s.dy = vec3<f32>(0.0);
  } else {
    s.dx = at_most(s.dx, s.pixel);
    s.dy = at_most(s.dy, s.pixel);
  }
  return s;
}

// How a plane's coordinates move per pixel across (xy) and down (zw):
// the surface's steps seen on the plane facing axis `a`, or `pixel` along
// each of the plane's axes where the steps are unknown (zero).
fn plane_steps(s: Surf, a: i32) -> vec4<f32> {
  if (dot(s.dx, s.dx) == 0.0) {
    return vec4<f32>(s.pixel, 0.0, 0.0, s.pixel);
  }
  return vec4<f32>(plane_uv(s.dx, a), plane_uv(s.dy, a));
}

// How far a pattern coordinate `d = dot(q, dir)` moves over one screen
// pixel, at the steepest: a mark one pixel wide in `d` is this wide.
fn screen_width(dir: vec2<f32>, j: vec4<f32>) -> f32 {
  return max(length(vec2<f32>(dot(dir, j.xy), dot(dir, j.zw))), 1e-12);
}

// The triplanar weights of a normal: the planes facing x, y and z.
fn tri_weights(n: vec3<f32>) -> vec3<f32> {
  let w = pow(abs(n), vec3<f32>(4.0));
  return w / max(w.x + w.y + w.z, 1e-6);
}

// A point's coordinates on the plane facing axis `a` (0 x, 1 y, 2 z).
fn plane_uv(p: vec3<f32>, a: i32) -> vec2<f32> {
  if (a == 0) { return p.zy; }
  if (a == 1) { return p.xz; }
  return p.xy;
}

// The octaves a pattern `pixels` logical pixels wide is drawn at on a
// surface: (the finer spacing in metres, a power of two, and how far
// towards the next coarser one, 0 to 1). A tonal art map's levels: the
// pattern keeps about its screen size while it stays on the surface.
fn octave(pixels: f32, pixel: f32) -> vec2<f32> {
  let l = clamp(log2(pixels * pixel), -16.0, 10.0);
  let lo = floor(l);
  return vec2<f32>(exp2(lo), l - lo);
}

// --- paint ---------------------------------------------------------------

// The image gradient at `px` (Sobel over the encoded luminance).
fn gradient(px: vec2<i32>) -> vec2<f32> {
  let tl = tone(load(px + vec2<i32>(-1, -1)));
  let t = tone(load(px + vec2<i32>(0, -1)));
  let tr = tone(load(px + vec2<i32>(1, -1)));
  let l = tone(load(px + vec2<i32>(-1, 0)));
  let r = tone(load(px + vec2<i32>(1, 0)));
  let bl = tone(load(px + vec2<i32>(-1, 1)));
  let b = tone(load(px + vec2<i32>(0, 1)));
  let br = tone(load(px + vec2<i32>(1, 1)));
  return vec2<f32>((tr + 2.0 * r + br) - (tl + 2.0 * l + bl), (bl + 2.0 * b + br) - (tl + 2.0 * t + tr));
}

// A Kuwahara filter: of the four sectors round the pixel, the mean of the
// one with the least variance, so flat regions merge into strokes while
// edges stay sharp. `anisotropy` turns and stretches the sectors along the
// edge through the pixel.
fn kuwahara(px: vec2<i32>, radius: f32, anisotropy: f32) -> vec3<f32> {
  let r = i32(ceil(radius));
  let g = gradient(px);
  let mag = length(g);
  var angle = 0.0;
  var stretch = 1.0;
  if (anisotropy > 0.0 && mag > 1e-3) {
    // Strokes run along the edge: perpendicular to the gradient.
    angle = atan2(g.y, g.x) + 1.5707963;
    stretch = 1.0 + anisotropy * 2.0;
  }
  let ca = cos(angle);
  let sa = sin(angle);
  var best = vec3<f32>(0.0);
  var best_var = 1e9;
  for (var s = 0; s < 4; s++) {
    let sx = select(-1, 1, (s & 1) == 1);
    let sy = select(-1, 1, (s & 2) == 2);
    var sum = vec3<f32>(0.0);
    var sum2 = vec3<f32>(0.0);
    var n = 0.0;
    for (var y = 0; y <= r; y++) {
      for (var x = 0; x <= r; x++) {
        // The sector's offset, turned and stretched along the stroke.
        let o = vec2<f32>(f32(x * sx), f32(y * sy));
        let t = vec2<f32>(o.x * ca - o.y * sa, o.x * sa + o.y * ca);
        let e = vec2<f32>(t.x * stretch, t.y / stretch);
        if (length(e) > radius * stretch) { continue; }
        let c = load(px + vec2<i32>(round(e)));
        sum += c;
        sum2 += c * c;
        n += 1.0;
      }
    }
    let mean = sum / max(n, 1.0);
    let v = sum2 / max(n, 1.0) - mean * mean;
    let variance = v.r + v.g + v.b;
    if (variance < best_var) {
      best_var = variance;
      best = mean;
    }
  }
  return best;
}

fn paint(in: VsOut) -> vec3<f32> {
  let px = vec2<i32>(in.pos.xy);
  var c = load(px);
  if (st.paint.x > 0.0) {
    c = kuwahara(px, st.paint.x, st.paint.y);
  }
  if (st.paint.w > 0.0) {
    // Wet in wet: colour from a ring two radii out bleeds in.
    let d = i32(max(st.paint.x, 1.0)) * 2;
    let around = (load(px + vec2<i32>(d, 0)) + load(px + vec2<i32>(-d, 0)) + load(px + vec2<i32>(0, d)) + load(px + vec2<i32>(0, -d))) * 0.25;
    c = mix(c, around, st.paint.w * 0.5);
  }
  if (st.paint.z > 0.0) {
    // Pigment pools where the colour changes: darker at the edges of regions.
    let g = length(gradient(px));
    c *= 1.0 - st.paint.z * clamp(g * 1.5, 0.0, 1.0) * 0.6;
  }
  if (st.offset.z > 0.5 && st.canvas.x > 0.0) {
    var grain = 0.0;
    if (st.surface.x > 0.5) {
      grain = canvas_surface(in.pos.xy);
    } else {
      let size = vec2<f32>(textureDimensions(canvas));
      let uv = in.pos.xy * st.canvas.y / size;
      grain = textureSample(canvas, repeat_sampler, uv).r;
    }
    c *= mix(1.0, grain * 0.5 + 0.5, st.canvas.x);
  }
  return c;
}

// The canvas on the surfaces: sampled triplanar at the pixel's point
// (canvas texels per metre, st.surface.y), its level from one pixel's
// footprint, so the grain stays on the scene as the camera moves. The sky
// keeps the screen canvas.
fn canvas_surface(p: vec2<f32>) -> f32 {
  let size = vec2<f32>(textureDimensions(canvas));
  var s = surface_at(p);
  if (st.surface.x > 1.5) {
    s = object_at(p, s);
  }
  if (!s.ok) {
    let lod = log2(max(st.canvas.y, 1.0));
    return textureSampleLevel(canvas, repeat_sampler, p * st.canvas.y / size, lod).r;
  }
  let k = st.surface.y;
  let lod = log2(max(k * s.pixel, 1.0));
  let w = tri_weights(s.n);
  // An instance's space starts at its own origin: no world offset.
  let o = select(st.canvas_origin, vec4<f32>(0.0), s.obj);
  let q = s.p * k;
  let gx = textureSampleLevel(canvas, repeat_sampler, q.zy / size + o.zy, lod).r;
  let gy = textureSampleLevel(canvas, repeat_sampler, q.xz / size + o.xw, lod).r;
  let gz = textureSampleLevel(canvas, repeat_sampler, q.xy / size + o.xy, lod).r;
  return gx * w.x + gy * w.y + gz * w.z;
}

@fragment
fn fs_paint(in: VsOut) -> @location(0) vec4<f32> {
  return vec4<f32>(paint(in), 1.0);
}

@fragment
fn fs_paint_srgb(in: VsOut) -> @location(0) vec4<f32> {
  return vec4<f32>(srgb_encode(clamp(paint(in), vec3<f32>(0.0), vec3<f32>(1.0))), 1.0);
}

// --- lines ---------------------------------------------------------------

// A stroke set: lines at `angle` with `spacing`, 1 on the line.
fn stroke(p: vec2<f32>, angle: f32, spacing: f32, width: f32) -> f32 {
  let d = p.x * cos(angle) + p.y * sin(angle);
  let f = abs(fract(d / spacing) - 0.5) * spacing;
  return 1.0 - smoothstep(width * 0.5, width * 0.5 + 0.75, f);
}

// 1 where the prepass breaks within the line's width: a neighbour farther
// by more than the depth step (a silhouette, drawn on the nearer surface)
// or one of about the same depth turned past the crease threshold, as the
// outline pass finds them (fxpost.wgsl fs_outline); 0 on the sky.
fn depth_edge(px: vec2<i32>) -> f32 {
  return depth_edge_at(px, max(st.ink.z, 1.0));
}

// The hatching's contour weight: depth edges within one stroke spacing,
// nearer ones counting more (1 at the edge, a third a spacing away), so the
// strokes gather along silhouettes and creases.
fn depth_band(px: vec2<i32>) -> f32 {
  let r = max(st.hatching.y, 2.0);
  return (contour_at(px, 1.0) + contour_at(px, r * 0.5) + contour_at(px, r)) / 3.0;
}

// 1 on the nearer side of a silhouette or beside a crease `r` pixels away.
// A silhouette is a break in the inverse depth, which is affine across a
// plane in screen space: of two opposite neighbours, d0 / d + d0 / d' - 2
// is 0 on any plane however steeply it recedes (a long floor never counts)
// and below minus the depth step where one of them lies behind the
// surface. A crease is a neighbour of about the same depth turned past the
// crease threshold, as depth_edge finds it.
fn contour_at(px: vec2<i32>, r: f32) -> f32 {
  let d0 = view_depth(px);
  if (d0 >= st.ink_depth.x * 0.999) { return 0.0; }
  let n0 = normal_at(px);
  for (var i = 0; i < 4; i++) {
    let a = f32(i) * 0.7853982;
    let o = vec2<i32>(round(vec2<f32>(cos(a), sin(a)) * r));
    let dp = view_depth(px + o);
    let dm = view_depth(px - o);
    if (d0 / dp + d0 / dm - 2.0 < -st.ink_depth.y) { return 1.0; }
    if (abs(dp - d0) < d0 * st.ink_depth.y * 4.0 && 1.0 - dot(normal_at(px + o), n0) > st.ink_depth.z) {
      return 1.0;
    }
    if (abs(dm - d0) < d0 * st.ink_depth.y * 4.0 && 1.0 - dot(normal_at(px - o), n0) > st.ink_depth.z) {
      return 1.0;
    }
  }
  return 0.0;
}

// depth_edge with the neighbours `r` pixels away.
fn depth_edge_at(px: vec2<i32>, r: f32) -> f32 {
  let d0 = view_depth(px);
  if (d0 >= st.ink_depth.x * 0.999) { return 0.0; }
  let n0 = normal_at(px);
  for (var i = 0; i < 8; i++) {
    let a = f32(i) * 0.7853982;
    let q = px + vec2<i32>(round(vec2<f32>(cos(a), sin(a)) * r));
    let d = view_depth(q);
    if (d > d0 * (1.0 + st.ink_depth.y)) { return 1.0; }
    if (abs(d - d0) < d0 * st.ink_depth.y * 4.0 && 1.0 - dot(normal_at(q), n0) > st.ink_depth.z) {
      return 1.0;
    }
  }
  return 0.0;
}

// The hatching's ink at tone `t` over the frame: darker tones add stroke
// sets, each turned a little further.
fn hatch_screen(p: vec2<f32>, t: f32) -> f32 {
  let levels = max(st.hatching.z, 1.0);
  var ink = 0.0;
  for (var k = 0.0; k < levels; k += 1.0) {
    let threshold = 1.0 - (k + 1.0) / (levels + 1.0);
    if (t < threshold) {
      let turn = st.hatching.w + k * (1.5707963 / max(levels - 1.0, 1.0)) + select(0.0, 1.5707963, (i32(k) & 1) == 1);
      ink = max(ink, stroke(p, turn, st.hatching.y, 1.0) * clamp((threshold - t) * levels * 2.0, 0.3, 1.0));
    }
  }
  return ink;
}

// A stroke set on a surface plane: lines through every multiple of
// `spacing` (so a coarser octave's lines are among a finer one's), `width`
// wide with an edge of `aa`, all in metres.
fn stroke_on(q: vec2<f32>, angle: f32, spacing: f32, width: f32, aa: f32) -> f32 {
  let d = q.x * cos(angle) + q.y * sin(angle);
  let f = abs(fract(d / spacing + 0.5) - 0.5) * spacing;
  return 1.0 - smoothstep(width * 0.5, width * 0.5 + aa, f);
}

// The tonal art map on one plane at one octave: the screen hatching's
// stroke sets and thresholds, laid in metres, each stroke one screen pixel
// wide across itself (`j`: the plane's steps per pixel, plane_steps).
fn hatch_plane(q: vec2<f32>, t: f32, spacing: f32, j: vec4<f32>) -> f32 {
  let levels = max(st.hatching.z, 1.0);
  var ink = 0.0;
  for (var k = 0.0; k < levels; k += 1.0) {
    let threshold = 1.0 - (k + 1.0) / (levels + 1.0);
    if (t < threshold) {
      let turn = st.hatching.w + k * (1.5707963 / max(levels - 1.0, 1.0)) + select(0.0, 1.5707963, (i32(k) & 1) == 1);
      let width = screen_width(vec2<f32>(cos(turn), sin(turn)), j);
      ink = max(ink, stroke_on(q, turn, spacing, width, 0.75 * width) * clamp((threshold - t) * levels * 2.0, 0.3, 1.0));
    }
  }
  return ink;
}

// The hatching on the surfaces: the stroke sets laid triplanar in world
// space, so they wrap round forms and stay on them as the camera moves;
// the spacing keeps about st.hatching.y pixels by fading every other line
// out between two octaves.
fn hatch_surface(s: Surf, t: f32) -> f32 {
  let o = octave(st.hatching.y, s.pixel);
  let w = tri_weights(s.n);
  var ink = 0.0;
  for (var a = 0; a < 3; a++) {
    if (w[a] < 0.002) { continue; }
    let q = plane_uv(s.p, a);
    let j = plane_steps(s, a);
    let fine = hatch_plane(q, t, o.x, j);
    let coarse = hatch_plane(q, t, o.x * 2.0, j);
    ink += w[a] * mix(fine, coarse, o.y);
  }
  return ink;
}

// Halftone dots over the frame, larger where the tone is darker.
fn dots_screen(p: vec2<f32>, t: f32) -> f32 {
  let a = st.halftone.z;
  let r = vec2<f32>(p.x * cos(a) - p.y * sin(a), p.x * sin(a) + p.y * cos(a)) / st.halftone.y;
  let cell = fract(r) - 0.5;
  let radius = (1.0 - t) * 0.72;
  return 1.0 - smoothstep(radius - 0.08, radius + 0.08, length(cell));
}

// Dots on a surface plane at one octave: centred on every multiple of
// `pitch`, so a coarser octave's dots are among a finer one's. The edge
// is a screen pixel wide along the radius (`j`: the plane's steps per
// pixel, plane_steps).
fn dots_plane(q: vec2<f32>, t: f32, pitch: f32, j: vec4<f32>) -> f32 {
  let a = st.halftone.z;
  let r = vec2<f32>(q.x * cos(a) - q.y * sin(a), q.x * sin(a) + q.y * cos(a)) / pitch;
  let cell = fract(r + 0.5) - 0.5;
  let radius = (1.0 - t) * 0.72;
  // The radius's direction back on the plane (the grid turned by -a).
  let u = cell / max(length(cell), 1e-6);
  let along = vec2<f32>(u.x * cos(a) + u.y * sin(a), -u.x * sin(a) + u.y * cos(a));
  let edge = 0.5 * screen_width(along, j) / pitch;
  return 1.0 - smoothstep(radius - edge, radius + edge, length(cell));
}

// The halftone on the surfaces, as hatch_surface lays the strokes.
fn dots_surface(s: Surf, t: f32) -> f32 {
  let o = octave(st.halftone.y, s.pixel);
  let w = tri_weights(s.n);
  var cover = 0.0;
  for (var a = 0; a < 3; a++) {
    if (w[a] < 0.002) { continue; }
    let q = plane_uv(s.p, a);
    let j = plane_steps(s, a);
    cover += w[a] * mix(dots_plane(q, t, o.x, j), dots_plane(q, t, o.x * 2.0, j), o.y);
  }
  return cover;
}

fn lines(in: VsOut) -> vec3<f32> {
  let px = vec2<i32>(in.pos.xy);
  let p = in.pos.xy;
  var c = load(px);
  let t = tone(c);
  var surf: Surf;
  surf.ok = false;
  surf.obj = false;
  if (st.surface.z > 0.5 || st.surface.w > 0.5) {
    surf = surface_at(p);
  }
  // Object space for the hatching or the halftone (each keeps its own).
  var osurf = surf;
  if (st.surface.z > 1.5 || st.surface.w > 1.5) {
    osurf = object_at(p, surf);
  }
  if (st.hatching.x > 0.0) {
    // Darker tones add stroke sets, each turned a little further, over a
    // paper-light ground; with hatchingDepth silhouettes and creases
    // darken the tone the strokes follow.
    var th = t;
    if (st.ink_depth.w > 0.0) {
      th = max(t - st.ink_depth.w * depth_band(px), 0.0);
    }
    var ink = 0.0;
    if (st.surface.z > 1.5 && osurf.ok) {
      ink = hatch_surface(osurf, th);
    } else if (st.surface.z > 0.5 && surf.ok) {
      ink = hatch_surface(surf, th);
    } else {
      ink = hatch_screen(p, th);
    }
    let ground = mix(c, vec3<f32>(1.0), st.hatching.x * 0.85);
    c = mix(ground, st.hatching_color.rgb, ink * st.hatching.x);
  }
  if (st.halftone.x > 0.0) {
    // Dots on a rotated grid, larger where the image is darker.
    var dot_cover = 0.0;
    if (st.surface.w > 1.5 && osurf.ok) {
      dot_cover = dots_surface(osurf, t);
    } else if (st.surface.w > 0.5 && surf.ok) {
      dot_cover = dots_surface(surf, t);
    } else {
      dot_cover = dots_screen(p, t);
    }
    let paper = mix(c, vec3<f32>(1.0), st.halftone.x);
    c = mix(paper, st.halftone_color.rgb, dot_cover * st.halftone.x);
  }
  if (st.ink.x > 0.5) {
    // Luminance edges, widened by the max over the line's width.
    let w = max(i32(round(st.ink.z * 0.5)), 0);
    var edge = 0.0;
    for (var y = -w; y <= w; y++) {
      for (var x = -w; x <= w; x++) {
        edge = max(edge, length(gradient(px + vec2<i32>(x, y))));
      }
    }
    var k = smoothstep(st.ink.y, st.ink.y + 0.1, edge * 0.25);
    if (st.ink.w > 0.0) {
      k = max(k, depth_edge(px) * st.ink.w);
    }
    c = mix(c, st.ink_color.rgb, k);
  }
  return c;
}

@fragment
fn fs_lines(in: VsOut) -> @location(0) vec4<f32> {
  return vec4<f32>(lines(in), 1.0);
}

@fragment
fn fs_lines_srgb(in: VsOut) -> @location(0) vec4<f32> {
  return vec4<f32>(srgb_encode(clamp(lines(in), vec3<f32>(0.0), vec3<f32>(1.0))), 1.0);
}

// --- quantize ------------------------------------------------------------

fn bayer(px: vec2<u32>, n: u32) -> f32 {
  // The ordered dither threshold of an n x n Bayer matrix (n 2, 4, 8),
  // bit-interleaved as the recursive definition gives it, in 0..1.
  var x = px.x % n;
  var y = px.y % n;
  var v = 0u;
  var bits = 0u;
  var size = n;
  while (size > 1u) {
    size = size >> 1u;
    let bx = (x / size) & 1u;
    let by = (y / size) & 1u;
    v = (v << 2u) | ((bx ^ by) << 1u) | by;
    bits += 2u;
  }
  return (f32(v) + 0.5) / f32(1u << bits);
}

// Interleaved gradient noise (Jimenez): blue-noise-like without a texture.
fn ign(p: vec2<f32>) -> f32 {
  return fract(52.9829189 * fract(0.06711056 * p.x + 0.00583715 * p.y));
}

// The void-and-cluster tile, repeated: byte v is the threshold (v + 0.5) / 256.
fn blue(cell: vec2<u32>) -> f32 {
  let v = textureLoad(blue_noise, vec2<i32>(cell % vec2<u32>(64u)), 0).r;
  return (round(v * 255.0) + 0.5) / 256.0;
}

fn dither_at(pos: vec2<f32>) -> f32 {
  let cell = vec2<u32>(pos / max(st.quantize.w, 1.0));
  if (st.quantize.y < -1.5) { return blue(cell); }
  if (st.quantize.y < 0.0) { return ign(vec2<f32>(cell)); }
  if (st.quantize.y >= 2.0) { return bayer(cell, u32(st.quantize.y)); }
  return 0.5;
}

// Oklab, for the nearest palette colour by perceived difference.
fn oklab(c: vec3<f32>) -> vec3<f32> {
  let l = 0.4122214708 * c.r + 0.5363325363 * c.g + 0.0514459929 * c.b;
  let m = 0.2119034982 * c.r + 0.6806995451 * c.g + 0.1073969566 * c.b;
  let s = 0.0883024619 * c.r + 0.2817188376 * c.g + 0.6299787005 * c.b;
  let l3 = pow(max(l, 0.0), 1.0 / 3.0);
  let m3 = pow(max(m, 0.0), 1.0 / 3.0);
  let s3 = pow(max(s, 0.0), 1.0 / 3.0);
  return vec3<f32>(
    0.2104542553 * l3 + 0.7936177850 * m3 - 0.0040720468 * s3,
    1.9779984951 * l3 - 2.4285922050 * m3 + 0.4505937099 * s3,
    0.0259040371 * l3 + 0.7827717662 * m3 - 0.8086757660 * s3);
}

fn quantize(in: VsOut) -> vec3<f32> {
  let px = vec2<i32>(in.pos.xy);
  var c = load(px);
  let d = (dither_at(in.pos.xy) - 0.5) * st.quantize.z;
  if (st.quantize.x >= 2.0) {
    // Steps in display space, the dither spread across one step.
    let n = st.quantize.x - 1.0;
    let e = srgb_encode(clamp(c, vec3<f32>(0.0), vec3<f32>(1.0)));
    c = srgb_decode(clamp(floor(e * n + 0.5 + d) / n, vec3<f32>(0.0), vec3<f32>(1.0)));
  }
  let count = i32(st.offset.w);
  if (count > 0) {
    // The nearest palette colour of the dithered colour (the dither moves
    // it by a share of a palette step, so neighbours alternate).
    let shifted = clamp(c + vec3<f32>(d * 0.12), vec3<f32>(0.0), vec3<f32>(1.0));
    let want = oklab(shifted);
    var best = c;
    var best_d = 1e9;
    for (var i = 0; i < count; i++) {
      let p = textureLoad(palette, vec2<i32>(i, 0), 0).rgb;
      let delta = oklab(p) - want;
      let dist = dot(delta, delta);
      if (dist < best_d) {
        best_d = dist;
        best = p;
      }
    }
    c = best;
  }
  return c;
}

@fragment
fn fs_quantize(in: VsOut) -> @location(0) vec4<f32> {
  return vec4<f32>(quantize(in), 1.0);
}

@fragment
fn fs_quantize_srgb(in: VsOut) -> @location(0) vec4<f32> {
  return vec4<f32>(srgb_encode(clamp(quantize(in), vec3<f32>(0.0), vec3<f32>(1.0))), 1.0);
}

// --- upscale -------------------------------------------------------------

fn upscale(in: VsOut) -> vec3<f32> {
  let scale = max(st.output.z, 1.0);
  let p = (in.pos.xy - st.offset.xy) / scale;
  let size = vec2<f32>(textureDimensions(source));
  if (p.x < 0.0 || p.y < 0.0 || p.x >= size.x || p.y >= size.y) {
    return st.background.rgb;
  }
  return textureLoad(source, vec2<i32>(floor(p)), 0).rgb;
}

@fragment
fn fs_upscale(in: VsOut) -> @location(0) vec4<f32> {
  return vec4<f32>(upscale(in), 1.0);
}

@fragment
fn fs_upscale_srgb(in: VsOut) -> @location(0) vec4<f32> {
  return vec4<f32>(srgb_encode(clamp(upscale(in), vec3<f32>(0.0), vec3<f32>(1.0))), 1.0);
}
