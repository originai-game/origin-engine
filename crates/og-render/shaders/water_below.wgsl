// The surface seen from under it (water.wgsl `fs`, the eye below the water).
//
// A ray leaving the water bends away from the surface's normal (Snell, 1.33), so the whole sky, horizon to horizon,
// is seen inside a cone 97 degrees wide overhead: Snell's window. Inside it the surface passes what Fresnel leaves
// at the angle the ray takes in the air; outside it the surface is a mirror of the water under it.

// The view depth the prepass clears to (post.rs FAR_DEPTH): the sky.
const FAR_VIEW_DEPTH: f32 = 1.0e9;
// The sun's disc through the surface, in the sun's light on a surface facing it: bright enough to bloom, as the sky
// pass draws it, without the disc's true radiance (tens of thousands of times the sky's), which no frame holds.
const SUN_THROUGH: f32 = 6.0;

// The light along `out`, a direction in the air over the water at `p`: the sky as the surface mirrors it from above
// (the reflection maps, under the frame's cloud cover), and the sun where `out` meets it (`sun`: how much of it
// reaches the surface). The waves turn `out` from pixel to pixel, so the disc is drawn as they break it up, and the
// slopes a pixel does not draw (`unseen`, their variance) spread it into a glow: a refracted ray turns a third as
// far as the facet it leaves by, and the glow dims as it widens.
fn sky_over(out: vec3<f32>, p: vec3<f32>, sun: f32, unseen: f32) -> vec3<f32> {
  let l = normalize(frame.light_dir.xyz);
  let sky = overcast(reflected_light(off_sun(out, 0.0), p, 0.0) * sky_as_drawn(), out);
  if (l.y <= 0.0) { return sky; }
  let c = dot(out, l);
  // The disc (2 degrees, the sky pass's).
  let disc = smoothstep(0.99926, 0.99985, c);
  let width = 0.0003 + 0.11 * unseen;
  let glow = 0.0003 / width * exp(-max(1.0 - c, 0.0) / width);
  return sky + frame.light_color.rgb * ((disc + glow) * sun * SUN_THROUGH);
}

// The surface's colour from below at `world`, its normal `n` and the unit vector to the eye `v`. `behind` is the
// scene drawn behind the surface along the view, used where something stands over the water there; where the sky
// is open (`open`) the window shows the sky along the ray as the surface bends it, so the horizon lies at the
// window's rim and the sun where it is seen from the water, not where the straight view would put it. The slopes
// a pixel does not draw (`unseen`) soften the rim: some of its facets pass the ray and some mirror it.
fn below_surface(
  world: vec3<f32>, n: vec3<f32>, v: vec3<f32>, behind: vec3<f32>, open: bool, sun: f32, unseen: f32,
) -> vec3<f32> {
  let nf = select(n, -n, dot(n, v) < 0.0);
  let ci = dot(nf, v);
  // The sine of the angle in the air, squared: 1 at the critical angle (about 49 degrees from the normal), where
  // it changes by 1.75 per radian the facet turns.
  let st2 = 1.7689 * (1.0 - ci * ci);
  let rim = 1.75 * sqrt(unseen);
  let passed = 1.0 - smoothstep(1.0 - 2.0 * rim, 1.0 + 2.0 * rim, st2);
  // The share that comes through is Fresnel's at the angle the ray leaves into the air, none past the critical
  // angle (the facets that pass the ray at the rim are those turned towards it).
  let ct = sqrt(clamp(1.0 - st2 + 2.0 * rim, 0.0, 1.0));
  let window = (0.98 - 0.98 * pow(1.0 - ct, 5.0)) * select(step(st2, 1.0), passed, rim > 0.0);
  // Outside it the surface mirrors the water below: the body's own light along the mirrored ray (dark looking
  // down, so the window stands bright in a dark surface the waves break up).
  var mirror = water.under.rgb;
  if (frame.fog_color.w > 1.5) { mirror = underwater_body(world, reflect(-v, nf), 1e4, 1.0, 0.0); }
  var through = behind;
  if (open && window > 0.0) {
    // Past the critical angle of the surface drawn the ray leaves along it: the horizon.
    var out = refract(-v, nf, 1.33);
    if (st2 >= 1.0) { out = normalize(nf * ci - v); }
    // (A facet tilted from the level looks under the horizon, where the reflection maps hold the ground.)
    out = normalize(vec3<f32>(out.x, max(out.y, 0.02), out.z));
    through = sky_over(out, world, sun, unseen);
  }
  return mix(mirror, through * water.shallow.rgb, window);
}
