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

// The look drawn by the physical sky (sky_scatter.wgsl): the scattered
// sunlight, the disc reddened by the air it crosses, the night's floor,
// then the stars and the moon as sky_look draws them.
fn sky_physical_look(d: vec3<f32>) -> vec3<f32> {
  let sun = normalize(frame.sky_sun_dir.xyz);
  let h0 = sky_viewer_km();
  let up = clamp(d.y, -1.0, 1.0);
  let light = frame.sky_glow.yzw;
  let sky = (physical_sky(d, sun, h0, 24u) * light + frame.sky_scatter[5].rgb) * frame.sky_horizon.w;
  let disc = smoothstep(frame.sky_sun.x, frame.sky_sun.y, dot(d, sun)) * frame.sky_sun.z;
  var color = sky + light * physical_sun(d, h0) * disc;
  let moon = moon_at(d, up);
  if (frame.sky_ground.w > 0.0 && up > 0.0) {
    let g = d * 220.0;
    let cell = floor(g);
    let h = lattice_hash(vec3<i32>(cell));
    let f = fract(g) - 0.5;
    let star = smoothstep(0.985, 1.0, h) * smoothstep(0.32, 0.0, length(f));
    let dark = 1.0 - clamp(dot(sky, vec3<f32>(0.2126, 0.7152, 0.0722)) * 4.0, 0.0, 1.0);
    color += vec3<f32>(star * frame.sky_ground.w * dark * 3.0) * smoothstep(0.0, 0.15, up) * (1.0 - moon.a);
  }
  return color + moon.rgb;
}

// ── clouds (clouds.rs) ──────────────────────────────────────────────
// frame.clouds: [0] coverage, density, base, thickness; [1] the wind's
// offset (xyz), the tile's size; [2] albedo, detail; [3] the high
// layer's cover and height, steps, on; [4] x the planet's radius.

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
  // Cumulus: a flat base rounding into domed tops.
  let gradient = smoothstep(0.0, 0.12, hf) * smoothstep(1.0, 0.45, hf);
  let at = vec3<f32>(p.x, h, p.z) + c[1].xyz;
  let uvw = at / c[1].w;
  let n = textureSampleLevel(cloud_noise, cloud_sampler, uvw.xzy, 0.0);
  let cover = textureSampleLevel(cloud_noise, cloud_sampler, uvw.xzy * 0.25, 0.0).a;
  let local = clamp(c[0].x + (cover - 0.5) * 0.7, 0.0, 1.0);
  var base = cloud_remap(n.r * gradient, 1.0 - local, 1.0);
  if (base <= 0.0) { return 0.0; }
  if (!cheap && c[2].w > 0.0) {
    let fine = textureSampleLevel(cloud_noise, cloud_sampler, uvw.xzy * 5.0, 0.0);
    let erode = mix(fine.g, fine.b, hf) * c[2].w;
    base = cloud_remap(base, erode * 0.6, 1.0);
  }
  return base * c[0].y;
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
  let ground = cloud_sphere(o, d, radius);
  if (ground.x > 0.0) { return background; }
  let to_light = normalize(frame.light_dir.xyz);
  let light = frame.light_color.rgb;
  let cos_t = dot(d, to_light);
  let phase = mix(cloud_hg(cos_t, 0.6), cloud_hg(cos_t, -0.25), 0.35);
  // The sky's light from above and from the horizon (the IBL's irradiance).
  let sky_top = textureSampleLevel(irradiance_map, ibl_sampler, vec3<f32>(0.0, 1.0, 0.0), 0.0).rgb * frame.params.x;
  let sky_side = textureSampleLevel(irradiance_map, ibl_sampler, normalize(vec3<f32>(d.x, 0.0, d.z) + vec3<f32>(0.0, 1e-3, 0.0)), 0.0).rgb * frame.params.x;
  var color = background;

  // The high thin layer (cirrus), streaked along the wind.
  if (c[3].x > 0.0) {
    let t = cloud_sphere(o, d, radius + c[3].y).y;
    if (t > 0.0) {
      let p = o + d * t;
      let uv = (vec2<f32>(p.x, p.z) + c[1].xz * 2.0) / (c[1].w * 2.5);
      let n = textureSampleLevel(cloud_noise, cloud_sampler, vec3<f32>(uv.x * 0.35, uv.y, 0.37), 0.0);
      let wisp = cloud_remap(n.r * 0.7 + n.g * 0.3, 1.0 - c[3].x * 0.8, 1.0) * (0.5 + 0.5 * n.b);
      let fade = exp(-t / 120000.0);
      let lit = light * (0.25 + phase * 1.5) * 0.35 + sky_top * 0.8;
      color = mix(color, lit * c[2].rgb, clamp(wisp, 0.0, 1.0) * 0.7 * fade);
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
  let steps = u32(c[3].z);
  let dt = (end - start) / f32(steps);
  // A different start per pixel turns banding into fine noise (TAA smooths it).
  var t = start + dt * lattice_hash(vec3<i32>(vec2<i32>(pixel), i32(frame.time.y) & 15));
  var transmittance = 1.0;
  var scattered = vec3<f32>(0.0);
  let light_step = c[0].w * 0.12;
  for (var i = 0u; i < steps; i++) {
    let p = o + d * t;
    let h = length(p) - radius;
    let sigma = cloud_density(p, h, false);
    if (sigma > 0.0) {
      // The column towards the light, in five widening steps.
      var od = 0.0;
      for (var k = 1u; k <= 5u; k++) {
        let q = p + to_light * light_step * f32(k * k) * 0.4;
        od += cloud_density(q, length(q) - radius, true) * light_step * f32(2u * k - 1u) * 0.4;
      }
      // Three octaves of multiple scattering (Wrenninge 2013).
      var sun = vec3<f32>(0.0);
      var a = 1.0;
      var b = 1.0;
      for (var m = 0u; m < 3u; m++) {
        sun += light * a * exp(-od * b) * mix(phase, 0.0795775, 1.0 - a);
        a *= 0.5;
        b *= 0.4;
      }
      let hf = clamp((h - c[0].z) / c[0].w, 0.0, 1.0);
      let ambient = mix(sky_side * 0.6, sky_top, hf) * (0.35 + 0.65 * hf);
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
