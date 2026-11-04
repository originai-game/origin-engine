// The probe GI's trace (gi_probes.rs), after the frame group (frame.wgsl,
// gi_common.wgsl, gi_probes.wgsl) and the light tables (clusters.rs): one
// thread a ray, each probe of this frame's slice of a cascade casting
// slice.w rays into the traced scene (gi_scene.rs) and writing what it
// found to the ray buffer: the radiance coming back along it and how far it
// went (negative on a back face, GI_MISS past everything). After Wicked
// Engine's ddgi_raytraceCS.hlsl (MIT, third_party/wicked-ddgi): the sky
// where a ray leaves the scene; where it hits, the sun and one local light
// picked at random (times their count), each behind a shadow ray, the
// probes' own light (the bounces before) and the surface's emission.

struct GiParams {
  rotation: mat3x3<f32>,
  // xyz: the cascade's first cell, w: its spacing.
  grid0: vec4<f32>,
  // xyz: its probes along x, y, z; w: its first probe's vec4.
  grid1: vec4<f32>,
  // x: the slice's period, y: its phase (the probes whose slot index is
  // phase mod period are updated), z: probes in it, w: rays per probe.
  slice: vec4<u32>,
  // x: the top level's root, y: instances, z: the bounce's scale,
  // w: how far a ray goes.
  trace: vec4<f32>,
  // x: the distance maps' range, y: the radiance's short window blend,
  // z: the distance maps' blend, w: 1 to start every probe again.
  blend: vec4<f32>,
  // x: the frame, y: the slice's first ray, z: its first estimator word,
  // w: local lights in the frame.
  misc: vec4<u32>,
};

@group(1) @binding(0) var<storage, read> gi_bvh: array<vec4<f32>>;
@group(1) @binding(1) var<storage, read_write> gi_rays: array<vec4<f32>>;
@group(1) @binding(2) var<uniform> gp: GiParams;

const GI_MISS: f32 = 1.0e9;
const GI_STACK: u32 = 32u;

struct GiHit {
  t: f32,
  // The instance record's first vec4 (0: none).
  record: u32,
  // The triangle's face normal in the instance's space (unnormalised).
  normal: vec3<f32>,
};

fn gi_safe_inverse(d: vec3<f32>) -> vec3<f32> {
  let s = select(vec3<f32>(1.0), vec3<f32>(-1.0), d < vec3<f32>(0.0));
  return 1.0 / select(d, s * 1e-12, abs(d) < vec3<f32>(1e-12));
}

// The entry distance of the ray into node `n`'s box, or a miss (> t_max).
fn gi_box(n: u32, o: vec3<f32>, inv: vec3<f32>, t_max: f32) -> f32 {
  let a = (gi_bvh[n].xyz - o) * inv;
  let b = (gi_bvh[n + 1u].xyz - o) * inv;
  let t0 = max(max(min(a.x, b.x), min(a.y, b.y)), max(min(a.z, b.z), 0.0));
  let t1 = min(min(max(a.x, b.x), max(a.y, b.y)), min(max(a.z, b.z), t_max));
  return select(GI_MISS * 2.0, t0, t0 <= t1);
}

// Möller and Trumbore's test of triangle `at` (both faces): its distance,
// or a miss.
fn gi_triangle(at: u32, o: vec3<f32>, d: vec3<f32>) -> f32 {
  let v0 = gi_bvh[at].xyz;
  let e1 = gi_bvh[at + 1u].xyz - v0;
  let e2 = gi_bvh[at + 2u].xyz - v0;
  let p = cross(d, e2);
  let det = dot(e1, p);
  if (abs(det) < 1e-12) { return GI_MISS * 2.0; }
  let inv = 1.0 / det;
  let s = o - v0;
  let u = dot(s, p) * inv;
  let q = cross(s, e1);
  let v = dot(d, q) * inv;
  let t = dot(e2, q) * inv;
  return select(GI_MISS * 2.0, t, u >= 0.0 && v >= 0.0 && u + v <= 1.0 && t > 0.0);
}

// The first hit along the ray from `o` along unit `d` within `t_max`; with
// `any`, any hit (a shadow ray). Both levels are walked with a stack of
// absolute vec4 links, the nearer child first (gi_scene.rs's `trace` is
// the same walk, tested on the CPU).
fn gi_trace(o: vec3<f32>, d: vec3<f32>, t_max: f32, any_hit: bool) -> GiHit {
  var best: GiHit;
  best.t = t_max;
  best.record = 0u;
  best.normal = vec3<f32>(0.0, 1.0, 0.0);
  if (gp.trace.y < 0.5) { return best; }
  let inv = gi_safe_inverse(d);
  var stack: array<u32, GI_STACK>;
  var sp = 1u;
  stack[0] = u32(gp.trace.x);
  loop {
    if (sp == 0u) { break; }
    sp -= 1u;
    let n = stack[sp];
    if (gi_box(n, o, inv, best.t) > best.t) { continue; }
    let link = u32(gi_bvh[n].w);
    let count = u32(gi_bvh[n + 1u].w);
    if (count == 0u) {
      // The nearer child last, so it is walked first.
      let tl = gi_box(link, o, inv, best.t);
      let tr = gi_box(link + 2u, o, inv, best.t);
      let near = select(link + 2u, link, tl <= tr);
      let far = select(link, link + 2u, tl <= tr);
      if (sp + 2u <= GI_STACK) {
        stack[sp] = far;
        stack[sp + 1u] = near;
        sp += 2u;
      }
      continue;
    }
    for (var k = 0u; k < count; k++) {
      let rec = link + 6u * k;
      let r0 = gi_bvh[rec];
      let r1 = gi_bvh[rec + 1u];
      let r2 = gi_bvh[rec + 2u];
      // The ray in the instance's space; its parameter keeps the world's
      // scale, as the direction is not renormalised.
      let lo = vec3<f32>(dot(r0, vec4<f32>(o, 1.0)), dot(r1, vec4<f32>(o, 1.0)), dot(r2, vec4<f32>(o, 1.0)));
      let ld = vec3<f32>(dot(r0.xyz, d), dot(r1.xyz, d), dot(r2.xyz, d));
      let linv = gi_safe_inverse(ld);
      var inner: array<u32, GI_STACK>;
      var ip = 1u;
      inner[0] = u32(gi_bvh[rec + 3u].x);
      loop {
        if (ip == 0u) { break; }
        ip -= 1u;
        let m = inner[ip];
        if (gi_box(m, lo, linv, best.t) > best.t) { continue; }
        let l2 = u32(gi_bvh[m].w);
        let c2 = u32(gi_bvh[m + 1u].w);
        if (c2 == 0u) {
          let tl = gi_box(l2, lo, linv, best.t);
          let tr = gi_box(l2 + 2u, lo, linv, best.t);
          if (ip + 2u <= GI_STACK) {
            inner[ip] = select(l2, l2 + 2u, tl <= tr);
            inner[ip + 1u] = select(l2 + 2u, l2, tl <= tr);
            ip += 2u;
          }
          continue;
        }
        for (var j = 0u; j < c2; j++) {
          let tri = l2 + 3u * j;
          let t = gi_triangle(tri, lo, ld);
          if (t < best.t) {
            best.t = t;
            best.record = rec;
            best.normal = cross(gi_bvh[tri + 1u].xyz - gi_bvh[tri].xyz, gi_bvh[tri + 2u].xyz - gi_bvh[tri].xyz);
            if (any_hit) { return best; }
          }
        }
      }
    }
  }
  return best;
}

fn gi_hash(x: u32) -> u32 {
  // PCG (Jarzynski and Olano, "Hash functions for GPU rendering", 2020).
  let state = x * 747796405u + 2891336453u;
  let word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
  return (word >> 22u) ^ word;
}

// The sky along `d`: the environment's radiance, a little blurred, at its
// intensity.
fn gi_sky(d: vec3<f32>) -> vec3<f32> {
  return textureSampleLevel(prefiltered_map, ibl_sampler, d, min(2.0, frame.params.y)).rgb * frame.params.x;
}

// The light leaving the surface hit at `p` (unit world normal `n`) back
// along `-d`: its albedo times what reaches it, plus its emission.
fn gi_shade(p: vec3<f32>, n: vec3<f32>, d: vec3<f32>, record: u32, seed: u32) -> vec3<f32> {
  let albedo = gi_bvh[record + 4u].rgb;
  let emissive = gi_bvh[record + 5u].rgb;
  let q = p + n * (0.002 + 0.01 * gp.grid0.w);
  // The sun, behind a shadow ray.
  var direct = vec3<f32>(0.0);
  let l = normalize(frame.light_dir.xyz);
  let n_dot_l = dot(n, l);
  if (n_dot_l > 0.0 && any(frame.light_color.rgb > vec3<f32>(0.0))) {
    if (gi_trace(q, l, gp.trace.w, true).record == 0u) {
      direct += frame.light_color.rgb * n_dot_l;
    }
  }
  // One local light at random, as many times as there are lights (UE4's
  // windowed inverse square, the cone of a spot, as pbr.wgsl).
  let lights = gp.misc.w;
  if (lights > 0u) {
    let light = light_at(gi_hash(seed) % lights);
    let to_light = light.position_range.xyz - q;
    let d2 = max(dot(to_light, to_light), 1e-4);
    let range = light.position_range.w;
    let ratio = d2 / (range * range);
    let window = clamp(1.0 - ratio * ratio, 0.0, 1.0);
    var atten = window * window / d2;
    let lv = to_light * inverseSqrt(d2);
    if (light.color_kind.w > 0.5) {
      let t = clamp((dot(-lv, light.direction_slot.xyz) - light.cone.x) * light.cone.y, 0.0, 1.0);
      atten *= t * t;
    }
    let ndl = dot(n, lv);
    if (atten > 0.0 && ndl > 0.0) {
      if (gi_trace(q, lv, sqrt(d2), true).record == 0u) {
        direct += light.color_kind.rgb * atten * ndl * f32(lights);
      }
    }
  }
  // The bounces before: the probes' light where the hit is, slightly less
  // than all of it so the feedback converges (Wicked's 0.95); past the
  // probes, the environment's.
  let bounce = gi_probe_light(p, n, -d, true);
  let env = textureSampleLevel(irradiance_map, ibl_sampler, n, 0.0).rgb * frame.params.x;
  let indirect = mix(env, bounce.rgb * 0.95, bounce.w) + hemi_irradiance(n);
  let reflected = albedo * (direct / GI_PI + indirect) * gp.trace.z;
  return reflected + emissive;
}

@compute @workgroup_size(64)
fn cs_trace(@builtin(global_invocation_id) id: vec3<u32>) {
  let rays = gp.slice.w;
  let k = id.x / rays;
  let ray = id.x % rays;
  if (k >= gp.slice.z) { return; }
  let flat = gp.slice.y + k * gp.slice.x;
  let counts = vec3<i32>(gp.grid1.xyz);
  let slot = vec3<i32>(i32(flat) % counts.x, (i32(flat) / counts.x) % counts.y, i32(flat) / (counts.x * counts.y));
  // The world cell the slot holds: the one of the grid's cells there.
  let start = vec3<i32>(gp.grid0.xyz);
  let w = start + (((slot - gi_slot(start, counts)) % counts) + counts) % counts;
  let at = u32(gp.grid1.w) + flat * GI_STRIDE;
  let state = probe_data.v[at];
  let fresh = abs(state.w) != gi_key(w, counts) || gp.blend.w > 0.5;
  let offset = select(state.xyz, vec3<f32>(0.0), fresh);
  let o = (vec3<f32>(w) + offset) * gp.grid0.w;
  let d = gi_ray_dir(ray, rays, gp.rotation);
  let hit = gi_trace(o, d, gp.trace.w, false);
  var out = vec4<f32>(gi_sky(d), GI_MISS);
  if (hit.record != 0u) {
    let rec = hit.record;
    // The face normal in the world: the inverse transform's transpose.
    var n = normalize(gi_bvh[rec].xyz * hit.normal.x + gi_bvh[rec + 1u].xyz * hit.normal.y + gi_bvh[rec + 2u].xyz * hit.normal.z);
    let back = dot(n, d) > 0.0;
    if (back && gi_bvh[rec + 3u].y > 0.5) { n = -n; }
    if (back && gi_bvh[rec + 3u].y < 0.5) {
      // Inside geometry: no light, and the distance marked for the
      // relocation and the classification (gi_blend.wgsl).
      out = vec4<f32>(0.0, 0.0, 0.0, -hit.t);
    } else {
      let seed = gi_hash(id.x ^ gi_hash(gp.misc.x * 9781u + gp.misc.y));
      out = vec4<f32>(gi_shade(o + d * hit.t, n, d, rec, seed), hit.t);
    }
  }
  out = vec4<f32>(clamp(out.rgb, vec3<f32>(0.0), vec3<f32>(6.0e4)), out.w);
  gi_rays[gp.misc.y + k * rays + ray] = out;
}
