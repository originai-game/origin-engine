// Stochastic screen-space reflections (ssr.rs): compute passes between the
// depth-normal prepass and the main pass, whose lit shaders blend the
// result over the reflection probes' radiance (pbr_fs.wgsl, physical.wgsl).
//
// 1. hiz_base, hiz_reduce: the prepass's view depth and its min-max depth
//    pyramid.
// 2. trace: one ray a pixel (or a 2x2 block at half resolution), its
//    direction drawn from the GGX distribution of visible normals (Dupuy
//    and Benyoub 2023, spherical caps), marched through the pyramid
//    (FidelityFX SSSR's hierarchical march, MIT, third_party/fidelityfx-sssr)
//    and validated as FidelityFX does. A hit reads the light the previous
//    frame's lit image shows where the hit point was then.
// 3. resolve: every pixel weighs its neighbours' rays by its own BRDF over
//    each ray's pdf (Stachowiak 2015, "Stochastic Screen-Space
//    Reflections"), the ray reuse that turns one ray into a glossy lobe.
// 4. prefilter: the FidelityFX reflection denoiser's spatial filter, an
//    edge-stopping blur over a Halton set of neighbours weighed by normal,
//    depth and distance from the local mean, wider as the lobe widens.
// 5. temporal: the FidelityFX reflection denoiser's reprojection (the hit
//    point's parallax for mirrors, the surface's motion otherwise, with its
//    disocclusion test) and temporal resolve (history clipped to the
//    neighbourhood's moments, accumulated up to a roughness-dependent
//    sample count).
//
// Everything is premultiplied by the confidence: rgb is the reflected
// radiance times how much of the lobe the screen resolved, a that share; a
// lit shader adds rgb and keeps (1 - a) of the probes' reflection.

struct Ssr {
  proj: mat4x4<f32>,
  inv_proj: mat4x4<f32>,
  view: mat4x4<f32>,           // world -> view
  inv_view: mat4x4<f32>,       // view -> world
  prev_view_proj: mat4x4<f32>, // world -> the previous frame's clip space
  screen: vec4<f32>,           // the scene's w, h, 1/w, 1/h
  trace: vec4<f32>,            // x: max distance, y: thickness, z: march iterations, w: max roughness
  params: vec4<f32>,           // x: edge fade, y: near, z: far, w: the frame's noise index
  flags: vec4<f32>,            // x: 1 with the previous lit image, y: 1 with the history, z: the ray divisor (1 or 2), w: pyramid levels
  ray: vec4<f32>,              // xy: the ray textures' size, zw: the half-resolution ray's pixel in its 2x2 block
  chain: vec4<f32>,            // the denoiser's grid (the rays'): w, h, 1/w, 1/h
};

@group(0) @binding(0) var<uniform> ssr: Ssr;
@group(0) @binding(1) var ssr_depth: texture_2d<f32>;    // lindepth: r view depth, g reflectivity (pbr_prepass.wgsl)
@group(0) @binding(2) var ssr_gbuffer: texture_2d<f32>;  // xy oct normal, zw velocity (uv per frame)
@group(0) @binding(3) var ssr_linear: sampler;

// The pyramid's four levels a dispatch writes (hiz_down), and the level it
// reads (the second dispatch's).
@group(1) @binding(0) var hiz_out0: texture_storage_2d<rg32float, write>;
@group(1) @binding(19) var hiz_out1: texture_storage_2d<rg32float, write>;
@group(1) @binding(20) var hiz_out2: texture_storage_2d<rg32float, write>;
@group(1) @binding(21) var hiz_out3: texture_storage_2d<rg32float, write>;
@group(1) @binding(1) var hiz_in: texture_2d<f32>;
// Levels 1 to 8 of the pyramid as mips 0 to 7, r the nearest view depth
// under each texel and g the farthest; level 0 is the prepass's view depth
// itself.
@group(1) @binding(2) var hiz: texture_2d<f32>;
@group(1) @binding(3) var prev_lit: texture_2d<f32>;
@group(1) @binding(4) var ray_radiance_out: texture_storage_2d<rgba16float, write>;
@group(1) @binding(5) var ray_hit_out: texture_storage_2d<rgba16float, write>;
@group(1) @binding(6) var ray_radiance: texture_2d<f32>;
@group(1) @binding(7) var ray_hit: texture_2d<f32>;
@group(1) @binding(8) var resolved_out: texture_storage_2d<rgba16float, write>;
@group(1) @binding(9) var resolved: texture_2d<f32>;
@group(1) @binding(10) var history: texture_2d<f32>;
@group(1) @binding(11) var history_meta: texture_2d<f32>;
@group(1) @binding(12) var history_out: texture_storage_2d<rgba16float, write>;
@group(1) @binding(13) var meta_out: texture_storage_2d<rgba16float, write>;
// The fast history (NRD's anti-lag): the neighbourhood's mean over the
// last four frames, this frame's out and last frame's.
@group(1) @binding(14) var fast_out: texture_storage_2d<rgba16float, write>;
@group(1) @binding(24) var history_fast: texture_2d<f32>;
@group(1) @binding(15) var filtered_out: texture_storage_2d<rgba16float, write>;
// The temporal pass's raw resolve, before the prefilter: its neighbourhood
// moments (the prefiltered signal's are correlated across pixels).
@group(1) @binding(16) var unfiltered: texture_2d<f32>;
// The neighbourhood's variance, accumulated over frames (SVGF's temporal
// moments): last frame's, and this frame's out.
@group(1) @binding(17) var history_spread: texture_2d<f32>;
@group(1) @binding(18) var spread_out: texture_storage_2d<rgba16float, write>;
// The denoiser's guide, one texel a pixel: xy the world normal (oct), z
// the view depth, w the roughness. The trace writes it from the prepass's
// two targets, each ray texel for the pixels it spans, so a neighbour's
// edge-stopping inputs are one load, not two.
@group(1) @binding(22) var guide_out: texture_storage_2d<rgba16float, write>;
@group(1) @binding(23) var guide: texture_2d<f32>;
// At half resolution, the full-resolution reflections the lit shaders
// read, upsampled from the history.
@group(1) @binding(25) var lit_out: texture_storage_2d<rgba16float, write>;

const SSR_PI: f32 = 3.14159265359;
const SSR_BIG: f32 = 3.0e38;
// Below this roughness a surface is a mirror: its own ray only, and the
// hit point's parallax for its history.
const SSR_MIRROR: f32 = 0.05;
// The share of the visible normals' cap, from its rim, a ray never samples.
const SSR_LOBE_BIAS: f32 = 0.3;
// The FidelityFX denoiser's disocclusion constants.
const SSR_DISOCCLUSION_NORMAL: f32 = 1.4;
const SSR_DISOCCLUSION_DEPTH: f32 = 1.0;
const SSR_DISOCCLUSION_THRESHOLD: f32 = 0.9;

fn ssr_size() -> vec2<i32> {
  return vec2<i32>(ssr.screen.xy);
}

fn ssr_clamp_px(px: vec2<i32>) -> vec2<i32> {
  return clamp(px, vec2<i32>(0), ssr_size() - 1);
}

// The denoiser's grid: the screen at full resolution, its 2x2 blocks at
// half.
fn ssr_chain_size() -> vec2<i32> {
  return vec2<i32>(ssr.chain.xy);
}

fn ssr_clamp_chain(c: vec2<i32>) -> vec2<i32> {
  return clamp(c, vec2<i32>(0), ssr_chain_size() - 1);
}

// The pixel that stands for denoiser texel `c` (its block's first): its
// depth, normal and roughness are the texel's, the same every frame.
fn ssr_rep(c: vec2<i32>) -> vec2<i32> {
  return ssr_clamp_px(c * i32(ssr.flags.z));
}

fn ssr_view_depth(px: vec2<i32>) -> f32 {
  return textureLoad(ssr_depth, ssr_clamp_px(px), 0).r;
}

fn ssr_is_sky(d: f32) -> bool {
  return d >= ssr.params.z * 0.999;
}

// The prepass's roughness (pbr_prepass.wgsl `reflectivity_bits`: the gloss
// in the low byte, the metalness above it, a painterly byte above both).
fn ssr_roughness(px: vec2<i32>) -> f32 {
  return ssr_decode_roughness(textureLoad(ssr_depth, ssr_clamp_px(px), 0).g);
}

fn ssr_decode_roughness(packed: f32) -> f32 {
  let g = packed - floor(packed / 65536.0) * 65536.0;
  let metal = floor(g / 256.0);
  return 1.0 - (g - metal * 256.0) / 255.0;
}

fn ssr_world_normal(px: vec2<i32>) -> vec3<f32> {
  return oct_decode(textureLoad(ssr_gbuffer, ssr_clamp_px(px), 0).xy);
}

fn ssr_guide(c: vec2<i32>) -> vec4<f32> {
  return textureLoad(guide, ssr_clamp_chain(c), 0);
}

fn ssr_view_normal(px: vec2<i32>) -> vec3<f32> {
  return normalize((ssr.view * vec4<f32>(ssr_world_normal(px), 0.0)).xyz);
}

fn ssr_velocity(px: vec2<i32>) -> vec2<f32> {
  return textureLoad(ssr_gbuffer, ssr_clamp_px(px), 0).zw;
}

// The view-space point at `uv` whose view depth is `d`.
fn ssr_view_pos(uv: vec2<f32>, d: f32) -> vec3<f32> {
  return view_point(ssr.inv_proj, uv, d);
}

// A view-space point as (uv, q) with q = 1 / z = -1 / depth, which is affine
// along a line on the screen (as the depth buffer's z is), so a ray is a
// straight segment in this space and the pyramid's depths compare in it.
fn ssr_screen(p: vec3<f32>) -> vec3<f32> {
  let c = ssr.proj * vec4<f32>(p, 1.0);
  let ndc = c.xy / c.w;
  return vec3<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5, 1.0 / p.z);
}

fn ssr_luma(c: vec3<f32>) -> f32 {
  return dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
}

// PCG3D (Jarzynski and Olano 2020, "Hash Functions for GPU Rendering"):
// three decorrelated words from pixel, frame and a stream.
fn ssr_pcg3d(seed: vec3<u32>) -> vec3<u32> {
  var v = seed * 1664525u + 1013904223u;
  v.x += v.y * v.z;
  v.y += v.z * v.x;
  v.z += v.x * v.y;
  v ^= v >> vec3<u32>(16u);
  v.x += v.y * v.z;
  v.y += v.z * v.x;
  v.z += v.x * v.y;
  return v;
}

// Three uniforms for pixel `px` on this frame, from stream `stream`: white
// noise that differs every frame (the temporal pass averages the frames'
// rays), with none of interleaved gradient noise's diagonal structure,
// which the ray reuse turned into hatching on rough surfaces.
fn ssr_rand(px: vec2<i32>, stream: u32) -> vec3<f32> {
  let h = ssr_pcg3d(vec3<u32>(vec2<u32>(px), u32(ssr.params.w) * 4u + stream));
  return vec3<f32>(h >> vec3<u32>(8u)) * (1.0 / 16777216.0);
}

// The lobe sample of pixel `px` on this frame: the R2 sequence (Roberts
// 2018, the plastic constant's 2D Kronecker sequence) over frames, shifted
// by a fixed per-pixel offset (Cranley-Patterson rotation). Each pixel's
// frames stratify its lobe, so the temporal pass's average converges
// nearly as 1/N where white noise left low-frequency mottle at 1/sqrt(N),
// and neighbouring pixels stay decorrelated (white in space).
fn ssr_lobe_sample(px: vec2<i32>) -> vec2<f32> {
  let h = ssr_pcg3d(vec3<u32>(vec2<u32>(px), 0x9e37u));
  let shift = vec2<f32>(h.xy >> vec2<u32>(8u)) * (1.0 / 16777216.0);
  let r2 = vec2<f32>(0.7548776662466927, 0.5698402909980532) * ssr.params.w;
  return fract(shift + r2);
}

// GGX: the distribution, Smith's masking of one direction, and the
// height-correlated visibility (Heitz 2014), for alpha = roughness².
fn ssr_d(n_dot_h: f32, a2: f32) -> f32 {
  let k = n_dot_h * n_dot_h * (a2 - 1.0) + 1.0;
  return a2 / (SSR_PI * k * k);
}

fn ssr_g1(n_dot_v: f32, a2: f32) -> f32 {
  return 2.0 * n_dot_v / (n_dot_v + sqrt(a2 + (1.0 - a2) * n_dot_v * n_dot_v));
}

fn ssr_vis(n_dot_v: f32, n_dot_l: f32, a2: f32) -> f32 {
  let gv = n_dot_l * sqrt(n_dot_v * n_dot_v * (1.0 - a2) + a2);
  let gl = n_dot_v * sqrt(n_dot_l * n_dot_l * (1.0 - a2) + a2);
  return 0.5 / max(gv + gl, 1e-6);
}

fn ssr_alpha(roughness: f32) -> f32 {
  return max(roughness * roughness, 1e-3);
}

// An orthonormal basis about `n` (Duff et al. 2017, "Building an
// Orthonormal Basis, Revisited").
fn ssr_basis(n: vec3<f32>) -> mat3x3<f32> {
  let s = select(-1.0, 1.0, n.z >= 0.0);
  let a = -1.0 / (s + n.z);
  let b = n.x * n.y * a;
  let t = vec3<f32>(1.0 + s * n.x * n.x * a, s * b, -s * n.x);
  let bt = vec3<f32>(b, s + n.y * n.y * a, -n.y);
  return mat3x3<f32>(t, bt, n);
}

// A microfacet normal from the visible normals of `wi` (tangent space,
// z up) for `alpha`: Dupuy and Benyoub 2023, "Sampling Visible GGX Normals
// with Spherical Caps".
fn ssr_vndf(wi: vec3<f32>, alpha: f32, u: vec2<f32>) -> vec3<f32> {
  let wi_std = normalize(vec3<f32>(wi.xy * alpha, wi.z));
  let phi = 2.0 * SSR_PI * u.x;
  let z = (1.0 - u.y) * (1.0 + wi_std.z) - wi_std.z;
  let sin_theta = sqrt(clamp(1.0 - z * z, 0.0, 1.0));
  let h_std = vec3<f32>(sin_theta * cos(phi), sin_theta * sin(phi), z) + wi_std;
  return normalize(vec3<f32>(h_std.xy * alpha, max(h_std.z, 1e-6)));
}

// ---- 1. The min-max depth pyramid.

// After FidelityFX SPD (MIT): a workgroup reduces a 16x16 block of the
// level below to its next four levels in workgroup memory, so the whole
// pyramid is two dispatches, not one barrier-bound dispatch a level. The
// pyramid spans the screen padded to 256 texels, where every level halves
// exactly; past the screen it holds no surface.
var<workgroup> hiz_lds: array<vec2<f32>, 64>;

// The nearest and the farthest of four (min, max) pairs.
fn hiz_join(a: vec2<f32>, b: vec2<f32>, c: vec2<f32>, d: vec2<f32>) -> vec2<f32> {
  return vec2<f32>(min(min(a.x, b.x), min(c.x, d.x)), max(max(a.y, b.y), max(c.y, d.y)));
}

fn hiz_down(first: vec2<f32>, lid: vec2<u32>, wg: vec2<u32>) {
  // first: this invocation's texel of the first level written.
  textureStore(hiz_out0, wg * 8u + lid, vec4<f32>(first, 0.0, 1.0));
  hiz_lds[lid.y * 8u + lid.x] = first;
  workgroupBarrier();
  var n = 4u;
  for (var k = 1u; k < 4u; k++) {
    var v = vec2<f32>(SSR_BIG, 0.0);
    let on = all(lid < vec2<u32>(n));
    if (on) {
      let c = lid * 2u;
      v = hiz_join(hiz_lds[c.y * 8u + c.x], hiz_lds[c.y * 8u + c.x + 1u],
                   hiz_lds[(c.y + 1u) * 8u + c.x], hiz_lds[(c.y + 1u) * 8u + c.x + 1u]);
    }
    workgroupBarrier();
    if (on) {
      hiz_lds[lid.y * 8u + lid.x] = v;
      let at = wg * n + lid;
      let o = vec4<f32>(v, 0.0, 1.0);
      if (k == 1u) { textureStore(hiz_out1, at, o); }
      if (k == 2u) { textureStore(hiz_out2, at, o); }
      if (k == 3u) { textureStore(hiz_out3, at, o); }
    }
    workgroupBarrier();
    n = n / 2u;
  }
}

// Levels 1 to 4 from the prepass's view depth.
@compute @workgroup_size(8, 8)
fn hiz_base(@builtin(local_invocation_id) lid: vec3<u32>, @builtin(workgroup_id) wg: vec3<u32>) {
  let p = vec2<i32>((wg.xy * 8u + lid.xy) * 2u);
  // Past the screen: no surface (nearest at infinity, farthest at 0).
  var v = vec2<f32>(SSR_BIG, 0.0);
  for (var y = 0; y < 2; y++) {
    for (var x = 0; x < 2; x++) {
      let q = p + vec2<i32>(x, y);
      if (all(q < ssr_size())) {
        let d = textureLoad(ssr_depth, q, 0).r;
        v = vec2<f32>(min(v.x, d), max(v.y, d));
      }
    }
  }
  hiz_down(v, lid.xy, wg.xy);
}

// Levels 5 to 8 from level 4.
@compute @workgroup_size(8, 8)
fn hiz_reduce(@builtin(local_invocation_id) lid: vec3<u32>, @builtin(workgroup_id) wg: vec3<u32>) {
  let p = (wg.xy * 8u + lid.xy) * 2u;
  let v = hiz_join(textureLoad(hiz_in, p, 0).rg, textureLoad(hiz_in, p + vec2<u32>(1u, 0u), 0).rg,
                   textureLoad(hiz_in, p + vec2<u32>(0u, 1u), 0).rg, textureLoad(hiz_in, p + vec2<u32>(1u, 1u), 0).rg);
  hiz_down(v, lid.xy, wg.xy);
}

// ---- 2. The trace.

// The nearest and the farthest view depth the pyramid holds at `pos`
// (screen space) on `level`.
fn ssr_hiz(pos: vec2<f32>, res: vec2<f32>, level: i32) -> vec2<f32> {
  if (level == 0) {
    return vec2<f32>(ssr_view_depth(vec2<i32>(floor(pos * res))));
  }
  let dims = vec2<i32>(textureDimensions(hiz, level - 1));
  let cell = clamp(vec2<i32>(floor(pos * res)), vec2<i32>(0), dims - 1);
  return textureLoad(hiz, cell, level - 1).rg;
}

// FidelityFX SSSR's hierarchical march (ffx_sssr.h, MIT) from `origin`
// along `dir` (both screen space, (uv, q)) up to `t_max` of `dir`: the
// point it stops at, w 1 where it reached a surface on the finest level, 2
// where it came out from behind an occluder into a surface it crossed
// while hidden, 0 where it reached none.
// The pyramid is min-max (Uludag 2014, "Hi-Z Screen-Space Cone-Traced
// Reflections", GPU Pro 5): a ray more than the thickness behind the
// farthest surface of a cell where it enters and where it leaves passed
// behind everything there, and crosses the cell on that level and climbs,
// as a ray in front of it does. FidelityFX's min-only march went down to
// the pixels instead and stopped on the first: the thickness test then
// threw the hit away (a hole where the ray's neighbours reflect what lies
// past the occluder), or a ray behind a wide occluder crept along it a
// pixel a step until its iterations ran out (another hole).
fn ssr_march(origin: vec3<f32>, dir: vec3<f32>, t_max: f32) -> vec4<f32> {
  let size0 = ssr.screen.xy;
  let levels = i32(ssr.flags.w);
  let iterations = u32(ssr.trace.z);
  let thickness = ssr.trace.y;
  let inv_dir = vec3<f32>(
    select(SSR_BIG, 1.0 / dir.x, dir.x != 0.0),
    select(SSR_BIG, 1.0 / dir.y, dir.y != 0.0),
    select(SSR_BIG, 1.0 / dir.z, dir.z != 0.0));
  // Boundaries a hair past the cell edge the ray leaves through, so a step
  // lands inside the next cell.
  let uv_offset = select(vec2<f32>(0.005), vec2<f32>(-0.005), dir.xy < vec2<f32>(0.0)) / size0;
  let floor_offset = select(vec2<f32>(1.0), vec2<f32>(0.0), dir.xy < vec2<f32>(0.0));
  var level = 0;
  var res = size0;
  // First out of the starting pixel, so the ray does not hit itself.
  var t: f32;
  {
    let plane = (floor(origin.xy * res) + floor_offset) / res + uv_offset;
    let tt = (plane - origin.xy) * inv_dir.xy;
    t = min(tt.x, tt.y);
  }
  var pos = origin + dir * t;
  var i = 0u;
  // The ray's view depth where it went behind an occluder, 0 while it is in
  // view.
  var hidden_from = 0.0;
  var emerged = false;
  while (i < iterations && level >= 0 && t <= t_max) {
    let cell = ssr_hiz(pos.xy, res, level);
    let surface = -1.0 / max(cell.x, 1e-6);
    let plane = (floor(pos.xy * res) + floor_offset) / res + uv_offset;
    var tt = (vec3<f32>(plane, surface) - origin) * inv_dir;
    // The depth plane only stops a ray moving away from the camera.
    if (dir.z <= 0.0) { tt.z = SSR_BIG; }
    let t_cell = min(tt.x, tt.y);
    let above = surface > pos.z;
    // Behind the cell's farthest surface by more than the thickness where
    // the ray enters the cell and where it leaves it (view depth is -1/q).
    let depth = -1.0 / pos.z;
    let back = cell.y + thickness;
    let through = !above && depth > back && -1.0 / (origin.z + dir.z * t_cell) > back;
    // Out from behind an occluder and behind a surface it was still in
    // front of when it went hidden (by any amount, the thickness is the
    // occluder's): it crossed that surface's depth where the occluder hides
    // it, and hits there. Every pixel of the cell lies past the occluder,
    // so the one it enters by is the first pixel of the surface past the
    // occluder's edge, on any level.
    if (!above && hidden_from > 0.0 && cell.x > hidden_from) {
      emerged = true;
      level = -1;
      break;
    }
    if (through && hidden_from == 0.0) { hidden_from = depth; }
    if (above) { hidden_from = 0.0; }
    // Through the cell's side without meeting its nearest surface, or
    // behind all of it: up a level; else down to look closer.
    let skipped = (above && t_cell < tt.z) || through;
    if (above) { t = min(t_cell, tt.z); }
    if (through) { t = t_cell; }
    pos = origin + dir * t;
    if (skipped) {
      level = min(level + 1, levels - 1);
    } else {
      level -= 1;
    }
    res = size0 * exp2(-f32(max(level, 0)));
    i++;
  }
  let found = level < 0 && t <= t_max;
  return vec4<f32>(pos, select(0.0, select(1.0, 2.0, emerged), found));
}

// The pixel a ray texel traces: itself, or at half resolution this frame's
// pixel of its 2x2 block.
fn ssr_ray_origin(rt: vec2<i32>) -> vec2<i32> {
  if (ssr.flags.z < 1.5) { return rt; }
  return ssr_clamp_px(rt * 2 + vec2<i32>(ssr.ray.zw));
}

// One ray a ray texel, and the guide of its denoiser texel.
@compute @workgroup_size(8, 8)
fn trace(@builtin(global_invocation_id) gid: vec3<u32>) {
  let rt = vec2<i32>(gid.xy);
  if (any(rt >= vec2<i32>(ssr.ray.xy))) { return; }
  {
    let q = ssr_rep(rt);
    let dr = textureLoad(ssr_depth, q, 0).rg;
    let oct = textureLoad(ssr_gbuffer, q, 0).xy;
    textureStore(guide_out, rt, vec4<f32>(oct, dr.r, ssr_decode_roughness(dr.g)));
  }
  let px = ssr_ray_origin(rt);
  let d = ssr_view_depth(px);
  let roughness = ssr_roughness(px);
  // w 0: no ray (the sky, a surface too rough to trace).
  if (ssr_is_sky(d) || roughness > ssr.trace.w) {
    textureStore(ray_radiance_out, rt, vec4<f32>(0.0));
    textureStore(ray_hit_out, rt, vec4<f32>(0.0));
    return;
  }
  let uv = (vec2<f32>(px) + 0.5) * ssr.screen.zw;
  let p = ssr_view_pos(uv, d);
  let n = ssr_view_normal(px);
  let v = -normalize(p);
  let alpha = ssr_alpha(roughness);
  let a2 = alpha * alpha;
  let basis = ssr_basis(n);
  let wi = vec3<f32>(dot(v, basis[0]), dot(v, basis[1]), max(dot(v, n), 1e-4));
  var l: vec3<f32>;
  var pdf: f32;
  if (roughness < SSR_MIRROR) {
    l = reflect(-v, n);
    pdf = 65000.0;
  } else {
    // The lobe's grazing rim left out (Stachowiak 2015's sampling bias,
    // UE's BRDF bias): its rare rays, weighted up by their small pdf,
    // reached bright things far off the lobe's core and lit a pixel and
    // its reuse ring as speckles no denoiser could average out in time.
    let u = ssr_lobe_sample(px) * vec2<f32>(1.0, 1.0 - SSR_LOBE_BIAS);
    let h = ssr_vndf(wi, alpha, u);
    let wo = reflect(-wi, h);
    // Below the horizon: the mirror direction instead (rare at grazing angles).
    if (wo.z <= 1e-3) {
      l = reflect(-v, n);
      pdf = 65000.0;
    } else {
      l = normalize(basis * wo);
      pdf = min(ssr_d(h.z, a2) * ssr_g1(wi.z, a2) / (4.0 * wi.z), 65000.0);
    }
  }
  let origin = p + n * (0.01 + 0.002 * d);
  // The ray's segment: its max distance, cut at the near plane.
  var reach = ssr.trace.x;
  let near_z = -ssr.params.y;
  if (l.z > 0.0) { reach = min(reach, 0.999 * (near_z - origin.z) / l.z); }
  let end = origin + l * max(reach, 0.0);
  let s0 = ssr_screen(origin);
  let s1 = ssr_screen(end);
  let dir = s1 - s0;
  // Where the segment leaves the screen.
  var t_max = 1.0;
  if (dir.x > 0.0) { t_max = min(t_max, (1.0 - s0.x) / dir.x); }
  if (dir.x < 0.0) { t_max = min(t_max, -s0.x / dir.x); }
  if (dir.y > 0.0) { t_max = min(t_max, (1.0 - s0.y) / dir.y); }
  if (dir.y < 0.0) { t_max = min(t_max, -s0.y / dir.y); }
  var radiance = vec4<f32>(0.0);
  var hit_view = end;
  let march = ssr_march(s0, dir, t_max);
  if (march.w > 0.5) {
    let hit = march.xyz;
    let hit_px = vec2<i32>(hit.xy * ssr.screen.xy);
    let surface_depth = ssr_view_depth(hit_px);
    // A ray out from behind an occluder met the surface where it is
    // hidden, at about the depth it shows past the occluder's edge.
    let emerged = march.w > 1.5;
    let ray_depth = select(-1.0 / hit.z, surface_depth, emerged);
    hit_view = ssr_view_pos(hit.xy, ray_depth);
    // FidelityFX's validation: not on the starting pixel, not the sky, not
    // a back face, and within the thickness behind the surface.
    let moved = any(abs(hit.xy - uv) >= 2.0 * ssr.screen.zw);
    let back = dot(ssr_view_normal(hit_px), l) > 0.0;
    if (moved && !ssr_is_sky(surface_depth) && !back) {
      let behind = 1.0 - smoothstep(0.0, ssr.trace.y, abs(ray_depth - surface_depth));
      let fade = max(ssr.params.x, 1e-3);
      let border = smoothstep(vec2<f32>(0.0), vec2<f32>(fade), hit.xy)
        * (1.0 - smoothstep(vec2<f32>(1.0 - fade), vec2<f32>(1.0), hit.xy));
      let far = 1.0 - smoothstep(0.8, 1.0, length(hit_view - origin) / max(ssr.trace.x, 1e-3));
      var confidence = behind * behind * border.x * border.y * far;
      // The light the hit point sent last frame, where it was then: from
      // the middle of its pixel, as the march stops on the edge of the
      // pixel it enters by and a filtered read there is half the pixel the
      // ray came over (a bright rim where that was a lamp).
      let at = (vec2<f32>(hit_px) + 0.5) * ssr.screen.zw;
      let prev = at - ssr_velocity(hit_px);
      if (ssr.flags.x < 0.5 || any(prev < vec2<f32>(0.0)) || any(prev > vec2<f32>(1.0))) {
        confidence = 0.0;
      }
      if (confidence > 0.0) {
        let light = max(textureSampleLevel(prev_lit, ssr_linear, prev, 0.0).rgb, vec3<f32>(0.0));
        radiance = vec4<f32>(light, confidence);
      }
    }
  }
  textureStore(ray_radiance_out, rt, radiance);
  textureStore(ray_hit_out, rt, vec4<f32>(hit_view, pdf));
}

// ---- 3. The resolve: ray reuse.

// A rotated, roughness-scaled ring of neighbour rays (the first offsets
// of FidelityFX's Halton(2, 3) prefilter set within 2 texels).
const SSR_REUSE: array<vec2<f32>, 8> = array<vec2<f32>, 8>(
  vec2<f32>(0.0, 1.0), vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, 0.0), vec2<f32>(-1.0, 1.0),
  vec2<f32>(1.0, -1.0), vec2<f32>(0.0, -1.0), vec2<f32>(-1.0, 0.0), vec2<f32>(1.0, 1.0));

@compute @workgroup_size(8, 8)
fn resolve(@builtin(global_invocation_id) gid: vec3<u32>) {
  let px = vec2<i32>(gid.xy);
  if (any(px >= ssr_chain_size())) { return; }
  let rp = ssr_rep(px);
  let d = ssr_view_depth(rp);
  let roughness = ssr_roughness(rp);
  if (ssr_is_sky(d) || roughness > ssr.trace.w) {
    textureStore(resolved_out, px, vec4<f32>(0.0));
    return;
  }
  let uv = (vec2<f32>(rp) + 0.5) * ssr.screen.zw;
  let p = ssr_view_pos(uv, d);
  let world_n = ssr_world_normal(rp);
  let n = normalize((ssr.view * vec4<f32>(world_n, 0.0)).xyz);
  let v = -normalize(p);
  let n_dot_v = max(dot(n, v), 1e-4);
  let alpha = ssr_alpha(roughness);
  let a2 = alpha * alpha;
  let ray_size = vec2<i32>(ssr.ray.xy);
  let home = px;
  // Mirrors take their own ray (a neighbour's lies off their lobe); rougher
  // surfaces the ring, wider as the lobe widens.
  let count = select(9, 1, roughness < SSR_MIRROR);
  let angle = 2.0 * SSR_PI * ssr_rand(px, 1u).x;
  let rot = mat2x2<f32>(cos(angle), sin(angle), -sin(angle), cos(angle));
  let radius = mix(1.0, 2.5, smoothstep(0.1, 0.6, roughness));
  var sum_w = 0.0;
  var sum_c = 0.0;
  var sum_tc = 0.0;
  var sum_l = vec3<f32>(0.0);
  var ring = SSR_REUSE;
  for (var k = 0; k < count; k++) {
    var rt = home;
    if (k > 0) { rt = clamp(home + vec2<i32>(round(rot * ring[k - 1] * radius)), vec2<i32>(0), ray_size - 1); }
    let hit = textureLoad(ray_hit, rt, 0);
    if (hit.w <= 0.0) { continue; }
    // A neighbour's ray started on another surface only if its pixel
    // differs in depth or facing.
    let opx = ssr_ray_origin(rt);
    // (Normals compared in the world: the dot is the same in view space.)
    let facing = max(dot(ssr_world_normal(opx), world_n), 1e-6);
    var bilateral = exp(16.0 * log(facing) - abs(ssr_view_depth(opx) - d) / max(d, 1e-3) * 32.0);
    if (k == 0) { bilateral = 1.0; }
    let l = normalize(hit.xyz - p);
    let n_dot_l = dot(n, l);
    if (n_dot_l <= 0.0) { continue; }
    let h = normalize(v + l);
    let brdf = ssr_d(max(dot(n, h), 0.0), a2) * ssr_vis(n_dot_v, n_dot_l, a2) * n_dot_l;
    let w = brdf / hit.w * bilateral;
    let r = textureLoad(ray_radiance, rt, 0);
    // Weighted by 1 / (1 + luma) (Karis 2014) so a lone bright hit cannot
    // dominate the lobe as a firefly.
    let tw = 1.0 / (1.0 + ssr_luma(r.rgb));
    sum_w += w;
    sum_c += w * r.a;
    sum_tc += w * r.a * tw;
    sum_l += w * r.a * tw * r.rgb;
  }
  var out = vec4<f32>(0.0);
  if (sum_w > 0.0 && sum_tc > 0.0) {
    let share = sum_c / sum_w;
    out = vec4<f32>(sum_l / sum_tc * share, share);
  }
  // Fade to the probes as the surface nears the roughest traced.
  out *= 1.0 - smoothstep(ssr.trace.w * 0.75, ssr.trace.w, roughness);
  textureStore(resolved_out, px, out);
}

// ---- 4. The spatial prefilter.

// FidelityFX's prefilter offsets: the first 15 of Halton(2, 3) stretched
// to [-3, 3], the centre left out.
const SSR_HALTON: array<vec2<f32>, 15> = array<vec2<f32>, 15>(
  vec2<f32>(0.0, 1.0), vec2<f32>(-2.0, 1.0), vec2<f32>(2.0, -3.0), vec2<f32>(-3.0, 0.0),
  vec2<f32>(1.0, 2.0), vec2<f32>(-1.0, -2.0), vec2<f32>(3.0, 0.0), vec2<f32>(-3.0, 3.0),
  vec2<f32>(0.0, -3.0), vec2<f32>(-1.0, -1.0), vec2<f32>(2.0, 1.0), vec2<f32>(-2.0, -2.0),
  vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 2.0), vec2<f32>(3.0, -1.0));
// Its edge-stopping constants (ffx_denoiser_reflections_config.h).
const SSR_NORMAL_SIGMA: f32 = 512.0;
const SSR_DEPTH_SIGMA: f32 = 4.0;
const SSR_RADIANCE_BIAS: f32 = 0.6;

@compute @workgroup_size(8, 8)
fn prefilter(@builtin(global_invocation_id) gid: vec3<u32>) {
  ssr_prefilter(vec2<i32>(gid.xy), 1.0, 2u);
}

// At half resolution a second pass over the first's output at twice the
// step (an a-trous iteration, Dammertz et al. 2010, as SVGF's): a block
// grid has a quarter of the rays a screen area has at full resolution, so
// one pass left the rough lobes' noise as blotches of the filter's size,
// which crawled when the camera moved.
@compute @workgroup_size(8, 8)
fn prefilter_wide(@builtin(global_invocation_id) gid: vec3<u32>) {
  ssr_prefilter(vec2<i32>(gid.xy), 2.0, 3u);
}

// The edge-stopping filter of `resolved` into `filtered_out` at `step`
// times its footprint, rotated by random stream `stream`.
fn ssr_prefilter(px: vec2<i32>, step: f32, stream: u32) {
  if (any(px >= ssr_chain_size())) { return; }
  let center = textureLoad(resolved, px, 0);
  let own = ssr_guide(px);
  let d = own.z;
  let roughness = own.w;
  // A mirror keeps its own ray: a blur would soften what it reflects.
  if (ssr_is_sky(d) || roughness > ssr.trace.w || roughness < SSR_MIRROR) {
    textureStore(filtered_out, px, center);
    return;
  }
  let n = oct_decode(own.xy);
  // The local mean the radiance weight compares against, so a lone bright
  // sample is pulled down as FidelityFX's average radiance does: a 3x3
  // tent in four bilinear reads at the pixel's corners.
  let corner = vec2<f32>(px) * ssr.chain.zw;
  let mean = 0.25 * (textureSampleLevel(resolved, ssr_linear, corner, 0.0)
    + textureSampleLevel(resolved, ssr_linear, corner + vec2<f32>(ssr.chain.z, 0.0), 0.0)
    + textureSampleLevel(resolved, ssr_linear, corner + vec2<f32>(0.0, ssr.chain.w), 0.0)
    + textureSampleLevel(resolved, ssr_linear, corner + ssr.chain.zw, 0.0));
  // The footprint grows with the lobe: a pixel at roughness 0.6 reaches
  // twice FidelityFX's 3 texels.
  let scale = mix(0.75, 2.0, smoothstep(SSR_MIRROR, 0.6, roughness)) * step;
  let w0 = max(exp(-SSR_RADIANCE_BIAS * length(center.rgb - mean.rgb)), 1e-2);
  var sum = center * w0;
  var weight = w0;
  let angle = 2.0 * SSR_PI * ssr_rand(px, stream).x;
  let rot = mat2x2<f32>(cos(angle), sin(angle), -sin(angle), cos(angle));
  var taps = SSR_HALTON;
  for (var i = 0; i < 15; i++) {
    let q = ssr_clamp_chain(px + vec2<i32>(round(rot * taps[i] * scale)));
    let s = textureLoad(resolved, q, 0);
    let g = ssr_guide(q);
    // The edge-stopping weights' logarithms, summed under one exp: the
    // normal's power, the depth's and the radiance's falloffs (the last
    // floored at 1e-2 as FidelityFX's), and a neighbour of another
    // roughness, which lies on another lobe.
    let facing = max(dot(n, oct_decode(g.xy)), 1e-6);
    let w = exp(SSR_NORMAL_SIGMA * log(facing)
      - abs(d - g.z) / max(d, 1e-3) * SSR_DEPTH_SIGMA * 8.0
      - min(SSR_RADIANCE_BIAS * length(s.rgb - mean.rgb), 4.6052)
      - abs(g.w - roughness) * 16.0);
    sum += s * w;
    weight += w;
  }
  // The wide pass only where the lobe is wide: a near-mirror's blotches
  // are small and its detail is not.
  let wide = smoothstep(0.1, 0.25, roughness) * (step - 1.0) + (2.0 - step);
  textureStore(filtered_out, px, mix(center, sum / weight, wide));
}

// ---- 5. The temporal pass.

// FidelityFX's ClipAABB (after Playdead's TAA, MIT): `x` pulled towards the
// box's centre until it lies inside.
fn ssr_clip(lo: vec4<f32>, hi: vec4<f32>, x: vec4<f32>) -> vec4<f32> {
  let center = 0.5 * (hi + lo);
  let extent = 0.5 * (hi - lo) + 1e-3;
  let offset = x - center;
  let unit = abs(offset / extent);
  let m = max(max(unit.x, unit.y), max(unit.z, unit.w));
  return select(x, center + offset / m, m > 1.0);
}

fn ssr_disocclusion(n: vec3<f32>, history_n: vec3<f32>, d: f32, history_d: f32) -> f32 {
  return exp(-abs(1.0 - max(dot(n, history_n), 0.0)) * SSR_DISOCCLUSION_NORMAL)
    * exp(-abs(history_d - d) / max(d, 1e-3) * SSR_DISOCCLUSION_DEPTH);
}

// The history's meta at `uv`: x sample count, y view depth, zw oct normal.
fn ssr_meta(uv: vec2<f32>) -> vec4<f32> {
  return textureLoad(history_meta, ssr_clamp_chain(vec2<i32>(uv * ssr.chain.xy)), 0);
}

fn ssr_on_screen(uv: vec2<f32>) -> bool {
  return all(uv > vec2<f32>(0.0)) && all(uv < vec2<f32>(1.0));
}

@compute @workgroup_size(8, 8)
fn temporal(@builtin(global_invocation_id) gid: vec3<u32>) {
  let px = vec2<i32>(gid.xy);
  if (any(px >= ssr_chain_size())) { return; }
  let rp = ssr_rep(px);
  let d = ssr_view_depth(rp);
  let world_n = ssr_world_normal(rp);
  let roughness = ssr_roughness(rp);
  let center = textureLoad(resolved, px, 0);
  if (ssr_is_sky(d) || roughness > ssr.trace.w) {
    textureStore(history_out, px, vec4<f32>(0.0));
    textureStore(meta_out, px, vec4<f32>(0.0, d, oct_encode(world_n)));
    textureStore(fast_out, px, vec4<f32>(0.0));
    textureStore(spread_out, px, vec4<f32>(0.0));
    return;
  }
  // The neighbourhood's mean and deviation over FidelityFX's 9x9 window
  // (radius 4), Gaussian-weighted with its K of 3, read every second texel
  // within the diamond (13 taps; the corners' Gaussian weight is a fifth of
  // the centre's or less, and their loads were a fifth of the pass): a 3x3
  // window's moments are as noisy as one frame of a rough surface, and a
  // box clipped to them threw the converged history away.
  var m1 = vec4<f32>(0.0);
  var m2 = vec4<f32>(0.0);
  var mw = 0.0;
  for (var y = -2; y <= 2; y++) {
    for (var x = -2 + abs(y); x <= 2 - abs(y); x++) {
      let o = vec2<f32>(f32(x), f32(y)) * 2.0;
      let w = exp(-3.0 * dot(o, o) / 25.0);
      let s = textureLoad(unfiltered, ssr_clamp_chain(px + vec2<i32>(x, y) * 2), 0);
      m1 += s * w;
      m2 += s * s * w;
      mw += w;
    }
  }
  m1 /= mw;
  let variance = max(m2 / mw - m1 * m1, vec4<f32>(0.0));
  var spread = variance;
  var fast = m1;
  var out = center;
  var samples = 1.0;
  if (ssr.flags.y > 0.5) {
    let uv = (vec2<f32>(rp) + 0.5) * ssr.screen.zw;
    // The texel's own centre, where its history lies (the representative
    // pixel's is a quarter texel off it at half resolution).
    let uv_c = (vec2<f32>(px) + 0.5) * ssr.chain.zw;
    let p = ssr_view_pos(uv, d);
    // The surface's own motion, and for a mirror the reflected point's:
    // a ray of the reflection's length straight through the surface
    // (FidelityFX's hit position reprojection).
    let surface_uv = uv_c - ssr_velocity(rp);
    var history_uv = surface_uv;
    let hit = textureLoad(ray_hit, px, 0);
    if (roughness < 0.25 && hit.w > 0.0) {
      let through = p + normalize(p) * length(hit.xyz - p);
      let c = ssr.prev_view_proj * (ssr.inv_view * vec4<f32>(through, 1.0));
      let hit_uv = vec2<f32>(c.x / c.w * 0.5 + 0.5, 0.5 - c.y / c.w * 0.5) + uv_c - uv;
      let hit_meta = ssr_meta(hit_uv);
      let surface_meta = ssr_meta(surface_uv);
      let hit_similar = dot(oct_decode(hit_meta.zw), world_n);
      if (ssr_on_screen(hit_uv) && hit_similar > 0.999
          && hit_similar + 1e-3 > dot(oct_decode(surface_meta.zw), world_n)) {
        history_uv = hit_uv;
      }
    }
    if (ssr_on_screen(history_uv)) {
      let m = ssr_meta(history_uv);
      let kept = ssr_disocclusion(world_n, oct_decode(m.zw), d, m.y);
      if (kept > SSR_DISOCCLUSION_THRESHOLD && m.x > 0.0) {
        // How still the history is on screen: 1 for a still camera over a
        // still surface.
        let still = 1.0 - smoothstep(0.05, 0.5, length((history_uv - uv_c) * ssr.screen.xy));
        // A still history accumulates up to 64 frames on rough surfaces.
        let most = max(8.0, mix(32.0, 64.0, still) * (1.0 - exp(-roughness * 100.0)));
        samples = min(most, m.x * kept + 1.0);
        let old = textureSampleLevel(history, ssr_linear, history_uv, 0.0);
        let history_px = ssr_clamp_chain(vec2<i32>(history_uv * ssr.chain.xy));
        // The fast history: the neighbourhood's mean over the last four
        // frames (NRD's fast history, Zhdan 2021), far steadier than one
        // frame's mean, which on a rough lobe's heavy tail swings with
        // whether the window's rays met the bright thing it reflects.
        fast = mix(textureLoad(history_fast, history_px, 0), m1, 0.25);
        // The box's spread: the neighbourhood's variance averaged over the
        // last four frames along the same reprojection (SVGF, Schied et
        // al. 2017), so a frame whose window met none of the lobe's bright
        // tail does not collapse it.
        let old_spread = textureLoad(history_spread, history_px, 0);
        spread = mix(old_spread, variance, 1.0 / min(samples, 4.0));
        let sigma = sqrt(spread);
        // Clip the history to the neighbourhood: FidelityFX's 1.4
        // deviations, wider for rough surfaces whose single frame is
        // noisier. The moments are the raw resolve's: the prefiltered
        // signal's neighbours share taps, so its spread understates a
        // frame's noise. A still history's box is centred on the fast
        // mean: centred on one frame's, it clipped a converged reflection
        // towards that frame's noise every frame, and the grain never
        // left; with the spread averaged over eight frames instead, the
        // box stayed wide after a reflected object moved away, and its
        // reflection trailed for 30 frames. The fast mean follows a real
        // change within four.
        let k = 1.4 * (1.0 + roughness);
        let c = mix(m1, fast, still);
        let clipped = ssr_clip(c - sigma * k, c + sigma * k, old);
        // And this frame's outliers to one of this frame's deviations
        // (FidelityFX's "clip outliers"), so a lone bright hit in a lobe's
        // tail is not kept as a firefly: the accumulated spread is wide
        // there and would let it through.
        let now = sqrt(variance);
        let fresh = ssr_clip(m1 - now, m1 + now, center);
        // Blended with each side weighted by 1 / (1 + luma) (Karis 2014,
        // "High Quality Temporal Supersampling"): a bright sample that
        // got through enters at a fraction of its weight instead of
        // lingering for the next 64 frames as a firefly.
        let a = 1.0 / samples;
        let wh = (1.0 - a) / (1.0 + ssr_luma(clipped.rgb));
        let wf = a / (1.0 + ssr_luma(fresh.rgb));
        out = (clipped * wh + fresh * wf) / max(wh + wf, 1e-6);
      }
    }
  }
  out = max(out, vec4<f32>(0.0));
  textureStore(history_out, px, out);
  textureStore(meta_out, px, vec4<f32>(samples, d, oct_encode(world_n)));
  textureStore(fast_out, px, fast);
  textureStore(spread_out, px, spread);
}

// ---- 6. The upsample (half resolution).

// Each pixel's reflection from the four denoiser texels round it, each
// weighed by its bilinear share and by how like the pixel its surface is
// (normal, depth, roughness; a joint bilateral upsample, Kopf et al. 2007):
// across an edge a pixel takes its own side's texels, not the blend of
// both. A pixel none of the four matches takes the likest.
@compute @workgroup_size(8, 8)
fn upsample(@builtin(global_invocation_id) gid: vec3<u32>) {
  let px = vec2<i32>(gid.xy);
  if (any(px >= ssr_size())) { return; }
  let d = ssr_view_depth(px);
  let roughness = ssr_roughness(px);
  if (ssr_is_sky(d) || roughness > ssr.trace.w) {
    textureStore(lit_out, px, vec4<f32>(0.0));
    return;
  }
  let n = ssr_world_normal(px);
  let c = (vec2<f32>(px) + 0.5) * ssr.chain.xy * ssr.screen.zw - 0.5;
  let base = vec2<i32>(floor(c));
  let f = c - vec2<f32>(base);
  var sum = vec4<f32>(0.0);
  var weight = 0.0;
  var likest = vec4<f32>(0.0);
  var likeness = -1.0;
  for (var k = 0; k < 4; k++) {
    let o = vec2<i32>(k & 1, k >> 1);
    let q = ssr_clamp_chain(base + o);
    let g = textureLoad(guide, q, 0);
    let facing = max(dot(n, oct_decode(g.xy)), 1e-6);
    let like = exp(16.0 * log(facing) - abs(g.z - d) / max(d, 1e-3) * 32.0
      - abs(g.w - roughness) * 16.0);
    let share = select(1.0 - f.x, f.x, o.x == 1) * select(1.0 - f.y, f.y, o.y == 1);
    let v = textureLoad(history, q, 0);
    sum += v * share * like;
    weight += share * like;
    if (like > likeness) {
      likeness = like;
      likest = v;
    }
  }
  textureStore(lit_out, px, select(likest, sum / weight, weight > 1e-4));
}
