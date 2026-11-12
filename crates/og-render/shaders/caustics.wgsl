// W2: a sea's surface curvature about the camera (water_caustics.wgsl), a uniform (the lit pipelines' storage
// buffers are spent): per texel its Hessian (xx, zz, xz) as three signed 10-bit words, each the square root of its
// share of CAUSTIC_H (finer near zero, where most of the sea's curvature lies).
struct CausticGrid { w: array<vec4<u32>, 1024> };
@group(0) @binding(12) var<uniform> caustic_grid: CausticGrid;
const CAUSTIC_H: f32 = 8.0;

fn caustic_word(q: u32) -> f32 {
  let s = (f32(q & 1023u) - 511.0) / 511.0;
  return sign(s) * s * s * CAUSTIC_H;
}

fn caustic_h(i: vec2<i32>, n: i32) -> vec3<f32> {
  let k = u32(clamp(i.y, 0, n - 1) * n + clamp(i.x, 0, n - 1));
  let w = caustic_grid.w[k / 4u][k % 4u];
  return vec3<f32>(caustic_word(w), caustic_word(w >> 10u), caustic_word(w >> 20u));
}

// The waves' own caustics at `p`, `depth` under a sea's surface (water_caustics.wgsl): the irradiance the surface's
// curvature focuses there, 1 / |det(I + d c H)| at the surface point the refracted sun came through, less 1 (0: as
// the flat surface lights it). Past the focus the rays cross and the pattern is chaos the grid cannot hold: the
// determinant's magnitude is held off zero by how far the receiver lies past the waves' focus.
fn wave_caustics(p: vec3<f32>, depth: f32) -> f32 {
  let g = frame.caustics_cells[0];
  let n = i32(g.w);
  let l = normalize(frame.light_dir.xyz);
  if (l.y <= 0.0) { return 0.0; }
  let t = refract(-l, vec3<f32>(0.0, 1.0, 0.0), 1.0 / 1.33);
  let s = p.xz - t.xz * (depth / max(-t.y, 0.05));
  let u = (s - g.xy) / g.z;
  let i = vec2<i32>(floor(u));
  let f = u - floor(u);
  let h = mix(mix(caustic_h(i, n), caustic_h(i + vec2<i32>(1, 0), n), f.x),
              mix(caustic_h(i + vec2<i32>(0, 1), n), caustic_h(i + vec2<i32>(1, 1), n), f.x), f.y);
  // Its edge fades out (the grid holds 25.6 m about the camera).
  let edge = max(abs(u.x / f32(n) - 0.5), abs(u.y / f32(n) - 0.5)) * 2.0;
  let keep = 1.0 - smoothstep(0.7, 1.0, edge);
  let k = depth * 0.248 / max(-t.y, 0.05);
  let det = (1.0 + k * h.x) * (1.0 + k * h.y) - k * k * h.z * h.z;
  let floor_det = 0.25 + 0.04 * depth;
  let e = 1.0 / max(abs(det), floor_det);
  return clamp(e - 1.0, -0.8, 3.0) * keep;
}

// Caustics under the frame's water: two scrolling cell patterns (bright
// where the waves focus light), fading in below the surface; under a sea
// (W2) its waves' own.
// Whether a pipeline has the caustics' code (pipeline_features.rs).
override OG_CAUSTICS: bool = true;

fn caustics(p: vec3<f32>) -> f32 {
  if (!OG_CAUSTICS) { return 0.0; }
  let c = frame.caustics;
  if (c.y <= 0.0) { return 0.0; }
  let area = frame.caustics_area;
  if (any(p.xz < area.xy) || any(p.xz > area.zw)) { return 0.0; }
  let depth = c.x - p.y;
  if (depth <= 0.0) { return 0.0; }
  if (frame.caustics_cells[1].w > 0.5) {
    return wave_caustics(p, depth) * c.y * smoothstep(0.0, 0.4, depth) * exp(-depth * c.w);
  }
  let t = frame.time.x;
  // `p` is from the frame's origin: each layer adds its cells there.
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

// 1 on the cell borders of a jittered grid, falling off towards the centres.
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
