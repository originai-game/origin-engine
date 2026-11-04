// The light volumes' buffer (lightmap.rs): the baked and realtime light
// volumes' header and probes, then the probe GI's header rows and probes
// (gi_common.wgsl). Read by every lit fragment through ambient_light.
struct ProbeWords { v: array<vec4<f32>> };
@group(0) @binding(10) var<storage, read> probe_data: ProbeWords;

// The distance moments (mean, mean²) of a probe's map at texel `t`.
fn gi_moments(at: u32, t: vec2<i32>) -> vec2<f32> {
  let i = u32(t.y) * GI_DEPTH + u32(t.x);
  let w = probe_data.v[at + GI_DEPTH_AT + (i >> 1u)];
  return select(w.xy, w.zw, (i & 1u) == 1u);
}

// The moments seen from the probe along `dir`, bilinear over its map with
// the octahedral fold at the edges.
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

// A probe's irradiance over π along `n` (its harmonics, deringed when they
// were written, so never negative).
fn gi_probe_irradiance(at: u32, n: vec3<f32>) -> vec3<f32> {
  let r = probe_data.v[at + GI_SH_AT];
  let g = probe_data.v[at + GI_SH_AT + 1u];
  let b = probe_data.v[at + GI_SH_AT + 2u];
  return max(vec3<f32>(r.w + dot(r.xyz, n), g.w + dot(g.xyz, n), b.w + dot(b.xyz, n)), vec3<f32>(0.0));
}

// Cascade `c`'s irradiance over π at `p` for normal `n`, seen along `v`
// (towards the eye), with `visibility` the probes' distance test (Wicked's
// ddgi_sample_irradiance, the bias of RTXGI's DDGIGetSurfaceBias, which
// the paper's section 5 describes): rgb, and w the cascade's weight there
// (0 outside it, 1 two cells inside its faces); 0 where no probe around p
// holds its cell yet or every one lies inside geometry.
fn gi_cascade(c: u32, p: vec3<f32>, n: vec3<f32>, v: vec3<f32>, visibility: bool) -> vec4<f32> {
  let head = probe_data.v[GI_HEADER_AT];
  let r0 = probe_data.v[GI_HEADER_AT + 1u + 2u * c];
  let r1 = probe_data.v[GI_HEADER_AT + 2u + 2u * c];
  let spacing = r0.w;
  let counts = vec3<i32>(r1.xyz);
  let first = u32(r1.w);
  // The biased point the probes are weighed from: off the surface along
  // its normal and towards the eye, so a probe test is never made at the
  // surface, where shadowed and lit meet.
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
    // A slot holding another cell (the grid just scrolled over it), never
    // traced, or a probe inside geometry is left out.
    if (state.w != gi_key(w, counts)) { continue; }
    let probe = (vec3<f32>(w) + state.xyz) * spacing;
    let trilinear = mix(1.0 - alpha, alpha, vec3<f32>(o));
    var weight = 1.0;
    // The smooth backface test: wrap shading of the direction to the
    // probe, so the probes behind a small detail still count a little.
    let to_probe = normalize(probe - p);
    let wrap = (dot(to_probe, n) + 1.0) * 0.5;
    weight *= wrap * wrap + 0.2;
    if (visibility) {
      // Chebyshev's bound on the probe seeing the biased point (variance
      // shadow maps, Donnelly and Lauritzen 2006, eq. 5), sharpened.
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
    // A little light is very visible: crush tiny weights, keeping the
    // curve continuous, before the trilinear weights.
    let crush = 0.2;
    if (weight < crush) { weight *= weight * weight / (crush * crush); }
    weight *= trilinear.x * trilinear.y * trilinear.z;
    sum += gi_probe_irradiance(at, n) * weight;
    total += weight;
  }
  if (total <= 1e-6) { return vec4<f32>(0.0); }
  return vec4<f32>(sum / total, edge);
}

// The probe GI's irradiance over π at `p` for `n` seen along `v`: the
// finest cascade holding p, faded into the next one over its outer two
// cells; w how much of it to take (0 off or outside every cascade).
fn gi_probe_light(p: vec3<f32>, n: vec3<f32>, v: vec3<f32>, visibility: bool) -> vec4<f32> {
  let cascades = min(u32(probe_data.v[GI_HEADER_AT].x), GI_CASCADES);
  var out = vec4<f32>(0.0);
  for (var c = 0u; c < cascades; c++) {
    let s = gi_cascade(c, p, n, v, visibility);
    if (s.w <= 0.0) { continue; }
    // Over what the coarser cascades left (front to back).
    let take = s.w * (1.0 - out.w);
    out = vec4<f32>(out.rgb + s.rgb * take, out.w + take);
    if (out.w >= 0.999) { break; }
  }
  if (out.w <= 0.0) { return vec4<f32>(0.0); }
  return vec4<f32>(out.rgb / out.w, out.w);
}

// `env` (an irradiance in the environment's units: the caller scales it by
// its intensity, frame.params.x) with the probe GI over it where the probes
// reach: the light the scene sends `p`, its sky occluded by what stands
// around it.
fn gi_ambient(env: vec3<f32>, n: vec3<f32>, p: vec3<f32>) -> vec3<f32> {
  if (probe_data.v[GI_HEADER_AT].x < 0.5) { return env; }
  let v = select(normalize(frame.view_pos.xyz - p), -frame.view_forward.xyz, frame.view_pos.w > 0.5);
  let g = gi_probe_light(p, n, v, true);
  return mix(env, g.rgb / max(frame.params.x, 1e-4), g.w);
}

// How much of the environment's reflection along `r` at `p` reaches it
// past what the probes see around it (1 where they see sky, towards 0 in a
// closed room): the probes' irradiance along r over the environment's,
// by luminance, for the reflections a probe GI scene has outside every
// reflection probe. 1 without the probe GI.
fn gi_reflection_occlusion(r: vec3<f32>, p: vec3<f32>, env: vec3<f32>) -> f32 {
  if (probe_data.v[GI_HEADER_AT].x < 0.5) { return 1.0; }
  let g = gi_probe_light(p, r, r, false);
  if (g.w <= 0.0) { return 1.0; }
  let luma = vec3<f32>(0.2126, 0.7152, 0.0722);
  let seen = dot(g.rgb, luma) / max(frame.params.x, 1e-4);
  let sky = max(dot(env, luma), 1e-4);
  return mix(1.0, clamp(seen / sky, 0.0, 1.0), g.w);
}

// `prefiltered` (the environment's reflection along `r` at `p`) occluded by
// what the probe GI sees there (gi_reflection_occlusion): a mirror under a
// roof stops reflecting the sky's full light. A reflection probe's capture
// already holds what stands around it, so where one covers `p` it is kept.
fn gi_reflected(prefiltered: vec3<f32>, r: vec3<f32>, p: vec3<f32>) -> vec3<f32> {
  if (probe_data.v[GI_HEADER_AT].x < 0.5) { return prefiltered; }
  let captured = select(0.0, probe_weight(p), frame.probe_min.w > 0.5);
  let k = gi_reflection_occlusion(r, p, ambient_light_level(r, p));
  return prefiltered * mix(k, 1.0, captured);
}
