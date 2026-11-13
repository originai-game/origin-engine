// The sky of a chosen look, as the sky pass draws it and the image-based
// light captures it: the gradient, the sun, the stars and the moon, with
// the clear sky's light and the far cloud deck the clouds and the water
// read. Part of the frame group's text (shader_source.rs FRAME_WGSL),
// after frame.wgsl, whose `frame` uniform it reads.

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
    let h = lattice3(vec3<i32>(cell), 0u);
    let f = fract(g) - 0.5;
    let star = smoothstep(0.985, 1.0, h) * smoothstep(0.32, 0.0, length(f));
    let dark = 1.0 - clamp(dot(sky, vec3<f32>(0.2126, 0.7152, 0.0722)) * 4.0, 0.0, 1.0);
    color += vec3<f32>(star * frame.sky_ground.w * dark * 3.0) * smoothstep(0.0, 0.15, up) * (1.0 - moon.a) * star_veil();
  }
  return color + moon.rgb;
}
