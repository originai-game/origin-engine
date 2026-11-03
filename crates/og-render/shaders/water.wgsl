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
// R5: a sea (Water::wind) displaces the grid by og_core::ocean's Gerstner
// waves over the absolute world's xz, the vertices taking the waves the
// grid can hold and the fragments every wave's normal, whitecaps where the
// crests bunch; a flow map (Water::flow_map) turns the flow per point.

struct WaterUniform {
  model: mat4x4<f32>,
  size: vec4<f32>,        // xy: extent along local x and z, z: segments, w: 1 with a height field
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
  ocean: vec4<f32>,       // x: 1 with a sea, y: the grid's centre cell (m), z: the crests' bunching where whitecaps start, w: over which they grow
  sea_wind: vec4<f32>,    // xy: the wind's direction in the frame's xz, z: its speed (m/s; 0 without a sea)
  // Per wave: (k d.x, k d.z, amplitude, phase at the frame's origin), (lean d.x, lean d.z, k lean, 1 when its crests
  // whiten) (water.rs ocean_gpu).
  swell: array<vec4<f32>, 64>,
};

@group(1) @binding(0) var<uniform> water: WaterUniform;
@group(1) @binding(1) var heights: texture_2d<f32>;
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

// The sea's displacement (x, height, z) at rest point p (frame xz): the
// waves at least four cells long (`cell`; 0: every wave), the shorter
// left to the fragments' normals.
fn ocean_displace(p: vec2<f32>, cell: f32) -> vec3<f32> {
  var d = vec3<f32>(0.0);
  for (var i = 0u; i < 32u; i++) {
    let a = water.swell[2u * i];
    let b = water.swell[2u * i + 1u];
    let keep = 1.0 - smoothstep(0.785, 1.571, length(a.xy) * cell);
    let th = dot(a.xy, p) + a.w;
    d += vec3<f32>(b.x * cos(th), a.z * sin(th), b.y * cos(th)) * keep;
  }
  return d;
}

// The sea's normal at rest point p (frame xz; Gerstner's, GPU Gems 1 §1)
// and, as w, how far the crests of the waves that whiten bunch there
// (Σ k lean sin over them: the surface's area lost, whitecaps past
// water.ocean.z), each wave faded where it is shorter than four of the
// pixel's `footprint` (m), and the mean square slope the pixel holds
// unresolved: the faded waves' (their (k a)² / 2), and the short waves'
// that the facets carry (see short_slope). A pixel of sea is a rough
// mirror, not a glass one (Bruneton, Neyret and Holzschuch 2010).
struct SeaNormal {
  n: vec3<f32>,
  bunch: f32,
  mss: f32,
};

// How much of a short wave's slope (one that never whitens) the normal
// draws; the rest of its slope variance, 1 - SHORT_SLOPE², is the facets'
// roughness. The waves carry the whole of Cox and Munk's mean square slope,
// but in waves no shorter than a metre or three: drawn whole, every near
// slope was a smooth mirror of the sky. A gale's short sea is choppier
// than its waves can draw: from 10 m/s the share falls to 0.45 at 20, and
// its surface reads rough, not glossy.
fn short_slope() -> f32 {
  return mix(0.7, 0.45, smoothstep(10.0, 20.0, water.sea_wind.z));
}

fn ocean_normal(p: vec2<f32>, footprint: f32) -> SeaNormal {
  var n = vec3<f32>(0.0, 1.0, 0.0);
  var bunch = 0.0;
  var mss = 0.0;
  let short = short_slope();
  for (var i = 0u; i < 32u; i++) {
    let a = water.swell[2u * i];
    let b = water.swell[2u * i + 1u];
    let fade = 1.0 - smoothstep(0.785, 1.571, length(a.xy) * footprint);
    let keep = fade * mix(short, 1.0, b.w);
    let th = dot(a.xy, p) + a.w;
    let c = cos(th) * a.z * keep;
    let s = b.z * sin(th) * keep;
    n -= vec3<f32>(a.x * c, s, a.y * c);
    bunch += s * b.w;
    let ka = length(a.xy) * a.z;
    mss += ka * ka * 0.5 * (1.0 - keep * keep);
  }
  var o: SeaNormal;
  o.n = normalize(n);
  o.bunch = bunch;
  o.mss = mss;
  return o;
}

// Bruneton et al. 2010's mean Fresnel over a rough sea's facets: the
// grazing mirror dims as the slopes it averages spread (sigma: their
// deviation along the view).
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

// A sea's grid: finest at its centre (where the camera is meant to be),
// its cells growing outward so it reaches far towards the horizon. g in
// [-1, 1] across the grid to [-1, 1] across the surface, w(g) = g (0.15 +
// 0.85 g²); its slope w' = 0.15 + 2.55 g² is the cell's size over the
// even grid's.
fn sea_warp(g: f32) -> vec2<f32> {
  return vec2<f32>(g * (0.15 + 0.85 * g * g), 0.15 + 2.55 * g * g);
}

// The height field at local point (x, z) of the surface, bilinear.
fn height_at(local: vec2<f32>) -> f32 {
  if (water.size.w < 0.5) { return 0.0; }
  let n = i32(water.look.w);
  let cell = water.look.z / f32(n - 1);
  let g = (local + vec2<f32>(water.look.z * 0.5)) / cell;
  let i = clamp(vec2<i32>(floor(g)), vec2<i32>(0), vec2<i32>(n - 2));
  let f = clamp(g - vec2<f32>(i), vec2<f32>(0.0), vec2<f32>(1.0));
  let a = textureLoad(heights, i, 0).r;
  let b = textureLoad(heights, i + vec2<i32>(1, 0), 0).r;
  let c = textureLoad(heights, i + vec2<i32>(0, 1), 0).r;
  let d = textureLoad(heights, i + vec2<i32>(1, 1), 0).r;
  return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}

@vertex
fn vs(@builtin(vertex_index) vi: u32) -> WaterOut {
  let segs = u32(water.size.z);
  let quad = vi / 6u;
  var corners = array<vec2<f32>, 6>(
    vec2<f32>(0.0, 0.0), vec2<f32>(0.0, 1.0), vec2<f32>(1.0, 1.0),
    vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(1.0, 0.0));
  let c = corners[vi % 6u];
  let cell = vec2<f32>(f32(quad % segs), f32(quad / segs)) + c;
  var local = (cell / f32(segs) - 0.5) * water.size.xy;
  // The sea's cell here, in metres (its waves shorter than four of it are left to the fragments).
  var sea_cell = 0.0;
  if (water.ocean.x > 0.5) {
    let wx = sea_warp(cell.x / f32(segs) * 2.0 - 1.0);
    let wz = sea_warp(cell.y / f32(segs) * 2.0 - 1.0);
    local = vec2<f32>(wx.x, wz.x) * water.size.xy * 0.5;
    sea_cell = water.ocean.y * max(wx.y, wz.y);
  }
  let h = height_at(local);
  // The height field's slope gives the geometric normal.
  let e = water.look.z / max(water.look.w - 1.0, 1.0);
  let hx = height_at(local + vec2<f32>(e, 0.0)) - height_at(local - vec2<f32>(e, 0.0));
  let hz = height_at(local + vec2<f32>(0.0, e)) - height_at(local - vec2<f32>(0.0, e));
  let n_local = normalize(vec3<f32>(-hx / (2.0 * e), 1.0, -hz / (2.0 * e)));
  var world = water.model * vec4<f32>(local.x, h, local.y, 1.0);
  let rest = world.xz;
  if (water.ocean.x > 0.5) {
    world = vec4<f32>(world.xyz + ocean_displace(rest, sea_cell), 1.0);
  }
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

@fragment
fn fs(in: WaterOut) -> @location(0) vec4<f32> {
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
  // The sea's own normal under the detail (a level surface: its waves are the world's), and its whitecaps.
  var base = in.normal;
  var whitecap = 0.0;
  var sea_mss = 0.0;
  if (water.ocean.x > 0.5) {
    var footprint = sea_footprint(in.world);
    if (frame.view_pos.w > 0.5) { footprint = max(length(dpdx(in.rest)), length(dpdy(in.rest))); }
    let sea = ocean_normal(in.rest, footprint);
    base = sea.n;
    sea_mss = sea.mss;
    // Whitecaps where the crests bunch past the wind's share (water.rs), broken up along the wind's waves.
    if (water.ocean.w > 0.0) {
      // Coordinates along the wind (x) and across it (y): foam is drawn out downwind.
      let wd = water.sea_wind.xy;
      let q = vec2<f32>(dot(in.rest, wd), dot(in.rest, vec2<f32>(-wd.y, wd.x)));
      let streaks = foam_noise(q * vec2<f32>(0.12, 0.45)) * 0.6 + foam_noise(q * vec2<f32>(0.6, 2.1)) * 0.4;
      let core = smoothstep(water.ocean.z, water.ocean.z + water.ocean.w, sea.bunch) * smoothstep(0.2, 0.5, streaks);
      // White water is lace, never a sheet: fine noise (its octaves faded to their mean where a pixel spans them)
      // cut at a level the crest's core lowers, its edge soft, so a whitecap fades out by its coverage. Only the
      // breaking crest itself (the core near 1) is opaque white.
      let fine = mix(foam_noise(q * vec2<f32>(4.5, 11.0)), 0.5, smoothstep(0.05, 0.12, footprint));
      let mid = mix(foam_noise(q * vec2<f32>(1.6, 3.8)), 0.5, smoothstep(0.15, 0.35, footprint));
      let lace = foam_noise(q * vec2<f32>(0.6, 1.4)) * 0.45 + mid * 0.33 + fine * 0.22;
      let cut = 1.0 - 0.6 * core;
      let cover = smoothstep(cut - 0.12, cut + 0.1, lace);
      whitecap = cover * mix(0.45, 0.92, core * core) * smoothstep(0.0, 0.2, core);
      // A gale's old foam: thin streaks drawn out along the wind across the whole sea (from 12 m/s), the wash the
      // broken crests leave behind them.
      let trail = foam_noise(q * vec2<f32>(0.04, 0.9)) * 0.65 + mid * 0.2 + fine * 0.15;
      let old = smoothstep(0.6, 0.78, trail) * 0.35 * smoothstep(12.0, 20.0, water.sea_wind.z);
      whitecap = max(whitecap, old);
    }
    // The detail ripples are the sea's own shortest waves: kept only where a pixel holds them (else they alias into
    // stripes towards the horizon).
    slope *= 1.0 - smoothstep(0.785, 1.571, water.waves.x * 6.2831853 * 3.9 * footprint);
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
  // Shadowed water scatters less light: its deep colour darkens.
  var deep = water.deep.rgb * mix(0.4, 1.0, sun);
  if (water_js) {
    // three.js Water's body, (sunColor · diffuse · 0.3 + scatter) · shadow: diffuse = max(L·N, 0) · sunColor · 0.5,
    // scatter = max(N·V, 0) · waterColor (the deep colour), sunColor = the light × glint / 2 (glint = spec 2 × sunColor
    // over the light). The ripples shade it, as they do the original's.
    let sc = frame.light_color.rgb * water.under.w * 0.5;
    deep = (sc * sc * 0.15 * max(dot(l, n), 0.0) + water.deep.rgb * max(dot(n, v), 0.0)) * sun;
  }
  var color = mix(below * water.shallow.rgb, deep, clamp(absorb * water.deep.w, 0.0, 1.0));

  // Reflection with Schlick's Fresnel.
  let r = reflect(-v, n);
  let f0 = water.extra.z;
  var fresnel = (f0 + (1.0 - f0) * pow(1.0 - max(dot(n, v), 0.0), 5.0)) * water.look.x;
  // A sea's facets: their slopes' deviation per axis, its Beckmann roughness and the mip it blurs the sky by.
  let sea = water.ocean.x > 0.5;
  let sigma = sqrt(max(sea_mss, 1e-6) * 0.5);
  var mip = 0.5;
  if (sea) {
    fresnel = (f0 + (1.0 - f0) * sea_fresnel(max(dot(n, v), 0.0), sigma)) * water.look.x;
    mip = max(sqrt(sqrt(sea_mss)) * frame.params.y, 0.5);
  }
  // The environment, or the reflection probe holding the surface (probes.rs).
  var reflected = reflected_light(r, in.world, mip) * frame.params.x * 1.8;
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
    // A ray reflected downwards meets the next wave, which shows its body and, dimly, the sky in turn. The facets
    // spread the reflection over about two of their slopes' deviations, so it turns down gradually.
    let up = reflected_light(vec3<f32>(r.x, abs(r.y), r.z), in.world, mip) * frame.params.x * 1.8;
    let w = 2.0 * sigma + 0.05;
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
    // The sun on the facets: Beckmann's distribution of their slopes (Cox and Munk's glitter), Schlick's Fresnel at the
    // half vector (none in shadow).
    let h = normalize(l + v);
    let nh = max(dot(n, h), 1e-3);
    let m2 = max(sea_mss, 2e-3);
    let t2 = (1.0 - nh * nh) / (nh * nh);
    let d = exp(-t2 / m2) / (3.14159265 * m2 * nh * nh * nh * nh);
    let fh = f0 + (1.0 - f0) * pow(1.0 - max(dot(h, v), 0.0), 5.0);
    let spec = d * fh / (4.0 * max(dot(n, v), 0.1)) * step(0.0, dot(n, l));
    color += frame.light_color.rgb * spec * water.look.x * sun;
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
    let patches = smoothstep(0.35, 0.75, foam_noise((in.local - drift) * 0.9) * 0.7 + foam_noise((in.local - drift) * 3.3) * 0.3);
    band = edge * (0.4 + 0.6 * edge) * (0.35 + 0.8 * noise) * patches * smoothstep(0.05, 0.6, speed);
  }
  let foam = clamp(band + crest * noise, 0.0, 1.0);
  let lit = max(dot(in.normal, l), 0.0) * frame.light_color.rgb * sun + vec3<f32>(frame.params.x * 0.6);
  color = mix(color, water.foam.rgb * lit, foam * step(0.001, water.foam.w));
  // A sea's whitecaps, whatever the shore foam's width: white water lit by the sun and the whole sky above it.
  let sky_lit = max(dot(in.normal, l), 0.0) * frame.light_color.rgb * sun + ambient_light_level(in.normal, in.world) * frame.params.x;
  color = mix(color, water.foam.rgb * sky_lit, whitecap);
  return vec4<f32>(apply_fog(color, in.world), 1.0);
}

// The surface's view depth for the screen AO (lib.rs, after the prepass): three.js' Water and Water2 meshes wrote depth,
// so the original's post AO saw the water's surface, not the bed under it. Drawn into a copy of the prepass depth,
// tested against the prepass, so the AO alone sees the water (the water itself still reads the bed's depth).
@fragment
fn fs_ao_depth(in: WaterOut) -> @location(0) vec4<f32> {
  return vec4<f32>(view_depth(in.world), 0.0, 0.0, 1.0);
}
