// The terrain's lit, prepass and normal-map passes. Prepended by
// frame.wgsl, the light tables (clusters.rs), lighting.wgsl and terrain.wgsl.
//
// Holes (terrain.wgsl terrain_hole) are discarded in every pass.
//
// Shading: the normal from the terrain's normal map (built from the
// heights on the GPU, mipmapped), up to sixteen layers weighted by the
// control map's channels times each layer's slope and altitude rule, the
// four strongest at each pixel blended (all of them with four or fewer),
// optionally sharpened by the layers' heights (their albedo's alpha, each
// layer's own height contrast), each layer sampled by world xz, triplanar,
// or past its projectSlope biplanar, each projection hex-tiled when it
// asks for anti-tiling, with its macro variation, lit like a
// StandardMaterial (cascaded shadows, local lights, image-based light),
// caustics under the frame's water, then fog.

struct TerrainFx { decal_info: vec4<f32> };
@group(3) @binding(0) var<uniform> fx: TerrainFx;
@group(3) @binding(2) var decal_atlas: texture_2d<f32>;
@group(3) @binding(3) var fx_sampler: sampler;
@group(3) @binding(5) var<storage, read> decal_list: array<DecalGpu>;
@group(3) @binding(6) var<storage, read> decal_cells: array<u32>;

struct TerrainOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world: vec3<f32>,
  @location(1) cur_clip: vec4<f32>,
  @location(2) prev_clip: vec4<f32>,
  // The point in the terrain's space (its maps are laid out there).
  @location(3) local: vec3<f32>,
  // The displaced surface's slope (terrain_relief: xy its `slope`, zw
  // its `bare` part), 0 where the ground is not displaced.
  @location(4) relief: vec4<f32>,
};

@vertex
fn vs(@builtin(vertex_index) vi: u32) -> TerrainOut {
  let local = terrain_local_vertex(vi);
  let p = terrain_world(local);
  var o: TerrainOut;
  o.clip = frame.view_proj * vec4<f32>(p, 1.0);
  o.world = p;
  o.local = local;
  o.cur_clip = terrain.view_proj * vec4<f32>(p, 1.0);
  o.prev_clip = terrain.prev_view_proj * vec4<f32>(p, 1.0);
  if (terrain.displace.x > 0.0) {
    o.relief = terrain_vertex_relief(vi);
  }
  return o;
}

// The terrain's maps at a point of its space: its world normal and the
// control map's channels, from the finest resident tile.
struct TerrainPoint {
  normal: vec3<f32>,
  // The same in the terrain's space (unit length).
  local_normal: vec3<f32>,
  control: array<vec4<f32>, 4>,
  // The normal map's spread there (TerrainMaps.variance).
  variance: f32,
  // The height there, when asked for (TerrainMaps.height).
  height: f32,
};

// World normal `n` turned by a displaced surface's `slope` (its height's
// change along the terrain's x and z, per metre) on the ground of
// `point`: the field's surface y = h(x, z) has the normal (-h_x, 1, -h_z)
// up to scale, the displaced one (-h_x - d_x, 1, -h_z - d_z).
fn terrain_tilt(point: TerrainPoint, n: vec3<f32>, slope: vec2<f32>) -> vec3<f32> {
  if (all(slope == vec2<f32>(0.0))) { return n; }
  let m = point.local_normal;
  let shift = terrain.normal * (-m.y * vec3<f32>(slope.x, 0.0, slope.y));
  return normalize(n + shift / max(length(terrain.normal * m), 1e-6));
}

fn terrain_point(local: vec3<f32>) -> TerrainPoint {
  let s = (local.xz - terrain_min()) / terrain_cell();
  return terrain_point_from(resident_from(0u, s), s, false);
}

// `terrain_point` at field coordinates `s` from the resident tile `first`
// already looked up there; `with_height` also reads the height.
fn terrain_point_from(first: Resident, s: vec2<f32>, with_height: bool) -> TerrainPoint {
  let m = terrain_maps_from(first, s, dpdxFine(s), dpdyFine(s), with_height);
  var o: TerrainPoint;
  o.local_normal = normalize(m.normal);
  o.normal = normalize(terrain.normal * o.local_normal);
  o.control = m.control;
  o.variance = m.variance;
  o.height = m.height;
  return o;
}

// ── Macro variation ─────────────────────────────────────────────────────
// What keeps a field of one layer from reading as one texture from afar:
// a slow drift of brightness and hue over the ground (value noise over
// world xz, measured in the layer's repeats, so it is the same wherever
// the frame's origin is), and the texture at a quarter of its frequency
// blended in with distance (10 to 40 repeats away).

fn value_noise(x: vec2<f32>) -> f32 {
  let i = floor(x);
  let f = x - i;
  let u = f * f * (3.0 - 2.0 * f);
  let c = vec2<i32>(i);
  // The cell's four corners' hashes in one loop (one copy of the hash).
  var h = vec4<f32>(0.0);
  for (var j = 0u; j < 4u; j++) {
    h[j] = hex_hash(c + vec2<i32>(i32(j & 1u), i32(j >> 1u))).x;
  }
  return mix(mix(h.x, h.y, u.x), mix(h.z, h.w, u.x), u.y);
}

fn layer_coarse(l: TerrainLayerGpu, p: vec3<f32>) -> f32 {
  if (l.look.w <= 0.0) { return 0.0; }
  let tile = 1.0 / max(l.mapping.x, 1e-6);
  return smoothstep(10.0 * tile, 40.0 * tile, distance(p, frame.view_pos.xyz)) * clamp(l.look.w, 0.0, 1.0);
}

fn macro_tint(l: TerrainLayerGpu, p: vec3<f32>) -> vec3<f32> {
  if (l.look.w <= 0.0) { return vec3<f32>(1.0); }
  let m = clamp(l.look.w, 0.0, 1.0);
  let at = (p.xz + frame.origin.xz) * l.mapping.x;
  // Three octaves in one loop (one copy of the noise's code): the slow
  // drift's two, then the hue's.
  let scale = array<f32, 3>(16.0, 5.0, 40.0);
  let shift = array<vec2<f32>, 3>(vec2<f32>(0.0), vec2<f32>(17.3, 5.1), vec2<f32>(3.1, 41.7));
  var v: array<f32, 3>;
  for (var j = 0u; j < 3u; j++) { v[j] = value_noise(at / scale[j] + shift[j]); }
  let slow = v[0] * 0.65 + v[1] * 0.35;
  let hue = v[2];
  let bright = 1.0 + m * 0.6 * (slow - 0.5);
  let warm = mix(vec3<f32>(0.92, 1.0, 1.07), vec3<f32>(1.07, 1.0, 0.88), hue);
  return bright * mix(vec3<f32>(1.0), warm, m);
}

// ── A layer at a point ──────────────────────────────────────────────────

// Its albedo and height alone (the prepass: no normal map is read).
fn sample_layer_albedo(i: u32, p: vec3<f32>, n: vec3<f32>, dpx: vec3<f32>, dpy: vec3<f32>) -> vec4<f32> {
  let l = terrain.layers[i];
  if ((u32(l.mapping.w) & 1u) == 0u) { return vec4<f32>(1.0); }
  let proj = layer_projection(l, n);
  let coarse = layer_coarse(l, p);
  var a = vec4<f32>(0.0);
  for (var k = 0u; k < proj.count; k++) {
    let t = plane_taps(l, proj.planes[k], p, dpx, dpy);
    a += taps_albedo(i, t, coarse).albedo * proj.w[k];
  }
  return a;
}

// Where a layer's albedo is read for a fragment `lift` metres above the
// field (the drawn displacement) on a face whose drawn normal is `face`
// (`n` the ground's): on the sides the layer's relief raises (a sett's
// wall), as far in from the arris as the fragment lies below it, so a
// wall shows its own stone's face unfolded down from its arris, at its
// own scale. The drawn wall is a ramp over the clipmap's finest spacing,
// and from above, its foot read the texture further out than its top did
// by as much as it is lower over its slope (`face.y / lean` a metre per
// metre of drop): the read moves in by that and a metre more, whatever
// the ramp's slope, from an arris a fifth of the layer's displacement
// over the field (a sett's crown: on a lower stone the read starts a
// little further in, still on that stone). Read from above where it
// stands, a wall took the one row of texels along its edge stretched
// down it into streaks; a side projection (biplanar) read whatever the
// layout had at that place, the joints' mortar in squares across the
// stones' walls.
struct UnfoldedAt {
  p: vec3<f32>,
  dpx: vec3<f32>,
  dpy: vec3<f32>,
  // How much of the albedo is read there (0: from above where it stands,
  // as before; between, the two reads blended: the read's place blended
  // instead stood still on some slope between, and streaked there).
  wall: f32,
};

fn unfolded_at(l: TerrainLayerGpu, p: vec3<f32>, n: vec3<f32>, face: vec3<f32>, lift: f32, dpx: vec3<f32>, dpy: vec3<f32>) -> UnfoldedAt {
  var o = UnfoldedAt(p, dpx, dpy, 0.0);
  if (l.offset.w <= 0.0) { return o; }
  // The face's lean from the ground, along the ground (out of the stone).
  let lean = face - n * dot(face, n);
  let out = vec3<f32>(lean.x, 0.0, lean.z);
  let len = length(out);
  // A side from some 20° off the ground's normal, whole from 37°: a
  // stone's top leans a few degrees, its arris and wall more.
  let wall = smoothstep(0.35, 0.6, length(lean));
  if (wall <= 0.0 || len < 1e-4) { return o; }
  let dir = out / len;
  let k = max(dot(face, n), 0.0) / len + 1.0;
  let depth = max(0.2 * l.offset.w - lift, 0.0);
  // (The depth changes down the wall as the height does: its footprint,
  // so the read's filter follows the unfolded face.)
  o.p = p - dir * depth * k;
  o.dpx = dpx + dir * dpx.y * k;
  o.dpy = dpy + dir * dpy.y * k;
  o.wall = wall;
  return o;
}

// Its albedo, height and detailed normal at a fragment `lift` metres
// above the field on a face whose drawn normal is `face` (`n`, the
// ground's, where the ground is not displaced): the normal map read about
// the ground's normal where the fragment stands (its relief is the
// ground's, the walls included), the albedo where `unfolded_at` puts it,
// tinted by its cell's tone (`cell_tone`, read from above where it
// stands: a stone's walls take its top's tone).
fn sample_layer(i: u32, p: vec3<f32>, n: vec3<f32>, face: vec3<f32>, lift: f32, dpx: vec3<f32>, dpy: vec3<f32>) -> LayerSample {
  let l = terrain.layers[i];
  let has_albedo = (u32(l.mapping.w) & 1u) != 0u;
  let has_normal = (u32(l.mapping.w) & 2u) != 0u;
  var s: LayerSample;
  s.albedo = vec4<f32>(1.0);
  s.normal = n;
  if (!has_albedo && !has_normal) { return s; }
  let proj = layer_projection(l, n);
  let at = unfolded_at(l, p, n, face, lift, dpx, dpy);
  let coarse = layer_coarse(l, p);
  var albedo = vec4<f32>(0.0);
  var normal = vec3<f32>(0.0);
  var tone = vec3<f32>(0.0);
  var tone_w = 0.0;
  for (var k = 0u; k < proj.count; k++) {
    let plane = proj.planes[k];
    // The taps here (the normal's too), then, for the albedo, on the
    // unfolded wall, in one loop (one copy of the taps' code: DXC inlines
    // every call).
    var t: PlaneTaps;
    var w = vec3<f32>(0.0);
    var a = vec4<f32>(0.0);
    for (var j = 0u; j < 2u; j++) {
      let share = select(at.wall, 1.0 - at.wall, j == 0u);
      if (j == 1u && (!has_albedo || share <= 0.0)) { break; }
      let here = j == 0u;
      let taps = plane_taps(l, plane, select(at.p, p, here), select(at.dpx, dpx, here), select(at.dpy, dpy, here));
      if (here) {
        t = taps;
        w = hex_weights(t);
      }
      if (!has_albedo || share <= 0.0) { continue; }
      let r = taps_albedo(i, taps, coarse);
      a += r.albedo * share;
      if (here) { w = r.w; }
    }
    if (has_albedo) { albedo += a * proj.w[k]; }
    if (has_normal) {
      let m = taps_normal(i, t, w);
      normal += plane_normal(proj.whiteout[k], plane, m.normal, n, l.mapping.z) * proj.w[k];
      if (plane == 0u) {
        tone += m.tone * proj.w[k];
        tone_w += proj.w[k];
      }
    }
  }
  if (has_albedo) {
    s.albedo = albedo;
    if (tone_w > 0.0) { s.albedo = vec4<f32>(albedo.rgb * tone / tone_w, albedo.a); }
  }
  if (has_normal) { s.normal = normalize(normal); }
  return s;
}

// The blended layers at a fragment: albedo, shading normal, roughness,
// and how high it lies in their relief (`layers_height`).
struct TerrainLook {
  albedo: vec3<f32>,
  normal: vec3<f32>,
  roughness: f32,
  height: f32,
};

// How high a fragment lies in its picked layers' relief under weights
// `w`, 0 (a hollow) to 1 (a crest): their height maps' heights blended,
// 0.5 (none known) for a layer without one. The weather fills the
// hollows first (weather_surface.wgsl `cavity`); the lit pass and the
// prepass both, so puddles and the reflections' gloss agree.
fn layers_height(pick: LayerPick, w: vec4<f32>, heights_of: vec4<f32>) -> f32 {
  var h = 0.0;
  var total = 0.0;
  for (var k = 0u; k < 4u; k++) {
    if (k < pick.n && w[k] > 0.0) {
      let mapped = (u32(terrain.layers[pick.ids[k]].mapping.w) & 1u) != 0u;
      h += w[k] * select(0.5, heights_of[k], mapped);
      total += w[k];
    }
  }
  return select(0.5, h / total, total > 0.0);
}

// The weather's cavity for a layers' height: the relief's crests and
// hollows, kept off the extremes (as pbr_fs.wgsl weather_cavity).
fn terrain_cavity(height: f32) -> f32 {
  return 0.15 + 0.7 * height;
}

// Whether the weather lays anything on the ground this frame (never in a
// pipeline without the weather's code, OG_WEATHER).
fn weather_active() -> bool {
  return OG_WEATHER && max(max(max(frame.weather.x, frame.weather.y), frame.weather.z), frame.weather_dust.w) > 0.0;
}

// The picked layers' roughness under weights `w` (the lit pass and the
// prepass both: their roughness is one function of the same weights).
fn blended_roughness(pick: LayerPick, w: vec4<f32>) -> f32 {
  if (u32(terrain.origin.w) == 0u) { return 0.9; }
  var roughness = 0.0;
  for (var k = 0u; k < 4u; k++) {
    if (k < pick.n && w[k] > 0.0) {
      roughness += terrain.layers[pick.ids[k]].color.w * w[k];
    }
  }
  return roughness;
}

// The layers at world point `p` whose geometric normal is `n_geo` (the
// terrain's, or a stamp's) under control channels `control`, its drawn
// face's normal `face` and its lift over the field (`sample_layer`).
fn terrain_look(p: vec3<f32>, n_geo: vec3<f32>, face: vec3<f32>, lift: f32, control: array<vec4<f32>, 4>, dpx: vec3<f32>, dpy: vec3<f32>) -> TerrainLook {
  let count = u32(terrain.origin.w);
  let pick = pick_layers(p, n_geo, control);
  // The picked layers' colours and normals (a layer of weight 0 skips its maps).
  var samples: array<LayerSample, 4>;
  var heights_of = vec4<f32>(1.0);
  for (var k = 0u; k < 4u; k++) {
    if (k < pick.n && (pick.w[k] > 0.0 || k == 0u)) {
      samples[k] = sample_layer(pick.ids[k], p, n_geo, face, lift, dpx, dpy);
      heights_of[k] = samples[k].albedo.a;
    }
  }
  let w = blend_weights(pick, heights_of);
  var albedo = vec3<f32>(0.0);
  var normal = vec3<f32>(0.0);
  for (var k = 0u; k < 4u; k++) {
    if (k < pick.n && w[k] > 0.0) {
      let l = terrain.layers[pick.ids[k]];
      albedo += samples[k].albedo.rgb * l.color.rgb * macro_tint(l, p) * w[k];
      normal += samples[k].normal * w[k];
    }
  }
  var height = layers_height(pick, w, heights_of);
  if (count == 0u) {
    albedo = vec3<f32>(0.5);
    normal = n_geo;
    height = 0.5;
  }
  return TerrainLook(albedo, normal, blended_roughness(pick, w), height);
}

// The roughness `terrain_look` gives a fragment (x) and its height in
// the layers' relief (y), from the layers' heights alone: the prepass
// reads no normal map and blends no colour. (The heights are read when
// the layers blend by them or the weather needs them.)
fn terrain_roughness(p: vec3<f32>, n_geo: vec3<f32>, control: array<vec4<f32>, 4>, dpx: vec3<f32>, dpy: vec3<f32>) -> vec2<f32> {
  let pick = pick_layers(p, n_geo, control);
  var heights_of = vec4<f32>(1.0);
  if ((terrain.params.x > 0.0 && !pick.fallback) || weather_active()) {
    for (var k = 0u; k < 4u; k++) {
      if (k < pick.n && (pick.w[k] > 0.0 || k == 0u)) {
        heights_of[k] = sample_layer_albedo(pick.ids[k], p, n_geo, dpx, dpy).a;
      }
    }
  }
  let w = blend_weights(pick, heights_of);
  let height = select(0.5, layers_height(pick, w, heights_of), u32(terrain.origin.w) > 0u);
  return vec2<f32>(blended_roughness(pick, w), height);
}

// The toon ramp's three bands for the terrain (pbr.wgsl toon_ramp's built-in
// one), each edge smoothed over a pixel of t.
fn terrain_band(t: f32) -> f32 {
  let e = max(fwidthFine(t), 1e-4);
  return 0.22 + 0.38 * smoothstep(0.08 - e, 0.08 + e, t) + 0.4 * smoothstep(0.45 - e, 0.45 + e, t);
}

// The share of layer `layer` at a point (world `p`, normal `n`, control
// channels `c`) over every layer's weight, as og.Grass places its blades
// (grass_generate.wgsl grass_layer_weight).
fn grass_layer_share(p: vec3<f32>, n: vec3<f32>, c: array<vec4<f32>, 4>, layer: u32) -> f32 {
  let count = min(u32(terrain.origin.w), 16u);
  let angle = degrees(acos(clamp(n.y, -1.0, 1.0)));
  let altitude = p.y - terrain.origin.y;
  var sum = 0.0;
  var mine = 0.0;
  for (var i = 0u; i < count; i++) {
    let l = terrain.layers[i];
    let w = c[i / 4u][i % 4u] * band(angle, l.slope.x, l.slope.y, l.slope.z) * band(altitude, l.altitude.x, l.altitude.y, l.altitude.z);
    sum += w;
    if (i == layer) { mine = w; }
  }
  if (sum < 1e-4) { return select(0.0, 1.0, layer == 0u); }
  return mine / sum;
}

// og.Grass's colour on the ground under and past its blades (grass.rs,
// after bevy_grass's far.wesl): where its layer shows, the ground takes the
// field's average colour from `fade.x` on, whole from `fade.y`, so the
// gaps between blades and the ground past them read as the same field
// and there is no seam where the blades thin out to nothing; per clump
// and dry drift as the blades are, paler where a gust presses it down.
// Whether a pipeline has og.Grass's far colour on the ground
// (pipeline_features.rs).
override OG_GRASS: bool = true;

fn grass_far(albedo: vec3<f32>, p: vec3<f32>, local: vec3<f32>, n: vec3<f32>, c: array<vec4<f32>, 4>) -> vec3<f32> {
  if (!OG_GRASS) { return albedo; }
  let g = terrain.grass;
  if (g.color.w < 0.0) { return albedo; }
  let amount = smoothstep(g.fade.x, g.fade.y, distance(p, frame.view_pos.xyz)) * g.fade.w * grass_layer_share(p, n, c, u32(g.color.w));
  if (amount <= 0.0) { return albedo; }
  let at = p.xz + frame.origin.xz;
  let sheen = grass_gust_sheen(g.wind, at, g.fade.z);
  let clump = grass_clump(local.xz - terrain_min(), g.clump.x, u32(g.clump.w));
  let v = g.clump.y;
  var col = g.color.rgb * mix(1.0 - 0.35 * v, 1.0 + 0.25 * v, clump.x);
  // The dry drifts as the ground's share of dry clumps, blended (a blade
  // is dry or not; the ground under many shows their mix), a little
  // more or less per clump.
  let dry = clamp(grass_dry_share(at, g.clump.z) * mix(0.7, 1.3, clump.y), 0.0, 1.0);
  col = mix(col, g.dry.rgb * 0.8, dry * 0.75);
  return mix(albedo, col * sheen, amount);
}

// Six repeating colours, shared by the layer and mip diagnostic legends.
fn terrain_debug_color(i: u32) -> vec3<f32> {
  let colors = array<vec3<f32>, 6>(
    vec3<f32>(0.1, 1.0, 0.1), vec3<f32>(1.0, 1.0, 0.1),
    vec3<f32>(1.0, 0.2, 0.1), vec3<f32>(0.1, 0.8, 1.0),
    vec3<f32>(0.6, 0.2, 1.0), vec3<f32>(1.0, 0.5, 0.1));
  return colors[i % 6u];
}

fn terrain_debug(in: TerrainOut, point: TerrainPoint) -> vec3<f32> {
  // Every derivative before the first branch: the views below return early on
  // per-pixel values (a layer's flags), and WGSL refuses a derivative in control
  // flow that depends on one (Tint, so the browser, rejects the whole module).
  let vertical = dpdyFine(in.world);
  let samples = (in.local.xz - terrain_min()) / terrain_cell();
  let gx = dpdxFine(samples);
  let gy = dpdyFine(samples);
  let cell = (in.local.xz - terrain_min()) / (level.camera.z * level.center.z);
  let edge = abs(fract(cell - 0.5) - 0.5) / max(fwidthFine(cell), vec2<f32>(1e-5));
  if (frame.light_color.w > 9.5) {
    let pick = pick_layers(in.world, point.normal, point.control);
    let layer = terrain.layers[pick.ids.x];
    if (frame.light_color.w > 10.5) {
      if ((u32(layer.mapping.w) & 1u) == 0u) { return debug_unsupported(in.clip.xy).rgb; }
      let dy = vertical * layer.mapping.x;
      let size = layer_dimensions(pick.ids.x);
      let density = vec3<f32>(length(dy.zy * size), length(dy.xz * size), length(dy.xy * size));
      // Linear RGB = d/(1+d): black 0, half intensity 1 texel per
      // screen pixel, approaching white as the footprint grows.
      return density / (vec3<f32>(1.0) + density);
    }
    if (layer.mapping.y < 0.5) { return vec3<f32>(0.0, 1.0, 0.0); }
    let weights = pow(abs(point.normal), vec3<f32>(4.0));
    return weights / max(dot(weights, vec3<f32>(1.0)), 1e-5);
  }
  if (frame.light_color.w > 8.5) {
    let resident = resident_from(0u, samples);
    let footprint = max(length(gx), length(gy)) / f32(1u << resident.lvl);
    let mip = max(log2(max(footprint, 1.0)), 0.0);
    return terrain_debug_color(u32(mip)) * (1.0 - 0.5 * fract(mip));
  }
  if (frame.light_color.w > 7.5) {
    let pick = pick_layers(in.world, point.normal, point.control);
    if (pick.fallback) { return vec3<f32>(1.0, 0.0, 1.0); }
    let share = pick.w.x / max(dot(pick.w, vec4<f32>(1.0)), 1e-5);
    return terrain_debug_color(pick.ids.x) * (0.25 + 0.75 * share);
  }
  var weights = pow(abs(point.normal), vec3<f32>(4.0));
  weights /= max(dot(weights, vec3<f32>(1.0)), 1e-5);
  let c = (in.local.xz - terrain_min()) / level.camera.z;
  let d = max(abs(c.x - level.camera.x), abs(c.y - level.camera.y)) / level.center.z;
  let seam = select(0.0, clamp((d - level.grid.y) / max(level.grid.z, 1.0), 0.0, 1.0), level.grid.z > 0.0);
  return mix(weights * mix(0.15, 1.0, clamp(min(edge.x, edge.y), 0.0, 1.0)), vec3<f32>(1.0), seam * 0.7);
}

// Whether a pipeline has the terrain's diagnostic views
// (pipeline_features.rs).
override OG_DEBUG: bool = true;

@fragment
fn fs(in: TerrainOut) -> @location(0) vec4<f32> {
  // One resident-tile lookup for the hole, the maps, the height and the
  // horizon (each inlined its own before).
  let s = (in.local.xz - terrain_min()) / terrain_cell();
  let first = resident_from(0u, s);
  if (terrain_hole_from(first)) { discard; }
  let p = in.world;
  let dpx = dpdxFine(p);
  let dpy = dpdyFine(p);
  // The height (for a displaced ground's lift) in the maps' loop.
  let point = terrain_point_from(first, s, terrain.displace.x > 0.0);
  // Nested, so a pipeline without the views folds the branch away whole.
  if (OG_DEBUG) {
    if (frame.light_color.w > 6.5) {
      return vec4<f32>(terrain_debug(in, point), 1.0);
    }
  }
  let n_geo = point.normal;
  // The drawn triangle's own plane near the eye where the ground is
  // displaced (`n_shadow` below; its layers' albedo, `sample_layer`).
  var n_face = n_geo;
  var lift = 0.0;
  if (terrain.displace.x > 0.0) {
    lift = (in.local.y - point.height) * terrain.displace.y;
    var facet = normalize(cross(dpx, dpy));
    facet = select(facet, -facet, dot(facet, n_geo) < 0.0);
    let reach = terrain.displace.x * length(terrain.model[0].xyz);
    let near = 1.0 - smoothstep(0.6 * reach, reach, distance(p, frame.view_pos.xyz));
    n_face = normalize(mix(n_geo, facet, near));
  }
  let look = terrain_look(p, n_geo, n_face, lift, point.control, dpx, dpy);
  let albedo = grass_far(look.albedo, p, in.local, n_geo, point.control);
  let roughness = toksvig_roughness(look.roughness, point.variance);
  // A displaced ground's shading takes the relief no normal map draws.
  // Its weathering keeps the field's normal (the drawn one is faceted).
  // Its shadow lookup takes the drawn triangle's own plane, the surface
  // the shadow passes drew: the receiver plane its filter compares along
  // and its offset follow the stone's steep side, which the field's
  // level normal does not (a tent of texels on that side stood above
  // the plane and shadowed the joint at its foot in round black specks);
  // and an interpolated normal is neither (its faces turned to a low sun
  // shadowed themselves in squares of shadow texels).
  let n_shade = terrain_tilt(point, look.normal, in.relief.zw);
  return terrain_shade(in.clip.xy, p, first, 1.0, albedo, n_shade, roughness, n_geo, n_face, look.height);
}

// The terrain's shading of a fragment at world `p` (the ground or one of
// its stamps): its layers' albedo, shading normal and roughness under
// the sun's cascades, the local lights and image-based light, caustics
// and fog, with the frame's diagnostic views.
// `reads`: the resident tile of the point of the ground whose horizon it
// reads (`resident_from`); `ground`: how much of the fragment is that ground (1 for the
// terrain; a stamp's blend share, its own face not being one); `n_geo`
// the surface's normal for weathering, `n_shadow` the geometry's own for
// the shadow lookup's offset (a stamp's blended foot takes the ground's
// normal, but its wall still faces out: offset along the ground's, the
// wall shadowed itself in a dark strip along its foot).
fn terrain_shade(clip: vec2<f32>, p: vec3<f32>, reads: Resident, ground: f32, albedo: vec3<f32>, normal: vec3<f32>, roughness: f32, n_geo: vec3<f32>, n_shadow: vec3<f32>, height: f32) -> vec4<f32> {
  var n = normalize(normal);
  var s: Surface;
  s.albedo = albedo;
  s.metallic = 0.0;
  s.roughness = clamp(roughness, 0.04, 1.0);
  s.f0 = vec3<f32>(0.04);
  let dpx = dpdx(p);
  let dpy = dpdy(p);
  var decal_emissive = vec3<f32>(0.0);
  if (abs(fx.decal_info.x) > 0.5) {
    let d = apply_decals(clip, p, n_geo, s.albedo, s.roughness, dpx, dpy);
    s.albedo = d.albedo;
    s.roughness = clamp(d.roughness, 0.04, 1.0);
    decal_emissive = d.emissive;
    if (d.normal.w > 0.0) { n = normalize(mix(n, d.normal.xyz, d.normal.w)); }
  }
  // The weather on the ground, its hollows (the layers' relief) first.
  let weathered = weather_surface_at(s, n, n_geo, p, terrain_cavity(height), 1.0, WEATHER_GROUND);
  s = weathered.s;
  n = weathered.n;
  if (frame.light_color.w > 1.5 && frame.light_color.w < 2.5) {
    return debug_unsupported(clip);
  }
  if (frame.light_color.w > 2.5 && frame.light_color.w < 3.5) {
    return vec4<f32>(s.albedo, 1.0);
  }
  let v = select(normalize(frame.view_pos.xyz - p), -frame.view_forward.xyz, frame.view_pos.w > 0.5);

  let l = normalize(frame.light_dir.xyz);
  let n_dot_l = max(dot(n, l), 0.0);
  var shadow = 1.0;
  if (frame.light_dir.w > 0.5 && n_dot_l > 0.0) { shadow = cascade_shadow(p, n_shadow, n_dot_l); }
  // Past the cascades (and with none) the horizon map shades the ground:
  // the sun behind a ridge (soft over 1.5°: the sun's disc and the bake's
  // reads), and the sky its horizon hides.
  var sky = 1.0;
  if (terrain.params.w > 0.5) {
    let h = terrain_horizon_from(reads);
    if (h.ok) {
      // World to the terrain's space: the inverse of its linear part, the
      // transpose of the normal matrix.
      let sl = transpose(terrain.normal) * l;
      let elevation = atan2(sl.y, length(sl.xz));
      let edge = horizon_towards(h, atan2(sl.z, sl.x));
      let sun = smoothstep(edge - 0.013, edge + 0.013, elevation);
      var far = 1.0;
      let count = u32(frame.light_dir.w);
      if (count > 0u) {
        let depth = dot(p - frame.view_pos.xyz, frame.view_forward.xyz);
        let last = frame.cascade_bias[count - 1u].z;
        far = smoothstep(0.9 * last, last, depth);
      }
      shadow *= mix(1.0, sun, far);
      sky = mix(1.0, horizon_occlusion(h), ground);
    }
  }
  var color = brdf(s, n, v, l) * frame.light_color.rgb * shadow;
  // og.MaterialStyle's shadow tint: the sunlight a cast shadow takes moves towards the tint, as on a lit material.
  var tinted = (1.0 - shadow) * n_dot_l;
  if (frame.outline_style.z > 0.5) {
    // og.MaterialStyle's toon: the ground in the built-in ramp's three bands
    // (as a lit material without its own gradientMap), matte (no hard
    // highlight), each band's edge smoothed over a pixel so it does not crawl.
    let band = terrain_band(n_dot_l * shadow);
    color = s.albedo / PI * band * frame.light_color.rgb;
    // The tint takes what the ramp lost to the shadow, once, from the final
    // shadow factor: inside the dark band a deeper shadow (two overlapping,
    // or a soft one deepened) loses nothing more, so it gains no more tint
    // (else overlapping shadows came out lighter).
    tinted = terrain_band(n_dot_l) - band;
  }
  if (frame.shadow_tint.w > 0.0 && n_dot_l > 0.0) {
    var tint = frame.shadow_tint.rgb * frame.shadow_tint.w;
    // Under toon, never more than the diffuse light the ramp took (as pbr_fs.wgsl).
    if (frame.outline_style.z > 0.5) { tint = min(tint, vec3<f32>(1.0 / PI)); }
    color += tinted * tint * s.albedo * frame.light_color.rgb;
  }
  color += s.albedo * frame.light_color.rgb * n_dot_l * shadow * caustics(p);
  // Point, spot and rect lights: this fragment's cluster's, or every light
  // in a reflection probe's face (area.wgsl surface_local_lights).
  color += surface_local_lights(clip, s, p, n, v);
  let n_dot_v = max(dot(n, v), 1e-4);
  let f_ibl = f_schlick_roughness(n_dot_v, s.f0, s.roughness);
  let kd_ibl = vec3<f32>(1.0) - f_ibl;
  // The environment, or the probe holding the tile blended with it (probes.rs),
  // at its intensity, plus the hemisphere light at its own.
  let irradiance = ambient_light(n, p) * frame.params.x + hemi_irradiance(n);
  let r = reflect(-v, n);
  // With the screen-space reflections over it (frame.wgsl ssr_reflected).
  let prefiltered = ssr_reflected(reflected_light(r, p, s.roughness * frame.params.y) * frame.params.x, p, clip);
  let lut = textureSample(brdf_lut, ibl_sampler, vec2<f32>(n_dot_v, s.roughness)).rg;
  let diffuse_ibl = kd_ibl * irradiance * s.albedo * sky;
  let specular_ibl = prefiltered * (s.f0 * lut.x + lut.y) * sky;  // Karis split sum (pbr_fs.wgsl)
  let view = u32(frame.light_color.w);
  if (view == 4u) { return vec4<f32>(diffuse_ibl, 1.0); }
  if (view == 5u) { return vec4<f32>(specular_ibl, 1.0); }
  if (view == 6u) { return vec4<f32>(color, 1.0); }
  color += diffuse_ibl + specular_ibl + decal_emissive;

  if (frame.light_color.w > 0.5 && frame.light_color.w < 1.5) {
    return vec4<f32>(cascade_tint(cascade_index(p)) * 4.0, 1.0);
  }
  return vec4<f32>(apply_fog(color, p), 1.0);
}

struct TerrainPrepass {
  @location(0) gbuffer: vec4<f32>,
  @location(1) depth: vec4<f32>,
  @location(2) outline: vec4<f32>,
};

// The prepass's targets for a fragment of the ground or a stamp: `n` its
// drawn normal, `n_geo` the field's (or the stamp's) the weather lies by,
// `layers` its roughness and height (terrain_roughness). Wet ground,
// puddles and their ripples as the lit pass shades them
// (weather_surface.wgsl weather_gloss), so reflections see them.
fn terrain_prepass_out(p: vec3<f32>, n: vec3<f32>, n_geo: vec3<f32>, layers: vec2<f32>, cur_clip: vec4<f32>, prev_clip: vec4<f32>) -> TerrainPrepass {
  let cur = cur_clip.xy / cur_clip.w;
  let prev = prev_clip.xy / max(prev_clip.w, 1e-6);
  let g = weather_gloss(clamp(layers.x, 0.04, 1.0), 0.0, n, n_geo, p, terrain_cavity(layers.y), 1.0, WEATHER_GROUND);
  var o: TerrainPrepass;
  o.gbuffer = vec4<f32>(oct_encode(g.n), (cur - prev) * vec2<f32>(0.5, -0.5));
  let reflectivity = reflectivity_bits(g.s.roughness, g.s.metallic);
  o.depth = vec4<f32>(dot(p - frame.view_pos.xyz, frame.view_forward.xyz), reflectivity, 0.0, 1.0);
  o.outline = vec4<f32>(0.0);
  return o;
}

@fragment
fn fs_prepass(in: TerrainOut) -> TerrainPrepass {
  // One resident-tile lookup for the hole and the maps, as fs.
  let s = (in.local.xz - terrain_min()) / terrain_cell();
  let first = resident_from(0u, s);
  if (terrain_hole_from(first)) { discard; }
  let point = terrain_point_from(first, s, false);
  // The layers' roughness as the lit pass shades it, for screen-space
  // reflections (a terrain is never metal): their weights alone.
  let layers = terrain_roughness(in.world, point.normal, point.control, dpdxFine(in.world), dpdyFine(in.world));
  let roughness = toksvig_roughness(layers.x, point.variance);
  return terrain_prepass_out(in.world, terrain_tilt(point, point.normal, in.relief.xy), point.normal, vec2<f32>(roughness, layers.y), in.cur_clip, in.prev_clip);
}

// ── Terrain stamps ──────────────────────────────────────────────────────
// A mesh drawn as part of the terrain (terrain.rs TerrainStampDraw): its
// own geometry, shaded by `terrain_look` at its world point with its own
// normal choosing the layers (the slope rules put rock on its faces and
// the ground's layers on its top), the control map read at the point's
// xz. Within `blend` metres above the ground under it the normal turns
// to the ground's, and the far grass tint comes in, so where it meets the
// terrain both are the same function of the same inputs: no seam (the
// runtime-virtual-texture blend without the texture). Never cut by the
// terrain's holes (a hole stamp cuts them under it).

@group(2) @binding(0) var<uniform> stamp: TerrainStampUniform;

struct StampIn {
  @location(0) position: vec3<f32>,
  @location(1) normal: vec3<f32>,
};

struct StampOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) cur_clip: vec4<f32>,
  @location(3) prev_clip: vec4<f32>,
};

@vertex
fn vs_stamp(v: StampIn) -> StampOut {
  let p = (stamp.model * vec4<f32>(v.position, 1.0)).xyz;
  var o: StampOut;
  o.clip = frame.view_proj * vec4<f32>(p, 1.0);
  o.world = p;
  o.normal = stamp.normal * v.normal;
  o.cur_clip = terrain.view_proj * vec4<f32>(p, 1.0);
  o.prev_clip = terrain.prev_view_proj * vec4<f32>(p, 1.0);
  return o;
}

// The ground under a stamp's fragment: its normal turned towards the
// ground's by the blend, the control channels there and the blend's share.
struct StampGround {
  normal: vec3<f32>,
  control: array<vec4<f32>, 4>,
  ground: f32,
  local: vec3<f32>,
  // The ground's point it reads (the terrain's space).
  reads: vec3<f32>,
};

// The ground it meets is read a cell and a half outwards along the mesh's
// normal across the terrain: under a wall's foot the field is carved
// below the mesh (og.terrain.stamp's depth), and the ground the eye sees
// meeting the wall is the uncarved ground just outside it. A face turned
// up reads the ground straight under it.
fn stamp_ground(p: vec3<f32>, n_mesh: vec3<f32>) -> StampGround {
  let local = (stamp.to_terrain * vec4<f32>(p, 1.0)).xyz;
  let n_local = (stamp.to_terrain * vec4<f32>(n_mesh, 0.0)).xyz;
  let out = n_local.xz / max(length(n_local), 1e-6) * 1.5 * terrain_cell();
  let s = (local.xz + out - terrain_min()) / terrain_cell();
  let m = terrain_maps_from(resident_from(0u, s), s, dpdxFine(s), dpdyFine(s), true);
  let above = (local.y - m.height) * stamp.params.y;
  var o: StampGround;
  // Wholly the ground over the band's lowest third, so the mesh's foot is
  // the ground it meets; then the mesh's own look takes over.
  let band = stamp.params.x;
  o.ground = select(0.0, 1.0 - smoothstep(band / 3.0, band, above), band > 0.0);
  o.normal = normalize(mix(n_mesh, normalize(terrain.normal * m.normal), o.ground));
  o.control = m.control;
  o.local = local;
  o.reads = vec3<f32>(local.x + out.x, local.y, local.z + out.y);
  return o;
}

@fragment
fn fs_stamp(in: StampOut) -> @location(0) vec4<f32> {
  let p = in.world;
  let dpx = dpdxFine(p);
  let dpy = dpdyFine(p);
  let g = stamp_ground(p, normalize(in.normal));
  let look = terrain_look(p, g.normal, g.normal, 0.0, g.control, dpx, dpy);
  let albedo = mix(look.albedo, grass_far(look.albedo, p, g.local, g.normal, g.control), g.ground);
  return terrain_shade(in.clip.xy, p, resident_from(0u, (g.reads.xz - terrain_min()) / terrain_cell()), g.ground, albedo, look.normal, look.roughness, g.normal, normalize(in.normal), look.height);
}

@fragment
fn fs_stamp_prepass(in: StampOut) -> TerrainPrepass {
  let p = in.world;
  let g = stamp_ground(p, normalize(in.normal));
  let roughness = terrain_roughness(p, g.normal, g.control, dpdxFine(p), dpdyFine(p));
  return terrain_prepass_out(p, g.normal, g.normal, roughness, in.cur_clip, in.prev_clip);
}
