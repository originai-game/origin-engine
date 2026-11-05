// The soft sun's moment pyramid (shadow_moments.rs; read by lighting.wgsl
// `pcss`): for each cascade, the mean of the four power moments
// (z, z², z³, z⁴) of its stored depth (z in [-1, 1]) over every block of
// 2^L x 2^L texels, for levels L = MOMENT_LEVEL0 to MOMENT_LEVELS. Moment
// shadow mapping (Peters and Klein, "Moment Shadow Mapping", I3D 2015;
// Peters, Münstermann, Wetzstein and Klein, "Improved Moment Shadow Maps
// for Translucent Occluders, Soft Shadows and Single Scattering", JCGT
// 6(1), 2017) reconstructs from such means how much of a region lies in
// front of a depth, so a wide penumbra is one filtered read instead of a
// few random taps, and a caster thinner than a texel keeps its share of
// every block it crosses instead of vanishing between taps.
//
// The pyramid lives in R32Float layers of the cascade array after the
// cascades (the lit pipelines have no texture slot for another binding):
// a moment texel is a 2 x 2 block of R32 texels (z, z² / z³, z⁴), and each
// moments layer packs three cascades as a quadtree, the three taking
// three quarters at each level and the fourth quarter holding the next
// one (lighting.wgsl `moment_origin` reads the same packing).
//
// One workgroup reduces a 128 x 128 block of depth texels: each thread
// four level-2 texels (4 x 4 depth texels each), then levels 3 to 7 in
// workgroup memory, so every depth texel is read once.

struct MomentParams {
  size: u32,       // the cascade map's side
  levels: u32,     // the deepest level written
  pad: vec2<u32>,
};
// The cascades' layers, and the moments' layers after them (views of the
// one cascade array).
@group(0) @binding(0) var depth_map: texture_2d_array<f32>;
@group(0) @binding(1) var moments: texture_storage_2d_array<r32float, write>;
@group(0) @binding(2) var<uniform> params: MomentParams;

const MOMENT_LEVEL0: u32 = 2u;

var<workgroup> tile: array<vec4<f32>, 256>;

fn moment_vector(d: f32) -> vec4<f32> {
  let z = 2.0 * d - 1.0;
  let z2 = z * z;
  return vec4<f32>(z, z2, z2 * z, z2 * z2);
}

// Where (cascade c, level l) starts in its moments layer, in R32 texels
// (lighting.wgsl `moment_origin` is the same).
fn origin(c: u32, l: u32) -> vec2<u32> {
  var base = 0u;
  for (var j = MOMENT_LEVEL0; j < l; j++) { base += 2u * (params.size >> j); }
  let slot = c % 3u;
  return vec2<u32>(base) + vec2<u32>(slot & 1u, slot >> 1u) * (2u * (params.size >> l));
}

fn store(c: u32, l: u32, t: vec2<u32>, m: vec4<f32>) {
  if (l > params.levels || any(t >= vec2<u32>(params.size >> l))) { return; }
  let p = vec2<i32>(origin(c, l) + 2u * t);
  let layer = i32(c / 3u);
  textureStore(moments, p, layer, vec4<f32>(m.x, 0.0, 0.0, 1.0));
  textureStore(moments, p + vec2<i32>(1, 0), layer, vec4<f32>(m.y, 0.0, 0.0, 1.0));
  textureStore(moments, p + vec2<i32>(0, 1), layer, vec4<f32>(m.z, 0.0, 0.0, 1.0));
  textureStore(moments, p + vec2<i32>(1, 1), layer, vec4<f32>(m.w, 0.0, 0.0, 1.0));
}

@compute @workgroup_size(16, 16)
fn reduce(
  @builtin(workgroup_id) wg: vec3<u32>,
  @builtin(local_invocation_id) lid: vec3<u32>,
  @builtin(local_invocation_index) li: u32,
) {
  let c = wg.z;
  let last = i32(params.size) - 1;
  // Level 2: four texels per thread, each the mean over 4 x 4 depth texels.
  let t2 = wg.xy * 32u + lid.xy * 2u;
  var sum3 = vec4<f32>(0.0);
  for (var k = 0u; k < 4u; k++) {
    let t = t2 + vec2<u32>(k & 1u, k >> 1u);
    var m = vec4<f32>(0.0);
    for (var y = 0; y < 4; y++) {
      for (var x = 0; x < 4; x++) {
        let at = min(vec2<i32>(t * 4u) + vec2<i32>(x, y), vec2<i32>(last));
        m += moment_vector(textureLoad(depth_map, at, i32(c), 0).r);
      }
    }
    m /= 16.0;
    store(c, 2u, t, m);
    sum3 += m;
  }
  // Level 3: this thread's four.
  let m3 = sum3 / 4.0;
  store(c, 3u, wg.xy * 16u + lid.xy, m3);
  tile[li] = m3;
  workgroupBarrier();
  // Levels 4 to 7: each the mean of 2 x 2 of the one before, in place in
  // `tile` (a level of n x n texels kept at its first n x n threads'
  // indices, row stride 16).
  var n = 8u;
  for (var l = 4u; l <= 7u; l++) {
    var m = vec4<f32>(0.0);
    let inside = lid.x < n && lid.y < n;
    if (inside) {
      let a = lid.y * 32u + lid.x * 2u;
      m = (tile[a] + tile[a + 1u] + tile[a + 16u] + tile[a + 17u]) / 4.0;
    }
    workgroupBarrier();
    if (inside) {
      tile[lid.y * 16u + lid.x] = m;
      store(c, l, wg.xy * n + lid.xy, m);
    }
    workgroupBarrier();
    n /= 2u;
  }
}
