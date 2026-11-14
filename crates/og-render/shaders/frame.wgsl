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
  clouds: array<vec4<f32>, 12>,  // the clouds (clouds.rs Clouds::gpu; [3].w 0: none; [4].w 1: captured in IBL; [6..10] the clear sky's light; [10].zw, [11] their shadow)
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
// The clouds' shadow (clouds/shadow.rs, cloud_shadow_bake.wgsl): how much of the sun's beam the layer lets through,
// over one period of the cloud field. A storage texture as the reflections are, read by lighting.wgsl `cloud_shadow`.
@group(0) @binding(14) var cloud_shadow_map: texture_storage_2d<r32float, read>;

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
  // The irradiance cube has one level: read without screen derivatives, so
  // a shader may gather after a fragment has returned.
  if (frame.probe_min.w < 0.5) { return gi_ambient_from(textureSampleLevel(irradiance_map, ibl_sampler, n, 0.0).rgb, s, n); }
  let top = frame.params.y;
  let env = textureSampleLevel(irradiance_map, ibl_sampler, n, top).rgb;
  let probe = textureSampleLevel(prefiltered_map, ibl_sampler, n, top).rgb * frame.probe_max.w;
  return gi_ambient_from(mix(env, probe, probe_weight(p)), s, n);
}

// ambient_light_from with the probe GI's own gather at `p`.
fn ambient_light(n: vec3<f32>, p: vec3<f32>) -> vec3<f32> {
  return ambient_light_from(n, p, gi_sample_at(p, n));
}

// The environment's diffuse light alone, without the probe GI: what the
// GI weighs its sky against, the style's flat ambient, the water's sky.
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
