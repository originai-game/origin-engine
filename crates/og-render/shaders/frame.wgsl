// Per-frame bindings shared by the passes that draw into the HDR target
// (pbr.wgsl, sky.wgsl, sprite.wgsl): the camera, the directional light and
// its cascades, the shadow maps and the image-based lighting. The renderer
// prepends this file to those shaders; the Rust side is `FrameUniform` in
// frame.rs. The local lights and their clusters (bindings 7 to 9) are
// declared after it in the lit shader only (clusters.rs).

const CASCADES: u32 = 4u;
const LOCAL_SHADOW_LAYERS: u32 = 96u;

// Magenta/black checker: this surface has no implementation of the selected
// diagnostic. Never substitute beauty shading for a missing diagnostic term.
fn debug_unsupported(pixel: vec2<f32>) -> vec4<f32> {
  let cell = vec2<u32>(pixel) / 8u;
  let bright = ((cell.x + cell.y) & 1u) == 0u;
  return vec4<f32>(select(vec3<f32>(0.0), vec3<f32>(1.0, 0.0, 1.0), bright), 1.0);
}

struct LocalLight {
  position_range: vec4<f32>,  // xyz: position, w: range
  color_kind: vec4<f32>,      // rgb: color * intensity, w: 0 point, 1 spot
  direction_slot: vec4<f32>,  // xyz: spot axis (away from the light), w: first shadow layer (-1: none)
  cone: vec4<f32>,            // x: cos outer, y: 1 / (cos inner - cos outer), z: normal offset per metre, w: depth bias
};

struct Frame {
  view_proj: mat4x4<f32>,     // jittered when TAA is on
  inv_view_proj: mat4x4<f32>,
  view_pos: vec4<f32>,        // xyz: camera position, w: 1 orthographic, 0 perspective
  light_dir: vec4<f32>,       // xyz: towards the light, w: cascades in use (0: no shadows)
  light_color: vec4<f32>,     // rgb: radiance, w: debug view (DebugView::lit_code: 1 cascades, 2 AO, 3 albedo, 4 IBL diffuse, 5 IBL specular, 6 direct)
  params: vec4<f32>,          // x: ibl intensity, y: prefiltered mips - 1, z: local lights drawn, w: 1 / cascade map size
  view_forward: vec4<f32>,    // xyz: camera forward, w: 1 / local shadow map size
  cascade_view_proj: array<mat4x4<f32>, CASCADES>,
  // x: normal offset (world), y: depth bias (normalized), z: how far from cascade_anchor it shades, w: where it
  // fades into the next (from the anchor; z: no blending; negative: a fixed shadow box, every receiver compared
  // towards the light)
  cascade_bias: array<vec4<f32>, CASCADES>,
  cascade_anchor: vec4<f32>,  // xyz: the point the cascades are nested round (shadow_fit.rs), the same in every view
  cluster: vec4<f32>,         // x, y: clusters per pixel across and down, z, w: depth slice scale and bias (of log2 depth in perspective)
  local_view_proj: array<mat4x4<f32>, LOCAL_SHADOW_LAYERS>,  // per layer: a spot's view or a cube face (paraboloid layers: unused)
  // The atmosphere (`AtmosphereGpu` in atmosphere.rs).
  fog_color: vec4<f32>,       // rgb: colour, w: 1 when fog is on, 2 under water (`underwater_light`: fog_height is x the surface's height, y the daylight's attenuation per metre; fog_sun.w the shafts' strength)
  fog_distance: vec4<f32>,    // x: density, y: start, z: end, w: max opacity
  fog_height: vec4<f32>,      // x: height, y: falloff, z: height density, w: sky amount
  fog_sun: vec4<f32>,         // rgb: scattering colour, w: power
  sky_zenith: vec4<f32>,      // rgb, w: 1 when the sky pass draws this look
  sky_horizon: vec4<f32>,     // rgb, w: brightness
  sky_ground: vec4<f32>,      // rgb, w: stars
  sky_sun: vec4<f32>,         // x: disc outer cos, y: inner cos, z: disc intensity, w: glow
  sky_glow: vec4<f32>,        // x: glow power, yzw: sun colour
  sky_sun_dir: vec4<f32>,     // xyz: towards the drawn sun
  sky_moon: vec4<f32>,        // xyz: towards the moon, w: its angular radius (0: none)
  sky_moon_look: vec4<f32>,   // rgb: the lit moon's colour, w: phase (0 new, 0.5 full)
  time: vec4<f32>,            // x: seconds, y: tick, z: ticks per second, w: previous drawn seconds
  sky_scatter: array<vec4<f32>, 38>,  // the physical sky (sky_scatter.wgsl; [0].w 0: the gradient)
  clouds: array<vec4<f32>, 10>,  // the clouds (clouds.rs Clouds::gpu; [3].w 0: none; [4].w 1: captured in IBL; [6..10] the clear sky's light)
  aurora: array<vec4<f32>, 5>,  // the aurora (aurora.rs Aurora::gpu; [0].y 0: none)
  weather: vec4<f32>,         // weather_surface.wgsl: x wetness, y puddles, z snow cover, w rain ripples
  weather_look: vec4<f32>,    // rgb: snow colour, w: puddle size (m)
  weather_dust: vec4<f32>,    // rgb: dust colour, w: dust cover
  weather_depth: vec4<f32>,   // x: lying snow (m)
  weather_view_proj: mat4x4<f32>,  // the weather occlusion map's view (weather.rs shelter_view)
  weather_shelter: vec4<f32>,  // x: its cascade_map layer + 1 (0: none), y: drip edge (texels), z: map side (texels), w: depth bias
  // Water (`water.rs`): caustics on what lies below the first surface that has them.
  caustics: vec4<f32>,        // x: the surface's height, y: strength (0: none), z: cells per metre, w: fade per metre of depth
  caustics_area: vec4<f32>,   // xy: the surface's min xz, zw: its max xz
  caustics_cells: array<vec4<f32>, 2>,  // [0].xy: the frame's origin in the ripples' tile (cells); a sea: its grid, [1].w 1
  // W1: the shore a sea's surf wets (water.rs caustics; weather_surface.wgsl shore_wetness).
  shore_wet: vec4<f32>,       // x: the sea's rest level, y: how high above it the surf wets (m), z: how wet (0: none)
  shore_area: vec4<f32>,      // its seabed's xz rectangle: xy min, zw max
  shadow: vec4<f32>,          // x: the sun's softness (0: hard), y: 1 when point lights' shadows are paraboloids, z: how much of the sun's shadow the cover scattered away (0: none), w: 1 + the drawn frame of 16 under TAA (0: not temporal)
  // Stylisation (stylise.rs): the retro-3D vertex path and the material style.
  stylise: vec4<f32>,         // xy: the vertex snap grid in cells (0: none), z: affine texturing, w: depth bits (0: full)
  shadow_tint: vec4<f32>,     // rgb: the tint shadowed light moves towards, w: amount (0: off)
  outline_style: vec4<f32>,   // x: the distance at which outlines are their width (0: constant), y: shadow darkness (0: off), z: 1 when the style is toon (the terrain's bands)
  style_line: vec4<f32>,      // rgb: the style's outline colour, w: its width in pixels (0: none), for surfaces without a material (the terrain)
  // A reflection probe (probes.rs): set in the frame group bound for a
  // draw inside the probe's box, whose irradiance_map is then the
  // environment's prefiltered cube and prefiltered_map the probe's.
  probe_min: vec4<f32>,       // xyz: the box's min corner, w: 1 with a probe
  probe_max: vec4<f32>,       // xyz: the box's max corner, w: the probe's intensity
  probe_pos: vec4<f32>,       // xyz: the capture point, w: the blend distance
  // The camera's axes in the world: billboards face the camera of the
  // group they are drawn with (sprite.wgsl face_axes).
  view_right: vec4<f32>,      // xyz: right
  view_up: vec4<f32>,         // xyz: up, w: 1 - the background's scale (a probe face: the environment's intensity; 0 elsewhere)
  hemi_sky: vec4<f32>,        // rgb: the hemisphere light's sky colour, w: intensity / pi (0: off)
  hemi_ground: vec4<f32>,     // rgb: its ground colour
  material_surface: vec4<f32>,  // the material style's surface: x 0 none, 1 clay, 2 paper, 3 plastic; y strength; z scale (m)
  origin: vec4<f32>,          // xyz: where the zero of the frame's coordinates is in the world (origin.rs)
  // The grounded dome (GroundProjection::gpu): [0] xyz its centre, w its height above the ground (0: none); [1].x its radius.
  env_dome: array<vec4<f32>, 2>,
  foliage_wind: array<array<vec4f, 9>, 2>, // current, previous drawn; FoliageWind
  // The material style's toon model (stylise.rs MaterialStyle): x the band
  // edges' softness in pixels (0: hard steps), y how smooth a toon
  // material's ambient light stays (0 one flat colour, 1 the PBR ambient),
  // zw unused.
  toon: vec4<f32>,
  ssr: vec4<f32>,             // screen-space reflections (ssr.rs): x 1 when ssr_tex holds this camera's, y intensity
  drawn: vec4<u32>,           // x: the renderer's frames drawn before this one (it counts on while the clock is paused), yzw unused
};

// The hemisphere light (three.js HemisphereLight) as irradiance in the IBL's units: added beside the environment's
// irradiance at its own strength (not the environment's intensity), so it reaches only the diffuse term (no
// reflection) and is scaled, like it, by AO and the material's envMapIntensity.
fn hemi_irradiance(n: vec3<f32>) -> vec3<f32> {
  return mix(frame.hemi_ground.rgb, frame.hemi_sky.rgb, 0.5 * n.y + 0.5) * frame.hemi_sky.w;
}

@group(0) @binding(0) var<uniform> frame: Frame;

// Whether a material's shader parameters changed in this frame: `stamp` is the drawn frame its uniform's `custom`
// was written in (material.wgsl `MaterialFactors.stamp`), so `previous_shader_params()` (hooks.rs) reads
// `previous_custom` in that frame only and the parameters' motion stops on the next one without another write. The
// casters' modules, which draw no motion, define it false (shadow_pass.wgsl).
fn params_moved(stamp: u32) -> bool {
  return stamp == frame.drawn.x;
}

// Shadow depths stored in R32Float colour targets and read with textureLoad
// (lighting.wgsl's explicit compares).
@group(0) @binding(1) var cascade_map: texture_2d_array<f32>;
@group(0) @binding(2) var local_shadow_map: texture_2d_array<f32>;
@group(0) @binding(3) var irradiance_map: texture_cube<f32>;
@group(0) @binding(4) var prefiltered_map: texture_cube<f32>;
@group(0) @binding(5) var brdf_lut: texture_2d<f32>;
@group(0) @binding(6) var ibl_sampler: sampler;
// Screen-space reflections (ssr.rs): rgb the reflected radiance over the
// share of the lobe the screen resolved, a that share. A storage texture, so it takes none of the lit pipelines'
// sixteen texture slots.
@group(0) @binding(13) var ssr_tex: texture_storage_2d<rgba16float, read>;

// The radiance a surface at `p` drawn at fragment `clip` reflects:
// `prefiltered` (the probes, the probe GI and the environment) with the
// main camera's screen-space reflections over it where they resolved the
// surface's lobe. Only the opaque surface the camera drew at the pixel
// reads them: one more than a pixel and a half off where `p` projects lies
// over another (a blended surface), and other cameras have none
// (`frame.ssr.x` 0).
fn ssr_reflected(prefiltered: vec3<f32>, p: vec3<f32>, clip: vec2<f32>) -> vec3<f32> {
  if (frame.ssr.x < 0.5) { return prefiltered; }
  let size = vec2<i32>(textureDimensions(ssr_tex));
  let c = frame.view_proj * vec4<f32>(p, 1.0);
  let px = vec2<f32>(c.x / c.w * 0.5 + 0.5, 0.5 - c.y / c.w * 0.5) * vec2<f32>(size);
  if (c.w <= 0.0 || any(abs(px - clip) > vec2<f32>(1.5))) { return prefiltered; }
  let s = textureLoad(ssr_tex, clamp(vec2<i32>(clip), vec2<i32>(0), size - 1));
  let k = frame.ssr.y;
  return prefiltered * (1.0 - s.a * k) + s.rgb * k;
}

// How much of the probe's light reaches `p`: 1 inside its box farther
// than the blend distance from every face, falling to 0 at the faces.
fn probe_weight(p: vec3<f32>) -> f32 {
  let d = min(p - frame.probe_min.xyz, frame.probe_max.xyz - p);
  return clamp(min(d.x, min(d.y, d.z)) / max(frame.probe_pos.w, 1e-3), 0.0, 1.0);
}

// The probe's cube direction for a ray from `p` along `r`: where the ray
// leaves the box, seen from the capture point (box projection).
fn probe_direction(r: vec3<f32>, p: vec3<f32>) -> vec3<f32> {
  let safe = select(r, vec3<f32>(1e-6), abs(r) < vec3<f32>(1e-6));
  let f = max((frame.probe_max.xyz - p) / safe, (frame.probe_min.xyz - p) / safe);
  let t = max(min(f.x, min(f.y, f.z)), 0.0);
  return p + r * t - frame.probe_pos.xyz;
}

// The environment's diffuse light along `n` at `p`: the irradiance cube,
// or with a probe the roughest prefiltered level of the environment and of
// the probe, blended by `probe_weight`; where the probe GI reaches
// (gi_probes.rs), its light over that, from its gather `s` there.
fn ambient_light_from(n: vec3<f32>, p: vec3<f32>, s: GiSample) -> vec3<f32> {
  if (frame.probe_min.w < 0.5) { return gi_ambient_from(textureSample(irradiance_map, ibl_sampler, n).rgb, s, n); }
  let top = frame.params.y;
  let env = textureSampleLevel(irradiance_map, ibl_sampler, n, top).rgb;
  let probe = textureSampleLevel(prefiltered_map, ibl_sampler, n, top).rgb * frame.probe_max.w;
  return gi_ambient_from(mix(env, probe, probe_weight(p)), s, n);
}

// ambient_light_from with the probe GI's own gather at `p`.
fn ambient_light(n: vec3<f32>, p: vec3<f32>) -> vec3<f32> {
  return ambient_light_from(n, p, gi_sample_at(p, n));
}

// `ambient_light` without screen derivatives (level 0 of the irradiance
// cube): for shaders that sample it after a fragment may have returned.
fn ambient_light_level(n: vec3<f32>, p: vec3<f32>) -> vec3<f32> {
  if (frame.probe_min.w < 0.5) { return textureSampleLevel(irradiance_map, ibl_sampler, n, 0.0).rgb; }
  let top = frame.params.y;
  let env = textureSampleLevel(irradiance_map, ibl_sampler, n, top).rgb;
  let probe = textureSampleLevel(prefiltered_map, ibl_sampler, n, top).rgb * frame.probe_max.w;
  return mix(env, probe, probe_weight(p));
}

// The environment reflected along `r` at `p` at prefiltered level `mip`:
// with a probe its box-projected capture blended with the environment.
// Where the environment is looked up for the ray from `eye` along unit
// `dir`: along it, or on a grounded dome (three.js GroundedSkybox) from the
// dome's centre towards where the ray leaves the dome or meets its flat
// ground. The sky pass draws the background through it and every
// reflection of the environment reads it, so a mirror shows the floor the
// frame shows around it.
fn dome_lookup(eye: vec3<f32>, dir: vec3<f32>) -> vec3<f32> {
  let h = frame.env_dome[0].w;
  if (h <= 0.0) { return dir; }
  let o = eye - frame.env_dome[0].xyz;
  let r = frame.env_dome[1].x;
  let b = dot(o, dir);
  let disc = b * b - (dot(o, o) - r * r);
  if (disc < 0.0) { return dir; }
  // The far root: from inside the dome, where the ray leaves it.
  var t = -b + sqrt(disc);
  if (t <= 0.0) { return dir; }
  if (dir.y < 0.0) {
    let tg = (-h - o.y) / dir.y;
    if (tg > 0.0) { t = min(t, tg); }
  }
  return normalize(o + dir * t);
}

fn reflected_light(r_in: vec3<f32>, p: vec3<f32>, mip: f32) -> vec3<f32> {
  let r = dome_lookup(p, r_in);
  if (frame.probe_min.w < 0.5) { return textureSampleLevel(prefiltered_map, ibl_sampler, r, mip).rgb; }
  let env = textureSampleLevel(irradiance_map, ibl_sampler, r, mip).rgb;
  let probe = textureSampleLevel(prefiltered_map, ibl_sampler, probe_direction(r_in, p), mip).rgb * frame.probe_max.w;
  return mix(env, probe, probe_weight(p));
}

// The prepass's reflectivity (lindepth's y, read by ssr.wgsl
// `ssr_surface`): the gloss (1 - roughness) in 255ths, the metalness in
// 255ths times 256 above it. The standard prepass, terrain and scattered
// copies write it.
fn reflectivity_bits(roughness: f32, metalness: f32) -> f32 {
  return round((1.0 - clamp(roughness, 0.0, 1.0)) * 255.0) + 256.0 * round(clamp(metalness, 0.0, 1.0) * 255.0);
}

// Fog: distance (exponential from start, or a linear ramp start..end) and
// exponential height fog integrated along the view ray, mixed towards the
// fog colour, which turns towards the sun's scattering colour looking into
// the sun. Off (fog_color.w 0): the colour is returned as it came.
fn fog_amount(p: vec3<f32>) -> vec4<f32> {
  return fog_amount_scaled(p, 1.0);
}

// `fog_amount` with the height fog's column scaled by `height` (the clouds
// take it as the sky does, sky.wgsl cloud_view_fog).
fn fog_amount_scaled(p: vec3<f32>, height: f32) -> vec4<f32> {
  let ortho = frame.view_pos.w > 0.5;
  // The ray's origin: the eye, or the pixel's own point on the near plane (orthographic).
  var origin = frame.view_pos.xyz;
  if (ortho) { origin = p - frame.view_forward.xyz * dot(p - frame.view_pos.xyz, frame.view_forward.xyz); }
  let to = p - origin;
  let d = length(to);
  let dir = select(frame.view_forward.xyz, to / max(d, 1e-6), d > 1e-6);
  let fd = frame.fog_distance;
  var clear = exp(-fd.x * max(d - fd.y, 0.0));
  if (fd.z > fd.y) { clear *= 1.0 - clamp((d - fd.y) / (fd.z - fd.y), 0.0, 1.0); }
  let fh = frame.fog_height;
  if (fh.z > 0.0) {
    let k = fh.y;
    let y0 = origin.y - fh.x;
    let dy = to.y;
    var column = fh.z * exp(-k * y0) * d;
    if (abs(k * dy) > 1e-4) { column = fh.z * exp(-k * y0) * (1.0 - exp(-k * dy)) / (k * dy) * d; }
    clear *= exp(-max(column, 0.0) * height);
  }
  let amount = min(1.0 - clear, fd.w);
  if (frame.fog_color.w > 1.5) { return vec4<f32>(underwater_light(origin, dir, d, amount), amount); }
  let sun = pow(max(dot(dir, normalize(frame.light_dir.xyz)), 0.0), max(frame.fog_sun.w, 1e-3));
  return vec4<f32>(mix(frame.fog_color.rgb, frame.fog_sun.rgb, sun), amount);
}

// The mean of the shafts' pattern (`underwater_body`), and how far over the water's even light a shaft at its
// brightest stands.
const SHAFT_MEAN: f32 = 0.165;
const SHAFT_GAIN: f32 = 1.4;

// A smooth value noise on the integer lattice (the light shafts' pattern below).
fn shaft_noise(p: vec2<f32>) -> f32 {
  let i = floor(p);
  let f = p - i;
  let u = f * f * (3.0 - 2.0 * f);
  let c = vec2<i32>(i);
  return mix(mix(lattice_hash(vec3<i32>(c, 5)), lattice_hash(vec3<i32>(c + vec2<i32>(1, 0), 5)), u.x),
             mix(lattice_hash(vec3<i32>(c + vec2<i32>(0, 1), 5)), lattice_hash(vec3<i32>(c + vec2<i32>(1, 1), 5)), u.x), u.y);
}

// The underwater view's light (fog_color.w 2, water.rs `underwater_fog`) on a ray from `origin` along `dir` for `d`
// metres, as the colour the fog's `amount` mixes in. The fog colour is the water looking level just under the surface.
// Daylight falls off with depth (fog_height: x the surface's height, y its attenuation per metre), so the light a ray
// gathers is the integral of e^{-K depth(s)} e^{-c s} over it (the single-scattering water body of Premoze and
// Ashikhmin, "Rendering natural waters", 2001): brighter looking up than down, and darker the deeper the eye.
// The sun's light comes down in shafts (fog_sun.w their strength), below.
fn underwater_light(origin: vec3<f32>, dir: vec3<f32>, d: f32, amount: f32) -> vec3<f32> {
  return underwater_body(origin, dir, d, amount, frame.fog_sun.w);
}

// `underwater_light` with the shafts at `shafts` of their strength (0: the water's even light alone, as the surface
// mirrors it from below: water.wgsl).
//
// The shafts are parallel, along the sun's refracted direction, so the eye sees them as lines through that
// direction's vanishing point: a pattern of the view ray's bearing around it alone, read as a noise on a circle
// (seamless all the way round), two drifting layers bright only where both are, a third splitting each beam into
// finer ones. That is what a march through the
// surface's light pattern converges to, at three noise reads and with no bars from too few steps. The eye's own
// position slides the pattern, so the shafts stay in the water as it moves. They fade at the vanishing point, where
// the eye looks along one shaft, lose contrast looking across them, where many overlap, and blur into the water's
// even light with depth.
fn underwater_body(origin: vec3<f32>, dir: vec3<f32>, d: f32, amount: f32, shafts: f32) -> vec3<f32> {
  let c = max(frame.fog_distance.x, 1e-5);
  let k = frame.fog_height.y;
  let depth0 = max(frame.fog_height.x - origin.y, 0.0);
  let slant = max(c - k * dir.y, 0.2 * c);
  let reach = 1.0 - exp(-slant * d);
  var colour = frame.fog_color.rgb * exp(-k * depth0) * reach * c / (slant * max(amount, 1e-4));
  let l = normalize(frame.light_dir.xyz);
  if (shafts > 0.0 && l.y > 0.0) {
    // Never more than 49 degrees off the vertical, so the x axis is a safe second axis.
    let t = refract(-l, vec3<f32>(0.0, 1.0, 0.0), 1.0 / 1.33);
    let u = normalize(cross(t, vec3<f32>(1.0, 0.0, 0.0)));
    let v = cross(t, u);
    let e = vec2<f32>(dot(dir, u), dot(dir, v));
    let off = length(e);
    let bearing = e / max(off, 1e-4);
    let eye = vec2<f32>(dot(origin, u), dot(origin, v));
    let time = frame.time.x;
    let a = shaft_noise(bearing * 6.0 + eye * 0.23 + vec2<f32>(time * 0.07, -time * 0.05));
    let b = shaft_noise(bearing * 17.0 + eye * 0.41 + vec2<f32>(-time * 0.09, time * 0.11) + 7.3 + off * 1.3);
    let f = shaft_noise(bearing * 47.0 + eye * 0.97 + vec2<f32>(time * 0.13, time * 0.08) + 3.1 - off * 2.1);
    // The fine layer splits each beam into the pencils of light the ripples make of it (its mean kept: SHAFT_MEAN).
    let pattern = smoothstep(0.35, 0.8, a) * smoothstep(0.3, 0.8, b) * (0.55 + 1.3 * smoothstep(0.4, 0.8, f));
    let seen = smoothstep(0.04, 0.3, off) * mix(1.0, 0.45, off * off) * exp(-0.05 * depth0);
    // About the pattern's mean, so the water keeps its brightness.
    colour *= max(1.0 + shafts * smoothstep(0.0, 0.15, l.y) * (pattern - SHAFT_MEAN) * seen * SHAFT_GAIN, 0.4);
  }
  return colour;
}

fn apply_fog(color: vec3<f32>, p: vec3<f32>) -> vec3<f32> {
  if (frame.fog_color.w < 0.5) { return color; }
  let f = fog_amount(p);
  return mix(color, f.rgb, f.a);
}

// How much of something blended (particles, strips) still shows through
// the fog: it fades into the fogged scene behind it. 1 without fog.
fn fog_keep(p: vec3<f32>) -> f32 {
  if (frame.fog_color.w < 0.5) { return 1.0; }
  return 1.0 - fog_amount(p).a;
}

// A darker patch of the moon (a mare) at `c` of radius `r` on the disc.
fn moon_mare(q: vec2<f32>, c: vec2<f32>, r: f32) -> f32 {
  return 1.0 - smoothstep(0.2 * r, r, length(q - c));
}

// The moon in direction `d`: rgb the light it adds, a its coverage (it
// hides the stars behind it). A disc lit by its phase from the right
// (waxing) or the left (waning), darker maria, a faint earthshine on the
// dark side and a halo; set below the horizon.
fn moon_at(d: vec3<f32>, up: f32) -> vec4<f32> {
  let radius = frame.sky_moon.w;
  if (radius <= 0.0) { return vec4<f32>(0.0); }
  let m = normalize(frame.sky_moon.xyz);
  let cos_m = dot(d, m);
  if (cos_m <= 0.0) { return vec4<f32>(0.0); }
  var right = cross(m, vec3<f32>(0.0, 1.0, 0.0));
  if (dot(right, right) < 1e-6) { right = vec3<f32>(1.0, 0.0, 0.0); }
  right = normalize(right);
  let top = cross(right, m);
  // On the disc: -1..1 across its radius.
  let q = vec2<f32>(dot(d, right), dot(d, top)) / sin(radius);
  let r = length(q);
  let above = smoothstep(-0.004, 0.004, up);
  let coverage = (1.0 - smoothstep(0.92, 1.0, r)) * above;
  let theta = frame.sky_moon_look.w * 6.2831853;
  let lit_fraction = 0.5 - 0.5 * cos(theta);
  // The halo: a few radii of scattered moonlight, as bright as the moon is
  // full, gone with the moon below the horizon.
  let halo = exp(-max(r - 1.0, 0.0) * 1.5) * 0.012 * lit_fraction * above * smoothstep(-0.02, 0.02, m.y);
  if (coverage <= 0.0) { return vec4<f32>(frame.sky_moon_look.rgb * halo, 0.0); }
  // The sphere facing the viewer, lit from (sin theta, 0, -cos theta).
  let n = vec3<f32>(q, sqrt(max(1.0 - dot(q, q), 0.0)));
  let l = vec3<f32>(sin(theta), 0.0, -cos(theta));
  let lit = smoothstep(-0.05, 0.05, dot(n, l)) * (0.72 + 0.28 * n.z);
  let maria = min(0.3 * moon_mare(q, vec2<f32>(-0.32, 0.28), 0.42)
    + 0.24 * moon_mare(q, vec2<f32>(0.2, 0.14), 0.32)
    + 0.26 * moon_mare(q, vec2<f32>(0.02, -0.4), 0.34)
    + 0.18 * moon_mare(q, vec2<f32>(-0.5, -0.12), 0.22)
    + 0.16 * moon_mare(q, vec2<f32>(0.4, 0.5), 0.2), 0.42);
  let light = frame.sky_moon_look.rgb * (lit * (1.0 - maria) + 0.012);
  return vec4<f32>(light * coverage + frame.sky_moon_look.rgb * halo, coverage);
}

// A uniform value in [0, 1) for an integer lattice cell, from integer
// operations only (noise.wgsl's PCG), so every GPU picks the same cells: the
// stars here, the caustic cells (lighting.wgsl) and the foam (water.wgsl).
// The `fract(sin(x) * 43758.5453)` hash this replaces scaled sin's last bits,
// which GPUs round differently, into unrelated values, so a cell lit on one
// GPU was dark on another.
fn lattice_hash(c: vec3<i32>) -> f32 {
  var h = (bitcast<u32>(c.x) * 73856093u) ^ (bitcast<u32>(c.y) * 19349663u) ^ (bitcast<u32>(c.z) * 83492791u);
  h = h * 747796405u + 2891336453u;
  h = ((h >> ((h >> 28u) + 4u)) ^ h) * 277803737u;
  h = (h >> 22u) ^ h;
  return f32(h >> 8u) / 16777216.0;
}

// The clear sky's light on a surface facing unit `n` (its irradiance over pi, the image-based light's units, at the
// sky's own strength): what clouds are lit by besides the sun, the cloud-free sky above and around them
// (atmosphere.rs clear_sky_light, made each frame on the CPU), never the image-based light, which the live sky fills
// with the clouds themselves (lit by their own capture they converged several times too dark). Overhead, or blended
// from the horizontal ones towards the sun's bearing, away from it and across it.
fn clear_sky_light(n: vec3<f32>) -> vec3<f32> {
  let c = frame.clouds;
  let s = normalize(vec3<f32>(frame.sky_sun_dir.x, 0.0, frame.sky_sun_dir.z) + vec3<f32>(0.0, 0.0, 1e-5));
  let h = normalize(vec3<f32>(n.x, 0.0, n.z) + vec3<f32>(1e-5, 0.0, 0.0));
  let w = dot(h, s);
  let side = mix(c[9].rgb, select(c[8].rgb, c[7].rgb, w > 0.0), abs(w));
  return mix(side, c[6].rgb, clamp(n.y, 0.0, 1.0));
}

// An overcast deck's base seen far off along `d`, as the sky's march lights a base (sky.wgsl clouds_over): its grey
// ambient from the clear sky's low light (clear_sky_light) and a little of the sun through it. a: how much of the sky
// the deck closes (none under a broken sky), the weight the distant sky, what lies under the horizon and a mirror of
// the sky take it by.
fn cloud_deck(d: vec3<f32>) -> vec4<f32> {
  let c = frame.clouds;
  if (c[3].w < 0.5 || c[0].w <= 0.0 || c[0].y <= 0.0) { return vec4<f32>(0.0); }
  let luma = vec3<f32>(0.2126, 0.7152, 0.0722);
  let sky_scale = frame.params.x / max(c[4].z, 0.05);
  let to_light = normalize(frame.light_dir.xyz);
  let sun_side = clear_sky_light(normalize(vec3<f32>(to_light.x, 0.0, to_light.z) + vec3<f32>(0.0, 1e-3, 0.0)));
  let view_side = clear_sky_light(normalize(vec3<f32>(d.x, 0.0, d.z) + vec3<f32>(0.0, 1e-3, 0.0)));
  let side = dot(mix(view_side, sun_side, 0.5), luma) * sky_scale;
  let light = dot(frame.light_color.rgb, luma) / max(c[4].y, 9.1e-4);
  return vec4<f32>(vec3<f32>(side * 0.21 + light * 0.02) * c[2].rgb * 2.0, smoothstep(0.6, 0.95, c[0].x));
}

// How much of the stars a cloud deck leaves (clouds.rs): none through an overcast, all of them under a broken sky,
// whose gaps the clouds' own march shows them through.
fn star_veil() -> f32 {
  let c = frame.clouds;
  if (c[3].w < 0.5 || c[0].w <= 0.0 || c[0].y <= 0.0) { return 1.0; }
  return 1.0 - smoothstep(0.5, 0.9, c[0].x);
}

// The procedural sky of a chosen look in direction `d` (sky_* fields): a
// zenith to horizon gradient, dark ground, the sun disc with its glow,
// stars where the sky is dark, and the moon over them.
fn sky_look(d: vec3<f32>) -> vec3<f32> {
  let sun = normalize(frame.sky_sun_dir.xyz);
  let up = clamp(d.y, -1.0, 1.0);
  var sky = mix(frame.sky_horizon.rgb, frame.sky_zenith.rgb, pow(max(up, 0.0), 0.55));
  sky = mix(frame.sky_ground.rgb, sky, smoothstep(-0.08, 0.02, up));
  let cos_sun = dot(d, sun);
  let glow = pow(max(cos_sun, 0.0), max(frame.sky_glow.x, 1.0)) * frame.sky_sun.w;
  let disc = smoothstep(frame.sky_sun.x, frame.sky_sun.y, cos_sun) * frame.sky_sun.z;
  var color = sky * frame.sky_horizon.w + frame.sky_glow.yzw * (glow + disc);
  let moon = moon_at(d, up);
  if (frame.sky_ground.w > 0.0 && up > 0.0) {
    // Stars: a hashed grid of points on the sky, dimmed by the sky's brightness.
    let g = d * 220.0;
    let cell = floor(g);
    let h = lattice_hash(vec3<i32>(cell));
    let f = fract(g) - 0.5;
    let star = smoothstep(0.985, 1.0, h) * smoothstep(0.32, 0.0, length(f));
    let dark = 1.0 - clamp(dot(sky, vec3<f32>(0.2126, 0.7152, 0.0722)) * 4.0, 0.0, 1.0);
    color += vec3<f32>(star * frame.sky_ground.w * dark * 3.0) * smoothstep(0.0, 0.15, up) * (1.0 - moon.a) * star_veil();
  }
  return color + moon.rgb;
}
