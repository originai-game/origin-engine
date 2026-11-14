// The meshlet path's cluster culling (meshlet.rs states each test on the
// CPU, meshlet_gpu.rs drives it). One workgroup takes one instance of a
// clustered batch, from the sorted instances GPU culling uploaded (region
// 0 of its buffer, so a work item keeps the instance's sorted slot, where
// its previous transform also is): it tests the instance's sphere against
// the frustum and GPU culling's depth pyramid, picks its level by
// screen-space error (lod.rs `select` without hysteresis) unless its
// caller picked it (RenderScene::instance_levels), then its 64
// threads test the level's clusters (frustum, normal cone, the pyramid)
// and mark visible cluster slots. meshlet_compact.wgsl compacts those
// slots in row/instance/cluster order, including soft-list overflow.
// A row is one batch in one view: view 0 is the camera, the others the
// planar reflections, views and reflection probe faces (their own eye,
// planes and level) and the shadow passes (a cascade's or a local light's
// face), whose rows test their own planes, never the cone, and draw the
// level the camera picks (their view rows hold the camera's eye: a
// caster's shadow is the shape it is drawn with). Only the camera's rows
// test the pyramid.
//
// The camera's occlusion runs in two phases (meshlet_gpu.rs), each a
// dispatch of this entry point: the early phase draws the clusters
// visible last frame (`vis`, one word per cluster slot of the row's
// items) tested against the frustum and cone alone; they are drawn into
// GPU culling's occluder depth and the pyramid is built again; the late
// phase tests every cluster against that pyramid, records which are
// visible for the next frame and appends those the early phase did not
// draw. Without a pyramid one phase draws what the frustum and cone keep
// and records it.

struct Params {
  // The camera's (the pyramid's projection).
  view_proj: mat4x4<f32>,
  // x: the scene's LOD bias; y: the first of `views`' rows holding the
  // levels callers picked, four sorted slots a row (255: none), when z
  // is 1; w: the camera's near plane.
  lod: vec4<f32>,
  // xy: the screen's size, z: the pyramid's levels (0: no occlusion
  // test), w: the batches.
  sizes: vec4<u32>,
  // x: the phase (0 the only one, 1 early, 2 late), y: last frame's half
  // of `vis` (its first word), z: this frame's.
  phase: vec4<u32>,
  // The soft list (meshlet_gpu.rs `plan`): x its first item, y its
  // capacity in clusters (0: none), z its count's word in `draws`; w the
  // word that sums the camera's drawn triangles (FrameStats::triangles).
  soft: vec4<u32>,
};

// a: mesh slot, first instance (its sorted slot), instances, first work
// item. b: first workgroup, flags (1: the cone may cull; 2: soft, its
// small clusters go to the soft list; from bit 2 its side), view, the
// mesh's most clusters at any level (a cluster slot's stride).
struct BatchRow {
  a: vec4<u32>,
  b: vec4<u32>,
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> instances: array<vec4<f32>>;
// Per view VIEW_ROWS rows, view 0 the camera's: VIEW_PLANES planes
// (frame.rs `Frustum`: a cascade's box and its swept camera slice; zero
// rows past them never cull), then (the eye, pixels per unit of length at
// distance 1: perspective, the view's height over 2 tan(fov / 2);
// orthographic, its height over the view's height in units), then (1 for
// a perspective view, 0 orthographic; 0, 0, 0). After every view's rows,
// the picked levels (`params.lod`).
@group(0) @binding(2) var<storage, read> views: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read> batches: array<BatchRow>;
// Per mesh slot 10 x vec4: its sphere, (levels, its first vertex in the
// vertex pool, a foliage mesh's deformation bound as bits (meshlet_gpu.rs:
// this frame's wind and the last drawn's, at sway 1; 0 for any other), 0),
// then per level
// (first cluster, clusters, error as bits, 0).
@group(0) @binding(4) var<storage, read> meshes: array<vec4<u32>>;
// Per cluster 3 x vec4: (first index, triangles, level, its foliage region
// plus one: meshlet_sway.wgsl), sphere, cone.
@group(0) @binding(5) var<storage, read> clusters: array<vec4<u32>>;
// At each stable cluster slot: (cluster + 1, eligible for software raster).
// Cleared once per frame, retained between the early and late phases.
@group(0) @binding(6) var<storage, read_write> selected: array<vec2<u32>>;
// Per batch a draw: (vertices, instances, first vertex, first instance).
@group(0) @binding(7) var<storage, read_write> draws: array<atomic<u32>>;
@group(0) @binding(8) var pyramid: texture_2d<f32>;
// Whether each cluster slot was visible: two halves, last frame's and
// this frame's (phase.y, phase.z).
@group(0) @binding(9) var<storage, read_write> vis: array<u32>;

const WG: u32 = 64u;
// meshlet_gpu.rs VIEW_PLANES (cull_gpu.rs MAX_PLANES).
const VIEW_PLANES: u32 = 12u;
const VIEW_ROWS: u32 = VIEW_PLANES + 2u;
const MESH_ROWS: u32 = 10u;
// meshlet.rs SOFT_PIXELS.
const SOFT_PIXELS: f32 = 24.0;

// meshlet.rs `outside`.
fn outside(view: u32, c: vec3<f32>, r: f32) -> bool {
  for (var i = 0u; i < VIEW_PLANES; i++) {
    let p = views[view * VIEW_ROWS + i];
    if (!plane_keeps(p, c, r * length(p.xyz))) { return true; }
  }
  return false;
}

// meshlet.rs `occluded`: the box about the sphere behind the pyramid
// everywhere it covers (cull_hiz.wgsl).
fn occluded(c: vec3<f32>, r: f32) -> bool {
  let box = hiz_sphere(params.view_proj, c, r, vec2<f32>(params.sizes.xy));
  return hiz_hidden(box, params.sizes.xy, params.sizes.z);
}

@compute @workgroup_size(64)
fn cull_clusters(@builtin(workgroup_id) wid: vec3<u32>,
                 @builtin(num_workgroups) nw: vec3<u32>,
                 @builtin(local_invocation_index) li: u32) {
  let count = params.sizes.w;
  if (count == 0u) { return; }
  let w = wid.x + wid.y * nw.x;
  // The batch whose workgroups hold `w` (their firsts ascend).
  var lo = 0u;
  var hi = count - 1u;
  loop {
    if (lo >= hi) { break; }
    let mid = (lo + hi + 1u) / 2u;
    if (batches[mid].b.x <= w) { lo = mid; } else { hi = mid - 1u; }
  }
  let row = batches[lo];
  let local = w - row.b.x;
  if (local >= row.a.z) { return; }
  let instance = row.a.y + local;
  let t0 = instances[instance * 4u];
  let t1 = instances[instance * 4u + 1u];
  let t2 = instances[instance * 4u + 2u];
  let c0 = t0.xyz;
  let c1 = vec3<f32>(t0.w, t1.xy);
  let c2 = vec3<f32>(t1.zw, t2.x);
  let m = mat4x4<f32>(vec4<f32>(c0, 0.0), vec4<f32>(c1, 0.0), vec4<f32>(c2, 0.0), vec4<f32>(t2.yzw, 1.0));
  // meshlet.rs `transform_scale`.
  let la = length(c0);
  let lb = length(c1);
  let lc = length(c2);
  let scale = max(la, max(lb, lc));
  let even = abs(la - lb) <= 1e-4 * scale && abs(la - lc) <= 1e-4 * scale
    && dot(c0, cross(c1, c2)) >= 0.0;
  let mesh = row.a.x * MESH_ROWS;
  let sphere = bitcast<vec4<f32>>(meshes[mesh]);
  let center = (m * vec4<f32>(sphere.xyz, 1.0)).xyz;
  let radius = sphere.w * scale;
  // A swaying mesh's spheres widen by how far its vertices can move (the
  // sway decoded as meshlet_foliage.wgsl `foliage_sway`): its
  // bound at sway 1 times the instance's sway (the bound grows less than
  // in proportion, so at least 1 is conservative), in the world.
  let sway = instances[instance * 4u + 3u].x;
  let grow = bitcast<f32>(meshes[mesh + 1u].z) * max(1.0, max(select(-sway - 1.0, sway, sway >= 0.0), 0.0)) * scale;
  let view = row.b.z;
  let seen = views[view * VIEW_ROWS + VIEW_PLANES];
  let eye = seen.xyz;
  // lod.rs `pixels_per_unit` at the sphere's near side, then `select`.
  let perspective = views[view * VIEW_ROWS + VIEW_PLANES + 1u].x > 0.5;
  var per_unit = seen.w;
  var inside = false;
  if (perspective) {
    let d = length(center - eye) - radius;
    if (d <= 1e-6) { inside = true; } else { per_unit = seen.w / d; }
  }
  let levels = meshes[mesh + 1u].x;
  let bias = params.lod.x;
  var level = 0u;
  var picked = 255u;
  if (params.lod.z > 0.5) {
    picked = u32(views[u32(params.lod.y) + instance / 4u][instance % 4u]);
  }
  if (picked < 255u) {
    level = min(picked, levels - 1u);
  } else if (!inside && bias > 0.0) {
    var error = 0.0;
    for (var k = 1u; k < levels; k++) {
      if (lod_holds(&error, bitcast<f32>(meshes[mesh + 2u + k].z), per_unit, scale, bias)) { level = k; } else { break; }
    }
  }
  let phase = params.phase.x;
  // The late phase is the camera's.
  if (phase == 2u && view != 0u) { return; }
  let occlusion = phase == 2u;
  if (outside(view, center, radius + grow) || (occlusion && occluded(center, radius + grow))) { return; }
  // The instance's cluster slots.
  let slots = row.a.w + local * row.b.w;
  let lv = meshes[mesh + 2u + level];
  // A swaying mesh's triangles turn: its cones do not hold.
  let cone = (row.b.y & 1u) != 0u && even && perspective && grow <= 0.0;
  for (var i = li; i < lv.y; i += WG) {
    let ci = lv.x + i;
    let s = bitcast<vec4<f32>>(clusters[ci * 3u + 1u]);
    let wc = (m * vec4<f32>(s.xyz, 1.0)).xyz;
    let wr = s.w * scale + grow;
    if (outside(view, wc, wr)) { continue; }
    if (cone) {
      // meshlet.rs `backfacing`.
      let k = bitcast<vec4<f32>>(clusters[ci * 3u + 2u]);
      if (k.w < 1.0) {
        let axis = normalize((m * vec4<f32>(k.xyz, 0.0)).xyz);
        let to = wc - eye;
        if (dot(to, axis) >= k.w * length(to) + wr) { continue; }
      }
    }
    let slot = slots + i;
    if (view == 0u) {
      let before = vis[params.phase.y + slot] != 0u;
      if (phase == 1u && !before) { continue; }
      if (phase == 2u) {
        if (occluded(wc, wr)) { continue; }
        vis[params.phase.z + slot] = 1u;
        if (before) { continue; }
      }
      if (phase == 0u) { vis[params.phase.z + slot] = 1u; }
    }
    // Drawn by the camera, here or in the visibility buffer: counted once.
    if (view == 0u) { atomicAdd(&draws[params.soft.w], clusters[ci * 3u].y); }
    // meshlet.rs `soft`: a small cluster past the near plane goes to the
    // visibility buffer. Compaction assigns its capacity by stable rank.
    var soft = 0u;
    if ((row.b.y & 2u) != 0u && perspective) {
      let depth = (params.view_proj * vec4<f32>(wc, 1.0)).w;
      let d = depth - wr;
      if (d > params.lod.w && 2.0 * wr * seen.w <= SOFT_PIXELS * d) {
        soft = 1u;
      }
    }
    selected[slot] = vec2<u32>(ci + 1u, soft);
  }
}
