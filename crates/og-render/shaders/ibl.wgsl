// IBL generation by render passes (no compute, so the WebGL2 tier can run it too):
// a procedural sky into a cubemap, its irradiance convolution, GGX-prefiltered
// specular mips, and the split-sum BRDF LUT. Each cube face is a fullscreen
// triangle whose fragment direction is derived from the face index.

struct Params {
  face: u32,
  roughness: f32,
  sample_count: u32,
  source_size: f32,
  sun_dir: vec4<f32>,
  sun_color: vec4<f32>,
  // A chosen look (`SkyLook::gpu`): zenith (w: 1 when set), horizon (w:
  // brightness), ground (w: stars), disc (outer, inner, intensity, glow),
  // glow power and sun colour. Unset: the built-in sky below.
  look: array<vec4<f32>, 5>,
  // A cooked environment map (fs_env_*): [0] (cos, sin) of its turn about
  // +y and the scale of what is copied; [1..8] its 27 irradiance harmonic
  // floats (coefficient k's channel c at float 3k + c).
  env: array<vec4<f32>, 8>,
};
@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var source_cube: texture_cube<f32>;
@group(0) @binding(2) var cube_sampler: sampler;

struct VsOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };

@vertex
fn vs(@builtin(vertex_index) i: u32) -> VsOut {
  var o: VsOut;
  let x = f32(i32(i & 1u) * 4 - 1);
  let y = f32(i32(i & 2u) * 2 - 1);
  o.pos = vec4<f32>(x, y, 0.0, 1.0);
  o.uv = vec2<f32>(x, y);
  return o;
}

fn face_dir(face: u32, uv: vec2<f32>) -> vec3<f32> {
  // uv is the fragment's NDC position in [-1,1] (y up; texture row 0 is NDC
  // y = +1). Faces in wgpu's order +x,-x,+y,-y,+z,-z with the WebGPU/Vulkan
  // cube convention (sc, tc) = per-face axes, tc = -uv.y. (The spike flipped
  // v a second time, which mirrored the sky vertically; it never drew it.)
  let u = uv.x;
  let v = uv.y;
  switch face {
    case 0u: { return normalize(vec3<f32>(1.0, v, -u)); }
    case 1u: { return normalize(vec3<f32>(-1.0, v, u)); }
    case 2u: { return normalize(vec3<f32>(u, 1.0, -v)); }
    case 3u: { return normalize(vec3<f32>(u, -1.0, v)); }
    case 4u: { return normalize(vec3<f32>(u, v, 1.0)); }
    default: { return normalize(vec3<f32>(-u, v, -1.0)); }
  }
}

const PI: f32 = 3.14159265359;

// The sky of a chosen look (frame.wgsl's sky_look without the stars, which
// would only add noise to the lighting).
fn looked_sky(d: vec3<f32>) -> vec3<f32> {
  let sun = normalize(params.sun_dir.xyz);
  let up = clamp(d.y, -1.0, 1.0);
  var sky = mix(params.look[1].rgb, params.look[0].rgb, pow(max(up, 0.0), 0.55));
  sky = mix(params.look[2].rgb, sky, smoothstep(-0.08, 0.02, up));
  let cos_sun = dot(d, sun);
  let glow = pow(max(cos_sun, 0.0), max(params.look[4].x, 1.0)) * params.look[3].w;
  let disc = smoothstep(params.look[3].x, params.look[3].y, cos_sun) * params.look[3].z;
  return sky * params.look[1].w + params.look[4].yzw * (glow + disc);
}

// A simple analytic sky: horizon haze, zenith blue, a sun disc with a glow, dark ground.
@fragment
fn fs_sky(in: VsOut) -> @location(0) vec4<f32> {
  let d = face_dir(params.face, in.uv);
  if (params.look[0].w > 0.5) { return vec4<f32>(looked_sky(d), 1.0); }
  let sun = normalize(params.sun_dir.xyz);
  let up = clamp(d.y, -1.0, 1.0);
  let zenith = vec3<f32>(0.18, 0.36, 0.78);
  let horizon = vec3<f32>(0.78, 0.84, 0.92);
  let ground = vec3<f32>(0.16, 0.14, 0.12);
  var sky = mix(horizon, zenith, pow(max(up, 0.0), 0.55));
  sky = mix(ground, sky, smoothstep(-0.08, 0.02, up));
  let cos_sun = dot(d, sun);
  let glow = pow(max(cos_sun, 0.0), 24.0) * 0.6;
  let disc = smoothstep(0.9993, 0.9997, cos_sun) * 40.0;
  let color = sky * 1.1 + params.sun_color.rgb * (glow + disc);
  return vec4<f32>(color, 1.0);
}

fn tangent_frame(n: vec3<f32>) -> mat3x3<f32> {
  var up = vec3<f32>(0.0, 1.0, 0.0);
  if (abs(n.y) > 0.999) { up = vec3<f32>(1.0, 0.0, 0.0); }
  let t = normalize(cross(up, n));
  let b = cross(n, t);
  return mat3x3<f32>(t, b, n);
}

@fragment
fn fs_irradiance(in: VsOut) -> @location(0) vec4<f32> {
  let n = face_dir(params.face, in.uv);
  let frame = tangent_frame(n);
  var sum = vec3<f32>(0.0);
  var count = 0.0;
  let step_phi = 2.0 * PI / 64.0;
  let step_theta = 0.5 * PI / 16.0;
  for (var p = 0.0; p < 2.0 * PI; p += step_phi) {
    for (var t = 0.0; t < 0.5 * PI; t += step_theta) {
      let tangent = vec3<f32>(sin(t) * cos(p), sin(t) * sin(p), cos(t));
      let dir = frame * tangent;
      // Sample a blurred mip so the sun disc does not alias into the convolution.
      sum += textureSampleLevel(source_cube, cube_sampler, dir, 3.0).rgb * cos(t) * sin(t);
      count += 1.0;
    }
  }
  return vec4<f32>(PI * sum / count, 1.0);
}

fn radical_inverse(bits_in: u32) -> f32 {
  var bits = bits_in;
  bits = (bits << 16u) | (bits >> 16u);
  bits = ((bits & 0x55555555u) << 1u) | ((bits & 0xAAAAAAAAu) >> 1u);
  bits = ((bits & 0x33333333u) << 2u) | ((bits & 0xCCCCCCCCu) >> 2u);
  bits = ((bits & 0x0F0F0F0Fu) << 4u) | ((bits & 0xF0F0F0F0u) >> 4u);
  bits = ((bits & 0x00FF00FFu) << 8u) | ((bits & 0xFF00FF00u) >> 8u);
  return f32(bits) * 2.3283064365386963e-10;
}

fn hammersley(i: u32, n: u32) -> vec2<f32> {
  return vec2<f32>(f32(i) / f32(n), radical_inverse(i));
}

fn importance_sample_ggx(xi: vec2<f32>, n: vec3<f32>, roughness: f32) -> vec3<f32> {
  let a = roughness * roughness;
  let phi = 2.0 * PI * xi.x;
  let cos_theta = sqrt((1.0 - xi.y) / (1.0 + (a * a - 1.0) * xi.y));
  let sin_theta = sqrt(1.0 - cos_theta * cos_theta);
  let h = vec3<f32>(cos(phi) * sin_theta, sin(phi) * sin_theta, cos_theta);
  return normalize(tangent_frame(n) * h);
}

fn d_ggx(n_dot_h: f32, roughness: f32) -> f32 {
  let a = roughness * roughness;
  let a2 = a * a;
  let d = n_dot_h * n_dot_h * (a2 - 1.0) + 1.0;
  return a2 / (PI * d * d + 1e-7);
}

@fragment
fn fs_prefilter(in: VsOut) -> @location(0) vec4<f32> {
  let n = face_dir(params.face, in.uv);
  let v = n;
  var sum = vec3<f32>(0.0);
  var weight = 0.0;
  let count = params.sample_count;
  for (var i = 0u; i < count; i++) {
    let xi = hammersley(i, count);
    let h = importance_sample_ggx(xi, n, params.roughness);
    let l = normalize(2.0 * dot(v, h) * h - v);
    let n_dot_l = max(dot(n, l), 0.0);
    if (n_dot_l > 0.0) {
      // Filtered importance sampling: pick the source mip from the sample's solid angle.
      let n_dot_h = max(dot(n, h), 0.0);
      let d = d_ggx(n_dot_h, params.roughness);
      let pdf = d * n_dot_h / (4.0 * max(dot(h, v), 1e-4)) + 1e-4;
      let sa_texel = 4.0 * PI / (6.0 * params.source_size * params.source_size);
      let sa_sample = 1.0 / (f32(count) * pdf + 1e-4);
      var mip = 0.0;
      if (params.roughness > 0.0) { mip = 0.5 * log2(sa_sample / sa_texel) + 1.0; }
      sum += textureSampleLevel(source_cube, cube_sampler, l, max(mip, 0.0)).rgb * n_dot_l;
      weight += n_dot_l;
    }
  }
  return vec4<f32>(sum / max(weight, 1e-4), 1.0);
}

fn g_smith_ibl(n_dot_v: f32, n_dot_l: f32, roughness: f32) -> f32 {
  let k = (roughness * roughness) / 2.0;
  let gv = n_dot_v / (n_dot_v * (1.0 - k) + k);
  let gl = n_dot_l / (n_dot_l * (1.0 - k) + k);
  return gv * gl;
}

@fragment
fn fs_brdf_lut(in: VsOut) -> @location(0) vec4<f32> {
  let n_dot_v = max(in.uv.x * 0.5 + 0.5, 1e-3);
  let roughness = clamp(0.5 - in.uv.y * 0.5, 0.02, 1.0);
  let v = vec3<f32>(sqrt(1.0 - n_dot_v * n_dot_v), 0.0, n_dot_v);
  let n = vec3<f32>(0.0, 0.0, 1.0);
  var a = 0.0;
  var b = 0.0;
  let count = 256u;
  for (var i = 0u; i < count; i++) {
    let xi = hammersley(i, count);
    let h = importance_sample_ggx(xi, n, roughness);
    let l = normalize(2.0 * dot(v, h) * h - v);
    let n_dot_l = max(l.z, 0.0);
    let n_dot_h = max(h.z, 0.0);
    let v_dot_h = max(dot(v, h), 0.0);
    if (n_dot_l > 0.0) {
      let g = g_smith_ibl(n_dot_v, n_dot_l, roughness);
      let g_vis = g * v_dot_h / (n_dot_h * n_dot_v + 1e-5);
      let fc = pow(1.0 - v_dot_h, 5.0);
      a += (1.0 - fc) * g_vis;
      b += fc * g_vis;
    }
  }
  return vec4<f32>(a / f32(count), b / f32(count), 0.0, 1.0);
}

// A reflection probe's radiance (probes.rs) from its capture, whose faces
// were drawn by right-handed cameras and so hold each face mirrored across
// its horizontal axis: the texel of `d` is read where the capture put it.
@fragment
fn fs_probe_copy(in: VsOut) -> @location(0) vec4<f32> {
  let d = face_dir(params.face, in.uv);
  let x_major = abs(d.x) >= max(abs(d.y), abs(d.z));
  let m = select(vec3<f32>(-d.x, d.y, d.z), vec3<f32>(d.x, d.y, -d.z), x_major);
  return vec4<f32>(textureSampleLevel(source_cube, cube_sampler, m, 0.0).rgb, 1.0);
}

// Downsample one cube face mip from the mip above (box filter via linear sampling).
@fragment
fn fs_downsample(in: VsOut) -> @location(0) vec4<f32> {
  let d = face_dir(params.face, in.uv);
  return textureSampleLevel(source_cube, cube_sampler, d, params.roughness);
}

// A cooked environment map turned about +y: the map's direction for world
// direction `d` (what was at +x is seen at -z after a quarter turn).
fn env_turned(d: vec3<f32>) -> vec3<f32> {
  let c = params.env[0].x;
  let s = params.env[0].y;
  return vec3<f32>(c * d.x - s * d.z, d.y, s * d.x + c * d.z);
}

// One level of a cooked environment map (level `roughness`), turned and
// scaled: the sky cube from level 0, each prefiltered mip from its level.
@fragment
fn fs_env_copy(in: VsOut) -> @location(0) vec4<f32> {
  let d = env_turned(face_dir(params.face, in.uv));
  let c = textureSampleLevel(source_cube, cube_sampler, d, params.roughness).rgb;
  return vec4<f32>(c * params.env[0].z, 1.0);
}

fn env_float(k: u32) -> f32 {
  let v = params.env[1u + k / 4u];
  switch k % 4u {
    case 0u: { return v.x; }
    case 1u: { return v.y; }
    case 2u: { return v.z; }
    default: { return v.w; }
  }
}

fn env_sh(k: u32) -> vec3<f32> {
  return vec3<f32>(env_float(3u * k), env_float(3u * k + 1u), env_float(3u * k + 2u));
}

// The irradiance over pi of a cooked environment map: its order-2
// harmonics (already convolved with the cosine lobe) at the turned direction.
@fragment
fn fs_env_irradiance(in: VsOut) -> @location(0) vec4<f32> {
  let d = env_turned(face_dir(params.face, in.uv));
  var e = env_sh(0u) * 0.282095;
  e += env_sh(1u) * (0.488603 * d.y);
  e += env_sh(2u) * (0.488603 * d.z);
  e += env_sh(3u) * (0.488603 * d.x);
  e += env_sh(4u) * (1.092548 * d.x * d.y);
  e += env_sh(5u) * (1.092548 * d.y * d.z);
  e += env_sh(6u) * (0.315392 * (3.0 * d.z * d.z - 1.0));
  e += env_sh(7u) * (1.092548 * d.x * d.z);
  e += env_sh(8u) * (0.546274 * (d.x * d.x - d.y * d.y));
  return vec4<f32>(max(e, vec3<f32>(0.0)), 1.0);
}
