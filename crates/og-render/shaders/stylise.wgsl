// The stylisation passes over the tonemapped image (stylise.rs, the IDL's
// Stylise): after the tonemap, at the logical resolution, each a fullscreen
// pass from one LDR target into the next (or the output, encoding to sRGB
// in the shader where the output is not an sRGB format):
//
//   paint     painterly smoothing (an anisotropic Kuwahara filter over a
//             smoothed structure tensor, aware of objects and detail),
//             pigment pooling at the edges, colour bleeding, a canvas laid
//             over the frame or, in surface or object space, on the
//             surfaces;
//   lines     cross-hatching from the tone (and, with `hatchingDepth`, the
//             prepass's contours), halftone dots on a rotated grid (both
//             over the frame or, in surface or object space, on the
//             surfaces), ink lines on luminance edges and, with
//             `inkDepth`, on the depth-normal prepass's silhouettes and
//             creases;
//   lens      barrel or pincushion distortion of the frame (after the
//             passes that read the depth, so their lines bend with it);
//   quantize  posterised channels and/or the nearest palette colour, mixed
//             by an ordered (Bayer), blue-noise or interleaved-gradient
//             dither whose cells lie over the frame or, in surface or object
//             space, on the surfaces (Obra Dinn's world-anchored dither); a
//             pixel already on a palette colour (art the cook conformed) is
//             kept as it is;
//   upscale   the logical frame scaled up by a whole number with nearest
//             filtering, centred in the output with the background round it;
//             or the render scale's frame (Renderer::set_render_scale)
//             filling the output, bilinear.
//
// Prepended by fullscreen.wgsl. Every pass binds the same layout.
struct Stylise {
  screen: vec4<f32>,          // logical w, h, 1/w, 1/h
  output: vec4<f32>,          // output w, h, the whole-number upscale (0: the render scale's, bilinear), 0
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
  lens: vec4<f32>,            // x: lens distortion (above 0 barrel), y: the scale keeping a barrel's farthest corner on the image, zw: the centre (uv)
  paint2: vec4<f32>,          // x: paintDetail (how far detail keeps its shape), y: paintCharacterDetail, z: 1 with the prepass bound (object ids, detail), w: 0
  dither_space: vec4<f32>,    // x: where the dither's cells lie (0 screen, 1 world, 2 object space, StyliseSpace::code)
  wash: vec4<f32>,            // the watercolour wash: x edge darkening, y granulation, z turbulence, w dry brush
  wash_paper: vec4<f32>,      // xyz: the wash's paper colour (linear), w: wobble (logical pixels)
  print: vec4<f32>,           // x: halftone mode (0 one ink, 1 cmyk), y: misregistration (logical pixels), z: halftone region (0 every tone, 1 shade, 2 highlight), w: 1 hatching keeps the colour under its strokes
  hatching_paper: vec4<f32>,  // rgb: the paper the hatching lies on (linear; white without hatching)
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
@group(0) @binding(10) var pre_outline: texture_2d<f32>;  // the prepass's outline target (Rgba8Unorm): a the object id; 0 without
@group(0) @binding(12) var scene_colour: texture_2d<f32>;  // the lit scene before exposure and the tone curve (the quantize and lines passes' conformed test; the source in the others)
@group(0) @binding(11) var conformed_colors: texture_2d<u32>;  // the conformed colours' hash table (stylise.rs conformed_table); 1 x 1 without

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
  // Paint, ink and quantization describe SDR paper/ink, even on an HDR surface.
  return clamp(textureLoad(source, clamp(px, vec2<i32>(0), size - 1), 0).rgb, vec3<f32>(0.0), vec3<f32>(1.0));
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

// The prepass texel under logical pixel `px` (`st.screen`): the same, or
// under TAAU, where this chain runs at the output's size, the scene pixel
// it lies in.
fn prepass_px(px: vec2<i32>, size: vec2<i32>) -> vec2<i32> {
  let p = vec2<i32>((vec2<f32>(px) + 0.5) * vec2<f32>(size) * st.screen.zw);
  return clamp(p, vec2<i32>(0), size - 1);
}

fn view_depth(px: vec2<i32>) -> f32 {
  let size = vec2<i32>(textureDimensions(pre_depth));
  return textureLoad(pre_depth, prepass_px(px, size), 0).r;
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
  return oct_decode(textureLoad(pre_normal, prepass_px(px, size), 0).xy);
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
  return textureLoad(pre_object, prepass_px(vec2<i32>(p), size), 0);
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

// The painterly filter (stylise.rs `Paint`), as the cook's texel-space
// paint (og-cook conform/texture2d/paint.rs, the same algorithm and
// constants, so the frame and the cook agree):
//
// 1. the structure tensor: Sobel derivatives of the colour, their outer
//    products summed over the channels and smoothed by a Gaussian (sigma
//    2, radius 3): the stroke's direction and how coherent the structure
//    is (Kyprianidis, Kang and Döllner 2009);
// 2. the anisotropic Kuwahara filter with eight sectors and polynomial
//    weights (Kyprianidis 2010), its ellipse along the stroke, squeezed
//    across it by the anisotropy: a port of Garrett Gunnell's (Acerola)
//    AnisotropicKuwahara.shader, MIT (third_party/acerola-postprocessing).
//    Coherent structure (an edge, a thin line) keeps its shape in a thin
//    ellipse along itself; incoherent texture (grass, noise) is painted
//    over in a round one;
// 3. detail awareness, from the prepass where the frame has one: no tap
//    crosses an object's edge (the outline target's id), and a pixel's
//    detail (its material's paintDetail, or Stylise.paintCharacterDetail
//    on a skinned instance: faces and hands) shrinks the radius, to the
//    source itself at 1;
// 4. pigment pooling as a density at the rims of regions (MNPR,
//    Montesdeoca et al. 2018), bleeding, and the canvas.
const PAINT_TENSOR_RADIUS: i32 = 3;
const PAINT_TENSOR_SIGMA: f32 = 2.0;
const PAINT_SHARPNESS: f32 = 8.0;
const PAINT_HARDNESS: f32 = 8.0;
const PAINT_ZERO_CROSSING: f32 = 0.58;
const PAINT_ALPHA_FLOOR: f32 = 0.25;

// The stroke frame at a pixel: the direction along the structure, its
// anisotropy (0 to 1) and edge strength (the larger eigenvalue's root).
struct PaintFrame {
  dir: vec2<f32>,
  aniso: f32,
  edge: f32,
};

fn paint_frame(px: vec2<i32>) -> PaintFrame {
  // The 9 x 9 block the 7 x 7 Sobel window reads, loaded once.
  var block: array<vec3<f32>, 81>;
  for (var j = -4; j <= 4; j++) {
    for (var i = -4; i <= 4; i++) {
      block[(j + 4) * 9 + i + 4] = load(px + vec2<i32>(i, j));
    }
  }
  var t = vec3<f32>(0.0);
  var weight = 0.0;
  for (var j = -PAINT_TENSOR_RADIUS; j <= PAINT_TENSOR_RADIUS; j++) {
    for (var i = -PAINT_TENSOR_RADIUS; i <= PAINT_TENSOR_RADIUS; i++) {
      let k = (j + 4) * 9 + i + 4;
      let sx = ((block[k - 9 + 1] + 2.0 * block[k + 1] + block[k + 9 + 1])
        - (block[k - 9 - 1] + 2.0 * block[k - 1] + block[k + 9 - 1])) * 0.25;
      let sy = ((block[k + 9 - 1] + 2.0 * block[k + 9] + block[k + 9 + 1])
        - (block[k - 9 - 1] + 2.0 * block[k - 9] + block[k - 9 + 1])) * 0.25;
      let g = exp(-f32(i * i + j * j) / (2.0 * PAINT_TENSOR_SIGMA * PAINT_TENSOR_SIGMA));
      t += vec3<f32>(dot(sx, sx), dot(sx, sy), dot(sy, sy)) * g;
      weight += g;
    }
  }
  t /= weight;
  let root = sqrt((t.x - t.z) * (t.x - t.z) + 4.0 * t.y * t.y);
  let l1 = 0.5 * (t.x + t.z + root);
  let l2 = 0.5 * (t.x + t.z - root);
  let v = vec2<f32>(l1 - t.x, -t.y);
  var f: PaintFrame;
  f.dir = select(vec2<f32>(0.0, 1.0), v / max(length(v), 1e-12), length(v) > 1e-12);
  f.aniso = select(0.0, (l1 - l2) / (l1 + l2), l1 + l2 > 1e-12);
  f.edge = sqrt(max(l1, 0.0));
  return f;
}

// The outline target's object id under a pixel (0: none; also without a
// prepass, st.paint2.z 0, where every tap counts).
fn paint_object(px: vec2<i32>) -> u32 {
  if (st.paint2.z < 0.5) { return 0u; }
  let size = vec2<i32>(textureDimensions(pre_outline));
  return u32(round(textureLoad(pre_outline, prepass_px(px, size), 0).a * 255.0));
}

// How much of its detail a pixel keeps (0 painted at the full radius, 1 the
// source itself): the prepass's detail byte (pbr_prepass.wgsl: above the
// reflectivity's 16 bits, the material's paintDetail in 7 bits, bit 7 a
// skinned instance) times Stylise.paintDetail.
fn paint_detail(px: vec2<i32>) -> f32 {
  if (st.paint2.z < 0.5 || st.paint2.x <= 0.0) { return 0.0; }
  let size = vec2<i32>(textureDimensions(pre_depth));
  let g = textureLoad(pre_depth, prepass_px(px, size), 0).g;
  let byte = u32(floor(g / 65536.0));
  let material = f32(byte & 127u) / 127.0;
  let character = select(0.0, st.paint2.y, (byte & 128u) != 0u);
  return clamp(max(material, character) * st.paint2.x, 0.0, 1.0);
}

// The anisotropic Kuwahara filter of `radius` at a pixel (`f` its stroke
// frame), leaving out taps on another object.
fn kuwahara(px: vec2<i32>, radius: f32, anisotropy: f32, f: PaintFrame) -> vec3<f32> {
  // Under half a pixel the ellipse holds the pixel alone: the source as
  // drawn. At 0 (paintDetail 1) the taps' ellipse coordinates divided by
  // it, and the NaN weights drew the material black.
  if (radius < 0.5) { return load(px); }
  var a = radius;
  var b = radius;
  if (anisotropy > 0.0) {
    let alpha = max((1.0 - min(anisotropy, 1.0)) / anisotropy, PAINT_ALPHA_FLOOR);
    a = radius * clamp((alpha + f.aniso) / alpha, 0.1, 2.0);
    b = radius * clamp(alpha / (alpha + f.aniso), 0.1, 2.0);
  }
  // phi = -atan2(dir.y, dir.x): the rotation taking the stroke onto x.
  let cos_phi = f.dir.x;
  let sin_phi = -f.dir.y;
  let max_x = i32(sqrt(a * a * cos_phi * cos_phi + b * b * sin_phi * sin_phi));
  let max_y = i32(sqrt(a * a * sin_phi * sin_phi + b * b * cos_phi * cos_phi));
  let zeta = 1.0 / max(radius, 1.0);
  let sin_zc = sin(PAINT_ZERO_CROSSING);
  let eta = (zeta + cos(PAINT_ZERO_CROSSING)) / (sin_zc * sin_zc);
  let own = paint_object(px);
  var m: array<vec4<f32>, 8>;
  var s: array<vec3<f32>, 8>;
  for (var y = -max_y; y <= max_y; y++) {
    for (var x = -max_x; x <= max_x; x++) {
      let o = vec2<f32>(f32(x), f32(y));
      var v = vec2<f32>((cos_phi * o.x - sin_phi * o.y) * 0.5 / a, (sin_phi * o.x + cos_phi * o.y) * 0.5 / b);
      if (dot(v, v) > 0.25) { continue; }
      let q = px + vec2<i32>(x, y);
      if (paint_object(q) != own) { continue; }
      let c = clamp(load(q), vec3<f32>(0.0), vec3<f32>(1.0));
      var w: array<f32, 8>;
      var sum = 0.0;
      for (var h = 0; h < 2; h++) {
        let vxx = zeta - eta * v.x * v.x;
        let vyy = zeta - eta * v.y * v.y;
        let z = vec4<f32>(max(0.0, v.y + vxx), max(0.0, -v.x + vyy), max(0.0, -v.y + vxx), max(0.0, v.x + vyy));
        let zz = z * z;
        w[h] = zz.x;
        w[2 + h] = zz.y;
        w[4 + h] = zz.z;
        w[6 + h] = zz.w;
        sum += zz.x + zz.y + zz.z + zz.w;
        v = 0.70710678 * vec2<f32>(v.x - v.y, v.x + v.y);
      }
      let g = exp(-3.125 * dot(v, v)) / max(sum, 1e-12);
      for (var k = 0; k < 8; k++) {
        let wk = w[k] * g;
        m[k] += vec4<f32>(c * wk, wk);
        s[k] += c * c * wk;
      }
    }
  }
  var out = vec4<f32>(0.0);
  for (var k = 0; k < 8; k++) {
    if (m[k].w <= 0.0) { continue; }
    let mean = m[k].rgb / m[k].w;
    let var3 = abs(s[k] / m[k].w - mean * mean);
    let sigma2 = var3.r + var3.g + var3.b;
    let w = 1.0 / (1.0 + pow(PAINT_HARDNESS * 1000.0 * sigma2, 0.5 * PAINT_SHARPNESS));
    out += vec4<f32>(mean * w, w);
  }
  if (out.w <= 0.0) { return load(px); }
  return clamp(out.rgb / out.w, vec3<f32>(0.0), vec3<f32>(1.0));
}

fn paint(in: VsOut) -> vec3<f32> {
  let px = vec2<i32>(in.pos.xy);
  var c = load(px);
  var f: PaintFrame;
  f.dir = vec2<f32>(0.0, 1.0);
  f.aniso = 0.0;
  f.edge = 0.0;
  if (st.paint.x > 0.0 || st.paint.z > 0.0) {
    f = paint_frame(px);
  }
  if (st.paint.x > 0.0) {
    // Detail shrinks the radius: at 1 the ellipse holds the pixel alone.
    let radius = st.paint.x * (1.0 - paint_detail(px));
    c = kuwahara(px, radius, st.paint.y, f);
  }
  if (st.paint.w > 0.0) {
    // Wet in wet: colour from a ring two radii out bleeds in.
    let d = i32(max(st.paint.x, 1.0)) * 2;
    let around = (load(px + vec2<i32>(d, 0)) + load(px + vec2<i32>(-d, 0)) + load(px + vec2<i32>(0, d)) + load(px + vec2<i32>(0, -d))) * 0.25;
    c = mix(c, around, st.paint.w * 0.5);
  }
  if (st.paint.z > 0.0) {
    // Pigment pools at the rims of regions: a denser pigment there, the
    // colour raised to 1 + density, so the rim darkens and keeps its hue.
    let density = st.paint.z * clamp(f.edge * 2.0, 0.0, 1.0);
    c = pow(max(c, vec3<f32>(0.0)), vec3<f32>(1.0 + density));
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

// --- watercolour -----------------------------------------------------------
// The wash over the painted frame (a pass after `paint`, reading its
// output), from Bousseau et al. 2006, "Interactive watercolor rendering
// with temporal coherence and abstraction", and MNPR (Montesdeoca et al.
// 2018, third_party/mnpr). A pigment density d about 1 modulates each
// colour, C' = C (1 - (1 - C)(d - 1)) relative to the paper: thinner
// towards the paper where d < 1, thicker where d > 1, strongest in the
// middle tones and leaving the paper and black alone, so the wash keeps
// its mean value where a multiply by the grain only darkens. The density
// comes from the paper's height about its mean (granulation: pigment
// settles in the pits) and from a slow value noise (turbulence: a wash
// dries patchy); pigment pools in a rim two to three pixels wide where the
// painted regions meet (edge darkening, MNPR's pow(C, 1 + d) from the
// painted frame's gradient); the dry brush leaves the paper's peaks bare
// under light colours; and the frame is read displaced by the paper's
// relief and a slow waver (substrate distortion, wobble). The cook's
// texel-space copy is og-cook's paint.rs `Wash`.

const WASH_TURBULENCE_CELL: f32 = 40.0;  // canvas texels per cell of the turbulence's noise
const WASH_WOBBLE_CELL: f32 = 14.0;      // canvas texels per cell of the waver's noise
const WASH_GRAIN_GAIN: f32 = 2.5;        // density per unit of the paper's height about its mean
const WASH_RELIEF_GAIN: f32 = 4.0;       // waver per unit of the paper's height gradient
const WASH_PEAK: f32 = 0.08;             // the height above the mean that is a peak (dry brush)

// A hash of an integer cell, 0 to 1 (the cook's `wash_hash`).
fn wash_hash(c: vec3<i32>, seed: u32) -> f32 {
  let u = bitcast<vec3<u32>>(c);
  var h = (u.x * 0x8da6b343u) ^ (u.y * 0xd8163841u) ^ (u.z * 0xcb1ab31fu) ^ (seed * 0x9e3779b9u);
  h = (h ^ (h >> 15u)) * 0x2c1b3c6du;
  h = (h ^ (h >> 12u)) * 0x297a2d39u;
  h = h ^ (h >> 15u);
  return f32(h & 0xffffffu) / 16777215.0;
}

// Smooth value noise, 0 to 1 about 0.5, one cell per unit.
fn wash_noise(p: vec3<f32>, seed: u32) -> f32 {
  let i = vec3<i32>(floor(p));
  let f = fract(p);
  let u = f * f * (3.0 - 2.0 * f);
  var n = 0.0;
  for (var k = 0; k < 8; k++) {
    let o = vec3<i32>(k & 1, (k >> 1) & 1, (k >> 2) & 1);
    let w = select(1.0 - u, u, o == vec3<i32>(1));
    n += wash_hash(i + o, seed) * w.x * w.y * w.z;
  }
  return n;
}

// The paper's height (its canvas's red, 0 to 1) under a logical pixel
// position, where the canvas lies (the screen, or the surfaces); 0.5 flat
// without a canvas.
fn paper_height(p: vec2<f32>) -> f32 {
  if (st.offset.z < 0.5) { return 0.5; }
  if (st.surface.x > 0.5) { return canvas_surface(p); }
  let size = vec2<f32>(textureDimensions(canvas));
  return textureSampleLevel(canvas, repeat_sampler, p * st.canvas.y / size, log2(max(st.canvas.y, 1.0))).r;
}

// The painted frame's gradient (a 3 x 3 Sobel of the colour over the 5 x 5
// block `b`) at the block's texel (x, y), 1 to 3.
fn wash_sobel(b: ptr<function, array<vec3<f32>, 25>>, x: i32, y: i32) -> f32 {
  let k = y * 5 + x;
  let sx = ((*b)[k - 5 + 1] + 2.0 * (*b)[k + 1] + (*b)[k + 5 + 1]) - ((*b)[k - 5 - 1] + 2.0 * (*b)[k - 1] + (*b)[k + 5 - 1]);
  let sy = ((*b)[k + 5 - 1] + 2.0 * (*b)[k + 5] + (*b)[k + 5 + 1]) - ((*b)[k - 5 - 1] + 2.0 * (*b)[k - 5] + (*b)[k - 5 + 1]);
  return sqrt(dot(sx, sx) + dot(sy, sy)) * 0.25;
}

fn watercolor(in: VsOut) -> vec3<f32> {
  let p = in.pos.xy;
  let paper = max(st.wash_paper.xyz, vec3<f32>(1e-3));
  let canvas_on = st.offset.z > 0.5;
  // The paper's point under the pixel, in canvas texels (the screen's, or
  // the surfaces' where the canvas lies on them), its height there and
  // the paper's mean height (the canvas's smallest mip).
  var q = vec3<f32>(p * st.canvas.y, 0.0);
  if (canvas_on && st.surface.x > 0.5) {
    var s = surface_at(p);
    if (st.surface.x > 1.5) { s = object_at(p, s); }
    if (s.ok) { q = s.p * st.surface.y; }
  }
  let height = paper_height(p);
  var mean = 0.5;
  if (canvas_on) {
    mean = textureSampleLevel(canvas, repeat_sampler, vec2<f32>(0.5), 16.0).r;
  }
  // Wobble: the frame read where the paper's relief and a slow waver
  // displace it.
  var off = vec2<f32>(0.0);
  if (st.wash_paper.w > 0.0) {
    let wave = vec2<f32>(wash_noise(q / WASH_WOBBLE_CELL, 7u), wash_noise(q / WASH_WOBBLE_CELL, 11u)) * 2.0 - 1.0;
    var relief = vec2<f32>(0.0);
    if (canvas_on) {
      relief = vec2<f32>(
        paper_height(p + vec2<f32>(1.0, 0.0)) - paper_height(p - vec2<f32>(1.0, 0.0)),
        paper_height(p + vec2<f32>(0.0, 1.0)) - paper_height(p - vec2<f32>(0.0, 1.0))) * 0.5;
    }
    off = st.wash_paper.w * clamp(wave * 2.0 + relief * WASH_RELIEF_GAIN, vec2<f32>(-1.0), vec2<f32>(1.0));
  }
  let size = vec2<f32>(textureDimensions(source));
  var c = clamp(textureSampleLevel(source, clamp_sampler, (p + off) / size, 0.0).rgb, vec3<f32>(0.0), vec3<f32>(1.0));
  // Edge darkening: the rim where the painted regions meet, the gradient
  // blurred over the pixel's 3 x 3 (a binomial weight), so the rim is a
  // soft band a few pixels wide and not a line (MNPR blurs its edge map
  // the same way).
  var rim = 0.0;
  if (st.wash.x > 0.0) {
    let o = vec2<i32>(floor(p + off));
    var b: array<vec3<f32>, 25>;
    for (var j = -2; j <= 2; j++) {
      for (var i = -2; i <= 2; i++) {
        b[(j + 2) * 5 + i + 2] = load(o + vec2<i32>(i, j));
      }
    }
    var g = 0.0;
    for (var j = 1; j <= 3; j++) {
      for (var i = 1; i <= 3; i++) {
        g += wash_sobel(&b, i, j) * f32((2 - abs(i - 2)) * (2 - abs(j - 2)));
      }
    }
    rim = smoothstep(0.02, 0.22, g / 16.0);
  }
  // The pigment density: granulation from the paper's height about its
  // mean, turbulence from the noise.
  var d = 1.0;
  if (st.wash.y > 0.0) {
    d += st.wash.y * clamp((mean - height) * WASH_GRAIN_GAIN, -1.0, 1.0);
  }
  if (st.wash.z > 0.0) {
    let n = wash_noise(q / WASH_TURBULENCE_CELL, 3u) * 0.6 + wash_noise(q / (WASH_TURBULENCE_CELL * 0.5), 5u) * 0.4;
    d += st.wash.z * clamp((n - 0.5) * 3.0, -1.0, 1.0);
  }
  // Bousseau's density on the pigment's transmittance over the paper.
  let t = clamp(c / paper, vec3<f32>(0.0), vec3<f32>(1.0));
  var t2 = clamp(t * (1.0 - (1.0 - t) * (d - 1.0)), vec3<f32>(0.0), vec3<f32>(1.0));
  // Dry brush: the paper's peaks left bare under the thinnest washes.
  if (st.wash.w > 0.0) {
    let peak = smoothstep(0.0, WASH_PEAK, height - mean);
    let thin = luma(t2);
    t2 = mix(t2, vec3<f32>(1.0), st.wash.w * peak * thin * thin);
  }
  c = t2 * paper;
  if (rim > 0.0) {
    c = pow(max(c, vec3<f32>(0.0)), vec3<f32>(1.0 + st.wash.x * rim));
  }
  return c;
}

@fragment
fn fs_watercolor(in: VsOut) -> @location(0) vec4<f32> {
  return vec4<f32>(watercolor(in), 1.0);
}

@fragment
fn fs_watercolor_srgb(in: VsOut) -> @location(0) vec4<f32> {
  return vec4<f32>(srgb_encode(clamp(watercolor(in), vec3<f32>(0.0), vec3<f32>(1.0))), 1.0);
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
// How far a stroke set has come in at tone `t` below its `threshold`: 0
// at the threshold, so the lightest strokes thin and fade in from clean
// paper instead of starting at a third of their ink, to 1 half a level
// darker.
fn fade_in(threshold: f32, t: f32, levels: f32) -> f32 {
  return smoothstep(0.0, 1.0, (threshold - t) * (levels + 1.0) * 2.0);
}

fn hatch_screen(p: vec2<f32>, t: f32) -> f32 {
  let levels = max(st.hatching.z, 1.0);
  var ink = 0.0;
  for (var k = 0.0; k < levels; k += 1.0) {
    let threshold = 1.0 - (k + 1.0) / (levels + 1.0);
    if (t < threshold) {
      let turn = st.hatching.w + k * (1.5707963 / max(levels - 1.0, 1.0)) + select(0.0, 1.5707963, (i32(k) & 1) == 1);
      let f = fade_in(threshold, t, levels);
      ink = max(ink, stroke(p, turn, st.hatching.y, f) * f);
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
      let f = fade_in(threshold, t, levels);
      ink = max(ink, stroke_on(q, turn, spacing, width * f, 0.75 * width) * f);
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

// Halftone dots over the frame, larger where the tone is darker, on a grid
// turned by `a` (radians).
fn dots_screen(p: vec2<f32>, t: f32, a: f32) -> f32 {
  let r = vec2<f32>(p.x * cos(a) - p.y * sin(a), p.x * sin(a) + p.y * cos(a)) / st.halftone.y;
  let cell = fract(r) - 0.5;
  let radius = (1.0 - t) * 0.72;
  return 1.0 - smoothstep(radius - 0.08, radius + 0.08, length(cell));
}

// Dots on a surface plane at one octave: centred on every multiple of
// `pitch`, so a coarser octave's dots are among a finer one's. The edge
// is a screen pixel wide along the radius (`j`: the plane's steps per
// pixel, plane_steps).
fn dots_plane(q: vec2<f32>, t: f32, pitch: f32, j: vec4<f32>, a: f32) -> f32 {
  let r = vec2<f32>(q.x * cos(a) - q.y * sin(a), q.x * sin(a) + q.y * cos(a)) / pitch;
  let cell = fract(r + 0.5) - 0.5;
  let radius = (1.0 - t) * 0.72;
  // The radius's direction back on the plane (the grid turned by -a).
  let u = cell / max(length(cell), 1e-6);
  let along = vec2<f32>(u.x * cos(a) + u.y * sin(a), -u.x * sin(a) + u.y * cos(a));
  let edge = 0.5 * screen_width(along, j) / pitch;
  return 1.0 - smoothstep(radius - edge, radius + edge, length(cell));
}

// The halftone on the surfaces, as hatch_surface lays the strokes: a grid
// turned by `angle`, moved by `shift` logical pixels (a plate's
// misregistration, measured in the footprint's metres).
fn dots_surface(s: Surf, t: f32, angle: f32, shift: vec2<f32>) -> f32 {
  let o = octave(st.halftone.y, s.pixel);
  let w = tri_weights(s.n);
  var cover = 0.0;
  for (var a = 0; a < 3; a++) {
    if (w[a] < 0.002) { continue; }
    let q = plane_uv(s.p, a) + shift * s.pixel;
    let j = plane_steps(s, a);
    cover += w[a] * mix(dots_plane(q, t, o.x, j, angle), dots_plane(q, t, o.x * 2.0, j, angle), o.y);
  }
  return cover;
}

// One plate's dots at `t` (the tone the plate prints as: 1 - its ink),
// where the halftone lies, its grid turned by `angle` and moved by
// `shift` logical pixels.
fn plate(p: vec2<f32>, surf: Surf, osurf: Surf, t: f32, angle: f32, shift: vec2<f32>) -> f32 {
  // No ink, no dots (a dot of radius 0 still softens to half cover at
  // the cell's centre).
  if (t >= 0.998) { return 0.0; }
  if (st.surface.w > 1.5 && osurf.ok) {
    return dots_surface(osurf, t, angle, shift);
  }
  if (st.surface.w > 0.5 && surf.ok) {
    return dots_surface(surf, t, angle, shift);
  }
  return dots_screen(p + shift, t, angle);
}

// Four-colour process print (Stylise.halftoneMode cmyk): the colour split
// into cyan, magenta, yellow and black (grey component replacement in
// display space), each printed as its own dot screen at the classic
// angles (black at halftoneAngle, cyan 30 degrees before it, magenta 30
// after, yellow 45 before: 15, 75, 0 and 45 at the default), the plates
// offset by the misregistration, the inks multiplied on `paper` (white,
// or the colour itself under Ben-Day dots), each plate's ink scaled by
// `reach` (1, or ben_day_reach so its dots never touch).
fn cmyk(p: vec2<f32>, surf: Surf, osurf: Surf, c: vec3<f32>, paper: vec3<f32>, reach: f32) -> vec3<f32> {
  let e = srgb_encode(clamp(c, vec3<f32>(0.0), vec3<f32>(1.0)));
  let k = 1.0 - max(e.r, max(e.g, e.b));
  let cmy = select((vec3<f32>(1.0) - e - k) / max(1.0 - k, 1e-4), vec3<f32>(0.0), k > 0.9999);
  let a = st.halftone.z;
  let r = st.print.y;
  let dc = plate(p, surf, osurf, 1.0 - cmy.x * reach, a - 0.5235988, vec2<f32>(r, 0.0));
  let dm = plate(p, surf, osurf, 1.0 - cmy.y * reach, a + 0.5235988, vec2<f32>(0.0, r));
  let dy = plate(p, surf, osurf, 1.0 - cmy.z * reach, a - 0.7853982, vec2<f32>(-0.7071, -0.7071) * r);
  let dk = plate(p, surf, osurf, 1.0 - k * reach, a, vec2<f32>(0.0));
  let ground = srgb_encode(clamp(paper, vec3<f32>(0.0), vec3<f32>(1.0)));
  let printed = ground * vec3<f32>(1.0 - dc, 1.0 - dm, 1.0 - dy) * (1.0 - dk);
  return srgb_decode(clamp(printed, vec3<f32>(0.0), vec3<f32>(1.0)));
}

// Where the halftone prints by the tone (Stylise.halftoneRegion): 1 on
// every tone, or only the shade (Ben-Day dots in the shadows, Hi-Fi Rush)
// or only the highlights. The shade is the darkest third of the tones and
// the highlights the lightest, each with a soft step: a mid-toned field
// in the light (a lawn, a crate) is the paper's, not dotted over.
// How much of its ink a plate prints as dots under a halftoneRegion
// (Ben-Day): a dot's radius is (1 - tone) * 0.72 of the cell, so at 0.55
// the darkest dot is 0.4 of the cell and never touches its neighbours.
// The shade is the darkest tones, where full-tone dots would merge into
// ink with a grid of paper holes between them: a checker, not dots.
const BEN_DAY_REACH: f32 = 0.55;

// Under a region the dots fade on surfaces turned away from the camera,
// their normal more than about 30 to 55 degrees off the view ray (the
// prepass's): a floor's cast shadow seen from a game camera's height keeps
// flat ink, where dots squeezed into its few rows read as a checker. Where
// no surface lies (the sky, a 2D scene) the dots print.
fn facing(p: vec2<f32>) -> f32 {
  let px = vec2<i32>(p);
  let d = view_depth(px);
  if (!(d > 0.0 && d < st.ink_depth.x * 0.999)) { return 1.0; }
  return smoothstep(0.6, 0.85, abs(dot(normal_at(px), view_dir(p))));
}

fn halftone_region(p: vec2<f32>, t: f32) -> f32 {
  if (st.print.z > 1.5) { return smoothstep(0.6, 0.75, t) * facing(p); }
  if (st.print.z > 0.5) { return (1.0 - smoothstep(0.25, 0.4, t)) * facing(p); }
  return 1.0;
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
    // ground lifted towards the paper (hatchingPaper); with hatchingDepth silhouettes and creases
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
    if (st.print.w > 0.5) {
      // Colour hatching (Stylise.hatchingKeepColor): the ground is a wash,
      // lifted towards the paper's lightness as far as one-ink hatching
      // lifts it, keeping its hue and, as pigment on paper does, more of a
      // strong colour's chroma than a weak tint's (in Oklab; one-ink's mix
      // towards white takes all of it): a saturated prop keeps its colour
      // while a weakly tinted lit floor or wall goes to paper, and the whole
      // wash lies on the paper's own colour (hatchingPaper); conformed
      // art keeps its authored colour, as the quantize pass keeps it. Each
      // stroke is the colour darkened towards the hatching colour, so the
      // hue stays under the strokes instead of turning grey.
      let lab = oklab(c);
      let lift = st.hatching.x * 0.85;
      var wash: vec3<f32>;
      if (is_conformed(drawn_at(p))) {
        // Conformed art keeps the colour it was authored in: lifted in
        // Oklab lightness, with half the chroma that lift would take.
        wash = vec3<f32>(mix(lab.x, 1.0, lift), lab.yz * (1.0 - lift * 0.5));
      } else {
        // A tint already near the paper's lightness (a pale sky) is the
        // paper's own tint and keeps it.
        // A weak tint thins out at twice the lift, so a lit floor's faint
        // cast from the sky (lavender-grey on a cool day) goes to the paper
        // while a saturated prop keeps all of its colour.
        let pigment = mix(1.0, smoothstep(0.02, 0.14, length(lab.yz)), min(lift * 2.0, 1.0) * (1.0 - smoothstep(0.8, 0.92, lab.x)));
        // The lightness of one-ink's ground (its mix towards white in
        // linear light), so the wash is as light as that paper.
        let paper = oklab(mix(c, vec3<f32>(1.0), lift)).x;
        wash = vec3<f32>(paper, lab.yz * (1.0 - lift * 0.5) * pigment);
      }
      // Laid on the paper: what the wash lets through, the paper reflects
      // (a watercolour's pigment filters the sheet under it), so a cream
      // sketchbook warms every wash and a weak tint reads as the page. The
      // paper's colour at the page's own lightness (over its luminance): a
      // cream sheet warms the wash without greying it.
      let sheet = st.hatching_paper.rgb / max(dot(st.hatching_paper.rgb, vec3<f32>(0.2126, 0.7152, 0.0722)), 1e-3);
      let ground = clamp(clamp(oklab_to_linear(wash), vec3<f32>(0.0), vec3<f32>(1.0)) * sheet, vec3<f32>(0.0), vec3<f32>(1.0));
      let stroke = c * 0.3 + st.hatching_color.rgb * 0.5;
      c = mix(ground, stroke, ink * st.hatching.x);
    } else {
      let ground = mix(c, st.hatching_paper.rgb, st.hatching.x * 0.85);
      c = mix(ground, st.hatching_color.rgb, ink * st.hatching.x);
    }
  }
  if (st.halftone.x > 0.0) {
    // Over every tone the dots reproduce the tone on white paper, merging
    // in the darks as print does. Under a region they are Ben-Day dots: a
    // tint printed over the region's own colour (lightened a little, so
    // the dots carry the shade), never touching, so a flat shade stays
    // one colour with separate dots on it.
    let region = st.print.z > 0.5;
    let reach = select(1.0, BEN_DAY_REACH, region);
    let ground = select(vec3<f32>(1.0), mix(c, vec3<f32>(1.0), 0.25), region);
    var printed: vec3<f32>;
    if (st.print.x > 0.5) {
      printed = mix(c, cmyk(p, surf, osurf, c, ground, reach), st.halftone.x);
    } else {
      // Dots on a rotated grid, larger where the image is darker.
      let dot_cover = plate(p, surf, osurf, 1.0 - (1.0 - t) * reach, st.halftone.z, vec2<f32>(0.0));
      let paper = mix(c, ground, st.halftone.x);
      printed = mix(paper, st.halftone_color.rgb, dot_cover * st.halftone.x);
    }
    c = mix(c, printed, halftone_region(p, t));
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
  // bit-interleaved as the recursive definition M(2n) = 4 M(n) + M(2)
  // gives it, in 0..1: the finest bit of the position is the most
  // significant of the threshold, so neighbouring pixels hold thresholds
  // half the range apart (dispersed). Taking the coarsest bit first would
  // cluster each level into one quadrant's block.
  let x = px.x % n;
  let y = px.y % n;
  var v = 0u;
  var bits = 0u;
  var size = 1u;
  while (size < n) {
    let bx = (x / size) & 1u;
    let by = (y / size) & 1u;
    v = (v << 2u) | ((bx ^ by) << 1u) | by;
    bits += 2u;
    size = size << 1u;
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

// The pattern's threshold at a cell (its coordinates wrap: every
// pattern's period, 2 to 64, divides 2^32).
fn threshold(cell: vec2<u32>) -> f32 {
  if (st.quantize.y < -1.5) { return blue(cell); }
  if (st.quantize.y < 0.0) { return ign(vec2<f32>(cell & vec2<u32>(0xffffu))); }
  if (st.quantize.y >= 2.0) { return bayer(cell, u32(st.quantize.y)); }
  return 0.5;
}

// The threshold of the cells on plane coordinates `q` (metres on a surface
// plane, or a direction's plane on the camera's cube) where one logical
// pixel moves `q` by `gu` along its first axis and `gv` along its second
// (each the change per pixel across and down): each axis takes its own
// whole octave (a power of two of the units), the finest whose cell is at
// least ditherScale pixels on the screen along both screen axes. So a
// floor seen at a grazing angle gets cells long along its depth and fine
// across, each about one dither cell on the screen, instead of square
// cells stretched into streaks, and the pattern stays the ordered one
// (no cell of another octave among them) while it stays on the surface.
fn dither_cells(q: vec2<f32>, gu: vec2<f32>, gv: vec2<f32>) -> f32 {
  let scale = max(st.quantize.w, 1.0);
  // 1/64 of an octave of slack, so a footprint exactly on a power of two
  // (an orthographic wall at one cell per pixel) keeps that octave.
  let lu = clamp(ceil(log2(scale * max(length(gu), 1e-9)) - 0.015625), -24.0, 12.0);
  let lv = clamp(ceil(log2(scale * max(length(gv), 1e-9)) - 0.015625), -24.0, 12.0);
  let cell = vec2<f32>(exp2(lu), exp2(lv));
  return threshold(bitcast<vec2<u32>>(vec2<i32>(floor(q / cell))));
}

// The world direction through logical pixel position `p`.
fn view_dir(p: vec2<f32>) -> vec3<f32> {
  let uv = p * st.screen.zw;
  let ndc = vec2<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0);
  let a = st.inv_proj * vec4<f32>(ndc, 0.0, 1.0);
  let b = st.inv_proj * vec4<f32>(ndc, 1.0, 1.0);
  return normalize((st.view_rotation * vec4<f32>(b.xyz / b.w - a.xyz / a.w, 0.0)).xyz);
}

// The dither on the sky and where no surface lies, with the dither on the
// surfaces: cells on a cube round the camera (Lucas Pope's Obra Dinn maps
// its dither on a sphere round the camera), so turning the camera carries
// the pattern with the sky instead of sliding it under the screen.
fn dither_sky(p: vec2<f32>) -> f32 {
  let d = view_dir(p);
  let ad = abs(d);
  var a = 2;
  if (ad.x >= ad.y && ad.x >= ad.z) { a = 0; } else if (ad.y >= ad.z) { a = 1; }
  let face = d / max(ad[a], 1e-6);
  // How a pixel's step across and down moves the point on the face.
  let nx = view_dir(p + vec2<f32>(1.0, 0.0));
  let ny = view_dir(p + vec2<f32>(0.0, 1.0));
  let sx = plane_uv(nx / max(abs(nx[a]), 1e-6) - face, a);
  let sy = plane_uv(ny / max(abs(ny[a]), 1e-6) - face, a);
  // The face's sign keeps opposite faces from sharing cells.
  return dither_cells(
    plane_uv(face, a) + select(0.0, 8.0, d[a] < 0.0),
    vec2<f32>(sx.x, sy.x),
    vec2<f32>(sx.y, sy.y),
  );
}

// The dither on a surface: cells on the plane its normal faces most (a
// threshold cannot blend between planes: the most-facing one takes it),
// sized along each of the plane's axes by the pixel steps there.
fn dither_surface(s: Surf) -> f32 {
  let w = tri_weights(s.n);
  var a = 2;
  if (w.x >= w.y && w.x >= w.z) { a = 0; } else if (w.y >= w.z) { a = 1; }
  let sx = plane_uv(s.dx, a);
  let sy = plane_uv(s.dy, a);
  return dither_cells(plane_uv(s.p, a), vec2<f32>(sx.x, sy.x), vec2<f32>(sx.y, sy.y));
}

fn dither_at(pos: vec2<f32>) -> f32 {
  if (st.dither_space.x > 0.5) {
    var s = surface_at(pos);
    if (st.dither_space.x > 1.5) {
      s = object_at(pos, s);
    }
    if (s.ok) {
      return dither_surface(s);
    }
    return dither_sky(pos);
  }
  return threshold(vec2<u32>(pos / max(st.quantize.w, 1.0)));
}

// Whether `c` is a colour the cook put conformed art on (a texture's own
// palette, Scene::add_conformed_colors), within CONFORMED_REACH sRGB
// levels on every channel (block compression and filtering move a flat
// conformed field by a few): the table of stylise.rs conformed_table,
// colours bucketed by sixteens per channel, so the reach round `c` spans
// at most two buckets a channel; each bucket's run of slots is walked
// from its Fibonacci slot to an empty one. A lit or fogged tone, or a
// plain material's gradient, lies farther and is quantised.
const CONFORMED_REACH: i32 = 6;

fn conformed(c: vec3<f32>) -> bool {
  let size = textureDimensions(conformed_colors).x;
  if (size < 2u) { return false; }
  let e = vec3<i32>(round(srgb_encode(clamp(c, vec3<f32>(0.0), vec3<f32>(1.0))) * 255.0));
  let lo = max(e - CONFORMED_REACH, vec3<i32>(0)) >> vec3<u32>(4u);
  let hi = min(e + CONFORMED_REACH, vec3<i32>(255)) >> vec3<u32>(4u);
  let slots = size * size;
  let bits = countTrailingZeros(slots);
  for (var r = lo.x; r <= hi.x; r++) {
    for (var g = lo.y; g <= hi.y; g++) {
      for (var b = lo.z; b <= hi.z; b++) {
        let bucket = u32((r << 8u) | (g << 4u) | b);
        var i = ((bucket + 1u) * 0x9E3779B1u) >> (32u - bits);
        for (var n = 0u; n < slots; n++) {
          let k = textureLoad(conformed_colors, vec2<i32>(i32(i % size), i32(i / size)), 0).r;
          if (k == 0u) { break; }
          let kc = vec3<i32>(i32((k >> 16u) & 255u), i32((k >> 8u) & 255u), i32(k & 255u));
          if (all(abs(kc - e) <= vec3<i32>(CONFORMED_REACH))) { return true; }
          i = (i + 1u) & (slots - 1u);
        }
      }
    }
  }
  return false;
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

fn oklab_to_linear(c: vec3<f32>) -> vec3<f32> {
  let l3 = c.x + 0.3963377774 * c.y + 0.2158037573 * c.z;
  let m3 = c.x - 0.1055613458 * c.y - 0.0638541728 * c.z;
  let s3 = c.x - 0.0894841775 * c.y - 1.2914855480 * c.z;
  let l = l3 * l3 * l3;
  let m = m3 * m3 * m3;
  let s = s3 * s3 * s3;
  return vec3<f32>(
    4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
    -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
    -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s);
}

// How near (Oklab distance) a pixel must lie to a palette colour to be
// that colour as it is: about one just-noticeable difference, wider than
// the 8-bit rounding of a conformed texel through the frame's targets.
const PALETTE_KEEP: f32 = 0.02;

// The nearest palette colour to `c` and its squared Oklab distance.
fn nearest_palette(c: vec3<f32>, count: i32) -> vec4<f32> {
  let want = oklab(c);
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
  return vec4<f32>(best, best_d);
}

// The pixel's colour in the lit scene before exposure and the tone curve
// (scene_colour). That image is at the render's size (smaller under TAAU)
// and the pass at its target's: the pixel's centre in its texels.
fn drawn_at(p: vec2<f32>) -> vec3<f32> {
  let ssize = vec2<i32>(textureDimensions(scene_colour));
  let at = vec2<i32>((p * vec2<f32>(ssize)) / vec2<f32>(textureDimensions(source)));
  return textureLoad(scene_colour, clamp(at, vec2<i32>(0), ssize - 1), 0).rgb;
}

// Whether a pixel drawn as `drawn` (drawn_at) is conformed art as the
// cook made it: lit past white is light on the art, not the art.
fn is_conformed(drawn: vec3<f32>) -> bool {
  return all(drawn <= vec3<f32>(1.001)) && conformed(max(drawn, vec3<f32>(0.0)));
}

fn quantize(in: VsOut) -> vec3<f32> {
  let px = vec2<i32>(in.pos.xy);
  var c = load(px);
  let count = i32(st.offset.w);
  // Conformed art as the cook made it (unlit, its own palette's colour,
  // told by the scene's linear colour before exposure, the tone curve,
  // ink, hatching or paint changed it, so a tone-mapped look keeps it too):
  // never posterised or dithered, only moved onto the frame's palette
  // when the look has one. Everything else (lit tones, plain or
  // unconformed materials) takes the look's posterise and dither below,
  // the fallback that makes a look look like itself on any material.
  let drawn = drawn_at(in.pos.xy);
  // Lit past white is quantised (is_conformed). Two tones
  // per channel (a 1-bit look) are the dither's to shade with, so its
  // conformed art dithers too: a lit white wall kept white read as a hole
  // punched through the dither, with no form.
  let one_bit = st.quantize.x > 1.5 && st.quantize.x < 2.5;
  if (!one_bit && is_conformed(drawn)) {
    if (count > 0) { return nearest_palette(c, count).rgb; }
    return c;
  }
  // A pixel already on a palette colour (art the cook put on the palette,
  // unlit) is that colour: neither posterised nor dithered, so a flat
  // field of a palette colour stays flat. Only the tones light, fog or
  // bloom made between the palette's colours are stepped and dithered.
  if (count > 0) {
    let kept = nearest_palette(c, count);
    if (kept.w <= PALETTE_KEEP * PALETTE_KEEP) {
      return kept.rgb;
    }
  }
  let d = (dither_at(in.pos.xy) - 0.5) * st.quantize.z;
  if (st.quantize.x >= 2.0) {
    // Steps in display space, the dither spread across one step.
    let n = st.quantize.x - 1.0;
    let e = srgb_encode(clamp(c, vec3<f32>(0.0), vec3<f32>(1.0)));
    c = srgb_decode(clamp(floor(e * n + 0.5 + d) / n, vec3<f32>(0.0), vec3<f32>(1.0)));
  }
  if (count > 0) {
    // The nearest palette colour of the dithered colour (the dither moves
    // it by a share of a palette step, so neighbours alternate).
    let shifted = clamp(c + vec3<f32>(d * 0.12), vec3<f32>(0.0), vec3<f32>(1.0));
    c = nearest_palette(shifted, count).rgb;
  }
  return c;
}

// Lens distortion (effects.rs `LensDistortion`): each pixel shows the image
// at the centre plus its offset scaled by 1 + amount r², r being 1 at the
// corners of a centred lens whatever the aspect, times the scale that
// keeps a barrel's farthest corner on the image; black beyond the image
// (a centre off the frame), nearest under a pixel grid.
fn lens(in: VsOut) -> vec3<f32> {
  let size = st.screen.xy;
  let c = st.lens.zw;
  let d = (in.uv - c) * size;
  let r2 = dot(d, d) / dot(size * 0.5, size * 0.5);
  let uv = c + (in.uv - c) * (1.0 + st.lens.x * r2) * st.lens.y;
  if (any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0))) {
    return vec3<f32>(0.0);
  }
  if (st.output.z > 1.5) {
    return textureLoad(source, clamp(vec2<i32>(floor(uv * size)), vec2<i32>(0), vec2<i32>(size) - 1), 0).rgb;
  }
  return sample_exact(source, clamp_sampler, uv).rgb;
}

@fragment
fn fs_lens(in: VsOut) -> @location(0) vec4<f32> {
  return vec4<f32>(lens(in), 1.0);
}

@fragment
fn fs_lens_srgb(in: VsOut) -> @location(0) vec4<f32> {
  return vec4<f32>(srgb_encode(clamp(lens(in), vec3<f32>(0.0), vec3<f32>(1.0))), 1.0);
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
  // The render scale's frame (output.z 0): the whole output, bilinear.
  if (st.output.z < 0.5) {
    return sample_exact(source, clamp_sampler, in.uv).rgb;
  }
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
