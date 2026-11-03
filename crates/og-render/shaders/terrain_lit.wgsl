// The terrain's lit, prepass and normal-map passes. Prepended by
// frame.wgsl, the light tables (clusters.rs), lighting.wgsl and terrain.wgsl.
//
// Holes (terrain.wgsl terrain_hole) are discarded in every pass.
//
// Shading: the normal from the terrain's normal map (built from the
// heights on the GPU, mipmapped), up to sixteen layers weighted by the
// control map's channels times each layer's slope and altitude rule, the
// four strongest at each pixel blended (all of them with four or fewer),
// optionally sharpened by the layers' heights (their albedo's alpha),
// each layer sampled by world xz (UV-scaled) or triplanar, lit like a
// StandardMaterial (cascaded shadows, local lights, image-based light),
// caustics under the frame's water, then fog.

struct TerrainOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world: vec3<f32>,
  @location(1) cur_clip: vec4<f32>,
  @location(2) prev_clip: vec4<f32>,
  // The point in the terrain's space (its maps are laid out there).
  @location(3) local: vec3<f32>,
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
  return o;
}

// The terrain's maps at a point of its space: its world normal and the
// control map's channels, from the finest resident tile.
struct TerrainPoint {
  normal: vec3<f32>,
  control: array<vec4<f32>, 4>,
};

fn terrain_point(local: vec3<f32>) -> TerrainPoint {
  let s = (local.xz - terrain_min()) / terrain_cell();
  let m = terrain_maps_at(s, dpdx(s), dpdy(s));
  var o: TerrainPoint;
  o.normal = normalize(terrain.normal * m.normal);
  o.control = m.control;
  return o;
}

fn layer_albedo(i: u32, uv: vec2<f32>, dx: vec2<f32>, dy: vec2<f32>) -> vec4<f32> {
  switch i {
    case 0u: { return textureSampleGrad(albedo0, layer_sampler, uv, dx, dy); }
    case 1u: { return textureSampleGrad(albedo1, layer_sampler, uv, dx, dy); }
    case 2u: { return textureSampleGrad(albedo2, layer_sampler, uv, dx, dy); }
    default: { return textureSampleGrad(albedo3, layer_sampler, uv, dx, dy); }
  }
}

fn layer_normal_map(i: u32, uv: vec2<f32>, dx: vec2<f32>, dy: vec2<f32>) -> vec3<f32> {
  var t: vec4<f32>;
  switch i {
    case 0u: { t = textureSampleGrad(normal0, layer_sampler, uv, dx, dy); }
    case 1u: { t = textureSampleGrad(normal1, layer_sampler, uv, dx, dy); }
    case 2u: { t = textureSampleGrad(normal2, layer_sampler, uv, dx, dy); }
    default: { t = textureSampleGrad(normal3, layer_sampler, uv, dx, dy); }
  }
  return t.xyz * 2.0 - 1.0;
}

// The weight of a rule over [lo, hi] faded over `fade` on both sides.
fn band(x: f32, lo: f32, hi: f32, fade: f32) -> f32 {
  let f = max(fade, 1e-4);
  return smoothstep(lo - f, lo, x) * (1.0 - smoothstep(hi, hi + f, x));
}

struct LayerSample {
  albedo: vec4<f32>,   // rgb colour, a: height
  normal: vec3<f32>,   // world normal with the layer's detail
};

// One layer at a point: planar over world xz, or triplanar (three
// projections blended by the normal, whiteout normal blending).
fn sample_layer(i: u32, p: vec3<f32>, n: vec3<f32>, dpx: vec3<f32>, dpy: vec3<f32>) -> LayerSample {
  let l = terrain.layers[i];
  let k = l.mapping.x;
  let has_albedo = (u32(l.mapping.w) & 1u) != 0u;
  let has_normal = (u32(l.mapping.w) & 2u) != 0u;
  var s: LayerSample;
  s.albedo = vec4<f32>(1.0);
  s.normal = n;
  if (l.mapping.y < 0.5) {
    // World xz from the frame's origin, plus the layer's uv there.
    let uv = p.xz * k + l.offset.xz;
    if (has_albedo) { s.albedo = layer_albedo(i, uv, dpx.xz * k, dpy.xz * k); }
    if (has_normal) {
      let m = layer_normal_map(i, uv, dpx.xz * k, dpy.xz * k);
      let t = normalize(vec3<f32>(1.0, 0.0, 0.0) - n * n.x);
      let b = normalize(vec3<f32>(0.0, 0.0, 1.0) - n * n.z);
      let d = vec3<f32>(m.xy * l.mapping.z, m.z);
      s.normal = normalize(t * d.x + b * d.y + n * d.z);
    }
  } else {
    var w = pow(abs(n), vec3<f32>(4.0));
    w = w / max(w.x + w.y + w.z, 1e-5);
    let ux = p.zy * k + l.offset.zy;
    let uy = p.xz * k + l.offset.xz;
    let uz = p.xy * k + l.offset.xy;
    if (has_albedo) {
      s.albedo = layer_albedo(i, ux, dpx.zy * k, dpy.zy * k) * w.x
        + layer_albedo(i, uy, dpx.xz * k, dpy.xz * k) * w.y
        + layer_albedo(i, uz, dpx.xy * k, dpy.xy * k) * w.z;
    }
    if (has_normal) {
      let st = l.mapping.z;
      var tx = layer_normal_map(i, ux, dpx.zy * k, dpy.zy * k);
      var ty = layer_normal_map(i, uy, dpx.xz * k, dpy.xz * k);
      var tz = layer_normal_map(i, uz, dpx.xy * k, dpy.xy * k);
      tx = vec3<f32>(tx.xy * st + n.zy, abs(tx.z) * n.x);
      ty = vec3<f32>(ty.xy * st + n.xz, abs(ty.z) * n.y);
      tz = vec3<f32>(tz.xy * st + n.xy, abs(tz.z) * n.z);
      s.normal = normalize(tx.zyx * w.x + ty.xzy * w.y + tz.xyz * w.z);
    }
  }
  return s;
}

// The four layers a pixel blends: the strongest raw weights (control map
// times the slope and altitude rules), ties to the lower layer, and those
// weights; with four layers or fewer, every layer. Where nothing shows
// (no weight past the height blend's floor, or a sum near 0), the first
// layer alone (`fallback`).
struct LayerPick {
  ids: vec4<u32>,
  w: vec4<f32>,
  n: u32,
  fallback: bool,
};

fn pick_layers(p: vec3<f32>, n: vec3<f32>, c: array<vec4<f32>, 4>) -> LayerPick {
  let count = min(u32(terrain.origin.w), 16u);
  let angle = degrees(acos(clamp(n.y, -1.0, 1.0)));
  let altitude = p.y - terrain.origin.y;
  var pick: LayerPick;
  pick.ids = vec4<u32>(0u, 1u, 2u, 3u);
  pick.w = vec4<f32>(-1.0);
  var top = 0.0;
  var sum = 0.0;
  for (var i = 0u; i < count; i++) {
    let l = terrain.layers[i];
    let w = c[i / 4u][i % 4u] * band(angle, l.slope.x, l.slope.y, l.slope.z) * band(altitude, l.altitude.x, l.altitude.y, l.altitude.z);
    top = max(top, w);
    sum += w;
    // Insert below every weight at least as large (ties keep the lower layer first).
    var at = 4u;
    for (var k = 0u; k < 4u; k++) {
      if (at == 4u && w > pick.w[k]) { at = k; }
    }
    if (at < 4u) {
      for (var k = 3u; k > at; k--) {
        pick.w[k] = pick.w[k - 1u];
        pick.ids[k] = pick.ids[k - 1u];
      }
      pick.w[at] = w;
      pick.ids[at] = i;
    }
  }
  pick.n = min(count, 4u);
  pick.w = max(pick.w, vec4<f32>(0.0));
  let hb = terrain.params.x;
  pick.fallback = select((sum < 1e-4), (top <= 1e-3), (hb > 0.0));
  if (pick.fallback) {
    pick.ids = vec4<u32>(0u, 1u, 2u, 3u);
    pick.w = vec4<f32>(1.0, 0.0, 0.0, 0.0);
    pick.n = min(count, 1u);
  }
  return pick;
}

// The picked layers' weights (summing to 1), sharpened by their heights.
fn blend_weights(pick: LayerPick, heights_of: vec4<f32>) -> vec4<f32> {
  var w = pick.w;
  let hb = terrain.params.x;
  if (hb > 0.0 && !pick.fallback) {
    var ha = vec4<f32>(-1e9);
    var top = -1e9;
    for (var k = 0u; k < 4u; k++) {
      if (k < pick.n && w[k] > 1e-3) {
        ha[k] = w[k] + heights_of[k] * hb;
        top = max(top, ha[k]);
      }
    }
    w = max(ha - vec4<f32>(top - 0.15), vec4<f32>(0.0));
  }
  let sum = w.x + w.y + w.z + w.w;
  if (sum < 1e-4) { return vec4<f32>(1.0, 0.0, 0.0, 0.0); }
  return w / sum;
}

// The blended layers at a fragment: albedo, shading normal, roughness.
struct TerrainLook {
  albedo: vec3<f32>,
  normal: vec3<f32>,
  roughness: f32,
};

fn terrain_look(in: TerrainOut, point: TerrainPoint, dpx: vec3<f32>, dpy: vec3<f32>) -> TerrainLook {
  let p = in.world;
  let n_geo = point.normal;
  let count = u32(terrain.origin.w);
  let pick = pick_layers(p, n_geo, point.control);
  // The picked layers' colours and normals (a layer of weight 0 skips its maps).
  var samples: array<LayerSample, 4>;
  var heights_of = vec4<f32>(1.0);
  for (var k = 0u; k < 4u; k++) {
    if (k < pick.n && (pick.w[k] > 0.0 || k == 0u)) {
      samples[k] = sample_layer(pick.ids[k], p, n_geo, dpx, dpy);
      heights_of[k] = samples[k].albedo.a;
    }
  }
  let w = blend_weights(pick, heights_of);
  var albedo = vec3<f32>(0.0);
  var normal = vec3<f32>(0.0);
  var roughness = 0.0;
  for (var k = 0u; k < 4u; k++) {
    if (k < pick.n && w[k] > 0.0) {
      let l = terrain.layers[pick.ids[k]];
      albedo += samples[k].albedo.rgb * l.color.rgb * w[k];
      normal += samples[k].normal * w[k];
      roughness += l.color.w * w[k];
    }
  }
  if (count == 0u) {
    albedo = vec3<f32>(0.5);
    normal = n_geo;
    roughness = 0.9;
  }
  return TerrainLook(albedo, normal, roughness);
}

// The toon ramp's three bands for the terrain (pbr.wgsl toon_ramp's built-in
// one), each edge smoothed over a pixel of t.
fn terrain_band(t: f32) -> f32 {
  let e = max(fwidth(t), 1e-4);
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
// field's colour from afar from `fade.x` on, whole from `fade.y`, so there
// is no seam where the blades thin out to nothing; per clump and dry
// clump as the blades are, paler where a gust presses the grass down.
fn grass_far(albedo: vec3<f32>, p: vec3<f32>, local: vec3<f32>, n: vec3<f32>, c: array<vec4<f32>, 4>) -> vec3<f32> {
  let g = terrain.grass;
  if (g.color.w < 0.0) { return albedo; }
  let amount = smoothstep(g.fade.x, g.fade.y, distance(p, frame.view_pos.xyz)) * g.fade.w * grass_layer_share(p, n, c, u32(g.color.w));
  if (amount <= 0.0) { return albedo; }
  let gust = grass_gust(g.wind, p.xz + frame.origin.xz, false);
  let sheen = 1.0 + g.fade.z * g.wind.dir.z * (1.4 * gust * gust - 0.35);
  let clump = grass_clump(local.xz - terrain_min(), g.clump.x, u32(g.clump.w));
  let v = g.clump.y;
  var col = g.color.rgb * mix(1.0 - 0.35 * v, 1.0 + 0.25 * v, clump.x);
  col = mix(col, g.dry.rgb, step(fract(clump.x * 7.31 + 0.17), g.clump.z) * 0.75);
  return mix(albedo, col * sheen, amount);
}

@fragment
fn fs(in: TerrainOut) -> @location(0) vec4<f32> {
  if (terrain_hole_at(in.local)) { discard; }
  let p = in.world;
  let dpx = dpdx(p);
  let dpy = dpdy(p);
  let point = terrain_point(in.local);
  let n_geo = point.normal;
  let look = terrain_look(in, point, dpx, dpy);
  let albedo = grass_far(look.albedo, p, in.local, n_geo, point.control);
  let normal = look.normal;
  let roughness = look.roughness;
  var n = normalize(normal);
  var s: Surface;
  s.albedo = albedo;
  s.metallic = 0.0;
  s.roughness = clamp(roughness, 0.04, 1.0);
  s.f0 = vec3<f32>(0.04);
  let weathered = weather_surface(s, n, n_geo, p);
  s = weathered.s;
  n = weathered.n;
  let v = select(normalize(frame.view_pos.xyz - p), -frame.view_forward.xyz, frame.view_pos.w > 0.5);

  let l = normalize(frame.light_dir.xyz);
  let n_dot_l = max(dot(n, l), 0.0);
  var shadow = 1.0;
  if (frame.light_dir.w > 0.5 && n_dot_l > 0.0) { shadow = cascade_shadow(p, n_geo, n_dot_l); }
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
  color += surface_local_lights(in.clip.xy, s, p, n, v);
  let n_dot_v = max(dot(n, v), 1e-4);
  let f_ibl = f_schlick_roughness(n_dot_v, s.f0, s.roughness);
  let kd_ibl = vec3<f32>(1.0) - f_ibl;
  // The environment, or the probe holding the tile blended with it (probes.rs),
  // at its intensity, plus the hemisphere light at its own.
  let irradiance = ambient_light(n, p) * frame.params.x + hemi_irradiance(n);
  let r = reflect(-v, n);
  let prefiltered = reflected_light(r, p, s.roughness * frame.params.y) * frame.params.x;
  let lut = textureSample(brdf_lut, ibl_sampler, vec2<f32>(n_dot_v, s.roughness)).rg;
  color += kd_ibl * irradiance * s.albedo + prefiltered * (f_ibl * lut.x + lut.y);

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

@fragment
fn fs_prepass(in: TerrainOut) -> TerrainPrepass {
  if (terrain_hole_at(in.local)) { discard; }
  let point = terrain_point(in.local);
  let n = point.normal;
  // The layers' roughness as the lit pass shades it, for screen-space
  // reflections (a terrain is never metal).
  let look = terrain_look(in, point, dpdx(in.world), dpdy(in.world));
  let cur = in.cur_clip.xy / in.cur_clip.w;
  let prev = in.prev_clip.xy / max(in.prev_clip.w, 1e-6);
  var o: TerrainPrepass;
  o.gbuffer = vec4<f32>(oct_encode(n), (cur - prev) * vec2<f32>(0.5, -0.5));
  let reflectivity = reflectivity_bits(clamp(look.roughness, 0.04, 1.0), 0.0);
  o.depth = vec4<f32>(dot(in.world - frame.view_pos.xyz, frame.view_forward.xyz), reflectivity, 0.0, 1.0);
  o.outline = vec4<f32>(0.0);
  return o;
}
