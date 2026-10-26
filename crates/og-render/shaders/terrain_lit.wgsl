// The terrain's lit, prepass and normal-map passes. Prepended by
// frame.wgsl, the light tables (clusters.rs), lighting.wgsl and terrain.wgsl.
//
// Shading: the normal from the terrain's normal map (built from the
// heights on the GPU, mipmapped), up to four layers weighted by the
// control map's channels times each layer's slope and altitude rule,
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
  control: vec4<f32>,
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
    let uv = p.xz * k;
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
    let ux = p.zy * k;
    let uy = p.xz * k;
    let uz = p.xy * k;
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

// The layers' weights at a point (summing to 1): control map times the
// slope and altitude rules, sharpened by the layers' heights. `p` and `n`
// are in the world, `c` the control map's channels there.
fn layer_weights(p: vec3<f32>, n: vec3<f32>, c: vec4<f32>, heights_of: vec4<f32>) -> vec4<f32> {
  let count = u32(terrain.origin.w);
  let angle = degrees(acos(clamp(n.y, -1.0, 1.0)));
  let altitude = p.y - terrain.origin.y;
  var w = vec4<f32>(0.0);
  for (var i = 0u; i < 4u; i++) {
    if (i >= count) { break; }
    let l = terrain.layers[i];
    w[i] = c[i] * band(angle, l.slope.x, l.slope.y, l.slope.z) * band(altitude, l.altitude.x, l.altitude.y, l.altitude.z);
  }
  let hb = terrain.params.x;
  if (hb > 0.0) {
    var ha = vec4<f32>(-1e9);
    var top = -1e9;
    for (var i = 0u; i < 4u; i++) {
      if (w[i] > 1e-3) {
        ha[i] = w[i] + heights_of[i] * hb;
        top = max(top, ha[i]);
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
  // Each layer's colour and normal (layers of weight 0 skip their maps).
  var samples: array<LayerSample, 4>;
  var heights_of = vec4<f32>(1.0);
  for (var i = 0u; i < 4u; i++) {
    if (i < count) {
      samples[i] = sample_layer(i, p, n_geo, dpx, dpy);
      heights_of[i] = samples[i].albedo.a;
    }
  }
  let w = layer_weights(p, n_geo, point.control, heights_of);
  var albedo = vec3<f32>(0.0);
  var normal = vec3<f32>(0.0);
  var roughness = 0.0;
  for (var i = 0u; i < 4u; i++) {
    if (i < count && w[i] > 0.0) {
      let l = terrain.layers[i];
      albedo += samples[i].albedo.rgb * l.color.rgb * w[i];
      normal += samples[i].normal * w[i];
      roughness += l.color.w * w[i];
    }
  }
  if (count == 0u) {
    albedo = vec3<f32>(0.5);
    normal = n_geo;
    roughness = 0.9;
  }
  return TerrainLook(albedo, normal, roughness);
}

@fragment
fn fs(in: TerrainOut) -> @location(0) vec4<f32> {
  let p = in.world;
  let dpx = dpdx(p);
  let dpy = dpdy(p);
  let point = terrain_point(in.local);
  let n_geo = point.normal;
  let look = terrain_look(in, point, dpx, dpy);
  let albedo = look.albedo;
  let normal = look.normal;
  let roughness = look.roughness;
  var n = normalize(normal);
  var s: Surface;
  s.albedo = albedo;
  s.metallic = 0.0;
  s.roughness = clamp(roughness, 0.04, 1.0);
  s.f0 = vec3<f32>(0.04);
  let v = select(normalize(frame.view_pos.xyz - p), -frame.view_forward.xyz, frame.view_pos.w > 0.5);

  let l = normalize(frame.light_dir.xyz);
  let n_dot_l = max(dot(n, l), 0.0);
  var shadow = 1.0;
  if (frame.light_dir.w > 0.5 && n_dot_l > 0.0) { shadow = cascade_shadow(p, n_geo, n_dot_l); }
  var color = brdf(s, n, v, l) * frame.light_color.rgb * shadow;
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
    let c = cascade_index(p);
    var tint = vec3<f32>(0.0);
    if (c == 0u) { tint = vec3<f32>(1.0, 0.0, 0.0); }
    if (c == 1u) { tint = vec3<f32>(0.0, 1.0, 0.0); }
    if (c == 2u) { tint = vec3<f32>(0.0, 0.0, 1.0); }
    return vec4<f32>(tint * 4.0, 1.0);
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
