// What of the weather reaches more than the lit surfaces
// (weather_surface.wgsl, after this): its noise, where a point lies open
// to the sky under the weather occlusion map, and the rain's rings on
// standing water, a puddle's and a lake's alike (water.wgsl). After
// lighting.wgsl, which declares the cascade array.

// Whether a pipeline has the weather's code (pipeline_features.rs).
override OG_WEATHER: bool = true;

// Smooth value noise over the plane, 0..1.
fn weather_noise(p: vec2<f32>) -> f32 {
  let c = floor(p);
  let f = p - c;
  let u = f * f * (3.0 - 2.0 * f);
  let ci = vec2<i32>(c);
  let a = lattice3(vec3<i32>(ci, 11), 0u);
  let b = lattice3(vec3<i32>(ci + vec2<i32>(1, 0), 11), 0u);
  let d = lattice3(vec3<i32>(ci + vec2<i32>(0, 1), 11), 0u);
  let e = lattice3(vec3<i32>(ci + vec2<i32>(1, 1), 11), 0u);
  return mix(mix(a, b, u.x), mix(d, e, u.x), u.y);
}

fn weather_fbm(p: vec2<f32>) -> f32 {
  return weather_noise(p) * 0.57 + weather_noise(p * 2.03 + 17.1) * 0.29 + weather_noise(p * 4.07 + 3.3) * 0.14;
}

// Raindrop rings on standing water: a drop per cell, each at its own time
// in a one-second cycle, its ring widening and fading. The normal's
// horizontal tilt.
fn weather_ripples(p: vec2<f32>, t: f32, footprint: f32) -> vec2<f32> {
  // A ring's crests are about 4 cm apart: where a pixel covers a
  // centimetre they alias into stripes, so they fade out first.
  let fade = 1.0 - smoothstep(0.004, 0.012, footprint);
  if (fade <= 0.0) { return vec2<f32>(0.0); }
  var tilt = vec2<f32>(0.0);
  for (var k = 0; k < 2; k++) {
    let q = p * 3.0 + vec2<f32>(f32(k) * 0.37, f32(k) * 0.71);
    let c = floor(q);
    let ci = vec2<i32>(c);
    let centre = c + vec2<f32>(lattice3(vec3<i32>(ci, 21 + k), 0u), lattice3(vec3<i32>(ci, 31 + k), 0u)) * 0.6 + 0.2;
    let phase = fract(t + lattice3(vec3<i32>(ci, 41 + k), 0u));
    let to = q - centre;
    let r = length(to);
    let ring = r - phase * 0.45;
    let wave = sin(ring * 50.0) * exp(-ring * ring * 300.0) * (1.0 - phase);
    tilt += to / max(r, 1e-3) * wave;
  }
  return tilt * 0.35 * fade;
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
  let helper = select(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), abs(geo.y) > 0.9);
  let t1 = normalize(cross(geo, helper));
  let t2 = cross(geo, t1);
  let nc = cross((frame.weather_view_proj * vec4<f32>(t1, 0.0)).xyz, (frame.weather_view_proj * vec4<f32>(t2, 0.0)).xyz);
  let per_texel = 2.0 / ws.z;
  var gradient = vec2<f32>(0.0);
  if (abs(nc.z) > 1e-8) {
    gradient = vec2<f32>(-nc.x, nc.y) / nc.z * per_texel;
  }
  var open = 0.0;
  for (var j = -2; j <= 2; j++) {
    for (var i = -2; i <= 2; i++) {
      let tap = at + vec2<f32>(f32(i), f32(j)) * step;
      let t = clamp(vec2<i32>(tap), vec2<i32>(0), vec2<i32>(side - 1));
      let nearest = textureLoad(cascade_map, t, layer, 0).r;
      // (At most 8 m along the fall either way: a wall's plane is no guide.)
      let plane = c.z + clamp(dot(gradient, tap - uv * ws.z), -0.01, 0.01);
      open += select(0.0, 1.0, plane - ws.w <= nearest);
    }
  }
  return smoothstep(0.1, 0.9, open / 25.0);
}
