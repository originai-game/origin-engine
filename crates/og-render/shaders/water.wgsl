// Planar water (the E5 round): a grid over the surface's local xz plane,
// displaced by an optional height field (og-physics' wave simulation),
// with animated normals from a sum of travelling waves, colour by depth
// (shallow to deep over depthFade, from the depth prepass), foam where the
// water meets geometry, refraction of a copy of the scene drawn so far,
// and reflection: screen-space (a march through the depth prepass), the
// sky's prefiltered cubemap where it misses, or the planar reflection
// pass's mirrored scene. Phase 2: the
// directional light's shadows on the surface, and the view from below
// (what is above inside Snell's window, the water's colour reflected
// outside it). Drawn after the opaque scene and the sky, before
// transparent meshes, writing depth. Prepended by frame.wgsl and
// lighting.wgsl.
//
// R5: a sea (Water::wind) is a clipmap about the camera (water.rs
// `clipmap`), out to a horizon ring, displaced over the absolute world's xz
// by every wave the tick floats bodies on (og_core::ocean, as Gerstner
// waves) and the FFT's cascades where the tick leaves their band out
// (ocean_fft), every one faded where the cells or the pixel cannot hold it;
// its normal and its facets' slope covariance from the cascades' slopes and
// their moments (LEAN). Shaded and foamed as web-ocean's sea (MIT,
// third_party/ocean/): anisotropic glitter, the grazing tilt of the visible
// facets, backlit crests' subsurface light, and foam that persists on the
// tick's time (ocean_fft/foam.rs). A flow map (Water::flow_map) turns the
// flow per point.

struct WaterUniform {
  model: mat4x4<f32>,
  size: vec4<f32>,        // xy: extent along local x and z, z: segments, w: 1 with a height field, 2 with its ripples
  shallow: vec4<f32>,     // rgb, w: depth fade (m)
  deep: vec4<f32>,        // rgb, w: opacity
  foam: vec4<f32>,        // rgb, w: foam width (m)
  waves: vec4<f32>,       // x: scale (waves per metre), y: speed, z: strength, w: refraction (px)
  look: vec4<f32>,        // x: reflectivity, y: 1 for screen-space reflections, z: height field size (m), w: resolution
  screen: vec4<f32>,      // w, h, 1/w, 1/h
  extra: vec4<f32>,       // x: shadows, y: 1 with the planar reflection, z: F0, w: reflection bias
  under: vec4<f32>,       // rgb: the underwater colour, w: three.js Water's glint (0: the half-vector highlight)
  normals: vec4<f32>,     // x: the normal map's repeats, yz: its flow (0: the lake's layers), w: 1 with a normal map
  ao: vec4<f32>,          // x: PostEffects.aoDirect, how much the screen AO darkens the reflection and the body (0: not at all), y: 1 with a flow map, z: 1 over a mesh
  layers: array<vec4<f32>, 2>,  // the lake's four layers' uv at the frame's origin, whole repeats dropped (xy, zw; origin.rs)
  ocean: vec4<f32>,       // x: 1 with a sea, y: the tick's grid's even cell (m), z: the Jacobian under which a crest foams as it folds, w: how fast it grows
  sea_wind: vec4<f32>,    // xy: the wind's direction in the frame's xz, z: its speed (m/s; 0 without a sea)
  clip: vec4<f32>,        // the sea's clipmap: x levels, y cells along a level's edge, z the horizon ring's radius, w 1 endless
  levels: array<vec4<f32>, 10>,  // per level: its centre (frame xz), its cell (m)
  inverse: mat4x4<f32>,   // the surface's frame from the world
  fft: vec4<f32>,         // x cascades, y the hulls in the sea, z the slope variance under the finest cascade
  cascades: array<vec4<f32>, 3>,  // per cascade: 1 / tile (1/m), its uv at the frame's origin, the tile (m)
  bands: vec4<f32>,       // each cascade's band's middle wavenumber (xyz; cascade_weight)
  // The persistent foam's two levels (ocean_fft/foam.rs), in the fields' layers after the cascades: per level 1 / its
  // edge (1/m), its uv at the frame's origin, its edge (m).
  foam_levels: array<vec4<f32>, 2>,
  // The foam micro-structure's uv at the frame's origin (whole repeats dropped): its lace (xy), finer lace (zw); its
  // bubbles (xy), its streaks along and across the wind (zw).
  foam_uv: array<vec4<f32>, 2>,
  // The og.Buoyancy hulls in the sea (water.rs `hulls`, fft.y of them): per hull its centre (frame xz) and its own x
  // axis (xz), then its half extents along its axes.
  hulls: array<vec4<f32>, 32>,
  // Per wave of og_core::ocean: (k d.x, k d.z, amplitude, phase at the frame's origin), (lean d.x, lean d.z, k lean, 0)
  // (water.rs ocean_gpu).
  swell: array<vec4<f32>, 64>,
  // W1: the seabed (water.rs shore_gpu): [0] its sample (0, 0) in the frame's xz, its cell (m), 1 with one; [1] its
  // columns and rows, the shore trains' wavenumber and amplitude; [2] their angular frequency and phase now, the surf's
  // foam.
  shore: array<vec4<f32>, 3>,
};

@group(1) @binding(0) var<uniform> water: WaterUniform;

// The light a sea's crests carry through them (rgb) and its strength (w): web-ocean's `sssCol`.
const SEA_SSS: vec4<f32> = vec4<f32>(0.035, 0.12, 0.085, 2.6);
// The sun's disc as a variance of slopes (its 0.27° radius, halved at reflection, squared): the glitter's least
// spread, so a calm sea mirrors a disc, not a point.
const SUN_DISC: f32 = 5.5e-6;
// A height field (layer 0, the height in r), or a sea's FFT fields: per cascade c, layer 3c (x displacement,
// height, z displacement, d Dx/dz), 3c + 1 (the height's slopes along x and z, d Dx/dx, d Dz/dz), 3c + 2 (the slopes'
// squares and product), mipped; after the cascades the persistent foam's two levels (r its cover, g how fresh it is)
// and its micro-structure (r lace, g finer lace, b soft density, a streaks).
@group(1) @binding(1) var heights: texture_2d_array<f32>;
@group(1) @binding(2) var scene_color: texture_2d<f32>;
@group(1) @binding(3) var scene_depth: texture_2d<f32>;
@group(1) @binding(4) var water_sampler: sampler;
// The planar reflection pass's frame (the mirrored camera's view, flipped
// left to right), when this surface is the one drawn with it.
@group(1) @binding(5) var reflection_tex: texture_2d<f32>;
// The ripples' tangent-space normal map (Water::normal_map).
@group(1) @binding(6) var normal_tex: texture_2d<f32>;
@group(1) @binding(7) var ripple_sampler: sampler;
// The frame's screen-space AO (white without one).
@group(1) @binding(8) var ao_tex: texture_2d<f32>;
// The flow map (Water::flow_map): rg the flow's direction * 0.5 + 0.5.
@group(1) @binding(9) var flow_tex: texture_2d<f32>;
// The seabed (Water::shore, og_core::shore): per sample its depth under the rest level, the signed distance to the
// waterline and that distance's gradient (xz).
@group(1) @binding(10) var shore_tex: texture_2d<f32>;

struct WaterOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) local: vec2<f32>,
  @location(3) slope: f32,
  // Where the normal map is read: the mesh's uv, or the grid's metres.
  @location(4) uv: vec2<f32>,
  // The sea's rest point (frame xz) the displaced vertex came from.
  @location(5) rest: vec2<f32>,
};

// How much of a wave of wavenumber k a spacing of `cell` metres holds: all of one eight cells long, none under four
// (og_core::ocean::grid_keep).
fn sea_keep(k: f32, cell: f32) -> f32 {
  return 1.0 - smoothstep(0.785, 1.571, k * cell);
}

// The FFT cascade c's uv at rest point p (frame xz).
fn cascade_uv(c: u32, p: vec2<f32>) -> vec2<f32> {
  let t = water.cascades[c];
  return p * t.x + t.yz;
}

// The tick's grid cell (m) at rest point p (frame xz): og_core::ocean::grid_cell over the surface's size about the
// entity (og-physics' WaterBody::surface), the cell a floating body's waves are faded by. g w(g) = u solved by
// Newton's method for each axis, as there.
fn tick_cell(p: vec2<f32>) -> f32 {
  let half = max(water.size.xy * 0.5, vec2<f32>(1e-3));
  let u = clamp((p - water.model[3].xz) / half, vec2<f32>(-1.0), vec2<f32>(1.0));
  var g = u;
  for (var i = 0; i < 8; i++) {
    let w = g * (0.15 + 0.85 * g * g);
    g -= (w - u) / (0.15 + 2.55 * g * g);
  }
  let slope = 0.15 + 2.55 * g * g;
  return water.ocean.y * max(slope.x, slope.y);
}

// How much of cascade c's height is drawn where the tick's cell is `tc`: none where the tick's waves cover its band,
// all where the tick fades them (water.rs cascade_weight).
fn cascade_weight(c: u32, tc: f32) -> f32 {
  return 1.0 - sea_keep(water.bands[c], tc);
}

// The seabed's rows and the tick's waves for shore.wgsl (prepended).
fn sea_shore(i: u32) -> vec4<f32> {
  return water.shore[i];
}

fn sea_wave(i: u32) -> vec4<f32> {
  return water.swell[i];
}

// The sea's displacement (x, height, z) at rest point p (frame xz) as vertices `cell` metres apart draw it: every wave
// of og_core::ocean as the tick floats bodies on it (faded at the tick's cell here, or the vertices' where those are
// coarser, and shoaled by the seabed under p), then each cascade where the tick leaves its band out, read at the mip
// whose texels are the cells and faded once its longest waves are under four cells.
fn ocean_displace(p: vec2<f32>, cell: f32) -> vec3<f32> {
  let sh = shoaling(p);
  let swell = ocean_swell(p, cell, sh);
  return swell + ocean_cascades(p, cell, p + swell.xz, sh);
}

// ocean_displace's waves of og_core::ocean (the tick's) at rest point p, faded at the tick's cell where they carry p
// to: og-physics reads that cell at the floater's own point (WaterBody::surface), and in a gale the crests lean the
// rest point 2 to 3 m from it, where the tick's grid is some 3 % coarser or finer and the half-faded waves drew the
// surface up to 4.6 cm off og.physics.waterHeight. The cell at p places the point to within a few centimetres, where
// the cell is the same to a thousandth.
fn ocean_swell(p: vec2<f32>, cell: f32, sh: Shoal) -> vec3<f32> {
  let first = ocean_swell_at(p, max(cell, tick_cell(p)), sh);
  return ocean_swell_at(p, max(cell, tick_cell(p + first.xz)), sh);
}

fn ocean_swell_at(p: vec2<f32>, spacing: f32, sh: Shoal) -> vec3<f32> {
  var d = vec3<f32>(0.0);
  for (var i = 0u; i < 32u; i++) {
    let a = water.swell[2u * i];
    let b = water.swell[2u * i + 1u];
    let k = length(a.xy);
    let keep = sea_keep(k, spacing) * shoal_keep(k, sh);
    let th = dot(a.xy, p) + a.w;
    d += vec3<f32>(b.x * cos(th), a.z * sin(th), b.y * cos(th)) * keep;
  }
  if (sh.amp > 0.0) {
    let keep = sea_keep(water.shore[1].z, spacing);
    d += vec3<f32>(sh.inshore.x * sh.lean * cos(sh.phase), sh.amp * sin(sh.phase),
      sh.inshore.y * sh.lean * cos(sh.phase)) * keep;
  }
  return d;
}

// How much of the FFT's chop is drawn where the surface lies over world point p (frame xz: the displaced point, not
// the rest point; the gale's crests lean metres, which put the calm metres off the hull): none under an og.Buoyancy hull, all from HULL_CALM metres
// beyond its footprint. The tick floats a hull on its own waves alone, so under it the drawn surface is the tick's (the
// chop it does not ride bared a keel corner by centimetres in a trough), and a hull damps the short waves round it as
// it displaces the water there.
const HULL_CALM: f32 = 1.5;

fn hull_calm(p: vec2<f32>) -> f32 {
  var k = 1.0;
  for (var i = 0u; i < u32(water.fft.y); i++) {
    let a = water.hulls[2u * i];
    let h = water.hulls[2u * i + 1u].xy;
    let q = p - a.xy;
    let u = vec2<f32>(dot(q, a.zw), dot(q, vec2<f32>(-a.w, a.z)));
    let out = length(max(abs(u) - h, vec2<f32>(0.0)));
    k = min(k, smoothstep(0.0, HULL_CALM, out));
  }
  return k;
}

// ocean_displace's FFT cascades (render only) at rest point p, calmed under the hulls: `at` is where the tick's waves
// carry p (its displaced point, frame xz).
fn ocean_cascades(p: vec2<f32>, cell: f32, at: vec2<f32>, sh: Shoal) -> vec3<f32> {
  var d = vec3<f32>(0.0);
  let tc = tick_cell(at);
  let calm = hull_calm(at);
  if (calm <= 0.0) { return d; }
  for (var c = 0u; c < u32(water.fft.x); c++) {
    let lod = log2(max(cell * 256.0 / water.cascades[c].w, 1.0));
    let fade = (1.0 - smoothstep(4.0, 6.0, lod)) * cascade_weight(c, tc) * shoal_keep(water.bands[c], sh);
    if (fade <= 0.0) { continue; }
    d += textureSampleLevel(heights, ripple_sampler, cascade_uv(c, p), 3u * c, min(lod, 4.0)).xyz * fade;
  }
  return d * calm;
}

// The sea's normal at rest point p (frame xz), each long wave faded where it is shorter than four of the pixel's
// `footprint` (m) and the cascades filtered over the pixel; the Jacobian of its horizontal displacement (`jac`: under 1
// where the crests bunch, under 0 where they fold); and as `cov`, the covariance of the slopes the pixel holds
// unresolved (xx, zz, xz): the faded long waves' ((k a)² / 2 along each one's direction), the cascades' within the pixel
// (their slopes' second moments less the squared mean: LEAN mapping, Olano and Baker 2010) and the spectrum's tail finer
// than the finest cascade. A pixel of sea is a rough mirror, not a glass one (Bruneton, Neyret and Holzschuch 2010).
// The slopes are the displaced surface's (each divided by the stretch of its axis), as web-ocean's (`ocean.js` fs).
struct SeaNormal {
  n: vec3<f32>,
  slope: vec2<f32>,
  jac: f32,
  cov: vec3<f32>,
};

// (gx, gy: how far p moves a pixel across and down the screen, metres, for the cascades' filtering.)
// (at: the point drawn, frame xz, where the hulls calm the cascades.)
fn ocean_normal(p: vec2<f32>, at: vec2<f32>, footprint: f32, gx: vec2<f32>, gy: vec2<f32>) -> SeaNormal {
  var s = vec2<f32>(0.0);
  var cov = vec3<f32>(water.fft.z * 0.5, water.fft.z * 0.5, 0.0);
  var j = vec3<f32>(0.0);
  let calm = hull_calm(at);
  let sh = shoaling(p);
  // og_core's waves as the surface carries them (faded at the tick's cell where they are drawn: ocean_swell, and shoaled
  // by the seabed), those the pixel cannot hold left to the facets' roughness.
  let tc = tick_cell(at);
  for (var i = 0u; i < 32u; i++) {
    let a = water.swell[2u * i];
    let b = water.swell[2u * i + 1u];
    let k = length(a.xy);
    let f = shoal_keep(k, sh);
    let carried = sea_keep(k, tc) * f;
    let keep = sea_keep(k, max(footprint, tc)) * f;
    let th = dot(a.xy, p) + a.w;
    s += a.xy * (a.z * cos(th) * keep);
    let sn = sin(th) * keep;
    j -= vec3<f32>(b.x * a.x, b.y * a.y, b.x * a.y) * sn;
    let ka = k * a.z;
    let d = a.xy / max(k, 1e-6);
    cov += vec3<f32>(d.x * d.x, d.y * d.y, d.x * d.y) * (ka * ka * 0.5 * max(carried * carried - keep * keep, 0.0));
  }
  if (sh.amp > 0.0) {
    // The shore trains: crests along the distance's lines, travelling inshore (their wave vector k · inshore).
    let k = water.shore[1].z;
    let kv = sh.inshore * k;
    let carried = sea_keep(k, tc);
    let keep = sea_keep(k, max(footprint, tc));
    s += kv * (sh.amp * cos(sh.phase) * keep);
    j -= vec3<f32>(sh.inshore.x * kv.x, sh.inshore.y * kv.y, sh.inshore.x * kv.y) * (sh.lean * sin(sh.phase) * keep);
    let ka = k * sh.amp;
    let d = sh.inshore;
    cov += vec3<f32>(d.x * d.x, d.y * d.y, d.x * d.y) * (ka * ka * 0.5 * max(carried * carried - keep * keep, 0.0));
  }
  for (var c = 0u; c < u32(water.fft.x); c++) {
    let uv = cascade_uv(c, p);
    let k = water.cascades[c].x;
    let w = cascade_weight(c, tc) * calm * shoal_keep(water.bands[c], sh);
    let first = textureSampleGrad(heights, ripple_sampler, uv, 3u * c + 1u, gx * k, gy * k);
    let second = textureSampleGrad(heights, ripple_sampler, uv, 3u * c + 2u, gx * k, gy * k);
    s += first.xy * w;
    cov += vec3<f32>(max(second.x - first.x * first.x, 0.0), max(second.y - first.y * first.y, 0.0),
      second.z - first.x * first.y) * w * w;
    j += vec3<f32>(first.zw, textureSampleGrad(heights, ripple_sampler, uv, 3u * c, gx * k, gy * k).w) * w;
  }
  var o: SeaNormal;
  o.slope = s / vec2<f32>(max(1.0 + j.x, 0.15), max(1.0 + j.y, 0.15));
  o.n = normalize(vec3<f32>(-o.slope.x, 1.0, -o.slope.y));
  o.jac = (1.0 + j.x) * (1.0 + j.y) - j.z * j.z;
  o.cov = cov;
  return o;
}

// web-ocean's Smith masking for an anisotropic Gaussian of slopes (`lambdaSmith`).
fn smith_lambda(w: vec3<f32>, cov: vec3<f32>) -> f32 {
  let xz2 = max(dot(w.xz, w.xz), 1e-8);
  let il = inverseSqrt(xz2);
  let az = w.xz * il;
  let sig2 = az.x * az.x * cov.x + az.y * az.y * cov.y + 2.0 * az.x * az.y * cov.z;
  let a = max(w.y * il, 1e-3) / sqrt(2.0 * max(sig2, 1e-7));
  if (a >= 1.6) { return 0.0; }
  return (1.0 - 1.259 * a + 0.396 * a * a) / (3.535 * a + 2.181 * a * a);
}

// The sea's foam at rest point p (frame xz) where the surface's Jacobian is `jac`, ported from web-ocean's `ocean.js` fs
// (MIT, third_party/ocean/): the persistent foam of the world levels about the camera (fine near, coarse far), the
// foam of a crest folding now beyond them, aged foam broken into streaks along the wind, all cut by the foam's
// micro-structure into a mask with crisp edges where dense and a soft, translucent margin where thin.
struct SeaFoam {
  cover: f32,
  mask: f32,
  // How fresh the foam is (the levels' g): the bubbles of a crest just broken.
  young: f32,
  // The bubbles' brightness inside it.
  bubbles: f32,
};

// The surf (W1) at rest point p: where the shore trains break, a fresh sheet at each crest's face and its trail
// behind it, fading by the time the next crest comes, and the wash where the water runs thin up the beach.
fn sea_surf(p: vec2<f32>) -> f32 {
  if (water.shore[0].w < 0.5 || water.shore[2].z <= 0.0) { return 0.0; }
  let sh = shoaling(p);
  if (sh.inside <= 0.0) { return 0.0; }
  let half = 3.14159265 / max(water.shore[1].z, 1e-6);
  // Breaking where the trains lean over: inshore of a third of their half length, at the breaking depth.
  let breaking = (1.0 - smoothstep(0.08 * half, 0.4 * half, sh.depth)) * step(1e-4, sh.amp);
  // The time since the crest passed, as a share of a period (the phase falls as time goes on; the crest at pi/2).
  let age = fract((1.5707963 - sh.phase) / 6.2831853);
  let trail = exp(-age * 3.5) * smoothstep(0.0, 0.03, age);
  let wash = (1.0 - smoothstep(0.0, 0.45, sh.depth)) * (0.45 + 0.55 * trail);
  return water.shore[2].z * sh.inside * clamp(max(breaking * trail, wash), 0.0, 1.0) * step(0.0, sh.depth - 1e-3);
}

fn sea_foam(p: vec2<f32>, jac: f32) -> SeaFoam {
  let first = 3u * u32(water.fft.x);
  let l0 = water.foam_levels[0];
  let l1 = water.foam_levels[1];
  let dc = abs(p - frame.view_pos.xz);
  let cheb = max(dc.x, dc.y);
  let w0 = 1.0 - smoothstep(0.30 * l0.w, 0.45 * l0.w, cheb);
  let w1 = 1.0 - smoothstep(0.30 * l1.w, 0.45 * l1.w, cheb);
  let fw0 = textureSample(heights, ripple_sampler, p * l0.x + l0.yz, first);
  let fw1 = textureSample(heights, ripple_sampler, p * l1.x + l1.yz, first + 1u);
  let fwl = mix(fw1, fw0, w0) * w1;
  let surf = sea_surf(p);
  let fresh = max(clamp((water.ocean.z - jac) * water.ocean.w, 0.0, 1.0), surf);
  var cover = clamp(max(max(fwl.r, fresh * (1.0 - w1 * 0.7)), surf) + fwl.g * 0.6, 0.0, 1.5);
  let wd = water.sea_wind.xy;
  let ws = vec2<f32>(dot(p, wd), dot(p, vec2<f32>(-wd.y, wd.x))) * vec2<f32>(0.035, 0.11) + water.foam_uv[1].zw;
  let streak = textureSample(heights, ripple_sampler, ws, first + 2u).a;
  let fresh_k = clamp(fwl.g * 2.0 + fresh, 0.0, 1.0);
  cover *= mix(smoothstep(0.25, 0.75, streak) * 1.5, 1.0, fresh_k);
  let pat = textureSample(heights, ripple_sampler, p * 0.23 + water.foam_uv[0].xy, first + 2u).r * 0.78
    + textureSample(heights, ripple_sampler, p * 0.71 + water.foam_uv[0].zw, first + 2u).g * 0.22;
  let fw = fwidth(pat) + 0.03;
  let soft = 0.04 + 0.14 * (1.0 - clamp(cover, 0.0, 1.0));
  var fm = smoothstep(1.0 - cover - fw - soft, 1.0 - cover + fw + soft, pat);
  // Where a pixel spans the pattern (its change across the pixel near its range), the mask is what the pattern's cut
  // covers of the pixel on average, the share of the lace over 1 - cover: about the cover itself. The cut softened by
  // fwidth alone drew distant foam as a grey sheet wherever there was any.
  fm = mix(fm, clamp(cover, 0.0, 1.0), smoothstep(0.15, 0.4, fw));
  var o: SeaFoam;
  o.cover = cover;
  o.mask = clamp(fm * smoothstep(0.0, 0.5, cover), 0.0, 1.0) * (0.68 + 0.32 * smoothstep(0.3, 0.9, cover));
  o.mask = max(o.mask, smoothstep(0.35, 0.9, fwl.g) * smoothstep(0.25, 0.6, pat + 0.25) * 0.95);
  o.young = fwl.g;
  o.bubbles = textureSample(heights, ripple_sampler, p * 1.9 + water.foam_uv[1].xy, first + 2u).g;
  return o;
}

// Bruneton et al. 2010's mean Fresnel over a rough sea's facets: the grazing mirror dims as the slopes it averages
// spread (sigma: their deviation along the view).
fn sea_fresnel(cos_v: f32, sigma: f32) -> f32 {
  return pow(1.0 - cos_v, 5.0 * exp(-2.69 * sigma)) / (1.0 + 22.7 * pow(sigma, 1.5));
}

// How many metres of the surface a pixel spans at `world` along the view's slant: the distance over the projection's
// scale, stretched by the grazing angle. Smooth across the grid's triangles, where screen derivatives of the rest
// point jump at every edge (and drew the edge as a line where they faded the waves).
fn sea_footprint(world: vec3<f32>) -> f32 {
  let to_eye = frame.view_pos.xyz - world;
  let dist = length(to_eye);
  let scale = length(vec3<f32>(frame.view_proj[0].y, frame.view_proj[1].y, frame.view_proj[2].y));
  let pixel = 2.0 / max(scale * water.screen.y, 1e-6);
  return dist * pixel / max(abs(to_eye.y) / max(dist, 1e-6), 0.02);
}

// A vertex of the sea's clipmap: its rest point (frame xz) and the spacing its waves are read at, or w < 0 for a
// vertex of a cell left out (the hole a level leaves for the one inside it).
//
// Every level is `s` × `s` cells about its centre; each level past the first leaves out the cells under the level
// inside it, whose square lies on its lattice (water.rs `clipmap`). Towards its outer edge a level morphs onto the
// next one's lattice: its odd vertices take the mean of their even neighbours' displacement (along the edge, or the
// cells' diagonal for a vertex odd both ways), and every vertex reads the waves at a spacing growing from its own
// cell to the next level's, so the edge it shares with the next level's hole is that level's, vertex for vertex.
// Past the outermost level a ring of quads reaches the horizon (clip.z), flat at its far edge.
// A bounded sea's rest point (not endless): clamped to its rectangle in the surface's frame, so the clipmap's cells
// past it fold onto its edge.
fn sea_clamp(p: vec2<f32>) -> vec2<f32> {
  if (water.clip.w > 0.5) { return p; }
  let local = (water.inverse * vec4<f32>(p.x, water.model[3].y, p.y, 1.0)).xz;
  let h = water.size.xy * 0.5;
  return (water.model * vec4<f32>(clamp(local, -h, h).x, 0.0, clamp(local, -h, h).y, 1.0)).xz;
}

struct SeaVertex {
  rest: vec2<f32>,
  d: vec3<f32>,
  skip: bool,
};

fn sea_vertex(vi: u32) -> SeaVertex {
  var corners = array<vec2<f32>, 6>(
    vec2<f32>(0.0, 0.0), vec2<f32>(0.0, 1.0), vec2<f32>(1.0, 1.0),
    vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(1.0, 0.0));
  let s = u32(water.clip.y);
  let per_level = s * s * 6u;
  let levels = u32(water.clip.x);
  let level = vi / per_level;
  var o: SeaVertex;
  o.skip = false;
  if (level >= levels) {
    // The horizon ring: the outermost level's edge (its even vertices) to the ring's radius.
    let k = vi - levels * per_level;
    let corner = corners[k % 6u];
    let half = s / 2u;
    let t = (k / 6u + u32(corner.x)) % (4u * half);
    let last = water.levels[levels - 1u];
    let h = f32(half) * last.z;
    let along = f32(t % half) * 2.0 * last.z;
    var q: vec2<f32>;
    switch (t / half) {
      case 0u: { q = vec2<f32>(-h + along, -h); }
      case 1u: { q = vec2<f32>(h, -h + along); }
      case 2u: { q = vec2<f32>(h - along, h); }
      default: { q = vec2<f32>(-h, h - along); }
    }
    if (corner.y > 0.5) {
      o.rest = sea_clamp(last.xy + q * max(water.clip.z / h, 1.0));
      o.d = vec3<f32>(0.0);
    } else {
      o.rest = sea_clamp(last.xy + q);
      o.d = ocean_displace(o.rest, 2.0 * last.z);
    }
    return o;
  }
  let k = vi % per_level;
  let quad = k / 6u;
  let cell_ij = vec2<f32>(f32(quad % s), f32(quad / s));
  let lv = water.levels[level];
  let c = lv.z;
  let half = f32(s) * 0.5;
  if (level > 0u) {
    let inner = water.levels[level - 1u];
    let middle = lv.xy + (cell_ij + 0.5 - half) * c;
    if (all(abs(middle - inner.xy) < vec2<f32>(half * 0.5 * c))) {
      o.skip = true;
      return o;
    }
  }
  let g = cell_ij + corners[k % 6u] - half;
  let p = lv.xy + g * c;
  o.rest = sea_clamp(p);
  let m = smoothstep(0.7, 0.95, max(abs(g.x), abs(g.y)) / half);
  let spacing = c * (1.0 + m);
  let odd = vec2<bool>(abs(g.x) % 2.0 > 0.5, abs(g.y) % 2.0 > 0.5);
  var d = ocean_displace(o.rest, spacing);
  if (m > 0.0 && (odd.x || odd.y)) {
    let e = vec2<f32>(select(0.0, c, odd.x), select(0.0, c, odd.y));
    let mean = 0.5 * (ocean_displace(sea_clamp(p - e), spacing) + ocean_displace(sea_clamp(p + e), spacing));
    d = mix(d, mean, m);
  }
  o.d = d;
  return o;
}

// The drawn sea's height probe (water.rs `WaterRenderer::probe_sea`, Renderer::sea_heights): per point (frame xz),
// the height the clipmap's vertices draw there, the rest point found by undoing the crests' lean as the tick does, at
// the spacing sea_vertex reads its level's vertices at (its cell, growing to twice at its edge): (the height, the tick's
// waves' share, the cascades' share).
@group(2) @binding(0) var<storage, read> probe_in: array<vec2<f32>>;
@group(2) @binding(1) var<storage, read_write> probe_out: array<vec4<f32>>;

fn probe_spacing(p: vec2<f32>) -> f32 {
  let half = water.clip.y * 0.5;
  let levels = u32(water.clip.x);
  for (var l = 0u; l < levels; l++) {
    let lv = water.levels[l];
    let d = max(abs(p.x - lv.x), abs(p.y - lv.y)) / (half * lv.z);
    if (d <= 1.0) { return lv.z * (1.0 + smoothstep(0.7, 0.95, d)); }
  }
  return 2.0 * water.levels[levels - 1u].z;
}

@compute @workgroup_size(64)
fn probe_sea(@builtin(global_invocation_id) id: vec3<u32>) {
  if (id.x >= arrayLength(&probe_in)) { return; }
  let goal = probe_in[id.x];
  var rest = goal;
  for (var i = 0; i < 8; i++) {
    rest = goal - ocean_displace(rest, probe_spacing(rest)).xz;
  }
  let cell = probe_spacing(rest);
  let sh = shoaling(rest);
  let swell3 = ocean_swell(rest, cell, sh);
  let swell = swell3.y;
  let cascades = ocean_cascades(rest, cell, rest + swell3.xz, sh).y;
  probe_out[id.x] = vec4<f32>(water.model[3].y + swell + cascades, swell, cascades, 0.0);
}

// The height field's texels at local point (x, z) of the surface, bilinear: r the height; with ripples
// (water_ripples.wgsl) g the ripples alone and b their foam.
fn field_at(local: vec2<f32>) -> vec4<f32> {
  if (water.size.w < 0.5) { return vec4<f32>(0.0); }
  let n = i32(water.look.w);
  let cell = water.look.z / f32(n - 1);
  let g = (local + vec2<f32>(water.look.z * 0.5)) / cell;
  let i = clamp(vec2<i32>(floor(g)), vec2<i32>(0), vec2<i32>(n - 2));
  let f = clamp(g - vec2<f32>(i), vec2<f32>(0.0), vec2<f32>(1.0));
  let a = textureLoad(heights, i, 0, 0);
  let b = textureLoad(heights, i + vec2<i32>(1, 0), 0, 0);
  let c = textureLoad(heights, i + vec2<i32>(0, 1), 0, 0);
  let d = textureLoad(heights, i + vec2<i32>(1, 1), 0, 0);
  return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}

// The height field at local point (x, z) of the surface, bilinear.
fn height_at(local: vec2<f32>) -> f32 {
  return field_at(local).r;
}

@vertex
fn vs(@builtin(vertex_index) vi: u32) -> WaterOut {
  let segs = u32(water.size.z);
  let quad = vi / 6u;
  var corners = array<vec2<f32>, 6>(
    vec2<f32>(0.0, 0.0), vec2<f32>(0.0, 1.0), vec2<f32>(1.0, 1.0),
    vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(1.0, 0.0));
  if (water.fft.x > 0.5) { return vs_sea(vi); }
  let c = corners[vi % 6u];
  let cell = vec2<f32>(f32(quad % segs), f32(quad / segs)) + c;
  let local = (cell / f32(segs) - 0.5) * water.size.xy;
  let h = height_at(local);
  // The height field's slope gives the geometric normal.
  let e = water.look.z / max(water.look.w - 1.0, 1.0);
  let hx = height_at(local + vec2<f32>(e, 0.0)) - height_at(local - vec2<f32>(e, 0.0));
  let hz = height_at(local + vec2<f32>(0.0, e)) - height_at(local - vec2<f32>(0.0, e));
  let n_local = normalize(vec3<f32>(-hx / (2.0 * e), 1.0, -hz / (2.0 * e)));
  let world = water.model * vec4<f32>(local.x, h, local.y, 1.0);
  let rest = world.xz;
  var o: WaterOut;
  o.clip = frame.view_proj * world;
  o.world = world.xyz;
  o.rest = rest;
  o.normal = normalize((water.model * vec4<f32>(n_local, 0.0)).xyz);
  o.local = local;
  o.slope = length(vec2<f32>(hx, hz)) / (2.0 * e);
  o.uv = local;
  return o;
}

// A sea's vertex (sea_vertex): its rest point on the surface's level, displaced; the geometric normal is the level's.
fn vs_sea(vi: u32) -> WaterOut {
  let v = sea_vertex(vi);
  var o: WaterOut;
  o.normal = normalize((water.model * vec4<f32>(0.0, 1.0, 0.0, 0.0)).xyz);
  o.slope = 0.0;
  if (v.skip) {
    // A cell under the level inside: all six corners at one point outside the view, drawing nothing.
    o.clip = vec4<f32>(2.0, 2.0, 2.0, 1.0);
    o.world = vec3<f32>(0.0);
    o.rest = vec2<f32>(0.0);
    o.local = vec2<f32>(0.0);
    o.uv = vec2<f32>(0.0);
    return o;
  }
  let level = water.model[3].y;
  let world = vec3<f32>(v.rest.x, level, v.rest.y) + v.d;
  o.clip = frame.view_proj * vec4<f32>(world, 1.0);
  o.world = world;
  o.rest = v.rest;
  o.local = (water.inverse * vec4<f32>(v.rest.x, level, v.rest.y, 1.0)).xz;
  o.uv = o.local;
  return o;
}

// A surface drawn over a mesh (Water::mesh): its vertices and normals in the
// surface's frame; the waves run over its local xz.
@vertex
fn vs_mesh(@location(0) position: vec3<f32>, @location(1) normal: vec3<f32>, @location(2) uv: vec2<f32>) -> WaterOut {
  let world = water.model * vec4<f32>(position, 1.0);
  var o: WaterOut;
  o.clip = frame.view_proj * world;
  o.world = world.xyz;
  o.normal = normalize((water.model * vec4<f32>(normal, 0.0)).xyz);
  o.local = position.xz;
  o.slope = 0.0;
  o.uv = uv;
  o.rest = world.xz;
  return o;
}

// The detail normal's slope from travelling waves (two pairs of crossing
// directions at two scales), in local xz.
fn wave_slope(p: vec2<f32>, t: f32) -> vec2<f32> {
  let k = water.waves.x * 6.2831853;
  let s = water.waves.y;
  var dirs = array<vec2<f32>, 4>(
    vec2<f32>(0.8, 0.6), vec2<f32>(-0.6, 0.8), vec2<f32>(0.3, -0.95), vec2<f32>(-0.9, -0.4));
  var scales = array<f32, 4>(1.0, 1.37, 2.71, 3.9);
  var slope = vec2<f32>(0.0);
  for (var i = 0; i < 4; i++) {
    let kk = k * scales[i];
    let phase = dot(dirs[i], p) * kk + t * s * sqrt(kk) * 1.6;
    slope += dirs[i] * cos(phase) * (1.0 / scales[i]);
  }
  return slope * water.waves.z * 0.25;
}

// One read of the normal map (its own tiled, anisotropic sampler). v runs up the image, as three.js samples its
// textures (flipY): the map's green is the slope along +v there, so the bumps light the way three.js lights them.
fn ripple_map(uv: vec2<f32>, gx: vec2<f32>, gy: vec2<f32>) -> vec3<f32> {
  let f = vec2<f32>(1.0, -1.0);
  return textureSampleGrad(normal_tex, ripple_sampler, uv * f, gx * f, gy * f).rgb;
}

// The slope (local xz) from the normal map (Water::normal_map): three.js Water's four drifting layers
// (getNoise: normal = (noise.x, noise.z, noise.y) * (1.5, 1, 1.5), noise = sum * 0.5 - 1) over world xz times the
// scale, or with a flow three.js Water2's two layers sliding along it, cross-faded each half cycle (cycle 0.15),
// normal = (r * 2 - 1, b, g * 2 - 1). Either is scaled by the wave strength.
fn map_slope(world: vec2<f32>, uv: vec2<f32>, t: f32) -> vec2<f32> {
  let flow = water.normals.yz;
  var n: vec3<f32>;
  if (dot(flow, flow) < 1e-12) {
    let p = world * water.normals.x;
    let gx = dpdx(p);
    let gy = dpdy(p);
    let tt = t * water.waves.y;
    // `world` is from the frame's origin: each layer adds its uv there.
    let o = water.layers;
    let s = ripple_map(p / 103.0 + o[0].xy + vec2<f32>(tt / 17.0, tt / 29.0), gx / 103.0, gy / 103.0)
      + ripple_map(p / 107.0 + o[0].zw - vec2<f32>(tt / -19.0, tt / 31.0), gx / 107.0, gy / 107.0)
      + ripple_map(p / vec2<f32>(8907.0, 9803.0) + o[1].xy + vec2<f32>(tt / 101.0, tt / 97.0), gx / vec2<f32>(8907.0, 9803.0), gy / vec2<f32>(8907.0, 9803.0))
      + ripple_map(p / vec2<f32>(1091.0, 1027.0) + o[1].zw - vec2<f32>(tt / 109.0, tt / -113.0), gx / vec2<f32>(1091.0, 1027.0), gy / vec2<f32>(1091.0, 1027.0));
    let noise = s * 0.5 - 1.0;
    n = normalize(vec3<f32>(noise.x * 1.5, noise.z, noise.y * 1.5));
  } else {
    let speed = length(flow);
    var dir = flow / speed * vec2<f32>(-1.0, 1.0);
    if (water.ao.y > 0.5) {
      // Water2's flow map: the direction (not unit) per point, read where three.js reads it (v up the image).
      dir = flow_at(uv) * vec2<f32>(-1.0, 1.0);
    }
    let cycle = 0.15;
    let half_cycle = cycle * 0.5;
    let o0 = (t * speed) % cycle;
    // Both offsets wrap in [0, cycle) (Water2.js onBeforeRender): unwrapped, the second layer jumped back a half
    // cycle at full weight each cycle (a pop every 0.15 / speed seconds).
    let o1 = (o0 + half_cycle) % cycle;
    let p = uv * water.normals.x;
    let gx = dpdx(p);
    let gy = dpdy(p);
    let a = ripple_map(p + dir * o0, gx, gy);
    let b = ripple_map(p + dir * o1, gx, gy);
    let c = mix(a, b, abs(half_cycle - o0) / half_cycle);
    n = normalize(vec3<f32>(c.r * 2.0 - 1.0, c.b, c.g * 2.0 - 1.0));
  }
  return -n.xz / max(n.y, 0.05) * water.waves.z;
}

// The flow map's direction at the mesh's uv, or at the grid's metres (u along x and v along z from the -x -z
// corner): rg * 2 - 1, the image's top at v 1 (three.js' flipY).
fn flow_at(uv: vec2<f32>) -> vec2<f32> {
  var at = uv;
  if (water.ao.z < 0.5) { at = uv / max(water.size.xy, vec2<f32>(1e-3)) + 0.5; }
  return textureSampleLevel(flow_tex, water_sampler, vec2<f32>(at.x, 1.0 - at.y), 0.0).rg * 2.0 - 1.0;
}

// A stream's ripples: wave fronts across the flow's direction `d` (unit), within half a radian of it, at the detail
// waves' scale and strength.
fn stream_slope(p: vec2<f32>, d: vec2<f32>, t: f32, fast: f32) -> vec2<f32> {
  // Where the flow is slower than the surface's flow (a flow map's still water, `fast` under 1), its ripples are
  // longer and lower; at the flow's own speed they are as before.
  let pace = mix(0.7, 1.0, smoothstep(0.3, 1.0, fast));
  let k = water.waves.x * 6.2831853 * pace;
  let side = vec2<f32>(-d.y, d.x);
  var turns = array<f32, 4>(0.0, 0.45, -0.3, 0.15);
  var scales = array<f32, 4>(1.0, 1.37, 1.9, 2.71);
  var slope = vec2<f32>(0.0);
  for (var i = 0; i < 4; i++) {
    let dir = d * cos(turns[i]) + side * sin(turns[i]);
    let kk = k * scales[i];
    // Each train's fronts warped and its height varied by a slow noise of its own, so the trains do not lock into a
    // regular grid of dimples.
    let fi = f32(i);
    let warp = foam_noise(p * 0.21 + vec2<f32>(fi * 3.7, fi * 1.3)) * 5.0;
    let gain = 0.45 + foam_noise(p * 0.33 + vec2<f32>(fi * 2.1, fi * 5.9)) * 1.1;
    let phase = dot(dir, p) * kk + warp + t * water.waves.y * sqrt(kk) * 1.6;
    slope += dir * cos(phase) * (gain / scales[i]);
  }
  return slope * water.waves.z * 0.25 * mix(0.55, 1.0, pace * pace);
}

// The travelling waves carried by the flow (Water::flow, metres a second; per point with a flow map), as Water2
// carries its layers: fronts across the flow's own direction, two copies sliding along the flow, each restarting
// every three seconds half a period apart and cross-faded so neither restart shows. A flow map moves each point by
// its own flow, so the fronts bend where the flow does (round a rock, by up to its speed times a period).
fn flowed_slope(p: vec2<f32>, uv: vec2<f32>, t: f32) -> vec2<f32> {
  var f = water.normals.yz;
  if (water.ao.y > 0.5) { f = flow_at(uv) * length(water.normals.yz); }
  let d = normalize(water.normals.yz);
  let period = 3.0;
  let ph0 = fract(t / period);
  let ph1 = fract(t / period + 0.5);
  let fast = length(f) / max(length(water.normals.yz), 1e-6);
  let s0 = stream_slope(p - f * ph0 * period, d, t, fast);
  let s1 = stream_slope(p - f * ph1 * period, d, t, fast);
  return mix(s0, s1, abs(ph0 - 0.5) * 2.0);
}

fn value_hash(p: vec2<f32>) -> f32 {
  return lattice_hash(vec3<i32>(vec2<i32>(p), 3));
}

// A cheap smooth noise for breaking up the foam.
fn foam_noise(p: vec2<f32>) -> f32 {
  let i = floor(p);
  let f = fract(p);
  let u = f * f * (3.0 - 2.0 * f);
  return mix(mix(value_hash(i), value_hash(i + vec2<f32>(1.0, 0.0)), u.x),
             mix(value_hash(i + vec2<f32>(0.0, 1.0)), value_hash(i + vec2<f32>(1.0, 1.0)), u.x), u.y);
}

fn view_depth(p: vec3<f32>) -> f32 {
  return dot(p - frame.view_pos.xyz, frame.view_forward.xyz);
}

fn screen_uv(p: vec3<f32>) -> vec3<f32> {
  let c = frame.view_proj * vec4<f32>(p, 1.0);
  let ndc = c.xyz / max(c.w, 1e-6);
  return vec3<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5, c.w);
}

// A screen-space reflection: march the reflected ray through the depth
// prepass; the colour where it passes behind the scene (w: 1), else w 0.
fn ssr(origin: vec3<f32>, r: vec3<f32>) -> vec4<f32> {
  let steps = 28;
  var t = 0.2;
  var last = 0.0;
  for (var i = 0; i < steps; i++) {
    let p = origin + r * t;
    let s = screen_uv(p);
    if (s.z <= 0.0 || any(s.xy < vec2<f32>(0.0)) || any(s.xy > vec2<f32>(1.0))) { break; }
    let px = vec2<i32>(s.xy * water.screen.xy);
    let scene = textureLoad(scene_depth, px, 0).r;
    let d = view_depth(p);
    if (d > scene && d - scene < 0.6 + t * 0.15) {
      // Refine between the last two samples.
      var lo = last;
      var hi = t;
      for (var j = 0; j < 4; j++) {
        let mid = 0.5 * (lo + hi);
        let q = origin + r * mid;
        let sq = screen_uv(q);
        let sd = textureLoad(scene_depth, vec2<i32>(sq.xy * water.screen.xy), 0).r;
        if (view_depth(q) > sd) { hi = mid; } else { lo = mid; }
      }
      let hit = screen_uv(origin + r * hi);
      // On a sea, a hit is where the refined ray meets what the screen holds: a ray that only passed behind a
      // floater's silhouette (the march's thickness is generous) mirrored its dark side into the near water.
      let held = textureLoad(scene_depth, vec2<i32>(hit.xy * water.screen.xy), 0).r;
      if (water.ocean.x > 0.5 && view_depth(origin + r * hi) - held > 0.25 + 0.02 * hi) {
        last = t;
        t = t * 1.25 + 0.15;
        continue;
      }
      let edge = min(min(hit.x, 1.0 - hit.x), min(hit.y, 1.0 - hit.y));
      let fade = clamp(edge * 8.0, 0.0, 1.0) * (1.0 - f32(i) / f32(steps));
      // The scene colour there was already darkened by its own screen AO (the whole colour × mix(0.35, 1, ao) by
      // aoDirect, pbr_fs.wgsl): taken back out, so that `fs` darkens the reflection once, by the AO at the water (as
      // a mirror drawn without AO and darkened as a whole by three.js-era post AO). aoDirect 0 or no AO: no change.
      let k_hit = mix(1.0, mix(0.35, 1.0, textureSampleLevel(ao_tex, water_sampler, hit.xy, 0.0).r), water.ao.x);
      return vec4<f32>(textureSampleLevel(scene_color, water_sampler, hit.xy, 0.0).rgb / k_hit, fade);
    }
    last = t;
    t = t * 1.25 + 0.15;
  }
  return vec4<f32>(0.0);
}

// A texel of the depth below, clamped to it (a target smaller than the
// screen the uniform names reads its edge).
fn depth_texel(p: vec2<f32>) -> vec2<i32> {
  return min(vec2<i32>(p), vec2<i32>(textureDimensions(scene_depth)) - vec2<i32>(1));
}

// The sky mirrored along `r` (`c`, from the reflection maps, which hold no clouds) under the frame's cloud cover:
// greyed as the clouds' own ambient is (sky.wgsl clouds_over), and under an overcast the deck itself, brighter
// overhead than at the horizon (a clear horizon mirrored at the sea's grazing rim drew a white line under a storm).
fn overcast(c: vec3<f32>, r: vec3<f32>) -> vec3<f32> {
  if (frame.clouds[3].w < 0.5) { return c; }
  let grey = mix(c, vec3<f32>(dot(c, vec3<f32>(0.2126, 0.7152, 0.0722))), frame.clouds[0].x * 0.7);
  let dk = cloud_deck(r);
  return mix(grey, dk.rgb * (1.0 + 0.6 * max(r.y, 0.0)), dk.a);
}

@fragment
fn fs(in: WaterOut) -> @location(0) vec4<f32> {
  if (frame.light_color.w > 0.5 && (frame.light_color.w < 1.5 || frame.light_color.w > 2.5)) {
    return debug_unsupported(in.clip.xy);
  }
  let t = frame.time.x;
  // (The map's gradients are taken in uniform control flow: the branch is on a uniform.)
  var slope = wave_slope(in.local, t);
  if (water.normals.w > 0.5) {
    slope = map_slope(in.world.xz, in.uv, t);
  } else if (dot(water.normals.yz, water.normals.yz) >= 1e-12) {
    slope = flowed_slope(in.local, in.uv, t);
  }
  let tangent_x = normalize((water.model * vec4<f32>(1.0, 0.0, 0.0, 0.0)).xyz);
  let tangent_z = normalize((water.model * vec4<f32>(0.0, 0.0, 1.0, 0.0)).xyz);
  // The sea's own normal under the detail (a level surface: its waves are the world's), and its foam.
  var base = in.normal;
  var sea_cov = vec3<f32>(0.0);
  var sea_f = SeaFoam(0.0, 0.0, 0.0, 0.0);
  if (water.ocean.x > 0.5) {
    var footprint = sea_footprint(in.world);
    // The pixel's footprint on the level surface, from where its view ray meets the level: smooth across the screen,
    // where the rest point's derivatives jump at every edge of the displaced triangles (and their mips drew the grid).
    // (A ray that never meets the level, a crest above the eye, takes the point itself.)
    let to = in.world - frame.view_pos.xyz;
    let along = (water.model[3].y - frame.view_pos.y) / min(to.y, -1e-6);
    let flat_hit = frame.view_pos.xz + to.xz * select(1.0, along, to.y < -0.01 * length(to) && along > 0.0);
    var gx = dpdx(flat_hit);
    var gy = dpdy(flat_hit);
    if (frame.view_pos.w > 0.5) {
      gx = dpdx(in.rest);
      gy = dpdy(in.rest);
      footprint = max(length(gx), length(gy));
    }
    let sea = ocean_normal(in.rest, in.world.xz, footprint, gx, gy);
    base = sea.n;
    // web-ocean: the slopes' change across the pixel widens the lobe (specular anti-aliasing).
    let dsx = dpdx(sea.slope);
    let dsy = dpdy(sea.slope);
    let aa = vec3<f32>(dsx.x * dsx.x + dsy.x * dsy.x, dsx.y * dsx.y + dsy.y * dsy.y, dsx.x * dsx.y + dsy.x * dsy.y) * 0.5;
    sea_cov = sea.cov + aa * min(1.0, 0.03 / max(max(aa.x, aa.y), 1e-8));
    sea_f = sea_foam(in.rest, sea.jac);
    // The detail ripples are the sea's own shortest waves: kept only where a pixel holds them (else they alias into
    // stripes towards the horizon).
    slope *= 1.0 - smoothstep(0.785, 1.571, water.waves.x * 6.2831853 * 3.9 * footprint);
  }
  // With ripples (water_ripples.wgsl): their own slope per pixel bends the grid's normal (finer than its vertices;
  // zero on open water, so its glare stays the grid's), and their foam.
  var ripple_foam = 0.0;
  if (water.size.w > 1.5 && water.ocean.x < 0.5) {
    let e = water.look.z / max(water.look.w - 1.0, 1.0);
    let rx = field_at(in.local + vec2<f32>(e, 0.0)).g - field_at(in.local - vec2<f32>(e, 0.0)).g;
    let rz = field_at(in.local + vec2<f32>(0.0, e)).g - field_at(in.local - vec2<f32>(0.0, e)).g;
    base = normalize(base - tangent_x * (rx / (2.0 * e)) - tangent_z * (rz / (2.0 * e)));
    ripple_foam = field_at(in.local).b;
  }
  var n = normalize(base - tangent_x * slope.x - tangent_z * slope.y);
  let v = select(normalize(frame.view_pos.xyz - in.world), -frame.view_forward.xyz, frame.view_pos.w > 0.5);
  // Seen from below: the camera is under the surface.
  let below_view = dot(in.normal, v) < 0.0;
  // three.js Water / Water2 (below) never turn a ripple's normal to the eye: a wavelet facing away at a grazing view
  // reflects fully (theta = max(dot(V, N), 0), Fresnel 1), its body scatters nothing, the mirror bends by its own N.xz.
  let three_water = water.under.w > 0.0 || (water.normals.w > 0.5 && dot(water.normals.yz, water.normals.yz) >= 1e-12);
  if (water.ocean.x > 0.5) {
    // A sea's swell turns its back slopes from the eye past a crest: a flipped normal mirrored the bright sky there
    // at Fresnel 1, a bright line along every crest's silhouette. Bend it to just face the eye instead (its
    // reflection then points down, into the sea: see below).
    n = normalize(n + v * max(0.05 - dot(n, v), 0.0));
  } else if (dot(n, v) < 0.0 && !three_water) { n = -n; }
  let l = normalize(frame.light_dir.xyz);
  var sun = 1.0;
  if (water.extra.x > 0.0 && frame.light_dir.w > 0.5) {
    let up = select(in.normal, -in.normal, dot(in.normal, l) < 0.0);
    sun = mix(1.0, cascade_shadow(in.world, up, max(dot(up, l), 0.0)), water.extra.x);
  }

  let px = in.clip.xy;
  let uv = px * water.screen.zw;
  let here = view_depth(in.world);
  let scene_here = textureLoad(scene_depth, depth_texel(px), 0).r;
  // Refraction: the scene behind, shifted by the detail normal, unless the
  // shifted sample is in front of the water.
  // three.js Water2 (a normal map with a flow): its look, term for term (see below). three.js Water (glint > 0): its body.
  let water2 = water.normals.w > 0.5 && dot(water.normals.yz, water.normals.yz) >= 1e-12;
  let water_js = water.under.w > 0.0;
  var shift = (n - in.normal).xz * water.waves.w * water.screen.zw * 40.0 / max(here * 0.2, 1.0);
  if (water2) {
    // Water2: uv + coord.z · N.xz · 0.05 (coord.z, the depth in [0, 1], is about 1), with screen v down here.
    let d = (n - in.normal).xz;
    shift = vec2<f32>(d.x, -d.y) * 0.05 * water.waves.w;
  }
  var ruv = clamp(uv + shift, vec2<f32>(0.0), vec2<f32>(1.0));
  var scene_at = textureLoad(scene_depth, depth_texel(ruv * water.screen.xy), 0).r;
  if (scene_at < here) {
    ruv = uv;
    scene_at = scene_here;
  }
  let below = textureSampleLevel(scene_color, water_sampler, ruv, 0.0).rgb;
  if (below_view) {
    // From below: what is above shows inside Snell's window (the view
    // steeper than the critical angle, about 49 degrees from the normal),
    // outside it the surface reflects the water's own colour.
    let window = smoothstep(0.6, 0.7, abs(dot(n, v)));
    let colour = mix(water.under.rgb, below * water.shallow.rgb, window);
    return vec4<f32>(apply_fog(colour, in.world), 1.0);
  }
  let thickness = max(scene_at - here, 0.0);
  let absorb = 1.0 - exp(-thickness / max(water.shallow.w, 1e-3));
  // The body scatters the light that reaches it: its deep colour is authored for daylight and darkens under less, at dusk, at night or under a storm's deck
  // (else the sea glowed a daylight teal under a dark sky). Shadowed water scatters less light still.
  let light_in = frame.light_color.rgb * max(l.y, 0.0) + ambient_light_level(vec3<f32>(0.0, 1.0, 0.0), in.world) * frame.params.x;
  // Full from 1.5 (the sun some 25° up on a clear day), then falling with the square of the light: a moonlit body
  // (the moon's light is a fair share of the sun's in the day-night look, so land reads at night) sits under the night
  // sky it lies beneath, as a real sea's does (its water-leaving light is a few percent of the sky's).
  let daylight_k = min(dot(light_in, vec3<f32>(0.2126, 0.7152, 0.0722)) / 1.5, 1.0);
  let daylight = daylight_k * daylight_k;
  var deep = water.deep.rgb * mix(0.4, 1.0, sun) * daylight;
  let sea = water.ocean.x > 0.5;
  // The sky's light on the sea from above and along its normal (web-ocean's skyUp and skyN).
  let amb_up = ambient_light_level(in.normal, in.world) * frame.params.x;
  let amb_n = ambient_light_level(n, in.world) * frame.params.x;
  // Under a closed cloud deck the light reaches the sea as the deck's own radiance, from the whole sky above (its a:
  // how much of the sky it closes): the foam and the crests take it (below). The sun itself is hidden by the deck
  // for every surface at its source (Clouds::light). The foam's light under a live sky is the captured deck that
  // amb_up and amb_n already carry (the analytical deck over it lit the foam twice); the crests keep the deck's.
  let deck = cloud_deck(vec3<f32>(0.0, 1.0, 0.0));
  let deck_up = select(deck, vec4<f32>(0.0), frame.clouds[4].w > 0.5);
  let sun_e = frame.light_color.rgb * sun;
  if (sea) {
    // web-ocean's body (`ocean.js` fs; the Sea of Thieves talk's subsurface term): light carried through the thin
    // upper part of a backlit crest, strongest looking towards the sun; and under foam the bubbles brighten and tint
    // the water beneath it.
    let crest = clamp((in.world.y - water.model[3].y - 0.05) / 0.9, 0.0, 1.0);
    let to_sun = max(dot(l, -v), 0.0);
    let thin = pow(clamp(0.5 - 0.5 * dot(l, n), 0.0, 1.0), 3.0);
    let dist = length(frame.view_pos.xyz - in.world);
    let k1 = SEA_SSS.w * crest * crest * pow(to_sun, 4.0) * thin * exp(-dist / 600.0);
    deep += SEA_SSS.rgb * sun_e * k1;
    let ambient = amb_up * 1.3 + amb_n * 0.5;
    deep += SEA_SSS.rgb * (sun_e * 0.05 + ambient * 0.35) * clamp(sea_f.cover, 0.0, 1.0);
    deep += vec3<f32>(0.10, 0.32, 0.30) * amb_up * sea_f.young * 0.35;
    // Under a closed deck the deck is the light the crests carry: the Atlas talk's (GDC 2019) scatter terms with the
    // deck for the sun, a Lambert share towards its bright side over the hidden sun and a share facing the eye, both
    // through the thin upper part of a wave (its height, as the sun's term above), so the crests lift a faint
    // blue-green from the troughs (the storm read as one dark sheet once the hidden sun no longer lit it).
    let l_deck = normalize(vec3<f32>(l.x, 1.0, l.z));
    let vn = max(dot(v, n), 0.0);
    let k_deck = crest * (0.8 * max(dot(n, l_deck), 0.0) + 0.6 * vn * vn) * exp(-dist / 600.0);
    deep += SEA_SSS.rgb * deck.rgb * 1.4 * deck.a * k_deck;
  }
  if (water_js) {
    // three.js Water's body, (sunColor · diffuse · 0.3 + scatter) · shadow: diffuse = max(L·N, 0) · sunColor · 0.5,
    // scatter = max(N·V, 0) · waterColor (the deep colour), sunColor = the light × glint / 2 (glint = spec 2 × sunColor
    // over the light). The ripples shade it, as they do the original's.
    let sc = frame.light_color.rgb * water.under.w * 0.5;
    deep = (sc * sc * 0.15 * max(dot(l, n), 0.0) + water.deep.rgb * max(dot(n, v), 0.0)) * sun;
  }
  var color = mix(below * water.shallow.rgb, deep, clamp(absorb * water.deep.w, 0.0, 1.0));

  // Reflection with Schlick's Fresnel.
  var r = reflect(-v, n);
  let f0 = water.extra.z;
  var fresnel = (f0 + (1.0 - f0) * pow(1.0 - saturate(dot(n, v)), 5.0)) * water.look.x;
  var mip = 0.5;
  var sig_v = 0.0;
  if (sea) {
    // web-ocean's tilt of the visible facets (`ocean.js` fs): at a grazing view the facets the eye sees lean towards
    // it (projected area and masking), so the reflector tilts up and mirrors higher sky. The Fresnel is Bruneton,
    // Neyret and Holzschuch's mean over a rough sea's facets (2010), which dims the grazing mirror as the slopes along
    // the view spread: web-ocean's partly smoothed normal kept it near 1 at the horizon, and with downward rays folded
    // up into the sky the far sea outshone the sky it mirrors (0.47 against 0.27 under the storm's deck). The sky is
    // blurred by the slopes' spread.
    let vh = normalize(vec3<f32>(v.x, 0.0, v.z) + vec3<f32>(1e-5, 0.0, 0.0));
    sig_v = sqrt(max(vh.x * vh.x * sea_cov.x + vh.z * vh.z * sea_cov.y + 2.0 * vh.x * vh.z * sea_cov.z, 1e-6));
    let a_v = max(v.y, 1e-3) / max(sqrt(1.0 - v.y * v.y), 1e-3) / (1.4142 * sig_v);
    let tilt = 0.32 * sig_v * (1.0 - smoothstep(0.0, 2.5, a_v));
    let nv = normalize(n + vh * tilt);
    r = reflect(-v, nv);
    fresnel = (f0 + (1.0 - f0) * sea_fresnel(saturate(dot(n, v)), sig_v)) * water.look.x;
    mip = max(sqrt(sqrt(sea_cov.x + sea_cov.y)) * frame.params.y, 0.5);
  }
  // The environment, or the reflection probe holding the surface (probes.rs), as bright as the sky is drawn: without the
  // twilight's adaptation (water.ao.w), which lifts the light on surfaces but not the sky they mirror. Under a cloud
  // deck the sky mirrored is the clouds' grey, as they light it (sky.wgsl).
  let sky_mirror = frame.params.x * water.ao.w * 1.8;
  var reflected = overcast(reflected_light(r, in.world, mip) * sky_mirror, r);
  if (water.extra.y > 0.5) {
    // The mirrored scene, flipped left to right, shifted by the waves.
    var shift_r = (n - in.normal).xz * water.screen.zw * 60.0;
    if (water.normals.w > 0.5 && dot(water.normals.yz, water.normals.yz) < 1e-12) {
      // three.js Water (the lake's normal map): the mirror bends by N.xz (0.001 + 1 / distance) x distortionScale 1.2, in uv.
      shift_r = (n - in.normal).xz * (0.001 + 1.0 / max(length(frame.view_pos.xyz - in.world), 0.1)) * 1.2;
    } else if (water2) {
      // Water2's own reflector, read at the refraction's uv (uv + coord.z · N.xz · 0.05, mirrored left to right).
      shift_r = shift;
    }
    let puv = clamp(vec2<f32>(1.0 - uv.x, uv.y) + vec2<f32>(-shift_r.x, shift_r.y), vec2<f32>(0.0), vec2<f32>(1.0));
    reflected = textureSampleLevel(reflection_tex, water_sampler, puv, 0.0).rgb;
  } else if (water.look.y > 0.5) {
    let hit = ssr(in.world, r);
    reflected = mix(reflected, hit.rgb, hit.a);
  }
  if (sea) {
    // A ray reflected downwards meets the next wave, which shows its body and, dimly, the sky in turn (it never
    // reaches the sky, nor the ground under the horizon the blurred environment holds there). The facets spread the
    // reflection over about two of their slopes' deviations, so it turns down gradually.
    let up_r = vec3<f32>(r.x, abs(r.y), r.z);
    let up = overcast(reflected_light(up_r, in.world, mip) * sky_mirror, up_r);
    let w = 2.0 * sig_v + 0.05;
    reflected = mix(deep + up * 0.25, reflected, smoothstep(-w, w, r.y));
  }
  let mirrored = reflected;
  reflected = water.extra.w + (1.0 - water.extra.w) * reflected;
  if (water2) {
    // Water2: color × mix(refraction, reflection, F): the tint is on the reflection too.
    reflected *= water.shallow.rgb;
  }
  // Screen-space AO: the scene behind (`below`) was darkened by it as it was lit; the reflection and the body are the
  // water's own, darkened here the same way (the whole colour × mix(0.35, 1, ao), as pbr_fs with aoDirect 1 and as
  // three.js-era post AO, which darkened the water's pixel whole), by PostEffects.aoDirect (water.ao.x): at the
  // default 0 the water, like every opaque surface's direct light, is not darkened by the screen AO.
  let ao_raw = textureSampleLevel(ao_tex, water_sampler, uv, 0.0).r;
  // Debug (DebugView::AmbientOcclusion): the ambient occlusion here, as pbr_fs.wgsl and scatter.wgsl.
  if (frame.light_color.w > 1.5 && frame.light_color.w < 2.5) { return vec4<f32>(vec3<f32>(ao_raw), 1.0); }
  let ao_k = mix(1.0, mix(0.35, 1.0, ao_raw), water.ao.x);
  reflected *= ao_k;
  if (water_js) { color *= ao_k; }
  color = mix(color, reflected, clamp(fresnel, 0.0, 1.0));

  if (water_js) {
    // three.js Water's glint: pow(eye . reflect(-L, N), 100) x the sun (glint = spec 2 x its sunColor over the light's
    // intensity) on the mirrored image, through the Fresnel mix: every wavelet facing the sun flashes with the sky.
    // Unshadowed, as there (its shadow mask is on the body alone).
    let g = pow(max(dot(v, reflect(-l, n)), 0.0), 100.0) * water.under.w;
    color += mirrored * frame.light_color.rgb * g * clamp(fresnel, 0.0, 1.0);
  } else if (sea) {
    // The sun's glitter, web-ocean's (`ocean.js` fs): the facets' slopes an anisotropic Gaussian about the pixel's
    // mean slope with the unresolved covariance (and the sun's disc), Smith's masking of the view and the light,
    // Schlick's Fresnel at the half vector (none in shadow).
    let h = normalize(l + v);
    let hs = -h.xz / max(h.y, 1e-3);
    let dsl = hs + n.xz / max(n.y, 1e-3);
    let cov = sea_cov + vec3<f32>(SUN_DISC, SUN_DISC, 0.0);
    let det = max(cov.x * cov.y - cov.z * cov.z, 1e-10);
    let e = (dsl.x * dsl.x * cov.y + dsl.y * dsl.y * cov.x - 2.0 * dsl.x * dsl.y * cov.z) / det;
    let pdf = exp(-0.5 * e) / (6.2831853 * sqrt(det));
    let hy2 = h.y * h.y;
    let dm = pdf / max(hy2 * hy2, 1e-6);
    let g = 1.0 / (1.0 + smith_lambda(v, cov) + smith_lambda(l, cov));
    let fh = f0 + (1.0 - f0) * pow(1.0 - clamp(dot(v, h), 0.0, 1.0), 5.0);
    let spec = fh * dm * g / (4.0 * max(v.y, 0.02)) * step(0.0, l.y);
    color += sun_e * spec * water.look.x;
  } else if (!water2) {
    // (Water2 has no highlight of its own: the sun shows only in what it reflects.)
    // The sun's highlight (none in shadow).
    let h = normalize(l + v);
    let spec = pow(max(dot(n, h), 0.0), 220.0) * 4.0;
    color += frame.light_color.rgb * spec * fresnel * 4.0 * sun;
  }

  // Foam: a band where the water meets geometry, broken up by noise, and on steep crests.
  let edge = 1.0 - clamp(thickness / max(water.foam.w, 1e-3), 0.0, 1.0);
  let noise = foam_noise(in.local * 6.0 + vec2<f32>(t * 0.3, -t * 0.2));
  let crest = clamp((in.slope - 0.35) * 3.0, 0.0, 1.0);
  var band = edge * edge * (0.55 + 0.6 * noise);
  let flowing = dot(water.normals.yz, water.normals.yz) >= 1e-12 && water.normals.w < 0.5 && water.ocean.x < 0.5;
  if (flowing) {
    // A stream's foam comes and goes along its banks: patches carried down by the flow, more of it where the water
    // runs fast and shallow, none where it is still (a uniform band read as a seam).
    var f = water.normals.yz;
    if (water.ao.y > 0.5) { f = flow_at(in.uv) * length(water.normals.yz); }
    let speed = length(f);
    let drift = f * t;
    // Cut at its middle, so half of a bank has none (a lower cut left a faint foam all along it: a bright line).
    let patches = smoothstep(0.55, 0.75, foam_noise((in.local - drift) * 0.9) * 0.7 + foam_noise((in.local - drift) * 3.3) * 0.3);
    // Faded by depth: none at the bank's very contact (a band peaked there read as one bright line along the whole
    // bank), most a third of the band out where the water piles against it.
    let d = 1.0 - edge;
    band = smoothstep(0.02, 0.3, d) * (1.0 - smoothstep(0.3, 1.0, d)) * (0.3 + 0.8 * noise) * patches * smoothstep(0.05, 0.6, speed);
  }
  // The ripples' foam: lumps with gaps between them, its cover growing with how much of it there is (a soft disc of
  // it read as a glow), drifting slowly.
  let lumps = foam_noise(in.local * 2.3 + vec2<f32>(t * 0.15, t * 0.1)) * 0.6 + foam_noise(in.local * 7.1 - vec2<f32>(t * 0.2, 0.0)) * 0.4;
  let amount = clamp(ripple_foam, 0.0, 1.0);
  let lumpy = smoothstep(1.0 - amount, 1.12 - amount, lumps) * (0.5 + 0.5 * amount);
  let foam = clamp(band + crest * noise + lumpy, 0.0, 1.0);
  let lit = max(dot(in.normal, l), 0.0) * frame.light_color.rgb * sun + vec3<f32>(frame.params.x * 0.6);
  color = mix(color, water.foam.rgb * lit, foam * step(0.001, water.foam.w));
  if (sea) {
    // A sea's foam, whatever the shore foam's width, web-ocean's (`ocean.js` fs): a matte, diffuse layer over the
    // water, its albedo the foam colour varied by the bubbles inside it, lit by the sun (half of it: the lumpy foam
    // shadows itself at a grazing sun) and by the whole sky above, nearly grey (foam scatters every part of it).
    // The sun's share is Lambert's, its irradiance over pi as web-ocean's (0.318) and pbr_fs's: the port had dropped
    // the 1/pi, lighting sunlit foam three times too bright.
    let alb = water.foam.rgb * (0.78 + 0.3 * sea_f.bubbles);
    let amb = amb_up + amb_n * 0.3;
    let amb_grey = mix(amb, vec3<f32>(dot(amb, vec3<f32>(0.2126, 0.7152, 0.0722))), 0.85);
    // The deck's irradiance over its radiance: its cosine-weighted mean, brighter overhead (overcast()'s 1 + 0.6 r.y)
    // as 1 + 0.6 * 2/3.
    let sky_light = mix(amb_grey * 1.55, deck_up.rgb * 1.4, deck_up.a);
    let foam_lit = alb * (sun_e * max(dot(n, l), 0.0) * 0.5 * 0.31831 + sky_light);
    color = mix(color, foam_lit, sea_f.mask * 0.95);
  }
  return vec4<f32>(apply_fog(color, in.world), 1.0);
}

// The surface's view depth for the screen AO (lib.rs, after the prepass): three.js' Water and Water2 meshes wrote depth,
// so the original's post AO saw the water's surface, not the bed under it. Drawn into a copy of the prepass depth,
// tested against the prepass, so the AO alone sees the water (the water itself still reads the bed's depth).
@fragment
fn fs_ao_depth(in: WaterOut) -> @location(0) vec4<f32> {
  return vec4<f32>(view_depth(in.world), 0.0, 0.0, 1.0);
}
