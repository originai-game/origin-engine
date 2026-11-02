// Weather on lit surfaces (weather.rs `SurfaceWeather`, the R4 round): wet,
// puddled and snowed-on surfaces from the frame uniform's weather rows and
// the weather occlusion map, a layer of the cascade array (no binding of
// its own: the lit stages' storage buffers and textures are spoken for).
// After lighting.wgsl, which declares `Surface`.
//   frame.weather:      x wetness, y puddles, z snow cover, w rain (ripples)
//   frame.weather_look: rgb snow colour, w puddle size (m)
// Every mask is placed by the world position (frame.origin added) and the
// geometric normal, so a normal map shades the cover without moving it.

struct Weathered {
  s: Surface,
  n: vec3<f32>,
};

// Smooth value noise over the plane, 0..1.
fn weather_noise(p: vec2<f32>) -> f32 {
  let c = floor(p);
  let f = p - c;
  let u = f * f * (3.0 - 2.0 * f);
  let ci = vec2<i32>(c);
  let a = lattice_hash(vec3<i32>(ci, 11));
  let b = lattice_hash(vec3<i32>(ci + vec2<i32>(1, 0), 11));
  let d = lattice_hash(vec3<i32>(ci + vec2<i32>(0, 1), 11));
  let e = lattice_hash(vec3<i32>(ci + vec2<i32>(1, 1), 11));
  return mix(mix(a, b, u.x), mix(d, e, u.x), u.y);
}

fn weather_fbm(p: vec2<f32>) -> f32 {
  return weather_noise(p) * 0.57 + weather_noise(p * 2.03 + 17.1) * 0.29 + weather_noise(p * 4.07 + 3.3) * 0.14;
}

// Raindrop rings on standing water: a drop per cell, each at its own time
// in a one-second cycle, its ring widening and fading. The normal's
// horizontal tilt.
fn weather_ripples(p: vec2<f32>, t: f32) -> vec2<f32> {
  var tilt = vec2<f32>(0.0);
  for (var k = 0; k < 2; k++) {
    let q = p * 3.0 + vec2<f32>(f32(k) * 0.37, f32(k) * 0.71);
    let c = floor(q);
    let ci = vec2<i32>(c);
    let centre = c + vec2<f32>(lattice_hash(vec3<i32>(ci, 21 + k)), lattice_hash(vec3<i32>(ci, 31 + k))) * 0.6 + 0.2;
    let phase = fract(t + lattice_hash(vec3<i32>(ci, 41 + k)));
    let to = q - centre;
    let r = length(to);
    let ring = r - phase * 0.45;
    let wave = sin(ring * 50.0) * exp(-ring * ring * 300.0) * (1.0 - phase);
    tilt += to / max(r, 1e-3) * wave;
  }
  return tilt * 0.35;
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

fn weather_surface(s_in: Surface, n_in: vec3<f32>, geo: vec3<f32>, p_frame: vec3<f32>) -> Weathered {
  var o: Weathered;
  o.s = s_in;
  o.n = n_in;
  var w = frame.weather;
  if (max(max(w.x, w.y), w.z) <= 0.0) { return o; }
  // Under a roof: dry, no puddles, no snow (the drip line softened).
  w *= weather_open(p_frame, geo);
  if (max(max(w.x, w.y), w.z) <= 0.0) { return o; }
  let p = p_frame + frame.origin.xyz;
  let up = clamp(geo.y, 0.0, 1.0);
  // The cover's noise: broad drifts, a finer break-up for a ragged edge,
  // and the height mixed in so a curved face's edge wanders too.
  let cover = weather_fbm(p.xz * 0.6 + p.y * 0.7) * 0.6 + weather_noise(p.xz * 7.1 + p.y * 5.3 + 7.0) * 0.4;

  // Snow settles on what faces up, deeper cover reaching steeper faces;
  // the noise breaks its edge. Snow keeps the surface under it dry.
  var snow = 0.0;
  if (w.z > 0.0) {
    let reach = mix(1.05, 0.15, w.z);
    snow = smoothstep(reach, reach + 0.04, up + (cover - 0.5) * 0.7 * (1.0 - w.z * 0.5));
    snow *= smoothstep(0.0, 0.08, w.z);
  }

  // (The surface's f0 is kept where the water and snow leave it: a
// physical material's comes from its index of refraction.)
// Wet: porous (rough, dielectric) surfaces darken as their pores fill
  // and keep their roughness, so their reflection is the dry surface's and
  // wet porous ground is darker than dry at every angle (any film glossier
  // than the dry surface reflected the bright horizon at grazing angles
  // and read brighter than dry ground); the standing water's gloss is the
  // puddles'. Sealed surfaces (smooth or metal: no pores to fill) gain a
  // sheen instead. Faces turned away from the sky get less.
  let wet = w.x * mix(0.55, 1.0, up) * (1.0 - snow);
  let porosity = (1.0 - o.s.metallic) * smoothstep(0.15, 0.85, o.s.roughness);
  o.s.albedo *= mix(1.0, 0.3, wet * porosity);
  o.s.roughness = mix(o.s.roughness, max(o.s.roughness * 0.5, 0.1), wet * wet * (1.0 - porosity));

  // Puddles: in the hollows of the noise on level ground, mirror-smooth
  // dark water over the surface, rippled by the rain.
  if (w.y > 0.0) {
    let hollow = 1.0 - weather_fbm(p.xz / max(frame.weather_look.w, 0.05) + 41.0);
    let level = smoothstep(0.93, 0.985, up);
    let puddle = smoothstep(1.0 - w.y * 0.65, 1.0 - w.y * 0.65 + 0.06, hollow) * level * (1.0 - snow);
    if (puddle > 0.0) {
      o.s.albedo *= mix(1.0, 0.25, puddle);
      o.s.f0 = mix(o.s.f0, vec3<f32>(0.02), puddle);
      o.s.metallic = mix(o.s.metallic, 0.0, puddle);
      o.s.roughness = mix(o.s.roughness, 0.03, puddle);
      var flat_n = geo;
      if (w.w > 0.0) {
        let tilt = weather_ripples(p.xz, frame.time.x) * w.w;
        flat_n = normalize(geo + vec3<f32>(tilt.x, 0.0, tilt.y));
      }
      o.n = normalize(mix(o.n, flat_n, puddle));
    }
  }

  if (snow > 0.0) {
    let sparkle = 0.92 + 0.08 * weather_noise(p.xz * 9.0);
    o.s.albedo = mix(o.s.albedo, frame.weather_look.rgb * sparkle, snow);
    o.s.metallic = mix(o.s.metallic, 0.0, snow);
    o.s.roughness = mix(o.s.roughness, 0.8, snow);
    o.s.f0 = mix(o.s.f0, vec3<f32>(0.04), snow);
    o.n = normalize(mix(o.n, geo, snow * 0.85));
  }
  return o;
}
