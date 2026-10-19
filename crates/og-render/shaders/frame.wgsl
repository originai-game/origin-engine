// Per-frame bindings shared by the passes that draw into the HDR target
// (pbr.wgsl, sky.wgsl, sprite.wgsl): the camera, the directional light and
// its cascades, the shadow maps and the image-based lighting. The renderer
// prepends this file to those shaders; the Rust side is `FrameUniform` in
// frame.rs. The local lights and their clusters (bindings 7 to 9) are
// declared after it in the lit shader only (clusters.rs).

const CASCADES: u32 = 3u;
const LOCAL_SHADOW_LAYERS: u32 = 12u;

struct LocalLight {
  position_range: vec4<f32>,  // xyz: position, w: range
  color_kind: vec4<f32>,      // rgb: color * intensity, w: 0 point, 1 spot
  direction_slot: vec4<f32>,  // xyz: spot axis (away from the light), w: shadow slot (-1: none)
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
  cascade_bias: array<vec4<f32>, 3>,    // x: normal offset (world), y: depth bias (normalized), z: slice far (view depth), w: unused
  cluster: vec4<f32>,         // x, y: clusters per pixel across and down, z, w: depth slice scale and bias (of log2 depth in perspective)
  local_view_proj: array<mat4x4<f32>, 12>,
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
  time: vec4<f32>,            // x: seconds, y: tick, z: ticks per second
};

@group(0) @binding(0) var<uniform> frame: Frame;
// Shadow depths stored in R32Float colour targets and read with textureLoad:
// WebGL2 can neither filter nor textureLod a depth texture.
@group(0) @binding(1) var cascade_map: texture_2d_array<f32>;
@group(0) @binding(2) var local_shadow_map: texture_2d_array<f32>;
@group(0) @binding(3) var irradiance_map: texture_cube<f32>;
@group(0) @binding(4) var prefiltered_map: texture_cube<f32>;
@group(0) @binding(5) var brdf_lut: texture_2d<f32>;
@group(0) @binding(6) var ibl_sampler: sampler;

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

// The procedural sky of a chosen look in direction `d` (sky_* fields): a
// zenith to horizon gradient, dark ground, the sun disc with its glow, and
// stars where the sky is dark.
fn sky_look(d: vec3<f32>) -> vec3<f32> {
  let sun = normalize(frame.sky_sun_dir.xyz);
  let up = clamp(d.y, -1.0, 1.0);
  var sky = mix(frame.sky_horizon.rgb, frame.sky_zenith.rgb, pow(max(up, 0.0), 0.55));
  sky = mix(frame.sky_ground.rgb, sky, smoothstep(-0.08, 0.02, up));
  let cos_sun = dot(d, sun);
  let glow = pow(max(cos_sun, 0.0), max(frame.sky_glow.x, 1.0)) * frame.sky_sun.w;
  let disc = smoothstep(frame.sky_sun.x, frame.sky_sun.y, cos_sun) * frame.sky_sun.z;
  var color = sky * frame.sky_horizon.w + frame.sky_glow.yzw * (glow + disc);
  if (frame.sky_ground.w > 0.0 && up > 0.0) {
    // Stars: a hashed grid of points on the sky, dimmed by the sky's brightness.
    let g = d * 220.0;
    let cell = floor(g);
    let h = fract(sin(dot(cell, vec3<f32>(12.9898, 78.233, 37.719))) * 43758.5453);
    let f = fract(g) - 0.5;
    let star = smoothstep(0.985, 1.0, h) * smoothstep(0.32, 0.0, length(f));
    let dark = 1.0 - clamp(dot(sky, vec3<f32>(0.2126, 0.7152, 0.0722)) * 4.0, 0.0, 1.0);
    color += vec3<f32>(star * frame.sky_ground.w * dark * 3.0) * smoothstep(0.0, 0.15, up);
  }
  return color;
}
