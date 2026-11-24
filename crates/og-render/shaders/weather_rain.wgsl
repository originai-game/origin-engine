// What of the weather reaches more than the lit surfaces
// (weather_surface.wgsl, after this): its noise, where a point lies open
// to the sky under the weather occlusion map, and the rain's rings on
// standing water, a puddle's and a lake's alike (water.wgsl). After
// lighting.wgsl, which declares the cascade array.

// Whether a pipeline has the weather's code (pipeline_features.rs).
override OG_WEATHER: bool = true;

// The weather's value noise over the plane, 0..1 (its own lattice layer).
fn weather_noise(p: vec2<f32>) -> f32 {
  return value_noise2(p, 11);
}

fn weather_fbm(p: vec2<f32>) -> f32 {
  return weather_noise(p) * 0.57 + weather_noise(p * 2.03 + 17.1) * 0.29 + weather_noise(p * 4.07 + 3.3) * 0.14;
}

// Rain running down a wall: thin rivulets along the fall, each starting
// at its own height (a ledge, a joint, a flaw) where it is darkest, just
// under a soft top edge, and fading out over a metre or three below it,
// a little uneven along its length. They flow while it rains and stay as
// dark stains once it stops. 0..1. `geo` faces sideways; `t` the seconds
// while it rains; `footprint` the metres a pixel covers there (a rivulet
// thinner than a pixel widens and dims, so a far wall does not shimmer).
fn weather_streaks(p: vec3<f32>, geo: vec3<f32>, t: f32, footprint: f32) -> f32 {
  // Across the wall: the horizontal direction in its plane.
  let side = normalize(vec2<f32>(-geo.z, geo.x) + vec2<f32>(1e-5, 0.0));
  let across = dot(p.xz, side);
  // Rivulets meander a little as they run down.
  let u = across + (weather_noise(vec2<f32>(across * 1.3, p.y * 0.9)) - 0.5) * 0.03;
  // One possible rivulet per 8 cm strip, at its own place in the strip,
  // 1.3 to 3.2 cm wide; three strips in ten run (seven in ten combed a
  // wall into dark stripes: a weathered wall sheds in a few runs).
  let cell = 0.08;
  let c = i32(floor(u / cell));
  let f = u / cell - f32(c);
  let centre = 0.25 + 0.5 * lattice3(vec3<i32>(c, 0, 83), 0u);
  let half = 0.08 + 0.12 * lattice3(vec3<i32>(c, 0, 89), 0u);
  let blur = max(half, 0.75 * footprint / cell);
  // A dark core and, round it, the stone the water soaks a little wider.
  let core = (1.0 - smoothstep(0.0, blur, abs(f - centre))) * min(half / blur, 1.0);
  let profile = max(core, 0.5 * (1.0 - smoothstep(0.0, 0.5, abs(f - centre))));
  let present = step(0.7, lattice3(vec3<i32>(c, 0, 101), 0u));
  // Its runs: one start every 3 to 8 m down the wall, each fading over
  // 1.5 to 4.5 m below its start (`d`, metres under it).
  let period = 3.0 + 5.0 * lattice3(vec3<i32>(c, 0, 97), 0u);
  let d = fract(-p.y / period + lattice3(vec3<i32>(c, 0, 103), 0u)) * period;
  let reach = 1.5 + 3.0 * lattice3(vec3<i32>(c, 0, 107), 0u);
  let run = smoothstep(0.0, 0.05, d) * exp(-d / reach);
  // Rivulets bunch where the wall sheds most (a broad band noise), and
  // the water is uneven along each one, sliding down in the rain.
  let band = mix(0.55, 1.0, smoothstep(0.3, 0.6, weather_noise(vec2<f32>(across * 0.7, 17.0))));
  let along = 0.7 + 0.3 * weather_noise(vec2<f32>(across * 20.0 + 41.0, p.y * 3.0 + t * 0.9));
  // At most 0.7: a rivulet is a darker, damper run, not ink.
  return min(profile * present * run * band * along * 1.2, 0.7);
}

// A raindrop's ring on standing water: the capillary wave packet its
// impact sends out (Lagarde's "Water drop" series, 2012-13, draws its
// rings), a few crests 2.5 cm apart under an envelope that broadens as the
// packet disperses, travelling at some 20 cm/s, steepest (about 0.3) near
// the impact and weakening as the ring spreads, gone 8 to 10 cm out.
const RIPPLE_CELL: f32 = 0.3;
const RIPPLE_PERIOD: f32 = 0.5;
const RIPPLE_SPEED: f32 = 0.2;
const RIPPLE_WAVE: f32 = 0.025;
const RIPPLE_SLOPE: f32 = 1.0;

// How much of the rings a pixel that covers `footprint` metres holds: all
// of them where it covers a sixth of their crests' spacing, none where it
// covers half of it (they alias into stripes there).
fn weather_ripples_held(footprint: f32) -> f32 {
  return 1.0 - smoothstep(0.004, 0.012, footprint);
}

// The mean square of weather_ripples' tilt along one axis at the rain's
// full strength, over a cell and a cycle (integrated numerically from the
// function below, all three layers): what the rings a pixel cannot hold
// add to a lake's slope variance (water.wgsl).
const WEATHER_RIPPLES_SPREAD: f32 = 8.9e-4;

// Raindrop rings on standing water: in each of three offset grids of 30 cm
// cells a drop falls every half second, at its own time and each time at a
// new place (67 drops a square metre a second), and its ring dies before it
// reaches the cell's edge. The normal's horizontal tilt, the height's
// slope. (A drop a second in each of two grids of 33 cm cells, each ring
// living a whole second and growing to 15 cm, drew a few sharp ellipses a
// hand across where a camera near the water caught a lamp in them.)
fn weather_ripples(p: vec2<f32>, t: f32, footprint: f32) -> vec2<f32> {
  let fade = weather_ripples_held(footprint);
  if (fade <= 0.0) { return vec2<f32>(0.0); }
  var tilt = vec2<f32>(0.0);
  for (var k = 0; k < 3; k++) {
    let q = p / RIPPLE_CELL + vec2<f32>(f32(k) / 3.0, f32(k) * 0.61);
    let c = floor(q);
    let ci = vec2<i32>(c);
    let cycle = t / RIPPLE_PERIOD + lattice3(vec3<i32>(ci, 41 + k), 0u);
    let n = i32(floor(cycle));
    let age = fract(cycle) * RIPPLE_PERIOD;
    let drop = vec3<i32>(ci, n * 3 + k);
    let centre = c + 0.4 + 0.2 * vec2<f32>(lattice3(drop, 21u), lattice3(drop, 31u));
    let to = (q - centre) * RIPPLE_CELL;
    let r = length(to);
    let front = 0.004 + RIPPLE_SPEED * age;
    let width = 0.004 + 0.02 * age;
    // Spreading thins it; it is gone before its envelope reaches the cell's
    // edge, 9 cm out at the nearest.
    let strength = RIPPLE_SLOPE * 0.015 / (0.015 + front) * (1.0 - smoothstep(0.07, 0.1, front));
    let x = r - front;
    let slope = strength * exp(-(x * x) / (width * width)) * sin(x * (2.0 * PI / RIPPLE_WAVE));
    // (No direction at the drop's own point: never a puncture there.)
    tilt += to / max(r, 1e-4) * slope * smoothstep(0.0, 0.003, r);
  }
  return tilt * fade;
}

// How open `p` (frame coordinates) is to the rain and snow, 0 under a roof
// to 1 under the sky: the weather occlusion map, the last layer of the
// cascade array (its index from frame.weather_shelter, never a constant),
// holds the depth along the fall of what is nearest the sky; a 5x5 filter
// softens the drip line. Each tap is compared at the depth the receiver's
// own plane (its geometric normal `geo`) has there, so a slope or a
// dome's curve never shelters itself (compared at its centre's depth, the
// uphill taps of a surface steeper than a few degrees read as a roof and
// kept the snow off rounded tops). Outside the map, or without one, all
// is open.
fn weather_open(p: vec3<f32>, geo: vec3<f32>) -> f32 {
  let ws = frame.weather_shelter;
  if (ws.x < 0.5) { return 1.0; }
  let c = frame.weather_view_proj * vec4<f32>(p, 1.0);
  let uv = vec2<f32>(c.x * 0.5 + 0.5, 0.5 - c.y * 0.5);
  if (any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0)) || c.z < 0.0 || c.z > 1.0) { return 1.0; }
  let layer = i32(ws.x - 0.5);
  let side = i32(ws.z);
  // The drip line wanders: the lookup is pushed about by noise of the
  // world position, up to the soft edge's width, so the snow-free patch
  // has a ragged rim rather than the roof's straight outline.
  let pw = p + frame.origin.xyz;
  let wander = vec2<f32>(weather_noise(pw.xz * 1.7 + 3.0), weather_noise(pw.xz * 1.7 + 29.0)) - 0.5;
  let at = uv * ws.z + wander * ws.y * 1.6;
  let step = max(ws.y * 0.5, 1.0);
  // The receiver's plane in the map's clip space: two tangents through the
  // view's linear part span it; its depth per texel follows from their
  // normal (uv runs x with clip x, y against clip y, 2 / side per texel).
  let span = orthonormal_basis(geo);
  let t1 = span[0];
  let t2 = span[1];
  let nc = cross((frame.weather_view_proj * vec4<f32>(t1, 0.0)).xyz, (frame.weather_view_proj * vec4<f32>(t2, 0.0)).xyz);
  let per_texel = 2.0 / ws.z;
  var gradient = vec2<f32>(0.0);
  if (abs(nc.z) > 1e-8) {
    gradient = vec2<f32>(-nc.x, nc.y) / nc.z * per_texel;
  }
  // Centre tap + 8-tap IGN-rotated Vogel disc over the same 2*step radius
  // (9 loads instead of 25, with smoother drip-line edges and no 5x5 grid aliasing).
  let radius = 2.0 * step;
  let base_px = clamp(vec2<i32>(at), vec2<i32>(0), vec2<i32>(side - 1));
  let base_plane = c.z + clamp(dot(gradient, at - uv * ws.z), -0.01, 0.01);
  var open = select(0.0, 1.0, base_plane - ws.w <= textureLoad(cascade_map, base_px, layer).r);
  let turn = ign(pw.xz * 19.3 + vec2<f32>(7.0, 13.0)) * 2.0 * PI;
  for (var k = 0u; k < 8u; k++) {
    let r = sqrt((f32(k) + 0.5) / 8.0) * radius;
    let a = f32(k) * 2.3999632 + turn;
    let tap = at + vec2<f32>(cos(a), sin(a)) * r;
    let t = clamp(vec2<i32>(tap), vec2<i32>(0), vec2<i32>(side - 1));
    let nearest = textureLoad(cascade_map, t, layer).r;
    // (At most 8 m along the fall either way: a wall's plane is no guide.)
    let plane = c.z + clamp(dot(gradient, tap - uv * ws.z), -0.01, 0.01);
    open += select(0.0, 1.0, plane - ws.w <= nearest);
  }
  return smoothstep(0.1, 0.9, open / 9.0);
}
