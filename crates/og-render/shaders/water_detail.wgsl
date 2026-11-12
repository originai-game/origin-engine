// The small waves of water.wgsl (appended to it): a lake's glints, the detail waves, the normal map, the flow and
// the foam's noise.

// The slopes' variance of a lake's capillary ripples, the centimetre wavelets no wave or normal map draws (some
// 2.5°; Cox and Munk's light-wind slopes are wider, but their longer waves are the ones the detail draws), at the
// default wave strength (0.35): the wind raises both, and still water (strength 0) has none and mirrors the sun's
// disc whole.
const CAPILLARY: f32 = 0.002;
// How many capillary facets a square metre of a lake holds (a facet some 4 mm across: a capillary wavelet's face).
const LAKE_FACETS: f32 = 60000.0;

// One draw of a Poisson count of mean `mu` from the uniform `u`: by its CDF while the mean is small, by its normal
// approximation (rounded, never negative) past 12.
fn poisson_count(mu: f32, u: f32, z: f32) -> f32 {
  if (mu > 12.0) { return max(floor(mu + sqrt(mu) * z + 0.5), 0.0); }
  var k = 0.0;
  var term = exp(-mu);
  var cdf = term;
  for (var i = 1; i < 40 && u > cdf; i++) {
    term *= mu / f32(i);
    cdf += term;
    k += 1.0;
  }
  return k;
}

// The lake's glints: how many times its mean glitter a pixel at `p` (local xz, covering `footprint` square metres)
// shows, where `share` of the facets face the sun. The surface is cut into cells the size of the pixel's footprint
// (at two powers of two, blended by where the footprint lies between them, so the cells never pop as the eye moves),
// each holding LAKE_FACETS per square metre, and each cell's count of facets that face the sun is drawn from its own
// hash (a Poisson law of its expected count), rerolled six times a second at the waves' own speed (never on water
// whose waves stand still): discrete counting of glinty facets per footprint, after Zirr and Kaplanyan ("Real-time
// Rendering of Procedural Multiscale Materials", I3D 2016) and Deliot and Belcour ("Real-Time Rendering of Glinty
// Appearances using Distributed Binomial Laws on Anisotropic Grids", HPG 2023), the binomial taken as its Poisson
// limit (a facet facing the sun is rare). A near pixel holds a few facets and shows one glint now and then, sharp and
// bright; a far one holds thousands and averages to the lobe.
fn lake_glints(p: vec2<f32>, footprint: f32, share: f32, t: f32) -> f32 {
  let area = max(footprint, 1e-10);
  let at = 0.5 * log2(area);
  let lo = floor(at);
  var sum = 0.0;
  for (var j = 0; j < 2; j++) {
    let level = lo + f32(j);
    let size = exp2(level);
    let cell = vec2<i32>(floor(p / size));
    let mu = LAKE_FACETS * size * size * share;
    let id = spray_hash((bitcast<u32>(cell.x) * 73856093u) ^ (bitcast<u32>(cell.y) * 19349663u) ^ (bitcast<u32>(i32(level)) * 83492791u));
    let step = floor(t * 6.0 * max(water.waves.y, 0.0) + spray_rand(id));
    let h = spray_hash(id ^ (bitcast<u32>(i32(step)) * 2654435761u));
    let u = spray_rand(h);
    let z = sqrt(-2.0 * log(max(spray_rand(h + 1u), 1e-7))) * cos(6.2831853 * spray_rand(h + 2u));
    let k = poisson_count(mu, u, z);
    let w = select(1.0 - (at - lo), at - lo, j == 1);
    sum += w * select(1.0, k / mu, mu > 1e-6);
  }
  return sum;
}

// The detail normal's slope (local xz) as a pixel of the surface sees it, and the covariance of the slopes inside the
// pixel that its mean leaves out (xx, zz, xz): LEAN mapping (Olano & Baker, "LEAN Mapping", I3D 2010) for the detail
// waves. Each wave a pixel cannot hold is faded out of the mean slope (as sea_keep fades the sea's), and its share of
// the slopes' variance moves into the covariance, which widens the highlight and blurs the reflection instead: a wave
// summed unfiltered past a pixel's reach aliased into moiré rings over a far lake or sea (terrain-far).
struct Detail {
  slope: vec2<f32>,
  cov: vec3<f32>,
};

// One cosine wave of the detail's along unit `d`, wavenumber `k`, slope amplitude `a` at `phase`, over a pixel of `w`
// metres: its share of the mean slope and of the covariance it leaves out (a cosine's slopes vary by a² / 2).
fn detail_wave(o: ptr<function, Detail>, d: vec2<f32>, k: f32, a: f32, phase: f32, w: f32) {
  let keep = sea_keep(k, w);
  (*o).slope += d * cos(phase) * a * keep;
  (*o).cov += vec3<f32>(d.x * d.x, d.y * d.y, d.x * d.y) * (0.5 * a * a * (1.0 - keep * keep));
}

// The detail from travelling waves (two pairs of crossing directions at two scales), in local xz, over a pixel of `w`
// metres (0: every wave kept).
fn wave_slope(p: vec2<f32>, t: f32, w: f32) -> Detail {
  let k = water.waves.x * 6.2831853;
  let s = water.waves.y;
  var dirs = array<vec2<f32>, 4>(
    vec2<f32>(0.8, 0.6), vec2<f32>(-0.6, 0.8), vec2<f32>(0.3, -0.95), vec2<f32>(-0.9, -0.4));
  var scales = array<f32, 4>(1.0, 1.37, 2.71, 3.9);
  var o = Detail(vec2<f32>(0.0), vec3<f32>(0.0));
  for (var i = 0; i < 4; i++) {
    let kk = k * scales[i];
    let phase = dot(dirs[i], p) * kk + t * s * sqrt(kk) * 1.6;
    detail_wave(&o, dirs[i], kk, water.waves.z * 0.25 / scales[i], phase, w);
  }
  return o;
}

// A Blinn-Phong lobe of power `s` widened by the slopes' covariance a pixel leaves out (LEAN): as a Beckmann lobe its
// slopes' variance is 2 / (s + 2), to which the covariance's trace adds; and the peak lowered by as much, so the light
// it reflects stays (the far highlight spreads into a glitter path instead of flickering on the few pixels it hits).
fn lean_power(s: f32, cov: vec3<f32>) -> f32 {
  return max(2.0 / (2.0 / (s + 2.0) + cov.x + cov.y) - 2.0, 1.0);
}

fn lean_energy(s: f32, cov: vec3<f32>) -> f32 {
  return (lean_power(s, cov) + 2.0) / (s + 2.0);
}

// Toksvig's variance ("Mipmapping Normal Maps", JGT 2005) of a normal-map texel `rgb`: the mips average the normals,
// which shortens them where the texels under a pixel disagree; (1 - |n|) / |n| is the slopes' variance they average.
fn toksvig(rgb: vec3<f32>) -> f32 {
  let len = length(rgb * 2.0 - 1.0);
  return clamp((1.0 - len) / max(len, 1e-3), 0.0, 1.0);
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
fn map_slope(world: vec2<f32>, uv: vec2<f32>, t: f32) -> Detail {
  let flow = water.normals.yz;
  // Every derivative in this file is Fine: plain dpdx/fwidth are the backend's choice, and HLSL's ddx is coarse
  // (one value per 2x2 quad) on D3D12, which sharpened the ripples' mips and their glints there. Metal's dfdx and
  // the Vulkan drivers' OpDPdx are already per pixel.
  var n: vec3<f32>;
  // The slopes' variance the mips averaged away (Toksvig), as a share of the normal's (isotropic).
  var lost = 0.0;
  if (dot(flow, flow) < 1e-12) {
    let p = world * water.normals.x;
    let gx = dpdxFine(p);
    let gy = dpdyFine(p);
    let tt = t * water.waves.y;
    // `world` is from the frame's origin: each layer adds its uv there.
    let o = water.layers;
    let r0 = ripple_map(p / 103.0 + o[0].xy + vec2<f32>(tt / 17.0, tt / 29.0), gx / 103.0, gy / 103.0);
    let r1 = ripple_map(p / 107.0 + o[0].zw - vec2<f32>(tt / -19.0, tt / 31.0), gx / 107.0, gy / 107.0);
    let r2 = ripple_map(p / vec2<f32>(8907.0, 9803.0) + o[1].xy + vec2<f32>(tt / 101.0, tt / 97.0), gx / vec2<f32>(8907.0, 9803.0), gy / vec2<f32>(8907.0, 9803.0));
    let r3 = ripple_map(p / vec2<f32>(1091.0, 1027.0) + o[1].zw - vec2<f32>(tt / 109.0, tt / -113.0), gx / vec2<f32>(1091.0, 1027.0), gy / vec2<f32>(1091.0, 1027.0));
    let noise = (r0 + r1 + r2 + r3) * 0.5 - 1.0;
    n = normalize(vec3<f32>(noise.x * 1.5, noise.z, noise.y * 1.5));
    // Four layers averaged: each one's variance a sixteenth, its slopes stretched 1.5 times along x and z.
    lost = (toksvig(r0) + toksvig(r1) + toksvig(r2) + toksvig(r3)) * (2.25 / 16.0);
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
    let gx = dpdxFine(p);
    let gy = dpdyFine(p);
    let a = ripple_map(p + dir * o0, gx, gy);
    let b = ripple_map(p + dir * o1, gx, gy);
    let wb = abs(half_cycle - o0) / half_cycle;
    let c = mix(a, b, wb);
    n = normalize(vec3<f32>(c.r * 2.0 - 1.0, c.b, c.g * 2.0 - 1.0));
    lost = mix(toksvig(a), toksvig(b), wb);
  }
  // Split between x and z (an isotropic share), scaled as the slope is.
  let v = 0.5 * lost * water.waves.z * water.waves.z;
  return Detail(-n.xz / max(n.y, 0.05) * water.waves.z, vec3<f32>(v, v, 0.0));
}

// The flow map's direction at the mesh's uv, or at the grid's metres (u along x and v along z from the -x -z
// corner): rg * 2 - 1, the image's top at v 1 (three.js' flipY).
fn flow_at(uv: vec2<f32>) -> vec2<f32> {
  return flow_texel(uv).rg * 2.0 - 1.0;
}

fn flow_texel(uv: vec2<f32>) -> vec4<f32> {
  var at = uv;
  if (water.ao.z < 0.5) { at = uv / max(water.size.xy, vec2<f32>(1e-3)) + 0.5; }
  return textureSampleLevel(flow_tex, water_sampler, vec2<f32>(at.x, 1.0 - at.y), 0.0);
}

// A stream's ripples: wave fronts across the flow's direction `d` (unit), within half a radian of it, at the detail
// waves' scale and strength.
fn stream_slope(p: vec2<f32>, d_in: vec2<f32>, t: f32, fast: f32, w: f32, river: bool) -> Detail {
  // Where the flow is slower than the surface's flow (a flow map's still water, `fast` under 1), its ripples are
  // longer and lower; at the flow's own speed they are as before.
  let pace = mix(0.7, 1.0, smoothstep(0.3, 1.0, fast));
  let k = water.waves.x * 6.2831853 * pace;
  // W5: a river's reach is glassy, its surface marked only by faint, irregular streaks drawn out along the current
  // (the shear's boils and seams), not by fronts across it: fronts at one wavenumber over a long reach read, at a
  // grazing view, as bank-to-bank bands. So its trains run along the flow (their wave vectors across it), lower.
  let d = select(d_in, vec2<f32>(-d_in.y, d_in.x), river);
  let side = vec2<f32>(-d.y, d.x);
  var turns = array<f32, 4>(0.0, 0.45, -0.3, 0.15);
  var scales = array<f32, 4>(1.0, 1.37, 1.9, 2.71);
  var o = Detail(vec2<f32>(0.0), vec3<f32>(0.0));
  let a = water.waves.z * 0.25 * mix(0.55, 1.0, pace * pace) * select(1.0, 0.35, river);
  for (var i = 0; i < 4; i++) {
    let dir = d * cos(turns[i]) + side * sin(turns[i]);
    let kk = k * scales[i];
    // Each train's fronts warped and its height varied by a slow noise of its own, so the trains do not lock into a
    // regular grid of dimples.
    let fi = f32(i);
    let warp = foam_noise(p * 0.21 + vec2<f32>(fi * 3.7, fi * 1.3)) * 5.0;
    let gain = 0.45 + foam_noise(p * 0.33 + vec2<f32>(fi * 2.1, fi * 5.9)) * 1.1;
    let phase = dot(dir, p) * kk + warp + t * water.waves.y * sqrt(kk) * 1.6;
    detail_wave(&o, dir, kk, a * gain / scales[i], phase, w);
  }
  return o;
}

// The travelling waves carried by the flow (Water::flow, metres a second; per point with a flow map), as Water2
// carries its layers: fronts across the flow's own direction, two copies sliding along the flow, each restarting
// every three seconds half a period apart and cross-faded so neither restart shows. A flow map moves each point by
// its own flow, so the fronts bend where the flow does (round a rock, by up to its speed times a period).
fn flowed_slope(p: vec2<f32>, uv: vec2<f32>, t: f32, w: f32) -> Detail {
  var f = water.normals.yz;
  if (water.ao.y > 0.5) { f = flow_at(uv) * length(water.normals.yz); }
  // A river's fronts turn with its current down its course (W5); a flow map's keep the flow's one direction.
  let d = select(normalize(water.normals.yz), normalize(f), water.waterline.w > 0.5 && dot(f, f) > 1e-8);
  let period = 3.0;
  let ph0 = fract(t / period);
  let ph1 = fract(t / period + 0.5);
  let fast = length(f) / max(length(water.normals.yz), 1e-6);
  let river = water.waterline.w > 0.5;
  let s0 = stream_slope(p - f * ph0 * period, d, t, fast, w, river);
  let s1 = stream_slope(p - f * ph1 * period, d, t, fast, w, river);
  let k = abs(ph0 - 0.5) * 2.0;
  return Detail(mix(s0.slope, s1.slope, k), mix(s0.cov, s1.cov, k));
}

fn value_hash(p: vec2<f32>) -> f32 {
  return lattice_hash(vec3<i32>(vec2<i32>(p), 3));
}

// A stream's bank foam at point q (its drift taken out): patches cut at their middle, so half of a bank has none (a
// lower cut left a faint foam all along it: a bright line).
fn bank_patches(q: vec2<f32>) -> f32 {
  return smoothstep(0.55, 0.75, foam_noise(q * 0.9) * 0.7 + foam_noise(q * 3.3) * 0.3);
}

// A cheap smooth noise for breaking up the foam.
fn foam_noise(p: vec2<f32>) -> f32 {
  let i = floor(p);
  let f = fract(p);
  let u = f * f * (3.0 - 2.0 * f);
  return mix(mix(value_hash(i), value_hash(i + vec2<f32>(1.0, 0.0)), u.x),
             mix(value_hash(i + vec2<f32>(0.0, 1.0)), value_hash(i + vec2<f32>(1.0, 1.0)), u.x), u.y);
}
