// The probe GI's blend (gi_probes.rs), after gi_common.wgsl: one workgroup
// a probe of this frame's slice, gathering its rays (gi_trace.wgsl) into
// its record in the light volumes' buffer. After Wicked Engine's
// ddgi_updateCS.hlsl (MIT, third_party/wicked-ddgi):
// - its radiance on a GI_RADIANCE² octahedral map, each texel filtered in
//   time by the multiscale mean estimator (below), then projected onto the
//   first-order harmonics the lit passes read, as irradiance over π;
// - its distance moments on a GI_DEPTH² map, blended in time;
// - its relocation off the surfaces nearest it, and whether it lies inside
//   geometry (most rays hit back faces: then the lit passes leave it out).

struct GiParams {
  rotation: mat3x3<f32>,
  grid0: vec4<f32>,
  grid1: vec4<f32>,
  slice: vec4<u32>,
  trace: vec4<f32>,
  blend: vec4<f32>,
  misc: vec4<u32>,
};

@group(0) @binding(0) var<storage, read> gi_rays: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> probes: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> estimator: array<vec4<f32>>;
@group(0) @binding(3) var<uniform> gp: GiParams;

const THREADS: u32 = 144u;
const TEXELS: u32 = 36u;
// The estimator's vec4 a radiance texel.
const WORDS: u32 = 3u;
// Back faces on more than this share of a probe's rays: it is inside.
const INSIDE: f32 = 0.25;
// How far a probe keeps from surfaces, as a share of the maps' range
// (Wicked's DDGI_KEEP_DISTANCE).
const KEEP: f32 = 0.1;

var<workgroup> cache: array<vec4<f32>, THREADS>;
var<workgroup> cache_dir: array<vec3<f32>, THREADS>;
var<workgroup> radiance: array<vec3<f32>, TEXELS>;
var<workgroup> depth: array<vec2<f32>, THREADS>;

// What the estimator keeps of a texel.
struct Estimate {
  mean: vec3<f32>,
  short_mean: vec3<f32>,
  vbbr: f32,
  variance: vec3<f32>,
  inconsistency: f32,
};

fn load_estimate(at: u32) -> Estimate {
  let a = estimator[at];
  let b = estimator[at + 1u];
  let c = estimator[at + 2u];
  var s: Estimate;
  s.mean = a.xyz;
  s.short_mean = b.xyz;
  s.variance = c.xyz;
  s.vbbr = a.w;
  s.inconsistency = b.w;
  return s;
}

fn store_estimate(at: u32, s: Estimate) {
  estimator[at] = vec4<f32>(s.mean, s.vbbr);
  estimator[at + 1u] = vec4<f32>(s.short_mean, s.inconsistency);
  estimator[at + 2u] = vec4<f32>(s.variance, 0.0);
}

// The multiscale mean estimator (Tomasz Stachowiak, kajiya, 2022; as
// Wicked's MultiscaleMeanEstimator): a short and a long running mean with
// a variance, the long one catching up fast when the two disagree beyond
// the noise (a light switched on) and slowly while they agree.
fn estimate(y_in: vec3<f32>, s_in: Estimate, short_blend: f32) -> Estimate {
  var s = s_in;
  var y = y_in;
  let luma = vec3<f32>(0.299, 0.587, 0.114);
  // Fireflies held down.
  {
    let dev = sqrt(max(vec3<f32>(1e-5), s.variance));
    let high = 0.1 + s.short_mean + dev * 8.0;
    y -= max(vec3<f32>(0.0), y - high);
  }
  let delta = y - s.short_mean;
  s.short_mean = mix(s.short_mean, y, short_blend);
  let delta2 = y - s.short_mean;
  // A longer window than the short mean's, so the variance is not biased
  // down as the short mean settles.
  s.variance = mix(s.variance, delta * delta2, short_blend * 0.5);
  let dev = sqrt(max(vec3<f32>(1e-5), s.variance));
  let short_diff = s.mean - s.short_mean;
  let relative = dot(luma, abs(short_diff) / max(vec3<f32>(1e-5), dev));
  s.inconsistency = mix(s.inconsistency, relative, 0.08);
  let reduction = clamp(dot(luma, 0.5 * s.short_mean / max(vec3<f32>(1e-5), dev)), 1.0 / 32.0, 1.0);
  var catch_up = clamp(smoothstep(0.0, 1.0, relative * max(0.02, s.inconsistency - 0.2)), 1.0 / 256.0, 1.0);
  catch_up *= s.vbbr;
  s.vbbr = mix(s.vbbr, reduction, 0.1);
  s.mean = mix(s.mean, y, clamp(catch_up, 0.0, 1.0));
  return s;
}

@compute @workgroup_size(144)
fn cs_blend(@builtin(local_invocation_index) t: u32, @builtin(workgroup_id) group: vec3<u32>) {
  let k = group.x;
  let rays = gp.slice.w;
  let flat = gp.slice.y + k * gp.slice.x;
  let counts = vec3<i32>(gp.grid1.xyz);
  let slot = vec3<i32>(i32(flat) % counts.x, (i32(flat) / counts.x) % counts.y, i32(flat) / (counts.x * counts.y));
  let start = vec3<i32>(gp.grid0.xyz);
  let w = start + (((slot - gi_slot(start, counts)) % counts) + counts) % counts;
  let key = gi_key(w, counts);
  let at = u32(gp.grid1.w) + flat * GI_STRIDE;
  let state = probes[at];
  let fresh = abs(state.w) != key || gp.blend.w > 0.5;
  // Its last update found it inside geometry (and moved it out): what it
  // has seen so far is the dark inside, so its light and distances start
  // again from this update's rays, as a fresh probe's do. Kept, they
  // would hold it dark for as long as the estimator takes to catch up (a
  // probe on a wall's plane would mottle the wall for a hundred frames).
  let restart = fresh || state.w == -key;
  let range = gp.blend.x;
  let first_ray = gp.misc.y + k * rays;

  // This thread's distance texel, and its radiance texel for the first 36.
  let dt = vec2<f32>(f32(t % GI_DEPTH), f32(t / GI_DEPTH));
  let depth_dir = gi_oct_decode((dt + 0.5) / f32(GI_DEPTH) * 2.0 - 1.0);
  let rt = vec2<f32>(f32(t % GI_RADIANCE), f32(t / GI_RADIANCE));
  let radiance_dir = gi_oct_decode((rt + 0.5) / f32(GI_RADIANCE) * 2.0 - 1.0);
  var moments = vec2<f32>(0.0);
  var moment_weight = 0.0;
  var light = vec3<f32>(0.0);
  var light_weight = 0.0;
  for (var base = 0u; base < rays; base += THREADS) {
    let n = min(THREADS, rays - base);
    if (t < n) {
      cache[t] = gi_rays[first_ray + base + t];
      cache_dir[t] = gi_ray_dir(base + t, rays, gp.rotation);
    }
    workgroupBarrier();
    for (var r = 0u; r < n; r++) {
      let ray = cache[r];
      let dir = cache_dir[r];
      // A back face counts as nearer than it is, so light from inside a
      // wall does not get through (Wicked's 0.9).
      let dist = min(select(ray.w, -ray.w * 0.9, ray.w < 0.0), range);
      let c = max(dot(depth_dir, dir), 0.0);
      let wd = pow(c, 50.0);
      if (wd > 1e-4) {
        moments += vec2<f32>(dist, dist * dist) * wd;
        moment_weight += wd;
      }
      let cr = max(dot(radiance_dir, dir), 0.0);
      let wr = cr * cr * cr * cr;
      if (wr > 1e-4) {
        light += ray.rgb * wr;
        light_weight += wr;
      }
    }
    workgroupBarrier();
  }

  // Radiance: the estimator, then the harmonics.
  if (t < TEXELS) {
    let word = gp.misc.z + (flat * TEXELS + t) * WORDS;
    var s = load_estimate(word);
    let y = select(s.mean, light / max(light_weight, 1e-6), light_weight > 1e-4);
    if (restart) {
      s.mean = y;
      s.short_mean = y;
      s.variance = vec3<f32>(0.0);
      s.vbbr = 0.0;
      s.inconsistency = 1.0;
    }
    s = estimate(y, s, gp.blend.y);
    store_estimate(word, s);
    radiance[t] = s.mean;
  }

  // Distances: blended in time, two texels a vec4.
  let old_at = at + GI_DEPTH_AT + (t >> 1u);
  let old = select(probes[old_at].xy, probes[old_at].zw, (t & 1u) == 1u);
  var m = select(old, moments / max(moment_weight, 1e-6), moment_weight > 1e-4);
  if (!restart) { m = mix(old, m, gp.blend.z); }
  depth[t] = m;
  workgroupBarrier();
  if (t < GI_DEPTH * GI_DEPTH / 2u) {
    probes[at + GI_DEPTH_AT + t] = vec4<f32>(depth[2u * t], depth[2u * t + 1u]);
  }

  if (t == 0u) {
    // The harmonics: each texel's radiance over its share of the sphere
    // (4π / 36), as probe_grid.wgsl projects a capture; deringed.
    var sh = array<vec4<f32>, 3>(vec4<f32>(0.0), vec4<f32>(0.0), vec4<f32>(0.0));
    for (var i = 0u; i < TEXELS; i++) {
      let p = vec2<f32>(f32(i % GI_RADIANCE), f32(i / GI_RADIANCE));
      let d = gi_oct_decode((p + 0.5) / f32(GI_RADIANCE) * 2.0 - 1.0);
      let basis = vec4<f32>(d * (2.0 / f32(TEXELS)), 1.0 / f32(TEXELS));
      sh[0] += basis * radiance[i].r;
      sh[1] += basis * radiance[i].g;
      sh[2] += basis * radiance[i].b;
    }
    for (var c = 0u; c < 3u; c++) {
      // The texels hold the radiance filtered by the cos⁴ lobe above,
      // which scales the first band by (n + 1) / (n + 2) = 5/6 (Funk and
      // Hecke's theorem for a normalised cosine power): undone, so a probe
      // over a darker floor keeps the sky's light on the floor beside it.
      sh[c] = vec4<f32>(sh[c].xyz * 1.2, sh[c].w);
      let band = length(sh[c].xyz);
      let keep = select(1.0, max(sh[c].w, 0.0) / band, band > max(sh[c].w, 0.0));
      probes[at + GI_SH_AT + c] = vec4<f32>(sh[c].xyz * keep, max(sh[c].w, 0.0));
    }

    // Relocation (Wicked's, and RTXGI's classification): pushed off the
    // surfaces nearer than KEEP of the range; inside geometry, through
    // the nearest back face.
    let spacing = gp.grid0.w;
    let keep = range * KEEP;
    var push = vec3<f32>(0.0);
    var backs = 0u;
    var nearest_back = vec4<f32>(0.0, 0.0, 0.0, 3.4e38);
    for (var r = 0u; r < rays; r++) {
      let ray = gi_rays[first_ray + r];
      let dir = gi_ray_dir(r, rays, gp.rotation);
      if (ray.w < 0.0) {
        backs += 1u;
        if (-ray.w < nearest_back.w) { nearest_back = vec4<f32>(dir, -ray.w); }
      } else if (ray.w < keep) {
        push -= dir * (keep - ray.w);
      }
    }
    let inside = f32(backs) > INSIDE * f32(rays);
    var offset = select(state.xyz * spacing, vec3<f32>(0.0), fresh);
    // Out of a surface's way it drifts back to its rest (the push is the
    // offset its rays ask for from where it is, so it settles where they
    // stop asking).
    if (inside) {
      offset = mix(offset, offset + nearest_back.xyz * (nearest_back.w + keep), 0.25);
    } else {
      offset = mix(offset, offset + push, 0.05) * 0.99;
    }
    offset = clamp(offset, vec3<f32>(-0.45 * spacing), vec3<f32>(0.45 * spacing)) / spacing;
    probes[at] = vec4<f32>(offset, select(key, -key, inside));
  }
}
