// The probe GI's sampling (gi_probes.rs), after gi_common.wgsl: what the
// lit modules (gi_probes.wgsl, through the frame group's binding 10) and
// the screen GI (ssgi.wgsl, through the effect group's binding 7) read of
// the probes. Each declares `probe_data` over the light volumes' buffer.
//
// A gather weighs the 8 probes around a point in each cascade once and
// returns their weighted harmonics (GiSample), which are linear in the
// direction: the caller evaluates them along the normal for the diffuse
// light and along the reflected ray for the reflections' occlusion, and
// the screen GI takes from them exactly what its rays' hits replace.

struct ProbeWords { v: array<vec4<f32>> };

// The probes' irradiance over π around a point, `[c1.xyz, c0]` per
// channel (E/π along n is c0 + c1·n), and w how much of it to take (0
// off or outside every cascade).
struct GiSample {
  r: vec4<f32>,
  g: vec4<f32>,
  b: vec4<f32>,
  w: f32,
};

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
  let p = (oct_encode(dir) * 0.5 + 0.5) * f32(GI_DEPTH) - 0.5;
  let base = vec2<i32>(floor(p));
  let f = p - floor(p);
  let a = gi_moments(at, gi_oct_wrap(base, n));
  let b = gi_moments(at, gi_oct_wrap(base + vec2<i32>(1, 0), n));
  let c = gi_moments(at, gi_oct_wrap(base + vec2<i32>(0, 1), n));
  let d = gi_moments(at, gi_oct_wrap(base + vec2<i32>(1, 1), n));
  return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}

// `s`'s irradiance over π along `n`, never negative (the harmonics were
// deringed when they were written; a weighted sum of them stays so along
// a unit n). `n` need not be unit: the screen GI evaluates the share its
// rays missed along a shortened direction.
fn gi_eval(s: GiSample, n: vec3<f32>) -> vec3<f32> {
  return max(vec3<f32>(s.r.w + dot(s.r.xyz, n), s.g.w + dot(s.g.xyz, n), s.b.w + dot(s.b.xyz, n)), vec3<f32>(0.0));
}

// Cascade `c`'s harmonics at `p` for normal `n`, seen along `v` (towards
// the eye), with `visibility` the probes' distance test (Wicked's
// ddgi_sample_irradiance, the bias of RTXGI's DDGIGetSurfaceBias, which
// the paper's section 5 describes), normalised; w the cascade's weight
// there (0 outside it, 1 two cells inside its faces), 0 where no probe
// around p holds its cell yet or every one lies inside geometry.
fn gi_cascade(c: u32, p: vec3<f32>, n: vec3<f32>, v: vec3<f32>, visibility: bool) -> GiSample {
  var out = GiSample(vec4<f32>(0.0), vec4<f32>(0.0), vec4<f32>(0.0), 0.0);
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
  if (edge <= 0.0) { return out; }
  let base = vec3<i32>(floor(r0.xyz + floor(g)));
  let alpha = fract(g);
  // gi_slot and gi_key of the base cell, once: the other corners are one
  // cell further along an axis, wrapping to slot 0 and the next key there
  // (integer division is slow, and eight corners did it eight times).
  let base_slot = gi_slot(base, counts);
  let base_wrap = (base - base_slot) / counts;
  var total = 0.0;
  for (var i = 0u; i < 8u; i++) {
    let o = vec3<i32>(vec3<u32>(i, i >> 1u, i >> 2u) & vec3<u32>(1u));
    let w = base + o;
    let past = (base_slot + o) >= counts;
    let slot = select(base_slot + o, vec3<i32>(0), past);
    let cycle = (base_wrap + select(vec3<i32>(0), vec3<i32>(1), past)) & vec3<i32>(255);
    let key = f32(cycle.x + cycle.y * 256 + cycle.z * 65536 + 1);
    let at = first + gi_flat(slot, counts) * GI_STRIDE;
    let state = probe_data.v[at];
    // A slot holding another cell (the grid just scrolled over it), never
    // traced, or a probe inside geometry is left out.
    if (state.w != key) { continue; }
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
    out.r += probe_data.v[at + GI_SH_AT] * weight;
    out.g += probe_data.v[at + GI_SH_AT + 1u] * weight;
    out.b += probe_data.v[at + GI_SH_AT + 2u] * weight;
    total += weight;
  }
  if (total <= 1e-6) { return GiSample(vec4<f32>(0.0), vec4<f32>(0.0), vec4<f32>(0.0), 0.0); }
  return GiSample(out.r / total, out.g / total, out.b / total, edge);
}

// The probes' harmonics at `p` for `n` seen along `v`: the finest cascade
// holding p, faded into the next one over its outer two cells.
fn gi_gather(p: vec3<f32>, n: vec3<f32>, v: vec3<f32>, visibility: bool) -> GiSample {
  var out = GiSample(vec4<f32>(0.0), vec4<f32>(0.0), vec4<f32>(0.0), 0.0);
  let cascades = min(u32(probe_data.v[GI_HEADER_AT].x), GI_CASCADES);
  for (var c = 0u; c < cascades; c++) {
    let s = gi_cascade(c, p, n, v, visibility);
    if (s.w <= 0.0) { continue; }
    // Over what the coarser cascades left (front to back).
    let take = s.w * (1.0 - out.w);
    out = GiSample(out.r + s.r * take, out.g + s.g * take, out.b + s.b * take, out.w + take);
    if (out.w >= 0.999) { break; }
  }
  if (out.w <= 0.0) { return out; }
  return GiSample(out.r / out.w, out.g / out.w, out.b / out.w, out.w);
}
