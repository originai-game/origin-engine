// Per-frame bindings shared by the passes that draw into the HDR target
// (pbr.wgsl, sky.wgsl, sprite.wgsl): the camera, the directional light and
// its cascades, the shadow maps and the image-based lighting. The renderer
// prepends this file to those shaders; the Rust side is `FrameUniform` in
// frame.rs. The local lights and their clusters (bindings 7 to 9) are
// declared after it in the lit shader only (clusters.rs).

const CASCADES: u32 = 3u;
const LOCAL_SHADOW_LAYERS: u32 = 48u;

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
  light_color: vec4<f32>,     // rgb: radiance, w: debug view (1: cascades)
  params: vec4<f32>,          // x: ibl intensity, y: prefiltered mips - 1, z: local lights drawn, w: 1 / cascade map size
  view_forward: vec4<f32>,    // xyz: camera forward, w: 1 / local shadow map size
  cascade_view_proj: array<mat4x4<f32>, 3>,
  cascade_bias: array<vec4<f32>, 3>,    // x: normal offset (world), y: depth bias (normalized), z: slice far (view depth), w: where it fades into the next (view depth; z: no blending)
  cluster: vec4<f32>,         // x, y: clusters per pixel across and down, z, w: depth slice scale and bias (of log2 depth in perspective)
  local_view_proj: array<mat4x4<f32>, 48>,  // per layer: a spot's view or a cube face (paraboloid layers: unused)
  // The atmosphere (`AtmosphereGpu` in atmosphere.rs).
  fog_color: vec4<f32>,       // rgb: colour, w: 1 when fog is on
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
  time: vec4<f32>,            // x: seconds, y: tick, z: ticks per second
  // Water (`water.rs`): caustics on what lies below the first surface that has them.
  caustics: vec4<f32>,        // x: the surface's height, y: strength (0: none), z: cells per metre, w: fade per metre of depth
  caustics_area: vec4<f32>,   // xy: the surface's min xz, zw: its max xz
  shadow: vec4<f32>,          // x: the sun's softness (0: hard), y: 1 when point lights' shadows are paraboloids, zw: unused
  // Stylisation (stylise.rs): the retro-3D vertex path and the material style.
  stylise: vec4<f32>,         // xy: the vertex snap grid in cells (0: none), z: affine texturing, w: depth bits (0: full)
  shadow_tint: vec4<f32>,     // rgb: the tint shadowed light moves towards, w: amount (0: off)
  outline_style: vec4<f32>,   // x: the distance at which outlines are their width (0: constant)
  // A reflection probe (probes.rs): set in the frame group bound for a
  // draw inside the probe's box, whose irradiance_map is then the
  // environment's prefiltered cube and prefiltered_map the probe's.
  probe_min: vec4<f32>,       // xyz: the box's min corner, w: 1 with a probe
  probe_max: vec4<f32>,       // xyz: the box's max corner, w: the probe's intensity
  probe_pos: vec4<f32>,       // xyz: the capture point, w: the blend distance
  // The camera's axes in the world: billboards face the camera of the
  // group they are drawn with (sprite.wgsl face_axes).
  view_right: vec4<f32>,      // xyz: right
  view_up: vec4<f32>,         // xyz: up
  hemi_sky: vec4<f32>,        // rgb: the hemisphere light's sky colour, w: intensity / (pi * ibl intensity) (0: off)
  hemi_ground: vec4<f32>,     // rgb: its ground colour
};

// The hemisphere light (three.js HemisphereLight) as irradiance in the IBL's units: added to the irradiance map's
// sample, so it reaches only the diffuse term (no reflection) and is scaled, like it, by AO.
fn hemi_irradiance(n: vec3<f32>) -> vec3<f32> {
  return mix(frame.hemi_ground.rgb, frame.hemi_sky.rgb, 0.5 * n.y + 0.5) * frame.hemi_sky.w;
}

@group(0) @binding(0) var<uniform> frame: Frame;
// Shadow depths stored in R32Float colour targets and read with textureLoad:
// WebGL2 can neither filter nor textureLod a depth texture.
@group(0) @binding(1) var cascade_map: texture_2d_array<f32>;
@group(0) @binding(2) var local_shadow_map: texture_2d_array<f32>;
@group(0) @binding(3) var irradiance_map: texture_cube<f32>;
@group(0) @binding(4) var prefiltered_map: texture_cube<f32>;
@group(0) @binding(5) var brdf_lut: texture_2d<f32>;
@group(0) @binding(6) var ibl_sampler: sampler;

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
// the probe, blended by `probe_weight`.
fn ambient_light(n: vec3<f32>, p: vec3<f32>) -> vec3<f32> {
  if (frame.probe_min.w < 0.5) { return textureSample(irradiance_map, ibl_sampler, n).rgb; }
  let top = frame.params.y;
  let env = textureSampleLevel(irradiance_map, ibl_sampler, n, top).rgb;
  let probe = textureSampleLevel(prefiltered_map, ibl_sampler, n, top).rgb * frame.probe_max.w;
  return mix(env, probe, probe_weight(p));
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
fn reflected_light(r: vec3<f32>, p: vec3<f32>, mip: f32) -> vec3<f32> {
  if (frame.probe_min.w < 0.5) { return textureSampleLevel(prefiltered_map, ibl_sampler, r, mip).rgb; }
  let env = textureSampleLevel(irradiance_map, ibl_sampler, r, mip).rgb;
  let probe = textureSampleLevel(prefiltered_map, ibl_sampler, probe_direction(r, p), mip).rgb * frame.probe_max.w;
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
    clear *= exp(-max(column, 0.0));
  }
  let amount = min(1.0 - clear, fd.w);
  let sun = pow(max(dot(dir, normalize(frame.light_dir.xyz)), 0.0), max(frame.fog_sun.w, 1e-3));
  return vec4<f32>(mix(frame.fog_color.rgb, frame.fog_sun.rgb, sun), amount);
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
    let h = fract(sin(dot(cell, vec3<f32>(12.9898, 78.233, 37.719))) * 43758.5453);
    let f = fract(g) - 0.5;
    let star = smoothstep(0.985, 1.0, h) * smoothstep(0.32, 0.0, length(f));
    let dark = 1.0 - clamp(dot(sky, vec3<f32>(0.2126, 0.7152, 0.0722)) * 4.0, 0.0, 1.0);
    color += vec3<f32>(star * frame.sky_ground.w * dark * 3.0) * smoothstep(0.0, 0.15, up) * (1.0 - moon.a);
  }
  return color + moon.rgb;
}
