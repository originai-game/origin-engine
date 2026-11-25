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
  authored: vec4<f32>, // xy: box-mip blur biases, zw: authored flags, for map and blend
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
// The clouds' Beer shadow map (clouds/shadow.rs, cloud_shadow_bake.wgsl):
// over the cloud field's period, the cloud along the ray to the sun as a
// slab: r where it begins (of the layer's height, from the top), g where
// it ends, b its optical depth. A texel of nothing until it is baked
// (frame.clouds[11].z).
@group(1) @binding(9) var cloud_light: texture_2d<f32>;

struct SkyOut {
  @builtin(position) pos: vec4<f32>,
  @location(0) ndc: vec2<f32>,
};

fn sky_triangle(i: u32) -> SkyOut {
  var o: SkyOut;
  o.ndc = fullscreen_clip(i).xy;
  o.pos = vec4<f32>(o.ndc, 0.0, 1.0);
  return o;
}

@vertex
fn vs_sky(@builtin(vertex_index) i: u32) -> SkyOut {
  return sky_triangle(i);
}

@fragment
fn fs_sky(in: SkyOut) -> @location(0) vec4<f32> {
  if (frame.light_color.w > 0.5) { return debug_unsupported(in.pos.xy); }
  // The view ray through the pixel: from its near point (perspective and orthographic).
  let near = frame.inv_view_proj * vec4<f32>(in.ndc, 1.0, 1.0);
  let eye = near.xyz / near.w;
  let dir = pixel_ray(in.ndc);
  return vec4<f32>(sky_fog(sky_background(eye, dir) * (1.0 - frame.view_up.w), dir), sky_tone_fraction());
}

// The HDR target's alpha already carries the tone-mapped share for camera
// plates and untonemapped sprites. An authored LDR sky uses that same path.
fn sky_tone_fraction() -> f32 {
  return select(1.0, sky_params.turn.z, sky_params.look.x > 0.5);
}

// The background along a view ray (through the grounded dome when there
// is one, frame.wgsl dome_lookup). HDR uses its GGX blur level. Authored
// LDR colours use ordinary minification and a blur bias on their own box
// mips. Each blended map has its own size and range.
fn sky_background(eye: vec3<f32>, dir: vec3<f32>) -> vec3<f32> {
  if (sky_params.look.x < 0.0) { return sky_params.turn.rgb; }
  let d = dome_lookup(eye, dir);
  if (sky_params.look.x > 0.5) {
    let c = sky_params.turn.x;
    let s = sky_params.turn.y;
    let m = vec3<f32>(c * d.x - s * d.z, d.y, s * d.x + c * d.z);
    var col: vec3<f32>;
    if (sky_params.authored.z > 0.5) {
      col = textureSampleBias(map_cube, ibl_sampler, m, sky_params.authored.x).rgb;
    } else {
      col = textureSampleLevel(map_cube, ibl_sampler, m, sky_params.look.y).rgb;
    }
    if (sky_params.look.w > 0.0) {
      var other: vec3<f32>;
      if (sky_params.authored.w > 0.5) {
        other = textureSampleBias(blend_cube, ibl_sampler, m, sky_params.authored.y).rgb;
      } else {
        other = textureSampleLevel(blend_cube, ibl_sampler, m, sky_params.look.y).rgb;
      }
      col = mix(col, other, sky_params.look.w);
    }
    return col * sky_params.look.z;
  }
  if (sky_params.look.y > 0.0) {
    return textureSampleLevel(prefiltered_map, ibl_sampler, d, sky_params.look.y).rgb;
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
  // From above a cloud base the lower sky is the cloud sea, not the
  // ground's stand-in. Its own fog and aerial perspective already apply.
  if (frame.fog_color.w < 0.5 || frame.drawn.z != 0u || below <= 0.0 || above_cloud_base()) { return color; }
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
// (x and z, wrapped as the wind's offset is), zw the run per metre it
// rises of the ray their shadow is baked along; [11] x the world's shadow
// map's side, y its strength, z 1 when the Beer shadow map holds the
// frame's clouds; [12], [13] the kind's numbers (clouds_shape.wgsl).

// ── the clouds' light ───────────────────────────────────────────────
// Cloud droplets' phase function: the blend of a Henyey-Greenstein and a
// Draine lobe that Jendersie and d'Eon fit to Mie scattering ("An
// Approximate Mie Scattering Function for Fog and Cloud Rendering",
// SIGGRAPH 2023 Talks, equations 2 to 7), at a droplet diameter of 6 µm:
// a narrow forward peak (a cloud's silver lining against the sun) over a
// broad lobe that rises again backwards (a cloud's relief with the sun
// behind the eye, which a forward lobe alone leaves to the ambient light).
const CLOUD_G_HG: f32 = 0.9774;
const CLOUD_G_D: f32 = 0.5212;
const CLOUD_ALPHA: f32 = 18.28;
const CLOUD_W_D: f32 = 0.4595;

// The phase function with its lobes `k` as eccentric: 1 the droplets'
// own; less, light that has scattered more than once (Wrenninge, Kulla
// and Lundqvist, "Oz: The Great and Volumetric", SIGGRAPH 2013 Talks).
fn cloud_phase(cos_t: f32, k: f32) -> f32 {
  return mix(hg_phase(cos_t, CLOUD_G_HG * k), draine_phase(cos_t, CLOUD_G_D * k, CLOUD_ALPHA * k), CLOUD_W_D);
}

// The phase function towards the eye of the sun's light scattered once,
// twice, three and four times (cloud_sunlit), each per steradian: the same
// along a ray.
fn cloud_phases(cos_t: f32) -> vec4<f32> {
  return vec4<f32>(cloud_phase(cos_t, 1.0), cloud_phase(cos_t, 0.5), cloud_phase(cos_t, 0.25), cloud_phase(cos_t, 0.125));
}

// The sun's light a point of cloud scatters towards the eye, per unit of
// that light and of the cloud's extinction there. `od`: the optical depth
// of cloud between the point and the sun; `near`: that of its first tens
// of metres; `deep`: that of the cloud's envelope, the cloud without its
// billows (cloud_light_depth); `phase`: cloud_phases; `inner`: the share
// of the light scattered more than once that reaches the eye from there
// (less at a cloud's skin, where half its directions are clear air, than
// in its body: Schneider's in-scatter probability, which draws the dark
// rims of sunlit billows; never on the light scattered once).
//   - scattered once: Beer's law and the droplets' phase function, which
//     is all of the silver lining;
//   - scattered twice, three and four times: Wrenninge's octaves (2013),
//     each with half the last's extinction, share and eccentricity (Hillaire
//     2016: an octave's share never above its extinction's scale, so none
//     adds energy; at seven times those shares, as they were, a cloud's
//     edge towards the sun was several times a white surface and clipped);
//   - scattered many times: even in every direction, a white cloud's own
//     light (CLOUD_DIFFUSE of the sun's returns, summed along a ray into
//     a thick cloud's sunlit face, about what a white surface does, the
//     sun's light over pi). It crosses the cloud as a diffusing slab lets
//     it through, falling as one over the depth of the envelope (Beer's
//     law for it left the shaded side of a cloud under a low sun black,
//     and the depth of the cloud as drawn showed every lump of a cloud's
//     top on its base, as bands), and is shaded by the cloud's first tens
//     of metres towards the sun: the crease between two billows, and
//     the face of a cloud that turns from the sun, by seven tenths.
fn cloud_sunlit(od: f32, near: f32, deep: f32, phase: vec4<f32>, inner: f32, cos_t: f32) -> f32 {
  // Hillaire/Wrenninge multi-scattering octaves: energy-conserving powers
  // (a^n, b^n, c^n) with a = 0.55, b = 0.5, c = 0.5.
  let again = 0.55 * exp(-0.5 * od) * phase.y
    + 0.3025 * exp(-0.25 * od) * phase.z
    + 0.166 * exp(-0.125 * od) * phase.w;
  // View-dependent powder effect (Nubis²): deepens sunlit billow crevices
  // when looking away from the sun while keeping glowing forward silver linings.
  let powder = mix(1.0, 1.0 - 0.55 * exp(-2.0 * od), clamp(0.5 - 0.5 * cos_t, 0.0, 1.0));
  let many = CLOUD_DIFFUSE * (0.28 + 0.72 * exp(-0.5 * near)) / (1.0 + CLOUD_DIFFUSION * deep);
  return exp(-od) * phase.x + (again * powder + many) * inner;
}

// The sky's and the ground's light on a point of cloud, as the light it
// scatters towards the eye per unit of its extinction (a cloud scatters a
// uniform light back as it is): the mean of what reaches it from above and
// from below. From above: the sky (`up`) through the cloud over the point;
// from below: the ground and the low sky (`down`) through the cloud under
// it; and of each, what the cloud on the point's other side turns back: a
// cloud scatters and does not absorb, so what a depth of it does not let
// through it sends back. A thick cloud's top is as bright as the sky and
// its base as the ground and the low sky that light it. (With each stream
// alone a base took half of the light below it and none other: darker
// than anything it could see, and a sunset's cloud a black body in a gold
// rim.) The light is turned back once: deep in a tall cloud, far from its
// top and its base, neither stream arrives, and a storm's flank is dark.
// (Turned back without end, as in a slab with no sides, the middle of
// any depth of cloud stood as bright as the mean of the sky and the
// ground: a storm's tower with no shade on it.) `tau_up` and `tau_down`:
// those optical depths.
fn cloud_skylit(up: vec3<f32>, down: vec3<f32>, tau_up: f32, tau_down: f32) -> vec3<f32> {
  let through_up = 1.0 / (1.0 + CLOUD_DIFFUSION * tau_up);
  let through_down = 1.0 / (1.0 + CLOUD_DIFFUSION * tau_down);
  return 0.5 * (up * (through_up * (2.0 - through_down)) + down * (through_down * (2.0 - through_up)));
}

// How much of its surroundings a point of cloud's surface sees, from the
// dimensional profile there (Schneider 2022, the ambient light's
// probability): the cloud's surface is where the profile meets the noise,
// so a low profile at it is a billow standing out to the cloud's envelope,
// open to the sky all round, and a high one the bottom of a crease between
// billows, which sees a strip of it. Under a sky with no sun to speak of
// (an overcast, an authored dusk) this is all of a cloud's relief.
fn cloud_open(profile: f32) -> f32 {
  // (Rounded where the profile becomes whole, so the root's slope does
  // not end there at once.)
  return mix(0.55, 1.0, (sqrt(max(1.0 - profile, 0.0) + 0.01) - 0.1) / 0.905);
}

// The ground's light on the clouds over it, as cloud_skylit takes it: a
// ground of a quarter albedo lit by the sun the cover leaves it and the
// sky (`top`). Under a map too: its lower half is the horizon drawn down
// or a photograph's foreground, not what lies under the clouds (lit by it
// as it is, a deck's bases were as bright as its tops).
fn cloud_ground(top: vec3<f32>, sun: vec3<f32>, up: f32) -> vec3<f32> {
  let c = frame.clouds;
  let reach = min(c[4].y, 1.0) * (1.0 - 0.6 * c[0].x);
  return CLOUD_GROUND * (sun * (max(up, 0.0) * reach / PI) + top * cloud_under());
}

// The share of what a cloud's base sees under it that is the air's light,
// the low sky's, and not the ground's: the air between the base and the
// ground it looks down on, a depth of the base's height over the cosine
// of how steeply it looks, lets through exp(-x / mu) of the ground, x the
// base's height in CLOUD_AIR; over every direction under the horizon that
// leaves 1 - E2(x) the air's (fitted). A cumulus base 1500 m up sees two
// fifths air; a deck 180 m over the ground sees the ground, and a sea of
// cloud kilometres over it the air alone. (At two fifths whatever the
// height, a low deck's bases were lit by the horizon all round them, as
// bright as the sky.)
fn cloud_air_share() -> f32 {
  let c = frame.clouds;
  let x = max(c[0].z + c[6].w, 0.0) / CLOUD_AIR;
  return 1.0 - exp(-1.9 * pow(x, 0.78));
}

// How much of the sky's light reaches under the layer: all of it through
// the gaps, and through the clouds what a slab of the layer's depth lets
// through.
fn cloud_under() -> f32 {
  let c = frame.clouds;
  let through = 1.0 / (1.0 + CLOUD_DIFFUSION * c[0].y * c[0].w * 0.7 * c[12].y);
  return 1.0 - c[0].x * (1.0 - through);
}

// The optical depth of cloud between the field's point `at`, `hf` of the
// way up the layer, and the sun, read from the Beer shadow map
// (cloud_shadow_bake.wgsl): the cloud along the ray the map was baked
// along as the slab of even density with its moments, and the share of
// that slab over the point. x: of the cloud as it is drawn; y: of its
// envelope. `slant`: the sun's own ray's length over that ray's.
fn cloud_light_depth(at: vec3<f32>, hf: f32, slant: f32, exact: bool) -> vec2<f32> {
  let c = frame.clouds;
  let base = at.xz - c[10].zw * (hf * c[0].w);
  var m = textureSampleLevel(cloud_light, cloud_sampler, base / (4.0 * c[1].w), 0.0);
  if (exact) { m = cloud_cubic(cloud_light, base / (4.0 * c[1].w)); }
  if (m.r <= 1e-3) { return vec2<f32>(0.0); }
  let mean = m.g / m.r;
  // An even slab's half height is the square root of three times its
  // spread; never thinner than a step of the bake.
  let half = sqrt(3.0 * max(m.b / m.r - mean * mean, 0.0)) + 0.01;
  return vec2<f32>(m.r, m.a) * (clamp((mean + half - (hf - 0.5)) / (2.0 * half), 0.0, 1.0) * slant);
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

// The key light reaching `p` (planet-centred metres, the planet of
// `radius`): under the physical sky the key above the air (cloud_key)
// through its transmittance at that height, else the scene's `light`.
fn cloud_sun(p: vec3<f32>, radius: f32, to_light: vec3<f32>, light: vec3<f32>, physical: bool) -> vec3<f32> {
  if (!physical) { return light; }
  let r_km = frame.sky_scatter[3].x + (length(p) - radius + frame.clouds[6].w) * 0.001;
  return light * scatter_sun_transmittance(r_km, dot(normalize(p), to_light));
}

// The light that keys the clouds under the physical sky: towards it, and
// its light above the air.
struct CloudKey {
  dir: vec3<f32>,
  light: vec3<f32>,
}

// The brighter of the sun and the moon as they reach the cloud base, the
// other's handed over as og.DayNight hands over the scene's key light
// (DayNightLook::keyed): the winner dimmed to nothing where the two are
// equal, over a fifth of their sum either side, so the clouds' light
// passes from the setting sun to the moon without a jump. Each above the
// air at 3 times the look's colour (the day-night noon). The clouds keep
// one light column: at night the sun's added nothing.
fn cloud_key() -> CloudKey {
  let sun = normalize(frame.sky_sun_dir.xyz);
  let sun_light = 3.0 * frame.sky_glow.yzw;
  let moon_light = 3.0 * sky_moon_light();
  if (frame.sky_moon.w <= 0.0) { return CloudKey(sun, sun_light); }
  let moon = normalize(frame.sky_moon.xyz);
  let base = frame.sky_scatter[3].x + (frame.clouds[0].z + frame.clouds[6].w) * 0.001;
  let s = dot(sun_light * scatter_sun_transmittance(base, sun.y), LUMA);
  let m = dot(moon_light * scatter_sun_transmittance(base, moon.y), LUMA);
  let swap = smoothstep(0.0, 0.2, abs(s - m) / max(s + m, 1e-30));
  if (m > s) { return CloudKey(moon, moon_light * swap); }
  return CloudKey(sun, sun_light * swap);
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
// whose march reaches the deck within the march's reach (the base some 24 km off) fades from its
// own column's march at that elevation (undithered) into the analytical far deck colour (`cloud_deck`)
// towards and below the horizon, with no repeated feature columns and no vertex-shader raymarch.
fn clouds_over(d: vec3<f32>, pixel: vec2<f32>, temporal_dither: bool) -> vec4<f32> {
  let c = frame.clouds;
  if (c[3].w < 0.5) { return vec4<f32>(0.0, 0.0, 0.0, 1.0); }
  let viewer = cloud_viewer();
  let dk = cloud_deck(d);
  let cover = dk.a;
  let lowest = clamp((c[0].z - viewer) / 24000.0, 0.002, 0.2);
  let low = cover > 0.0 && viewer <= c[0].z && d.y < lowest;
  // Both marches in one loop (one copy of the march's code: DXC inlines
  // every call): under the deck's strip the deck along this column's
  // azimuth at the lowest elevation, undithered (k 0); then the column's
  // own (1), unless the deck covers it.
  var own = vec4<f32>(0.0);
  var column = vec4<f32>(0.0, 0.0, 0.0, 1.0);
  for (var k = 0u; k < 2u; k++) {
    if (k == 0u && !low) { continue; }
    if (k == 1u && low && cover >= 0.999) { break; }
    var ray = d;
    var at = pixel;
    var td = temporal_dither;
    if (k == 0u) {
      let flat_d = normalize(vec3<f32>(d.x, 0.0, d.z) + vec3<f32>(1e-6, 0.0, 0.0));
      ray = flat_d * sqrt(1.0 - lowest * lowest) + vec3<f32>(0.0, lowest, 0.0);
      at = vec2<f32>(-1.0);
      td = false;
    }
    let m = clouds_march(ray, at, td);
    if (k == 0u) { own = m; } else { column = m; }
  }
  if (!low) { return column; }
  let far_deck = vec4<f32>(cloud_view_fog(dk.rgb, d, 60000.0), 0.0);
  let deck = mix(own, far_deck, smoothstep(0.0, 0.55, (lowest - d.y) / lowest));
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
// front (no distance heuristic): lit as the sky behind is, by the sun, the
// moon and the night's floor (by the sun alone, a far cloud cut a black
// band out of a moonlit sky). The clouds shade that air (as UE's cloud
// shadow does its aerial perspective): the sun reaches it as it reaches
// the ground (Clouds::light, [4].y), and under a closed deck its sky is
// the deck, lighting it from the upper half of its directions only, so it
// scatters half the deck's radiance as much as it hides what is behind
// it, spectrally: a deck seen far off stays the deck's grey, darker than
// overhead as an overcast's horizon is.
fn cloud_air(color: vec4<f32>, scattered: vec3<f32>, transmittance: f32, at: f32, d: vec3<f32>, to_light: vec3<f32>, light: vec3<f32>) -> vec4<f32> {
  let c = frame.clouds;
  // Its samples by the path's length, 4 to 12 over 100 km (Hillaire 2020's
  // sample count for the sky's rays): a cloud some kilometres off has as
  // little air to integrate in front of it as it varies over.
  let steps = u32(round(mix(4.0, 12.0, clamp(at * 1e-5, 0.0, 1.0))));
  let sun_dir = normalize(frame.sky_sun_dir.xyz);
  let moon_light = sky_moon_light();
  let moon_dir = select(sun_dir, normalize(frame.sky_moon.xyz), frame.sky_moon.w > 0.0);
  let air = air_path(d, sun_dir, moon_dir, moon_light, sky_viewer_km(), at * 0.001, steps);
  // The deck's base as cloud_deck lights it (the clear sky's low light,
  // a fiftieth of the key), its key the one above the air that lights the
  // clouds (cloud_key, `to_light` and `light`), not the scene's light:
  // that is the clouds' own dimming of it only where the scene was lit by
  // Clouds::light_scene, and a lightning flash turns it towards the bolt.
  let sun_side = clear_sky_light(normalize(vec3<f32>(to_light.x, 0.0, to_light.z) + vec3<f32>(0.0, 1e-3, 0.0)));
  let view_side = clear_sky_light(normalize(vec3<f32>(d.x, 0.0, d.z) + vec3<f32>(0.0, 1e-3, 0.0)));
  let above = light * scatter_sun_transmittance(frame.sky_scatter[3].x + (c[0].z + c[6].w) * 0.001, to_light.y);
  // Keep dusk/sunset sky hue intact, only gently softening deep overcast decks.
  let lit_base = (mix(view_side, sun_side, 0.5) * 0.21 + above * 0.02) * c[2].rgb * 2.0;
  let base = mix(lit_base, vec3<f32>(dot(lit_base, LUMA)), 0.35 * smoothstep(0.65, 1.0, c[0].x));
  let cover = cloud_deck(vec3<f32>(0.0, 1.0, 0.0)).a;
  let scale = frame.sky_glow.yzw * frame.sky_horizon.w;
  let under = 0.5 * base * (vec3<f32>(1.0) - air.through);
  // The moonlit air and the night's floor (sky_physical_look's
  // sky_scatter[5], in the share this path holds of it) as its sky light:
  // the clouds hide the moon as they hide that sky, and a closed deck
  // replaces both with its own.
  let night = (air.moon + frame.sky_scatter[5].rgb * (vec3<f32>(1.0) - air.through)) * frame.sky_horizon.w;
  let air_light = air.sun * scale * min(c[4].y, 1.0) + mix(air.sky * scale + night, under, cover);
  return vec4<f32>(color.rgb * transmittance + scattered * air.through + air_light * (1.0 - transmittance) * (1.0 - frame.view_up.w), color.a * transmittance);
}

// The fine steps to a stride inside a cloud (clouds_march).
const CLOUD_FINE: u32 = 5u;
// Energy-conserving diffuse back-scatter term (~1/pi minus multi-scatter
// octaves' share) and diffusion falloff (0.75 * (1 - g), g = 0.85).
const CLOUD_DIFFUSE: f32 = 0.26;
const CLOUD_DIFFUSION: f32 = 0.1125;
// The albedo of the ground under a procedural sky's clouds (cloud_ground).
const CLOUD_GROUND: f32 = 0.25;
// The depth of air in which the ground seen through it gives way to the
// air's own light (cloud_air_share): a hazy day's.
const CLOUD_AIR: f32 = 5000.0;

// Soft shoulder ceiling on the forward silver-lining peak so grazing backlit
// rims stay luminous without hard clipping or single-sample sparkle.
const CLOUD_PEAK: f32 = 6.0;
// How much of what is behind a sample the cloud in front of it lets
// through, under which the light's near samples are one (clouds_march).
const CLOUD_BEHIND: f32 = 0.5;
// What a cloud lets through of what is behind it where the march stops:
// the samples past there are the dearest (lit, in the cloud's depth) and
// add a fiftieth of the pixel at most.
const CLOUD_OPAQUE: f32 = 0.02;
// How far the march reaches: past it the air has taken the clouds.
const CLOUD_REACH: f32 = 160000.0;
// The distance a cloud fades into a sky that has no air to integrate over
// (a gradient, a map), at most.
const CLOUD_HAZE: f32 = 27000.0;

// The high layer, then the volumetric layer, faded into the sky with distance.
fn clouds_march(d: vec3<f32>, pixel: vec2<f32>, temporal_dither: bool) -> vec4<f32> {
  let c = frame.clouds;
  if (c[3].w < 0.5) { return vec4<f32>(0.0, 0.0, 0.0, 1.0); }
  let radius = c[4].x;
  let viewer = cloud_viewer();
  let o = vec3<f32>(0.0, radius + viewer, 0.0);
  // A ray that meets the ground from below the cloud base sees no clouds.
  // From above the base (mountain or flight) it looks down onto the layer
  // and stops before the planet surface.
  let earth = cloud_sphere(o, d, radius - c[6].w);
  if (viewer <= c[0].z && earth.x > 0.0) { return vec4<f32>(0.0, 0.0, 0.0, 1.0); }
  // Under the physical sky the clouds are lit by their key, the sun or by
  // night the moon (cloud_key), at their own height: the key above the
  // air through the sky's own transmittance down to the sample, so a
  // sunset's low clouds glow orange to pink from below, they stay lit a
  // while after the ground's sunset and a moon lights their undersides
  // facing it; never the scene's directional light, which a lightning
  // flash turns towards the bolt.
  // Otherwise the scene's sun, whose dimming by the clouds' own cover
  // (Clouds::light, [4].y) is undone: they are lit from above it.
  let physical = scatter_on();
  var to_light = normalize(frame.light_dir.xyz);
  var light = frame.light_color.rgb / max(c[4].y, 9.1e-4);
  if (physical) {
    let key = cloud_key();
    to_light = key.dir;
    light = key.light;
  }
  let cos_t = dot(d, to_light);
  var phase = cloud_phases(cos_t);
  // Smooth shoulder on forward Mie peak instead of a hard clamp.
  phase.x = phase.x / (1.0 + phase.x / CLOUD_PEAK);
  // Clear-sky light from above and the horizon (sky_look.wgsl clear_sky_light). Maps use their SH ambient and
  // Environment::intensity before the cover dims it. A procedural sky uses its cloud-free samples at the drawn
  // sky's strength, never the live IBL capture, its strength on surfaces or its cover's dimming.
  let clear_top = clear_sky_light(vec3<f32>(0.0, 1.0, 0.0));
  // An aurora over them lights them too, as it lights the night (aurora_cloud_light): the high layer and the far
  // one take it with the sky's, the march through the cloud below.
  let aurora_top = aurora_cloud_light(0);
  let aurora_side = aurora_cloud_light(1);
  let sky_top = clear_top + aurora_top;
  // The sky's low light, half of it from the horizon under the sun (its
  // glow: the orange of a sunset's cloud bases).
  let sun_side = low_sky_light(normalize(vec3<f32>(to_light.x, 0.0, to_light.z) + vec3<f32>(0.0, 1e-3, 0.0)));
  let view_side = low_sky_light(normalize(vec3<f32>(d.x, 0.0, d.z) + vec3<f32>(0.0, 1e-3, 0.0)));
  let sky_side = mix(view_side, sun_side, 0.5);
  let bottom = radius + c[0].z;
  // What lights a cloud from above, and from below: the ground, and
  // through the air under the layer the low sky.
  let sky_up = mix(clear_top, sky_side, 0.35);
  let lit_ground = cloud_ground(sky_up, cloud_sun(vec3<f32>(0.0, bottom, 0.0), radius, to_light, light, physical), to_light.y);
  let sky_low = mix(lit_ground, sky_side, cloud_air_share());
  let overhead = clamp(to_light.y, 0.0, 1.0);
  var color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
  let dk = cloud_deck(d);
  let deck_cover = dk.a;
  let deck = cloud_view_fog(dk.rgb, d, 60000.0);
  let r = length(o);

  // The high thin layer (cirrus), streaked along the active wind heading.
  if (c[3].x > 0.0) {
    let cirrus_r = radius + c[3].y;
    let hit = cloud_sphere(o, d, cirrus_r);
    let t = select(hit.y, hit.x, r > cirrus_r && hit.x > 0.0);
    if (t > 0.0 && (earth.x <= 0.0 || t < earth.x)) {
      let p = o + d * t;
      let raw_uv = (vec2<f32>(p.x, p.z) + c[14].zw) / (c[1].w * 2.0);
      // Rotate cirrus coordinates into the wind direction stored in c[11].w
      // so wispy fibres align naturally with wind drift.
      let wa = c[11].w;
      let cw = cos(wa);
      let sw = sin(wa);
      let uv = vec2<f32>(raw_uv.x * cw + raw_uv.y * sw, -raw_uv.x * sw + raw_uv.y * cw);
      // Subtle lateral curl warp breaks up linear streaks into natural mare's tails.
      let warp = (textureSampleLevel(cloud_noise, cloud_sampler, vec3<f32>(uv * 0.125, 0.23), 1.0).rg - 0.5) * 0.18;
      let wuv = uv + warp;
      let patches = textureSampleLevel(cloud_noise, cloud_sampler, vec3<f32>(wuv.x * 0.125 + 0.5, wuv.y * 0.25, 0.71), 1.0).b;
      let fibres = textureSampleLevel(cloud_noise, cloud_sampler, vec3<f32>(wuv.x * 0.22, wuv.y * 0.95, 0.37), 1.0).g;
      let wisp = cloud_remap(patches, 0.74 - 0.5 * c[3].x, 0.74) * (0.28 + 0.72 * smoothstep(0.22, 0.78, fibres));
      let fade = exp(-t / 120000.0);
      // Ice crystals: broad forward halo plus soft back-scatter.
      let ice = mix(hg_phase(cos_t, 0.8), hg_phase(cos_t, -0.25), 0.4);
      let lit = cloud_sun(p, radius, to_light, light, physical) * (0.25 + ice * 1.5) * 0.2 + sky_top * 0.8;
      color = cloud_layer_mix(color, cloud_view_fog(lit * c[2].rgb, d, t), clamp(wisp, 0.0, 1.0) * 0.6 * fade);
    }
  }

  // The volumetric layer between the shells at base and base + thickness
  // (no higher than the kind's clouds stand: a sheet is a quarter of it).
  if (c[0].w <= 0.0 || c[0].y <= 0.0) { return color; }
  let stand = c[12].y;
  let top = bottom + c[0].w * stand;
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
  if (viewer > c[0].z && earth.x > 0.0) {
    end = min(end, earth.x);
  }
  let fallback = cloud_layer_mix(color, deck, deck_cover);
  if (start >= CLOUD_REACH) { return fallback; }
  end = min(end, CLOUD_REACH);
  if (start >= end) { return color; }
  let steps = c[3].z;
  let grow = 1.0 / steps;
  let least = c[0].w / (0.6 * steps);
  let spread = length(frame.view_rays[1].xyz) / 270.0;
  // When temporal cloud reprojection is active (`temporal_dither`), `pixel`
  // already carries `dither_turn(phase)` from `clouds_march_at`; skip the
  // TAA `frame.shadow.w` offset so the two dither sequences do not collide.
  let taa_dither = select(max(frame.shadow.w - 1.0, 0.0) * 5.588238, 0.0, temporal_dither);
  var jitter = ign(pixel + taa_dither);
  var light_jitter = ign(pixel.yx + 31.7 + taa_dither);
  if (pixel.x < 0.0) {
    jitter = 0.5;
    light_jitter = 0.5;
  }
  var t = start + max(start * grow, least) * jitter;
  var dt = least;
  var dt0 = max(t * grow, least) / f32(CLOUD_FINE);
  var fine = 0u;
  var strode = true;
  var lengthened = false;
  var recross = 0.0;
  var transmittance = 1.0;
  var scattered = vec3<f32>(0.0);
  var depth = 0.0;
  let light_step = c[0].w * 0.012;
  // Cap Beer shadow-map slant scaling to 1.35 and smoothly cross-fade into
  // a true-sun geometric light march below ~13 degrees elevation.
  let raw_slant = min(1.0 / max(to_light.y, 0.05), 20.0) / sqrt(1.0 + dot(c[10].zw, c[10].zw));
  let slant = min(raw_slant, 1.35);
  let low_sun = 1.0 - smoothstep(0.06, 0.22, to_light.y);
  let mapped = c[11].z > 0.5;
  let flash = c[5];
  let flash_at = vec3<f32>(flash.x - frame.view_pos.x, radius + flash.y + frame.origin.y, flash.z - frame.view_pos.z);
  let span = max(end - start, 1.0);
  var sun_at: array<vec3<f32>, 4>;
  for (var k = 0u; k < 4u; k++) {
    let up = normalize(o + d * select(start, end, k >= 2u));
    sun_at[k] = cloud_sun(up * select(bottom, bottom + c[0].w, k % 2u == 1u), radius, to_light, light, physical);
  }
  let fog_far = cloud_fog(d, 5000.0);
  for (var i = 0u; i < u32(steps) * 8u; i++) {
    if (t >= end) { break; }
    let p = o + d * t;
    let h = length(p) - radius;
    let hf = (h - c[0].z) / c[0].w;
    let at = cloud_noise_at(p, h);
    if (fine == 0u) {
      let far_col = cloud_column(cloud_map_at(at, false), hf);
      let far_profile = cloud_profile(far_col, hf);
      let stride = max(t * grow, least);
      if (far_profile <= 0.0 || far_profile <= cloud_coarse(at, cloud_up(far_col, hf), stride * 0.5)) {
        t += stride;
        strode = true;
        continue;
      }
      dt0 = stride / f32(CLOUD_FINE);
      dt = dt0;
      lengthened = false;
      fine = CLOUD_FINE + 1u;
      if (strode) {
        t = max(t - stride, start) + dt * jitter;
        strode = false;
        continue;
      }
    }
    // Keep fine step adaptive to current ray distance across deep cloud paths.
    dt0 = max(t * grow, least) / f32(CLOUD_FINE);
    let behind = transmittance < CLOUD_BEHIND;
    let m = cloud_map_at(at, !behind);
    let col = cloud_column(m, hf);
    let profile = cloud_profile(col, hf);
    var cloud = vec3<f32>(0.0);
    if (profile > 0.0) { cloud = cloud_density_from(at, col, hf, profile, t * spread, t * spread * 128.0 < c[1].w); }
    let sigma = cloud.x;
    if (sigma > 0.0 && lengthened) {
      recross = t;
      t = t - dt + dt0 * jitter;
      dt = dt0;
      fine = CLOUD_FINE + 1u;
      lengthened = false;
      continue;
    }
    fine -= 1u;
    if (sigma > 0.0) {
      lengthened = false;
      fine = CLOUD_FINE + 1u;
      let along = clamp((t - start) / span, 0.0, 1.0);
      let sun_here = mix(mix(sun_at[0], sun_at[1], hf), mix(sun_at[2], sun_at[3], hf), along);
      var od = 0.0;
      var deep = 0.0;
      let reach_near = 3.0 * light_step;
      if (behind) {
        od = sigma * reach_near;
      } else {
        let q_near = p + to_light * ((0.25 + 0.5 * light_jitter) * reach_near);
        let qn_h = length(q_near) - radius;
        let qn_hf = (qn_h - c[0].z) / c[0].w;
        let qn_at = cloud_noise_at(q_near, qn_h);
        let q_col = cloud_column(cloud_map_at(qn_at, true), qn_hf);
        let q_profile = cloud_profile(q_col, qn_hf);
        if (q_profile > 0.0) {
          od = cloud_density_from(qn_at, q_col, qn_hf, q_profile, 1.5 * light_step, false).x * reach_near;
        }
      }
      let near = od;
      let q = p + to_light * (3.0 * light_step);
      let q_hf = (length(q) - radius - c[0].z) / c[0].w;
      let need_march = !mapped || (low_sun > 0.01 && !behind);
      var march_od = od;
      if (need_march) {
        var reach = 4.0 * light_step;
        for (var k = 0u; k < 4u; k++) {
          let far_q = p + to_light * (reach * 1.5 - light_step);
          let far_h = length(far_q) - radius;
          march_od += cloud_density_in(cloud_noise_at(far_q, far_h), (far_h - c[0].z) / c[0].w, reach, false) * reach;
          reach *= 2.0;
        }
      }
      if (mapped) {
        var map_od = od;
        var map_deep = od;
        if (q_hf < 1.0) {
          let far_od = cloud_light_depth(cloud_noise_at(q, length(q) - radius), max(q_hf, 0.0), slant, !behind);
          map_deep = od + far_od.y;
          map_od = od + far_od.x;
        }
        if (need_march) {
          od = mix(map_od, march_od, low_sun);
          deep = mix(map_deep, march_od, low_sun);
        } else {
          od = map_od;
          deep = map_deep;
        }
      } else {
        od = march_od;
        deep = march_od;
      }
      let within = mix(0.6, 1.0, smoothstep(0.0, 0.3, cloud.y));
      var sun = sun_here * cloud_sunlit(od, near, deep, phase, within, cos_t);
      if (flash.w > 0.0) {
        let to = flash_at - p;
        let dist2 = dot(to, to);
        sun += vec3<f32>(0.75, 0.82, 1.0) * flash.w * (0.25 / PI) * exp(-sqrt(dist2) * c[0].y * 0.03) / (1.0 + dist2 / 3.6e5);
      }
      let solid = c[0].y * c[0].w * 0.7 * cloud_ease(col.inside / CLOUD_SIDE);
      let tau_up = solid * max(col.top - hf, 0.0);
      let tau_down = solid * max(hf, 0.0) * cloud_base(col, hf);
      let open = cloud_open(profile);
      let below = sky_low * (open * (1.0 - overhead * (1.0 - 1.0 / (1.0 + CLOUD_DIFFUSION * solid * col.top))));
      var ambient = cloud_skylit(sky_up * open, below, tau_up, tau_down);
      // Only softly mute chroma deep inside heavy overcast cores so sunset and
      // sky-lit cumulus keep rich blue/amber atmospheric colour.
      let storm_grey = smoothstep(0.65, 1.0, c[0].x) * smoothstep(0.3, 1.0, tau_up * 0.2) * 0.35;
      ambient = mix(ambient, vec3<f32>(luma(ambient)), storm_grey);
      ambient += (aurora_top / (1.0 + CLOUD_DIFFUSION * tau_up) + aurora_side * 0.6 * (1.0 - c[0].x)) * open;
      var fog = fog_far;
      if (t < 5000.0) { fog = cloud_fog(d, t); }
      let s = mix((sun + ambient) * c[2].rgb * (1.0 - frame.view_up.w), fog.rgb, fog.a) * sigma;
      let seg = exp(-sigma * dt);
      scattered += transmittance * (s - s * seg) / sigma;
      depth += t * transmittance * (1.0 - seg);
      transmittance *= seg;
      if (transmittance < CLOUD_OPAQUE) { break; }
      dt = dt0 * (1.0 + 6.0 * (1.0 - transmittance));
    } else {
      if (t >= recross) {
        let clear = min(cloud_clear(cloud.z), max(t * grow, least));
        let fine_step = dt0 * (1.0 + 6.0 * (1.0 - transmittance));
        lengthened = clear > fine_step * 1.2;
        dt = max(fine_step, clear);
      } else {
        lengthened = false;
      }
    }
    t += dt;
  }
  transmittance = max(transmittance - CLOUD_OPAQUE, 0.0) / (1.0 - CLOUD_OPAQUE);
  let range_fade = 1.0 - smoothstep(CLOUD_REACH * 0.6, CLOUD_REACH, start);
  var at = start;
  if (transmittance < 0.999) { at = depth / (1.0 - transmittance); }
  var marched = vec4<f32>(color.rgb * transmittance + scattered, color.a * transmittance);
  if (physical) {
    if (transmittance < 1.0) { marched = cloud_air(color, scattered, transmittance, at, d, to_light, light); }
  } else {
    let haze = select(CLOUD_HAZE, clamp((bottom - r) * 19.1, 2000.0, CLOUD_HAZE), r < bottom);
    let fade = exp(-at / haze);
    let mean = mix(deck, scattered / max(1.0 - transmittance, 1e-3), smoothstep(0.05, 0.5, 1.0 - transmittance));
    marched = mix(cloud_layer_mix(color, mean, deck_cover), marched, fade);
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

// The aurora's light on the clouds, as clear_sky_light holds the sky's: k 0 on a surface facing up, 1 on one facing
// the horizon (the mean round it). The footprint pass sums it from the whole display each frame (aurora_footprint.wgsl
// `light`), per line into the texture's last row, so a cloud under a bright display is lit green from above and by
// the red of its tops: a night cloud is the aurora's grey-green, never a black cut-out against it.
fn aurora_cloud_light(k: i32) -> vec3<f32> {
  let a = frame.aurora;
  let row = i32(a[1].w) - 1;
  if (a[0].y < 0.5 || i32(textureDimensions(aurora_map).y) <= row) { return vec3<f32>(0.0); }
  let l = textureLoad(aurora_map, vec2<i32>(k, row), 0).xyz;
  return max(l.x * a[2].rgb + l.y * a[3].rgb + l.z * a[4].rgb, vec3<f32>(0.0));
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
  if (frame.sky_zenith.w < 0.5) {
    return sky_fog(sky_background(frame.view_pos.xyz, dir) * (1.0 - frame.view_up.w), dir);
  }
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

// The same cloud layer over a procedural, HDR, authored LDR or colour
// background. Clear authored pixels retain their untonemapped share;
// cloud scattering uses the HDR share, like the scene's other light.
fn sky_clouds_stored(clouds: vec4<f32>, background: vec3<f32>, dir: vec3<f32>) -> vec4<f32> {
  let color = cloud_layer_over(clouds, background);
  if (frame.sky_zenith.w > 0.5) { return sky_stored(land_fog(color, dir)); }
  var stored = sky_stored(color);
  stored.a = mix(1.0, sky_tone_fraction(), clouds.a);
  return stored;
}

@fragment
fn fs_sky_look(in: SkyOut) -> @location(0) vec4<f32> {
  if (frame.light_color.w > 0.5) { return debug_unsupported(in.pos.xy); }
  let dir = pixel_ray(in.ndc);
  let clouds = clouds_over(dir, in.pos.xy, false);
  return sky_clouds_stored(clouds, sky_look_behind(dir, in.pos.xy), dir);
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
  let levels = i32(textureNumLevels(clouds_depth));
  if (levels > 1) {
    // depth_mips weights toward far (sky = FAR_DEPTH has infinite priority):
    // if any full-res pixel in a mip block is sky, the mip texel is sky.
    let mip = clamp(i32(ceil(log2(max(max(scale.x, scale.y), 1.0)))), 0, levels - 1);
    let mlast = vec2<i32>(textureDimensions(clouds_depth, mip)) - 1;
    let mlo = clamp(max(lo, vec2<i32>(0)) >> vec2<u32>(u32(mip)), vec2<i32>(0), mlast);
    let mhi = clamp(max(hi - 1, vec2<i32>(0)) >> vec2<u32>(u32(mip)), vec2<i32>(0), mlast);
    for (var y = mlo.y; y <= mhi.y; y++) {
      for (var x = mlo.x; x <= mhi.x; x++) {
        if (is_sky_depth(textureLoad(clouds_depth, vec2<i32>(x, y), mip).r)) { return false; }
      }
    }
    return true;
  }
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
  let viewer = cloud_viewer();
  return viewer <= c[0].z && cloud_sphere(vec3<f32>(0.0, c[4].x + viewer, 0.0), d, c[4].x - c[6].w).x > 0.0;
}

// The cloud history's step (clouds.rs `CloudsTemporal::prepare`): the march and the resolve of a frame that reprojects.
struct CloudsTemporal {
  // The unjittered view-projection of the previous frame.
  prev_view_proj: mat4x4<f32>,
  // xy: the cloud target's size, zw: which texel of each two by two the march takes this frame.
  size: vec4<f32>,
  // x: 1 when the march takes one texel of each two by two (into the quarter target), 0 every texel; y: the
  // march's dither's turn this frame (0 without a history, which would not average them).
  mode: vec4<f32>,
  // xyz: frame-space displacement to previous frame (`origin_delta - wind_delta`); w: 1 when valid.
  shift: vec4<f32>,
};
@group(2) @binding(2) var<uniform> clouds_temporal: CloudsTemporal;
// The resolve's reads: this frame's march at a quarter of the cloud target's texels, and the history.
@group(2) @binding(3) var clouds_quarter: texture_2d<f32>;
@group(2) @binding(4) var clouds_history: texture_2d<f32>;

// The cloud target's texel along `ndc`, at `pos` of the target: `texel` is a texel in NDC.
fn clouds_low_texel(ndc: vec2<f32>, pos: vec2<f32>, texel: vec2<f32>, temporal_dither: bool) -> vec4<f32> {
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
  let layer = clouds_over(d, pos, temporal_dither);
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
  // (The position is the dither's: the quarter target's own texel, where neighbours are a texel apart as the
  // noise is made for. At the cloud target's texel, two apart, the noise of a row's texels rose a ninth at a
  // time: runs of texels that started their march alike, which drew streaks along the rows of a cloud's base.
  // It is moved by the frame's turn (CloudsTemporal: a part of a texel, which moves the noise's value by as much
  // of its range), so the resolve's blend of each march into the history averages the grain with any
  // antialiasing or none.)
  return MarchAt(ndc, in.pos.xy + clouds_temporal.mode.y, vec2<f32>(2.0, -2.0) / size);
}

@fragment
fn fs_clouds_low(in: SkyOut) -> @location(0) vec4<f32> {
  // A derivative: before any branch.
  let m = clouds_march_at(in);
  return clouds_low_texel(m.ndc, m.pos, m.texel, clouds_temporal.mode.x > 0.5);
}

// In a frame with a prepass: nothing where its surfaces cover every pixel that reads the texel.
@fragment
fn fs_clouds_low_depth(in: SkyOut) -> @location(0) vec4<f32> {
  let m = clouds_march_at(in);
  // Marching one texel of a two by two: nothing only where every pixel that reads any of the four is covered (the
  // resolve fills the other three from this one). When resolving, mark covered texels with r < 0 so
  // fs_clouds_resolve excludes them from the 3x3 clamp and never bleeds black halos onto foreground silhouettes.
  let temporal = clouds_temporal.mode.x > 0.5;
  let block = select(1.0, 2.0, temporal);
  if (clouds_covered(in.pos.xy, m.texel * block)) {
    return select(vec4<f32>(0.0, 0.0, 0.0, 1.0), vec4<f32>(-1.0, 0.0, 0.0, 1.0), temporal);
  }
  return clouds_low_texel(m.ndc, m.pos, m.texel, temporal);
}

// Temporal upsampling resolve (Schneider 2015) with finite-distance cloud shell,
// wind/origin shift reprojection and foreground-silhouette exclusion.
@fragment
fn fs_clouds_resolve(in: SkyOut) -> @location(0) vec4<f32> {
  let at = vec2<i32>(in.pos.xy);
  let quarter = vec2<i32>(textureDimensions(clouds_quarter)) - 1;
  let q = min(at / 2, quarter);
  let fresh = textureLoad(clouds_quarter, q, 0);
  if (fresh.r < 0.0) { return vec4<f32>(0.0, 0.0, 0.0, 1.0); }
  var lo = fresh;
  var hi = fresh;
  for (var k = 0; k < 9; k++) {
    let n = textureLoad(clouds_quarter, clamp(q + vec2<i32>(k % 3 - 1, k / 3 - 1), vec2<i32>(0), quarter), 0);
    if (n.r >= 0.0) {
      lo = min(lo, n);
      hi = max(hi, n);
    }
  }
  let dir = pixel_ray(in.ndc);
  var prev = clouds_temporal.prev_view_proj * vec4<f32>(dir, 0.0);
  if (clouds_temporal.shift.w > 0.5 && frame.clouds[3].w > 0.5 && frame.clouds[0].w > 0.0) {
    let c = frame.clouds;
    let radius = c[4].x;
    let viewer = cloud_viewer();
    let o = vec3<f32>(0.0, radius + viewer, 0.0);
    let mid_r = radius + c[0].z + 0.35 * c[0].w * c[12].y;
    let hit = cloud_sphere(o, dir, mid_r);
    let cloud_dist = select(hit.y, hit.x, length(o) > mid_r && hit.x > 0.0);
    if (cloud_dist > 0.0 && cloud_dist < CLOUD_REACH) {
      let p_prev = frame.view_pos.xyz + dir * cloud_dist + clouds_temporal.shift.xyz;
      prev = clouds_temporal.prev_view_proj * vec4<f32>(p_prev, 1.0);
    }
  }
  let uv = vec2<f32>(0.5, -0.5) * prev.xy / max(prev.w, 1e-6) + 0.5;
  var out = fresh;
  if (prev.w > 0.0 && all(uv >= vec2<f32>(0.0)) && all(uv <= vec2<f32>(1.0))) {
    let seen = clamp(textureSampleLevel(clouds_history, ibl_sampler, uv, 0.0), lo, hi);
    let marched = all((at & vec2<i32>(1)) == vec2<i32>(clouds_temporal.size.zw));
    out = select(seen, mix(seen, fresh, 1.0 / 3.0), marched);
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
  return sky_clouds_stored(clouds, sky_look_behind(dir, in.pos.xy), dir);
}
