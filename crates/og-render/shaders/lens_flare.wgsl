// Lens flare (og.LensFlare, post/flare.rs). Prepended by
// fullscreen.wgsl. Every pass binds the same layout; slots a pass does
// not use hold placeholders.
//
// The screen-space flare is Filament's (google/filament 5ce4cbf19,
// filament/src/materials/flare/flare.mat, Apache-2.0,
// third_party/lens-flare/filament/), itself after John Chapman's "Screen
// Space Lens Flare" (2017). Its ghosts mirror the bright parts of the
// bloom chain's first level through the centre of the frame, and its halo
// is a ring of them warped outwards, both with chromatic aberration. It
// is drawn at a quarter of the frame and blurred by Filament's 9-tap
// separable Gaussian (PostProcessManager.cpp gaussianBlurPass, kernel
// width 9, sigma 10/6). The tone curve then adds it, Filament's starburst
// mask (post.wgsl `starburst`) on its halo only.
//
// The sun and the brightest lights also flare as sources of their own:
// each casts ghosts where the taps would mirror it, sprites of a smooth
// aperture holding a share of its disc's radiance, and feeds the halo's
// taps, all scaled by how much of it the depth shows. That fraction comes from
// a grid of view-depth samples around the light's position on screen, on
// the GPU with no readback (`fs_flare_occlusion`), as Wicked Engine's
// lens flare finds it (turanszkij/WickedEngine e8be9c773,
// WickedEngine/shaders/lensFlareVS.hlsl, MIT,
// third_party/lens-flare/wicked/). Around each source the bloom taps that
// show it (the sky round the sun, a lamp's own bulb) are masked out, so
// it flares once, through its own disc and occlusion.
//
// Units: the flare leaves here exposed exactly once, as bloom.0 is (the
// prefilter multiplies by the exposure), so the tone curve adds it as it
// adds bloom, without exposing it again. The bloom taps are already
// exposed; a light's disc is its unexposed radiance (`colors`) times
// `flare_exposure()`, the same exposure the prefilter read.

const FLARE_LIGHTS: u32 = 8u;

struct Flare {
  size: vec4<f32>,      // xy: this pass's target size, z: the frame's aspect (w / h), w: 1 / aspect
  ghosts: vec4<f32>,    // x: count, y: spacing, z: threshold, w: chromatic aberration (uv)
  halo: vec4<f32>,      // x: radius, y: thickness (0: none), z: threshold, w: the sky's view depth (the camera's far plane)
  exposure: vec4<f32>,  // as post.wgsl Post.exposure
  meter: vec4<f32>,     // x: exposure compensation, y: the lights (0 to FLARE_LIGHTS)
  mask: vec4<f32>,      // x, y: the bloom taps masked round the sun and round a light, in its disc's radii; z: a source ghost's least aperture radius (height units); w: a source ghost's share of its exposed radiance
  // Each light: xy its uv, z the view depth a sample must reach to see
  // it (below 0: the sky, for the sun), w its disc's radius (height units).
  lights: array<vec4<f32>, 8>,
  // rgb: the light's radiance as seen, unexposed (flare_tap exposes it),
  // w: its flare's gain.
  colors: array<vec4<f32>, 8>,
};
@group(0) @binding(0) var<uniform> flare: Flare;
@group(0) @binding(1) var flare_linear: sampler;
@group(0) @binding(2) var flare_source: texture_2d<f32>;      // bloom.0, or the flare being blurred
@group(0) @binding(3) var flare_depth: texture_2d<f32>;       // the prepass view depth
@group(0) @binding(4) var flare_luminance: texture_2d<f32>;   // the metered or adapted exposure
@group(0) @binding(5) var flare_visibility: texture_2d<f32>;  // flare.vis: each light's visible fraction

// post.wgsl `exposure`, from this pass's copy of the Post rows.
fn flare_exposure() -> f32 {
  if (flare.exposure.y > 1.5) {
    return exp2(textureLoad(flare_luminance, vec2<i32>(0), 0).r);
  }
  if (flare.exposure.y > 0.5) {
    let avg = textureLoad(flare_luminance, vec2<i32>(0), 0).r;
    return exp2(clamp(log2(0.18) - avg + flare.meter.x, flare.exposure.z, flare.exposure.w));
  }
  return exp2(flare.exposure.x);
}

fn flare_is_sky(d: f32) -> bool {
  return d >= flare.halo.w * 0.999;
}

// Distance in units of the frame's height.
fn flare_distance(a: vec2<f32>, b: vec2<f32>) -> f32 {
  return length((a - b) * vec2<f32>(flare.size.z, 1.0));
}

// Whether a sample of view depth `d` shows a light that needs `z` (below 0:
// the sky).
fn flare_sees(d: f32, z: f32) -> bool {
  return select(d >= z, flare_is_sky(d), z < 0.0);
}

// One light's visible fraction (Wicked's lensFlareVS: the depth samples
// at or behind the light, of a grid around it), into texel x of flare.vis:
// a 9 x 9 grid over its disc, samples off the frame not counted.
@fragment
fn fs_flare_occlusion(in: VsOut) -> @location(0) vec4<f32> {
  let i = u32(in.pos.x);
  if (i >= u32(flare.meter.y)) { return vec4<f32>(0.0); }
  let light = flare.lights[i];
  let size = vec2<f32>(textureDimensions(flare_depth));
  let center = light.xy * size;
  // The radius in depth texels (height units times the height), at
  // least one texel so a far light reads its own pixel's neighbours.
  let r = max(light.w * size.y, 1.0);
  var samples = 0.0;
  var visible = 0.0;
  for (var y = 0; y < 9; y++) {
    for (var x = 0; x < 9; x++) {
      let o = vec2<f32>(f32(x), f32(y)) / 4.0 - 1.0;
      if (dot(o, o) > 1.0) { continue; }
      let p = center + o * r;
      if (any(p < vec2<f32>(0.0)) || any(p >= size)) { continue; }
      let d = textureLoad(flare_depth, vec2<i32>(p), 0).r;
      samples += 1.0;
      visible += select(0.0, 1.0, flare_sees(d, light.z));
    }
  }
  return vec4<f32>(visible / max(samples, 1.0), 0.0, 0.0, 1.0);
}

// The bloom chain's first level at `uv`, masked out near each source
// (the sky round the sun, a lamp's bulb): the source's own disc, weighted
// by its visibility, replaces it. Every tap in the mask goes, whatever its
// depth: the bloom chain's downsample spreads a partly hidden source's
// visible sliver onto the occluder's pixels beside it, and those, ghosted
// on top of the disc, brightened the flare as the sun slid behind a wall.
fn flare_screen(uv: vec2<f32>) -> vec3<f32> {
  let n = min(u32(flare.meter.y), FLARE_LIGHTS);
  for (var i = 0u; i < n; i++) {
    let light = flare.lights[i];
    let radii = select(flare.mask.y, flare.mask.x, light.z < 0.0);
    if (flare_distance(uv, light.xy) < flare_radius(i) * radii) {
      return vec3<f32>(0.0);
    }
  }
  return textureSampleLevel(flare_source, flare_linear, uv, 0.0).rgb;
}

// Light `i`'s disc's radius (height units): at least one texel of this
// pass's target (a far lamp's disc is under a pixel, which the bloom
// chain's downsample would have spread over a texel too).
fn flare_radius(i: u32) -> f32 {
  return max(flare.lights[i].w, 1.0 / flare.size.y);
}

// flare.mat `sampleColor` thresholded: the bloom level's red, green and
// blue `offset` apart along the line to the centre, each over `threshold`
// and clamped to 1.
fn flare_tap(uv: vec2<f32>, threshold: f32) -> vec3<f32> {
  // (Filament: avoids a division by zero in normalize.)
  let offset = normalize(vec2<f32>(0.5001) - uv) * flare.ghosts.w;
  let screen = vec3<f32>(
    flare_screen(uv + offset).r,
    flare_screen(uv).g,
    flare_screen(uv - offset).b);
  return clamp(screen - threshold, vec3<f32>(0.0), vec3<f32>(1.0));
}

// A source ghost's least radius in this pass's texels: smaller, a ghost of
// a far source was under two texels, and where its quarter-frame texels
// fell moved its brightness frame to frame by more than its occlusion did.
const FLARE_APERTURE_TEXELS: f32 = 6.0;
// The aperture's flat top, as a fraction of its radius; its rim rolls off
// smoothly over the rest. `FLARE_APERTURE_AREA` is its integral over the
// unit disc, as a fraction of the disc (0.36 + 0.288).
const FLARE_APERTURE_EDGE: f32 = 0.6;
const FLARE_APERTURE_AREA: f32 = 0.648;
// The brightness a source ghost's peak rolls off above, and twice it the
// most it reaches (exposed units, before the intensity), as Filament
// clamps each tap at 1.
const FLARE_CORE_KNEE: f32 = 0.5;

// A defocused aperture: an even disc with a soft rim, never a line or a
// ring inside it.
fn flare_aperture(t: f32) -> f32 {
  return 1.0 - smoothstep(FLARE_APERTURE_EDGE, 1.0, t);
}

// A source ghost's least radius (height units).
fn flare_aperture_radius() -> f32 {
  return max(flare.mask.z, FLARE_APERTURE_TEXELS / flare.size.y);
}

// The light of source `i`'s disc, exposed, over `threshold`, times the
// gain `mask.w` and spread over an aperture of radius `ra` with its
// energy kept, its peak rolled off past FLARE_CORE_KNEE (so a sun's ghost
// is veiling glare, never a white disc), then times the share of it the
// depth shows and its gain. The roll-off comes before the visible share,
// so the ghost stays in proportion to how much of its source is seen.
fn flare_source_light(i: u32, threshold: f32, ra: f32, exposure: f32) -> vec3<f32> {
  let seen = textureLoad(flare_visibility, vec2<i32>(i32(i), 0), 0).r * flare.colors[i].w;
  if (seen <= 0.0) { return vec3<f32>(0.0); }
  let r = flare_radius(i);
  let spread = (r / ra) * (r / ra) / FLARE_APERTURE_AREA;
  let l = max(flare.colors[i].rgb * exposure - threshold, vec3<f32>(0.0)) * (flare.mask.w * spread);
  let peak = max(l.r, max(l.g, l.b));
  let over = max(peak - FLARE_CORE_KNEE, 0.0);
  let rolled = FLARE_CORE_KNEE + over / (1.0 + over / FLARE_CORE_KNEE);
  return l * (min(rolled, peak) / max(peak, 1e-6) * seen);
}

// The sources' share of the halo tap at `uv` (the halo warps the frame,
// so its arcs of a source are taken where its taps land on the source's
// aperture).
fn flare_sources_at(uv: vec2<f32>, threshold: f32) -> vec3<f32> {
  let n = min(u32(flare.meter.y), FLARE_LIGHTS);
  if (n == 0u) { return vec3<f32>(0.0); }
  let exposure = flare_exposure();
  var c = vec3<f32>(0.0);
  for (var i = 0u; i < n; i++) {
    let ra = max(flare_radius(i), flare_aperture_radius());
    let t = flare_distance(uv, flare.lights[i].xy) / ra;
    if (t >= 1.0) { continue; }
    c += flare_source_light(i, threshold, ra, exposure) * flare_aperture(t);
  }
  return c;
}

// The sources' ghosts at `q` (this pass's uv), drawn where the ghost taps
// would mirror them, as sprites of the lens's defocused aperture: ghost
// `k` of a source at `p` stands where `fract(1 - q + (q - 0.5) s k)`
// reaches `p` (`flare_ghosts`), at `1 - (p - s k / 2) / (1 - s k)`, and
// is the source's aperture magnified by `1 / |1 - s k|` (at most 3 times)
// with its light spread over it, so a large ghost is a faint one. Taken in
// the taps' own space instead, the magnified ghosts had their sources'
// brightness over up to 25 times the area, and every row of the
// quarter-frame texels they stretched showed. Not split into red, green
// and blue (the offsets copied the disc three times in rainbow colours),
// and one coating tint per ghost.
fn flare_source_ghosts(q: vec2<f32>) -> vec3<f32> {
  let n = min(u32(flare.meter.y), FLARE_LIGHTS);
  if (n == 0u) { return vec3<f32>(0.0); }
  let exposure = flare_exposure();
  let s = flare.ghosts.y;
  var c = vec3<f32>(0.0);
  for (var i = 0u; i < n; i++) {
    let ra0 = max(flare_radius(i), flare_aperture_radius());
    let p = flare.lights[i].xy;
    for (var k = 0.0; k < flare.ghosts.x; k += 1.0) {
      let m = 1.0 - s * k;
      if (abs(m) < 0.05) { continue; }
      let center = 1.0 - (p - 0.5 * s * k) / m;
      let ra = ra0 * clamp(1.0 / abs(m), 1.0, 3.0);
      let t = flare_distance(q, center) / ra;
      if (t >= 1.0) { continue; }
      c += flare_source_light(i, flare.ghosts.z, ra, exposure) * flare_aperture(t)
        * flare_ghost_tint(k);
    }
  }
  return c;
}

// A coating tint per ghost (amber, teal, violet in turn), each of luma 1,
// in place of a rainbow across one ghost.
fn flare_ghost_tint(i: f32) -> vec3<f32> {
  let k = u32(i) % 3u;
  let tint = select(select(vec3<f32>(0.78, 0.6, 1.0), vec3<f32>(0.55, 1.05, 1.0), k == 1u),
    vec3<f32>(1.25, 0.95, 0.6), k == 0u);
  return tint / dot(tint, LUMA);
}

// flare.mat `smoothstep01` and `ring`.
fn flare_smoothstep01(t: f32) -> f32 {
  return t * t * (3.0 - 2.0 * t);
}

fn flare_ring(x: f32, c: f32, r: f32) -> f32 {
  return 1.0 - flare_smoothstep01(min(abs(x - c) / r, 1.0));
}

// flare.mat `ghosts`: taps along the line through the centre, wrapped.
fn flare_ghosts(vertex_uv: vec2<f32>) -> vec3<f32> {
  let uv = 1.0 - vertex_uv;
  let v = (0.5 - uv) * flare.ghosts.y;
  var color = vec3<f32>(0.0);
  for (var i = 0.0; i < flare.ghosts.x; i += 1.0) {
    color += flare_tap(fract(uv + v * i), flare.ghosts.z) * flare_ghost_tint(i);
  }
  return color;
}

// flare.mat `halo`: the frame warped outwards by a fixed length, kept on
// a ring (round whatever the aspect).
fn flare_halo(uv: vec2<f32>) -> vec3<f32> {
  let aspect = vec2<f32>(flare.size.z, flare.size.w);
  var v = vec2<f32>(0.5001) - uv;
  v.x *= aspect.x;
  v = normalize(v);
  v.x *= aspect.y;
  let tap = uv + v * flare.halo.x;
  let s = flare_tap(tap, flare.halo.z) + flare_sources_at(tap, flare.halo.z);
  let uv_weight = (uv - vec2<f32>(0.5, 0.0)) * vec2<f32>(aspect.x, 1.0) + vec2<f32>(0.5, 0.0);
  let d = distance(uv_weight, vec2<f32>(0.5));
  return s * flare_ring(d, flare.halo.x, flare.halo.y);
}

// flare.mat `postProcess`: the ghosts and the halo at a quarter of the
// frame, the sources' ghosts as aperture sprites beside the screen's. The
// alpha holds the halo's luma alone, which the blur keeps and the tone
// curve reads to put the starburst on the halo only (post.wgsl): over a
// ghost, its rays from the centre of the frame drew lines across the disc.
@fragment
fn fs_flare(in: VsOut) -> @location(0) vec4<f32> {
  var color = vec3<f32>(0.0);
  if (flare.ghosts.x > 0.0) {
    color += flare_ghosts(in.uv) + flare_source_ghosts(in.uv);
  }
  var halo = vec3<f32>(0.0);
  if (flare.halo.y > 0.0) {
    halo = flare_halo(in.uv);
  }
  return vec4<f32>(color + halo, dot(halo, LUMA));
}

// separableGaussianBlur.fs with gaussianBlurPass's coefficients for
// kernel width 9, sigma 10/6: the centre tap and two bilinear pairs each
// side, each pair two texels' Gaussian weights at their weighted offset.
// The alpha (the halo's luma) is blurred with the colour.
fn flare_blur(uv: vec2<f32>, axis: vec2<f32>) -> vec4<f32> {
  let sigma = 10.0 / 6.0;
  let alpha = 1.0 / (2.0 * sigma * sigma);
  var sum = textureSampleLevel(flare_source, flare_linear, uv, 0.0);
  var total = 1.0;
  for (var i = 1; i < 3; i++) {
    let x0 = f32(i * 2 - 1);
    let x1 = f32(i * 2);
    let k0 = exp(-alpha * x0 * x0);
    let k1 = exp(-alpha * x1 * x1);
    let k = k0 + k1;
    let o = axis * (x0 + k1 / k);
    sum += (textureSampleLevel(flare_source, flare_linear, uv + o, 0.0)
      + textureSampleLevel(flare_source, flare_linear, uv - o, 0.0)) * k;
    total += 2.0 * k;
  }
  return sum / total;
}

@fragment
fn fs_flare_blur_x(in: VsOut) -> @location(0) vec4<f32> {
  let texel = 1.0 / vec2<f32>(textureDimensions(flare_source));
  return flare_blur(in.uv, vec2<f32>(texel.x, 0.0));
}

@fragment
fn fs_flare_blur_y(in: VsOut) -> @location(0) vec4<f32> {
  let texel = 1.0 / vec2<f32>(textureDimensions(flare_source));
  return flare_blur(in.uv, vec2<f32>(0.0, texel.y));
}
