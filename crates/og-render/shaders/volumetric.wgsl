// Volumetric fog (volumetric.rs), after frame.wgsl, the light tables,
// lighting.wgsl and area.wgsl. The medium: `density` per metre plus height
// fog (`height density` below `height`, thinning above it by `falloff`),
// scattering `colour × scattering` of the sun's and the local lights' light
// with a Henyey-Greenstein phase of `anisotropy`. The light that reaches it
// from every side is what the plain fog's colour stands for: a depth of the
// scene's fog nothing else lights tends to it (`own`), so the air of a night
// scene is the plain fog's haze, not black. A fog
// volume's colour is an albedo alone, so it takes that light from the
// environment (`ambient_in`: the environment's, the probe GI's where it
// reaches, the hemisphere light's), evenly.
//
// The froxel grid: `inject` lights every froxel (160 × 90 × 64 cells over the
// view, slices quadratically deeper to `distance`) by the sun through its
// cascades and the cluster's point and rect lights through their
// shadows and cookies; `integrate` sums them front to back into the
// in-scattered light and the transmittance at each slice's far edge;
// `fs_froxel` reads that at each pixel's depth and resolves spot cones on
// that pixel's ray, using the grid's extinction. The raymarch: `fs_raymarch`
// marches each pixel's view ray in steps, lit by the sun and the first
// RAYMARCH_LIGHTS lights of each step's cluster. Both
// output (in-scattered light, transmittance), blended over the lit image
// as colour × transmittance + light. Past `distance` the scene's fog goes
// on to each pixel's depth without the grid (`far_air`): its own colour and
// the unshadowed sun's light in it.
//
// Under water (`VOL_WATER`, volumetric.rs `Medium::Water`) the medium is the water the eye is in and the pass draws
// the sun's own light in it (`water_in`): the light shafts. The frame's fog has already drawn the water's even light
// and dimmed what is seen through it, so the transmittance it writes is 1.

// True in the water's pipelines: each medium compiles its own, so the fog's carries none of the water's code.
override VOL_WATER: bool = false;

// Fog inside a region (volumetric.rs `FogVolume`, og.FogVolume): its shape as og_env's `Volume::outside` reads it.
struct FogVolume {
  at: vec4<f32>,       // xyz: its entity's position (frame coordinates), w: a sphere's radius (0: the box, < 0: no bounds)
  turn: vec4<f32>,     // the quaternion that turns the frame's axes into its entity's
  half: vec4<f32>,     // xyz: the box's half size, w: the fade past its surface (m; 0: a hard edge)
  scatter: vec4<f32>,  // rgb: colour × scattering, w: extinction per metre inside it
};

const FOG_VOLUMES: u32 = 16u;

struct Vol {
  scatter: vec4<f32>,  // rgb: colour × scattering, w: anisotropy
  medium: vec4<f32>,   // x: density, y: height, z: height density, w: falloff
  extent: vec4<f32>,   // x: distance, y: raymarch steps, zw: the target's size in pixels
  grid: vec4<f32>,     // xyz: froxels across, down and deep, w: the frame's noise seed
  local: vec4<f32>,    // x: how many of `volumes` are set, y: far-air sky coverage, zw: distance fog start/end
  own: vec4<f32>,      // rgb: the scene fog's ambient (VolumetricFog::ambient), w: its maximum opacity
  volumes: array<FogVolume, FOG_VOLUMES>,
};

@group(1) @binding(0) var<uniform> vol: Vol;

// How much of a fog volume reaches p: 1 inside, falling linearly to 0 over its fade past its surface
// (og_env::volume::Volume::outside, term for term, then frame.wgsl's region_weight).
fn fog_volume_weight(v: FogVolume, p: vec3<f32>) -> f32 {
  if (v.at.w < 0.0) { return 1.0; }
  let d = p - v.at.xyz;
  var outside = max(length(d) - v.at.w, 0.0);
  if (v.at.w == 0.0) {
    let local = d + 2.0 * cross(v.turn.xyz, cross(v.turn.xyz, d) + v.turn.w * d);
    outside = length(max(abs(local) - v.half.xyz, vec3<f32>(0.0)));
  }
  return region_weight(outside, v.half.w);
}

// The medium at a point, per metre: the scene's fog and, added to it, each fog volume that reaches the point.
struct MediumAt {
  scatter: vec3<f32>,  // what it scatters of the sun's and the local lights' light (colour × scattering × extinction)
  sigma: f32,          // its extinction
  own: vec3<f32>,      // the scene fog's own light: the light from every side in it, as its colour holds it
  local: vec3<f32>,    // the fog volumes' share of `scatter`, which scatters the environment's light (`ambient_in`) too
};

// span.xy: the sample's interval in metres from the eye; span.z: the length
// its column is averaged over (one equal share of a froxel, or a march step).
// The distance contribution uses interval overlap, not a jittered point test.
fn vol_medium(p: vec3<f32>, dir: vec3<f32>, span: vec3<f32>) -> MediumAt {
  // The water's: its fog's, below its surface.
  if (VOL_WATER) {
    let sigma = select(0.0, max(frame.fog_distance.x, 1e-5), p.y < frame.fog_height.x);
    return MediumAt(vol.scatter.rgb * sigma, sigma, vec3<f32>(0.0), vec3<f32>(0.0));
  }
  var sigma = vol.medium.x + vol.medium.z * exp(-vol.medium.w * max(p.y - vol.medium.y, 0.0));
  // Keep the original arithmetic for start=0, no ramp and no opacity cap.
  if (vol.local.z > 0.0) {
    let overlap = max(span.y - max(span.x, vol.local.z), 0.0);
    sigma = vol.medium.x * overlap / max(span.z, 1e-20)
      + vol.medium.z * exp(-vol.medium.w * max(p.y - vol.medium.y, 0.0));
  }
  if (vol.local.w > vol.local.z || vol.own.w < 1.0) {
    sigma = fog_column(dir, span.xy) / max(span.z, 1e-20);
  }
  var m = MediumAt(vol.scatter.rgb * sigma, sigma, vol.own.rgb * sigma, vec3<f32>(0.0));
  let count = min(u32(vol.local.x), FOG_VOLUMES);
  for (var i = 0u; i < count; i++) {
    let v = vol.volumes[i];
    let reach = v.scatter.w * fog_volume_weight(v, p);
    m.sigma += reach;
    m.local += v.scatter.rgb * reach;
  }
  m.scatter += m.local;
  return m;
}

// The view depth of a slice edge at fraction f of the grid's depth.
fn slice_depth(f: f32) -> f32 {
  return vol.extent.x * f * f;
}

// The world direction of the view ray through uv (0..1, y down).
fn view_ray(uv: vec2<f32>) -> vec3<f32> {
  return pixel_ray(vec2<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0));
}

// The sun's light scattered towards the eye at p (dir: the view ray).
fn sun_in(p: vec3<f32>, dir: vec3<f32>) -> vec3<f32> {
  let l = normalize(frame.light_dir.xyz);
  var shadow = 1.0;
  if (frame.light_dir.w > 0.5) { shadow = cascade_shadow_lookup(p, l, 1.0, false); }
  return frame.light_color.rgb * shadow * hg_phase(dot(dir, l), vol.scatter.w);
}

// The sun's light in the water at p scattered towards the eye (dir: the view ray), as a sample `width` metres
// across holds it, per unit of the water's extinction: the sun's share of the water's own light there
// (frame.wgsl `underwater_sun` of `underwater_body`'s source, the fog colour falling off with depth), times what
// the waves make of the sun at p (caustics.wgsl `water_focus`, the light field that draws the caustics on the bed),
// the sun's shadow there, and a Henyey-Greenstein phase about the sun's refracted direction over an even one (water
// scatters forwards: brighter looking up the light). Summed along a view ray that is the light shafts: parallel
// beams along the refracted sun, each under the patch of surface that focuses it, cut where something shadows
// them, converging on the sun's place in Snell's window only as perspective makes parallel lines converge.
// (Light scattered once in a medium lit through a refracting surface: Nishita and Nakamae, "Method of displaying
// optical effects within water using accumulation buffer", SIGGRAPH 1994; Iwasaki, Dobashi and Nishita, "An
// efficient method for rendering underwater optical effects using graphics hardware", CGF 2002.)
fn water_in(p: vec3<f32>, dir: vec3<f32>, width: f32) -> vec3<f32> {
  let l = normalize(frame.light_dir.xyz);
  let depth = frame.fog_height.x - p.y;
  if (depth <= 0.0 || l.y <= 0.0) { return vec3<f32>(0.0); }
  var shadow = 1.0;
  if (frame.light_dir.w > 0.5) { shadow = cascade_shadow_lookup(p, l, 1.0, false); }
  let up_light = -refract(-l, vec3<f32>(0.0, 1.0, 0.0), 1.0 / 1.33);
  let phase = 4.0 * PI * hg_phase(dot(dir, up_light), vol.scatter.w);
  let light = underwater_sun() * exp(-frame.fog_height.y * depth) * phase * shadow * max(1.0 + water_focus(p, width), 0.0);
  return frame.fog_color.rgb * light;
}

// The light scattered towards the eye at p (frag: its pixel; `most`: as `locals_in`), a sample `width` metres
// across: the medium's, but for the light from every side (MediumAt's `own` and `local`).
// span.xy bounds the integration step along dir relative to p; span.z is its uniform sample.
fn medium_in(p: vec3<f32>, dir: vec3<f32>, frag: vec2<f32>, most: u32, width: f32, m: MediumAt, span: vec3<f32>, spots: bool) -> vec3<f32> {
  if (VOL_WATER) { return m.scatter * water_in(p, dir, width); }
  return m.scatter * sun_in(p, dir) + locals_in(p, dir, frag, most, m.scatter, span, spots);
}

// The environment's and the hemisphere light's light from every side about p, as an even phase scatters it
// towards the eye: their mean radiance. The mean over every direction of an irradiance over π is the mean
// radiance, and the six axes take that mean exactly for an irradiance's harmonics (bands 0 to 2: the first
// band's terms cancel between opposite axes, the second's sum to zero over the six). The same light the lit
// surfaces take along their normals (surface_shade.wgsl).
fn env_in(p: vec3<f32>) -> vec3<f32> {
  let x = vec3<f32>(1.0, 0.0, 0.0);
  let y = vec3<f32>(0.0, 1.0, 0.0);
  let z = vec3<f32>(0.0, 0.0, 1.0);
  let env = ambient_light_level(x, p) + ambient_light_level(-x, p) + ambient_light_level(y, p)
    + ambient_light_level(-y, p) + ambient_light_level(z, p) + ambient_light_level(-z, p);
  return env * (frame.params.x / 6.0) + 0.5 * (frame.hemi_sky.rgb + frame.hemi_ground.rgb) * frame.hemi_sky.w;
}

// The light from every side at p that a fog volume scatters evenly towards the eye: `env` (`env_in`) and, where
// the probe GI reaches, its light over the environment's as the lit surfaces take it (gi_probes.wgsl
// `gi_ambient_from`): the probes' harmonics' even term, what the scene around p sends it with its sky occluded,
// so the smoke in a closed room is as dark as the room. None in the water, whose even light the frame's fog draws.
fn ambient_in(p: vec3<f32>, env: vec3<f32>) -> vec3<f32> {
  if (VOL_WATER) { return vec3<f32>(0.0); }
  if (!OG_GI || probe_data.v[GI_HEADER_AT].x < 0.5) { return env; }
  // A point in the air has no surface: no normal to weigh the probes by and no bias off it.
  let s = gi_gather(p, vec3<f32>(0.0), vec3<f32>(0.0), true);
  if (s.w <= 0.0) { return env; }
  let hemi = 0.5 * (frame.hemi_sky.rgb + frame.hemi_ground.rgb) * frame.hemi_sky.w;
  return mix(env - hemi, max(vec3<f32>(s.r.w, s.g.w, s.b.w), vec3<f32>(0.0)), s.w) + hemi;
}

// The height fog's extinction summed over a stretch of ray `len` metres long between the heights `ya` and `yb`
// above the height it is densest below (vol_medium's law: constant below it, thinning by `falloff` above).
fn height_column(ya: f32, yb: f32, len: f32) -> f32 {
  let k = vol.medium.w;
  let lo = min(ya, yb);
  let hi = max(ya, yb);
  if (hi <= 0.0) { return vol.medium.z * len; }
  // The share of the stretch above the height, from max(lo, 0) to hi.
  let above = select(1.0, hi / max(hi - lo, 1e-6), lo < 0.0);
  let a = max(lo, 0.0);
  let rise = k * (hi - a);
  var mean = exp(-k * a);
  if (rise > 1e-4) { mean *= (1.0 - exp(-rise)) / rise; }
  return vol.medium.z * len * (above * mean + (1.0 - above));
}

// The plain fog's distance transmittance is exp(-density * max(t-start,0))
// times 1-clamp((t-start)/(end-start),0,1) when end>start (media.wgsl).
// Its optical-depth difference gives a column over any interval, including
// one crossing start or end. The floor represents full fog without infinities.
fn distance_column(span: vec2<f32>) -> f32 {
  var tau = vol.medium.x * max(span.y - max(span.x, vol.local.z), 0.0);
  if (vol.local.w > vol.local.z) {
    let clear = vec2<f32>(1.0) - clamp((span - vec2<f32>(vol.local.z)) / (vol.local.w - vol.local.z), vec2<f32>(0.0), vec2<f32>(1.0));
    tau += log(max(clear.x, 1e-20) / max(clear.y, 1e-20));
  }
  return tau;
}

// Only the global fog spends the opacity budget. The plain path caps the
// distance and height fog together; local FogVolumes are independent media.
fn fog_column(dir: vec3<f32>, span: vec2<f32>) -> f32 {
  let y = frame.view_pos.y - vol.medium.y;
  let len = max(span.y - span.x, 0.0);
  let tau = distance_column(span) + height_column(y + dir.y * span.x, y + dir.y * span.y, len);
  if (vol.own.w >= 1.0) { return tau; }
  let before = distance_column(vec2<f32>(0.0, span.x)) + height_column(y, y + dir.y * span.x, span.x);
  let budget = -log(max(1.0 - vol.own.w, 1e-20));
  return min(tau, max(budget - before, 0.0));
}

// The air past the grid along the view ray `dir` to the pixel's view depth `depth` (rgb: its in-scattered
// light, a: its transmittance): the scene's fog going on from `distance`, its own colour and the unshadowed sun's
// light in it (no cascade reaches there), summed in closed form. Behind the sky it is 5 km of it, as much as
// the fog covers the sky, and all of it under the horizon, which stands for land (the frame's fog: sky.wgsl
// `sky_fog`). The fog volumes end inside the grid.
fn far_air(dir: vec3<f32>, depth: f32) -> vec4<f32> {
  if (VOL_WATER) { return vec4<f32>(0.0, 0.0, 0.0, 1.0); }
  let sky = is_sky_depth(depth);
  let stretch = 1.0 / max(dot(dir, frame.view_forward.xyz), 1e-3);
  let t0 = vol.extent.x * stretch;
  let t1 = select(depth * stretch, t0 + 5000.0, sky);
  let len = t1 - t0;
  if (len <= 0.0) { return vec4<f32>(0.0, 0.0, 0.0, 1.0); }
  let y0 = frame.view_pos.y + dir.y * t0 - vol.medium.y;
  var tau = vol.medium.x * len + height_column(y0, y0 + dir.y * len, len);
  if (vol.local.z > 0.0) {
    tau = vol.medium.x * max(t1 - max(t0, vol.local.z), 0.0) + height_column(y0, y0 + dir.y * len, len);
  }
  if (vol.local.w > vol.local.z || vol.own.w < 1.0) { tau = fog_column(dir, vec2<f32>(t0, t1)); }
  var hidden = 1.0 - exp(-tau);
  if (sky) { hidden *= mix(vol.local.y, 1.0, smoothstep(0.0, 0.05, -dir.y)); }
  if (hidden <= 0.0) { return vec4<f32>(0.0, 0.0, 0.0, 1.0); }
  let l = normalize(frame.light_dir.xyz);
  let sun = vol.scatter.rgb * frame.light_color.rgb * hg_phase(dot(dir, l), vol.scatter.w);
  return vec4<f32>((sun + vol.own.rgb) * hidden, 1.0 - hidden);
}

// Intersect a step p + t*dir with the spot's finite cone. A spot can be much
// thinner than a depth stratum; drawing its one sample from this interval,
// weighted by the interval's share of the step, integrates that thin support
// instead of randomly lighting or missing the entire stratum.
fn spot_span(light: LocalLight, p: vec3<f32>, dir: vec3<f32>, span: vec2<f32>) -> vec2<f32> {
  let v = p - light.position_range.xyz;
  let vd = dot(v, dir);
  let sphere = vd * vd - dot(v, v) + light.position_range.w * light.position_range.w;
  if (sphere <= 0.0) { return vec2<f32>(0.0); }
  var lo = max(span.x, -vd - sqrt(sphere));
  var hi = min(span.y, -vd + sqrt(sphere));
  let va = dot(v, light.direction_slot.xyz);
  let da = dot(dir, light.direction_slot.xyz);
  // Select the forward half of the double cone before clipping its quadratic.
  if (da > 0.0) { lo = max(lo, -va / da); }
  else if (da < 0.0) { hi = min(hi, -va / da); }
  else if (va <= 0.0) { return vec2<f32>(0.0); }
  if (hi <= lo) { return vec2<f32>(0.0); }
  let c2 = light.cone.x * light.cone.x;
  let a = c2 - da * da;
  let b = c2 * vd - va * da;
  let c = c2 * dot(v, v) - va * va;
  if (a == 0.0) {
    if (b > 0.0) { hi = min(hi, -c / (2.0 * b)); }
    else if (b < 0.0) { lo = max(lo, -c / (2.0 * b)); }
    else if (c > 0.0) { return vec2<f32>(0.0); }
  } else {
    let discriminant = b * b - a * c;
    if (discriminant <= 0.0) {
      if (a > 0.0) { return vec2<f32>(0.0); }
    } else {
      let q = -b - select(-sqrt(discriminant), sqrt(discriminant), b >= 0.0);
      let r0 = min(q / a, c / q);
      let r1 = max(q / a, c / q);
      if (a > 0.0) { lo = max(lo, r0); hi = min(hi, r1); }
      else if (lo < r0) { hi = min(hi, r0); }
      else { lo = max(lo, r1); }
    }
  }
  return vec2<f32>(lo, hi);
}

// One local light's radiance scattered towards the eye, before the medium's tint.
fn local_in(light: LocalLight, p: vec3<f32>, dir: vec3<f32>) -> vec3<f32> {
  let kind = light.color_kind.w;
  let to_light = light.position_range.xyz - p;
  let d2 = max(dot(to_light, to_light), 1e-4);
  var atten = local_falloff(light, d2);
  let l = to_light * inverseSqrt(d2);
  if (kind < -0.5) {
    // A rect light as a small panel: its area, facing p.
    let side = cross(light.direction_slot.xyz, light.cone.xyz);
    let area = 4.0 * length(side);
    var facing = dot(l, normalize(side));
    if (kind < -1.5) { facing = abs(facing); }
    atten *= area * max(facing, 0.0);
    if (atten > 0.0) { atten *= rect_shadow(light, p, l); }
  } else {
    if (kind > 0.5) { atten *= spot_cone(light, l); }
    if (atten > 0.0) { atten *= local_shadow_lookup(light, p, l, false); }
  }
  if (atten <= 0.0) { return vec3<f32>(0.0); }
  return light.color_kind.rgb * atten * light_cookie(light, -l) * hg_phase(dot(dir, l), vol.scatter.w);
}

// The cluster's lights scattered towards the eye at p (frag: its pixel),
// the first `most` of them (the cluster keeps the brightest at the camera
// first): every one in the froxel grid, RAYMARCH_LIGHTS in the raymarch.
fn locals_in(p: vec3<f32>, dir: vec3<f32>, frag: vec2<f32>, most: u32, scatter: vec3<f32>, span: vec3<f32>, spots: bool) -> vec3<f32> {
  var sum = vec3<f32>(0.0);
  if (frame.params.z < 0.5) { return sum; }
  let word = cluster_word(cluster_at(frag, p));
  let first = word & 0xffffffu;
  let count = min(word >> 24u, most);
  for (var k = 0u; k < count; k++) {
    let light = light_at(cluster_ref(first + k));
    let kind = light.color_kind.w;
    if (kind > 0.5 && !spots) { continue; }
    var at = p;
    var weight = 1.0;
    var tint = scatter;
    if (kind > 0.5) {
      let interval = spot_span(light, p, dir, span.xy);
      if (interval.y <= interval.x) { continue; }
      at += dir * mix(interval.x, interval.y, span.z);
      weight = (interval.y - interval.x) / (span.y - span.x);
      let along = dot(p - frame.view_pos.xyz, dir);
      tint = vol_medium(at, dir, vec3<f32>(vec2<f32>(along) + interval, interval.y - interval.x)).scatter;
    }
    sum += tint * weight * local_in(light, at, dir);
  }
  return sum;
}

struct VolOut {
  @builtin(position) pos: vec4<f32>,
};

@vertex
fn vs_volumetric(@builtin(vertex_index) i: u32) -> VolOut {
  var o: VolOut;
  o.pos = fullscreen_clip(i);
  return o;
}

@group(1) @binding(4) var scene_depth: texture_2d<f32>;
fn pixel_depth(frag: vec2<f32>) -> f32 {
  let size = vec2<i32>(textureDimensions(scene_depth));
  let at = clamp(vec2<i32>(frag * vec2<f32>(size) / vol.extent.zw), vec2<i32>(0), size - 1);
  return textureLoad(scene_depth, at, 0).r;
}
