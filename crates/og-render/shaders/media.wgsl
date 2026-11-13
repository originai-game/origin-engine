// What the air and the water between the eye and a point do to its light,
// for every pass that draws into the HDR target: the frame's fog (distance
// and height, og.Fog) and the underwater view's water body (the sun's
// shafts in it are the water volume's). Part of the frame group's text (shader_source.rs FRAME_WGSL),
// after frame.wgsl, whose `frame` uniform it reads.

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

// The underwater view's light (fog_color.w 2, water.rs `underwater_fog`) on a ray from `origin` along `dir` for `d`
// metres, as the colour the fog's `amount` mixes in. The fog colour is the water looking level just under the surface.
// Daylight falls off with depth (fog_height: x the surface's height, y its attenuation per metre), so the light a ray
// gathers is the integral of e^{-K depth(s)} e^{-c s} over it (the single-scattering water body of Premoze and
// Ashikhmin, "Rendering natural waters", 2001): brighter looking up than down, and darker the deeper the eye.
// The sun's own share of it is the volume's to draw (`underwater_sun`), in shafts.
fn underwater_light(origin: vec3<f32>, dir: vec3<f32>, d: f32, amount: f32) -> vec3<f32> {
  return underwater_body(origin, dir, d, amount, underwater_sun());
}

// The share of the water body's light that is the sun's own, scattered once (fog_sun.w, none with the sun down),
// which the water volume draws and this fog leaves out (volumetric.wgsl `water_in`): the sun's light in the water is
// not even, the waves focus it and whatever stands in it shadows it, so it reaches the eye as shafts. The rest (the
// sky's light and what has scattered more than once) is even. 0 where no volume is drawn: the body is whole.
fn underwater_sun() -> f32 {
  return frame.fog_sun.w * smoothstep(0.0, 0.15, normalize(frame.light_dir.xyz).y);
}

// `underwater_light` less `sun` of it (0: the water's whole even light, as the surface mirrors it from below:
// water.wgsl).
fn underwater_body(origin: vec3<f32>, dir: vec3<f32>, d: f32, amount: f32, sun: f32) -> vec3<f32> {
  let c = max(frame.fog_distance.x, 1e-5);
  let k = frame.fog_height.y;
  let depth0 = max(frame.fog_height.x - origin.y, 0.0);
  let slant = max(c - k * dir.y, 0.2 * c);
  let reach = 1.0 - exp(-slant * d);
  return frame.fog_color.rgb * exp(-k * depth0) * reach * c / (slant * max(amount, 1e-4)) * (1.0 - sun);
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
