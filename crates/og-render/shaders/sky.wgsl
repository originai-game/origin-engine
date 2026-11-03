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

struct SkyOut { @builtin(position) pos: vec4<f32>, @location(0) ndc: vec2<f32> };

@vertex
fn vs_sky(@builtin(vertex_index) i: u32) -> SkyOut {
  var o: SkyOut;
  let x = f32(i32(i & 1u) * 4 - 1);
  let y = f32(i32(i & 2u) * 2 - 1);
  o.pos = vec4<f32>(x, y, 1.0, 1.0);
  o.ndc = vec2<f32>(x, y);
  return o;
}

@fragment
fn fs_sky(in: SkyOut) -> @location(0) vec4<f32> {
  // The view ray through the pixel: near point to far point (perspective and orthographic).
  let near = frame.inv_view_proj * vec4<f32>(in.ndc, 0.0, 1.0);
  let far = frame.inv_view_proj * vec4<f32>(in.ndc, 1.0, 1.0);
  let dir = normalize(far.xyz / far.w - near.xyz / near.w);
  return vec4<f32>(sky_fog(textureSampleLevel(sky_map, ibl_sampler, dir, 0.0).rgb, dir), 1.0);
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
  // The disc is the sunlight through the air, the same transmittance as
  // the sky's (deep orange at sunset), replacing the glow under it a little
  // brighter than that glow, rolled off as the glow is (at a higher knee:
  // the brightest thing in the sky), and darkened
  // towards its limb (a sun's centre is brighter than its edge). Gone once
  // the sun has set.
  let through = physical_sun(d, h0);
  let peak = max(max(through.r, through.g), through.b);
  let hue = through / max(peak, 1e-6);
  let strength = min(dot(through, vec3<f32>(0.2126, 0.7152, 0.0722)) * frame.sky_sun.z, 1.5);
  let glow = max(max(sky.r, sky.g), sky.b) * 1.15;
  let cos_d = dot(d, sun);
  let disc = smoothstep(frame.sky_sun.x, frame.sky_sun.y, cos_d) * smoothstep(0.0, 1e-3, peak) * min(frame.sky_sun.z, 1.0);
  // How far from the centre to the limb, 0 to 1 (by the disc's outer cosine).
  let edge = clamp(sqrt(max(1.0 - cos_d, 0.0) / max(1.0 - frame.sky_sun.x, 1e-7)), 0.0, 1.0);
  let limb = 0.75 + 0.25 * sqrt(max(1.0 - edge * edge, 0.0));
  var color = mix(sky, sky_rolloff(light * hue * max(strength, glow), 0.9, 0.3) * limb, disc);
  let moon = moon_at(d, up);
  if (frame.sky_ground.w > 0.0 && up > 0.0) {
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

// ── clouds (clouds.rs) ──────────────────────────────────────────────
// frame.clouds: [0] coverage, density, base, thickness; [1] the wind's
// offset (xyz), the tile's size; [2] albedo, detail; [3] the high
// layer's cover and height, steps, on; [4] x the planet's radius, y and z
// the sun's and the sky's dimming by the cover (Clouds::light); [5] a
// lightning flash in the cloud: where (frame coordinates), how bright.

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
// origin's xz) of height `h` above the ground.
fn cloud_density(p: vec3<f32>, h: f32, cheap: bool) -> f32 {
  let c = frame.clouds;
  let hf = (h - c[0].z) / max(c[0].w, 1.0);
  if (hf <= 0.0 || hf >= 1.0) { return 0.0; }
  let at = vec3<f32>(p.x, h, p.z) + c[1].xyz;
  let uvw = at / c[1].w;
  // The lobes vary in size: the shape is read at two scales, blended by a
  // slow noise across the sky (small popcorn here, broad towers there).
  let small = textureSampleLevel(cloud_noise, cloud_sampler, uvw.xzy, 0.0).r;
  let broad = textureSampleLevel(cloud_noise, cloud_sampler, uvw.xzy * 0.45 + vec3<f32>(0.31, 0.17, 0.53), 0.0).r;
  let cover_px = textureSampleLevel(cloud_noise, cloud_sampler, uvw.xzy * 0.25, 0.0);
  let cover = cover_px.a;
  let shape = mix(small, broad, smoothstep(0.3, 0.7, textureSampleLevel(cloud_noise, cloud_sampler, uvw.xzy * 0.21 + vec3<f32>(0.6, 0.0, 0.2), 0.0).a));
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

// The clouds over `background` along unit `d` (before the fog): the high
// layer, then the volumetric layer, faded into the sky with distance.
fn clouds_over(background: vec3<f32>, d: vec3<f32>, pixel: vec2<f32>) -> vec3<f32> {
  let c = frame.clouds;
  if (c[3].w < 0.5) { return background; }
  let radius = c[4].x;
  let viewer = max(frame.view_pos.y + frame.origin.y, 0.0);
  let o = vec3<f32>(0.0, radius + viewer, 0.0);
  // A ray that meets the ground sees no clouds, unless the eye is above the
  // cloud base: then it looks down onto the layer (a sea of cloud from a
  // mountain), and the march below stops at the base, before the ground.
  let ground = cloud_sphere(o, d, radius);
  if (ground.x > 0.0 && viewer <= c[0].z) { return background; }
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
  let light = frame.light_color.rgb / max(c[4].y, 0.05);
  let cos_t = dot(d, to_light);
  let phase = mix(cloud_hg(cos_t, 0.6), cloud_hg(cos_t, -0.25), 0.35);
  // The sky's light from above and from the horizon (the IBL's irradiance).
  let sky_scale = frame.params.x / max(c[4].z, 0.05);
  let sky_top = textureSampleLevel(irradiance_map, ibl_sampler, vec3<f32>(0.0, 1.0, 0.0), 0.0).rgb * sky_scale;
  // The bases see the sky's low light, half of it from the horizon under
  // the sun (its glow: the orange of a sunset's cloud bases).
  let sun_side = textureSampleLevel(irradiance_map, ibl_sampler, normalize(vec3<f32>(to_light.x, 0.0, to_light.z) + vec3<f32>(0.0, 1e-3, 0.0)), 0.0).rgb;
  let view_side = textureSampleLevel(irradiance_map, ibl_sampler, normalize(vec3<f32>(d.x, 0.0, d.z) + vec3<f32>(0.0, 1e-3, 0.0)), 0.0).rgb;
  let sky_side = mix(view_side, sun_side, 0.5) * sky_scale;
  var color = background;

  // The high thin layer (cirrus), streaked along the wind.
  if (c[3].x > 0.0) {
    let t = cloud_sphere(o, d, radius + c[3].y).y;
    if (t > 0.0) {
      let p = o + d * t;
      let uv = (vec2<f32>(p.x, p.z) + c[1].xz * 2.0) / (c[1].w * 2.5);
      let n = textureSampleLevel(cloud_noise, cloud_sampler, vec3<f32>(uv.x * 0.35, uv.y, 0.37), 0.0);
      let wisp = cloud_remap(n.r * 0.7 + n.g * 0.3, 1.0 - c[3].x, 1.0 - c[3].x * 0.3) * (0.5 + 0.5 * n.b);
      let fade = exp(-t / 120000.0);
      let lit = cloud_sun(p, radius, to_light, light, physical) * (0.25 + phase * 1.5) * 0.2 + sky_top * 0.8;
      color = mix(color, lit * c[2].rgb, clamp(wisp, 0.0, 1.0) * 0.6 * fade);
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
  if (start >= far) { return color; }
  end = min(end, start + far * 0.5);
  // Up to twice the steps towards the horizon, where the path is longest.
  let steps = u32(c[3].z * (2.0 - abs(d.y)));
  let dt = (end - start) / f32(steps);
  // A different start per pixel turns banding into fine noise: interleaved
  // gradient noise (Jimenez 2014), whose neighbours differ evenly, so a
  // single frame shows an even fine grain rather than speckle (TAA
  // smooths it), offset each frame.
  let ign = fract(52.9829189 * fract(dot(pixel + f32(i32(frame.time.y) & 15) * 5.588238, vec2<f32>(0.06711056, 0.00583715))));
  var t = start + dt * ign;
  var transmittance = 1.0;
  var scattered = vec3<f32>(0.0);
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
      let s = (sun + ambient) * c[2].rgb * sigma;
      let seg = exp(-sigma * dt);
      scattered += transmittance * (s - s * seg) / sigma;
      transmittance *= seg;
      if (transmittance < 0.01) { break; }
    }
    t += dt;
  }
  // The air between the viewer and the clouds blends them into the sky.
  let fade = exp(-start / (far * 0.45));
  return mix(color, color * transmittance + scattered, fade);
}

// The procedural sky drawn per pixel with the chosen look (frame.sky_*):
// its sun is where it is this frame, not where the lighting was baked.
@fragment
fn fs_sky_look(in: SkyOut) -> @location(0) vec4<f32> {
  let dir = view_dir(in.ndc);
  var sky = sky_look(dir);
  if (scatter_on()) { sky = sky_physical_look(dir); }
  return vec4<f32>(sky_fog(clouds_over(sky, dir, in.pos.xy), dir), 1.0);
}
