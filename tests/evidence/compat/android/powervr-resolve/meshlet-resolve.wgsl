






const CASCADES: u32 = 4u;
const LOCAL_SHADOW_LAYERS: u32 = 96u;



fn debug_unsupported(pixel: vec2<f32>) -> vec4<f32> {
  let cell = vec2<u32>(pixel) / 8u;
  let bright = ((cell.x + cell.y) & 1u) == 0u;
  return vec4<f32>(select(vec3<f32>(0.0), vec3<f32>(1.0, 0.0, 1.0), bright), 1.0);
}

struct LocalLight {
  position_range: vec4<f32>,
  color_kind: vec4<f32>,
  direction_slot: vec4<f32>,
  cone: vec4<f32>,
};

struct Frame {
  view_proj: mat4x4<f32>,
  inv_view_proj: mat4x4<f32>,
  view_pos: vec4<f32>,
  light_dir: vec4<f32>,
  light_color: vec4<f32>,
  params: vec4<f32>,
  view_forward: vec4<f32>,
  cascade_view_proj: array<mat4x4<f32>, CASCADES>,
  cascade_bias: array<vec4<f32>, CASCADES>,
  cluster: vec4<f32>,
  local_view_proj: array<mat4x4<f32>, LOCAL_SHADOW_LAYERS>,

  fog_color: vec4<f32>,
  fog_distance: vec4<f32>,
  fog_height: vec4<f32>,
  fog_sun: vec4<f32>,
  sky_zenith: vec4<f32>,
  sky_horizon: vec4<f32>,
  sky_ground: vec4<f32>,
  sky_sun: vec4<f32>,
  sky_glow: vec4<f32>,
  sky_sun_dir: vec4<f32>,
  sky_moon: vec4<f32>,
  sky_moon_look: vec4<f32>,
  time: vec4<f32>,
  sky_scatter: array<vec4<f32>, 38>,
  clouds: array<vec4<f32>, 10>,
  aurora: array<vec4<f32>, 5>,
  weather: vec4<f32>,
  weather_look: vec4<f32>,
  weather_view_proj: mat4x4<f32>,
  weather_shelter: vec4<f32>,

  caustics: vec4<f32>,
  caustics_area: vec4<f32>,
  caustics_cells: array<vec4<f32>, 2>,
  shadow: vec4<f32>,

  stylise: vec4<f32>,
  shadow_tint: vec4<f32>,
  outline_style: vec4<f32>,



  probe_min: vec4<f32>,
  probe_max: vec4<f32>,
  probe_pos: vec4<f32>,


  view_right: vec4<f32>,
  view_up: vec4<f32>,
  hemi_sky: vec4<f32>,
  hemi_ground: vec4<f32>,
  material_surface: vec4<f32>,
  origin: vec4<f32>,

  env_dome: array<vec4<f32>, 2>,
  foliage_wind: array<array<vec4f, 9>, 2>,




  toon: vec4<f32>,
};




fn hemi_irradiance(n: vec3<f32>) -> vec3<f32> {
  return mix(frame.hemi_ground.rgb, frame.hemi_sky.rgb, 0.5 * n.y + 0.5) * frame.hemi_sky.w;
}

@group(0) @binding(0) var<uniform> frame: Frame;


@group(0) @binding(1) var cascade_map: texture_2d_array<f32>;
@group(0) @binding(2) var local_shadow_map: texture_2d_array<f32>;
@group(0) @binding(3) var irradiance_map: texture_cube<f32>;
@group(0) @binding(4) var prefiltered_map: texture_cube<f32>;
@group(0) @binding(5) var brdf_lut: texture_2d<f32>;
@group(0) @binding(6) var ibl_sampler: sampler;



fn probe_weight(p: vec3<f32>) -> f32 {
  let d = min(p - frame.probe_min.xyz, frame.probe_max.xyz - p);
  return clamp(min(d.x, min(d.y, d.z)) / max(frame.probe_pos.w, 1e-3), 0.0, 1.0);
}



fn probe_direction(r: vec3<f32>, p: vec3<f32>) -> vec3<f32> {
  let safe = select(r, vec3<f32>(1e-6), abs(r) < vec3<f32>(1e-6));
  let f = max((frame.probe_max.xyz - p) / safe, (frame.probe_min.xyz - p) / safe);
  let t = max(min(f.x, min(f.y, f.z)), 0.0);
  return p + r * t - frame.probe_pos.xyz;
}





fn ambient_light(n: vec3<f32>, p: vec3<f32>) -> vec3<f32> {
  if (frame.probe_min.w < 0.5) { return gi_ambient(textureSample(irradiance_map, ibl_sampler, n).rgb, n, p); }
  let top = frame.params.y;
  let env = textureSampleLevel(irradiance_map, ibl_sampler, n, top).rgb;
  let probe = textureSampleLevel(prefiltered_map, ibl_sampler, n, top).rgb * frame.probe_max.w;
  return gi_ambient(mix(env, probe, probe_weight(p)), n, p);
}



fn ambient_light_level(n: vec3<f32>, p: vec3<f32>) -> vec3<f32> {
  if (frame.probe_min.w < 0.5) { return textureSampleLevel(irradiance_map, ibl_sampler, n, 0.0).rgb; }
  let top = frame.params.y;
  let env = textureSampleLevel(irradiance_map, ibl_sampler, n, top).rgb;
  let probe = textureSampleLevel(prefiltered_map, ibl_sampler, n, top).rgb * frame.probe_max.w;
  return mix(env, probe, probe_weight(p));
}









fn dome_lookup(eye: vec3<f32>, dir: vec3<f32>) -> vec3<f32> {
  let h = frame.env_dome[0].w;
  if (h <= 0.0) { return dir; }
  let o = eye - frame.env_dome[0].xyz;
  let r = frame.env_dome[1].x;
  let b = dot(o, dir);
  let disc = b * b - (dot(o, o) - r * r);
  if (disc < 0.0) { return dir; }

  var t = -b + sqrt(disc);
  if (t <= 0.0) { return dir; }
  if (dir.y < 0.0) {
    let tg = (-h - o.y) / dir.y;
    if (tg > 0.0) { t = min(t, tg); }
  }
  return normalize(o + dir * t);
}

fn reflected_light(r_in: vec3<f32>, p: vec3<f32>, mip: f32) -> vec3<f32> {
  let r = dome_lookup(p, r_in);
  if (frame.probe_min.w < 0.5) { return textureSampleLevel(prefiltered_map, ibl_sampler, r, mip).rgb; }
  let env = textureSampleLevel(irradiance_map, ibl_sampler, r, mip).rgb;
  let probe = textureSampleLevel(prefiltered_map, ibl_sampler, probe_direction(r_in, p), mip).rgb * frame.probe_max.w;
  return mix(env, probe, probe_weight(p));
}





fn reflectivity_bits(roughness: f32, metalness: f32) -> f32 {
  return round((1.0 - clamp(roughness, 0.0, 1.0)) * 255.0) + 256.0 * round(clamp(metalness, 0.0, 1.0) * 255.0);
}





fn fog_amount(p: vec3<f32>) -> vec4<f32> {
  return fog_amount_scaled(p, 1.0);
}



fn fog_amount_scaled(p: vec3<f32>, height: f32) -> vec4<f32> {
  let ortho = frame.view_pos.w > 0.5;

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
  let sun = pow(max(dot(dir, normalize(frame.light_dir.xyz)), 0.0), max(frame.fog_sun.w, 1e-3));
  return vec4<f32>(mix(frame.fog_color.rgb, frame.fog_sun.rgb, sun), amount);
}

fn apply_fog(color: vec3<f32>, p: vec3<f32>) -> vec3<f32> {
  if (frame.fog_color.w < 0.5) { return color; }
  let f = fog_amount(p);
  return mix(color, f.rgb, f.a);
}



fn fog_keep(p: vec3<f32>) -> f32 {
  if (frame.fog_color.w < 0.5) { return 1.0; }
  return 1.0 - fog_amount(p).a;
}


fn moon_mare(q: vec2<f32>, c: vec2<f32>, r: f32) -> f32 {
  return 1.0 - smoothstep(0.2 * r, r, length(q - c));
}





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

  let q = vec2<f32>(dot(d, right), dot(d, top)) / sin(radius);
  let r = length(q);
  let above = smoothstep(-0.004, 0.004, up);
  let coverage = (1.0 - smoothstep(0.92, 1.0, r)) * above;
  let theta = frame.sky_moon_look.w * 6.2831853;
  let lit_fraction = 0.5 - 0.5 * cos(theta);


  let halo = exp(-max(r - 1.0, 0.0) * 1.5) * 0.012 * lit_fraction * above * smoothstep(-0.02, 0.02, m.y);
  if (coverage <= 0.0) { return vec4<f32>(frame.sky_moon_look.rgb * halo, 0.0); }

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







fn lattice_hash(c: vec3<i32>) -> f32 {
  var h = (bitcast<u32>(c.x) * 73856093u) ^ (bitcast<u32>(c.y) * 19349663u) ^ (bitcast<u32>(c.z) * 83492791u);
  h = h * 747796405u + 2891336453u;
  h = ((h >> ((h >> 28u) + 4u)) ^ h) * 277803737u;
  h = (h >> 22u) ^ h;
  return f32(h >> 8u) / 16777216.0;
}






fn clear_sky_light(n: vec3<f32>) -> vec3<f32> {
  let c = frame.clouds;
  let s = normalize(vec3<f32>(frame.sky_sun_dir.x, 0.0, frame.sky_sun_dir.z) + vec3<f32>(0.0, 0.0, 1e-5));
  let h = normalize(vec3<f32>(n.x, 0.0, n.z) + vec3<f32>(1e-5, 0.0, 0.0));
  let w = dot(h, s);
  let side = mix(c[9].rgb, select(c[8].rgb, c[7].rgb, w > 0.0), abs(w));
  return mix(side, c[6].rgb, clamp(n.y, 0.0, 1.0));
}





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



fn star_veil() -> f32 {
  let c = frame.clouds;
  if (c[3].w < 0.5 || c[0].w <= 0.0 || c[0].y <= 0.0) { return 1.0; }
  return 1.0 - smoothstep(0.5, 0.9, c[0].x);
}




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


























const GI_HEADER_AT: u32 = 16u;
const GI_CASCADES: u32 = 4u;
const GI_DEPTH: u32 = 12u;
const GI_STRIDE: u32 = 76u;
const GI_SH_AT: u32 = 1u;
const GI_DEPTH_AT: u32 = 4u;

const GI_RADIANCE: u32 = 6u;
const GI_PI: f32 = 3.14159265359;



fn gi_oct_encode(n: vec3<f32>) -> vec2<f32> {
  let p = n.xy / (abs(n.x) + abs(n.y) + abs(n.z));
  if (n.z >= 0.0) { return p; }
  return (1.0 - abs(p.yx)) * select(vec2<f32>(-1.0), vec2<f32>(1.0), p >= vec2<f32>(0.0));
}

fn gi_oct_decode(e: vec2<f32>) -> vec3<f32> {
  var v = vec3<f32>(e, 1.0 - abs(e.x) - abs(e.y));
  if (v.z < 0.0) {
    let s = select(vec2<f32>(-1.0), vec2<f32>(1.0), v.xy >= vec2<f32>(0.0));
    v = vec3<f32>((1.0 - abs(v.yx)) * s, v.z);
  }
  return normalize(v);
}






fn gi_oct_wrap(p: vec2<i32>, n: i32) -> vec2<i32> {
  let c = clamp(p, vec2<i32>(0), vec2<i32>(n - 1));
  let out_x = p.x < 0 || p.x >= n;
  let out_y = p.y < 0 || p.y >= n;
  if (out_x && out_y) { return vec2<i32>(n - 1) - c; }
  if (out_x) { return vec2<i32>(c.x, n - 1 - c.y); }
  if (out_y) { return vec2<i32>(n - 1 - c.x, c.y); }
  return p;
}





fn gi_slot(w: vec3<i32>, counts: vec3<i32>) -> vec3<i32> {
  return ((w % counts) + counts) % counts;
}

fn gi_key(w: vec3<i32>, counts: vec3<i32>) -> f32 {
  let g = ((w - gi_slot(w, counts)) / counts) & vec3<i32>(255);
  return f32(g.x + g.y * 256 + g.z * 65536 + 1);
}

fn gi_flat(s: vec3<i32>, counts: vec3<i32>) -> u32 {
  return u32((s.z * counts.y + s.y) * counts.x + s.x);
}





const GI_FIXED_RAYS: u32 = 16u;

fn gi_fibonacci(i: f32, n: f32) -> vec3<f32> {
  let golden = 0.61803398875;
  let phi = 2.0 * GI_PI * fract(i * golden);
  let cos_theta = 1.0 - (2.0 * i + 1.0) / n;
  let sin_theta = sqrt(clamp(1.0 - cos_theta * cos_theta, 0.0, 1.0));
  return vec3<f32>(cos(phi) * sin_theta, sin(phi) * sin_theta, cos_theta);
}

fn gi_ray_dir(i: u32, n: u32, rotation: mat3x3<f32>) -> vec3<f32> {
  if (i < GI_FIXED_RAYS) { return gi_fibonacci(f32(i), f32(GI_FIXED_RAYS)); }
  return normalize(rotation * gi_fibonacci(f32(i - GI_FIXED_RAYS), f32(n - GI_FIXED_RAYS)));
}



struct ProbeWords { v: array<vec4<f32>> };
@group(0) @binding(10) var<storage, read> probe_data: ProbeWords;


fn gi_moments(at: u32, t: vec2<i32>) -> vec2<f32> {
  let i = u32(t.y) * GI_DEPTH + u32(t.x);
  let w = probe_data.v[at + GI_DEPTH_AT + (i >> 1u)];
  return select(w.xy, w.zw, (i & 1u) == 1u);
}



fn gi_moments_along(at: u32, dir: vec3<f32>) -> vec2<f32> {
  let n = i32(GI_DEPTH);
  let p = (gi_oct_encode(dir) * 0.5 + 0.5) * f32(GI_DEPTH) - 0.5;
  let base = vec2<i32>(floor(p));
  let f = p - floor(p);
  let a = gi_moments(at, gi_oct_wrap(base, n));
  let b = gi_moments(at, gi_oct_wrap(base + vec2<i32>(1, 0), n));
  let c = gi_moments(at, gi_oct_wrap(base + vec2<i32>(0, 1), n));
  let d = gi_moments(at, gi_oct_wrap(base + vec2<i32>(1, 1), n));
  return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}



fn gi_probe_irradiance(at: u32, n: vec3<f32>) -> vec3<f32> {
  let r = probe_data.v[at + GI_SH_AT];
  let g = probe_data.v[at + GI_SH_AT + 1u];
  let b = probe_data.v[at + GI_SH_AT + 2u];
  return max(vec3<f32>(r.w + dot(r.xyz, n), g.w + dot(g.xyz, n), b.w + dot(b.xyz, n)), vec3<f32>(0.0));
}







fn gi_cascade(c: u32, p: vec3<f32>, n: vec3<f32>, v: vec3<f32>, visibility: bool) -> vec4<f32> {
  let head = probe_data.v[GI_HEADER_AT];
  let r0 = probe_data.v[GI_HEADER_AT + 1u + 2u * c];
  let r1 = probe_data.v[GI_HEADER_AT + 2u + 2u * c];
  let spacing = r0.w;
  let counts = vec3<i32>(r1.xyz);
  let first = u32(r1.w);



  let biased = p + (n * head.z + v * head.w) * spacing;
  let g = biased / spacing - r0.xyz;
  let cmax = vec3<f32>(counts - vec3<i32>(1));
  let inside = min(min(g.x, min(g.y, g.z)), min(cmax.x - g.x, min(cmax.y - g.y, cmax.z - g.z)));
  let edge = clamp(inside * 0.5, 0.0, 1.0);
  if (edge <= 0.0) { return vec4<f32>(0.0); }
  let base = vec3<i32>(floor(r0.xyz + floor(g)));
  let alpha = fract(g);
  var sum = vec3<f32>(0.0);
  var total = 0.0;
  for (var i = 0u; i < 8u; i++) {
    let o = vec3<i32>(vec3<u32>(i, i >> 1u, i >> 2u) & vec3<u32>(1u));
    let w = base + o;
    let at = first + gi_flat(gi_slot(w, counts), counts) * GI_STRIDE;
    let state = probe_data.v[at];


    if (state.w != gi_key(w, counts)) { continue; }
    let probe = (vec3<f32>(w) + state.xyz) * spacing;
    let trilinear = mix(1.0 - alpha, alpha, vec3<f32>(o));
    var weight = 1.0;


    let to_probe = normalize(probe - p);
    let wrap = (dot(to_probe, n) + 1.0) * 0.5;
    weight *= wrap * wrap + 0.2;
    if (visibility) {


      let from_probe = biased - probe;
      let dist = length(from_probe);
      let m = gi_moments_along(at, from_probe / max(dist, 1e-5));
      let variance = abs(m.y - m.x * m.x);
      let over = max(dist - m.x, 0.0);
      var chebyshev = variance / max(variance + over * over, 1e-8);
      chebyshev = max(chebyshev * chebyshev * chebyshev, 0.0);
      weight *= select(chebyshev, 1.0, dist <= m.x);
    }
    weight = max(weight, 0.01);


    let crush = 0.2;
    if (weight < crush) { weight *= weight * weight / (crush * crush); }
    weight *= trilinear.x * trilinear.y * trilinear.z;
    sum += gi_probe_irradiance(at, n) * weight;
    total += weight;
  }
  if (total <= 1e-6) { return vec4<f32>(0.0); }
  return vec4<f32>(sum / total, edge);
}




fn gi_probe_light(p: vec3<f32>, n: vec3<f32>, v: vec3<f32>, visibility: bool) -> vec4<f32> {
  let cascades = min(u32(probe_data.v[GI_HEADER_AT].x), GI_CASCADES);
  var out = vec4<f32>(0.0);
  for (var c = 0u; c < cascades; c++) {
    let s = gi_cascade(c, p, n, v, visibility);
    if (s.w <= 0.0) { continue; }

    let take = s.w * (1.0 - out.w);
    out = vec4<f32>(out.rgb + s.rgb * take, out.w + take);
    if (out.w >= 0.999) { break; }
  }
  if (out.w <= 0.0) { return vec4<f32>(0.0); }
  return vec4<f32>(out.rgb / out.w, out.w);
}





fn gi_ambient(env: vec3<f32>, n: vec3<f32>, p: vec3<f32>) -> vec3<f32> {
  if (probe_data.v[GI_HEADER_AT].x < 0.5) { return env; }
  let v = select(normalize(frame.view_pos.xyz - p), -frame.view_forward.xyz, frame.view_pos.w > 0.5);
  let g = gi_probe_light(p, n, v, true);
  return mix(env, g.rgb / max(frame.params.x, 1e-4), g.w);
}






fn gi_reflection_occlusion(r: vec3<f32>, p: vec3<f32>, env: vec3<f32>) -> f32 {
  if (probe_data.v[GI_HEADER_AT].x < 0.5) { return 1.0; }
  let g = gi_probe_light(p, r, r, false);
  if (g.w <= 0.0) { return 1.0; }
  let luma = vec3<f32>(0.2126, 0.7152, 0.0722);
  let seen = dot(g.rgb, luma) / max(frame.params.x, 1e-4);
  let sky = max(dot(env, luma), 1e-4);
  return mix(1.0, clamp(seen / sky, 0.0, 1.0), g.w);
}





fn gi_reflected(prefiltered: vec3<f32>, r: vec3<f32>, p: vec3<f32>) -> vec3<f32> {
  if (probe_data.v[GI_HEADER_AT].x < 0.5) { return prefiltered; }
  let captured = select(0.0, probe_weight(p), frame.probe_min.w > 0.5);
  let k = gi_reflection_occlusion(r, p, ambient_light_level(r, p));
  return prefiltered * mix(k, 1.0, captured);
}
















var<private> oit_pass: bool;






struct OitOutput {
  @location(0) accum: vec4<f32>,
  @location(1) reveal: vec2<f32>,
  @location(2) front: vec4<f32>,
};





fn oit_weight(alpha: f32, depth: f32) -> f32 {
  let a = min(1.0, alpha * 10.0) + 0.01;
  let z = 1.0 - clamp(depth, 0.0, 1.0) * 0.9;
  return clamp(a * a * a * 1e8 * z * z * z, 1e-2, 3e3) / 3e3;
}




fn oit_weighted(color: vec4<f32>, depth: f32) -> OitOutput {
  let alpha = clamp(color.a, 0.0, 1.0);
  let premultiplied = clamp(color.rgb * alpha, vec3<f32>(0.0), vec3<f32>(64.0));
  return OitOutput(
    vec4<f32>(premultiplied, alpha) * oit_weight(alpha, depth),
    vec2<f32>(alpha, 0.0),
    vec4<f32>(0.0));
}



fn oit_front(color: vec4<f32>) -> OitOutput {
  let alpha = clamp(color.a, 0.0, 1.0);
  let premultiplied = clamp(color.rgb * alpha, vec3<f32>(0.0), vec3<f32>(64.0));
  return OitOutput(vec4<f32>(0.0), vec2<f32>(0.0, alpha), vec4<f32>(premultiplied, alpha));
}


fn oit_hash(h: u32, word: u32) -> u32 {
  return (h ^ word) * 16777619u;
}






fn oit_key(model: mat4x4<f32>, local: vec3<f32>) -> vec2<u32> {
  var draw = 2166136261u;
  for (var c = 0u; c < 4u; c++) {
    for (var r = 0u; r < 3u; r++) {
      draw = oit_hash(draw, bitcast<u32>(model[c][r]));
    }
  }
  var primitive = 2166136261u;
  for (var r = 0u; r < 3u; r++) {
    primitive = oit_hash(primitive, bitcast<u32>(local[r]));
  }
  return vec2<u32>(draw, primitive);
}



fn oit_key4(draw: vec4<f32>, local: vec3<f32>) -> vec2<u32> {
  var d = 2166136261u;
  for (var c = 0u; c < 4u; c++) {
    d = oit_hash(d, bitcast<u32>(draw[c]));
  }
  var primitive = 2166136261u;
  for (var r = 0u; r < 3u; r++) {
    primitive = oit_hash(primitive, bitcast<u32>(local[r]));
  }
  return vec2<u32>(d, primitive);
}
















@group(0) @binding(12) var<storage, read_write> oit_arena: array<atomic<u32>>;

const OIT_HEADER_WORDS: u32 = 4u;
const OIT_NODE_WORDS: u32 = 7u;

fn oit_store(position: vec4<f32>, color: vec4<f32>, mask: u32, key: vec2<u32>) {
  let alpha = clamp(color.a, 0.0, 1.0);

  if (!(alpha > 0.0) || mask == 0u) { return; }
  let width = atomicLoad(&oit_arena[0]);
  let height = atomicLoad(&oit_arena[1]);
  let budget = atomicLoad(&oit_arena[2]);
  let px = vec2<u32>(position.xy);
  if (px.x >= width || px.y >= height) { return; }
  let p = px.y * width + px.x;
  let slot = atomicAdd(&oit_arena[OIT_HEADER_WORDS + p], 1u);
  if (slot >= budget) {
    if (slot == budget) { atomicAdd(&oit_arena[3], 1u); }
    return;
  }
  let node = OIT_HEADER_WORDS + width * height + (p * budget + slot) * OIT_NODE_WORDS;
  let rgb = clamp(color.rgb, vec3<f32>(0.0), vec3<f32>(65504.0));
  atomicStore(&oit_arena[node], bitcast<u32>(position.z));
  atomicStore(&oit_arena[node + 1u], bitcast<u32>(rgb.r));
  atomicStore(&oit_arena[node + 2u], bitcast<u32>(rgb.g));
  atomicStore(&oit_arena[node + 3u], bitcast<u32>(rgb.b));
  atomicStore(&oit_arena[node + 4u], bitcast<u32>(alpha));
  atomicStore(&oit_arena[node + 5u], key.x);
  atomicStore(&oit_arena[node + 6u], (key.y & 0x0fffffffu) | ((mask & 15u) << 28u));
}


















const OIT_FRONT_ALPHA: f32 = 0.125;




const OIT_FRONT_ULPS: u32 = 4u;

fn oit_front_word(position: vec4<f32>) -> u32 {
  let width = atomicLoad(&oit_arena[0]);
  let height = atomicLoad(&oit_arena[1]);
  let px = vec2<u32>(position.xy);
  if (px.x >= width || px.y >= height) { return 0xffffffffu; }
  return OIT_HEADER_WORDS + px.y * width + px.x;
}

fn oit_front_mark(position: vec4<f32>, alpha: f32) {
  if (!(clamp(alpha, 0.0, 1.0) >= OIT_FRONT_ALPHA)) { return; }
  let w = oit_front_word(position);
  if (w == 0xffffffffu) { return; }
  atomicMax(&oit_arena[w], ~bitcast<u32>(max(position.z, 0.0)));
}

fn oit_front_of(position: vec4<f32>, alpha: f32) -> bool {
  if (!(clamp(alpha, 0.0, 1.0) >= OIT_FRONT_ALPHA)) { return false; }
  let w = oit_front_word(position);
  if (w == 0xffffffffu) { return false; }
  let marked = atomicLoad(&oit_arena[w]);
  if (marked == 0u) { return false; }
  return bitcast<u32>(max(position.z, 0.0)) <= ~marked + OIT_FRONT_ULPS;
}


fn oit_hybrid(color: vec4<f32>, position: vec4<f32>) -> OitOutput {
  if (oit_front_of(position, color.a)) {
    return oit_front(color);
  }
  return oit_weighted(color, position.z);
}
// The local lights and their clusters (clusters.rs).
const CLUSTER_X: u32 = 16u;
const CLUSTER_Y: u32 = 9u;
const CLUSTER_Z: u32 = 24u;
@group(0) @binding(7) var<storage, read> light_list: array<LocalLight, 256>;
@group(0) @binding(8) var<storage, read> cluster_table: array<u32, 3456>;
@group(0) @binding(9) var<storage, read> cluster_refs: array<u32, 55296>;

fn light_at(i: u32) -> LocalLight {
  return light_list[i];
}

fn cluster_word(c: u32) -> u32 {
  return cluster_table[c];
}

fn cluster_ref(j: u32) -> u32 {
  return (cluster_refs[j >> 2u] >> ((j & 3u) * 8u)) & 255u;
}

// The lights a fragment at pixel `frag` and world position `p` shades: x
// its cluster's first reference and y their count, or, in a frame without
// clusters (a reflection probe's capture, probes.rs), x NO_CLUSTER and y
// every light of the frame. Read the k-th with `frame_light`.
const NO_CLUSTER: u32 = 0xffffffffu;
fn frame_lights(frag: vec2<f32>, p: vec3<f32>) -> vec2<u32> {
  if (frame.params.z < 0.5) { return vec2<u32>(0u); }
  if (frame.cluster.x > 0.0) {
    let word = cluster_word(cluster_at(frag, p));
    return vec2<u32>(word & 0xffffffu, word >> 24u);
  }
  return vec2<u32>(NO_CLUSTER, u32(frame.params.z));
}

fn frame_light(lights: vec2<u32>, k: u32) -> LocalLight {
  var i = k;
  if (lights.x != NO_CLUSTER) { i = cluster_ref(lights.x + k); }
  return light_at(i);
}

// The cluster of a fragment at pixel `frag` and world position `p`.
fn cluster_at(frag: vec2<f32>, p: vec3<f32>) -> u32 {
  let x = min(u32(max(frag.x * frame.cluster.x, 0.0)), CLUSTER_X - 1u);
  let y = min(u32(max(frag.y * frame.cluster.y, 0.0)), CLUSTER_Y - 1u);
  let depth = dot(p - frame.view_pos.xyz, frame.view_forward.xyz);
  var s = depth * frame.cluster.z + frame.cluster.w;
  if (frame.view_pos.w < 0.5) { s = log2(max(depth, 1e-6)) * frame.cluster.z + frame.cluster.w; }
  let z = u32(clamp(s, 0.0, f32(CLUSTER_Z - 1u)));
  return (z * CLUSTER_Y + y) * CLUSTER_X + x;
}




fn pcg(v: u32) -> u32 {
  let state = v * 747796405u + 2891336453u;
  let word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
  return (word >> 22u) ^ word;
}


fn hash_unit(v: u32) -> f32 {
  return f32(pcg(v) >> 8u) / 16777216.0;
}

fn lattice3(c: vec3<i32>, seed: u32) -> f32 {
  let h = (bitcast<u32>(c.x) * 73856093u) ^ (bitcast<u32>(c.y) * 19349663u) ^ (bitcast<u32>(c.z) * 83492791u) ^ seed;
  return hash_unit(h);
}


fn value_noise3(p: vec3<f32>, seed: u32) -> f32 {
  return value_noise3_at(p, vec3<i32>(0), seed);
}




fn value_noise3_at(p: vec3<f32>, whole: vec3<i32>, seed: u32) -> f32 {
  let fl = floor(p);
  let i = vec3<i32>(fl) + whole;
  let f = p - fl;
  let u = f * f * (3.0 - 2.0 * f);
  let c000 = lattice3(i, seed);
  let c100 = lattice3(i + vec3<i32>(1, 0, 0), seed);
  let c010 = lattice3(i + vec3<i32>(0, 1, 0), seed);
  let c110 = lattice3(i + vec3<i32>(1, 1, 0), seed);
  let c001 = lattice3(i + vec3<i32>(0, 0, 1), seed);
  let c101 = lattice3(i + vec3<i32>(1, 0, 1), seed);
  let c011 = lattice3(i + vec3<i32>(0, 1, 1), seed);
  let c111 = lattice3(i + vec3<i32>(1, 1, 1), seed);
  let x00 = c000 + (c100 - c000) * u.x;
  let x10 = c010 + (c110 - c010) * u.x;
  let x01 = c001 + (c101 - c001) * u.x;
  let x11 = c011 + (c111 - c011) * u.x;
  let y0 = x00 + (x10 - x00) * u.y;
  let y1 = x01 + (x11 - x01) * u.y;
  return y0 + (y1 - y0) * u.z;
}




struct MaterialFactors {
  base_color: vec4<f32>,
  mr: vec4<f32>,
  emissive: vec4<f32>,
  flags: vec4<f32>,
  toon: vec4<f32>,
  rim: vec4<f32>,
  flash: vec4<f32>,
  dissolve: vec4<f32>,
  outline: vec4<f32>,
  coat: vec4<f32>,
  uv: array<vec4<f32>, 10>,
  custom: array<vec4<f32>, 8>,
  previous_custom: array<vec4<f32>, 8>,
};

@group(1) @binding(0) var<uniform> material: MaterialFactors;



const UV_MAP: u32 = 0u;
const UV_METALNESS: u32 = 1u;
const UV_NORMAL: u32 = 2u;
const UV_EMISSIVE: u32 = 3u;
const UV_AO: u32 = 4u;


fn material_uv(slot: u32, uv: vec2<f32>) -> vec2<f32> {
  let m = material.uv[slot * 2u];
  let t = material.uv[slot * 2u + 1u];
  return vec2<f32>(m.x * uv.x + m.y * uv.y, m.z * uv.x + m.w * uv.y) + t.xy;
}



@group(1) @binding(1) var base_color_tex: texture_2d<f32>;
@group(1) @binding(2) var metalness_tex: texture_2d<f32>;
@group(1) @binding(3) var roughness_tex: texture_2d<f32>;
@group(1) @binding(4) var normal_tex: texture_2d<f32>;
@group(1) @binding(5) var emissive_tex: texture_2d<f32>;
@group(1) @binding(6) var ao_tex: texture_2d<f32>;
@group(1) @binding(7) var material_sampler: sampler;
@group(1) @binding(8) var gradient_tex: texture_2d<f32>;
@group(1) @binding(9) var dissolve_tex: texture_2d<f32>;



fn dissolve_noise(uv: vec2<f32>, local_pos: vec3<f32>) -> f32 {
  if (material.dissolve.w < 0.0) { return textureSampleLevel(dissolve_tex, material_sampler, uv, 0.0).r; }
  let p = local_pos * material.dissolve.w;
  return 0.6 * value_noise3(p, 17u) + 0.4 * value_noise3(p * 2.7 + vec3<f32>(5.2, 1.3, 7.7), 91u);
}








fn dissolve_at(uv: vec2<f32>, local_pos: vec3<f32>) -> vec4<f32> {
  let amount = material.rim.w;
  if (amount <= 0.0) { return vec4<f32>(0.0); }
  let noise = dissolve_noise(uv, local_pos);
  if (noise < amount) { return vec4<f32>(0.0, 0.0, 0.0, 1.0); }
  let width = max(material.flash.w, 1e-4);
  let edge = 1.0 - clamp((noise - amount) / width, 0.0, 1.0);
  return vec4<f32>(material.dissolve.rgb * edge * edge, 0.0);
}



@group(3) @binding(0) var<storage, read> joint_palette: array<mat4x4<f32>>;
@group(3) @binding(1) var<storage, read> morph_deltas: array<vec4<f32>>;
@group(3) @binding(2) var<storage, read> baked_palette: array<mat4x4<f32>>;

fn joint_matrix(i: u32) -> mat4x4<f32> {
  return joint_palette[i];
}





fn palette_joint(palette: vec4<f32>, j: u32) -> mat4x4<f32> {
  let base = u32(palette.x);
  let count = u32(max(palette.y, 1.0));
  let k = min(j, count - 1u);
  if (palette.w < -0.5) {
    let t = -palette.w - 1.0;
    return baked_palette[base + k] * (1.0 - t) + baked_palette[base + count + k] * t;
  }
  return joint_palette[base + k];
}

fn morph_delta(i: u32) -> vec4<f32> {
  return morph_deltas[i];
}



fn morph_weight(slot: u32, t: u32) -> f32 {
  let m = joint_palette[slot + t / 16u];
  return m[(t % 16u) / 4u][t % 4u];
}

struct Morphed {
  position: vec3<f32>,
  normal: vec3<f32>,
};









fn morphed(position: vec3<f32>, normal: vec3<f32>, vi: u32, palette: vec4<f32>, weights: f32, deformed: f32) -> Morphed {
  var m: Morphed;
  if (deformed > 0.5) {
    let d = joint_palette[u32(deformed) - 1u + vi / 2u];
    let c = (vi % 2u) * 2u;
    m.position = d[c].xyz;
    m.normal = d[c + 1u].xyz;
    return m;
  }
  m.position = position;
  m.normal = normal;
  if (palette.w < 0.5 || weights < 0.5) { return m; }
  let slot = u32(weights) - 1u;
  let h = morph_deltas[u32(palette.z) + vi];
  let first = u32(h.x);
  let count = u32(h.y);
  for (var k = 0u; k < count; k++) {
    let d = morph_deltas[first + k * 2u];
    let w = morph_weight(slot, u32(d.w));
    if (w != 0.0) {
      m.position += d.xyz * w;
      m.normal += morph_deltas[first + k * 2u + 1u].xyz * w;
    }
  }
  return m;
}
// Material shader hooks (hooks.rs).
struct Hook {
  world: vec3<f32>,
  local: vec3<f32>,
  normal: vec3<f32>,
  uv: vec2<f32>,
  color: vec4<f32>,
  time: f32,
  eye: vec3<f32>,
  front: f32,
  origin: vec3<f32>,
};
// The fragment's facing (+1 front, -1 back), set by the fragment entry points
// that know it; 1 elsewhere (vertex stages, shadow passes).
var<private> hook_front: f32 = 1.0;
struct Params {
  unused: f32,
};
fn shader_params() -> Params {
  var p: Params;
  return p;
}
fn previous_shader_params() -> Params {
  var p: Params;
  return p;
}

fn vertex_offset(h: Hook, p: Params) -> vec3<f32> { return vec3<f32>(0.0); }
fn surface_color(h: Hook, p: Params, c: vec4<f32>) -> vec4<f32> { return c; }
fn surface_normal(h: Hook, p: Params, n: vec3<f32>) -> vec3<f32> { return n; }
fn surface_emissive(h: Hook, p: Params, e: vec3<f32>) -> vec3<f32> { return e; }
fn moved_clip(m: mat4x4<f32>, p: vec4<f32>) -> vec4<f32> { return mul_point(m, p); }





const PI: f32 = 3.14159265359;

fn d_ggx(n_dot_h: f32, a: f32) -> f32 {
  let a2 = a * a;
  let d = n_dot_h * n_dot_h * (a2 - 1.0) + 1.0;
  return a2 / (PI * d * d + 1e-7);
}

fn g_smith(n_dot_v: f32, n_dot_l: f32, roughness: f32) -> f32 {
  let r = roughness + 1.0;
  let k = (r * r) / 8.0;
  let gv = n_dot_v / (n_dot_v * (1.0 - k) + k);
  let gl = n_dot_l / (n_dot_l * (1.0 - k) + k);
  return gv * gl;
}

fn f_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
  return f0 + (1.0 - f0) * pow(saturate(1.0 - cos_theta), 5.0);
}

fn f_schlick_roughness(cos_theta: f32, f0: vec3<f32>, roughness: f32) -> vec3<f32> {
  return f0 + (max(vec3<f32>(1.0 - roughness), f0) - f0) * pow(saturate(1.0 - cos_theta), 5.0);
}










fn pcf(map: texture_2d_array<f32>, uv: vec2<f32>, layer: u32, ref_depth: f32, slope: vec2<f32>) -> f32 {
  let size = vec2<i32>(textureDimensions(map));
  let p = uv * vec2<f32>(size) - 0.5;
  let base = vec2<i32>(floor(p));
  let f = fract(p);
  var lit: array<f32, 4>;
  for (var i = 0; i < 4; i++) {
    let o = vec2<i32>(i & 1, i >> 1u);
    let at = clamp(base + o, vec2<i32>(0), size - 1);
    let r = ref_depth + dot(slope, vec2<f32>(o) - f);
    lit[i] = select(0.0, 1.0, r <= textureLoad(map, at, i32(layer), 0).r);
  }
  return mix(mix(lit[0], lit[1], f.x), mix(lit[2], lit[3], f.x), f.y);
}












fn pcf_tent(map: texture_2d_array<f32>, uv: vec2<f32>, layer: u32, ref_depth: f32, slope: vec2<f32>) -> f32 {
  let size = vec2<i32>(textureDimensions(map));
  let p = uv * vec2<f32>(size) - 1.0;
  let base = vec2<i32>(floor(p));
  let f = fract(p);
  let wx = array<f32, 3>(1.0 - f.x, 1.0, f.x);
  let wy = array<f32, 3>(1.0 - f.y, 1.0, f.y);
  var lit = 0.0;
  for (var j = 0; j < 3; j++) {
    var row = 0.0;
    for (var i = 0; i < 3; i++) {
      let at = clamp(base + vec2<i32>(i, j), vec2<i32>(0), size - 1);
      let r = ref_depth + dot(slope, vec2<f32>(f32(i), f32(j)) - 0.5 - f);
      row += wx[i] * select(0.0, 1.0, r <= textureLoad(map, at, i32(layer), 0).r);
    }
    lit += wy[j] * row;
  }
  return lit / 4.0;
}










fn receiver_slope(a: vec3<f32>, b: vec3<f32>, c: vec3<f32>, size: vec2<f32>, n_dot_l: f32) -> vec2<f32> {
  let d1 = (b.xy - a.xy) * size;
  let d2 = (c.xy - a.xy) * size;
  let det = d1.x * d2.y - d1.y * d2.x;
  if (abs(det) < 1e-12) { return vec2<f32>(0.0); }
  let z1 = b.z - a.z;
  let z2 = c.z - a.z;
  let g = vec2<f32>(z1 * d2.y - z2 * d1.y, z2 * d1.x - z1 * d2.x) / det;
  let sin_l = sqrt(max(1.0 - n_dot_l * n_dot_l, 0.0));
  return g * min(1.0, 4.0 * n_dot_l / max(sin_l, 1e-6));
}


fn plane_tangents(n: vec3<f32>) -> array<vec3<f32>, 2> {
  let k = select(vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(0.0, 1.0, 0.0), abs(n.x) > 0.9);
  let t = normalize(cross(n, k));
  return array<vec3<f32>, 2>(t, cross(n, t));
}



fn cascade_index(p: vec3<f32>) -> u32 {
  let count = u32(frame.light_dir.w);
  let depth = dot(p - frame.view_pos.xyz, frame.view_forward.xyz);
  for (var c = 0u; c < count; c++) {
    if (depth <= frame.cascade_bias[c].z) { return c; }
  }
  return CASCADES;
}



fn cascade_tint(c: u32) -> vec3<f32> {
  switch c {
    case 0u: { return vec3<f32>(1.0, 0.0, 0.0); }
    case 1u: { return vec3<f32>(0.0, 1.0, 0.0); }
    case 2u: { return vec3<f32>(0.0, 0.0, 1.0); }
    case 3u: { return vec3<f32>(1.0, 1.0, 0.0); }
    default: { return vec3<f32>(0.0); }
  }
}

















var<private> shadow_closed: bool = false;








fn shadow_turn(texel: vec2<f32>) -> f32 {
  let k = max(frame.shadow.w - 1.0, 0.0);
  return 6.2831853 * fract(52.9829189 * fract(dot(texel + k * 5.588238, vec2<f32>(0.06711056, 0.00583715))));
}











const PCSS_TAPS: i32 = 16;
const PCSS_MAX_TEXELS: f32 = 24.0;

fn vogel(i: i32, n: i32, radius: f32, turn: f32) -> vec2<f32> {
  let r = sqrt((f32(i) + 0.5) / f32(n)) * radius;
  let a = f32(i) * 2.39996323 + turn;
  return vec2<f32>(cos(a), sin(a)) * r;
}

fn pcss(uv: vec2<f32>, c: u32, ref_depth: f32, bias: vec4<f32>, tan_l: f32) -> f32 {
  let size = vec2<f32>(textureDimensions(cascade_map));
  let isize = vec2<i32>(size);

  let texel = bias.x;
  let depth_range = texel / max(bias.y, 1e-12);
  let softness = frame.shadow.x;



  let slope = select(bias.y * min(tan_l, 4.0), 0.0, shadow_closed);
  let reference = select(ref_depth, ref_depth + 1.25 * bias.y, shadow_closed);
  let turn = shadow_turn(uv * size);
  let search = clamp(softness * ref_depth * depth_range / texel, 1.0, PCSS_MAX_TEXELS);
  var blockers = 0.0;
  var sum = 0.0;
  for (var i = 0; i < PCSS_TAPS; i++) {
    let o = vogel(i, PCSS_TAPS, search, turn);
    let at = clamp(vec2<i32>(floor(uv * size + o)), vec2<i32>(0), isize - 1);
    let d = textureLoad(cascade_map, at, i32(c), 0).r;
    if (d < reference - slope * length(o)) {
      blockers += 1.0;
      sum += d;
    }
  }
  if (blockers < 0.5) { return 1.0; }
  let blocker = sum / blockers;








  let disc = softness * (ref_depth - blocker) * depth_range / texel;
  let penumbra = min(sqrt(disc * disc + 1.0), PCSS_MAX_TEXELS);
  var lit = 0.0;
  for (var i = 0; i < PCSS_TAPS; i++) {
    let o = vogel(i, PCSS_TAPS, penumbra, turn);
    lit += pcf(cascade_map, uv + o / size, c, reference - slope * length(o), vec2<f32>(0.0));
  }
  lit /= f32(PCSS_TAPS);
  let fill = frame.shadow.z;
  if (fill <= 0.0) { return lit; }









  let cone = min((ref_depth - blocker) * depth_range / texel, 4.0 * PCSS_MAX_TEXELS);
  var open = 0.0;
  for (var i = 0; i < PCSS_TAPS; i++) {
    let o = vogel(i, PCSS_TAPS, cone, turn + 1.3);
    let at = clamp(vec2<i32>(floor(uv * size + o)), vec2<i32>(0), isize - 1);
    open += select(0.0, 1.0, textureLoad(cascade_map, at, i32(c), 0).r >= reference - slope * length(o));
  }
  return mix(lit, 1.0, fill * open / f32(PCSS_TAPS));
}








fn cascade_sample(c: u32, p: vec3<f32>, n: vec3<f32>, n_dot_l: f32, wide: bool) -> f32 {
  let bias = frame.cascade_bias[c];
  let sin_l = sqrt(max(1.0 - n_dot_l * n_dot_l, 0.0));
  let offset = p + n * bias.x * (0.25 + sin_l);
  let sp = frame.cascade_view_proj[c] * vec4<f32>(offset, 1.0);
  let uv = vec2<f32>(sp.x * 0.5 + 0.5, 0.5 - sp.y * 0.5);



  if (sp.z > 1.0 || any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0))) { return -1.0; }
  if (frame.shadow.x > 0.0) {
    return pcss(uv, c, sp.z - bias.y, bias, sin_l / max(n_dot_l, 0.05));
  }
  if (!wide) { return pcf(cascade_map, uv, c, sp.z - bias.y, vec2<f32>(0.0)); }

  if (shadow_closed) { return pcf_tent(cascade_map, uv, c, sp.z + 0.25 * bias.y, vec2<f32>(0.0)); }

  let t = plane_tangents(n);
  let sb = frame.cascade_view_proj[c] * vec4<f32>(offset + t[0] * bias.x, 1.0);
  let sc = frame.cascade_view_proj[c] * vec4<f32>(offset + t[1] * bias.x, 1.0);
  let size = vec2<f32>(textureDimensions(cascade_map));
  let slope = receiver_slope(
    vec3<f32>(uv, sp.z),
    vec3<f32>(sb.x * 0.5 + 0.5, 0.5 - sb.y * 0.5, sb.z),
    vec3<f32>(sc.x * 0.5 + 0.5, 0.5 - sc.y * 0.5, sc.z),
    size,
    n_dot_l,
  );
  return pcf_tent(cascade_map, uv, c, sp.z - bias.y, slope);
}







fn cascade_shadow(p: vec3<f32>, n: vec3<f32>, n_dot_l: f32) -> f32 {
  return cascade_shadow_lookup(p, n, n_dot_l, true);
}

fn cascade_shadow_lookup(p: vec3<f32>, n: vec3<f32>, n_dot_l: f32, wide: bool) -> f32 {
  let c = cascade_index(p);
  if (c >= CASCADES) { return 1.0; }
  var shadow = cascade_sample(c, p, n, n_dot_l, wide);
  if (shadow < 0.0) { return 1.0; }
  let count = u32(frame.light_dir.w);
  let depth = dot(p - frame.view_pos.xyz, frame.view_forward.xyz);
  let band = frame.cascade_bias[c].w;
  if (c + 1u < count && depth > band) {
    var next = cascade_sample(c + 1u, p, n, n_dot_l, wide);
    if (next < 0.0) { next = 1.0; }
    shadow = mix(shadow, next, smoothstep(band, frame.cascade_bias[c].z, depth));
  }
  let last = frame.cascade_bias[count - 1u].z;
  return mix(shadow, 1.0, smoothstep(0.9 * last, last, depth));
}


fn cube_face(d: vec3<f32>) -> u32 {
  let a = abs(d);
  if (a.x >= a.y && a.x >= a.z) { return select(1u, 0u, d.x > 0.0); }
  if (a.y >= a.z) { return select(3u, 2u, d.y > 0.0); }
  return select(5u, 4u, d.z > 0.0);
}





const PARABOLOID_SCALE: f32 = 0.9;




fn paraboloid_uv_ref(light: LocalLight, q: vec3<f32>, sigma: f32, k: f32) -> vec3<f32> {
  let to = q - light.position_range.xyz;
  let l = vec3<f32>(to.x, sigma * to.z, -sigma * to.y);
  let d = max(length(l), 1e-6);
  let u = l / d;
  let ndc = vec2<f32>(-u.x, u.y) * (PARABOLOID_SCALE / (1.0 + u.z));
  return vec3<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5, d * k / light.position_range.w);
}




fn perspective_uv_ref(light: LocalLight, layer: u32, q: vec3<f32>, k: f32) -> vec4<f32> {
  let clip = frame.local_view_proj[layer] * vec4<f32>(q, 1.0);
  let ndc = clip.xy / clip.w;
  let r = length(q - light.position_range.xyz) * k / light.position_range.w;
  return vec4<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5, r, clip.w);
}













fn local_bias(light: LocalLight, past: f32) -> f32 {
  return select(1.0 - light.cone.w, 1.0 + 10.0 * past * light.cone.w, shadow_closed);
}



fn local_shadow(light: LocalLight, p: vec3<f32>, n: vec3<f32>) -> f32 {
  return local_shadow_lookup(light, p, n, true);
}
















fn local_wide(uv: vec2<f32>, layer: u32, ref_depth: f32, slope: vec2<f32>) -> f32 {
  let size = vec2<i32>(textureDimensions(local_shadow_map));
  let p = uv * vec2<f32>(size) - 0.5;
  let base = vec2<i32>(floor(p)) - 3;
  let f = fract(p);
  var wide = 0.0;
  var narrow = 0.0;
  var blockers = 0.0;
  var sum = 0.0;
  for (var j = 0; j < 8; j++) {
    let wy = select(select(1.0, f.y, j == 7), 1.0 - f.y, j == 0);
    let ny = select(0.0, select(select(1.0, f.y, j == 5), 1.0 - f.y, j == 2), j >= 2 && j <= 5);
    for (var i = 0; i < 8; i++) {
      let wx = select(select(1.0, f.x, i == 7), 1.0 - f.x, i == 0);
      let nx = select(0.0, select(select(1.0, f.x, i == 5), 1.0 - f.x, i == 2), i >= 2 && i <= 5);
      let at = clamp(base + vec2<i32>(i, j), vec2<i32>(0), size - 1);
      let d = textureLoad(local_shadow_map, at, i32(layer), 0).r;
      let r = ref_depth + dot(slope, vec2<f32>(f32(i - 3), f32(j - 3)) - f);
      let w = wx * wy;
      if (r <= d) {
        wide += w;
        narrow += nx * ny;
      } else {
        blockers += w;
        sum += w * d;
      }
    }
  }
  if (blockers <= 0.0) { return 1.0; }
  let blocker = max(sum / blockers, 1e-6);
  let t = clamp((ref_depth - blocker) / blocker / 0.3, 0.0, 1.0);
  return mix(narrow / 9.0, wide / 49.0, t);
}



fn local_filter(uv: vec2<f32>, layer: u32, ref_depth: f32, slope: vec2<f32>, wide: bool) -> f32 {
  if (!wide) { return pcf(local_shadow_map, uv, layer, ref_depth, vec2<f32>(0.0)); }
  return local_wide(uv, layer, ref_depth, slope);
}











fn local_shadow_lookup(light: LocalLight, p: vec3<f32>, n: vec3<f32>, wide: bool) -> f32 {
  let slot = i32(light.direction_slot.w);
  if (slot < 0) { return 1.0; }
  let range = light.position_range.w;
  let to_frag = p - light.position_range.xyz;
  let dist = length(to_frag);
  let n_dot_l = clamp(dot(n, -to_frag / max(dist, 1e-6)), 0.0, 1.0);
  let sin_l = sqrt(1.0 - n_dot_l * n_dot_l);
  let offset = p + n * light.cone.z * dist * (0.25 + sin_l);
  var layer = u32(slot);
  let size = vec2<f32>(textureDimensions(local_shadow_map));
  let paraboloid = light.color_kind.w < 0.5 && frame.shadow.y > 0.5;
  let k = local_bias(light, select(0.25, 0.5, paraboloid));
  let plane = wide && !shadow_closed;


  let t = plane_tangents(n);
  let h = dist * 2.0 / size.x;
  if (paraboloid) {
    let below = offset.y - light.position_range.y <= 0.0;
    let sigma = select(-1.0, 1.0, below);
    layer += select(1u, 0u, below);
    let a = paraboloid_uv_ref(light, offset, sigma, k);
    var slope = vec2<f32>(0.0);
    if (plane) {
      slope = receiver_slope(
        a,
        paraboloid_uv_ref(light, offset + t[0] * h, sigma, k),
        paraboloid_uv_ref(light, offset + t[1] * h, sigma, k),
        size,
        n_dot_l,
      );
    }
    return local_filter(a.xy, layer, a.z, slope, wide);
  }
  if (light.color_kind.w < 0.5) { layer += cube_face(to_frag); }
  let a = perspective_uv_ref(light, layer, offset, k);
  if (a.w <= 0.0) { return 1.0; }
  let uv = a.xy;
  if (light.color_kind.w > 0.5 && (any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0)))) { return 1.0; }
  let at = clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0));
  var slope = vec2<f32>(0.0);
  if (plane) {
    slope = receiver_slope(
      a.xyz,
      perspective_uv_ref(light, layer, offset + t[0] * h, k).xyz,
      perspective_uv_ref(light, layer, offset + t[1] * h, k).xyz,
      size,
      n_dot_l,
    );
  }
  return local_filter(at, layer, a.z, slope, wide);
}

struct Surface {
  albedo: vec3<f32>,
  metallic: f32,
  roughness: f32,
  f0: vec3<f32>,


  coat: f32,
  coat_roughness: f32,
  coat_normal: vec3<f32>,
  translucency: f32,
};




fn leaf_transmission(s: Surface, n: vec3f, v: vec3f, l: vec3f) -> vec3f {
  if (s.translucency <= 0.0) { return vec3f(0.0); }
  let back = max(-dot(n, l), 0.0);
  let through = pow(max(dot(v, -normalize(l + n * 0.3)), 0.0), 6.0);
  return s.albedo * (1.0 - s.metallic) * s.translucency * back * (3.0 * through + 0.16);
}



fn leaf_shadow_normal(s: Surface, n: vec3f, to_light: vec3f) -> vec3f {
  return select(n, -n, s.translucency > 0.0 && dot(n, to_light) < 0.0);
}

fn brdf(s: Surface, n: vec3<f32>, v: vec3<f32>, l: vec3<f32>) -> vec3<f32> {
  let h = normalize(v + l);
  let n_dot_l = max(dot(n, l), 0.0);
  let n_dot_v = max(dot(n, v), 1e-4);
  let n_dot_h = max(dot(n, h), 0.0);
  let h_dot_v = max(dot(h, v), 0.0);
  let a = s.roughness * s.roughness;
  let d = d_ggx(n_dot_h, a);
  let g = g_smith(n_dot_v, n_dot_l, s.roughness);
  let f = f_schlick(h_dot_v, s.f0);
  let spec = (d * g * f) / (4.0 * n_dot_v * n_dot_l + 1e-4);
  let kd = (vec3<f32>(1.0) - f) * (1.0 - s.metallic);
  let base = (kd * s.albedo / PI + spec) * n_dot_l + leaf_transmission(s, n, v, l);
  if (s.coat <= 0.0) { return base; }


  let nc = s.coat_normal;
  let c_dot_l = max(dot(nc, l), 0.0);
  let ac = s.coat_roughness * s.coat_roughness;
  let fc = f_schlick(h_dot_v, vec3<f32>(0.04)).x * s.coat;
  let dc = d_ggx(max(dot(nc, h), 0.0), ac);
  let vc = 0.25 / max(h_dot_v * h_dot_v, 1e-4);
  return base * (1.0 - fc) + vec3<f32>(dc * vc * fc * c_dot_l);
}


fn oct_encode(n: vec3<f32>) -> vec2<f32> {
  let p = n.xy / (abs(n.x) + abs(n.y) + abs(n.z));
  if (n.z >= 0.0) { return p; }
  return (1.0 - abs(p.yx)) * select(vec2<f32>(-1.0), vec2<f32>(1.0), p >= vec2<f32>(0.0));
}



fn caustics(p: vec3<f32>) -> f32 {
  let c = frame.caustics;
  if (c.y <= 0.0) { return 0.0; }
  let area = frame.caustics_area;
  if (any(p.xz < area.xy) || any(p.xz > area.zw)) { return 0.0; }
  let depth = c.x - p.y;
  if (depth <= 0.0) { return 0.0; }
  let t = frame.time.x;

  let q = p.xz * c.z;
  let ca = frame.caustics_cells[0];
  let cb = frame.caustics_cells[1];
  let a = caustic_cells(q + ca.zw + vec2<f32>(t * 0.21, t * 0.13), ca.xy);
  let b = caustic_cells(q * 1.37 + cb.zw + vec2<f32>(-t * 0.17, t * 0.23) + 3.7, cb.xy);
  let fade = smoothstep(0.0, 0.4, depth) * exp(-depth * c.w);
  return pow(min(a, b), 3.0) * 4.0 * c.y * fade;
}

fn caustic_hash(p: vec2<f32>) -> vec2<f32> {
  let c = vec2<i32>(p);
  return vec2<f32>(lattice_hash(vec3<i32>(c, 1)), lattice_hash(vec3<i32>(c, 2)));
}


fn caustic_cells(p: vec2<f32>, whole: vec2<f32>) -> f32 {
  let i = floor(p) + whole;
  let f = fract(p);
  var d1 = 8.0;
  var d2 = 8.0;
  for (var y = -1; y <= 1; y++) {
    for (var x = -1; x <= 1; x++) {
      let o = vec2<f32>(f32(x), f32(y));
      let r = o + caustic_hash(i + o) - f;
      let d = dot(r, r);
      if (d < d1) { d2 = d1; d1 = d; } else if (d < d2) { d2 = d; }
    }
  }
  return 1.0 - clamp((sqrt(d2) - sqrt(d1)) * 2.2, 0.0, 1.0);
}










struct Weathered {
  s: Surface,
  n: vec3<f32>,
};


fn weather_noise(p: vec2<f32>) -> f32 {
  let c = floor(p);
  let f = p - c;
  let u = f * f * (3.0 - 2.0 * f);
  let ci = vec2<i32>(c);
  let a = lattice_hash(vec3<i32>(ci, 11));
  let b = lattice_hash(vec3<i32>(ci + vec2<i32>(1, 0), 11));
  let d = lattice_hash(vec3<i32>(ci + vec2<i32>(0, 1), 11));
  let e = lattice_hash(vec3<i32>(ci + vec2<i32>(1, 1), 11));
  return mix(mix(a, b, u.x), mix(d, e, u.x), u.y);
}

fn weather_fbm(p: vec2<f32>) -> f32 {
  return weather_noise(p) * 0.57 + weather_noise(p * 2.03 + 17.1) * 0.29 + weather_noise(p * 4.07 + 3.3) * 0.14;
}




fn weather_ripples(p: vec2<f32>, t: f32) -> vec2<f32> {
  var tilt = vec2<f32>(0.0);
  for (var k = 0; k < 2; k++) {
    let q = p * 3.0 + vec2<f32>(f32(k) * 0.37, f32(k) * 0.71);
    let c = floor(q);
    let ci = vec2<i32>(c);
    let centre = c + vec2<f32>(lattice_hash(vec3<i32>(ci, 21 + k)), lattice_hash(vec3<i32>(ci, 31 + k))) * 0.6 + 0.2;
    let phase = fract(t + lattice_hash(vec3<i32>(ci, 41 + k)));
    let to = q - centre;
    let r = length(to);
    let ring = r - phase * 0.45;
    let wave = sin(ring * 50.0) * exp(-ring * ring * 300.0) * (1.0 - phase);
    tilt += to / max(r, 1e-3) * wave;
  }
  return tilt * 0.35;
}











fn weather_open(p: vec3<f32>, geo: vec3<f32>) -> f32 {
  let ws = frame.weather_shelter;
  if (ws.x < 0.5) { return 1.0; }
  let c = frame.weather_view_proj * vec4<f32>(p, 1.0);
  let uv = vec2<f32>(c.x * 0.5 + 0.5, 0.5 - c.y * 0.5);
  if (any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0)) || c.z < 0.0 || c.z > 1.0) { return 1.0; }
  let layer = i32(ws.x - 0.5);
  let side = i32(ws.z);



  let pw = p + frame.origin.xyz;
  let wander = vec2<f32>(weather_noise(pw.xz * 1.7 + 3.0), weather_noise(pw.xz * 1.7 + 29.0)) - 0.5;
  let at = uv * ws.z + wander * ws.y * 1.6;
  let step = max(ws.y * 0.5, 1.0);



  let helper = select(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), abs(geo.y) > 0.9);
  let t1 = normalize(cross(geo, helper));
  let t2 = cross(geo, t1);
  let nc = cross((frame.weather_view_proj * vec4<f32>(t1, 0.0)).xyz, (frame.weather_view_proj * vec4<f32>(t2, 0.0)).xyz);
  let per_texel = 2.0 / ws.z;
  var gradient = vec2<f32>(0.0);
  if (abs(nc.z) > 1e-8) {
    gradient = vec2<f32>(-nc.x, nc.y) / nc.z * per_texel;
  }
  var open = 0.0;
  for (var j = -2; j <= 2; j++) {
    for (var i = -2; i <= 2; i++) {
      let tap = at + vec2<f32>(f32(i), f32(j)) * step;
      let t = clamp(vec2<i32>(tap), vec2<i32>(0), vec2<i32>(side - 1));
      let nearest = textureLoad(cascade_map, t, layer, 0).r;

      let plane = c.z + clamp(dot(gradient, tap - uv * ws.z), -0.01, 0.01);
      open += select(0.0, 1.0, plane - ws.w <= nearest);
    }
  }
  return smoothstep(0.1, 0.9, open / 25.0);
}

fn weather_surface(s_in: Surface, n_in: vec3<f32>, geo: vec3<f32>, p_frame: vec3<f32>) -> Weathered {
  var o: Weathered;
  o.s = s_in;
  o.n = n_in;
  var w = frame.weather;
  if (max(max(w.x, w.y), w.z) <= 0.0) { return o; }

  w *= weather_open(p_frame, geo);
  if (max(max(w.x, w.y), w.z) <= 0.0) { return o; }
  let p = p_frame + frame.origin.xyz;
  let up = clamp(geo.y, 0.0, 1.0);


  let cover = weather_fbm(p.xz * 0.6 + p.y * 0.7) * 0.6 + weather_noise(p.xz * 7.1 + p.y * 5.3 + 7.0) * 0.4;



  var snow = 0.0;
  if (w.z > 0.0) {
    let reach = mix(1.05, 0.15, w.z);
    snow = smoothstep(reach, reach + 0.04, up + (cover - 0.5) * 0.7 * (1.0 - w.z * 0.5));
    snow *= smoothstep(0.0, 0.08, w.z);
  }










  let wet = w.x * mix(0.55, 1.0, up) * (1.0 - snow);
  let porosity = (1.0 - o.s.metallic) * smoothstep(0.15, 0.85, o.s.roughness);
  o.s.albedo *= mix(1.0, 0.3, wet * porosity);
  o.s.roughness = mix(o.s.roughness, max(o.s.roughness * 0.5, 0.1), wet * wet * (1.0 - porosity));



  if (w.y > 0.0) {
    let hollow = 1.0 - weather_fbm(p.xz / max(frame.weather_look.w, 0.05) + 41.0);
    let level = smoothstep(0.93, 0.985, up);
    let puddle = smoothstep(1.0 - w.y * 0.65, 1.0 - w.y * 0.65 + 0.06, hollow) * level * (1.0 - snow);
    if (puddle > 0.0) {
      o.s.albedo *= mix(1.0, 0.25, puddle);
      o.s.f0 = mix(o.s.f0, vec3<f32>(0.02), puddle);
      o.s.metallic = mix(o.s.metallic, 0.0, puddle);
      o.s.roughness = mix(o.s.roughness, 0.03, puddle);
      var flat_n = geo;
      if (w.w > 0.0) {
        let tilt = weather_ripples(p.xz, frame.time.x) * w.w;
        flat_n = normalize(geo + vec3<f32>(tilt.x, 0.0, tilt.y));
      }
      o.n = normalize(mix(o.n, flat_n, puddle));
    }
  }

  if (snow > 0.0) {
    let sparkle = 0.92 + 0.08 * weather_noise(p.xz * 9.0);
    o.s.albedo = mix(o.s.albedo, frame.weather_look.rgb * sparkle, snow);
    o.s.metallic = mix(o.s.metallic, 0.0, snow);
    o.s.roughness = mix(o.s.roughness, 0.8, snow);
    o.s.f0 = mix(o.s.f0, vec3<f32>(0.04), snow);
    o.n = normalize(mix(o.n, geo, snow * 0.85));
  }
  return o;
}
const PROBE_VOLUMES: u32 = 8u;

// The baked lightmap texel of this fragment (fs_lightmap sets it; a: 1),
// irradiance over π.
var<private> baked_lightmap: vec4<f32>;

fn probe_irradiance(at: u32, n: vec3<f32>) -> vec3<f32> {
  let r = probe_data.v[at];
  let g = probe_data.v[at + 1u];
  let b = probe_data.v[at + 2u];
  return max(vec3<f32>(r.w + dot(r.xyz, n), g.w + dot(g.xyz, n), b.w + dot(b.xyz, n)), vec3<f32>(0.0));
}

// The light volumes' irradiance over π at p for normal n: the baked ones'
// (`baked`) and the realtime ones' (`live`, probe_grid.rs), each with how
// much of it to take (w: 0 outside every volume of its kind, 1 a spacing
// inside one). A header's packed counts carry bit 21 on a realtime volume.
struct VolumeLight { baked: vec4<f32>, live: vec4<f32> };

fn light_volumes(p: vec3<f32>, n: vec3<f32>) -> VolumeLight {
  live_sh = array<vec4<f32>, 3>(vec4<f32>(0.0), vec4<f32>(0.0), vec4<f32>(0.0));
  var sum = array<vec3<f32>, 2>(vec3<f32>(0.0), vec3<f32>(0.0));
  var weight = array<f32, 2>(0.0, 0.0);
  for (var i = 0u; i < PROBE_VOLUMES; i++) {
    let h0 = probe_data.v[2u * i];
    let h1 = probe_data.v[2u * i + 1u];
    if (h0.w < 0.0) { break; }
    let packed = u32(h1.w);
    let kind = (packed >> 21u) & 1u;
    let c = vec3<u32>(packed & 127u, (packed >> 7u) & 127u, (packed >> 14u) & 127u);
    let cmax = vec3<f32>(c - vec3<u32>(1u));
    let size = max(h1.xyz - h0.xyz, vec3<f32>(1e-4));
    let d = min(p - h0.xyz, h1.xyz - p) / (size / cmax);
    // A baked volume fades in over its outer spacing; a realtime one is
    // whole up to its faces and fades out over a spacing beyond them, so
    // the walls, floor and ceiling just outside its probes take them.
    let inside = min(d.x, min(d.y, d.z));
    let w = clamp(select(inside, inside + 1.0, kind == 1u), 0.0, 1.0);
    if (w <= 0.0) { continue; }
    let g = clamp((p - h0.xyz) / size * cmax, vec3<f32>(0.0), cmax);
    let g0 = min(vec3<u32>(floor(g)), c - vec3<u32>(2u));
    let f = g - vec3<f32>(g0);
    var irr = vec3<f32>(0.0);
    for (var k = 0u; k < 8u; k++) {
      let o = vec3<u32>(k & 1u, (k >> 1u) & 1u, k >> 2u);
      let q = g0 + o;
      let wk = select(1.0 - f.x, f.x, o.x == 1u) * select(1.0 - f.y, f.y, o.y == 1u) * select(1.0 - f.z, f.z, o.z == 1u);
      let at = u32(h0.w) + ((q.z * c.y + q.y) * c.x + q.x) * 3u;
      irr += probe_irradiance(at, n) * wk;
      if (kind == 1u) {
        // The realtime probes' harmonics themselves, for their reflections.
        let ww = wk * w;
        live_sh[0] += probe_data.v[at] * ww;
        live_sh[1] += probe_data.v[at + 1u] * ww;
        live_sh[2] += probe_data.v[at + 2u] * ww;
      }
    }
    sum[kind] += irr * w;
    weight[kind] += w;
  }
  var out: VolumeLight;
  out.baked = select(vec4<f32>(0.0), vec4<f32>(sum[0] / weight[0], min(weight[0], 1.0)), weight[0] > 0.0);
  out.live = select(vec4<f32>(0.0), vec4<f32>(sum[1] / weight[1], min(weight[1], 1.0)), weight[1] > 0.0);
  if (weight[1] > 0.0) {
    let inv = 1.0 / weight[1];
    live_sh = array<vec4<f32>, 3>(live_sh[0] * inv, live_sh[1] * inv, live_sh[2] * inv);
  }
  return out;
}

// What `ambient_irradiance` leaves for `volume_reflected`: the realtime
// volumes' harmonics at the fragment and how much of them it took.
var<private> live_sh: array<vec4<f32>, 3>;
var<private> live_weight: f32;

// The realtime light volumes' reflection at `p` along `r` for
// `roughness`, given `prefiltered` (the reflected environment, or a
// ReflectionProbe's capture, in its units before its intensity): the
// light the probes' harmonics give along `r`, a soft image of the room
// (red off a red wall, light from a lit ceiling, dark below), which a
// sky's reflection would not be. Where a ReflectionProbe covers `p` its
// sharp capture (drawn with the probes' light) is kept for smooth
// surfaces, blurring into the probes' light as the roughness rises: a
// mirror in a room wants a probe. Outside every realtime volume (and on
// lightmapped fragments) `prefiltered` as it came.
fn volume_reflected(prefiltered: vec3<f32>, p: vec3<f32>, r: vec3<f32>, roughness: f32) -> vec3<f32> {
  if (live_weight <= 0.0) { return prefiltered; }
  let scale = 1.0 / max(frame.params.x, 1e-4);
  let along = max(vec3<f32>(live_sh[0].w + dot(live_sh[0].xyz, r), live_sh[1].w + dot(live_sh[1].xyz, r), live_sh[2].w + dot(live_sh[2].xyz, r)), vec3<f32>(0.0)) * scale;
  let probe = select(0.0, probe_weight(p), frame.probe_min.w > 0.5);
  let sharp = probe * (1.0 - smoothstep(0.4, 1.0, roughness));
  return mix(prefiltered, mix(along, prefiltered, sharp), live_weight);
}

// The diffuse ambient light of a lit fragment, in the environment
// irradiance's units (the caller scales it by the environment intensity,
// which baked and captured light do not take): its lightmap texel, else
// the realtime light volumes around it over the baked ones over `env`.
fn ambient_irradiance(env: vec3<f32>, p: vec3<f32>, n: vec3<f32>) -> vec3<f32> {
  let scale = 1.0 / max(frame.params.x, 1e-4);
  if (baked_lightmap.a > 0.5) { return baked_lightmap.rgb * scale; }
  let v = light_volumes(p, n);
  live_weight = v.live.w;
  let baked = mix(env, v.baked.rgb * scale, v.baked.w);
  return mix(baked, v.live.rgb * scale, v.live.w);
}

















struct LtcWords { v: array<vec4<u32>, 1024> };
@group(0) @binding(11) var<uniform> ltc_table: LtcWords;

struct LtcSample {
  m: vec4<f32>,
  fresnel: vec2<f32>,
};



fn ltc_half(h: u32) -> f32 {
  let e = (h >> 10u) & 31u;
  let m = f32(h & 1023u);
  let v = select((1.0 + m / 1024.0) * exp2(f32(e) - 15.0), m * exp2(-24.0), e == 0u);
  return select(v, -v, (h & 0x8000u) != 0u);
}

fn ltc_pair(w: u32) -> vec2<f32> {
  return vec2<f32>(ltc_half(w & 0xffffu), ltc_half(w >> 16u));
}

fn ltc_texel(x: u32, y: u32) -> LtcSample {
  let t = ltc_table.v[y * 32u + x];
  var s: LtcSample;
  s.m = vec4<f32>(ltc_pair(t.x), ltc_pair(t.y));
  s.fresnel = ltc_pair(t.z);
  return s;
}


fn ltc_lookup(uv: vec2<f32>) -> LtcSample {
  let g = clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0)) * 31.0;
  let i = min(vec2<u32>(floor(g)), vec2<u32>(30u));
  let f = g - vec2<f32>(i);
  let a = ltc_texel(i.x, i.y);
  let b = ltc_texel(i.x + 1u, i.y);
  let c = ltc_texel(i.x, i.y + 1u);
  let d = ltc_texel(i.x + 1u, i.y + 1u);
  var s: LtcSample;
  s.m = mix(mix(a.m, b.m, f.x), mix(c.m, d.m, f.x), f.y);
  s.fresnel = mix(mix(a.fresnel, b.fresnel, f.x), mix(c.fresnel, d.fresnel, f.x), f.y);
  return s;
}

fn ltc_edge(v1: vec3<f32>, v2: vec3<f32>) -> vec3<f32> {
  let x = dot(v1, v2);
  let y = abs(x);
  let a = 0.8543985 + (0.4965155 + 0.0145206 * y) * y;
  let b = 3.4175940 + (4.1616724 + y) * y;
  let v = a / b;
  let theta = select(0.5 * inverseSqrt(max(1.0 - x * x, 1e-7)) - v, v, x > 0.0);
  return cross(v1, v2) * theta;
}



fn ltc_evaluate(n: vec3<f32>, v: vec3<f32>, p: vec3<f32>, m_inv: mat3x3<f32>, rect: array<vec3<f32>, 4>) -> f32 {
  var t1 = v - n * dot(v, n);
  if (dot(t1, t1) < 1e-10) {
    t1 = select(vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(0.0, 1.0, 0.0), abs(n.x) > 0.9);
    t1 = t1 - n * dot(t1, n);
  }
  t1 = normalize(t1);
  let t2 = -cross(n, t1);
  let m = m_inv * transpose(mat3x3<f32>(t1, t2, n));
  var c = rect;
  for (var i = 0u; i < 4u; i++) {
    c[i] = normalize(m * (c[i] - p));
  }
  var f = ltc_edge(c[0], c[1]);
  f += ltc_edge(c[1], c[2]);
  f += ltc_edge(c[2], c[3]);
  f += ltc_edge(c[3], c[0]);
  let l = length(f);
  return max((l * l + f.z) / (l + 1.0), 0.0);
}



struct AreaShade {
  diffuse: vec3<f32>,
  specular: vec3<f32>,
};

fn area_shade(light: LocalLight, s: Surface, p: vec3<f32>, n: vec3<f32>, v: vec3<f32>) -> AreaShade {
  var o: AreaShade;
  o.diffuse = vec3<f32>(0.0);
  o.specular = vec3<f32>(0.0);
  let center = light.position_range.xyz;
  var hw = light.direction_slot.xyz;
  let hh = light.cone.xyz;
  let to_p = p - center;
  let d2 = max(dot(to_p, to_p), 1e-4);
  let range = light.position_range.w;
  let ratio = d2 / (range * range);
  let window = clamp(1.0 - ratio * ratio, 0.0, 1.0);
  if (window <= 0.0) { return o; }


  if (dot(to_p, cross(hw, hh)) >= 0.0) {
    if (light.color_kind.w > -1.5) { return o; }
    hw = -hw;
  }
  let rect = array<vec3<f32>, 4>(center + hw - hh, center - hw - hh, center - hw + hh, center + hw + hh);
  let n_dot_v = clamp(dot(n, v), 1e-4, 1.0);
  let t = ltc_lookup(vec2<f32>(s.roughness, sqrt(1.0 - n_dot_v)));
  let m_inv = mat3x3<f32>(vec3<f32>(t.m.x, 0.0, t.m.y), vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(t.m.z, 0.0, t.m.w));
  let fresnel = s.f0 * t.fresnel.x + (vec3<f32>(1.0) - s.f0) * t.fresnel.y;
  let identity = mat3x3<f32>(vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(0.0, 0.0, 1.0));
  let radiance = light.color_kind.rgb * window * window;
  o.specular = radiance * fresnel * ltc_evaluate(n, v, p, m_inv, rect);
  o.diffuse = radiance * s.albedo * (1.0 - s.metallic) * ltc_evaluate(n, v, p, identity, rect);
  if (s.translucency > 0.0) {

    o.diffuse += radiance * s.albedo * (1.0 - s.metallic) * s.translucency * ltc_evaluate(-n, -v, p, identity, rect);
  }
  return o;
}

fn area_light(light: LocalLight, s: Surface, p: vec3<f32>, n: vec3<f32>, v: vec3<f32>) -> vec3<f32> {
  let o = area_shade(light, s, p, n, v);
  return (o.diffuse + o.specular) * rect_shadow(light, p, leaf_shadow_normal(s, n, light.position_range.xyz - p));
}



const RECT_SHADOW_TAN: f32 = 3.7320508;



fn rect_shadow_tap(i: u32, n: u32, turn: f32) -> vec2<f32> {
  let r = sqrt((f32(i) + 0.5) / f32(n));
  let a = f32(i) * 2.3999632 + turn;
  return vec2<f32>(cos(a), sin(a)) * r;
}




fn rect_shadow_turn(p: vec3<f32>) -> f32 {
  var h = bitcast<u32>(p.x) ^ (bitcast<u32>(p.y) * 0x9e3779b9u) ^ (bitcast<u32>(p.z) * 0x85ebca6bu);
  h = h * 747796405u + 2891336453u;
  h = ((h >> ((h >> 28u) + 4u)) ^ h) * 277803737u;
  h = (h >> 22u) ^ h;
  return f32(h >> 8u) * (6.2831853 / 16777216.0);
}








fn rect_shadow(light: LocalLight, p: vec3<f32>, n: vec3<f32>) -> f32 {
  let slot = i32(light.direction_slot.w);
  if (slot < 0) { return 1.0; }
  let center = light.position_range.xyz;
  let range = light.position_range.w;
  let hw = light.direction_slot.xyz;
  let hh = light.cone.xyz;
  let to = p - center;
  var layer = u32(slot);
  if (dot(to, cross(hw, hh)) > 0.0) {

    if (light.color_kind.w > -1.5) { return 1.0; }
    layer += 1u;
  }
  let dist = length(to);

  let texels = f32(textureDimensions(local_shadow_map).x);
  let offset = p + n * (4.0 * RECT_SHADOW_TAN / texels) * dist;
  let clip = frame.local_view_proj[layer] * vec4<f32>(offset, 1.0);
  if (clip.w <= 0.0) { return 1.0; }
  let ndc = clip.xy / clip.w;
  let uv = vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
  if (any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0))) { return 1.0; }
  let receiver = length(offset - center);
  let ref_depth = receiver / range - 0.002;


  let size = max(length(hw), length(hh));
  let per_metre = 1.0 / (2.0 * receiver * RECT_SHADOW_TAN);
  let texel = 1.0 / texels;
  let search = clamp(size * per_metre, 2.0 * texel, 48.0 * texel);
  let size_i = vec2<i32>(textureDimensions(local_shadow_map));
  let turn = rect_shadow_turn(p);
  var blockers = 0.0;
  var count = 0.0;
  for (var i = 0u; i < 16u; i++) {
    let q = clamp(uv + rect_shadow_tap(i, 16u, turn) * search, vec2<f32>(0.0), vec2<f32>(1.0));
    let at = clamp(vec2<i32>(q * vec2<f32>(size_i)), vec2<i32>(0), size_i - 1);
    let d = textureLoad(local_shadow_map, at, i32(layer), 0).r;
    if (d < ref_depth) {
      blockers += d;
      count += 1.0;
    }
  }
  if (count < 0.5) { return 1.0; }
  let blocker = max(blockers / count * range, 1e-3);
  let penumbra = clamp(size * (receiver - blocker) / blocker * per_metre, texel, 48.0 * texel);
  var lit = 0.0;
  for (var i = 0u; i < 24u; i++) {
    let q = clamp(uv + rect_shadow_tap(i, 24u, turn) * penumbra, vec2<f32>(0.0), vec2<f32>(1.0));
    lit += pcf(local_shadow_map, q, layer, ref_depth, vec2<f32>(0.0));
  }
  return lit / 24.0;
}

fn cookie_texel(layer: u32, at: vec2<i32>) -> vec3<f32> {
  let u = u32(textureLoad(local_shadow_map, at, i32(layer), 0).r);
  return vec3<f32>(f32((u >> 16u) & 255u), f32((u >> 8u) & 255u), f32(u & 255u)) / 255.0;
}

fn cookie_sample(layer: u32, uv: vec2<f32>) -> vec3<f32> {
  let size = vec2<i32>(textureDimensions(local_shadow_map));
  let q = clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0)) * vec2<f32>(size) - 0.5;
  let base = vec2<i32>(floor(q));
  let f = q - floor(q);
  let a = cookie_texel(layer, clamp(base, vec2<i32>(0), size - 1));
  let b = cookie_texel(layer, clamp(base + vec2<i32>(1, 0), vec2<i32>(0), size - 1));
  let c = cookie_texel(layer, clamp(base + vec2<i32>(0, 1), vec2<i32>(0), size - 1));
  let d = cookie_texel(layer, clamp(base + vec2<i32>(1, 1), vec2<i32>(0), size - 1));
  return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}



fn light_cookie(light: LocalLight, d: vec3<f32>) -> vec3<f32> {
  let kind = light.color_kind.w;
  if (kind > 1.5) {
    let layer = u32(kind - 2.0 + 0.5);
    let axis = light.direction_slot.xyz;
    let up0 = select(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(0.0, 0.0, 1.0), abs(axis.y) > 0.99);
    let right = normalize(cross(axis, up0));
    let up = cross(right, axis);
    let z = dot(d, axis);
    if (z <= 1e-4) { return vec3<f32>(0.0); }
    let c = light.cone.x;
    let tan_outer = sqrt(max(1.0 - c * c, 0.0)) / max(c, 1e-4);
    let xy = vec2<f32>(dot(d, right), dot(d, up)) / (z * tan_outer);
    return cookie_sample(layer, vec2<f32>(xy.x * 0.5 + 0.5, 0.5 - xy.y * 0.5));
  }
  if (kind > -0.5 && kind < 0.5 && light.cone.y < 0.0) {
    let base = u32(-light.cone.y - 1.0 + 0.5);
    let face = cube_face(d);
    let a = abs(d);
    var sc = 0.0;
    var tc = 0.0;
    var ma = 1.0;
    switch face {
      case 0u: { sc = -d.z; tc = -d.y; ma = a.x; }
      case 1u: { sc = d.z; tc = -d.y; ma = a.x; }
      case 2u: { sc = d.x; tc = d.z; ma = a.y; }
      case 3u: { sc = d.x; tc = -d.z; ma = a.y; }
      case 4u: { sc = d.x; tc = -d.y; ma = a.z; }
      default: { sc = -d.x; tc = -d.y; ma = a.z; }
    }
    let uv = vec2<f32>(sc, tc) / max(ma, 1e-6) * 0.5 + 0.5;
    return cookie_sample(base + face, uv);
  }
  return vec3<f32>(1.0);
}






fn surface_local_light(light: LocalLight, s: Surface, p: vec3<f32>, n: vec3<f32>, v: vec3<f32>) -> vec3<f32> {
  if (light.color_kind.w < -0.5) { return area_light(light, s, p, n, v); }
  let to_light = light.position_range.xyz - p;
  let d2 = max(dot(to_light, to_light), 1e-4);
  let range = light.position_range.w;
  let ratio = d2 / (range * range);
  let window = clamp(1.0 - ratio * ratio, 0.0, 1.0);
  var atten = window * window / d2;
  let l = to_light * inverseSqrt(d2);
  if (light.color_kind.w > 0.5) {
    let cos_angle = dot(-l, light.direction_slot.xyz);
    let t = clamp((cos_angle - light.cone.x) * light.cone.y, 0.0, 1.0);
    atten *= t * t;
  }
  if (atten <= 0.0 || (dot(n, l) <= 0.0 && s.translucency <= 0.0)) { return vec3<f32>(0.0); }
  let radiance = light.color_kind.rgb * light_cookie(light, -l);
  return brdf(s, n, v, l) * radiance * atten * local_shadow(light, p, leaf_shadow_normal(s, n, l));
}






fn surface_local_lights(frag: vec2<f32>, s: Surface, p: vec3<f32>, n: vec3<f32>, v: vec3<f32>) -> vec3<f32> {
  var color = vec3<f32>(0.0);
  let lights = frame_lights(frag, p);
  for (var k = 0u; k < lights.y; k++) {
    color += surface_local_light(frame_light(lights, k), s, p, n, v);
  }
  return color;
}





fn instance_matrix(c0: vec3<f32>, c1: vec3<f32>, c2: vec3<f32>, c3: vec3<f32>) -> mat4x4<f32> {
  return mat4x4<f32>(vec4<f32>(c0, 0.0), vec4<f32>(c1, 0.0), vec4<f32>(c2, 0.0), vec4<f32>(c3, 1.0));
}








fn mul_point(m: mat4x4<f32>, p: vec4<f32>) -> vec4<f32> {
  return fma(m[3], vec4<f32>(p.w), fma(m[2], vec4<f32>(p.z), fma(m[1], vec4<f32>(p.y), m[0] * p.x)));
}

















struct DecalGpu {
  inv: mat4x4<f32>,
  axis: vec4<f32>,
  uv_rect: vec4<f32>,
  color: vec4<f32>,
  emissive: vec4<f32>,
  normal_rect: vec4<f32>,
  normal: vec4<f32>,
};
struct Fx {
  view_proj_unjittered: mat4x4<f32>,
  prev_view_proj: mat4x4<f32>,
  screen: vec4<f32>,
  ao: vec4<f32>,
  decal_info: vec4<f32>,
  planar: vec4<f32>,
};
@group(2) @binding(0) var<uniform> fx: Fx;
@group(2) @binding(1) var ssao_tex: texture_2d<f32>;






fn ambient_normal(clip: vec2<f32>, n: vec3<f32>) -> vec3<f32> {
  if (fx.ao.x < 0.5 || fx.ao.w < 1.5) { return n; }
  let b = textureSampleLevel(ssao_tex, fx_sampler, clip * fx.screen.zw, 0.0).gba;
  let l = length(b);
  if (l < 0.25) { return n; }


  let bn = b / l;
  return normalize(bn + n * max(0.05 - dot(bn, n), 0.0));
}
@group(2) @binding(2) var decal_atlas: texture_2d<f32>;
@group(2) @binding(3) var fx_sampler: sampler;



@group(2) @binding(5) var<storage, read> decal_list: array<DecalGpu>;
@group(2) @binding(6) var<storage, read> decal_cells: array<u32>;



struct VertexIn {
  @location(0) position: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
  @location(3) color: vec4<f32>,

  @location(15) tangent: vec4<f32>,
};


struct InstanceIn {
  @location(4) m0: vec3<f32>,
  @location(5) m1: vec3<f32>,
  @location(6) m2: vec3<f32>,
  @location(7) m3: vec3<f32>,
};








struct InstanceFxIn {
  @location(8) look: vec4<f32>,
};


struct SkinnedInstanceIn {
  @location(4) m0: vec3<f32>,
  @location(5) m1: vec3<f32>,
  @location(6) m2: vec3<f32>,
  @location(7) m3: vec3<f32>,
  @location(8) palette: vec4<f32>,
  @location(11) morph0: vec4<f32>,
  @location(12) morph1: vec4<f32>,
  @location(13) look: vec4<f32>,
};

struct SkinIn {
  @location(9) joints: vec4<u32>,
  @location(10) weights: vec4<f32>,
};



struct PrevIn {
  @location(11) p0: vec3<f32>,
  @location(12) p1: vec3<f32>,
  @location(13) p2: vec3<f32>,
  @location(14) p3: vec3<f32>,
};

struct VertexOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world_pos: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
  @location(3) color: vec4<f32>,
  @location(4) local_pos: vec3<f32>,

  @location(5) tangent: vec4<f32>,


  @location(6) affine: vec3<f32>,

  @location(8) @interpolate(flat) look: vec4<f32>,


  @location(9) @interpolate(flat) oit_key: vec2<u32>,
};



fn look_rgb(packed: f32) -> vec3<f32> {
  let u = u32(packed);
  return vec3<f32>(f32((u >> 16u) & 255u), f32((u >> 8u) & 255u), f32(u & 255u)) / 255.0;
}



fn look_tint(c: vec4<f32>, look: vec4<f32>) -> vec4<f32> {
  if (look.w <= 0.0) { return c; }
  return vec4<f32>(c.rgb * (vec3<f32>(1.0) - look_rgb(look.w)), c.a);
}


fn look_flash(c: vec3<f32>, look: vec4<f32>) -> vec3<f32> {
  if (look.y <= 0.0) { return c; }
  return mix(c, look_rgb(look.z), look.y);
}




fn look_receives(look: vec4<f32>) -> bool {
  return look.x >= 0.0;
}



fn look_layer(look: vec4<f32>) -> f32 {
  return select(-look.x - 1.0, look.x, look_receives(look));
}



fn stylised_uv(uv: vec2<f32>, affine: vec3<f32>) -> vec2<f32> {
  if (frame.stylise.z <= 0.0 || affine.z <= 0.0) { return uv; }
  return mix(uv, affine.xy / affine.z, frame.stylise.z);
}




fn retro_clip(clip: vec4<f32>) -> vec4<f32> {
  var c = clip;
  if (c.w <= 0.0) { return c; }
  if (frame.stylise.x > 0.0) {
    let half = frame.stylise.xy * 0.5;
    let ndc = c.xy / c.w;
    c = vec4<f32>(round(ndc * half) / half * c.w, c.zw);
  }
  if (frame.stylise.w > 0.0) {
    let steps = exp2(frame.stylise.w);
    c.z = floor(c.z / c.w * steps) / steps * c.w;
  }
  return c;
}

struct PrepassOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world_pos: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
  @location(3) color: vec4<f32>,
  @location(4) local_pos: vec3<f32>,
  @location(5) cur_clip: vec4<f32>,
  @location(6) prev_clip: vec4<f32>,


  @location(8) @interpolate(flat) object: u32,
};





fn object_id(m0: vec3<f32>, m1: vec3<f32>, m2: vec3<f32>, m3: vec3<f32>) -> u32 {
  var h = 2166136261u;
  for (var c = 0u; c < 3u; c++) {
    h = (h ^ bitcast<u32>(m3[c])) * 16777619u;
    h = (h ^ bitcast<u32>(m0[c] + m1[c] * 3.0 + m2[c] * 7.0)) * 16777619u;
  }
  h = h ^ (h >> 16u);
  return 1u + h % 255u;
}




fn world_normal(model: mat4x4<f32>, n: vec3<f32>) -> vec3<f32> {
  let a = model[0].xyz;
  let b = model[1].xyz;
  let c = model[2].xyz;
  let cofactor = mat3x3<f32>(cross(b, c), cross(c, a), cross(a, b));
  let mirrored = dot(a, cross(b, c)) < 0.0;
  return normalize(cofactor * n) * select(1.0, -1.0, mirrored);
}





struct LitPlace {
  clip: vec4<f32>,
  world: vec3<f32>,
  normal: vec3<f32>,
};

fn lit_place(model: mat4x4<f32>, position: vec3<f32>, normal: vec3<f32>, uv: vec2<f32>, color: vec4<f32>) -> LitPlace {
  var world = model * vec4<f32>(position, 1.0);
  let n = world_normal(model, normal);

  var h: Hook;
  h.world = world.xyz;
  h.local = position;
  h.normal = n;
  h.uv = uv;
  h.color = color;
  h.time = frame.time.x;
  h.origin = frame.origin.xyz;
  h.eye = frame.view_pos.xyz;
  h.front = 1.0;
  world = vec4<f32>(world.xyz + vertex_offset(h, shader_params()), world.w);
  var p: LitPlace;
  p.clip = retro_clip(moved_clip(frame.view_proj, world));
  p.world = world.xyz;
  p.normal = n;
  return p;
}

fn lit_vertex(model: mat4x4<f32>, v: VertexIn) -> VertexOut {
  let p = lit_place(model, v.position, v.normal, v.uv, v.color);
  var o: VertexOut;
  o.clip = p.clip;
  o.affine = vec3<f32>(v.uv * o.clip.w, o.clip.w);
  o.world_pos = p.world;
  o.normal = p.normal;
  o.uv = v.uv;
  o.color = v.color;
  o.local_pos = v.position;
  o.look = vec4<f32>(0.0);
  o.oit_key = oit_key(model, v.position);

  let t = (model * vec4<f32>(v.tangent.xyz, 0.0)).xyz;
  o.tangent = vec4<f32>(select(vec3<f32>(0.0), normalize(t), dot(t, t) > 1e-20), v.tangent.w);
  return o;
}


fn skin_matrix(s: SkinIn, palette: vec4<f32>) -> mat4x4<f32> {
  let total = s.weights.x + s.weights.y + s.weights.z + s.weights.w;
  if (total < 1e-6) {
    return mat4x4<f32>(vec4<f32>(1.0, 0.0, 0.0, 0.0), vec4<f32>(0.0, 1.0, 0.0, 0.0), vec4<f32>(0.0, 0.0, 1.0, 0.0), vec4<f32>(0.0, 0.0, 0.0, 1.0));
  }
  let w = s.weights / total;
  return palette_joint(palette, s.joints.x) * w.x
    + palette_joint(palette, s.joints.y) * w.y
    + palette_joint(palette, s.joints.z) * w.z
    + palette_joint(palette, s.joints.w) * w.w;
}

@vertex
fn vs(v: VertexIn, i: InstanceIn, k: InstanceFxIn) -> VertexOut {
  var o = lit_vertex(instance_matrix(i.m0, i.m1, i.m2, i.m3), v);
  o.look = k.look;
  return o;
}




fn morph_vertex(v: VertexIn, vi: u32, i: SkinnedInstanceIn) -> VertexIn {
  let m = morphed(v.position, v.normal, vi, i.palette, i.morph0.x, i.morph0.y);
  var o = v;
  o.position = m.position;
  o.normal = m.normal;
  return o;
}

@vertex
fn vs_skinned(@builtin(vertex_index) vi: u32, v: VertexIn, i: SkinnedInstanceIn, s: SkinIn) -> VertexOut {
  let model = instance_matrix(i.m0, i.m1, i.m2, i.m3) * skin_matrix(s, i.palette);
  var o = lit_vertex(model, morph_vertex(v, vi, i));
  o.look = i.look;
  return o;
}

fn prepass_vertex(model: mat4x4<f32>, prev: mat4x4<f32>, v: VertexIn) -> PrepassOut {
  return prepass_vertex_from(model, prev, v, v.position);
}



fn prepass_vertex_from(model: mat4x4<f32>, prev: mat4x4<f32>, v: VertexIn, before: vec3<f32>) -> PrepassOut {
  let o = lit_vertex(model, v);
  var p: PrepassOut;
  p.clip = o.clip;
  p.world_pos = o.world_pos;
  p.normal = o.normal;
  p.uv = o.uv;
  p.color = o.color;
  p.local_pos = o.local_pos;
  p.cur_clip = fx.view_proj_unjittered * vec4<f32>(o.world_pos, 1.0);
  var previous = prev * vec4<f32>(before, 1.0);
  var h: Hook;
  h.world = previous.xyz;
  h.local = before;
  h.normal = world_normal(prev, v.normal);
  h.uv = v.uv;
  h.color = v.color;
  h.time = frame.time.w;
  h.origin = frame.origin.xyz;
  h.eye = frame.view_pos.xyz;
  h.front = 1.0;
  previous = vec4<f32>(previous.xyz + vertex_offset(h, previous_shader_params()), previous.w);
  p.prev_clip = mul_point(fx.prev_view_proj, previous);
  p.object = object_id(model[0].xyz, model[1].xyz, model[2].xyz, model[3].xyz);
  return p;
}

@vertex
fn vs_prepass(v: VertexIn, i: InstanceIn, prev: PrevIn) -> PrepassOut {
  return prepass_vertex(instance_matrix(i.m0, i.m1, i.m2, i.m3), instance_matrix(prev.p0, prev.p1, prev.p2, prev.p3), v);
}


@vertex
fn vs_prepass_surface(v: VertexIn, i: InstanceIn, prev: PrevIn, @location(9) before: vec3<f32>) -> PrepassOut {
  return prepass_vertex_from(instance_matrix(i.m0,i.m1,i.m2,i.m3), instance_matrix(prev.p0,prev.p1,prev.p2,prev.p3), v, before);
}

@vertex
fn vs_prepass_skinned(@builtin(vertex_index) vi: u32, v: VertexIn, i: SkinnedInstanceIn, s: SkinIn) -> PrepassOut {
  let model = instance_matrix(i.m0, i.m1, i.m2, i.m3) * skin_matrix(s, i.palette);






  var prev = model;
  let section = fx.decal_info.w;
  let now = morph_vertex(v, vi, i);
  var before = now.position;
  if (i.palette.w < -0.5) {


    if (i.morph1.x > 0.5) {
      let then = vec4<f32>(i.morph1.y, i.palette.y, 0.0, -1.0 - i.morph1.z);
      prev = joint_palette[u32(i.morph1.x) - 1u] * skin_matrix(s, then);
    }
  } else if (section > 0.5) {
    if (dot(s.weights, vec4<f32>(1.0)) >= 1e-6) {
      prev = skin_matrix(s, vec4<f32>(i.palette.x + section, i.palette.yzw));
    }
    if (i.morph0.x > 0.5 || i.morph0.y > 0.5) {
      let weights = select(0.0, i.morph0.x + section, i.morph0.x > 0.5);
      let deformed = select(0.0, i.morph0.y + section, i.morph0.y > 0.5);
      before = morphed(v.position, v.normal, vi, i.palette, weights, deformed).position;
    }
  }
  var p = prepass_vertex_from(model, prev, now, before);



  p.object = object_id(i.m0, i.m1, i.m2, i.m3) | 256u;
  return p;
}


override EFFECTS: bool = true;





var<private> toon_aa: f32 = 0.0;



var<private> toon_metal_lit: f32 = 1.0;


fn toon_step(x: f32, edge: f32) -> f32 {
  if (toon_aa <= 0.0) { return select(0.0, 1.0, x > edge); }
  return smoothstep(edge - toon_aa, edge + toon_aa, x);
}




fn toon_ramp(x: f32) -> f32 {
  let t = clamp(x, 0.0, 1.0);
  if (material.toon.x < 1.5) {
    let w = i32(textureDimensions(gradient_tex).x);
    let p = t * f32(w);
    let i = clamp(i32(p), 0, w - 1);
    let here = textureLoad(gradient_tex, vec2<i32>(i, 0), 0).r;
    if (toon_aa <= 0.0) { return here; }

    let f = p - floor(p);
    let hw = max(toon_aa * f32(w), 1e-4);
    if (f < 0.5) {
      let before = textureLoad(gradient_tex, vec2<i32>(max(i - 1, 0), 0), 0).r;
      return mix(before, here, smoothstep(-hw, hw, f));
    }
    let after = textureLoad(gradient_tex, vec2<i32>(min(i + 1, w - 1), 0), 0).r;
    return mix(here, after, smoothstep(-hw, hw, f - 1.0));
  }
  return 0.22 + 0.38 * toon_step(t, 0.08) + 0.4 * toon_step(t, 0.45);
}








fn toon_light(s: Surface, n: vec3<f32>, v: vec3<f32>, l: vec3<f32>, shadow: f32) -> vec3<f32> {
  let n_dot_l = max(dot(n, l), 0.0);
  let band = toon_ramp(n_dot_l * shadow);
  let h = normalize(v + l);
  let shine = exp2(10.0 * (1.0 - s.roughness) + 1.0);
  let lobe = pow(max(dot(n, h), 0.0), shine) * shadow * select(0.0, 1.0, n_dot_l > 0.0);
  let spec = toon_step(lobe, 0.5);
  let peak = min((shine + 2.0) / 8.0, 4.0);
  let kd = mix(s.albedo, s.f0 * toon_metal_lit, s.metallic) / PI;
  return kd * band + s.f0 * spec * peak * mix(1.0 - s.roughness, 1.0, s.metallic);
}







fn shadow_keep(sun: f32, mode: u32) -> f32 {
  let d = frame.outline_style.y;
  if (d <= 0.0 || (mode & 8u) != 0u) { return 1.0; }
  var lit = smoothstep(0.0, 0.25, sun);
  if (EFFECTS && material.toon.x > 0.5) {
    let dark = toon_ramp(0.0);
    lit = clamp((toon_ramp(sun) - dark) / max(1.0 - dark, 1e-3), 0.0, 1.0);
  }
  return 1.0 - d * (1.0 - lit);
}



fn local_light(light: LocalLight, s: Surface, p: vec3<f32>, n: vec3<f32>, v: vec3<f32>, receives: bool) -> vec3<f32> {

  if (light.color_kind.w < -0.5) {
    let a = area_shade(light, s, p, n, v);
    return (a.diffuse + a.specular) * select(1.0, rect_shadow(light, p, leaf_shadow_normal(s, n, light.position_range.xyz - p)), receives);
  }
  let to_light = light.position_range.xyz - p;
  let d2 = max(dot(to_light, to_light), 1e-4);
  let range = light.position_range.w;
  let ratio = d2 / (range * range);

  let window = clamp(1.0 - ratio * ratio, 0.0, 1.0);
  var atten = window * window / d2;
  let l = to_light * inverseSqrt(d2);
  if (light.color_kind.w > 0.5) {
    let cos_angle = dot(-l, light.direction_slot.xyz);
    let t = clamp((cos_angle - light.cone.x) * light.cone.y, 0.0, 1.0);
    atten *= t * t;
  }
  if (atten <= 0.0) { return vec3<f32>(0.0); }
  if (dot(n, l) <= 0.0 && s.translucency <= 0.0) { return vec3<f32>(0.0); }
  var shadow = 1.0;
  if (receives) { shadow = local_shadow(light, p, leaf_shadow_normal(s, n, l)) * contact_shadow(light, p); }

  let radiance = light.color_kind.rgb * light_cookie(light, -l);
  if (EFFECTS && material.toon.x > 0.5) {
    return toon_light(s, n, v, l, shadow) * radiance * atten;
  }
  return brdf(s, n, v, l) * radiance * atten * shadow;
}

struct DecalResult {
  albedo: vec3<f32>,
  roughness: f32,
  emissive: vec3<f32>,

  normal: vec4<f32>,
};


fn decal_stored(c: vec3<f32>) -> vec3<f32> {
  return select(1.055 * pow(c, vec3<f32>(1.0 / 2.4)) - 0.055, c * 12.92, c <= vec3<f32>(0.0031308));
}





fn frame_decals(frag: vec2<f32>, p: vec3<f32>) -> vec2<u32> {
  if (fx.decal_info.x < 0.0 || frame.cluster.x <= 0.0) {
    return vec2<u32>(NO_CLUSTER, u32(abs(fx.decal_info.x)));
  }
  let word = decal_cells[cluster_at(frag, p)];
  return vec2<u32>(word & 0xffffffu, word >> 24u);
}











fn decal_sample(rect: vec4<f32>, t: vec2<f32>, gx: vec2<f32>, gy: vec2<f32>) -> vec4<f32> {
  let size = vec2<f32>(textureDimensions(decal_atlas));
  let lo = min(rect.xy, rect.zw);
  let hi = max(rect.xy, rect.zw);
  let span = rect.zw - rect.xy;
  let dx = gx * span * size;
  let dy = gy * span * size;
  let footprint = max(dot(dx, dx), dot(dy, dy));
  let region = max(min(hi.x - lo.x, hi.y - lo.y) * min(size.x, size.y), 1.0);
  let cap = max(log2(region / 8.0), 0.0);
  let lod = clamp(0.5 * log2(max(footprint, 1e-8)), 0.0, cap);
  let reach = 1.5 * exp2(ceil(lod)) / size;
  let mid = (lo + hi) * 0.5;
  let uv = clamp(mix(rect.xy, rect.zw, t), min(lo + reach, mid), max(hi - reach, mid));
  return textureSampleLevel(decal_atlas, fx_sampler, uv, lod);
}





fn apply_decals(frag: vec2<f32>, p: vec3<f32>, n: vec3<f32>, albedo: vec3<f32>, roughness: f32, dpx: vec3<f32>, dpy: vec3<f32>) -> DecalResult {
  var r: DecalResult;
  r.albedo = albedo;
  r.roughness = roughness;
  r.emissive = vec3<f32>(0.0);
  r.normal = vec4<f32>(n, 0.0);
  let range = frame_decals(frag, p);
  let cells = CLUSTER_X * CLUSTER_Y * CLUSTER_Z;
  for (var k = 0u; k < range.y; k++) {
    var i = k;
    if (range.x != NO_CLUSTER) { i = decal_cells[cells + range.x + k]; }
    let d = decal_list[i];
    let local = (d.inv * vec4<f32>(p, 1.0)).xyz;
    if (any(abs(local) > vec3<f32>(0.5))) { continue; }
    let facing = dot(n, d.axis.xyz);
    let fade = smoothstep(d.axis.w, d.axis.w + fx.decal_info.z, facing) * (1.0 - smoothstep(0.35, 0.5, abs(local.z)));
    if (fade <= 0.0) { continue; }
    let t = vec2<f32>(local.x + 0.5, 0.5 - local.y);
    var texel = vec4<f32>(1.0);
    let gx = (d.inv * vec4<f32>(dpx, 0.0)).xy * vec2<f32>(1.0, -1.0);
    let gy = (d.inv * vec4<f32>(dpy, 0.0)).xy * vec2<f32>(1.0, -1.0);
    if (fx.decal_info.y > 0.5) {
      texel = decal_sample(d.uv_rect, t, gx, gy);
    }
    let c = d.color * texel;
    let a = clamp(c.a * fade, 0.0, 1.0);
    r.albedo = mix(r.albedo, c.rgb, a);
    if (d.emissive.w >= 0.0) { r.roughness = mix(r.roughness, d.emissive.w, a); }
    r.emissive += d.emissive.rgb * texel.rgb * a;
    if (d.normal.x != 0.0 && fx.decal_info.y > 0.5) {

      var m = decal_sample(d.normal_rect, t, gx, gy).xyz;
      if (fx.decal_info.y > 1.5) { m = decal_stored(m); }
      let ts = vec3<f32>((m.xy * 2.0 - 1.0) * d.normal.x, max(m.z * 2.0 - 1.0, 1e-3));
      let ux = vec3<f32>(d.inv[0].x, d.inv[1].x, d.inv[2].x);
      var tu = ux - n * dot(n, ux);
      if (dot(tu, tu) > 1e-12) {
        tu = normalize(tu);
        let tv = cross(n, tu);
        let bent = normalize(tu * ts.x + tv * ts.y + n * ts.z);



        let w = clamp(fade, 0.0, 1.0);
        r.normal = vec4<f32>(normalize(mix(r.normal.xyz, bent, w)), max(r.normal.w, w));
      }
    }
  }
  return r;
}
fn planar_reflection(p: vec3<f32>, shift: vec2<f32>) -> vec4<f32> { return vec4<f32>(0.0); }
fn screen_ao(p: vec3<f32>) -> f32 {
  if (fx.ao.x < 0.5 || oit_pass) { return 1.0; }
  let c = fx.view_proj_unjittered * vec4<f32>(p, 1.0);
  let ndc = c.xy / max(c.w, 1e-4);
  return textureSampleLevel(ssao_tex, fx_sampler, vec2<f32>(0.5 + 0.5 * ndc.x, 0.5 - 0.5 * ndc.y), 0.0).r;
}
// A local light's screen-space contact shadow at world point `p`
// (fxpost.wgsl `fs_contact`, its mask in the effect group's AO texture's
// g, b or a): a contact light's shadow slot carries a quarter, a half or
// three quarters past its layer (render.rs). 1 for every other light, in a
// reflection probe's face (no clusters: the mask is the camera's) and
// where the AO texture is the white placeholder (views, reflections), and
// for a blended surface in the order-independent passes (oit.wgsl
// `oit_pass`), which the opaque surface's mask behind it would darken.
fn contact_shadow(light: LocalLight, p: vec3<f32>) -> f32 {
  let w = light.direction_slot.w;
  let c = i32(round(fract(max(w, 0.0)) * 4.0)) - 1;
  if (c < 0 || frame.cluster.x <= 0.0 || oit_pass) { return 1.0; }
  let clip = fx.view_proj_unjittered * vec4<f32>(p, 1.0);
  let ndc = clip.xy / max(clip.w, 1e-4);
  let m = textureSampleLevel(ssao_tex, fx_sampler, vec2<f32>(0.5 + 0.5 * ndc.x, 0.5 - 0.5 * ndc.y), 0.0);
  return select(select(m.a, m.b, c == 1), m.g, c == 0);
}













override DISCARD: bool = true;







override ANALYTIC: bool = false;


var<private> analytic_uv: vec4<f32>;
var<private> analytic_dx: vec3<f32>;
var<private> analytic_dy: vec3<f32>;
var<private> analytic_local_dx: vec3<f32>;
var<private> analytic_local_dy: vec3<f32>;


fn analytic_grad(slot: u32, uv: vec2<f32>, d: vec2<f32>) -> vec2<f32> {
  return material_uv(slot, uv + d) - material_uv(slot, uv);
}


fn material_sample(t: texture_2d<f32>, slot: u32, uv: vec2<f32>) -> vec4<f32> {
  let at = material_uv(slot, uv);
  if (ANALYTIC) {
    return textureSampleGrad(t, material_sampler, at, analytic_grad(slot, uv, analytic_uv.xy), analytic_grad(slot, uv, analytic_uv.zw));
  }
  return textureSample(t, material_sampler, at);
}















fn toon_metal_bands(s: Surface, n: vec3<f32>, v: vec3<f32>, p: vec3<f32>) -> f32 {
  let w = vec3<f32>(0.2126, 0.7152, 0.0722);
  let flat_env = dot(reflected_light(vec3<f32>(0.0, 1.0, 0.0), p, frame.params.y), w);
  if (flat_env < 1e-3) { return 1.0; }
  let k = dot(reflected_light(reflect(-v, n), p, s.roughness * frame.params.y), w) / flat_env;
  return toon_ramp(0.45 * k * k) / max(toon_ramp(1.0), 1e-3);
}





fn toon_edges(n_dot_l: f32) {
  let px = frame.toon.x;
  if (ANALYTIC) {
    toon_aa = 0.01 * px;
  } else {
    toon_aa = clamp(fwidth(n_dot_l), 1e-4, 0.25) * px;
  }
}

fn world_dx(p: vec3<f32>) -> vec3<f32> {
  if (ANALYTIC) { return analytic_dx; }
  return dpdx(p);
}

fn world_dy(p: vec3<f32>) -> vec3<f32> {
  if (ANALYTIC) { return analytic_dy; }
  return dpdy(p);
}

@fragment
fn fs(in: VertexOut, @builtin(front_facing) front_facing: bool) -> @location(0) vec4<f32> {
  hook_front = select(-1.0, 1.0, front_facing);
  return lit_fragment(in);
}





@fragment
fn fs_oit(in: VertexOut, @builtin(front_facing) front_facing: bool) -> OitOutput {
  hook_front = select(-1.0, 1.0, front_facing);
  oit_pass = true;
  return oit_weighted(lit_fragment(in), in.clip.z);
}

@fragment
fn fs_oit_list(in: VertexOut, @builtin(front_facing) front_facing: bool, @builtin(sample_mask) mask: u32) -> OitOutput {
  hook_front = select(-1.0, 1.0, front_facing);
  oit_pass = true;
  let c = lit_fragment(in);
  oit_store(in.clip, c, mask, in.oit_key);
  return oit_weighted(c, in.clip.z);
}

@fragment
fn fs_oit_front(in: VertexOut, @builtin(front_facing) front_facing: bool) -> OitOutput {
  hook_front = select(-1.0, 1.0, front_facing);
  oit_pass = true;
  let c = lit_fragment(in);
  return oit_hybrid(c, in.clip);
}

@fragment
fn fs_oit_mark(in: VertexOut, @builtin(front_facing) front_facing: bool) {
  hook_front = select(-1.0, 1.0, front_facing);
  oit_pass = true;
  let c = lit_fragment(in);
  oit_front_mark(in.clip, c.a);
}


fn lit_fragment(in: VertexOut) -> vec4<f32> {
  let uv = stylised_uv(in.uv, in.affine);
  let base_uv = material_uv(UV_MAP, uv);
  var base_sample: vec4<f32>;
  if (ANALYTIC) {
    let gx = analytic_grad(UV_MAP, uv, analytic_uv.xy);
    let gy = analytic_grad(UV_MAP, uv, analytic_uv.zw);
    base_sample = textureSampleGrad(base_color_tex, material_sampler, base_uv, gx, gy);
  } else {
    base_sample = textureSample(base_color_tex, material_sampler, base_uv);
  }

  var hook: Hook;
  hook.world = in.world_pos;
  hook.local = in.local_pos;
  hook.normal = in.normal;
  hook.uv = uv;
  hook.color = in.color;
  hook.time = frame.time.x;
  hook.origin = frame.origin.xyz;
  hook.eye = frame.view_pos.xyz;
  hook.front = hook_front;
  let params = shader_params();
  let base = surface_color(hook, params, look_tint(material.base_color * base_sample * in.color, in.look));
  var dissolve = vec4<f32>(0.0);
  if (EFFECTS) { dissolve = dissolve_at(uv, in.local_pos); }


  let mode = u32(material.flags.w);

  shadow_closed = (mode & 32u) != 0u;


  let a2c = (mode & 16u) != 0u && material.emissive.w >= 0.0;
  let coverage = clamp((base.a - material.emissive.w) / max(fwidth(base.a), 1e-4) + 0.5, 0.0, 1.0);
  if (DISCARD && ((material.emissive.w >= 0.0 && select(base.a < material.emissive.w, coverage <= 0.0, a2c)) || dissolve.w > 0.5)) {
    discard;
  }
  let dissolve_edge = dissolve.rgb;

  let alpha = select(select(1.0, coverage, a2c), base.a, (mode & 4u) != 0u);
  if (frame.light_color.w > 6.5) { return debug_unsupported(in.clip.xy); }
  if ((mode & 2u) != 0u) {

    let e = surface_emissive(hook, params, material.emissive.rgb * material_sample(emissive_tex, UV_EMISSIVE, uv).rgb);
    var unlit = base.rgb + e + dissolve_edge;
    if (EFFECTS && material.toon.w > 0.0) { unlit = mix(unlit, material.flash.rgb, material.toon.w); }
    unlit = look_flash(unlit, in.look);
    if (frame.light_color.w > 1.5 && frame.light_color.w < 2.5) {

      let ssao = textureSampleLevel(ssao_tex, fx_sampler, in.clip.xy * fx.screen.zw, 0.0).r;
      return vec4<f32>(vec3<f32>(select(1.0, ssao, fx.ao.x > 0.5)), 1.0);
    }
    if (frame.light_color.w > 2.5 && frame.light_color.w < 3.5) { return vec4<f32>(base.rgb, alpha); }
    if (frame.light_color.w > 0.5) { return debug_unsupported(in.clip.xy); }
    return vec4<f32>(apply_fog(unlit, in.world_pos), alpha);
  }
  let metalness_sample = material_sample(metalness_tex, UV_METALNESS, uv);
  let roughness_sample = material_sample(roughness_tex, UV_METALNESS, uv);
  let dpx = world_dx(in.world_pos);
  let dpy = world_dy(in.world_pos);
  var s: Surface;
  s.metallic = clamp(material.mr.x * metalness_sample.b, 0.0, 1.0);
  s.roughness = clamp(material.mr.y * roughness_sample.g, 0.04, 1.0);
  s.albedo = base.rgb;

  var n = normalize(in.normal);
  if ((mode & 1u) != 0u) {

    n = normalize(cross(dpx, dpy));
  }

  let v = select(normalize(frame.view_pos.xyz - in.world_pos), -frame.view_forward.xyz, frame.view_pos.w > 0.5);
  let n_vertex = n;
  let face = select(1.0, -1.0, dot(n, v) < 0.0);
  n = n * face;
  var decal_emissive = vec3<f32>(0.0);
  var decal_normal = vec4<f32>(0.0);
  if (EFFECTS && abs(fx.decal_info.x) > 0.5) {
    let d = apply_decals(in.clip.xy, in.world_pos, n, s.albedo, s.roughness, dpx, dpy);
    s.albedo = d.albedo;
    s.roughness = clamp(d.roughness, 0.04, 1.0);
    decal_emissive = d.emissive;
    decal_normal = d.normal;
  }
  s.f0 = mix(vec3<f32>(0.04), s.albedo, s.metallic);
  let normal_uv = material_uv(UV_NORMAL, uv);
  var normal_sample: vec4<f32>;
  if (ANALYTIC) {
    let gx = analytic_grad(UV_NORMAL, uv, analytic_uv.xy);
    let gy = analytic_grad(UV_NORMAL, uv, analytic_uv.zw);
    normal_sample = textureSampleGrad(normal_tex, material_sampler, normal_uv, gx, gy);
  } else {
    normal_sample = textureSample(normal_tex, material_sampler, normal_uv);
  }
  if (material.flags.x > 0.5) {
    var map = normal_sample.xyz * 2.0 - 1.0;
    if (material.flags.x > 1.5) {

      map.z = sqrt(max(1.0 - dot(map.xy, map.xy), 0.0));
    }
    map = vec3<f32>(map.xy * material.mr.zw, map.z);


    var t = in.tangent.xyz - n_vertex * dot(n_vertex, in.tangent.xyz);
    if (dot(t, t) > 1e-12) {
      t = normalize(t);
      let b = cross(n_vertex, t) * select(1.0, -1.0, in.tangent.w < 0.0);

      n = normalize(mat3x3<f32>(t, b, n_vertex) * map) * face;
    }
  }
  n = surface_normal(hook, params, n);

  if (decal_normal.w > 0.0) { n = normalize(mix(n, decal_normal.xyz, decal_normal.w)); }




  let ms = frame.material_surface;
  var wrap = 0.0;
  var wrap_tint = vec3<f32>(1.0);
  if (ms.x > 0.5 && (mode & 8u) == 0u) {
    let kind = u32(ms.x + 0.5);
    if (kind != 2u) {



      var coarse_sample: vec4<f32>;
      if (ANALYTIC) {
        let gx = analytic_grad(UV_MAP, uv, analytic_uv.xy) * 64.0;
        let gy = analytic_grad(UV_MAP, uv, analytic_uv.zw) * 64.0;
        coarse_sample = textureSampleGrad(base_color_tex, material_sampler, base_uv, gx, gy);
      } else {
        coarse_sample = textureSampleBias(base_color_tex, material_sampler, base_uv, 6.0);
      }
      let coarse = look_tint(material.base_color * coarse_sample * in.color, in.look).rgb;
      s.albedo = mix(s.albedo, coarse, select(0.5, 0.7, kind == 3u) * ms.y);
    }



    var footprint: f32;
    if (ANALYTIC) {
      footprint = max(length(analytic_local_dx), length(analytic_local_dy));
    } else {
      footprint = max(length(dpdx(in.local_pos)), length(dpdy(in.local_pos)));
    }
    let fade = clamp((ms.z / max(footprint, 1e-6) - 3.0) / 6.0, 0.0, 1.0);
    let made = material_surface(kind, ms.y, in.local_pos / ms.z, s, fade);
    s = made.s;
    wrap = made.wrap;
    wrap_tint = made.tint;



    let hm = made.height * ms.z;
    let r1 = cross(dpy, n);
    let r2 = cross(n, dpx);
    let det = dot(dpx, r1);



    var dh: vec2<f32>;
    if (ANALYTIC) {
      let hx = material_surface(kind, ms.y, (in.local_pos + analytic_local_dx) / ms.z, s, fade).height * ms.z;
      let hy = material_surface(kind, ms.y, (in.local_pos + analytic_local_dy) / ms.z, s, fade).height * ms.z;
      dh = vec2<f32>(hx - hm, hy - hm);
    } else {
      dh = vec2<f32>(dpdx(hm), dpdy(hm));
    }
    let grad = sign(det) * (dh.x * r1 + dh.y * r2);





    let facing = abs(det) / max(length(cross(dpx, dpy)), 1e-20);
    let bumped = abs(det) * n - grad * smoothstep(0.02, 0.1, facing);
    n = select(n, normalize(bumped), dot(bumped, bumped) > 1e-30);
    s.f0 = mix(vec3<f32>(0.04), s.albedo, s.metallic);
  }

  let weathered = weather_surface(s, n, normalize(n_vertex * face), in.world_pos);
  s = weathered.s;
  n = weathered.n;
  s.coat = material.coat.x;
  s.coat_roughness = clamp(material.coat.y, 0.04, 1.0);
  s.coat_normal = n_vertex * face;
  s.translucency = material.coat.z;


  let l = normalize(frame.light_dir.xyz);
  let n_dot_l = max(dot(n, l), 0.0);
  let receives = look_receives(in.look);
  var shadow = 1.0;
  if (frame.light_dir.w > 0.5 && (n_dot_l > 0.0 || s.translucency > 0.0) && receives) {
    shadow = cascade_shadow(in.world_pos, leaf_shadow_normal(s, normalize(in.normal), l), abs(dot(n, l)));
  }
  if (EFFECTS && material.toon.x > 0.5) {
    toon_edges(dot(n, l));
    if (s.metallic > 0.0) {
      toon_metal_lit = toon_metal_bands(s, n, v, in.world_pos);
    }
  }
  var color: vec3<f32>;
  if (EFFECTS && material.toon.x > 0.5) {
    color = toon_light(s, n, v, l, shadow) * frame.light_color.rgb;
  } else {
    color = brdf(s, n, v, l) * frame.light_color.rgb * shadow;
  }
  if (wrap > 0.0) {




    let raw = dot(n, l);
    let wrapped = max(raw + wrap, 0.0) / ((1.0 + wrap) * (1.0 + wrap));
    let extra = wrapped - max(raw, 0.0);
    let gain = max(extra, 0.0) * wrap_tint + vec3<f32>(min(extra, 0.0));
    color += s.albedo / PI * (1.0 - s.metallic) * gain * frame.light_color.rgb * select(1.0, shadow, raw > 0.0);
  }

  color += s.albedo * frame.light_color.rgb * n_dot_l * shadow * caustics(in.world_pos);


  let keep = shadow_keep(n_dot_l * shadow, mode);
  color *= keep;






  if (frame.shadow_tint.w > 0.0 && n_dot_l > 0.0 && (mode & 8u) == 0u) {
    var tinted = (1.0 - shadow) * n_dot_l;
    var tint = frame.shadow_tint.rgb * frame.shadow_tint.w;
    if (EFFECTS && material.toon.x > 0.5) {
      tinted = toon_ramp(n_dot_l) - toon_ramp(n_dot_l * shadow);

      tint = min(tint, vec3<f32>(1.0 / PI));
    }
    color += tinted * tint * s.albedo * frame.light_color.rgb;
  }




  let lights = frame_lights(in.clip.xy, in.world_pos);
  for (var k = 0u; k < lights.y; k++) {
    color += local_light(frame_light(lights, k), s, in.world_pos, n, v, receives);
  }


  let n_dot_v = clamp(dot(n, v), 1e-4, 1.0);
  let f_ibl = f_schlick_roughness(n_dot_v, s.f0, s.roughness);
  let kd_ibl = (vec3<f32>(1.0) - f_ibl) * (1.0 - s.metallic);


  let irradiance = ambient_irradiance(ambient_light(select(n, ambient_normal(in.clip.xy, n), EFFECTS), in.world_pos), in.world_pos, n);
  var diffuse_ibl = (irradiance * frame.params.x + hemi_irradiance(n)) * s.albedo;
  let r = reflect(-v, n);
  let mip = s.roughness * frame.params.y;

  let prefiltered = gi_reflected(volume_reflected(reflected_light(r, in.world_pos, mip), in.world_pos, r, s.roughness), r, in.world_pos);
  let lut = textureSample(brdf_lut, ibl_sampler, vec2<f32>(n_dot_v, s.roughness)).rg;
  var specular_ibl = prefiltered * (f_ibl * lut.x + lut.y) * frame.params.x;
  if (EFFECTS && material.toon.x > 0.5 && frame.toon.y < 1.0) {




    let up = vec3<f32>(0.0, 1.0, 0.0);
    let flat_irradiance = 0.5 * (
      ambient_irradiance(ambient_light_level(up, in.world_pos), in.world_pos, up) + ambient_irradiance(ambient_light_level(-up, in.world_pos), in.world_pos, -up));
    let flat_hemi = 0.5 * (frame.hemi_sky.rgb + frame.hemi_ground.rgb) * frame.hemi_sky.w;
    let flat_diffuse = (flat_irradiance * frame.params.x + flat_hemi) * s.albedo;



    let flat_specular = mix(
      reflected_light(up, in.world_pos, frame.params.y) * s.f0 * frame.params.x,
      (flat_irradiance * frame.params.x + flat_hemi) * s.f0 * toon_metal_lit,
      s.metallic);
    diffuse_ibl = mix(flat_diffuse, diffuse_ibl, frame.toon.y);
    specular_ibl = mix(flat_specular, specular_ibl, frame.toon.y);
  }
  let ao_sample = material_sample(ao_tex, UV_AO, uv).r;


  var ibl_scale = material.flags.z * keep;
  if (material.flags.y >= 0.0) {

    ibl_scale *= mix(1.0, ao_sample, material.flags.y);
  }

  var ao_all = 1.0;


  var bounce = vec3<f32>(0.0);
  if (EFFECTS && fx.ao.x > 0.5 && !oit_pass) {


    let ao_texel = textureSampleLevel(ssao_tex, fx_sampler, in.clip.xy * fx.screen.zw, 0.0);
    let ssao_sample = ao_texel.r;
    if (fx.ao.w > 0.5 && fx.ao.w < 1.5) { bounce = ao_texel.gba * (ibl_scale / max(material.flags.z, 1e-4)); }
    ibl_scale *= mix(mix(1.0, ssao_sample, fx.ao.y), mix(0.35, 1.0, ssao_sample), fx.ao.z);
    ao_all = mix(1.0, mix(0.35, 1.0, ssao_sample), fx.ao.z);
    color *= ao_all;
  }
  var ibl = kd_ibl * diffuse_ibl + specular_ibl;



  if (material.coat.x > 0.0) {

    let c_dot_v = max(dot(s.coat_normal, v), 1e-4);
    let fc = f_schlick(c_dot_v, vec3<f32>(0.04)).x * s.coat;
    let rc = reflect(-v, s.coat_normal);
    let coat_env = reflected_light(rc, in.world_pos, s.coat_roughness * frame.params.y);
    let coat_lut = textureSampleLevel(brdf_lut, ibl_sampler, vec2<f32>(c_dot_v, s.coat_roughness), 0.0).rg;
    ibl = ibl * (1.0 - fc) + coat_env * (0.04 * coat_lut.x + coat_lut.y) * s.coat * frame.params.x;
  }
  let dbg_direct = color;
  color += ibl * ibl_scale + kd_ibl * s.albedo * bounce;

  color += surface_emissive(hook, params, material.emissive.rgb * material_sample(emissive_tex, UV_EMISSIVE, uv).rgb) * ao_all;
  if (EFFECTS && abs(fx.decal_info.x) > 0.5) { color += decal_emissive; }
  if (EFFECTS && material.rim.w > 0.0) { color += dissolve_edge; }
  if (EFFECTS && material.toon.y > 0.0) {

    color += material.rim.rgb * material.toon.y * pow(1.0 - n_dot_v, material.toon.z);
  }
  if (EFFECTS && material.toon.w > 0.0) {
    color = mix(color, material.flash.rgb, material.toon.w);
  }
  color = look_flash(color, in.look);

  if (frame.light_color.w > 2.5) {

    let m = u32(frame.light_color.w + 0.5);
    if (m == 3u) { return vec4<f32>(s.albedo, 1.0); }
    if (m == 4u) { return vec4<f32>(kd_ibl * (diffuse_ibl * ibl_scale + s.albedo * bounce), 1.0); }
    if (m == 5u) { return vec4<f32>(specular_ibl * ibl_scale, 1.0); }
    return vec4<f32>(dbg_direct, 1.0);
  }
  if (frame.light_color.w > 1.5) {

    let ssao = textureSampleLevel(ssao_tex, fx_sampler, in.clip.xy * fx.screen.zw, 0.0).r;
    return vec4<f32>(vec3<f32>(select(1.0, ssao, fx.ao.x > 0.5)), 1.0);
  }
  if (frame.light_color.w > 0.5) {

    return vec4<f32>(cascade_tint(cascade_index(in.world_pos)) * 4.0, 1.0);
  }
  return vec4<f32>(apply_fog(color, in.world_pos), alpha);
}




struct MadeSurface {
  s: Surface,
  height: f32,
  wrap: f32,
  tint: vec3<f32>,
};





fn material_surface(kind: u32, k: f32, p: vec3<f32>, surface: Surface, detail: f32) -> MadeSurface {
  let kd = k * detail;
  var o: MadeSurface;
  o.s = surface;
  o.tint = vec3<f32>(1.0);
  if (kind == 1u) {


    let lump = value_noise3(p, 11u) * 0.65 + value_noise3(p * 2.3 + 7.1, 12u) * 0.35;
    let dent = smoothstep(0.6, 0.85, value_noise3(p * 1.6 + 3.7, 13u));
    o.height = (lump * 0.45 - dent * 0.3) * kd;
    o.s.roughness = max(o.s.roughness, 0.75);
    o.s.metallic = 0.0;
    o.s.albedo = o.s.albedo * (1.0 + (lump - 0.5) * 0.25 * kd - dent * 0.12 * kd);
    o.wrap = 0.5 * k;
    o.tint = vec3<f32>(1.0, 0.72, 0.55);
  } else if (kind == 2u) {


    let mottle = value_noise3(p, 21u);
    let fibre = value_noise3(p * vec3<f32>(3.0, 0.5, 3.0) + 1.3, 22u);
    let g = mottle * 0.55 + fibre * 0.45;
    o.height = g * 0.2 * kd;
    o.s.roughness = 1.0;
    o.s.metallic = 0.0;
    o.s.albedo = o.s.albedo * (1.0 + (g - 0.5) * 0.6 * kd);
    o.wrap = 0.25 * k;
    o.tint = vec3<f32>(1.0, 0.96, 0.88);
  } else {


    o.height = value_noise3(p, 31u) * 0.04 * kd;
    o.s.roughness = mix(o.s.roughness, clamp(o.s.roughness, 0.3, 0.45), k);
    o.s.metallic = o.s.metallic * (1.0 - k);
    let luma = dot(o.s.albedo, vec3<f32>(0.2126, 0.7152, 0.0722));
    o.s.albedo = max(mix(vec3<f32>(luma), o.s.albedo, 1.0 + 0.35 * k), vec3<f32>(0.0));
    o.wrap = 0.3 * k;
    o.tint = vec3<f32>(1.0, 0.9, 0.82);
  }
  return o;
}


struct PrepassTargets {

  @location(0) gbuffer: vec4<f32>,

  @location(1) depth: vec4<f32>,


  @location(2) outline: vec4<f32>,
};

@fragment
fn fs_prepass(in: PrepassOut, @builtin(front_facing) front_facing: bool) -> PrepassTargets {
  hook_front = select(-1.0, 1.0, front_facing);
  return prepass_targets(in);
}



fn prepass_targets(in: PrepassOut) -> PrepassTargets {
  let base_uv = material_uv(UV_MAP, in.uv);
  var base_sample: vec4<f32>;
  if (ANALYTIC) {
    let gx = analytic_grad(UV_MAP, in.uv, analytic_uv.xy);
    let gy = analytic_grad(UV_MAP, in.uv, analytic_uv.zw);
    base_sample = textureSampleGrad(base_color_tex, material_sampler, base_uv, gx, gy);
  } else {
    base_sample = textureSample(base_color_tex, material_sampler, base_uv);
  }

  var hook: Hook;
  hook.world = in.world_pos;
  hook.local = in.local_pos;
  hook.normal = in.normal;
  hook.uv = in.uv;
  hook.color = in.color;
  hook.time = frame.time.x;
  hook.origin = frame.origin.xyz;
  hook.eye = frame.view_pos.xyz;
  hook.front = hook_front;
  let base = surface_color(hook, shader_params(), material.base_color * base_sample * in.color);
  let dissolved = dissolve_at(in.uv, in.local_pos).w > 0.5;
  if (DISCARD && ((material.emissive.w >= 0.0 && base.a < material.emissive.w) || dissolved)) {
    discard;
  }
  var n = normalize(in.normal);
  if ((u32(material.flags.w) & 1u) != 0u) {
    n = normalize(cross(world_dx(in.world_pos), world_dy(in.world_pos)));
  }
  let v = select(normalize(frame.view_pos.xyz - in.world_pos), -frame.view_forward.xyz, frame.view_pos.w > 0.5);
  if (dot(n, v) < 0.0) { n = -n; }
  let cur = in.cur_clip.xy / in.cur_clip.w;
  let prev = in.prev_clip.xy / max(in.prev_clip.w, 1e-6);
  let velocity = (cur - prev) * vec2<f32>(0.5, -0.5);
  var o: PrepassTargets;
  o.gbuffer = vec4<f32>(oct_encode(n), velocity);



  let unlit = (u32(material.flags.w) & 2u) != 0u;
  let metalness = material.mr.x * material_sample(metalness_tex, UV_METALNESS, in.uv).b;
  let roughness = material.mr.y * material_sample(roughness_tex, UV_METALNESS, in.uv).g;
  let reflectivity = select(reflectivity_bits(roughness, metalness), 0.0, unlit);



  let detail = u32(round(clamp(material.uv[3].z, 0.0, 1.0) * 127.0)) | (((in.object >> 8u) & 1u) << 7u);
  o.depth = vec4<f32>(dot(in.world_pos - frame.view_pos.xyz, frame.view_forward.xyz), reflectivity + 65536.0 * f32(detail), 0.0, 1.0);




  var width = material.outline.w;
  let unstyled = (u32(material.flags.w) & 8u) != 0u;
  if (frame.outline_style.x > 0.0 && frame.view_pos.w < 0.5 && width > 0.0 && !unstyled) {
    let depth = max(dot(in.world_pos - frame.view_pos.xyz, frame.view_forward.xyz), 0.01);



    width = clamp(width * frame.outline_style.x / depth, 1.0, max(min(31.0, width * 4.0), 1.0));
  }
  o.outline = outline_bytes(material.outline.rgb, width, in.object & 255u);
  return o;
}




fn outline_bytes(color: vec3<f32>, width: f32, object: u32) -> vec4<f32> {
  let c = vec3<u32>(round(clamp(color, vec3<f32>(0.0), vec3<f32>(1.0)) * vec3<f32>(31.0, 63.0, 31.0)));
  let packed = (c.r << 11u) | (c.g << 5u) | c.b;
  let w = u32(round(clamp(width, 0.0, 31.875) * 8.0));
  return vec4<f32>(f32(packed >> 8u), f32(packed & 255u), f32(w), f32(object)) / 255.0;
}


















struct LeanVertexIn {
  @location(0) position: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(3) color: vec4<f32>,
};




struct CompactVertexIn {
  @location(0) position: vec4<f32>,
  @location(1) normal: vec2<f32>,
  @location(3) color: vec4<f32>,
};


fn oct_decode(e: vec2<f32>) -> vec3<f32> {
  var v = vec3<f32>(e.x, e.y, 1.0 - abs(e.x) - abs(e.y));
  let t = max(-v.z, 0.0);
  v.x += select(t, -t, v.x >= 0.0);
  v.y += select(t, -t, v.y >= 0.0);
  return normalize(v);
}

struct LeanOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world_pos: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) color: vec4<f32>,

  @location(3) @interpolate(flat) look: vec4<f32>,
};

@vertex
fn vs_lean(v: LeanVertexIn, i: InstanceIn, k: InstanceFxIn) -> LeanOut {
  let p = lit_place(instance_matrix(i.m0, i.m1, i.m2, i.m3), v.position, v.normal, vec2<f32>(0.0), v.color);
  var o: LeanOut;
  o.clip = p.clip;
  o.world_pos = p.world;
  o.normal = p.normal;
  o.color = v.color;
  o.look = k.look;
  return o;
}

@vertex
fn vs_lean_compact(v: CompactVertexIn, i: InstanceIn, k: InstanceFxIn) -> LeanOut {
  let n = oct_decode(v.normal);
  let p = lit_place(instance_matrix(i.m0, i.m1, i.m2, i.m3), v.position.xyz, n, vec2<f32>(0.0), v.color);
  var o: LeanOut;
  o.clip = p.clip;
  o.world_pos = p.world;
  o.normal = p.normal;
  o.color = v.color;
  o.look = k.look;
  return o;
}

@fragment
fn fs_lean(in: LeanOut) -> @location(0) vec4<f32> {
  var full: VertexOut;
  full.clip = in.clip;
  full.world_pos = in.world_pos;
  full.normal = in.normal;
  full.uv = vec2<f32>(0.0);
  full.color = in.color;
  full.local_pos = vec3<f32>(0.0);
  full.tangent = vec4<f32>(0.0);
  full.affine = vec3<f32>(0.0);
  full.look = in.look;
  return lit_fragment(full);
}

struct LeanPrepassOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world_pos: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) cur_clip: vec4<f32>,
  @location(3) prev_clip: vec4<f32>,
  @location(4) @interpolate(flat) object: u32,
};

@vertex
fn vs_prepass_lean(@location(0) position: vec3<f32>, @location(1) normal: vec3<f32>, i: InstanceIn, prev: PrevIn) -> LeanPrepassOut {
  return prepass_lean(position, normal, i, prev);
}

fn prepass_lean(position: vec3<f32>, normal: vec3<f32>, i: InstanceIn, prev: PrevIn) -> LeanPrepassOut {
  let p = lit_place(instance_matrix(i.m0, i.m1, i.m2, i.m3), position, normal, vec2<f32>(0.0), vec4<f32>(1.0));
  var o: LeanPrepassOut;
  o.clip = p.clip;
  o.world_pos = p.world;
  o.normal = p.normal;
  o.cur_clip = fx.view_proj_unjittered * vec4<f32>(p.world, 1.0);
  o.prev_clip = mul_point(fx.prev_view_proj, instance_matrix(prev.p0, prev.p1, prev.p2, prev.p3) * vec4<f32>(position, 1.0));
  o.object = object_id(i.m0, i.m1, i.m2, i.m3);
  return o;
}

@vertex
fn vs_prepass_lean_compact(@location(0) position: vec4<f32>, @location(1) normal: vec2<f32>, i: InstanceIn, prev: PrevIn) -> LeanPrepassOut {
  return prepass_lean(position.xyz, oct_decode(normal), i, prev);
}

@fragment
fn fs_prepass_lean(in: LeanPrepassOut) -> PrepassTargets {
  var full: PrepassOut;
  full.clip = in.clip;
  full.world_pos = in.world_pos;
  full.normal = in.normal;
  full.uv = vec2<f32>(0.0);
  full.color = vec4<f32>(1.0);
  full.local_pos = vec3<f32>(0.0);
  full.cur_clip = in.cur_clip;
  full.prev_clip = in.prev_clip;
  full.object = in.object;
  return prepass_targets(full);
}






@group(3) @binding(4) var lightmap_pages: texture_2d_array<f32>;
@group(3) @binding(5) var lightmap_sampler: sampler;


struct LightmapIn {
  @location(9) lightmap: vec3<f32>,
};



fn lightmap_texel(l: vec3<f32>) -> vec4<f32> {
  let page = i32(l.z + 0.5);
  return vec4<f32>(textureSample(lightmap_pages, lightmap_sampler, l.xy, page).rgb, 1.0);
}









struct LightmappedOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world_pos: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
  @location(3) color: vec4<f32>,
  @location(4) local_pos: vec3<f32>,
  @location(5) tangent: vec4<f32>,
  @location(6) affine: vec3<f32>,
  @location(7) lightmap: vec3<f32>,
  @location(8) @interpolate(flat) look: vec4<f32>,
  @location(9) @interpolate(flat) oit_key: vec2<u32>,
};

@vertex
fn vs_lightmap(v: VertexIn, i: InstanceIn, k: InstanceFxIn, m: LightmapIn) -> LightmappedOut {
  let o = lit_vertex(instance_matrix(i.m0, i.m1, i.m2, i.m3), v);
  var l: LightmappedOut;
  l.clip = o.clip;
  l.world_pos = o.world_pos;
  l.normal = o.normal;
  l.uv = o.uv;
  l.color = o.color;
  l.local_pos = o.local_pos;
  l.tangent = o.tangent;
  l.affine = o.affine;
  l.lightmap = m.lightmap;
  l.look = k.look;
  l.oit_key = o.oit_key;
  return l;
}



@vertex
fn vs_lightmap_compact(v: CompactVertexIn, i: InstanceIn, k: InstanceFxIn, m: LightmapIn) -> LightmappedOut {
  let n = oct_decode(v.normal);
  let model = instance_matrix(i.m0, i.m1, i.m2, i.m3);
  let p = lit_place(model, v.position.xyz, n, vec2<f32>(0.0), v.color);
  var l: LightmappedOut;
  l.clip = p.clip;
  l.world_pos = p.world;
  l.normal = p.normal;
  l.uv = vec2<f32>(0.0);
  l.color = v.color;
  l.local_pos = vec3<f32>(0.0);
  l.tangent = vec4<f32>(0.0);
  l.affine = vec3<f32>(0.0);
  l.lightmap = m.lightmap;
  l.look = k.look;
  l.oit_key = oit_key(model, v.position.xyz);
  return l;
}


fn lightmapped_fragment(in: LightmappedOut) -> vec4<f32> {
  baked_lightmap = lightmap_texel(in.lightmap);
  var o: VertexOut;
  o.clip = in.clip;
  o.world_pos = in.world_pos;
  o.normal = in.normal;
  o.uv = in.uv;
  o.color = in.color;
  o.local_pos = in.local_pos;
  o.tangent = in.tangent;
  o.affine = in.affine;
  o.look = in.look;
  return lit_fragment(o);
}

@fragment
fn fs_lightmap(in: LightmappedOut, @builtin(front_facing) front_facing: bool) -> @location(0) vec4<f32> {
  hook_front = select(-1.0, 1.0, front_facing);
  return lightmapped_fragment(in);
}


@fragment
fn fs_lightmap_oit(in: LightmappedOut, @builtin(front_facing) front_facing: bool) -> OitOutput {
  hook_front = select(-1.0, 1.0, front_facing);
  oit_pass = true;
  return oit_weighted(lightmapped_fragment(in), in.clip.z);
}

@fragment
fn fs_lightmap_oit_list(in: LightmappedOut, @builtin(front_facing) front_facing: bool, @builtin(sample_mask) mask: u32) -> OitOutput {
  hook_front = select(-1.0, 1.0, front_facing);
  oit_pass = true;
  let c = lightmapped_fragment(in);
  oit_store(in.clip, c, mask, in.oit_key);
  return oit_weighted(c, in.clip.z);
}

@fragment
fn fs_lightmap_oit_front(in: LightmappedOut, @builtin(front_facing) front_facing: bool) -> OitOutput {
  hook_front = select(-1.0, 1.0, front_facing);
  oit_pass = true;
  let c = lightmapped_fragment(in);
  return oit_hybrid(c, in.clip);
}

@fragment
fn fs_lightmap_oit_mark(in: LightmappedOut, @builtin(front_facing) front_facing: bool) {
  hook_front = select(-1.0, 1.0, front_facing);
  oit_pass = true;
  let c = lightmapped_fragment(in);
  oit_front_mark(in.clip, c.a);
}










const WIND_TAU: f32 = 6.283185307;

const WIND_TIME_PERIOD: f32 = 3600.0;

const WIND_GUST_CELL: f32 = 14.0;
const WIND_GUST_PERIOD: i32 = 256;


const WIND_WAVELENGTH: f32 = 10.0;


struct WindField {
  dir: vec4<f32>,
  phase: vec4<f32>,
  offset: vec4<f32>,
  time: vec4<f32>,



  grid: vec4<f32>,
  grid_outer: vec4<f32>,
};


fn wind_hash(x: u32) -> u32 {
  let state = x * 747796405u + 2891336453u;
  let word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
  return (word >> 22u) ^ word;
}

fn wind_hash2(a: u32, b: u32) -> u32 {
  return wind_hash(a ^ wind_hash(b));
}


fn wind_unit(h: u32) -> f32 {
  return f32(h >> 8u) / 16777216.0;
}



fn wind_periodic(frequency: f32) -> f32 {
  let step = WIND_TAU / WIND_TIME_PERIOD;
  return round(frequency / step) * step;
}


fn wind_lattice(c: vec2<i32>) -> f32 {
  let m = WIND_GUST_PERIOD - 1;
  return wind_unit(wind_hash2(u32(c.x & m), u32(c.y & m) ^ 0x9e3779b9u));
}

fn wind_noise(p: vec2<f32>) -> f32 {
  let i = floor(p);
  let f = p - i;
  let u = f * f * (3.0 - 2.0 * f);
  let c = vec2<i32>(i);
  let a = wind_lattice(c);
  let b = wind_lattice(c + vec2<i32>(1, 0));
  let d = wind_lattice(c + vec2<i32>(0, 1));
  let e = wind_lattice(c + vec2<i32>(1, 1));
  return mix(mix(a, b, u.x), mix(d, e, u.x), u.y);
}




fn wind_gust(w: WindField, at: vec2<f32>, previous: bool) -> f32 {
  let offset = select(w.offset.xy, w.offset.zw, previous);
  let p = at / WIND_GUST_CELL - offset;
  return 0.65 * wind_noise(p) + 0.35 * wind_noise(p * 2.0 + vec2<f32>(37.0, 11.0));
}






fn wind_gust_share(w: WindField, at: vec2<f32>, previous: bool) -> f32 {
  let dir = w.dir.xy;
  let phase = select(w.phase.xy, w.phase.zw, previous);
  let x = WIND_TAU / WIND_WAVELENGTH * dot(at, dir);
  let wave = (sin(x - phase.x) + 0.5 * sin(2.3 * x + 1.7 - phase.y)) / 1.5;
  return clamp(0.5 * wave + 1.4 * (wind_gust(w, at, previous) - 0.5), -1.0, 1.0);
}



fn wind_lean_analytic(w: WindField, at: vec2<f32>, previous: bool) -> vec2<f32> {
  if (w.dir.z <= 0.0) { return vec2<f32>(0.0); }
  return w.dir.xy * w.dir.z * (1.0 + w.dir.w * wind_gust_share(w, at, previous));
}









fn wind_field_level(tex: texture_2d<f32>, corner: vec2<f32>, cell: f32, n: f32, rows: f32, rel: vec2<f32>) -> vec4<f32> {
  let q = (rel - corner) / cell - 0.5;
  let inside = min(min(q.x, q.y), min(n - 1.0 - q.x, n - 1.0 - q.y)) / (0.125 * n);
  let c = clamp(q, vec2<f32>(0.0), vec2<f32>(n - 1.001));
  let i = vec2<i32>(floor(c));
  let f = c - floor(c);
  let r = i32(rows);
  let a = textureLoad(tex, vec2<i32>(i.x, r + i.y), 0);
  let b = textureLoad(tex, vec2<i32>(i.x + 1, r + i.y), 0);
  let d = textureLoad(tex, vec2<i32>(i.x, r + i.y + 1), 0);
  let e = textureLoad(tex, vec2<i32>(i.x + 1, r + i.y + 1), 0);
  return mix(mix(a, b, f.x), mix(d, e, f.x), f.y) * clamp(inside, 0.0, 1.0);
}





fn wind_lean(tex: texture_2d<f32>, w: WindField, rel: vec2<f32>) -> vec4<f32> {
  let at = rel + w.time.zw;
  let analytic = vec4<f32>(wind_lean_analytic(w, at, false), wind_lean_analytic(w, at, true));
  if (w.grid.w < 0.5) { return analytic; }
  let n = w.grid_outer.w;
  let fine = wind_field_level(tex, w.grid.xy, w.grid.z, n, 0.0, rel);
  let coarse = wind_field_level(tex, w.grid_outer.xy, w.grid_outer.z, n, n, rel);


  let wf = clamp(wind_field_weight(w.grid.xy, w.grid.z, n, rel), 0.0, 1.0);
  let wc = clamp(wind_field_weight(w.grid_outer.xy, w.grid_outer.z, n, rel), 0.0, 1.0);
  let far = coarse + analytic * (1.0 - wc);
  return fine + far * (1.0 - wf);
}


fn wind_field_weight(corner: vec2<f32>, cell: f32, n: f32, rel: vec2<f32>) -> f32 {
  let q = (rel - corner) / cell - 0.5;
  return min(min(q.x, q.y), min(n - 1.0 - q.x, n - 1.0 - q.y)) / (0.125 * n);
}








fn vegetation_trample(tex: texture_2d<f32>, grid: vec4<f32>, root: vec3<f32>, tall: f32, recovery: f32) -> vec3<f32> {
  if (grid.w < 1.0) { return vec3<f32>(0.0); }
  let q = (root.xz - grid.xy) / grid.z - 0.5;
  let n = grid.w;
  if (any(q < vec2<f32>(0.0)) || any(q > vec2<f32>(n - 1.0))) { return vec3<f32>(0.0); }

  let i = vec2<i32>(floor(q));
  let f = q - floor(q);
  let j = min(i + vec2<i32>(1), vec2<i32>(i32(n) - 1));
  let s00 = textureLoad(tex, i, 0);
  let s10 = textureLoad(tex, vec2<i32>(j.x, i.y), 0);
  let s01 = textureLoad(tex, vec2<i32>(i.x, j.y), 0);
  let s11 = textureLoad(tex, j, 0);
  var push = vec2<f32>(0.0);
  var crush = 0.0;
  let w = array<f32, 4>((1.0 - f.x) * (1.0 - f.y), f.x * (1.0 - f.y), (1.0 - f.x) * f.y, f.x * f.y);
  let ss = array<vec4<f32>, 4>(s00, s10, s01, s11);
  for (var k = 0; k < 4; k++) {
    let c = ss[k];
    let amount = length(c.xy);
    if (amount <= 0.0 || root.y + tall < c.z) { continue; }

    let back = smoothstep(0.0, max(recovery, 1e-3), c.w);
    let a = amount * (1.0 - back) * w[k];
    push += c.xy / amount * a;
    crush += a;
  }
  return vec3<f32>(push, crush);
}







struct FoliageWind {
  dir_strength: vec4f,
  gust_top: vec4f,
  origin: vec4f,
  field: WindField,
};



fn foliage_wind_unpack(v: array<vec4f, 9>) -> FoliageWind {
  return FoliageWind(v[0], v[1], v[2], WindField(v[3], v[4], v[5], v[6], v[7], v[8]));
}



fn foliage_normal(n: vec3f, anchor: vec3f, center: vec3f, strength: f32) -> vec3f {
  if (strength <= 0.0) { return n; }
  let radial = anchor - center;
  var dome = vec3f(0.0, 1.0, 0.0);
  if (dot(radial, radial) > 1e-12) {
    dome = normalize(normalize(radial) + vec3f(0.0, 0.45, 0.0));
  }
  let blended = mix(n, dome, clamp(strength, 0.0, 1.0));
  if (dot(blended, blended) <= 1e-12) { return dome; }
  return normalize(blended);
}

fn foliage_rotate(p: vec3f, axis: vec3f, angle: f32) -> vec3f {
  let c = cos(angle);
  return p * c + cross(axis, p) * sin(angle) + axis * dot(axis, p) * (1.0 - c);
}







fn foliage_offset(p: vec3f, f0: vec4f, f1: vec4f, model: mat4x4f, wind: FoliageWind, lean_xz: vec2f) -> vec3f {
  let lean = length(lean_xz);
  if (lean <= 1e-6 || wind.gust_top.y <= 0.0) { return vec3f(0.0); }


  let a = model[0].xyz;
  let b = model[1].xyz;
  let c = model[2].xyz;
  let d = vec3f(lean_xz.x, 0.0, lean_xz.y) / lean;
  let inverse_d = vec3f(dot(cross(b, c), d), dot(cross(c, a), d), dot(cross(a, b), d)) / dot(a, cross(b, c));
  let horizontal = vec3f(inverse_d.x, 0.0, inverse_d.z);
  if (dot(horizontal, horizontal) <= 1e-12) { return vec3f(0.0); }
  let direction = normalize(horizontal);
  let axis = cross(vec3f(0.0, 1.0, 0.0), direction);
  let branch_world = (model * vec4f(f0.xyz, 1.0)).xyz;
  let branch_phase = wind.origin.w * dot(branch_world + wind.origin.xyz, d);
  let branch_wave = (sin(branch_phase - wind.gust_top.z) + 0.5 * sin(2.3 * branch_phase + 1.7 - wind.gust_top.w)) / 1.5;
  let leaf_phase = dot(f1.xyz, vec3f(1.7, 2.3, 1.1)) + branch_phase;

  let flutter = (sin(3.0 * wind.gust_top.z + leaf_phase) + 0.5 * sin(2.0 * wind.gust_top.w + leaf_phase)) / 1.5;
  let leaf_axis = normalize(direction + vec3f(0.0, 0.6, 0.0));
  var moved = f1.xyz + foliage_rotate(p - f1.xyz, leaf_axis, atan(0.35 * lean * f1.w) * flutter);
  moved = f0.xyz + foliage_rotate(moved - f0.xyz, axis, atan(lean * (1.0 - f0.w)) * branch_wave);
  let height = clamp(moved.y / wind.gust_top.y, 0.0, 1.0);
  moved = foliage_rotate(moved, axis, atan(lean * height * height));
  return (model * vec4f(moved - p, 0.0)).xyz;
}



fn foliage_seed(world_root: vec3f) -> u32 {
  var h = 2166136261u;
  for (var i = 0u; i < 3u; i++) {
    h = (h ^ bitcast<u32>(world_root[i])) * 16777619u;
  }
  return h ^ (h >> 16u);
}




fn foliage_sway(layer: f32) -> f32 {
  return max(select(-layer - 1.0, layer, layer >= 0.0), 0.0);
}















struct MeshletDraw {
  flags: vec4<u32>,
  wind: array<FoliageWind, 2>,
};


const SWAY_HEADER_ROWS: u32 = 2u;



fn meshlet_sway_row(region: u32, index: u32) -> u32 {
  return region - 1u + SWAY_HEADER_ROWS + index * 2u;
}









fn meshlet_sway(p: vec3<f32>, header: vec4<f32>, f0: vec4<f32>, f1: vec4<f32>,
                model: mat4x4<f32>, wind: FoliageWind, sway: f32, previous: bool) -> vec3<f32> {
  if (sway <= 0.0) { return vec3<f32>(0.0); }
  var w = wind;
  w.gust_top.y = header.w;
  let lean4 = wind_lean(meshlet_wind_tex, wind.field, model[3].xz);
  let lean = select(lean4.xy, lean4.zw, previous) * sway;
  let world = foliage_offset(p, f0, f1, model, w, lean);
  let a = model[0].xyz;
  let b = model[1].xyz;
  let c = model[2].xyz;
  let det = dot(a, cross(b, c));
  if (abs(det) <= 1e-20) { return vec3<f32>(0.0); }
  return vec3<f32>(dot(cross(b, c), world), dot(cross(c, a), world), dot(cross(a, b), world)) / det;
}









@group(3) @binding(8) var<uniform> meshlet_draw: MeshletDraw;

@group(3) @binding(18) var meshlet_wind_tex: texture_2d<f32>;

@group(3) @binding(9) var<storage, read> meshlet_instances: array<vec4<f32>>;

@group(3) @binding(10) var<storage, read> meshlet_items: array<vec2<u32>>;

@group(3) @binding(11) var<storage, read> meshlet_clusters: array<vec4<u32>>;
@group(3) @binding(12) var<storage, read> meshlet_indices: array<u32>;

@group(3) @binding(13) var<storage, read> meshlet_vertices: array<vec4<f32>>;


@group(3) @binding(14) var<storage, read> meshlet_previous: array<vec4<f32>>;

@group(3) @binding(17) var<storage, read> meshlet_foliage: array<vec4<f32>>;

struct MeshletVertex {
  ok: bool,
  v: VertexIn,
  model: mat4x4<f32>,
  look: vec4<f32>,

  slot: u32,

  region: u32,
  index: u32,

  rest: vec3<f32>,
};


fn meshlet_rows(region: u32, index: u32) -> array<vec4<f32>, 3> {
  let at = meshlet_sway_row(region, index);
  return array<vec4<f32>, 3>(meshlet_foliage[region - 1u], meshlet_foliage[at], meshlet_foliage[at + 1u]);
}

fn meshlet_transform(t0: vec4<f32>, t1: vec4<f32>, t2: vec4<f32>) -> mat4x4<f32> {
  return instance_matrix(t0.xyz, vec3<f32>(t0.w, t1.xy), vec3<f32>(t1.zw, t2.x), t2.yzw);
}


fn meshlet_level_color(level: u32) -> f32 {
  var c = vec3<f32>(0.9, 0.08, 0.08);
  if (level == 0u) { c = vec3<f32>(0.1, 0.85, 0.15); }
  else if (level == 1u) { c = vec3<f32>(0.95, 0.85, 0.1); }
  else if (level == 2u) { c = vec3<f32>(1.0, 0.45, 0.05); }
  let q = vec3<u32>(c * 255.0 + 0.5);
  return f32((q.x << 16u) | (q.y << 8u) | q.z);
}

fn meshlet_vertex(vi: u32, ii: u32) -> MeshletVertex {
  var r: MeshletVertex;
  let item = meshlet_items[ii];
  let c = meshlet_clusters[item.y * 3u];
  if (vi / 3u >= c.y) {
    r.ok = false;
    return r;
  }
  r.ok = true;
  let index = meshlet_indices[c.x + vi];
  let a = meshlet_vertices[index * 4u];
  let b = meshlet_vertices[index * 4u + 1u];
  r.v.position = a.xyz;
  r.v.normal = vec3<f32>(a.w, b.xy);
  r.v.uv = b.zw;
  r.v.color = meshlet_vertices[index * 4u + 2u];
  r.v.tangent = meshlet_vertices[index * 4u + 3u];
  let t0 = meshlet_instances[item.x * 4u];
  let t1 = meshlet_instances[item.x * 4u + 1u];
  let t2 = meshlet_instances[item.x * 4u + 2u];
  r.model = meshlet_transform(t0, t1, t2);
  r.slot = item.x;
  r.look = meshlet_instances[item.x * 4u + 3u];
  r.region = c.w;
  r.index = index;
  r.rest = r.v.position;
  if (c.w != 0u) {
    let f = meshlet_rows(c.w, index);
    r.v.position += meshlet_sway(r.rest, f[0], f[1], f[2], r.model, meshlet_draw.wind[0], foliage_sway(r.look.x), false);
  }
  if (meshlet_draw.flags.x != 0u) {
    r.look.y = 0.8;
    r.look.z = meshlet_level_color(c.z);
  }
  return r;
}

@vertex
fn vs_meshlet(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VertexOut {
  let p = meshlet_vertex(vi, ii);
  if (!p.ok) {
    var o: VertexOut;
    o.clip = vec4<f32>(0.0);
    return o;
  }
  var v = p.v;
  if (p.region != 0u) {


    let f = meshlet_rows(p.region, p.index);
    v.normal = foliage_normal(v.normal, f[2].xyz, f[0].xyz, material.coat.w);
  }
  var o = lit_vertex(p.model, v);
  o.look = p.look;
  return o;
}

@vertex
fn vs_prepass_meshlet(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> PrepassOut {
  let p = meshlet_vertex(vi, ii);
  if (!p.ok) {
    var o: PrepassOut;
    o.clip = vec4<f32>(0.0);
    return o;
  }

  let q = p.slot * 4u;
  let prev = meshlet_transform(meshlet_previous[q], meshlet_previous[q + 1u], meshlet_previous[q + 2u]);


  var before = p.rest;
  if (p.region != 0u) {
    let f = meshlet_rows(p.region, p.index);
    before += meshlet_sway(p.rest, f[0], f[1], f[2], prev, meshlet_draw.wind[1], foliage_sway(meshlet_previous[q + 3u].x), true);
  }
  return prepass_vertex_from(p.model, prev, p.v, before);
}








@group(3) @binding(15) var<uniform> meshlet_screen: vec4<u32>;
@group(3) @binding(16) var<storage, read> meshlet_attrs: array<vec4<u32>>;

struct MeshletResolveOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) @interpolate(flat) row: u32,
};

@vertex
fn vs_meshlet_resolve(@builtin(vertex_index) vi: u32, @builtin(instance_index) row: u32) -> MeshletResolveOut {
  var o: MeshletResolveOut;
  let p = vec2<f32>(f32((vi << 1u) & 2u), f32(vi & 2u));
  o.clip = vec4<f32>(p * 2.0 - 1.0, 0.0, 1.0);
  o.row = row;
  return o;
}


fn meshlet_oct(e: vec2<f32>) -> vec3<f32> {
  var n = vec3<f32>(e, 1.0 - abs(e.x) - abs(e.y));
  if (n.z < 0.0) {
    n = vec3<f32>((1.0 - abs(n.yx)) * select(vec2<f32>(-1.0), vec2<f32>(1.0), n.xy >= vec2<f32>(0.0)), n.z);
  }
  return normalize(n);
}

struct MeshletRecord {

  ok: bool,
  depth: f32,
  v: VertexOut,
  prev_world: vec3<f32>,
  object: u32,


  lit_uv: vec4<f32>,
};

fn meshlet_record(pos: vec4<f32>, row: u32) -> MeshletRecord {
  var r: MeshletRecord;


  let pixel = u32(pos.y) * meshlet_screen.x + u32(pos.x);
  let slot = meshlet_attrs[pixel / 4u][pixel % 4u];
  r.ok = false;
  r.depth = 1.0;
  if (slot == 0u) { return r; }
  let at = meshlet_screen.z / 4u + (slot - 1u) * meshlet_screen.w;
  let a = meshlet_attrs[at];
  let b = meshlet_attrs[at + 1u];
  let c = meshlet_attrs[at + 2u];
  if (c.z != row) { return r; }
  r.ok = true;
  r.depth = bitcast<f32>(a.w);
  r.v.clip = pos;
  r.v.world_pos = bitcast<vec3<f32>>(a.xyz);
  r.v.normal = meshlet_oct(unpack2x16snorm(b.x));
  r.v.uv = bitcast<vec2<f32>>(b.yz);
  r.v.color = unpack4x8unorm(b.w);

  r.v.local_pos = vec3<f32>(
    bitcast<f32>(meshlet_attrs[at + 4u].w),
    bitcast<f32>(meshlet_attrs[at + 6u].w),
    bitcast<f32>(meshlet_attrs[at + 7u].w),
  );
  r.v.tangent = vec4<f32>(meshlet_oct(unpack2x16snorm(c.x)), bitcast<f32>(c.y));
  if (r.v.tangent.w == 0.0) { r.v.tangent = vec4<f32>(0.0); }
  r.v.affine = vec3<f32>(r.v.uv, 1.0);
  r.v.look = bitcast<vec4<f32>>(meshlet_attrs[at + 3u]);
  r.prev_world = bitcast<vec3<f32>>(meshlet_attrs[at + 4u].xyz);

  analytic_uv = bitcast<vec4<f32>>(meshlet_attrs[at + 5u]);
  analytic_dx = bitcast<vec3<f32>>(meshlet_attrs[at + 6u].xyz);
  analytic_dy = bitcast<vec3<f32>>(meshlet_attrs[at + 7u].xyz);
  r.lit_uv = analytic_uv;
  if (meshlet_screen.w > 8u) {

    let lx = bitcast<vec4<f32>>(meshlet_attrs[at + 8u]);
    let ly = bitcast<vec4<f32>>(meshlet_attrs[at + 9u]);
    analytic_local_dx = lx.xyz;
    analytic_local_dy = ly.xyz;

    r.v.affine = vec3<f32>(lx.w, ly.w, 1.0);
    r.lit_uv = bitcast<vec4<f32>>(meshlet_attrs[at + 10u]);


    let d = meshlet_attrs[at + 11u];
    if (d.y != 0u && material.coat.w > 0.0) {
      r.v.normal = normalize(mix(normalize(r.v.normal), meshlet_oct(unpack2x16snorm(d.x)), clamp(material.coat.w, 0.0, 1.0)));
    }
  }
  r.object = c.w;
  return r;
}

struct MeshletResolveLit {
  @location(0) color: vec4<f32>,
  @builtin(frag_depth) depth: f32,
};

@fragment
fn fs_meshlet_resolve(in: MeshletResolveOut) -> MeshletResolveLit {
  let r = meshlet_record(in.clip, in.row);
  if (!r.ok) { discard; }
  var o: MeshletResolveLit;
  o.depth = r.depth;

  hook_front = 1.0;
  analytic_uv = r.lit_uv;
  o.color = lit_fragment(r.v);
  return o;
}

struct MeshletResolvePrepass {
  @location(0) gbuffer: vec4<f32>,
  @location(1) depth: vec4<f32>,
  @location(2) outline: vec4<f32>,
  @builtin(frag_depth) frag_depth: f32,
};

@fragment
fn fs_prepass_meshlet_resolve(in: MeshletResolveOut) -> MeshletResolvePrepass {
  let r = meshlet_record(in.clip, in.row);
  if (!r.ok) { discard; }
  var p: PrepassOut;
  p.clip = in.clip;
  p.world_pos = r.v.world_pos;
  p.normal = r.v.normal;
  p.uv = r.v.uv;
  p.color = r.v.color;
  p.local_pos = r.v.local_pos;
  p.cur_clip = fx.view_proj_unjittered * vec4<f32>(r.v.world_pos, 1.0);
  p.prev_clip = mul_point(fx.prev_view_proj, vec4<f32>(r.prev_world, 1.0));
  p.object = r.object;
  hook_front = 1.0;
  let t = prepass_targets(p);
  var o: MeshletResolvePrepass;
  o.gbuffer = t.gbuffer;
  o.depth = t.depth;
  o.outline = t.outline;
  o.frag_depth = r.depth;
  return o;
}
