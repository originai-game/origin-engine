// W2: a sea's caustics (water.rs `Caustics`): the curvature of the drawn surface on a camera-centred grid, read by
// lighting.wgsl `caustics` on everything under it.
//
// The refracted-area method (Wallace's WebGL Water, the method: light refracted through a curved surface converges or
// spreads, and the irradiance on a receiver is the surface's area over the area its rays cover there; Watt, "Light-water
// interaction using backward beam tracing", SIGGRAPH 1990). Under small slopes a ray through surface point x reaches
// depth d at x + d c ∇h(x), c = 1 - 1/1.33, so the map's Jacobian is I + d c H with H the height's Hessian, and the
// irradiance 1 / |det(I + d c H)|. The Hessian does not depend on the depth, so one grid serves every receiver; the
// shading evaluates the determinant at the receiver's own depth.
//
// The Hessian is the drawn sea's slopes differenced: og_core::ocean's waves (those a texel of the grid holds) and the
// cascades' shorter waves (their mips chosen so a texel holds them).

struct CausticParams {
  // xy the grid's texel (0, 0) centre in the frame's xz, z its texel (m), w its texels along an edge.
  grid: vec4<f32>,
  // x the cascades, yzw unused.
  sea: vec4<f32>,
  // As water.wgsl's WaterUniform.
  cascades: array<vec4<f32>, 3>,
  swell: array<vec4<f32>, 64>,
};

@group(0) @binding(0) var<uniform> cp: CausticParams;
@group(0) @binding(1) var fields: texture_2d_array<f32>;
@group(0) @binding(2) var lin: sampler;
// Per texel the Hessian (xx, zz, xz) as lighting.wgsl `caustic_h` reads it.
@group(0) @binding(3) var<storage, read_write> hessian: array<u32>;

const CAUSTIC_H: f32 = 8.0;

fn word(h: f32) -> u32 {
  let s = sign(h) * sqrt(min(abs(h) / CAUSTIC_H, 1.0));
  return u32(round(s * 511.0 + 511.0));
}

fn sea_keep(k: f32, cell: f32) -> f32 {
  return 1.0 - smoothstep(0.785, 1.571, k * cell);
}

// The sea's slope (∂h/∂x, ∂h/∂z) at frame point p: its waves the grid's texel holds, and each cascade read at the mip
// that holds the texel.
fn slope_at(p: vec2<f32>) -> vec2<f32> {
  var s = vec2<f32>(0.0);
  let ts = cp.grid.z;
  for (var i = 0u; i < 32u; i++) {
    let a = cp.swell[2u * i];
    s += a.xy * (a.z * cos(dot(a.xy, p) + a.w) * sea_keep(length(a.xy), ts));
  }
  for (var c = 0u; c < u32(cp.sea.x); c++) {
    let t = cp.cascades[c];
    let lod = log2(max(ts / (t.w / 256.0), 1.0));
    s += textureSampleLevel(fields, lin, p * t.x + t.yz, 3u * c + 1u, lod).xy;
  }
  return s;
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
  let n = u32(cp.grid.w);
  if (id.x >= n || id.y >= n) { return; }
  let ts = cp.grid.z;
  let p = cp.grid.xy + vec2<f32>(id.xy) * ts;
  let dx = slope_at(p + vec2<f32>(ts, 0.0)) - slope_at(p - vec2<f32>(ts, 0.0));
  let dz = slope_at(p + vec2<f32>(0.0, ts)) - slope_at(p - vec2<f32>(0.0, ts));
  let hxx = dx.x / (2.0 * ts);
  let hzz = dz.y / (2.0 * ts);
  let hxz = 0.25 * (dx.y + dz.x) / ts;
  hessian[id.y * n + id.x] = word(hxx) | (word(hzz) << 10u) | (word(hxz) << 20u);
}
