// The aurora's footprint (aurora.rs): per texel of a square about the
// viewer, the electron flux into the curtains and the distance to the
// nearest one, which the sky's march (sky.wgsl aurora_over) steps by
// (Lawlor & Genetti 2011, sections 2 and 3.2).
//
// The square is warped radially: texel coordinate q in [-1, 1]² is the
// point q |q| R about the viewer (R the footprint's radius, km), so the
// texels are finest overhead and widest at the horizon, where a curtain
// is far and foreshortened.
//
// Out, per texel: r the sheet's flux (the Kelvin–Helmholtz strip wrapped
// along the nearest curve of each curtain, summed over curtains), g the
// distance to the nearest sheet (km, never more than the true one), b the
// flux-weighted characteristic energy (keV), a the red line's flux, the
// sheet's spread across tens of km (O(1D) lives 110 s and drifts).

struct Params {
  head: vec4<f32>,  // x side, y radius (km), z curtains, w points per curtain
  look: vec4<f32>,  // x the strip's snapshot (0..3), y the sheet's half-width (km), z the red spread (km), w its scale
  curtain: array<vec4<f32>, 6>,  // x characteristic energy (keV)
  points: array<vec4<f32>, 768>,  // xy about the viewer (km), z the strip along (periods), w brightness
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var strip: texture_2d<f32>;
@group(0) @binding(2) var strip_sampler: sampler;
@group(0) @binding(3) var footprint: texture_storage_2d<rgba16float, write>;

const MAX_NEAR: u32 = 128u;

var<workgroup> near: array<u32, 128>;
var<workgroup> near_count: atomic<u32>;
var<workgroup> tile_min: atomic<u32>;

fn place(q: vec2<f32>) -> vec2<f32> {
  return q * length(q) * params.head.y;
}

// The distance from `x` to the segment a–b.
fn segment_distance(x: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
  let ab = b - a;
  let h = clamp(dot(x - a, ab) / max(dot(ab, ab), 1e-6), 0.0, 1.0);
  return length(x - a - ab * h);
}

@compute @workgroup_size(8, 8)
fn main(
  @builtin(global_invocation_id) gid: vec3<u32>,
  @builtin(local_invocation_index) li: u32,
  @builtin(workgroup_id) wid: vec3<u32>,
) {
  let n = u32(params.head.x);
  let per = u32(params.head.w);
  let curtains = u32(params.head.z);
  let hw = params.look.y;
  let red_sigma = params.look.z;
  // A segment farther than this from every texel adds nothing to it.
  let reach = max(hw, 3.0 * red_sigma);
  if (li == 0u) {
    atomicStore(&near_count, 0u);
    atomicStore(&tile_min, bitcast<u32>(1.0e9));
  }
  workgroupBarrier();
  // The tile's bounding circle in the world: its centre's place and the
  // farthest of its corners and edge middles (the warp bends the edges a
  // little; the margin covers it).
  let to_q = 2.0 / f32(n);
  let t0 = vec2<f32>(wid.xy * 8u) * to_q - 1.0;
  let t1 = t0 + 8.0 * to_q;
  let tc = place((t0 + t1) * 0.5);
  var radius = 0.0;
  for (var k = 0u; k < 9u; k++) {
    let f = vec2<f32>(f32(k % 3u), f32(k / 3u)) * 0.5;
    radius = max(radius, length(place(mix(t0, t1, f)) - tc));
  }
  radius = radius * 1.05 + 0.1;
  let segments = curtains * (per - 1u);
  for (var j = li; j < segments; j += 64u) {
    let i = (j / (per - 1u)) * per + j % (per - 1u);
    let d = segment_distance(tc, params.points[i].xy, params.points[i + 1u].xy) - radius;
    atomicMin(&tile_min, bitcast<u32>(max(d, 0.0)));
    if (d < reach) {
      let s = atomicAdd(&near_count, 1u);
      if (s < MAX_NEAR) { near[s] = j; }
    }
  }
  workgroupBarrier();
  if (gid.x >= n || gid.y >= n) { return; }
  let x = place((vec2<f32>(gid.xy) + 0.5) * to_q - 1.0);
  let count = atomicLoad(&near_count);
  // Every segment off the list is at least `reach` away (when the list
  // holds them all); the tile's bound holds regardless.
  var nearest = bitcast<f32>(atomicLoad(&tile_min));
  if (count <= MAX_NEAR) { nearest = max(nearest, reach); }
  // Per curtain, its nearest segment: the signed distance across, the
  // strip's coordinate along, the brightness.
  var best = array<vec4<f32>, 6>();
  for (var c = 0u; c < 6u; c++) { best[c] = vec4<f32>(1.0e9, 0.0, 0.0, 0.0); }
  for (var k = 0u; k < min(count, MAX_NEAR); k++) {
    let j = near[k];
    let c = j / (per - 1u);
    let i = c * per + j % (per - 1u);
    let a = params.points[i];
    let b = params.points[i + 1u];
    let ab = b.xy - a.xy;
    let h = clamp(dot(x - a.xy, ab) / max(dot(ab, ab), 1e-6), 0.0, 1.0);
    let off = x - a.xy - ab * h;
    let dist = length(off);
    nearest = min(nearest, dist);
    if (dist < abs(best[c].x)) {
      let side = select(-1.0, 1.0, ab.x * off.y - ab.y * off.x >= 0.0);
      best[c] = vec4<f32>(dist * side, mix(a.z, b.z, h), mix(a.w, b.w, h), 0.0);
    }
  }
  // The strip's snapshots blended by activity: r smooth .. a turbulent.
  let snap = params.look.x;
  let weights = max(vec4<f32>(1.0) - abs(vec4<f32>(snap) - vec4<f32>(0.0, 1.0, 2.0, 3.0)), vec4<f32>(0.0));
  var flux = 0.0;
  var energy = 0.0;
  var weight = 0.0;
  var red = 0.0;
  for (var c = 0u; c < curtains; c++) {
    let s = best[c].x;
    if (abs(s) >= reach) { continue; }
    if (abs(s) < hw) {
      let across = s / hw * 0.5 + 0.5;
      let dye = dot(textureSampleLevel(strip, strip_sampler, vec2<f32>(best[c].y, across), 0.0), weights);
      let f = dye * best[c].z;
      flux += f;
      energy += f * params.curtain[c].x;
      weight += f;
    }
    let r = best[c].z * params.look.w * exp(-s * s / (2.0 * red_sigma * red_sigma));
    red += r;
    energy += r * params.curtain[c].x;
    weight += r;
  }
  // The energy of the curtains lighting the texel, the first's elsewhere.
  let e0 = select(params.curtain[0].x, energy / max(weight, 1e-9), weight > 1e-6);
  textureStore(footprint, vec2<i32>(gid.xy), vec4<f32>(flux, max(nearest - hw, 0.0), e0, red));
}
