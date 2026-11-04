// Weather on lit surfaces (weather.rs `SurfaceWeather`): wet, streaked,
// puddled, snowed-on and dusty surfaces from the frame uniform's weather
// rows and the weather occlusion map, a layer of the cascade array (no
// binding of its own: the lit stages' storage buffers and textures are
// spoken for). After lighting.wgsl, which declares `Surface`.
//   frame.weather:       x wetness, y puddles, z snow cover, w rain (ripples)
//   frame.weather_look:  rgb snow colour, w puddle size (m)
//   frame.weather_dust:  rgb dust colour, w dust cover
//   frame.weather_depth: x lying snow (m)
// Every mask is placed by the world position (frame.origin added) and the
// geometric normal, so a normal map shades the cover without moving it.
// The model follows Lagarde's "Water drop" series (2012-13: porosity
// darkens, a sealed surface glosses, standing water is a flat dielectric
// film, rain drops ring it) and Lagarde and Harduin's "The art and
// rendering of Remember Me" (GDC 2013: wet cavities first, drying from the
// tops, streaks running down walls).

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

// How wet a sea's surf leaves `p` (frame coordinates; W1, water.rs shore_wet): everything over its seabed up to the
// band its surf runs up, the band's top ragged, drying above it.
fn shore_wetness(p: vec3<f32>) -> f32 {
  let s = frame.shore_wet;
  if (s.z <= 0.0) { return 0.0; }
  let a = frame.shore_area;
  if (any(p.xz < a.xy) || any(p.xz > a.zw)) { return 0.0; }
  let q = p + frame.origin.xyz;
  let ragged = (weather_fbm(q.xz * 0.35) - 0.5) * 0.6 * s.y;
  return s.z * (1.0 - smoothstep(0.55 * s.y, s.y, p.y - s.x + ragged));
}

// Rain running down a wall: thin rivulets along the fall, broken up along
// their length, that flow while it rains and stay as dark stains once it
// stops. 0..1. `geo` faces sideways; `t` the seconds while it rains.
fn weather_streaks(p: vec3<f32>, geo: vec3<f32>, t: f32) -> f32 {
  // Across the wall: the horizontal direction in its plane.
  let side = normalize(vec2<f32>(-geo.z, geo.x) + vec2<f32>(1e-5, 0.0));
  // Rivulets meander a little as they run down.
  let u = dot(p.xz, side) + (weather_noise(vec2<f32>(dot(p.xz, side) * 1.3, p.y * 0.9)) - 0.5) * 0.08;
  // Rivulets a few centimetres wide, bunched where the wall sheds most
  // (a broad band noise).
  let lanes = weather_noise(vec2<f32>(u * 7.0, 3.1)) * 0.6 + weather_noise(vec2<f32>(u * 29.0, 9.7)) * 0.4;
  let band = smoothstep(0.25, 0.6, weather_noise(vec2<f32>(u * 0.7, 17.0)));
  // Each strip of wall a hand wide sheds its own runs: a run starts at a
  // ledge or a flaw at its own height, is darkest there and thins out
  // below it, and a gap follows before the next one (a third of the
  // strips stay dry), so no rivulet spans the wall like a curtain.
  let col = i32(floor(u * 4.0));
  let period = 1.5 + 3.0 * lattice_hash(vec3<i32>(col, 0, 83));
  let y = fract(p.y / period + lattice_hash(vec3<i32>(col, 0, 89)));
  let reach = 0.35 + 0.55 * lattice_hash(vec3<i32>(col, 0, 97));
  let run = smoothstep(1.0 - reach, 1.0 - reach * 0.4, y) * (1.0 - smoothstep(0.88, 1.0, y));
  let present = step(0.33, lattice_hash(vec3<i32>(col, 0, 101)));
  // The water itself is broken along its length, sliding down in the rain.
  let along = weather_noise(vec2<f32>(u * 7.0 + 41.0, p.y * 0.35 + t * 0.9));
  return smoothstep(0.45, 0.58, lanes) * band * run * present * smoothstep(0.1, 0.4, along);
}

// Drops hitting a wet surface: a crown of tiny splashes, one per cell at
// its own time in a third of a second, as the normal's horizontal tilt.
fn weather_splashes(p: vec2<f32>, t: f32) -> vec2<f32> {
  let q = p * 9.0;
  let c = floor(q);
  let ci = vec2<i32>(c);
  let centre = c + vec2<f32>(lattice_hash(vec3<i32>(ci, 51)), lattice_hash(vec3<i32>(ci, 61))) * 0.7 + 0.15;
  let phase = fract(t * 3.0 + lattice_hash(vec3<i32>(ci, 71)));
  let to = q - centre;
  let r = length(to);
  let bump = exp(-r * r * 60.0) * (1.0 - phase) * (1.0 - phase);
  return to / max(r, 1e-3) * bump;
}

// The weather on one surface: `geo` its geometric normal, `p_frame` its
// point (frame coordinates), `cavity` how high it lies in its own relief
// (0 a hollow, 1 a crest, 0.5 none known: a terrain's blended layer
// height), `amount` how much of the weather reaches it (its material's
// weathering), `standing` 1 where water can stand and drops splash (a
// solid surface), 0 where it runs off (a grass blade).
fn weather_surface_at(s_in: Surface, n_in: vec3<f32>, geo: vec3<f32>, p_frame: vec3<f32>, cavity: f32, amount: f32, standing: f32) -> Weathered {
  var o: Weathered;
  o.s = s_in;
  o.n = n_in;
  var w = frame.weather * amount;
  var dust = frame.weather_dust.w * amount;
  // A sea's surf (W1, shore_wetness) wets its shore whatever the sky does.
  let surf = frame.shore_wet.z > 0.0;
  if (max(max(max(w.x, w.y), w.z), dust) <= 0.0 && !surf) { return o; }
  // How sharply the surface curves, per metre (its normal's change over
  // its position's across the pixel): standing water lies only on what is
  // flat over a puddle's width, not on a rock's crown or a ball's top.
  // (Taken here, in uniform control flow, before the masks branch.)
  let bend = length(fwidth(geo)) / max(length(fwidth(p_frame)), 1e-5);
  let surf_wet = shore_wetness(p_frame) * amount;
  if (max(max(max(w.x, w.y), w.z), max(dust, surf_wet)) <= 0.0) { return o; }
  // Under a roof: dry, no puddles, no snow (the drip line softened). Dust
  // is blown in under it, thinner. The surf's wet reaches under one.
  var open = 1.0;
  if (max(max(max(w.x, w.y), w.z), dust) > 0.0) { open = weather_open(p_frame, geo); }
  w *= open;
  dust *= mix(0.45, 1.0, open);
  w.x = max(w.x, surf_wet);
  let p = p_frame + frame.origin.xyz;
  let up = clamp(geo.y, 0.0, 1.0);
  let raining = w.w > 0.0;
  // The cover's noise: broad drifts, a finer break-up for a ragged edge,
  // and the height mixed in so a curved face's edge wanders too.
  let cover = weather_fbm(p.xz * 0.6 + p.y * 0.7) * 0.6 + weather_noise(p.xz * 7.1 + p.y * 5.3 + 7.0) * 0.4;
  let low = 0.5 - cavity;  // > 0 in a hollow

  // Dust and sand settle on what faces up, in the low parts first; the
  // wind leaves the crests bare. A dusty surface is matte and dull.
  if (dust > 0.0) {
    // Lying sand on what faces up (reaching steeper faces as it deepens),
    // over a fine film on every face it can reach that dulls even metal.
    let settle = up + (cover - 0.5) * 0.6 + low * 0.8;
    let lying = smoothstep(1.15 - dust, 1.35 - dust, settle) * min(dust * 2.5, 1.0);
    let film = dust * mix(0.25, 0.45, cover) * (1.0 - 0.5 * max(-geo.y, 0.0));
    let d = max(lying, film);
    o.s.albedo = mix(o.s.albedo, frame.weather_dust.rgb * (0.9 + 0.2 * cover), d * 0.92);
    o.s.metallic = mix(o.s.metallic, 0.0, d);
    o.s.roughness = mix(o.s.roughness, 0.97, d);
    o.s.f0 = mix(o.s.f0, vec3<f32>(0.04), d);
    o.n = normalize(mix(o.n, geo, d * 0.6));
  }

  // Snow settles on what faces up, deeper cover reaching steeper faces
  // and the hollows filling first; the noise breaks its edge. Snow keeps
  // the surface under it dry.
  var snow = 0.0;
  if (w.z > 0.0) {
    let reach = mix(1.05, 0.15, w.z);
    snow = smoothstep(reach, reach + 0.04, up + (cover - 0.5) * 0.7 * (1.0 - w.z * 0.5) + low * 0.25);
    snow *= smoothstep(0.0, 0.08, w.z);
  }

  // How wet this point is. In the rain all of it soaks alike; once the
  // rain stops the crests and the open drifts dry first and the hollows
  // stay dark longest (the stored wetness falls through the dampness).
  // (The noise's values bunch about a half: stretched to fill 0..1.)
  let damp_noise = clamp((weather_fbm(p.xz * 0.45 + 13.0) - 0.5) * 2.6 + 0.5, 0.0, 1.0);
  let damp = clamp(0.6 * damp_noise + 0.6 * (0.5 - cavity) + 0.15 * (1.0 - up) + 0.2, 0.0, 1.0);
  let drying = smoothstep(1.0 - w.x - 0.08, 1.0 - w.x + 0.08, damp);
  var wet = select(drying, w.x, raining) * mix(0.55, 1.0, up) * (1.0 - snow);

  // Rain runs down what faces sideways in streaks, flowing while it rains.
  let wall = 1.0 - smoothstep(0.35, 0.75, abs(geo.y));
  if (wall > 0.0 && w.x > 0.0) {
    let streak = weather_streaks(p, geo, frame.time.x * select(0.0, 1.0, raining)) * wall * w.x;
    // Between the rivulets the wall is only damp. A rivulet soaks into
    // porous stone (darker, barely glossier) and runs as a film down a
    // sealed one (paint, metal, glass: a gloss).
    wet = mix(wet * 0.6, 1.0, streak);
    let sealed = 1.0 - (1.0 - o.s.metallic) * smoothstep(0.15, 0.85, o.s.roughness);
    o.s.roughness = mix(o.s.roughness, min(o.s.roughness, 0.12), streak * sealed * select(0.6, 1.0, raining));
  }

  // Puddles: where the noise's hollows and the surface's own lie lowest
  // on level ground; the ground round their shore is soaked.
  var puddle = 0.0;
  var shore = 0.0;
  if (w.y > 0.0 && standing > 0.0) {
    let hollow = 1.0 - weather_fbm(p.xz / max(frame.weather_look.w, 0.05) + 41.0) + low * 0.8;
    let level = smoothstep(0.93, 0.985, up);
    let edge = 1.0 - w.y * 0.65;
    let flat_enough = 1.0 - smoothstep(0.08, 0.35, bend);
    puddle = smoothstep(edge, edge + 0.1, hollow) * level * flat_enough * (1.0 - snow) * standing;
    shore = smoothstep(edge - 0.12, edge, hollow) * level * flat_enough * (1.0 - snow) * standing;
    wet = max(wet, shore * w.x);
  }

  // (The surface's f0 is kept where the water and snow leave it: a
  // physical material's comes from its index of refraction.)
  // Wet: porous (rough, dielectric) surfaces darken as their pores fill
  // and the water left in them glosses them part way (a wet road is dark
  // and catches the lamps, but is no mirror: the standing water's gloss is
  // the puddles'); sealed surfaces (smooth or metal: no pores to fill)
  // gloss under a film instead.
  let porosity = (1.0 - o.s.metallic) * smoothstep(0.15, 0.85, o.s.roughness);
  o.s.albedo *= mix(1.0, 0.3, wet * porosity);
  o.s.roughness = mix(o.s.roughness, max(o.s.roughness * 0.5, 0.1), wet * wet * (1.0 - porosity));
  o.s.roughness = mix(o.s.roughness, max(o.s.roughness * 0.75, 0.4), wet * wet * porosity);

  // Drops splash on the wet level ground outside the puddles.
  if (raining && wet > 0.0 && standing > 0.0) {
    let tilt = weather_splashes(p.xz, frame.time.x) * w.w * wet * up * (1.0 - puddle) * 0.6 * standing;
    o.n = normalize(o.n + vec3<f32>(tilt.x, 0.0, tilt.y));
  }

  // Standing water: mirror-smooth dark water over the surface, rippled by
  // the rain.
  if (puddle > 0.0) {
    o.s.albedo *= mix(1.0, 0.25, puddle);
    o.s.f0 = mix(o.s.f0, vec3<f32>(0.02), puddle);
    o.s.metallic = mix(o.s.metallic, 0.0, puddle);
    o.s.roughness = mix(o.s.roughness, 0.03, puddle);
    var flat_n = geo;
    if (raining) {
      let tilt = weather_ripples(p.xz, frame.time.x) * w.w;
      flat_n = normalize(geo + vec3<f32>(tilt.x, 0.0, tilt.y));
    }
    o.n = normalize(mix(o.n, flat_n, puddle));
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

// The weather on a surface with no relief known and every material's
// share (the terrain's call until its layers pass their height).
fn weather_surface(s_in: Surface, n_in: vec3<f32>, geo: vec3<f32>, p_frame: vec3<f32>) -> Weathered {
  return weather_surface_at(s_in, n_in, geo, p_frame, 0.5, 1.0, 1.0);
}

// What the depth-normal prepass writes of the weather: the gloss, metalness
// and normal the lit pass shades with (the same masks; the colour is not
// needed, but the relief is the lit pass's), so screen-space reflections
// see wet ground and puddles as mirrors where the lit pass draws them.
fn weather_gloss(roughness: f32, metalness: f32, n: vec3<f32>, geo: vec3<f32>, p_frame: vec3<f32>, cavity: f32, amount: f32) -> Weathered {
  var s: Surface;
  s.albedo = vec3<f32>(1.0);
  s.metallic = metalness;
  s.roughness = roughness;
  s.f0 = mix(vec3<f32>(0.04), s.albedo, metalness);
  return weather_surface_at(s, n, geo, p_frame, cavity, amount, 1.0);
}

// The weather on a grass blade (grass.wgsl): wet blades darken and gloss,
// snow lies on the blades' upper sides and dust dulls them; no water
// stands on them. A blade's normal is near level, so the cover is placed
// by a normal leaning up. `t` is 0 at the blade's root, 1 at its tip: the
// snow lying on the ground buries the lower part of every blade (deeper
// cover, higher up), so a snowed meadow is a white blanket with the tips
// standing out of it, not frosted grass.
fn weather_grass(s_in: Surface, n_in: vec3<f32>, p_frame: vec3<f32>, t: f32) -> Weathered {
  let geo = normalize(n_in * 0.35 + vec3<f32>(0.0, 1.0, 0.0));
  var o = weather_surface_at(s_in, n_in, geo, p_frame, 0.5, 1.0, 0.0);
  let cover = frame.weather.z;
  if (cover > 0.0) {
    let pw = p_frame + frame.origin.xyz;
    let line = cover * 0.85 - 0.1 + (weather_noise(pw.xz * 2.3 + 5.0) - 0.5) * 0.3;
    let buried = (1.0 - smoothstep(line - 0.12, line + 0.08, t)) * weather_open(p_frame, vec3<f32>(0.0, 1.0, 0.0));
    o.s.albedo = mix(o.s.albedo, frame.weather_look.rgb * (0.94 + 0.06 * weather_noise(pw.xz * 9.0)), buried);
    o.s.roughness = mix(o.s.roughness, 0.8, buried);
    o.s.f0 = mix(o.s.f0, vec3<f32>(0.04), buried);
  }
  return o;
}

// How much lying snow weighs a blade down at `p_frame`, 0..1 (grass
// droop): the snow's cover where the sky reaches.
fn weather_snow_weight(p_frame: vec3<f32>) -> f32 {
  let w = frame.weather.z;
  if (w <= 0.0) { return 0.0; }
  return smoothstep(0.0, 0.6, w) * weather_open(p_frame, vec3<f32>(0.0, 1.0, 0.0));
}

// How far lying snow lifts a surface facing `geo` at `p_frame`, in metres
// (a terrain's grid, terrain_vertex.wgsl): the snow's depth where it
// covers level, open ground, nothing on steep faces or under a roof. The
// CPU's heights (collision, og.terrain.sample, picking, what stands on the
// ground) do not follow it: centimetres, not metres.
fn weather_snow_lift(p_frame: vec3<f32>, geo: vec3<f32>) -> f32 {
  let w = frame.weather.z;
  if (w <= 0.0 || frame.weather_depth.x <= 0.0) { return 0.0; }
  let level = smoothstep(mix(1.0, 0.3, w), 1.0, geo.y);
  return frame.weather_depth.x * smoothstep(0.0, 0.3, w) * level * weather_open(p_frame, geo);
}
