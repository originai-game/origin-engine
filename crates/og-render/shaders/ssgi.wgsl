// Screen-space global illumination (ssgi.rs), after fxpost.wgsl: in the
// AO pass's place, each pixel casts gi1.x rays over the
// cosine-weighted hemisphere of its normal, marches each through the
// prepass's view depth and, where one passes just behind a surface facing
// it, reads the light that surface sent last frame from tex_a (TAA's
// history, or the GI's copy of the lit image) where the prepass velocity
// says the hit pixel was. Writes r: the ambient occlusion (the scene's AO
// term, or the share of rays that hit nothing), gba: the rays' average
// light (the irradiance over pi a diffuse surface receives), which
// fs_gi_blur and fs_gi_upsample then filter as the AO's blur does.

// With the probe GI on as well (`method: both`), the rays reach the near
// field, one probe spacing, and their hits replace the probes' light from
// the directions they hit and no more: the pass gathers the probes once
// (gi_sample.wgsl) and takes from their harmonics the light along the
// hits' directions, which a lit pass's ambient occlusion alone would take
// as an even share of every direction's. Rays whose light the screen
// cannot give leave it to the probes.

// The light volumes' buffer with the probe GI's probes (the frame group's
// binding 10, gi_probes.wgsl).
@group(0) @binding(7) var<storage, read> probe_data: ProbeWords;

const GI_GOLDEN: f32 = 0.61803398875;

// A view-space point's screen uv.
fn gi_uv(q: vec3<f32>) -> vec2<f32> {
  let c = fxp.proj * vec4<f32>(q, 1.0);
  let ndc = c.xy / c.w;
  return vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
}
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

// How far along `dir` from `origin` (view space) a ray stays on screen and
// in front of the near plane, at most `t_max`: the projected ray is linear
// in t before the divide, so each of the frustum's side planes and the
// near plane cuts it at one t.
fn gi_on_screen(origin: vec3<f32>, dir: vec3<f32>, t_max: f32) -> f32 {
  let c0 = fxp.proj * vec4<f32>(origin, 1.0);
  let cd = fxp.proj * vec4<f32>(dir, 0.0);
  var t = t_max;
  // x <= w, -x <= w, y <= w, -y <= w: a + t b <= 0.
  let a = vec4<f32>(c0.x - c0.w, -c0.x - c0.w, c0.y - c0.w, -c0.y - c0.w);
  let b = vec4<f32>(cd.x - cd.w, -cd.x - cd.w, cd.y - cd.w, -cd.y - cd.w);
  for (var i = 0; i < 4; i++) {
    if (b[i] > 0.0) { t = min(t, -a[i] / b[i]); }
  }
  if (dir.z > 0.0) { t = min(t, (-fxp.cam.z - origin.z) / dir.z); }
  return max(t, 0.0);
}

// The light one ray carries back: last frame's lit image at the surface it
// hits (black when it hits nothing the screen shows), and whether it hit.
// `origin` and `dir` in view space, `max_distance` the farthest it goes.
//
// The ray is cut where it leaves the screen and its steps are spread over
// what is left in screen space (perspective-correct: t/w and 1/w are
// linear there), finer near the pixel for the contact: so the step budget
// reaches across the screen, not a fixed world distance whose far steps
// fall off screen or whose near ones never reach the far wall.
fn gi_ray(origin: vec3<f32>, dir: vec3<f32>, jitter: f32, probes: bool, max_distance: f32) -> vec4<f32> {
  let steps = max(u32(fxp.gi0.w), 1u);
  let thickness = fxp.gi0.y;
  let t_end = gi_on_screen(origin, dir, max_distance);
  let w0 = (fxp.proj * vec4<f32>(origin, 1.0)).w;
  let w1 = (fxp.proj * vec4<f32>(origin + dir * t_end, 1.0)).w;
  var t_prev = 0.0;
  for (var i = 0u; i < steps; i++) {
    let k = (f32(i) + jitter) / f32(steps);
    let u = k * k;
    let t = (u * t_end / w1) / ((1.0 - u) / w0 + u / w1);
    let q = origin + dir * t;
    let s = gi_uv(q);
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
      // Without last frame's light, or where its hit lay off screen, the
      // probes give that direction's light; without them it is occluded.
      let unknown = vec4<f32>(0.0, 0.0, 0.0, select(fade, 0.0, probes));
      if (fxp.gi1.z < 0.5) { return unknown; }
      // Where the hit pixel was on the previous frame (gbuffer zw: its
      // screen-space velocity, uv per frame).
      let gsize = vec2<i32>(textureDimensions(tex_gbuffer));
      let velocity = textureLoad(tex_gbuffer, prepass_px(hit_px, gsize), 0).zw;
      let prev = s - velocity;
      if (any(prev < vec2<f32>(0.0)) || any(prev > vec2<f32>(1.0))) {
        return unknown;
      }
      var light = max(textureSampleLevel(tex_a, fx_linear, prev, 0.0).rgb, vec3<f32>(0.0));
      // Fireflies: a few rays meeting a tiny bright emitter are held to a
      // level the blur can spread.
      let l = gi_luma(light);
      light *= min(1.0, 32.0 / max(l, 1e-4));
      let edge = min(min(prev.x, 1.0 - prev.x), min(prev.y, 1.0 - prev.y));
      // Fading out by the screen's edge, a hit gives way to the probes.
      let seen = fade * smoothstep(0.0, 0.05, edge);
      return vec4<f32>(light * seen, select(fade, seen, probes));
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
  let probes = probe_data.v[GI_HEADER_AT].x > 0.5;
  // With the probes, the rays reach one spacing of their finest cascade:
  // the near field, which the probes cannot resolve. Farther, the probes
  // see what the rays would, and L1 harmonics are too smooth to say how
  // much of their light came from the directions a long ray hits, so
  // the two would count it twice.
  let reach = select(fxp.gi0.x, min(fxp.gi0.x, probe_data.v[GI_HEADER_AT + 1u].w), probes);
  var light = vec3<f32>(0.0);
  var hits = 0.0;
  // The hits' directions, each by its weight (view space).
  var towards = vec3<f32>(0.0);
  for (var i = 0u; i < rays; i++) {
    let u = fract(vec2<f32>(n0 + f32(i) * GI_GOLDEN, n1 + f32(i) * GI_PLASTIC));
    let dir = gi_cosine_dir(n, vec2<f32>(min(u.x, 0.999), u.y));
    let r = gi_ray(origin, dir, fract(n0 * 7.13 + f32(i) * 0.37), probes, reach);
    light += r.rgb;
    hits += r.a;
    towards += dir * r.a;
  }
  light *= fxp.gi0.z / f32(rays);
  var ao = 1.0 - fxp.gi2.x * hits / f32(rays);
  if (fxp.gi1.w > 1.5) {
    ao = ssao_term(in);
  } else if (fxp.gi1.w > 0.5) {
    ao = gtao_term(in);
  }
  if (probes) {
    // The lit pass adds the probes' light times `ao`; what reaches the
    // pixel is theirs less what the hits replace. With E/π along a unit n
    // c0 + c1·n (gi_sample.wgsl), the radiance they stand for is L1,
    // L(d) = c0 + 1.5 c1·d, and the rays sample the hemisphere by cosine,
    // so the hits' share of it over π is h c0 + 1.5 c1·s, with h the
    // hits' weight and s their weighted directions, each over the rays.
    // The bounce carries the difference: exact for L1 light, where an even
    // share (`ao` alone) takes too little from a bright wall beside a
    // dark sky and counts its light twice.
    let to_world = transpose(mat3x3<f32>(fxp.view[0].xyz, fxp.view[1].xyz, fxp.view[2].xyz));
    let wn = to_world * n;
    let wp = to_world * (p - fxp.view[3].xyz);
    // Towards the eye, or under an orthographic camera its axis (as the
    // lit pass's gi_sample_at).
    let eye = -(to_world * fxp.view[3].xyz);
    let towards_eye = select(normalize(eye - wp), to_world * vec3<f32>(0.0, 0.0, 1.0), fxp.cam.y > 0.5);
    let g = gi_gather(wp, wn, towards_eye, true);
    if (g.w > 0.0) {
      let h = hits / f32(rays);
      let s = to_world * towards / f32(rays);
      let all = gi_eval(g, wn);
      let hit = vec3<f32>(g.r.w, g.g.w, g.b.w) * h + 1.5 * vec3<f32>(dot(g.r.xyz, s), dot(g.g.xyz, s), dot(g.b.xyz, s));
      light += g.w * (max(all - hit, vec3<f32>(0.0)) - ao * all);
    }
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
  // GTAO's visibility (r) overshoots 1 before it is averaged (fxpost.wgsl
  // gtao_bent); after, it is clamped.
  let mean = sum / weight;
  return vec4<f32>(min(mean.r, 1.0), mean.gba);
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
