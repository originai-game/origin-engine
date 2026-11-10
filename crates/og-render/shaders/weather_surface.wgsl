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

// How water lies on a surface (weather_surface_at's `standing`): it runs
// off (a grass blade: no puddles, no splashes); it stands where the
// surface is level and flat over a puddle's width, judged from the
// change of its normal across the pixel (a mesh: none on a ball's crown
// or a rock's top); or it stands on the ground wherever it is level (a
// terrain: its field is flat at a puddle's width, and its normal, read
// from the map level each pixel's footprint picks, steps where that level
// changes, which read as a crease and left dry pin-points in the water).
const WEATHER_RUNS_OFF: f32 = 0.0;
const WEATHER_STANDS: f32 = 1.0;
const WEATHER_GROUND: f32 = 2.0;

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
    let centre = c + vec2<f32>(lattice_hash(vec3<i32>(ci, 21 + k)), lattice_hash(vec3<i32>(ci, 31 + k))) * 0.6 + 0.2;
    let phase = fract(t + lattice_hash(vec3<i32>(ci, 41 + k)));
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
  let centre = 0.25 + 0.5 * lattice_hash(vec3<i32>(c, 0, 83));
  let half = 0.08 + 0.12 * lattice_hash(vec3<i32>(c, 0, 89));
  let blur = max(half, 0.75 * footprint / cell);
  // A dark core and, round it, the stone the water soaks a little wider.
  let core = (1.0 - smoothstep(0.0, blur, abs(f - centre))) * min(half / blur, 1.0);
  let profile = max(core, 0.5 * (1.0 - smoothstep(0.0, 0.5, abs(f - centre))));
  let present = step(0.7, lattice_hash(vec3<i32>(c, 0, 101)));
  // Its runs: one start every 3 to 8 m down the wall, each fading over
  // 1.5 to 4.5 m below its start (`d`, metres under it).
  let period = 3.0 + 5.0 * lattice_hash(vec3<i32>(c, 0, 97));
  let d = fract(-p.y / period + lattice_hash(vec3<i32>(c, 0, 103))) * period;
  let reach = 1.5 + 3.0 * lattice_hash(vec3<i32>(c, 0, 107));
  let run = smoothstep(0.0, 0.05, d) * exp(-d / reach);
  // Rivulets bunch where the wall sheds most (a broad band noise), and
  // the water is uneven along each one, sliding down in the rain.
  let band = mix(0.55, 1.0, smoothstep(0.3, 0.6, weather_noise(vec2<f32>(across * 0.7, 17.0))));
  let along = 0.7 + 0.3 * weather_noise(vec2<f32>(across * 20.0 + 41.0, p.y * 3.0 + t * 0.9));
  // At most 0.7: a rivulet is a darker, damper run, not ink.
  return min(profile * present * run * band * along * 1.2, 0.7);
}

// Drops hitting a wet surface: each throws a tiny crown that spreads as
// a ring through the film and dies within a third of a second, one per
// cell at its own time. The normal's horizontal tilt. Rings a pixel
// cannot resolve (`footprint`, metres a pixel covers) fade out rather
// than sparkle, and a ring is never a bump: a bump at a cell's centre
// read as a puncture in a wet stone seen close up.
fn weather_splashes(p: vec2<f32>, t: f32, footprint: f32) -> vec2<f32> {
  let fade = 1.0 - smoothstep(0.002, 0.008, footprint);
  if (fade <= 0.0) { return vec2<f32>(0.0); }
  let q = p * 9.0;
  let c = floor(q);
  let ci = vec2<i32>(c);
  let centre = c + vec2<f32>(lattice_hash(vec3<i32>(ci, 51)), lattice_hash(vec3<i32>(ci, 61))) * 0.7 + 0.15;
  let phase = fract(t * 3.0 + lattice_hash(vec3<i32>(ci, 71)));
  let to = q - centre;
  let r = length(to);
  let ring = r - (0.03 + 0.2 * phase);
  let crest = exp(-ring * ring * 900.0) * (1.0 - phase) * (1.0 - phase);
  return to / max(r, 1e-3) * crest * 0.15 * fade;
}

// Whether a pipeline has the weather's code (pipeline_features.rs).
override OG_WEATHER: bool = true;

// The weather on one surface: `geo` its geometric normal, `p_frame` its
// point (frame coordinates), `cavity` how high it lies in its own relief
// (0 a hollow, 1 a crest, 0.5 none known: a terrain's blended layer
// height), `amount` how much of the weather reaches it (its material's
// weathering), `standing` how water lies on it (WEATHER_RUNS_OFF,
// WEATHER_STANDS, WEATHER_GROUND).
fn weather_surface_at(s_in: Surface, n_in: vec3<f32>, geo: vec3<f32>, p_frame: vec3<f32>, cavity: f32, amount: f32, standing: f32) -> Weathered {
  var o: Weathered;
  o.s = s_in;
  o.n = n_in;
  if (!OG_WEATHER) { return o; }
  var w = frame.weather * amount;
  var dust = frame.weather_dust.w * amount;
  // A sea's surf (W1, shore_wetness) wets its shore whatever the sky does.
  let surf = frame.shore_wet.z > 0.0;
  if (max(max(max(w.x, w.y), w.z), dust) <= 0.0 && !surf) { return o; }
  // How sharply the surface curves, per metre (its normal's change over
  // its position's across the pixel): standing water lies only on what is
  // flat over a puddle's width, not on a rock's crown or a ball's top.
  // (Taken here, in uniform control flow, before the masks branch.)
  let footprint = length(fwidthFine(p_frame));
  let bend = select(length(fwidthFine(geo)) / max(footprint, 1e-5), 0.0, standing > 1.5);
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
  // wind leaves the crests bare. Lying sand covers the tops whole and
  // thins out down steeper faces; a fine film reaches every face it can
  // and dulls even polished metal (the grains scatter what the metal
  // would mirror: rougher, its f0 dropping towards the sand's).
  if (dust > 0.0) {
    let lying = smoothstep(0.42, 0.88, up + (cover - 0.5) * 0.5 + low * 0.6) * min(dust * 1.6, 1.0);
    let film = dust * (0.45 + 0.4 * up) * mix(0.7, 1.3, cover) * (1.0 - 0.6 * max(-geo.y, 0.0));
    let d = max(lying, film);
    // The bare metal between the grains is scattered too: fully rough
    // where a film lies, its share (metallic, f0) the grains' cover.
    let dull = max(d, min(film * 3.0, 1.0));
    o.s.albedo = mix(o.s.albedo, frame.weather_dust.rgb * (0.9 + 0.2 * cover), d);
    o.s.metallic = mix(o.s.metallic, 0.0, d);
    o.s.roughness = mix(o.s.roughness, 0.95, dull);
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
    let streak = weather_streaks(p, geo, frame.time.x * select(0.0, 1.0, raining), footprint) * wall * w.x;
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
    puddle = smoothstep(edge, edge + 0.1, hollow) * level * flat_enough * (1.0 - snow) * min(standing, 1.0);
    shore = smoothstep(edge - 0.12, edge, hollow) * level * flat_enough * (1.0 - snow) * min(standing, 1.0);
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
  // The film drains off what stands proud of the relief (a sett's crown
  // keeps only its pores' gloss, its low edges stay sheeted) and off
  // whatever the shading normal tilts away from the surface (a stone's
  // rounded edge, a crack's wall). Where the film is gone the stone dries
  // and wets in fine patches: its pores fill unevenly (the drops' last
  // landings, the stone's own grain), so a drained crown is mottled, dark
  // where it is soaked and paler where it is only damp, under a grainy,
  // broken highlight, not a varnish.
  // (With no relief known, cavity 0.5, the film covers the level ground.)
  let relief_film = clamp(1.0 - (cavity - 0.5) * 3.0, 0.0, 1.0);
  let lying = smoothstep(0.85, 0.97, dot(o.n, geo));
  let drained = 1.0 - relief_film * lying;
  // (Each scale fades to its mean where a pixel covers half its 4 or 1.5 cm
  // patches: unfiltered, the far ground took them in aliased blotches.)
  let m1 = mix(0.5, weather_noise(p.xz * 23.0 + 5.0), 1.0 - smoothstep(0.01, 0.022, footprint));
  let m2 = mix(0.5, weather_noise(p.xz * 67.0 + 9.0), 1.0 - smoothstep(0.004, 0.008, footprint));
  let mottle = smoothstep(0.3, 0.7, m1 * 0.6 + m2 * 0.4);
  let soaked = wet * mix(1.0, mix(0.3, 1.0, mottle), drained * (1.0 - puddle));
  o.s.albedo *= mix(1.0, 0.3, soaked * porosity);
  o.s.roughness = mix(o.s.roughness, max(o.s.roughness * 0.5, 0.1), wet * wet * (1.0 - porosity));
  o.s.roughness = mix(o.s.roughness, max(o.s.roughness * 0.75, 0.4), wet * wet * porosity);
  // In the rain the pores are full and a film of water runs over the
  // level ground (Lagarde's flooded phase): it is nearly as smooth as the
  // water, so each lamp stretches into a long streak down the street
  // (GGX elongates a point light's reflection at grazing views). The film
  // drains when the rain stops, leaving the pores' modest gloss.
  if (raining) {
    let sheet = wet * smoothstep(0.75, 0.95, up) * min(w.w * 1.5, 1.0) * relief_film * lying;
    o.s.roughness = mix(o.s.roughness, min(o.s.roughness, 0.2), sheet * sheet);
  }
  let exposed = wet * porosity * drained;
  if (exposed > 0.0) {
    let grain = weather_noise(p.xz * 31.0 + 3.0) * 0.6 + weather_noise(p.xz * 7.0 + 11.0) * 0.4;
    o.s.roughness = mix(o.s.roughness, mix(0.32, 0.7, grain), exposed);
  }

  // Drops splash on the wet level ground outside the puddles.
  if (raining && wet > 0.0 && standing > 0.0) {
    let tilt = weather_splashes(p.xz, frame.time.x, footprint) * w.w * wet * up * (1.0 - puddle) * min(standing, 1.0);
    o.n = normalize(o.n + vec3<f32>(tilt.x, 0.0, tilt.y));
  }

  // Standing water: mirror-smooth dark water over the surface, rippled by
  // the rain.
  if (puddle > 0.0) {
    o.s.albedo *= mix(1.0, 0.25, puddle);
    o.s.f0 = mix(o.s.f0, vec3<f32>(0.02), puddle);
    o.s.metallic = mix(o.s.metallic, 0.0, puddle);
    // The water's own mirror and flat face take over only where it covers
    // the surface: across its shore the surface is partly drowned, and a
    // half-smooth, half-tilted normal there drew a bright seam along the
    // foot of each stone (the steep face sharply mirroring the ground).
    let covered = smoothstep(0.5, 0.9, puddle);
    o.s.roughness = mix(o.s.roughness, 0.03, covered);
    var flat_n = geo;
    if (raining) {
      let tilt = weather_ripples(p.xz, frame.time.x, footprint) * w.w;
      flat_n = normalize(geo + vec3<f32>(tilt.x, 0.0, tilt.y));
    }
    o.n = normalize(mix(o.n, flat_n, covered));
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
  return weather_surface_at(s_in, n_in, geo, p_frame, 0.5, 1.0, WEATHER_STANDS);
}

// What the depth-normal prepass writes of the weather: the gloss, metalness
// and normal the lit pass shades with (the same masks; the colour is not
// needed, but the relief is the lit pass's), so screen-space reflections
// see wet ground and puddles as mirrors where the lit pass draws them.
fn weather_gloss(roughness: f32, metalness: f32, n: vec3<f32>, geo: vec3<f32>, p_frame: vec3<f32>, cavity: f32, amount: f32, standing: f32) -> Weathered {
  var s: Surface;
  s.albedo = vec3<f32>(1.0);
  s.metallic = metalness;
  s.roughness = roughness;
  s.f0 = mix(vec3<f32>(0.04), s.albedo, metalness);
  return weather_surface_at(s, n, geo, p_frame, cavity, amount, standing);
}

// The weather on a grass blade (grass.wgsl): wet blades darken and gloss,
// snow lies on the blades' upper sides and dust dulls them; no water
// stands on them. A blade's normal is near level, so the cover is placed
// by a normal leaning up. `t` is 0 at the blade's root, 1 at its tip: the
// snow lying on the ground buries the lower part of every blade (deeper
// cover, higher up), so a snowed meadow is a white blanket with the tips
// standing out of it, not frosted grass.
fn weather_grass(s_in: Surface, n_in: vec3<f32>, p_frame: vec3<f32>, t: f32) -> Weathered {
  if (!OG_WEATHER) { return Weathered(s_in, n_in); }
  let geo = normalize(n_in * 0.35 + vec3<f32>(0.0, 1.0, 0.0));
  var o = weather_surface_at(s_in, n_in, geo, p_frame, 0.5, 1.0, WEATHER_RUNS_OFF);
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
