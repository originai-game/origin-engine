// Sky pass: the environment's sky cubemap drawn as the background. A
// fullscreen triangle at the far plane (depth 1, LessEqual, no depth write)
// after the opaque batches, so it only shades pixels no geometry covered.
// Prepended by frame.wgsl; the sky cube is the sky pass's own group (it
// would cost every lit pipeline a texture slot in the frame group).

@group(1) @binding(0) var sky_map: texture_cube<f32>;
// The clouds' noise volume (clouds.rs): r Perlin-Worley, g and b Worley
// detail, a coverage; tiling, sampled with repeat.
@group(1) @binding(1) var cloud_noise: texture_3d<f32>;
@group(1) @binding(2) var cloud_sampler: sampler;
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
// vs_sky_look; zero from vs_sky, whose fragment stage draws no clouds.
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
  // The view ray through the pixel: near point to far point (perspective and orthographic).
  let near = frame.inv_view_proj * vec4<f32>(in.ndc, 0.0, 1.0);
  let far = frame.inv_view_proj * vec4<f32>(in.ndc, 1.0, 1.0);
  let eye = near.xyz / near.w;
  let dir = normalize(far.xyz / far.w - eye);
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

fn view_dir(ndc: vec2<f32>) -> vec3<f32> {
  let near = frame.inv_view_proj * vec4<f32>(ndc, 0.0, 1.0);
  let far = frame.inv_view_proj * vec4<f32>(ndc, 1.0, 1.0);
  return normalize(far.xyz / far.w - near.xyz / near.w);
}

// The fog over the sky: what the fog hides 5 km out along the view, times
// the fog's sky amount (fog_height.w).
fn sky_fog(color: vec3<f32>, dir: vec3<f32>) -> vec3<f32> {
  if (frame.fog_color.w < 0.5) { return color; }
  let f = fog_amount(frame.view_pos.xyz + dir * 5000.0);
  return mix(color, f.rgb, f.a * frame.fog_height.w);
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

// The look drawn by the physical sky (sky_scatter.wgsl): the scattered
// sunlight, the disc reddened by the air it crosses, the night's floor,
// then the stars and the moon as sky_look draws them.
fn sky_physical_look(d: vec3<f32>) -> vec3<f32> {
  let sun = normalize(frame.sky_sun_dir.xyz);
  let h0 = sky_viewer_km();
  let up = clamp(d.y, -1.0, 1.0);
  let light = frame.sky_glow.yzw;
  let sky = sky_rolloff((physical_sky(d, sun, h0, 24u) * light + frame.sky_scatter[5].rgb) * frame.sky_horizon.w, 0.6, 0.8);
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
  var disc = 0.0;
  if (radius > 0.0) {
    disc = clamp((radius - angle) / pixel_angle + 0.5, 0.0, 1.0);
  }
  // How far from the centre to the physical limb, 0 to 1.
  let edge = clamp(angle / max(radius, 1e-7), 0.0, 1.0);
  let limb = 0.75 + 0.25 * sqrt(max(1.0 - edge * edge, 0.0));
  var color = sky + light * through * max(frame.sky_sun.z, 0.0) * limb * disc;
  let moon = moon_at(d, up);
  if (frame.sky_ground.w > 0.0 && up > 0.0) {
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

// ── clouds (clouds.rs) ──────────────────────────────────────────────
// frame.clouds: [0] coverage, density, base, thickness; [1] the wind's
// offset (xyz), the tile's size; [2] albedo, detail; [3] the high
// layer's cover and height, steps, on; [4] x the planet's radius, y and z
// the sun's and the sky's dimming by the cover (Clouds::light); [5] a
// lightning flash in the cloud: where (frame coordinates), how bright;
// [6..10] the clear sky's light; [10] xy the viewer's place in the world
// (x and z, wrapped as the wind's offset is).

fn cloud_remap(v: f32, lo: f32, hi: f32) -> f32 {
  return clamp((v - lo) / max(hi - lo, 1e-4), 0.0, 1.0);
}

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

// The cloud density at `p` (metres, planet-centred, the viewer over the
// origin's xz) of height `h` above the ground. The clouds stand over the
// world: the viewer's place in it ([10].xy) and the wind's offset ([1])
// place the point in the noise, each wrapped at four tiles, which every
// scale below repeats in (a scale four tiles are no period of changed
// every cloud's shape at a wrap).
fn cloud_density(p: vec3<f32>, h: f32, cheap: bool) -> f32 {
  let c = frame.clouds;
  let hf = (h - c[0].z) / max(c[0].w, 1.0);
  if (hf <= 0.0 || hf >= 1.0) { return 0.0; }
  // (Less the offset: the wind carries the shapes along it.)
  let at = vec3<f32>(p.x + c[10].x, h, p.z + c[10].y) - c[1].xyz;
  let uvw = at / c[1].w;
  // The lobes vary in size: the shape is read at two scales, blended by a
  // slow noise across the sky (small popcorn here, broad towers there).
  let small = textureSampleLevel(cloud_noise, cloud_sampler, uvw.xzy, 0.0).r;
  let broad = textureSampleLevel(cloud_noise, cloud_sampler, uvw.xzy * 0.5 + vec3<f32>(0.31, 0.17, 0.53), 0.0).r;
  let cover_px = textureSampleLevel(cloud_noise, cloud_sampler, uvw.xzy * 0.25, 0.0);
  let cover = cover_px.a;
  let shape = mix(small, broad, smoothstep(0.3, 0.7, textureSampleLevel(cloud_noise, cloud_sampler, uvw.xzy * 0.25 + vec3<f32>(0.6, 0.0, 0.2), 0.0).a));
  // The coverage varies across the sky most when it is broken, and not
  // at all when clear or overcast (an overcast stays a ceiling).
  // (The power: the cells at different heights overlap, so a column
  // meets cloud more often than one level does; with it the sky over a
  // viewer is about `coverage` covered, 0.3 a fair-weather sky.)
  let c0 = c[0].x;
  let local = clamp(pow(c0, 1.25) + (cover - 0.5) * 0.5 * min(1.0, 4.0 * c0 * (1.0 - c0)), 0.0, 1.0);
  // The shape channel is uniform: a threshold of 1 - local leaves a
  // fraction `local` in cloud near the base. Higher up the threshold
  // rises, so only the strongest cells reach the top (domes); the base is
  // cut flat at the condensation level, the cloud densest just above it.
  let th = mix(1.0 - local, 1.0, smoothstep(0.3, 1.0, hf) * 0.85);
  var base = cloud_remap(shape, th, min(th + 0.35, 1.0)) * smoothstep(0.0, 0.025, hf) * (1.0 + 0.5 * (1.0 - smoothstep(0.02, 0.2, hf)));
  if (!cheap && c[2].w > 0.0) {
    // The fine noise eats the tops into cauliflower and leaves the flat
    // bases whole.
    let fine = textureSampleLevel(cloud_noise, cloud_sampler, uvw.xzy * 5.0, 0.0);
    let erode = mix(fine.g, fine.b, hf) * c[2].w * (0.25 + 1.1 * hf);
    base = cloud_remap(base, erode * 0.6, 1.0);
  }
  return base * c[0].y;
}

// The sunlight reaching `p` (planet-centred metres, the planet of
// `radius`): under the physical sky the sun above the air through its
// transmittance at that height, else the scene's `light`.
fn cloud_sun(p: vec3<f32>, radius: f32, to_light: vec3<f32>, light: vec3<f32>, physical: bool) -> vec3<f32> {
  if (!physical) { return light; }
  let r_km = frame.sky_scatter[3].x + (length(p) - radius) * 0.001;
  return 3.0 * frame.sky_glow.yzw * scatter_sun_transmittance(r_km, dot(normalize(p), to_light));
}

fn cloud_hg(cos_t: f32, g: f32) -> f32 {
  let g2 = g * g;
  return (1.0 - g2) / (12.566371 * pow(max(1.0 + g2 - 2.0 * g * cos_t, 1e-4), 1.5));
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

// A cloud is a finite-distance participating surface, not the sky at
// infinity: the distance fog to it as meshes take it, independent of
// Fog.sky. Height fog is different: a cloud above it lies behind the same
// finite column as the sky beside it, so it takes that column as the sky
// does (Fog.sky's share): a hazed cloud against an unhazed sky drew a
// bright flat band of washed-out cloud at the horizon. Scale its light
// before fog, just as the background/probe capture does.
fn cloud_view_fog(color: vec3<f32>, d: vec3<f32>, distance: f32) -> vec3<f32> {
  let lit = color * (1.0 - frame.view_up.w);
  if (frame.fog_color.w < 0.5) { return lit; }
  let f = fog_amount_scaled(frame.view_pos.xyz + d * distance, frame.fog_height.w);
  return mix(lit, f.rgb, f.a);
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
  let air = air_path(d, to_light, sky_viewer_km(), at * 0.001, 12u);
  // The deck's base as cloud_deck lights it (the clear sky's low light,
  // a fiftieth of the sun), its sun the one above the air that lights the
  // clouds (cloud_sun), not the scene's light: that is the clouds' own
  // dimming of it only where the scene was lit by Clouds::light_scene,
  // and a lightning flash turns it towards the bolt.
  let luma = vec3<f32>(0.2126, 0.7152, 0.0722);
  let sky_scale = frame.params.x / max(c[4].z, 0.05);
  let sun_dir = normalize(frame.sky_sun_dir.xyz);
  let sun_side = clear_sky_light(normalize(vec3<f32>(sun_dir.x, 0.0, sun_dir.z) + vec3<f32>(0.0, 1e-3, 0.0)));
  let view_side = clear_sky_light(normalize(vec3<f32>(d.x, 0.0, d.z) + vec3<f32>(0.0, 1e-3, 0.0)));
  let above = 3.0 * frame.sky_glow.yzw * scatter_sun_transmittance(frame.sky_scatter[3].x + c[0].z * 0.001, sun_dir.y);
  // In the hue of the light falling on it (a dusk's lavender, warmer
  // towards the glow), greyed by the cover as the march greys a cloud's
  // ambient: a cloud scatters every wavelength alike.
  let lit_base = (mix(view_side, sun_side, 0.5) * sky_scale * 0.21 + above * 0.02) * c[2].rgb * 2.0;
  let base = mix(lit_base, vec3<f32>(dot(lit_base, luma)), c[0].x * 0.7);
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
    sun += sun_here * k * (1.0 - exp(-od)) / max(od, 1e-4) * mix(phase, 0.0795775, 1.0 - k);
    k *= 0.6;
    e *= 0.3;
  }
  var ambient = mix(sky_side * 0.6, sky_top, 0.5) * 0.675;
  ambient = mix(ambient, vec3<f32>(dot(ambient, vec3<f32>(0.2126, 0.7152, 0.0722))), c[0].x * 0.7);
  let lit = cloud_view_fog((sun + ambient) * c[2].rgb, d, at);
  if (physical) { return cloud_air(color, lit * opacity, 1.0 - opacity, at, d, to_light); }
  // The gradient sky has no air to integrate: fade into it over 27 km.
  return cloud_layer_mix(color, lit, opacity * exp(-a / 27000.0));
}

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
  // (Clouds::light, [4].yz) is undone: they are lit from above it.
  let physical = scatter_on();
  var to_light = normalize(frame.light_dir.xyz);
  if (physical) { to_light = normalize(frame.sky_sun_dir.xyz); }
  let light = frame.light_color.rgb / max(c[4].y, 9.1e-4);
  let cos_t = dot(d, to_light);
  let phase = mix(cloud_hg(cos_t, 0.6), cloud_hg(cos_t, -0.25), 0.35);
  // The clear sky's light from above and from the horizon (sky_look.wgsl clear_sky_light: never the image-based light,
  // which the live sky fills with these clouds).
  let sky_scale = frame.params.x / max(c[4].z, 0.05);
  let sky_top = clear_sky_light(vec3<f32>(0.0, 1.0, 0.0)) * sky_scale;
  // The bases see the sky's low light, half of it from the horizon under
  // the sun (its glow: the orange of a sunset's cloud bases).
  let sun_side = clear_sky_light(normalize(vec3<f32>(to_light.x, 0.0, to_light.z) + vec3<f32>(0.0, 1e-3, 0.0)));
  let view_side = clear_sky_light(normalize(vec3<f32>(d.x, 0.0, d.z) + vec3<f32>(0.0, 1e-3, 0.0)));
  let sky_side = mix(view_side, sun_side, 0.5) * sky_scale;
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
  // reach; else what lies past the march's reach, behind the march. One
  // loop (one copy of far_clouds' code: DXC inlines every call).
  let full = end;
  end = min(end, start + far * 0.5);
  var fallback = vec4<f32>(0.0);
  var behind_far = vec4<f32>(0.0);
  for (var j = 0u; j < 2u; j++) {
    let layer = far_clouds(color, d, select(start, end, j == 1u), full, o, to_light, light, physical, phase, sky_top, sky_side);
    if (j == 1u) { behind_far = layer; break; }
    fallback = cloud_layer_mix(layer, deck, deck_cover);
    if (start >= far) { return fallback; }
  }
  // Up to twice the steps towards the horizon, where the path is longest.
  let steps = u32(c[3].z * (2.0 - abs(d.y)));
  let dt = (end - start) / f32(steps);
  // A different start per pixel turns banding into fine noise: interleaved
  // gradient noise (Jimenez 2014), whose neighbours differ evenly, so a
  // single frame shows an even fine grain rather than speckle (TAA
  // smooths it), offset each frame.
  var jitter = ign(pixel + f32(i32(frame.time.y) & 15) * 5.588238);
  // (A pixel of -1: undithered, the march's middle.)
  if (pixel.x < 0.0) { jitter = 0.5; }
  var t = start + dt * jitter;
  var transmittance = 1.0;
  var scattered = vec3<f32>(0.0);
  var depth = 0.0;
  let light_step = c[0].w * 0.12;
  // A lightning strike's flash, a point in the cloud over the bolt's root
  // (frame coordinates in [5].xyz, made planet-centred about the viewer).
  let flash = c[5];
  let flash_at = vec3<f32>(flash.x - frame.view_pos.x, radius + flash.y + frame.origin.y, flash.z - frame.view_pos.z);
  for (var i = 0u; i < steps; i++) {
    let p = o + d * t;
    let h = length(p) - radius;
    let sigma = cloud_density(p, h, false);
    if (sigma > 0.0) {
      let sun_here = cloud_sun(p, radius, to_light, light, physical);
      // The column towards the light, in five widening steps.
      var od = 0.0;
      for (var k = 1u; k <= 5u; k++) {
        let q = p + to_light * light_step * f32(k * k) * 0.4;
        od += cloud_density(q, length(q) - radius, true) * light_step * f32(2u * k - 1u) * 0.4;
      }
      // Three octaves of multiple scattering (Wrenninge 2013), the later
      // ones reaching deep into the shadowed side, so it is lifted above
      // near-black.
      var sun = vec3<f32>(0.0);
      var a = 1.0;
      var b = 1.0;
      for (var m = 0u; m < 3u; m++) {
        sun += sun_here * a * exp(-od * b) * mix(phase, 0.0795775, 1.0 - a);
        a *= 0.6;
        b *= 0.3;
      }
      if (flash.w > 0.0) {
        // The flash lights the cloud from inside about the root: falling
        // off over 600 m and dimmed by the cloud between, scattered evenly
        // (it has bounced many times), no streaks.
        let to = flash_at - p;
        let dist2 = dot(to, to);
        sun += vec3<f32>(0.75, 0.82, 1.0) * flash.w * 0.0795775 * exp(-sqrt(dist2) * c[0].y * 0.03) / (1.0 + dist2 / 3.6e5);
      }
      let hf = clamp((h - c[0].z) / c[0].w, 0.0, 1.0);
      var ambient = mix(sky_side * 0.6, sky_top, hf) * (0.35 + 0.65 * hf);
      // Under a cover of cloud the light between them is the clouds' own,
      // grey, not the clear sky's blue.
      ambient = mix(ambient, vec3<f32>(dot(ambient, vec3<f32>(0.2126, 0.7152, 0.0722))), c[0].x * 0.7);
      let s = cloud_view_fog((sun + ambient) * c[2].rgb, d, t) * sigma;
      let seg = exp(-sigma * dt);
      scattered += transmittance * (s - s * seg) / sigma;
      // The depth the cloud's light comes from, weighted by what it adds.
      depth += t * transmittance * (1.0 - seg);
      transmittance *= seg;
      if (transmittance < 0.01) { break; }
    }
    t += dt;
  }
  // The air between the viewer and the clouds blends them into the sky (or, under an overcast, into the deck's own
  // mean colour along this ray, the fallback's where the ray found little cloud).
  // Fog controls atmospheric extinction, not the finite march's reach.
  // Height fog can leave distant cloud visible: taper coverage to the
  // fallback before the hard start>=far exit, with either fog setting.
  let range_fade = 1.0 - smoothstep(far * 0.5, far, start);
  var marched = vec4<f32>(behind_far.rgb * transmittance + scattered, behind_far.a * transmittance);
  if (physical) {
    marched = cloud_air(behind_far, scattered, transmittance, depth / max(1.0 - transmittance, 1e-4), d, to_light);
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
// before bloom. SDR/AgX and opt-in HDR/LPM own the display peak mapping.
fn sky_stored(color: vec3<f32>) -> vec4<f32> {
  return vec4<f32>(clamp(color, vec3<f32>(0.0), vec3<f32>(65504.0)), 1.0);
}

@fragment
fn fs_sky_look(in: SkyOut) -> @location(0) vec4<f32> {
  if (frame.light_color.w > 0.5) { return debug_unsupported(in.pos.xy); }
  let dir = view_dir(in.ndc);
  let clouds = clouds_over(dir, in.pos.xy, in.grey);
  return sky_stored(cloud_layer_over(clouds, sky_look_behind(dir, in.pos.xy)));
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
  if (textureLoad(clouds_depth, clamp(vec2<i32>(pos * scale), vec2<i32>(0), last), 0).r >= 1.0e8) { return false; }
  for (var y = lo.y; y < hi.y; y++) {
    for (var x = lo.x; x < hi.x; x++) {
      if (textureLoad(clouds_depth, clamp(vec2<i32>(x, y), vec2<i32>(0), last), 0).r >= 1.0e8) { return false; }
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

// The cloud target's texel: `texel` is a texel in NDC.
fn clouds_low_texel(in: SkyOut, texel: vec2<f32>) -> vec4<f32> {
  var d = view_dir(in.ndc);
  // The horizon is an edge in the clouds (a ray that meets the ground sees none, one just over it the farthest)
  // and the sky's draw reads this target between its texels: a texel under the horizon beside one over it marches
  // that one's ray, so a pixel over the horizon reads only clouds over it. A pixel under it reads none
  // (fs_sky_look_low).
  if (cloud_grounded(d)) {
    for (var k = 0u; k < 9u; k++) {
      let near = view_dir(in.ndc + (vec2<f32>(f32(k % 3u), f32(k / 3u)) - 1.0) * texel);
      if (near.y > d.y) { d = near; }
    }
  }
  let layer = clouds_over(d, in.pos.xy, in.grey);
  return vec4<f32>(clamp(layer.rgb, vec3<f32>(0.0), vec3<f32>(65504.0)), layer.a);
}

@fragment
fn fs_clouds_low(in: SkyOut) -> @location(0) vec4<f32> {
  // A derivative: before any branch.
  return clouds_low_texel(in, vec2<f32>(dpdx(in.ndc.x), dpdy(in.ndc.y)));
}

// In a frame with a prepass: nothing where its surfaces cover every pixel that reads the texel.
@fragment
fn fs_clouds_low_depth(in: SkyOut) -> @location(0) vec4<f32> {
  let texel = vec2<f32>(dpdx(in.ndc.x), dpdy(in.ndc.y));
  if (clouds_covered(in.pos.xy, texel)) { return vec4<f32>(0.0, 0.0, 0.0, 1.0); }
  return clouds_low_texel(in, texel);
}

@fragment
fn fs_sky_look_low(in: SkyOut) -> @location(0) vec4<f32> {
  if (frame.light_color.w > 0.5) { return debug_unsupported(in.pos.xy); }
  let dir = view_dir(in.ndc);
  // Under the horizon there are no clouds but an overcast's strip (clouds_over), which has no edge there.
  var clouds = vec4<f32>(0.0, 0.0, 0.0, 1.0);
  if (!cloud_grounded(dir) || cloud_deck(dir).a > 0.0) {
    let uv = vec2<f32>(0.5 + 0.5 * in.ndc.x, 0.5 - 0.5 * in.ndc.y);
    clouds = textureSampleLevel(clouds_low, ibl_sampler, uv, 0.0);
  }
  return sky_stored(cloud_layer_over(clouds, sky_look_behind(dir, in.pos.xy)));
}
