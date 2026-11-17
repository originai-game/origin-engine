// Sky pass: the environment's sky cubemap drawn as the background. A
// fullscreen triangle at the far plane (depth 1, LessEqual, no depth write)
// after the opaque batches, so it only shades pixels no geometry covered.
// Prepended by frame.wgsl; the sky cube is the sky pass's own group (it
// would cost every lit pipeline a texture slot in the frame group).

@group(1) @binding(0) var sky_map: texture_cube<f32>;
// The background's look (SkyParams in scene/environment.rs).
struct SkyParams {
  look: vec4<f32>,  // x: 1 when an environment map is drawn from map_cube, y: the blurriness's prefiltered level (0: sharp), z: the map's scale, w: the blended map's weight
  turn: vec4<f32>,  // xy: (cos, sin) of the map's turn about +y
};
@group(1) @binding(3) var<uniform> sky_params: SkyParams;
// An environment map behind the scene, drawn from its own cube at its full
// size (the sky cube is SKY_SIZE, which would cap a large map), and the map
// it blends towards; the sky cube again without a map.
@group(1) @binding(4) var map_cube: texture_cube<f32>;
@group(1) @binding(5) var blend_cube: texture_cube<f32>;
// The aurora's footprint and deposition table (aurora.rs): a 1×1
// placeholder until a frame has an aurora.
@group(1) @binding(6) var aurora_map: texture_2d<f32>;

// grey: the overcast strip's shared grey (clouds_over), a cloud layer as clouds_march answers one, made by
// vs_sky_look; zero from vs_sky, for the fragment stages that march no clouds (each pipeline with vs_sky_look
// compiles a march in its vertex stage).
struct SkyOut {
  @builtin(position) pos: vec4<f32>,
  @location(0) ndc: vec2<f32>,
  @location(1) @interpolate(flat) grey: vec4<f32>,
};

fn sky_triangle(i: u32) -> SkyOut {
  var o: SkyOut;
  o.ndc = fullscreen_clip(i).xy;
  o.pos = vec4<f32>(o.ndc, 1.0, 1.0);
  o.grey = vec4<f32>(0.0);
  return o;
}

@vertex
fn vs_sky(@builtin(vertex_index) i: u32) -> SkyOut {
  return sky_triangle(i);
}

// The sky's triangle for fs_sky_look, with the overcast strip's grey (clouds_over): the deck's mean along four fixed
// azimuths about the view at the lowest elevation, undithered. It is the same for every pixel, so it is marched
// here, at the triangle's three vertices, not at each pixel of the strip (four marches a pixel: most of a storm
// sky's cost).
@vertex
fn vs_sky_look(@builtin(vertex_index) i: u32) -> SkyOut {
  var o = sky_triangle(i);
  let c = frame.clouds;
  let viewer = max(frame.view_pos.y + frame.origin.y, 0.0);
  if (c[3].w < 0.5 || viewer > c[0].z || cloud_deck(vec3<f32>(0.0, 1.0, 0.0)).a <= 0.0) { return o; }
  let lowest = clamp((c[0].z - viewer) / 24000.0, 0.002, 0.2);
  let ahead = frame.view_forward.xyz;
  let f = normalize(vec3<f32>(ahead.x, 0.0, ahead.z) + vec3<f32>(1e-6, 0.0, 0.0));
  // Every march in one loop (one copy of the march's code: DXC inlines every call).
  for (var k = 0u; k < 4u; k++) {
    let a = (f32(k) - 1.5) * 0.3;
    let az = vec3<f32>(f.x * cos(a) - f.z * sin(a), 0.0, f.x * sin(a) + f.z * cos(a));
    let flat_d = normalize(az + vec3<f32>(1e-6, 0.0, 0.0));
    let ray = flat_d * sqrt(1.0 - lowest * lowest) + vec3<f32>(0.0, lowest, 0.0);
    o.grey += clouds_march(ray, vec2<f32>(-1.0)) * 0.25;
  }
  return o;
}

@fragment
fn fs_sky(in: SkyOut) -> @location(0) vec4<f32> {
  if (frame.light_color.w > 0.5) { return debug_unsupported(in.pos.xy); }
  // The view ray through the pixel: from its near point (perspective and orthographic).
  let near = frame.inv_view_proj * vec4<f32>(in.ndc, 0.0, 1.0);
  let eye = near.xyz / near.w;
  let dir = pixel_ray(in.ndc);
  return vec4<f32>(sky_fog(sky_background(eye, dir) * (1.0 - frame.view_up.w), dir), 1.0);
}

// The background along a view ray (through the grounded dome when there
// is one, frame.wgsl dome_lookup): with a blurriness the prefiltered (GGX)
// level it names, scaled as the map; otherwise an environment map from its
// own cube, turned, scaled and blended, or the sky cube.
fn sky_background(eye: vec3<f32>, dir: vec3<f32>) -> vec3<f32> {
  let d = dome_lookup(eye, dir);
  if (sky_params.look.y > 0.0) {
    return textureSampleLevel(prefiltered_map, ibl_sampler, d, sky_params.look.y).rgb * sky_params.look.z;
  }
  if (sky_params.look.x > 0.5) {
    let c = sky_params.turn.x;
    let s = sky_params.turn.y;
    let m = vec3<f32>(c * d.x - s * d.z, d.y, s * d.x + c * d.z);
    var col = textureSampleLevel(map_cube, ibl_sampler, m, 0.0).rgb;
    if (sky_params.look.w > 0.0) {
      col = mix(col, textureSampleLevel(blend_cube, ibl_sampler, m, 0.0).rgb, sky_params.look.w);
    }
    return col * sky_params.look.z;
  }
  return textureSampleLevel(sky_map, ibl_sampler, d, 0.0).rgb;
}

// The fog over the sky: what the fog hides 5 km out along the view, times
// the fog's sky amount (fog_height.w).
fn sky_fog(color: vec3<f32>, dir: vec3<f32>) -> vec3<f32> {
  if (frame.fog_color.w < 0.5) { return color; }
  let f = fog_amount(frame.view_pos.xyz + dir * 5000.0);
  return mix(color, f.rgb, f.a * frame.fog_height.w);
}

// What the procedural sky draws under the horizon (its ground, or an
// overcast's strip: clouds_over) stands for land, which the fog hides
// whatever it leaves of the sky (og.Fog `sky`): over the horizon's first 3°
// the rest of the fog 5 km out that sky_fog and cloud_view_fog left, so it
// takes the whole amount as land drawn as far off does. At the sky's share,
// where the world's own land left the view open the overcast's strip read
// as a flat band under the fogged hills. Not in a cube face (the live sky's
// capture, a probe): its sky under the horizon stands for the ground round
// the point, whose light the surfaces take from below.
fn land_fog(color: vec3<f32>, dir: vec3<f32>) -> vec3<f32> {
  let below = smoothstep(0.0, 0.05, -dir.y);
  if (frame.fog_color.w < 0.5 || frame.drawn.z != 0u || below <= 0.0) { return color; }
  let f = fog_amount(frame.view_pos.xyz + dir * 5000.0);
  // Fogged by f.a s already; the rest t takes it to f.a: (1 - f.a) = (1 - f.a s)(1 - t).
  let shown = f.a * frame.fog_height.w;
  let rest = (f.a - shown) / max(1.0 - shown, 1e-4);
  return mix(color, f.rgb, rest * below);
}

// The viewer's height for the physical sky, in km: the look's altitude
// plus the camera's height in the world.
fn sky_viewer_km() -> f32 {
  return max(frame.sky_scatter[3].z + (frame.view_pos.y + frame.origin.y) * 0.001, 0.001);
}

// A strongly coloured highlight rolled off keeping its hue: past `knee`
// its largest channel approaches twice the knee, every channel scaled
// alike, and the weaker channels pushed down by `chroma` (the tone curve
// maps each channel alone and lifts the weaker ones of a bright colour
// relative to the strongest, so a sunset aureole of hue (1, 0.53, 0.14)
// read pale yellow; pre-darkened, it reads that hue), all only as far as
// the colour is saturated (a pale sky is left as it is).
fn sky_rolloff(c: vec3<f32>, knee: f32, chroma: f32) -> vec3<f32> {
  let m = max(max(c.r, c.g), c.b);
  if (m <= knee) { return c; }
  let saturation = 1.0 - min(min(c.r, c.g), c.b) / m;
  let w = smoothstep(0.3, 0.7, saturation);
  let rolled = knee + (m - knee) / (1.0 + (m - knee) / knee);
  let over = smoothstep(knee, 3.0 * knee, m);
  let hue = pow(c / m, vec3<f32>(1.0 + chroma * w * over));
  return mix(c, hue * rolled, w);
}

// The moon's light above the air that the physical sky scatters
// (SkyLook::moon_light): zero without a moon.
fn sky_moon_light() -> vec3<f32> {
  if (frame.sky_moon.w <= 0.0) { return vec3<f32>(0.0); }
  return frame.sky_moon_look.rgb * frame.sky_sun_dir.w;
}

// The look drawn by the physical sky (sky_scatter.wgsl): the scattered
// sunlight and moonlight, the disc reddened by the air it crosses, the
// night's floor, then the stars and the moon as sky_look draws them.
fn sky_physical_look(d: vec3<f32>) -> vec3<f32> {
  let sun = normalize(frame.sky_sun_dir.xyz);
  let h0 = sky_viewer_km();
  let up = clamp(d.y, -1.0, 1.0);
  let light = frame.sky_glow.yzw;
  let moon_light = sky_moon_light();
  let moon_dir = select(sun, normalize(frame.sky_moon.xyz), frame.sky_moon.w > 0.0);
  let sky = sky_rolloff((physical_sky_lit(d, sun, light, moon_dir, moon_light, h0, 24u) + frame.sky_scatter[5].rgb) * frame.sky_horizon.w, 0.6, 0.8);
  // Composite the solar radiance after atmospheric scattering, as in
  // Hillaire 2020 / UE SkyAtmosphere (third_party/hillaire-sky.md).
  // sun_intensity is radiance in the engine's scene-linear units, not a
  // display value. Keep its HDR energy for bloom and the output tonemapper;
  // atmospheric extinction, not a hue-normalized brightness cap, reddens it.
  let through = physical_sun(d, h0);
  let cos_d = dot(d, sun);
  // Filter the limb over one pixel's angular footprint, independently of
  // the authored radius. The aureole is scattering plus bloom, not a
  // radius-relative feather. Ray derivatives stay well behaved at centre.
  let angle = atan2(length(cross(d, sun)), cos_d);
  // The legacy SkyLook uniforms bracket the authored radius at 0.8/1.2.
  let radius = 0.5 * (acos(clamp(frame.sky_sun.x, -1.0, 1.0)) + acos(clamp(frame.sky_sun.y, -1.0, 1.0)));
  let pixel_angle = max(max(length(dpdxFine(d)), length(dpdyFine(d))), 1e-7);
  // A disc narrower than a pixel (the real sun's, 0.27°, on a capture
  // face's texels or in a wide view) is drawn a pixel wide with its light
  // spread over it: its light, and the glow bloom draws from it, stay.
  let drawn = max(radius, pixel_angle);
  var disc = 0.0;
  if (radius > 0.0) {
    disc = clamp((drawn - angle) / pixel_angle + 0.5, 0.0, 1.0) * (radius / drawn) * (radius / drawn);
  }
  // How far from the centre to the physical limb, 0 to 1.
  let edge = clamp(angle / drawn, 0.0, 1.0);
  let limb = 0.75 + 0.25 * sqrt(max(1.0 - edge * edge, 0.0));
  var color = sky + light * through * max(frame.sky_sun.z, 0.0) * limb * disc;
  let moon = moon_at(d, up, 0.0);
  if (frame.sky_ground.w > 0.0 && up > 0.0) {
    let g = d * 220.0;
    let cell = floor(g);
    let h = lattice3(vec3<i32>(cell), 0u);
    let f = fract(g) - 0.5;
    let bright = smoothstep(0.985, 1.0, h);
    let star = bright * smoothstep(0.32, 0.0, length(f));
    // A star shows by its contrast with the sky round it: its radiance is
    // fixed and the eye's threshold rises with the background (Blackwell
    // 1946, "Contrast thresholds of the human eye"). Under a moonless sky
    // all of them show, under a full moon's only the brighter, fewer still
    // near the moon, and none by day. `bright` is the star's magnitude.
    let seen = smoothstep(24.0, 96.0, 3.0 * bright / max(luma(sky), 1e-6));
    color += vec3<f32>(star * frame.sky_ground.w * seen * 3.0) * smoothstep(0.0, 0.15, up) * (1.0 - moon.a) * star_veil();
  }
  return color + moon.rgb;
}

// ── clouds (clouds.rs) ──────────────────────────────────────────────
// frame.clouds: [0] coverage, density, base, thickness; [1] the wind's
// offset (xyz), the tile's size; [2] albedo, detail; [3] the high
// layer's cover and height, steps, on; [4] x the planet's radius, y and z
// the sun's and the sky's dimming by the cover (Clouds::light); [5] a
// lightning flash in the cloud: where (frame coordinates), how bright;
// [6..10] the clear sky's light; [10] xy the viewer's place in the world
// (x and z, wrapped as the wind's offset is).

// Where a ray from `o` along unit `d` meets the sphere of radius `r`
// centred on the origin: (near, far), or (-1, -1) when it misses.
fn cloud_sphere(o: vec3<f32>, d: vec3<f32>, r: f32) -> vec2<f32> {
  let b = dot(o, d);
  let c = dot(o, o) - r * r;
  let disc = b * b - c;
  if (disc < 0.0) { return vec2<f32>(-1.0); }
  let s = sqrt(disc);
  return vec2<f32>(-b - s, -b + s);
}

// The sunlight reaching `p` (planet-centred metres, the planet of
// `radius`): under the physical sky the sun above the air through its
// transmittance at that height, else the scene's `light`.
fn cloud_sun(p: vec3<f32>, radius: f32, to_light: vec3<f32>, light: vec3<f32>, physical: bool) -> vec3<f32> {
  if (!physical) { return light; }
  let r_km = frame.sky_scatter[3].x + (length(p) - radius) * 0.001;
  return 3.0 * frame.sky_glow.yzw * scatter_sun_transmittance(r_km, dot(normalize(p), to_light));
}

// A cloud layer over a background it has not seen: rgb its light, a how much of the background it lets through
// (what a march answers: every step of one is linear in what lies behind it, so the clouds are marched without their
// background, at the sky's vertices and into the low-resolution cloud target as at a pixel). No clouds: (0, 1).
fn cloud_layer_over(layer: vec4<f32>, background: vec3<f32>) -> vec3<f32> {
  return layer.rgb + layer.a * background;
}

// `over` drawn over `layer` by the weight `w`.
fn cloud_layer_mix(layer: vec4<f32>, over: vec3<f32>, w: f32) -> vec4<f32> {
  return vec4<f32>(mix(layer.rgb, over, w), layer.a * (1.0 - w));
}

// The clouds along unit `d` (before the fog). Under an overcast, a ray under the lowest elevation
// whose march reaches the deck within the march's reach (the base some 24 km off) shows the deck as it is marched
// there, not an estimate of it: the strip from there down past the horizon to the sea's or the land's far edge had
// drawn a flat grey twice the deck's (cloud_deck alone, too bright). Each row of the strip would repeat that one
// elevation (the deck's features drawn out into columns), so the strip fades from its own column's march there,
// undithered, into one grey every pixel shares: the deck's mean there along four fixed azimuths about the view
// (`grey`, from vs_sky_look). The
// far base fades into it towards the horizon and below it, with no copy of a feature to fan or streak.
fn clouds_over(d: vec3<f32>, pixel: vec2<f32>, grey: vec4<f32>) -> vec4<f32> {
  let c = frame.clouds;
  if (c[3].w < 0.5) { return vec4<f32>(0.0, 0.0, 0.0, 1.0); }
  let viewer = max(frame.view_pos.y + frame.origin.y, 0.0);
  let cover = cloud_deck(d).a;
  let lowest = clamp((c[0].z - viewer) / 24000.0, 0.002, 0.2);
  let low = cover > 0.0 && viewer <= c[0].z && d.y < lowest;
  // Both marches in one loop (one copy of the march's code: DXC inlines
  // every call): under the deck's strip the deck along this column's
  // azimuth at the lowest elevation, undithered (k 0); then the column's
  // own (1), unless the deck covers it. The four fixed azimuths' mean is
  // vs_sky_look's.
  var own = vec4<f32>(0.0);
  var column = vec4<f32>(0.0, 0.0, 0.0, 1.0);
  for (var k = 0u; k < 2u; k++) {
    if (k == 0u && !low) { continue; }
    if (k == 1u && low && cover >= 0.999) { break; }
    var ray = d;
    var at = pixel;
    if (k == 0u) {
      let flat_d = normalize(vec3<f32>(d.x, 0.0, d.z) + vec3<f32>(1e-6, 0.0, 0.0));
      ray = flat_d * sqrt(1.0 - lowest * lowest) + vec3<f32>(0.0, lowest, 0.0);
      at = vec2<f32>(-1.0);
    }
    let m = clouds_march(ray, at);
    if (k == 0u) { own = m; } else { column = m; }
  }
  if (!low) { return column; }
  let deck = mix(own, grey, smoothstep(0.0, 0.5, (lowest - d.y) / lowest));
  if (cover >= 0.999) { return deck; }
  return mix(column, deck, cover);
}

// A cloud stands in the sky, and the fog reaches it as it reaches the sky
// behind it (sky_fog: the fog 5 km out, by Fog.sky's share), at its own
// distance while it is nearer than that. Fogged as a mesh at its distance,
// a cloud some kilometres off was the fog's one colour against a sky the
// fog had hardly touched: a flat cut-out with no light or shade of its
// own, and a bright washed-out band of them at the horizon. Scale its
// light before fog, just as the background/probe capture does.
fn cloud_view_fog(color: vec3<f32>, d: vec3<f32>, distance: f32) -> vec3<f32> {
  let f = cloud_fog(d, distance);
  return mix(color * (1.0 - frame.view_up.w), f.rgb, f.a);
}

// cloud_view_fog's fog at `distance` along `d`: its colour, and how much
// of it the cloud's light is mixed towards (none with the fog off).
fn cloud_fog(d: vec3<f32>, distance: f32) -> vec4<f32> {
  if (frame.fog_color.w < 0.5) { return vec4<f32>(0.0); }
  let f = fog_amount(frame.view_pos.xyz + d * min(distance, 5000.0));
  return vec4<f32>(f.rgb, f.a * frame.fog_height.w);
}

// Cloud light seen through the air in front of it (Hillaire 2020 §5.5,
// aerial perspective, under the physical sky): `color` the background
// along `d` (already holding that air's light), `scattered` and
// `transmittance` what the cloud adds and lets through, `at` metres the
// depth its light comes from. The cloud adds its light through the air,
// and the air's light it hides behind it is replaced by the same air's in
// front (no distance heuristic). The clouds shade that air (as UE's cloud
// shadow does its aerial perspective): the sun reaches it as it reaches
// the ground (Clouds::light, [4].y), and under a closed deck its sky is
// the deck, lighting it from the upper half of its directions only, so it
// scatters half the deck's radiance as much as it hides what is behind
// it, spectrally: a deck seen far off stays the deck's grey, darker than
// overhead as an overcast's horizon is.
fn cloud_air(color: vec4<f32>, scattered: vec3<f32>, transmittance: f32, at: f32, d: vec3<f32>, to_light: vec3<f32>) -> vec4<f32> {
  let c = frame.clouds;
  // Its samples by the path's length, 4 to 12 over 100 km (Hillaire 2020's
  // sample count for the sky's rays): a cloud some kilometres off has as
  // little air to integrate in front of it as it varies over.
  let steps = u32(round(mix(4.0, 12.0, clamp(at * 1e-5, 0.0, 1.0))));
  let air = air_path(d, to_light, sky_viewer_km(), at * 0.001, steps);
  // The deck's base as cloud_deck lights it (the clear sky's low light,
  // a fiftieth of the sun), its sun the one above the air that lights the
  // clouds (cloud_sun), not the scene's light: that is the clouds' own
  // dimming of it only where the scene was lit by Clouds::light_scene,
  // and a lightning flash turns it towards the bolt.
  let sun_dir = normalize(frame.sky_sun_dir.xyz);
  let sun_side = clear_sky_light(normalize(vec3<f32>(sun_dir.x, 0.0, sun_dir.z) + vec3<f32>(0.0, 1e-3, 0.0)));
  let view_side = clear_sky_light(normalize(vec3<f32>(d.x, 0.0, d.z) + vec3<f32>(0.0, 1e-3, 0.0)));
  let above = 3.0 * frame.sky_glow.yzw * scatter_sun_transmittance(frame.sky_scatter[3].x + c[0].z * 0.001, sun_dir.y);
  // In the hue of the light falling on it (a dusk's lavender, warmer
  // towards the glow), greyed by the cover as the march greys a cloud's
  // ambient: a cloud scatters every wavelength alike.
  let lit_base = (mix(view_side, sun_side, 0.5) * 0.21 + above * 0.02) * c[2].rgb * 2.0;
  let base = mix(lit_base, vec3<f32>(dot(lit_base, LUMA)), c[0].x * 0.7);
  let cover = cloud_deck(vec3<f32>(0.0, 1.0, 0.0)).a;
  let scale = frame.sky_glow.yzw * frame.sky_horizon.w;
  let under = 0.5 * base * (vec3<f32>(1.0) - air.through);
  let air_light = air.sun * scale * min(c[4].y, 1.0) + mix(air.sky * scale, under, cover);
  return vec4<f32>(color.rgb * transmittance + scattered * air.through + air_light * (1.0 - transmittance) * (1.0 - frame.view_up.w), color.a * transmittance);
}

// The volumetric layer between `a` and `b` metres along `d`, past where
// the march reaches: its cells are below a pixel there, so the layer is
// drawn as what it is on average (the far-cloud representation of
// horizon cloud banks): a ray through `len` of a layer whose cells cover
// a fraction `pc` of it meets one with probability 1 - (1 - pc)^(len /
// cell), lit as the march lights a cloud (its sun through the mean
// self-shadow of a cell, three octaves of multiple scattering, the sky's
// ambient at mid-layer) and seen through the same fog and air.
fn far_clouds(color: vec4<f32>, d: vec3<f32>, a: f32, b: f32, o: vec3<f32>, to_light: vec3<f32>, light: vec3<f32>, physical: bool, phase: f32, sky_top: vec3<f32>, sky_side: vec3<f32>) -> vec4<f32> {
  let c = frame.clouds;
  let len = b - a;
  if (len <= 0.0 || c[0].x <= 0.0) { return color; }
  let radius = c[4].x;
  // cloud_density: a fraction pow(coverage, 1.25) at the base, thinning to
  // the tops (about 0.6 of it over the layer).
  let pc = clamp(pow(c[0].x, 1.25) * 0.6, 0.0, 0.999);
  let cell = c[1].w * 0.25;
  let opacity = 1.0 - pow(1.0 - pc, len / cell);
  if (opacity <= 1e-4) { return color; }
  let at = a + min(0.5 * len, cell / max(pc, 0.05));
  let p = o + d * at;
  let sun_here = cloud_sun(p, radius, to_light, light, physical);
  // The mean of exp(-sigma x) over a half cell's depth towards the sun.
  let depth = c[0].y * 0.5 * c[0].w * 0.5;
  var sun = vec3<f32>(0.0);
  var k = 1.0;
  var e = 1.0;
  for (var m = 0u; m < 3u; m++) {
    let od = depth * e;
    sun += sun_here * k * (1.0 - exp(-od)) / max(od, 1e-4) * mix(phase, 0.25 / PI, 1.0 - k);
    k *= 0.6;
    e *= 0.3;
  }
  var ambient = mix(sky_side * 0.6, sky_top, 0.5) * 0.675;
  ambient = mix(ambient, vec3<f32>(luma(ambient)), c[0].x * 0.7);
  let lit = cloud_view_fog((sun + ambient) * c[2].rgb, d, at);
  if (physical) { return cloud_air(color, lit * opacity, 1.0 - opacity, at, d, to_light); }
  // The gradient sky has no air to integrate: fade into it over 27 km.
  return cloud_layer_mix(color, lit, opacity * exp(-a / 27000.0));
}

// The fine steps to a stride inside a cloud (clouds_march).
const CLOUD_FINE: u32 = 6u;

// The high layer, then the volumetric layer, faded into the sky with distance.
fn clouds_march(d: vec3<f32>, pixel: vec2<f32>) -> vec4<f32> {
  let c = frame.clouds;
  if (c[3].w < 0.5) { return vec4<f32>(0.0, 0.0, 0.0, 1.0); }
  let radius = c[4].x;
  let viewer = max(frame.view_pos.y + frame.origin.y, 0.0);
  let o = vec3<f32>(0.0, radius + viewer, 0.0);
  // A ray that meets the ground sees no clouds, unless the eye is above the
  // cloud base: then it looks down onto the layer (a sea of cloud from a
  // mountain), and the march below stops at the base, before the ground.
  let ground = cloud_sphere(o, d, radius);
  if (ground.x > 0.0 && viewer <= c[0].z) { return vec4<f32>(0.0, 0.0, 0.0, 1.0); }
  // Under the physical sky the clouds are lit by the sun at their own
  // height: the sun above the air (3 times the look's sunlight colour,
  // the day-night noon) through the sky's own transmittance down to the
  // sample, so a sunset's low clouds glow orange to pink from below and
  // they stay lit a while after the ground's sunset; never the scene's
  // directional light, which a lightning flash turns towards the bolt.
  // Otherwise the scene's sun, whose dimming by the clouds' own cover
  // (Clouds::light, [4].y) is undone: they are lit from above it.
  let physical = scatter_on();
  var to_light = normalize(frame.light_dir.xyz);
  if (physical) { to_light = normalize(frame.sky_sun_dir.xyz); }
  let light = frame.light_color.rgb / max(c[4].y, 9.1e-4);
  let cos_t = dot(d, to_light);
  let phase = mix(hg_phase(cos_t, 0.8), hg_phase(cos_t, -0.25), 0.4);
  // The clear sky's light from above and from the horizon (sky_look.wgsl clear_sky_light: never the image-based light,
  // which the live sky fills with these clouds). As bright as the sky is drawn: the clouds are part of it, so neither
  // the image-based light's strength (Environment::intensity, a light on surfaces, and a twilight's adaptation with
  // it) nor the cover's dimming of it lights them.
  let sky_top = clear_sky_light(vec3<f32>(0.0, 1.0, 0.0));
  // The bases see the sky's low light, half of it from the horizon under
  // the sun (its glow: the orange of a sunset's cloud bases).
  let sun_side = clear_sky_light(normalize(vec3<f32>(to_light.x, 0.0, to_light.z) + vec3<f32>(0.0, 1e-3, 0.0)));
  let view_side = clear_sky_light(normalize(vec3<f32>(d.x, 0.0, d.z) + vec3<f32>(0.0, 1e-3, 0.0)));
  let sky_side = mix(view_side, sun_side, 0.5);
  var color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
  // Under an overcast the distance fades the deck into more deck, not into the clear sky behind it: a bright clear
  // band at the horizon under a storm. The deck's base far off: its grey ambient (as the march lights a base, below)
  // and a little of the sun through it.
  let dk = cloud_deck(d);
  let deck_cover = dk.a;
  let deck = cloud_view_fog(dk.rgb, d, 60000.0);

  // The high thin layer (cirrus), streaked along the wind.
  if (c[3].x > 0.0) {
    let t = cloud_sphere(o, d, radius + c[3].y).y;
    if (t > 0.0) {
      let p = o + d * t;
      // (Twice the wind up there; scales the offsets' wrap repeats in.)
      let uv = (vec2<f32>(p.x, p.z) + c[10].xy - c[1].xz * 2.0) / (c[1].w * 2.0);
      let n = textureSampleLevel(cloud_noise, cloud_sampler, vec3<f32>(uv.x * 0.25, uv.y, 0.37), 0.0);
      let wisp = cloud_remap(n.r * 0.7 + n.g * 0.3, 1.0 - c[3].x, 1.0 - c[3].x * 0.3) * (0.5 + 0.5 * n.b);
      let fade = exp(-t / 120000.0);
      let lit = cloud_sun(p, radius, to_light, light, physical) * (0.25 + phase * 1.5) * 0.2 + sky_top * 0.8;
      color = cloud_layer_mix(color, cloud_view_fog(lit * c[2].rgb, d, t), clamp(wisp, 0.0, 1.0) * 0.6 * fade);
    }
  }

  // The volumetric layer between the shells at base and base + thickness.
  if (c[0].w <= 0.0 || c[0].y <= 0.0) { return color; }
  let r = length(o);
  let bottom = radius + c[0].z;
  let top = bottom + c[0].w;
  let outer = cloud_sphere(o, d, top);
  if (outer.y <= 0.0) { return color; }
  let inner = cloud_sphere(o, d, bottom);
  var start = 0.0;
  var end = outer.y;
  if (r < bottom) {
    start = inner.y;
  } else if (r > top) {
    start = outer.x;
    if (inner.x > 0.0) { end = inner.x; }
  } else if (inner.x > 0.0) {
    end = inner.x;
  }
  let far = 60000.0;
  // The far layer over the whole path, the fallback past the march's
  // reach (only where the march starts past half of it: nearer, the march
  // is drawn alone); else what lies past the march's reach, behind the
  // march. One loop (one copy of far_clouds' code: DXC inlines every call).
  let full = end;
  end = min(end, start + far * 0.5);
  var fallback = vec4<f32>(0.0);
  var behind_far = vec4<f32>(0.0);
  for (var j = 0u; j < 2u; j++) {
    if (j == 0u && start <= far * 0.5) { continue; }
    let layer = far_clouds(color, d, select(start, end, j == 1u), full, o, to_light, light, physical, phase, sky_top, sky_side);
    if (j == 1u) { behind_far = layer; break; }
    fallback = cloud_layer_mix(layer, deck, deck_cover);
    if (start >= far) { return fallback; }
  }
  // Up to twice the strides towards the horizon, where the path is longest.
  // The march crosses clear air in strides, reading the density's bound
  // (cloud_density_bound: never less than it with its detail). A cumulus takes most of the
  // light within some tens of metres and a stride is 60 to 300 m: marched in
  // strides a cloud is one sample deep, its silhouette the stride's jitter
  // and its light that one sample's. So a stride that finds cloud goes back
  // to the last clear sample and crosses the cloud in fine steps, until a
  // stride's length of them is clear again (Schneider 2015, the cheap and
  // the full sample). A fine step is no shorter than what a pixel of the
  // cloud target resolves at its distance.
  let steps = u32(c[3].z * (2.0 - abs(d.y)));
  let stride = (end - start) / f32(steps);
  // A different start per pixel turns banding into fine noise: interleaved
  // gradient noise (Jimenez 2014), whose neighbours differ evenly, so a
  // single frame shows an even fine grain rather than speckle, offset each
  // drawn frame under TAA (frame.shadow.w, as the shadows' spiral turns), so
  // its history averages the grain away. Offset by the tick, a frame drawn
  // without one (a paused game, a still capture) kept the same grain and
  // TAA held it.
  var jitter = ign(pixel + max(frame.shadow.w - 1.0, 0.0) * 5.588238);
  // (A pixel of -1: undithered, the march's middle.)
  if (pixel.x < 0.0) { jitter = 0.5; }
  var t = start + stride * jitter;
  var dt = stride;
  // The fine steps left before the march strides again (0: striding), and
  // whether the last move was a stride over air the fine steps have not
  // crossed.
  var fine = 0u;
  var strode = true;
  // The fine samples lit so far, the far light samples' optical depth
  // (their nearer and farther pair's, read by turns) and the fine step the
  // cloud was entered with.
  var lit = 0u;
  var far_od = vec2<f32>(0.0);
  var dt0 = stride;
  var transmittance = 1.0;
  var scattered = vec3<f32>(0.0);
  var depth = 0.0;
  // The first of the six steps towards the light, each twice the last: the
  // near ones see a billow's own shadow, the far ones the cloud's.
  let light_step = c[0].w * 0.012;
  // A lightning strike's flash, a point in the cloud over the bolt's root
  // (frame coordinates in [5].xyz, made planet-centred about the viewer).
  let flash = c[5];
  let flash_at = vec3<f32>(flash.x - frame.view_pos.x, radius + flash.y + frame.origin.y, flash.z - frame.view_pos.z);
  // What lights a sample and what lies in front of it change over
  // kilometres, not over a fine step, so neither is worked out at every
  // sample: the sun at the samples'
  // height (cloud_sun) is read at the march's two ends at the layer's base
  // and top and interpolated between, and the fog, the same past 5 km
  // (cloud_fog), is read once for all the samples past there.
  let span = max(end - start, 1.0);
  var sun_at: array<vec3<f32>, 4>;
  for (var k = 0u; k < 4u; k++) {
    let up = normalize(o + d * select(start, end, k >= 2u));
    sun_at[k] = cloud_sun(up * select(bottom, top, k % 2u == 1u), radius, to_light, light, physical);
  }
  let fog_far = cloud_fog(d, 5000.0);
  for (var i = 0u; i < steps * CLOUD_FINE; i++) {
    if (t >= end) { break; }
    let p = o + d * t;
    let h = length(p) - radius;
    if (fine == 0u) {
      if (cloud_density_bound(p, h) <= 0.0) {
        t += stride;
        strode = true;
        continue;
      }
      dt0 = max(stride / f32(CLOUD_FINE), t * 0.002);
      dt = dt0;
      fine = u32(stride / dt) + 1u;
      // Back over the stride that reached the cloud, once: the shell the
      // erosion empties holds no fine sample, and a march that went back
      // from it again would never leave it.
      if (strode) { t = max(t - stride, start) + dt * jitter; }
      strode = false;
      continue;
    }
    // (The weather map's value here serves the light's two near samples too.)
    let map = cloud_map(cloud_noise_at(p, h));
    let sigma = cloud_density_mapped(p, h, 1u, map);
    fine -= 1u;
    if (sigma > 0.0) {
      fine = u32(stride / dt) + 1u;
      let hf = clamp((h - c[0].z) / c[0].w, 0.0, 1.0);
      let along = clamp((t - start) / span, 0.0, 1.0);
      let sun_here = mix(mix(sun_at[0], sun_at[1], hf), mix(sun_at[2], sun_at[3], hf), along);
      // The column towards the light: six steps, the first two through the
      // eroded cloud, each sample somewhere in its own step by the pixel's
      // jitter (at the steps' middles a cloud's shade fell in terraces, one
      // to a step). The four far ones (90 m to 1.4 km off: the cloud's own
      // shadow, which changes over hundreds of metres, not over a fine
      // step; read at every sample they were half the march's cost) are
      // read two at a time by turns, the steps of 4 and 16 times the first
      // step's length, then of 8 and 32, each pair held for the next sample
      // (all four at the first): every sample reads two, the nearer (within
      // 350 m) with the weather map's value here.
      var od = 0.0;
      var reach = light_step;
      for (var k = 0u; k < 2u; k++) {
        let q = p + to_light * (reach * (1.0 + jitter) - light_step);
        od += cloud_density_mapped(q, length(q) - radius, 1u, map) * reach;
        reach *= 2.0;
      }
      let turn = lit % 2u;
      if (lit == 0u) {
        far_od = vec2<f32>(0.0);
      } else if (turn == 0u) {
        far_od.x = 0.0;
      } else {
        far_od.y = 0.0;
      }
      for (var k = 0u; k < select(1u, 2u, lit == 0u); k++) {
        let pair = (turn + k) % 2u;
        let near = reach * select(1.0, 2.0, pair == 1u);
        let q = p + to_light * (near * (1.0 + jitter) - light_step);
        let q_far = p + to_light * (near * 4.0 * (1.0 + jitter) - light_step);
        let s = cloud_density_mapped(q, length(q) - radius, 0u, map) * near + cloud_density(q_far, length(q_far) - radius, true) * near * 4.0;
        far_od += select(vec2<f32>(s, 0.0), vec2<f32>(0.0, s), pair == 1u);
      }
      lit += 1u;
      od += far_od.x + far_od.y;
      // Four octaves of multiple scattering (Wrenninge 2013; Hillaire,
      // "Physically Based Sky, Atmosphere and Cloud Rendering in
      // Frostbite", 2016), the later ones reaching deep into the shadowed
      // side, so it is lifted above near-black, and a sunlit face returns
      // most of the sun's light as a thick cloud does (three at 0.6 left it
      // a mid grey).
      var sun = vec3<f32>(0.0);
      var a = 1.0;
      var b = 1.0;
      for (var m = 0u; m < 4u; m++) {
        sun += sun_here * a * exp(-od * b) * mix(phase, 0.25 / PI, 1.0 - a);
        a *= 0.75;
        b *= 0.33;
      }
      if (flash.w > 0.0) {
        // The flash lights the cloud from inside about the root: falling
        // off over 600 m and dimmed by the cloud between, scattered evenly
        // (it has bounced many times), no streaks.
        let to = flash_at - p;
        let dist2 = dot(to, to);
        sun += vec3<f32>(0.75, 0.82, 1.0) * flash.w * (0.25 / PI) * exp(-sqrt(dist2) * c[0].y * 0.03) / (1.0 + dist2 / 3.6e5);
      }
      // The sky's light reaches a thin edge whole and a dense core through
      // the cloud round it: the body's own shade, which a flat ambient
      // term lacks.
      let open = 1.0 - 0.35 * clamp(sigma / max(c[0].y, 1e-6), 0.0, 1.0);
      var ambient = mix(sky_side * 0.6, sky_top, hf) * (0.35 + 0.65 * hf) * open;
      // Under a cover of cloud the light between them is the clouds' own,
      // grey, not the clear sky's blue.
      ambient = mix(ambient, vec3<f32>(luma(ambient)), c[0].x * 0.7);
      var fog = fog_far;
      if (t < 5000.0) { fog = cloud_fog(d, t); }
      let s = mix((sun + ambient) * c[2].rgb * (1.0 - frame.view_up.w), fog.rgb, fog.a) * sigma;
      // The step's light as the step itself dims it (Hillaire 2016, the
      // energy-conserving integration).
      let seg = exp(-sigma * dt);
      scattered += transmittance * (s - s * seg) / sigma;
      // The depth the cloud's light comes from, weighted by what it adds.
      depth += t * transmittance * (1.0 - seg);
      transmittance *= seg;
      if (transmittance < 0.01) { break; }
      // Behind the cloud's visible surface the fine steps lengthen with
      // what is already hidden, up to five times at nothing let through:
      // what they find is seen through the cloud in front, and the step's
      // integration above is exact over any length of even density.
      dt = dt0 * (1.0 + 4.0 * (1.0 - transmittance));
    }
    t += dt;
  }
  // The march stops with a hundredth of what is behind the cloud left, and
  // an opaque cloud leaves none: the sun's disc, some thousand times the
  // sky beside it, drew through a cloud that hid it as a circle on the
  // cloud. What the march stops at is opaque.
  transmittance = max(transmittance - 0.01, 0.0) / 0.99;
  // The air between the viewer and the clouds blends them into the sky (or, under an overcast, into the deck's own
  // mean colour along this ray, the fallback's where the ray found little cloud).
  // Fog controls atmospheric extinction, not the finite march's reach.
  // Height fog can leave distant cloud visible: taper coverage to the
  // fallback before the hard start>=far exit, with either fog setting.
  let range_fade = 1.0 - smoothstep(far * 0.5, far, start);
  var marched = vec4<f32>(behind_far.rgb * transmittance + scattered, behind_far.a * transmittance);
  if (physical) {
    // (A march that met no cloud lets its background through as it is.)
    if (transmittance < 1.0) {
      marched = cloud_air(behind_far, scattered, transmittance, depth / max(1.0 - transmittance, 1e-4), d, to_light);
    }
  } else {
    // The gradient sky has no air to integrate: fade into it over 27 km.
    let fade = exp(-start / (far * 0.45));
    let mean = mix(deck, scattered / max(1.0 - transmittance, 1e-3), smoothstep(0.05, 0.5, 1.0 - transmittance));
    marched = mix(cloud_layer_mix(behind_far, mean, deck_cover), marched, fade);
  }
  return mix(fallback, marched, range_fade);
}

// ── the aurora (aurora.rs) ──────────────────────────────────────────
// A port of Lawlor & Genetti, "Interactive Volume Rendering Aurora on the
// GPU" (J. WSCG 19, 2011): emission only, marched through the shell of
// the upper atmosphere, the step read from the footprint's distance field
// (section 3.2), each sample the footprint's flux times the deposition
// table at the sample's height and the flux's characteristic energy
// (section 2.1). Line heights and colours after Whiter et al., Ann.
// Geophys. 41, 2023 (CC BY 4.0). frame.aurora: [0] on (y), the planet's
// radius and the footprint's radius (km); [1] the shell's bottom and top
// (km), the footprint's side and the texture's rows; [2..4] the prompt
// nitrogen bands', the green and the red line's colours, w: the least
// step (km), the largest, the margin per texel.

// The aurora's light along `d` over `background`, dimmed by the air below it.
fn aurora_over(background: vec3<f32>, d: vec3<f32>, pixel: vec2<f32>) -> vec3<f32> {
  let a = frame.aurora;
  if (a[0].y < 0.5) { return background; }
  let radius = a[0].z;
  let reach = a[0].w;
  let viewer = max(frame.view_pos.y + frame.origin.y, 0.0) * 0.001;
  let o = vec3<f32>(0.0, radius + viewer, 0.0);
  // The ground hides it (the eye is always under the shell here).
  if (cloud_sphere(o, d, radius).x > 0.0) { return background; }
  let bottom = cloud_sphere(o, d, radius + a[1].x);
  let top = cloud_sphere(o, d, radius + a[1].y);
  if (top.y <= 0.0) { return background; }
  var start = max(bottom.y, 0.0);
  if (viewer > a[1].x) { start = 0.0; }
  let end = min(top.y, start + 3000.0);
  let side = a[1].z;
  let rows = a[1].w;
  let least = a[2].w;
  let most = a[3].w;
  // Interleaved gradient noise offsets the first step per pixel (as the
  // clouds' march does), so TAA resolves the steps into a smooth integral.
  var jitter = ign(pixel + f32(i32(frame.time.y) & 15) * 5.588238);
  if (pixel.x < 0.0) { jitter = 0.5; }
  var t = start + jitter * max(least, start * 0.004);
  var light = vec3<f32>(0.0);
  for (var i = 0; i < 128; i++) {
    if (t >= end) { break; }
    let p = o + d * t;
    let r = length(p);
    let h = r - radius;
    // The footprint's place: the ground distance from the viewer's foot
    // (azimuthal equidistant), square-root warped.
    let flat = vec2<f32>(p.x, p.z);
    let fl = length(flat);
    let ground = radius * atan2(fl, p.y);
    let q = flat / max(fl, 1e-6) * sqrt(ground / reach);
    // The step: as far as the nearest curtain (less a texel, which the
    // filtering may hide it in), at least a cone's width, at most `most`.
    let cone = max(least, t * 0.004);
    if (max(abs(q.x), abs(q.y)) >= 1.0) {
      t += most;
      continue;
    }
    let uv = vec2<f32>(q.x * 0.5 + 0.5, (q.y * 0.5 + 0.5) * side / rows);
    let f = textureSampleLevel(aurora_map, ibl_sampler, uv, 0.0);
    let texel = a[4].w * sqrt(ground * reach) + 0.1;
    let step = clamp(f.g - texel, cone, most);
    if (f.r + f.a > 0.0) {
      // The table's row by the characteristic energy (0.5 keV, then every
      // 0.67 octave), its column by height; rows interpolate.
      let row = clamp(log2(max(f.b, 0.01) / 0.5) / 0.67, 0.0, 7.0);
      let x = clamp((h - a[1].x) / (a[1].y - a[1].x), 0.0, 1.0);
      let tuv = vec2<f32>((x * (side - 1.0) + 0.5) / side, (side + row + 0.5) / rows);
      let e = textureSampleLevel(aurora_map, ibl_sampler, tuv, 0.0);
      light += (f.r * (e.r * a[2].rgb + e.g * a[3].rgb) + f.a * e.b * a[4].rgb) * step;
    }
    t += step;
  }
  if (max(max(light.r, light.g), light.b) <= 0.0) { return background; }
  return background + light * aurora_air(d, viewer);
}

// How much of the aurora's light the lower air lets through along `d`
// from a viewer `viewer` km up: under the physical sky its transmittance
// to space; else the paper's airmass (appendix A) of an exponential
// atmosphere of 8 km scale height, integrated in 12 widening steps, through
// Rayleigh extinction at sea level and a little haze.
fn aurora_air(d: vec3<f32>, viewer: f32) -> vec3<f32> {
  if (scatter_on()) {
    return scatter_sun_transmittance(frame.sky_scatter[3].x + sky_viewer_km(), d.y);
  }
  let radius = frame.aurora[0].z;
  let o = vec3<f32>(0.0, radius + viewer, 0.0);
  var mass = 0.0;
  var t = 0.0;
  var dt = 0.25;
  for (var i = 0; i < 12; i++) {
    let h = length(o + d * (t + dt * 0.5)) - radius;
    mass += exp(-h / 8.0) * dt;
    t += dt;
    dt *= 1.9;
  }
  let extinction = vec3<f32>(5.8, 13.5, 33.1) * 1e-3 + vec3<f32>(4e-3);
  return exp(-extinction * mass);
}

// The procedural sky drawn per pixel with the chosen look (frame.sky_*):
// its sun is where it is this frame, not where the lighting was baked.
// The live sky draws its capture's faces with it too (sky_capture.rs),
// through the face cameras' frame uniforms, whose stars are off (they
// would only add noise to the light): what moves in it moves in the
// reflections and the ambient light.
fn sky_look_behind(dir: vec3<f32>, pixel: vec2<f32>) -> vec3<f32> {
  var sky = sky_look(dir);
  if (scatter_on()) { sky = sky_physical_look(dir); }
  sky = aurora_over(sky, dir, pixel);
  // Fog.sky applies to the distant background alone; clouds integrate
  // their own view fog at each scattering sample, never a second time here.
  return sky_fog(sky * (1.0 - frame.view_up.w), dir);
}

// Storage bound only, not a display rolloff: avoid half-float infinity
// before bloom. The tone curves (SDR or HDR) own the display peak mapping.
fn sky_stored(color: vec3<f32>) -> vec4<f32> {
  return vec4<f32>(clamp(color, vec3<f32>(0.0), vec3<f32>(65504.0)), 1.0);
}

@fragment
fn fs_sky_look(in: SkyOut) -> @location(0) vec4<f32> {
  if (frame.light_color.w > 0.5) { return debug_unsupported(in.pos.xy); }
  let dir = pixel_ray(in.ndc);
  let clouds = clouds_over(dir, in.pos.xy, in.grey);
  return sky_stored(land_fog(cloud_layer_over(clouds, sky_look_behind(dir, in.pos.xy)), dir));
}

// The main camera's clouds, marched at half the frame's size each way (the cloud pass, pass_clouds.rs: a march is
// the sky's whole cost and a cloud has no edge a pixel wide) and drawn from there over the sky at the frame's own:
// the sun's disc, the stars and the aurora keep their pixels behind them, and a mesh its edge against them (the sky
// pass is depth-tested). The live sky's faces, the probes, the mirrors and the views march at their own pixels
// (fs_sky_look).
@group(2) @binding(0) var clouds_low: texture_2d<f32>;
// The frame's view depth for the march in a frame with a prepass (its lindepth: post.rs FAR_DEPTH where it drew
// nothing).
@group(2) @binding(1) var clouds_depth: texture_2d<f32>;

// Whether the prepass drew a surface at every pixel of the frame that reads the cloud target's texel at `pos`
// (the sky's draw reads between texels: the pixels within a texel of its centre). The sky fails its depth test at
// all of them, so the texel is never read. `texel` is a texel in NDC.
fn clouds_covered(pos: vec2<f32>, texel: vec2<f32>) -> bool {
  let size = vec2<f32>(textureDimensions(clouds_depth));
  // The frame's pixels a texel of the target.
  let scale = size * abs(texel) * 0.5;
  let lo = vec2<i32>(floor((pos - 1.0) * scale - 0.5));
  let hi = vec2<i32>(ceil((pos + 1.0) * scale - 0.5));
  let last = vec2<i32>(size) - 1;
  // The pixel under the texel's centre first: over the sky that is the only read.
  if (is_sky_depth(textureLoad(clouds_depth, clamp(vec2<i32>(pos * scale), vec2<i32>(0), last), 0).r)) { return false; }
  for (var y = lo.y; y < hi.y; y++) {
    for (var x = lo.x; x < hi.x; x++) {
      if (is_sky_depth(textureLoad(clouds_depth, clamp(vec2<i32>(x, y), vec2<i32>(0), last), 0).r)) { return false; }
    }
  }
  return true;
}

// Whether a ray along unit `d` meets the ground from under the cloud base: it sees no clouds (clouds_march).
fn cloud_grounded(d: vec3<f32>) -> bool {
  let c = frame.clouds;
  let viewer = max(frame.view_pos.y + frame.origin.y, 0.0);
  return viewer <= c[0].z && cloud_sphere(vec3<f32>(0.0, c[4].x + viewer, 0.0), d, c[4].x).x > 0.0;
}

// The cloud history's step (clouds.rs `CloudsLow::prepare`): the march and the resolve of a frame that reprojects.
struct CloudsTemporal {
  // The view-projection the history's frame marched with (its jitter too).
  prev_view_proj: mat4x4<f32>,
  // xy: the cloud target's size, zw: which texel of each two by two the march takes this frame.
  size: vec4<f32>,
  // x: 1 when the march takes one texel of each two by two (into the quarter target), 0 every texel.
  mode: vec4<f32>,
};
@group(2) @binding(2) var<uniform> clouds_temporal: CloudsTemporal;
// The resolve's reads: this frame's march at a quarter of the cloud target's texels, and the history.
@group(2) @binding(3) var clouds_quarter: texture_2d<f32>;
@group(2) @binding(4) var clouds_history: texture_2d<f32>;

// The cloud target's texel along `ndc`, at `pos` of the target: `texel` is a texel in NDC.
fn clouds_low_texel(ndc: vec2<f32>, pos: vec2<f32>, grey: vec4<f32>, texel: vec2<f32>) -> vec4<f32> {
  var d = pixel_ray(ndc);
  // The horizon is an edge in the clouds (a ray that meets the ground sees none, one just over it the farthest)
  // and the sky's draw reads this target between its texels: a texel under the horizon beside one over it marches
  // that one's ray, so a pixel over the horizon reads only clouds over it. A pixel under it reads none
  // (fs_sky_look_low).
  if (cloud_grounded(d)) {
    for (var k = 0u; k < 9u; k++) {
      let near = pixel_ray(ndc + (vec2<f32>(f32(k % 3u), f32(k / 3u)) - 1.0) * texel);
      if (near.y > d.y) { d = near; }
    }
  }
  let layer = clouds_over(d, pos, grey);
  return vec4<f32>(clamp(layer.rgb, vec3<f32>(0.0), vec3<f32>(65504.0)), layer.a);
}

// Where a fragment of the march falls in the cloud target: its texel's NDC, its position and a texel in NDC.
// Marching one texel of each two by two (`clouds_temporal.mode.x`), a fragment of the quarter target is the texel
// of its two by two the frame takes; otherwise it is its own.
struct MarchAt {
  ndc: vec2<f32>,
  pos: vec2<f32>,
  texel: vec2<f32>,
};

fn clouds_march_at(in: SkyOut) -> MarchAt {
  let texel = vec2<f32>(dpdx(in.ndc.x), dpdy(in.ndc.y));
  if (clouds_temporal.mode.x < 0.5) { return MarchAt(in.ndc, in.pos.xy, texel); }
  let size = clouds_temporal.size.xy;
  let at = min(floor(in.pos.xy) * 2.0 + clouds_temporal.size.zw, size - 1.0) + 0.5;
  let ndc = vec2<f32>(at.x / size.x * 2.0 - 1.0, 1.0 - at.y / size.y * 2.0);
  return MarchAt(ndc, at, vec2<f32>(2.0, -2.0) / size);
}

@fragment
fn fs_clouds_low(in: SkyOut) -> @location(0) vec4<f32> {
  // A derivative: before any branch.
  let m = clouds_march_at(in);
  return clouds_low_texel(m.ndc, m.pos, in.grey, m.texel);
}

// In a frame with a prepass: nothing where its surfaces cover every pixel that reads the texel.
@fragment
fn fs_clouds_low_depth(in: SkyOut) -> @location(0) vec4<f32> {
  let m = clouds_march_at(in);
  // Marching one texel of a two by two: nothing only where every pixel that reads any of the four is covered (the
  // resolve fills the other three from this one).
  let block = select(1.0, 2.0, clouds_temporal.mode.x > 0.5);
  if (clouds_covered(in.pos.xy, m.texel * block)) { return vec4<f32>(0.0, 0.0, 0.0, 1.0); }
  return clouds_low_texel(m.ndc, m.pos, in.grey, m.texel);
}

// The resolve of a frame that marched one texel of each two by two (Schneider 2015, the temporal upsampling of the
// clouds): every texel reads the history where its ray was the frame before (a cloud is far enough that its
// direction alone reprojects it), held within what this frame marched round it so a cloud that moved or a flash
// leaves no ghost; the texel the frame marched blends its march into that, so the march's dither averages over the
// frames that march it. A ray the history did not see reads the march. Into this frame's cloud target, which the
// sky reads and the next frame's resolve reprojects.
@fragment
fn fs_clouds_resolve(in: SkyOut) -> @location(0) vec4<f32> {
  let at = vec2<i32>(in.pos.xy);
  let quarter = vec2<i32>(textureDimensions(clouds_quarter)) - 1;
  let q = min(at / 2, quarter);
  let fresh = textureLoad(clouds_quarter, q, 0);
  var lo = fresh;
  var hi = fresh;
  for (var k = 0; k < 9; k++) {
    let n = textureLoad(clouds_quarter, clamp(q + vec2<i32>(k % 3 - 1, k / 3 - 1), vec2<i32>(0), quarter), 0);
    lo = min(lo, n);
    hi = max(hi, n);
  }
  let prev = clouds_temporal.prev_view_proj * vec4<f32>(pixel_ray(in.ndc), 0.0);
  let uv = vec2<f32>(0.5, -0.5) * prev.xy / max(prev.w, 1e-6) + 0.5;
  var out = fresh;
  if (prev.w > 0.0 && all(uv >= vec2<f32>(0.0)) && all(uv <= vec2<f32>(1.0))) {
    let seen = clamp(textureSampleLevel(clouds_history, ibl_sampler, uv, 0.0), lo, hi);
    let marched = all((at & vec2<i32>(1)) == vec2<i32>(clouds_temporal.size.zw));
    out = select(seen, mix(seen, fresh, 0.5), marched);
  }
  return out;
}

@fragment
fn fs_sky_look_low(in: SkyOut) -> @location(0) vec4<f32> {
  if (frame.light_color.w > 0.5) { return debug_unsupported(in.pos.xy); }
  let dir = pixel_ray(in.ndc);
  // Under the horizon there are no clouds but an overcast's strip (clouds_over), which has no edge there.
  var clouds = vec4<f32>(0.0, 0.0, 0.0, 1.0);
  if (!cloud_grounded(dir) || cloud_deck(dir).a > 0.0) {
    let uv = vec2<f32>(0.5 + 0.5 * in.ndc.x, 0.5 - 0.5 * in.ndc.y);
    clouds = textureSampleLevel(clouds_low, ibl_sampler, uv, 0.0);
  }
  return sky_stored(land_fog(cloud_layer_over(clouds, sky_look_behind(dir, in.pos.xy)), dir));
}
