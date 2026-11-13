// Scatter culling (scatter_cull.rs): each chunk's copies tested one by one
// before the passes that draw them, the survivors compacted, in order, into
// the frame's copy list and counted into the chunk's indirect draws.
//
// A copy goes when it stands past the scatter's distance along x and z or
// when the scatter's density map is below the copy's own value at its
// place. What is left is the chunk's second set, which the passes off the
// main camera draw (shadows, reflections, probe faces: a copy the camera
// does not see may still cast). The main camera's set, the first, also
// drops a copy whose bounding sphere (the mesh's radius times its scale,
// grown by the furthest the wind and the trample move it) is outside the
// view, or that the frame's depth pyramid (cull_gpu.rs, its occluders'
// farthest depth) covers everywhere it would draw.
//
// Each set is a region of the list per level of detail of the mesh, then
// one for the copies that draw as the card (the impostor), each with a
// draw of its own. A copy's level is its distance against the level's
// error (lod.rs: the level whose error is under the set's pixels on
// screen); a copy past the impostor distance is in the card's region only,
// so no pass runs the mesh's vertices for it, and one in the crossfade
// before it is in both.
//
// One workgroup per chunk of at most 1024 copies (scatter.rs CHUNK_CELLS
// squared): each lane takes four consecutive copies, a prefix sum over the
// lanes gives each its place in each region, so the lists keep the chunk's
// order on every run.

// scatter_cull.rs ViewGpu.
struct CullView {
  view_proj: mat4x4<f32>,       // the jittered view the pyramid was drawn with
  planes: array<vec4<f32>, 6>,  // inside: dot(xyz, p) + w >= 0
  camera: vec4<f32>,            // xyz: the camera (frame coordinates)
  sizes: vec4<u32>,             // x, y: the pyramid's base (the frame's pixels), z: its levels (0: no occlusion)
  lod: vec4<f32>,               // metres of distance per metre of a level's error past which it draws: x for the main camera, y for the second set
};

// scatter_cull.rs ItemGpu: one chunk.
struct CullItem {
  to_terrain: mat4x4<f32>,  // the terrain's space from the frame's (its density map's uv)
  item: vec4<u32>,          // x: the chunk's first place in the list, y: its copies, z: its draws' row, w: 1 with a density map
  mesh: vec4<u32>,          // x: the mesh's index count, y: 1 when placed on the GPU (scatter_place.wgsl: its count follows its copies), z, w: its second and third levels' index counts
  shape: vec4<f32>,         // x: the mesh's radius about its origin, y: how far a copy moves at most (m), z: the scatter's distance (m), w: the distance past which a copy draws as the card (0: never)
  extent: vec4<f32>,        // xy: the terrain's size (its space), zw: the second and third levels' errors (m, at a copy's scale of 1)
  bins: vec4<u32>,          // x: the places each region holds, y: the mesh's levels (1 to 3), z: 1 with a card region, w: the sets (1: the main camera's only)
  bands: vec4<u32>,         // x: the first level the main camera's set has a region for, y: how many it has (0: no copy draws its mesh), zw: the second set's
};

@group(0) @binding(0) var<uniform> view: CullView;
@group(0) @binding(1) var<uniform> item: CullItem;
@group(0) @binding(2) var<storage, read_write> list: array<vec4<f32>>;
// Per chunk two sets of 20 words (CULL_SET_WORDS): three indexed indirect
// draws, one per level (index count, instances, first index, base vertex,
// first instance), then a plain one, the card's (vertices, instances,
// first vertex, first instance), then a pad.
@group(0) @binding(3) var<storage, read_write> draws: array<u32>;
@group(0) @binding(4) var pyramid: texture_2d<f32>;
// The chunk's copies (instances.rs GpuTransform, four vectors each).
@group(1) @binding(0) var<storage, read> copies: array<vec4<f32>>;
@group(2) @binding(0) var density_map: texture_2d<f32>;
@group(2) @binding(1) var density_sampler: sampler;

const CULL_LANES: u32 = 256u;
const CULL_PER_LANE: u32 = 4u;
const CULL_SET_WORDS: u32 = 20u;

fn cull_outside(c: vec3<f32>, r: f32) -> bool {
  for (var i = 0; i < 6; i++) {
    if (dot(view.planes[i].xyz, c) + view.planes[i].w < -r) { return true; }
  }
  return false;
}

// meshlet_cull.wgsl `occluded`: the box about the sphere behind the
// pyramid everywhere it covers.
fn cull_occluded(c: vec3<f32>, r: f32) -> bool {
  if (view.sizes.z == 0u) { return false; }
  let size = vec2<f32>(view.sizes.xy);
  var lo = vec2<f32>(1e30);
  var hi = vec2<f32>(-1e30);
  var near = 1e30;
  for (var i = 0u; i < 8u; i++) {
    let corner = c + r * vec3<f32>(
      select(-1.0, 1.0, (i & 1u) != 0u),
      select(-1.0, 1.0, (i & 2u) != 0u),
      select(-1.0, 1.0, (i & 4u) != 0u));
    let clip = view.view_proj * vec4<f32>(corner, 1.0);
    if (clip.w <= 1e-6) { return false; }
    let ndc = clip.xyz / clip.w;
    let px = vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5) * size;
    lo = min(lo, px);
    hi = max(hi, px);
    near = min(near, ndc.z);
  }
  if (near <= 0.0) { return false; }
  let x0 = u32(clamp(floor(lo.x) - 1.0, 0.0, size.x - 1.0));
  let y0 = u32(clamp(floor(lo.y) - 1.0, 0.0, size.y - 1.0));
  let x1 = u32(clamp(floor(hi.x) + 1.0, 0.0, size.x - 1.0));
  let y1 = u32(clamp(floor(hi.y) + 1.0, 0.0, size.y - 1.0));
  let texels = max(x1 - x0, y1 - y0) + 1u;
  var level = 0u;
  while ((1u << level) < texels) { level++; }
  if (level >= view.sizes.z) { return false; }
  let dims = max(vec2<u32>(1u), view.sizes.xy >> vec2<u32>(level));
  let t0 = min(vec2<u32>(x0, y0) >> vec2<u32>(level), dims - 1u);
  let t1 = min(vec2<u32>(x1, y1) >> vec2<u32>(level), dims - 1u);
  var farthest = 0.0;
  for (var y = t0.y; y <= t1.y; y++) {
    for (var x = t0.x; x <= t1.x; x++) {
      farthest = max(farthest, textureLoad(pyramid, vec2<u32>(x, y), level).r);
    }
  }
  return near > farthest + 1e-6;
}

// The regions copy `k` of the chunk is listed in, a bit each: x in the
// main camera's set, y in the second.
fn cull_regions(k: u32) -> vec2<u32> {
  let v0 = copies[k * 4u];
  let v2 = copies[k * 4u + 2u];
  let at = v2.yzw;
  let scale = length(v0.xyz);
  let flat = at.xz - view.camera.xz;
  let d2 = dot(flat, flat);
  if (d2 > item.shape.z * item.shape.z) { return vec2<u32>(0u); }
  if (item.item.w == 1u) {
    let local = (item.to_terrain * vec4<f32>(at, 1.0)).xyz;
    let uv = local.xz / item.extent.xy + 0.5;
    let q = vec2<i32>(round(local.xz * 64.0));
    let own = hash_to_unit(hash_lowbias(bitcast<u32>(q.x) ^ hash_lowbias(bitcast<u32>(q.y) + 0x9e3779b9u)));
    if (own >= textureSampleLevel(density_map, density_sampler, uv, 0.0).r) { return vec2<u32>(0u); }
  }
  // Its mesh nearer than the impostor distance, its card from where the
  // crossfade starts (scatter_common.wgsl scatter_near).
  let d = sqrt(d2);
  let far = item.shape.w;
  let card = select(0u, 1u, item.bins.z == 1u && far > 0.0 && d > 0.9 * far);
  var sets = vec2<u32>(0u);
  for (var s = 0u; s < 2u; s++) {
    // The set's regions: the levels the chunk's copies can draw, as the
    // CPU bounded them (scatter_cull.rs band), then the card's.
    let first = item.bands[s * 2u];
    let count = item.bands[s * 2u + 1u];
    sets[s] = card << count;
    if (count > 0u && (far <= 0.0 || d < far)) {
      var level = 0u;
      for (var l = 1u; l < item.bins.y; l++) {
        if (d > item.extent[l + 1u] * scale * view.lod[s]) { level = l; }
      }
      sets[s] |= 1u << (clamp(level, first, first + count - 1u) - first);
    }
  }
  let r = item.shape.x * scale + item.shape.y;
  if (cull_outside(at, r) || cull_occluded(at, r)) { sets.x = 0u; }
  if (item.bins.w < 2u) { sets.y = 0u; }
  return sets;
}

// Per lane, the copies it lists in each region of the two sets.
var<workgroup> lane_counts: array<array<vec4<u32>, 2>, CULL_LANES>;

fn cull_count(mask: u32) -> vec4<u32> {
  return (vec4<u32>(mask) >> vec4<u32>(0u, 1u, 2u, 3u)) & vec4<u32>(1u);
}

@compute @workgroup_size(256)
fn cull(@builtin(local_invocation_index) lane: u32) {
  var n = item.item.y;
  if (item.mesh.y == 1u) {
    // scatter_place.wgsl PLACE_TAIL: its indexed draw's instances.
    n = min(bitcast<u32>(copies[4096u].y), n);
  }
  var kept: array<vec2<u32>, CULL_PER_LANE>;
  var mine: array<vec4<u32>, 2>;
  for (var j = 0u; j < CULL_PER_LANE; j++) {
    let k = lane * CULL_PER_LANE + j;
    kept[j] = vec2<u32>(0u);
    if (k < n) { kept[j] = cull_regions(k); }
    mine[0] += cull_count(kept[j].x);
    mine[1] += cull_count(kept[j].y);
  }
  lane_counts[lane] = mine;
  workgroupBarrier();
  // An inclusive scan over the lanes (Hillis and Steele).
  for (var step = 1u; step < CULL_LANES; step <<= 1u) {
    var add: array<vec4<u32>, 2>;
    if (lane >= step) { add = lane_counts[lane - step]; }
    workgroupBarrier();
    lane_counts[lane][0] += add[0];
    lane_counts[lane][1] += add[1];
    workgroupBarrier();
  }
  // The regions before the set's own: the main camera's set's.
  var before = 0u;
  for (var s = 0u; s < item.bins.w; s++) {
    let regions = item.bands[s * 2u + 1u] + item.bins.z;
    var at = lane_counts[lane][s] - mine[s];
    for (var j = 0u; j < CULL_PER_LANE; j++) {
      let k = lane * CULL_PER_LANE + j;
      for (var r = 0u; r < regions; r++) {
        if ((kept[j][s] & (1u << r)) == 0u) { continue; }
        let to = item.item.x + (before + r) * item.bins.x + at[r];
        for (var c = 0u; c < 4u; c++) {
          list[to * 4u + c] = copies[k * 4u + c];
        }
        at[r] += 1u;
      }
    }
    before += regions;
  }
  if (lane == CULL_LANES - 1u) {
    let counts = vec3<u32>(item.mesh.x, item.mesh.z, item.mesh.w);
    for (var s = 0u; s < 2u; s++) {
      let total = lane_counts[lane][s];
      let row = item.item.z * 2u * CULL_SET_WORDS + s * CULL_SET_WORDS;
      let first = item.bands[s * 2u];
      let count = item.bands[s * 2u + 1u];
      for (var l = 0u; l < 3u; l++) {
        let listed = l >= first && l < first + count && s < item.bins.w;
        draws[row + l * 5u] = counts[l];
        draws[row + l * 5u + 1u] = select(0u, total[min(max(l, first) - first, 3u)], listed);
        draws[row + l * 5u + 2u] = 0u;
        draws[row + l * 5u + 3u] = 0u;
        draws[row + l * 5u + 4u] = 0u;
      }
      draws[row + 15u] = 6u;
      draws[row + 16u] = select(0u, total[count], item.bins.z == 1u && s < item.bins.w);
      draws[row + 17u] = 0u;
      draws[row + 18u] = 0u;
      draws[row + 19u] = 0u;
    }
  }
}
