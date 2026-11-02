// Screen-space global illumination (ssgi.rs), after fxpost.wgsl and
// ssr.wgsl: in the AO pass's place, each pixel casts gi1.x rays over the
// cosine-weighted hemisphere of its normal, marches each through the
// prepass's view depth and, where one passes just behind a surface facing
// it, reads the light that surface sent last frame from tex_a (TAA's
// history, or the GI's copy of the lit image) where the prepass velocity
// says the hit pixel was. Writes r: the ambient occlusion (the scene's AO
// term, or the share of rays that hit nothing), gba: the rays' average
// light (the irradiance over pi a diffuse surface receives), which
// fs_gi_blur and fs_gi_upsample then filter as the AO's blur does.

const GI_GOLDEN: f32 = 0.61803398875;
const GI_PLASTIC: f32 = 0.75487766625;

// A direction about view-space normal `n` with density cos(theta) / pi,
// for the uniform pair `u`.
fn gi_cosine_dir(n: vec3<f32>, u: vec2<f32>) -> vec3<f32> {
  let r = sqrt(u.x);
  let phi = 2.0 * FX_PI * u.y;
  let up = select(vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(0.0, 1.0, 0.0), abs(n.x) > 0.9);
  let t = normalize(cross(up, n));
  let b = cross(n, t);
  return normalize(t * (r * cos(phi)) + b * (r * sin(phi)) + n * sqrt(max(1.0 - u.x, 0.0)));
}

fn gi_luma(c: vec3<f32>) -> f32 {
  return dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
}

// The light one ray carries back: last frame's lit image at the surface it
// hits (black when it hits nothing the screen shows), and whether it hit.
// `origin` and `dir` in view space.
fn gi_ray(origin: vec3<f32>, dir: vec3<f32>, jitter: f32) -> vec4<f32> {
  let steps = max(u32(fxp.gi0.w), 1u);
  let max_distance = fxp.gi0.x;
  let thickness = fxp.gi0.y;
  var t_prev = 0.0;
  for (var i = 0u; i < steps; i++) {
    // Fine steps near the surface (the contact), coarse ones far away.
    let k = (f32(i) + jitter) / f32(steps);
    let t = k * k * max_distance;
    let q = origin + dir * t;
    if (q.z > -fxp.cam.z) { break; }
    let s = ssr_uv(q);
    if (any(s < vec2<f32>(0.0)) || any(s > vec2<f32>(1.0))) { break; }
    let scene = depth_uv(s);
    let behind = -q.z - scene;
    if (behind > 0.0) {
      if (behind > thickness + (t - t_prev) || is_sky(scene)) { break; }
      let hit_px = vec2<i32>(s * fxp.screen.xy);
      // Only the side of a surface that faces the ray sends it light.
      let hit_n = normalize((fxp.view * vec4<f32>(world_normal(hit_px), 0.0)).xyz);
      if (dot(hit_n, dir) > 0.1) { break; }
      let fade = 1.0 - smoothstep(0.6, 1.0, t / max_distance);
      if (fxp.gi1.z < 0.5) { return vec4<f32>(0.0, 0.0, 0.0, fade); }
      // Where the hit pixel was on the previous frame (gbuffer zw: its
      // screen-space velocity, uv per frame).
      let gsize = vec2<i32>(textureDimensions(tex_gbuffer));
      let velocity = textureLoad(tex_gbuffer, prepass_px(hit_px, gsize), 0).zw;
      let prev = s - velocity;
      if (any(prev < vec2<f32>(0.0)) || any(prev > vec2<f32>(1.0))) {
        return vec4<f32>(0.0, 0.0, 0.0, fade);
      }
      var light = max(textureSampleLevel(tex_a, fx_linear, prev, 0.0).rgb, vec3<f32>(0.0));
      // Fireflies: a few rays meeting a tiny bright emitter are held to a
      // level the blur can spread.
      let l = gi_luma(light);
      light *= min(1.0, 32.0 / max(l, 1e-4));
      let edge = min(min(prev.x, 1.0 - prev.x), min(prev.y, 1.0 - prev.y));
      return vec4<f32>(light * fade * smoothstep(0.0, 0.05, edge), fade);
    }
    t_prev = t;
  }
  return vec4<f32>(0.0);
}

@fragment
fn fs_ssgi(in: VsOut) -> @location(0) vec4<f32> {
  let px = ao_pixel(in.pos.xy);
  let d = depth_at(px);
  // The AO debug views (DebugView::AoDepth) as the AO pass writes them.
  if (fxp.cam.w > 0.5 && fxp.cam.w < 1.5) { return vec4<f32>(fract(d), 0.0, 0.0, 0.0); }
  if (is_sky(d)) { return vec4<f32>(1.0, 0.0, 0.0, 0.0); }
  let uv = ao_uv(px, in.uv);
  let p = view_pos(uv, d);
  let n = normalize((fxp.view * vec4<f32>(world_normal(px), 0.0)).xyz);
  let rays = max(u32(fxp.gi1.x), 1u);
  // The per-pixel noise, moved every frame under TAA (gi1.y), which then
  // averages the frames' different rays.
  let pos = in.pos.xy + vec2<f32>(fxp.gi1.y);
  let n0 = ign(pos);
  let n1 = ign(pos.yx + vec2<f32>(19.0, 47.0));
  let origin = p + n * (0.01 + 0.002 * d);
  var light = vec3<f32>(0.0);
  var hits = 0.0;
  for (var i = 0u; i < rays; i++) {
    let u = fract(vec2<f32>(n0 + f32(i) * GI_GOLDEN, n1 + f32(i) * GI_PLASTIC));
    let dir = gi_cosine_dir(n, vec2<f32>(min(u.x, 0.999), u.y));
    let r = gi_ray(origin, dir, fract(n0 * 7.13 + f32(i) * 0.37));
    light += r.rgb;
    hits += r.a;
  }
  light *= fxp.gi0.z / f32(rays);
  var ao = 1.0 - fxp.gi2.x * hits / f32(rays);
  if (fxp.gi1.w > 1.5) {
    ao = ssao_term(in);
  } else if (fxp.gi1.w > 0.5) {
    ao = gtao_term(in);
  }
  return vec4<f32>(ao, light);
}

// The 4x4 blur of fs_ao_blur over the GI's four channels, weighed by depth
// and by how far the normals turn (the bounce changes across a crease).
@fragment
fn fs_gi_blur(in: VsOut) -> @location(0) vec4<f32> {
  let px = vec2<i32>(in.pos.xy);
  if (fxp.cam.w > 0.5) { return textureLoad(tex_b, px, 0); }
  let k = ao_divisor();
  let d0 = depth_at(px * k);
  if (ao_blur_sky(d0)) { return vec4<f32>(1.0, 0.0, 0.0, 0.0); }
  let n0 = world_normal(px * k);
  let size = vec2<i32>(textureDimensions(tex_b));
  let scale = max(d0 * 0.04, 1e-3);
  var sum = vec4<f32>(0.0);
  var weight = 0.0;
  for (var y = -2; y <= 1; y++) {
    for (var x = -2; x <= 1; x++) {
      let q = clamp(px + vec2<i32>(x, y), vec2<i32>(0), size - 1);
      let facing = max(dot(world_normal(q * k), n0), 0.0);
      let w = exp(-abs(depth_at(q * k) - d0) / scale) * facing * facing * facing * facing + 1e-6;
      sum += textureLoad(tex_b, q, 0) * w;
      weight += w;
    }
  }
  return sum / weight;
}

// fs_ao_upsample over the GI's four channels.
@fragment
fn fs_gi_upsample(in: VsOut) -> @location(0) vec4<f32> {
  let px = vec2<i32>(in.pos.xy);
  let d0 = depth_at(px);
  if (ao_blur_sky(d0)) { return vec4<f32>(1.0, 0.0, 0.0, 0.0); }
  let size = vec2<i32>(textureDimensions(tex_b));
  let base = px / 2;
  let f = vec2<f32>(px - base * 2) * 0.5;
  let scale = max(d0 * 0.04, 1e-3);
  var sum = vec4<f32>(0.0);
  var weight = 0.0;
  var nearest = vec4<f32>(1.0, 0.0, 0.0, 0.0);
  var nearest_dz = 3.4e38;
  for (var y = 0; y <= 1; y++) {
    for (var x = 0; x <= 1; x++) {
      let q = min(base + vec2<i32>(x, y), size - 1);
      let a = textureLoad(tex_b, q, 0);
      let dz = abs(depth_at(q * 2) - d0);
      let bilinear = select(1.0 - f.x, f.x, x == 1) * select(1.0 - f.y, f.y, y == 1);
      let w = bilinear * exp(-dz / scale);
      sum += a * w;
      weight += w;
      if (dz < nearest_dz) {
        nearest_dz = dz;
        nearest = a;
      }
    }
  }
  return select(nearest, sum / max(weight, 1e-5), weight > 1e-3);
}
