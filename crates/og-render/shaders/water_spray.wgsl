// The sea's spray (appended to water.wgsl).

// ── W4: spray off the breaking crests ──
//
// Each droplet is a function of the clock and its slot, with no state (Sea of Thieves' and Atlas' spray are GPU
// particles spawned where the crests break; here the spawn, the flight and the death are closed-form, so the spray is
// a function of the sea's clock as the rest of the sea is, with no simulation to warm up or step): slot i of a
// population lives over and over, each life of 0.9 to 2.4 s starting after a pause of its own; each life a hash picks a
// point on the foam level about the camera (toroidal, so a droplet's point stays in the world as the camera moves) and
// keeps it only where the foam is freshest (a crest breaking now) and the wind carries spray (from some 10 m/s,
// Beaufort 5's "some spray", to a full gale's at 20 m/s; Andreas et al. 1995, "The spray contribution to net
// evaporation from the sea"). The wind tears it off the drawn crest rather than the crest throwing it: it leaves with
// the crest's speed downwind and a small lift, which the wind's shear flattens as it strengthens, and flies under
// gravity and the air's drag in the wind over the sea (a droplet's velocity relaxing to the air's at its response
// rate), in closed form. The air is the sea's wind brought down to the spray's height by the log profile over a sea
// (Charnock's roughness, Garratt's drag coefficient), gusting along the wind, so a gale's spray streams low and
// downwind off the crests in sheets instead of standing over them in plumes.
//
// Two populations: droplets over the fine foam level (64 m, sprites some 0.1 to 0.4 m), spume over the coarse one
// (512 m, sheets some 1 to 4 m torn off the crests in a gale).

const SPRAY_SLOTS: u32 = 4096u;

fn spray_hash(x: u32) -> u32 {
  let s = x * 747796405u + 2891336453u;
  let w = ((s >> ((s >> 28u) + 4u)) ^ s) * 277803737u;
  return (w >> 22u) ^ w;
}

fn spray_rand(h: u32) -> f32 {
  return f32(spray_hash(h) >> 8u) / 16777216.0;
}

// The wind's speed `z` metres over the sea under a 10 m wind of `u10` m/s: the neutral log profile
// u(z) = u* / κ ln(z / z0), with the friction velocity from Garratt's (1977) drag coefficient
// (Cd = (0.75 + 0.067 u10) 10⁻³) and Charnock's roughness (z0 = 0.011 u*² / g): some 0.8 of the 10 m wind at 1.5 m
// in a gale, 0.6 at a decimetre.
fn sea_wind_at(u10: f32, z: f32) -> f32 {
  let ustar = u10 * sqrt((0.75 + 0.067 * u10) * 1e-3);
  let z0 = max(0.011 * ustar * ustar / 9.81, 1e-5);
  return u10 * log(max(z, 2.0 * z0) / z0) / log(10.0 / z0);
}

struct SprayOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world: vec3<f32>,
  // The sprite's corner (-1 to 1) and its opacity.
  @location(1) corner: vec2<f32>,
  @location(2) alpha: f32,
  @location(3) @interpolate(flat) sheet: u32,
};

@vertex
fn vs_spray(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> SprayOut {
  var o: SprayOut;
  o.clip = vec4<f32>(2.0, 2.0, 2.0, 1.0);
  o.world = vec3<f32>(0.0);
  o.corner = vec2<f32>(0.0);
  o.alpha = 0.0;
  o.sheet = ii / SPRAY_SLOTS;
  let wind = water.sea_wind.z;
  let gust = smoothstep(10.0, 20.0, wind) * water.spray.x;
  if (gust <= 0.0 || !is_sea()) { return o; }
  let pop = ii / SPRAY_SLOTS;
  let slot = ii % SPRAY_SLOTS;
  if (f32(slot) >= water.spray.z) { return o; }
  // The slots come in plumes: a crest throws its spray as a stream, not one puff, so the members of a plume share
  // its lives and its point and leave it one after another over the first 60 % of a life, each with its own throw
  // and size (a lone sprite read as a dot hanging over the sea; a stream reads as spindrift off a crest).
  // A spume slot already spans the whole trail: stacking members at its head makes an opaque strip.
  let plume = select(4u, 1u, pop == 1u);
  let member = slot % plume;
  let seed = spray_hash((slot / plume) * 2u + pop + 0x51ed270bu);
  // Droplet plumes pause between bursts; a sheet streams while its crest keeps breaking.
  let life = mix(0.9, 2.4, spray_rand(seed));
  let period = life * select(1.6 + spray_rand(seed + 1u), 1.0, pop == 1u);
  let clock = water.spray.y + spray_rand(seed + 2u) * period;
  let n = floor(clock / period);
  // (The last member leaves at 0.6 of a life and lives one: within the period, at least 1.6 lives.)
  let tau = clock - n * period - life * 0.6 * f32(member) / f32(plume);
  if (tau < 0.0 || tau >= life) { return o; }
  let h = spray_hash(seed ^ (u32(i32(n)) * 2654435761u));
  // The member's own throw and size.
  let hm = spray_hash(h + member * 977u);
  // Its point on the foam level, the one nearest the camera with that uv (the level is toroidal about it).
  let lv = water.foam_levels[pop];
  // The window its slots fall in: the level's square (toroidal, so a point stays where it is in the world while the
  // window moves, but for those wrapping at its edges, faded there) centred a third of it ahead of the camera, where it
  // looks (about the camera, two thirds of the slots fell behind it or under the frame).
  let fwd = frame.view_forward.xz;
  let ahead = select(vec2<f32>(0.0), fwd / max(length(fwd), 1e-6), dot(fwd, fwd) > 1e-6);
  // The spume's window is the near 320 m where its sheets are seen (to its fade's end), not its level's 512 m: over
  // that, nine in ten of its plumes fell past the fade or out of the frame, and a gale's spume all but never showed.
  let win = select(lv.w * 0.9, 320.0, pop == 1u);
  let cam = frame.view_pos.xz + ahead * (0.33 * win);
  let cam_uv = cam / win;
  // Only where a crest breaks now: where the surface folds (its Jacobian, the whitecaps' own test in sea_foam) or the
  // foam level's freshest foam was laid. (The level's freshness alone, decaying within a second, rarely held a slot.)
  // A gale's crests break over a few percent of the sea, so each life tries up to six points for one (the spume, its
  // sheets fewer and seen further, ten).
  let first = 3u * u32(water.fft.x);
  let ts = lv.w / 256.0;
  var rest = vec2<f32>(0.0);
  var edge = 0.0;
  var keep = 0.0;
  var found = false;
  let tries = select(6u, 10u, pop == 1u);
  for (var t = 0u; t < tries && !found; t++) {
    let ht = h + 16u * t;
    let off = fract(vec2<f32>(spray_rand(ht), spray_rand(ht + 1u)) - cam_uv + 0.5) - 0.5;
    rest = cam + off * win;
    edge = 1.0 - smoothstep(0.4, 0.5, max(abs(off.x), abs(off.y)));
    let fw = textureSampleLevel(heights, ripple_sampler, rest * lv.x + lv.yz, first + pop, 0.0);
    // (At the fine level's texel, for either population: the coarse level's 2 m smoothed every gale crest's fold
    // away, and its spume never found a crest to leave.)
    let texel = water.foam_levels[0].w / 256.0;
    let jac = ocean_normal(rest, rest, vec2<f32>(texel, 0.0), vec2<f32>(0.0, texel)).jac;
    // The spume is torn off every steep crest in a full gale, breaking or not (Beaufort 9 and past: dense streaks of
    // spume blown off the crests): its test takes crests within a third of folding as the wind rises past 17 m/s.
    let torn = select(0.0, 0.35 * smoothstep(17.0, 25.0, wind), pop == 1u);
    let breaking = clamp((water.ocean.z + torn - jac) * water.ocean.w, 0.0, 1.0);
    keep = smoothstep(0.03, 0.35, max(breaking, fw.g)) * gust;
    found = spray_rand(ht + 2u) < keep;
  }
  if (!found) { return o; }
  // Resolve the same fine crest used by the breaking test, not the coarse foam texel.
  let d = ocean_displace(rest, water.foam_levels[0].w / 256.0);
  let start = vec3<f32>(rest.x, water.model[3].y, rest.y) + d;
  // Torn off with the crest's speed downwind (a breaking crest of the wind sea runs at some 0.4 to 0.7 of the wind)
  // and a small lift: the stronger the wind, the flatter its shear lays the spray (at 12 m/s a droplet rises some
  // 0.4 m, in a gale some 0.15; the spume's sheets, heavier and caught broadside, less). (Thrown up at 1 to 7 m/s, a
  // gale's spume stood over its crest in a plume as tall as the crest.)
  let wd = water.sea_wind.xy;
  let side = vec2<f32>(-wd.y, wd.x);
  let along = wind * mix(0.4, 0.7, spray_rand(hm + 3u));
  let across = wind * 0.06 * (spray_rand(hm + 4u) * 2.0 - 1.0);
  let flat = mix(1.0, 0.55, smoothstep(12.0, 24.0, wind));
  let up = mix(1.2, 3.4, spray_rand(hm + 5u)) * flat * select(1.0, 0.75, pop == 1u);
  let v0 = vec3<f32>(wd.x * along + side.x * across, up, wd.y * along + side.y * across);
  // The air it flies in: the wind at a metre and a half over the crest (droplets) or half a metre (the spume's sheets,
  // skimming), gusting along the wind: the gust noise of 40 m patches carried downwind at the wind's speed, the same
  // for a plume's members, so neighbouring plumes stream together.
  let gusting = 0.8 + 0.4 * foam_noise((rest - wd * wind * water.spray.y) / 40.0);
  let air = vec3<f32>(wd.x, 0.0, wd.y) * sea_wind_at(wind, select(1.5, 0.5, pop == 1u)) * gusting;
  // The response rate (1/s): small droplets follow the air within a fraction of a second, sheets of spume slower.
  let k = select(2.2, 1.1, pop == 1u);
  let decay = (1.0 - exp(-k * tau)) / k;
  var p = start + air * tau + (v0 - air) * decay;
  // Vertically it settles at its fall speed (a cloud of fine spray droplets, some 50 µm, 0.4 m/s; the spume's
  // drops, half a millimetre and more, 1.5), reached at the rate gravity sets: falling at its terminal speed in still
  // air (g / k, 4.5 and 9 m/s) a low-flying droplet was back in the sea within half a second.
  let vs = select(0.4, 1.5, pop == 1u);
  let kv = 9.81 / vs;
  let ev = exp(-kv * tau);
  p.y = start.y + (v0.y + vs) * (1.0 - ev) / kv - vs * tau;
  // The sprite: growing as the spray spreads, fading in as it leaves the crest and out as it settles or falls back.
  let rnd = spray_rand(hm + 6u);
  var size = select(mix(0.2, 0.7, rnd) * (1.0 + 1.5 * tau / life), mix(1.0, 2.6, rnd) * (1.0 + 0.8 * tau / life), pop == 1u);
  var alpha = smoothstep(0.0, 0.12 * life, tau) * (1.0 - smoothstep(0.55 * life, life, tau));
  // The sea under it (its rest point taken as where it flies: the sea's sideways sway is a metre or two).
  let sea_y = water.model[3].y + ocean_displace(p.xz, ts).y;
  if (pop == 1u) {
    // The tail skims the sea; the ribbon's head stays on the breaking crest below.
    // Lifting a whole billboard by its radius left grey discs hanging over the sea.
    p.y = max(start.y + 0.2, sea_y + 0.1);
  } else {
    // A droplet falls back into the sea: faded as it nears the water under it.
    alpha *= smoothstep(sea_y - 0.2, sea_y + 0.3, p.y);
  }
  alpha *= select(0.85, 0.8, pop == 1u) * mix(0.6, 1.0, keep) * edge;
  // At least a pixel and a half across: a smaller one is drawn that size with its coverage kept (no shimmer).
  let dist = length(p - frame.view_pos.xyz);
  // Spray is seen near: past some 130 m (the spume's sheets, torn off a gale's crests in streams seen far across the
  // sea, 320 m) it is a haze over the crests, not sprites. (Its droplets stream 15 to 40 m downwind in a life; faded
  // from 15 to 35 m, then 25 to 60 m, a gale's spray left the view before it read as spindrift.)
  alpha *= 1.0 - smoothstep(select(50.0, 120.0, pop == 1u), select(130.0, 320.0, pop == 1u), dist);
  // (The spume's sheets come in from 12 to 35 m: nearer, a sheet metres wide fogged the lens; the near spray is the
  // droplets'.)
  alpha *= select(1.0, smoothstep(12.0, 35.0, dist), pop == 1u);
  let scale = length(vec3<f32>(frame.view_proj[0].y, frame.view_proj[1].y, frame.view_proj[2].y));
  let floor_size = 1.5 * dist * 2.0 / max(scale * water.screen.y, 1e-6);
  if (size < floor_size) {
    alpha *= (size * size) / (floor_size * floor_size);
    size = floor_size;
  }
  if (alpha < 1e-3) { return o; }
  var corners = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0),
    vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, 1.0), vec2<f32>(-1.0, 1.0));
  let c = corners[vi % 6u];
  // Drawn as a wisp along its flight, not a round puff: spindrift
  // streams off a crest, and a lone round sprite read as a dot hanging over the sea.
  let to_eye = normalize(frame.view_pos.xyz - p);
  let e = exp(-k * tau);
  var vel = air + (v0 - air) * e;
  vel.y = (v0.y + vs) * ev - vs;
  var along_v = vel - to_eye * dot(vel, to_eye);
  if (dot(along_v, along_v) < 1e-6) { along_v = cross(vec3<f32>(0.0, 1.0, 0.0), to_eye) + vec3<f32>(1e-5, 0.0, 0.0); }
  let right = normalize(along_v);
  let upv = cross(to_eye, right);
  // Droplets stretch two to four times along their flight, preserving area.
  let stretch = sqrt(clamp(length(along_v) * 0.16, 2.0, 4.0));
  var world = p + (right * c.x * stretch + upv * c.y / stretch) * size * 0.5;
  if (pop == 1u) {
    let u = 0.5 * (c.x + 1.0);
    let head = start + vec3<f32>(0.0, 0.05, 0.0);
    let flight = p - head;
    let across_sheet = normalize(cross(to_eye, flight) + vec3<f32>(1e-5, 0.0, 0.0));
    world = mix(head, p, u) + across_sheet * c.y * size * 0.3 * (1.0 - 0.6 * u);
  }
  o.clip = frame.view_proj * vec4<f32>(world, 1.0);
  o.world = world;
  o.corner = c;
  o.alpha = alpha;
  return o;
}

@fragment
fn fs_spray(in: SprayOut) -> @location(0) vec4<f32> {
  let r2 = dot(in.corner, in.corner);
  var a: f32;
  if (in.sheet == 1u) {
    let u = 0.5 * (in.corner.x + 1.0);
    let across = in.corner.y * in.corner.y;
    let strands = 0.4 + 0.6 * foam_noise(vec2<f32>(u * 6.0 - water.spray.y * 2.0, in.corner.y * 10.0));
    a = in.alpha * exp(-3.0 * across) * (1.0 - across) * smoothstep(0.0, 0.12, u) * pow(1.0 - u, 1.5) * strands;
  } else {
    if (r2 >= 1.0) { discard; }
    a = in.alpha * exp(-2.0 * r2) * (1.0 - r2);
  }
  // Droplets scatter forward (a Henyey–Greenstein lobe of g 0.75, mixed with an even share for the many bounces in a
  // dense cloud of them), lit by the sun through the shadows and by the sky above.
  let l = normalize(frame.light_dir.xyz);
  let v = normalize(frame.view_pos.xyz - in.world);
  let g = 0.75;
  let cos_t = dot(l, -v);
  let hg = (1.0 - g * g) / (12.5663706 * pow(1.0 + g * g - 2.0 * g * cos_t, 1.5));
  var sun = 1.0;
  if (water.extra.x > 0.0 && frame.light_dir.w > 0.5) {
    sun = mix(1.0, cascade_shadow(in.world, vec3<f32>(0.0, 1.0, 0.0), max(l.y, 0.0)), water.extra.x);
  }
  let sun_e = frame.light_color.rgb * sun * step(0.0, l.y);
  let sky = ambient_light_level(vec3<f32>(0.0, 1.0, 0.0), in.world) * frame.params.x;
  let deck = cloud_deck(vec3<f32>(0.0, 1.0, 0.0));
  let deck_up = select(deck, vec4<f32>(0.0), frame.clouds[4].w > 0.5);
  // Lit by the sky as the sea's foam is (nearly grey, its irradiance over the whole sky above): spume is foam torn off
  // the crest and a dense cloud of droplets scatters as much; lit at 0.9 of the sky's radiance, it sat at a third of
  // the foam it leaves and vanished under an overcast.
  let sky_grey = mix(sky, vec3<f32>(dot(sky, vec3<f32>(0.2126, 0.7152, 0.0722))), 0.85) * 1.55;
  let ambient = mix(sky_grey, deck_up.rgb * 1.4, deck_up.a);
  let lit = water.foam.rgb * (sun_e * (0.6 * hg + 0.4 * 0.0795775) + ambient);
  let fogged = apply_fog(lit, in.world);
  return vec4<f32>(fogged * a, a);
}
