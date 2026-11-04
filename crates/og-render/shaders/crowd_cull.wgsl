// Crowd culling (crowd.rs, crowd_gpu.rs): per view row, every crowd
// member's clip box is tested against the row's planes (the camera's also
// against the occluders' depth pyramid, cull_hiz.wgsl), its level is
// picked from the camera, and the visible members are compacted, level by
// level and in member order, into the row's region of skinned instance
// records (skin.rs `SkinnedInstanceGpu`). Each (crowd, part, level) gets an
// indexed indirect draw whose first instance is where its members start in
// the region. No atomics decide an order: a deterministic prefix sum over
// one-hot level vectors does.
//
// One workgroup row (workgroup_id.y) per view row:
//   visibility  writes each member's one-hot level (zero: not drawn) and
//               each workgroup's sum
//   scan_blocks scans the workgroup sums (one workgroup per row)
//   prefix      turns the one-hots into the exclusive prefix (N + 1 entries)
//   scatter     writes the visible members' records and the draws
//   totals      sums the frame's counts for its stats (cull_gpu.rs
//               `CROWD_COUNT_WORDS`) after the draws

const WG: u32 = 256u;

// Row kinds (cull_gpu.rs `PassKind`).
const SHADOW: u32 = 0u;
const OCCLUDERS: u32 = 2u;
const MAIN: u32 = 3u;

// The floats of a member (crowd.rs `MemberGpu`) and of a skinned record.
const MEMBER_FLOATS: u32 = 20u;
const RECORD_FLOATS: u32 = 28u;

struct Params {
  // x: members (N), y: crowds, z: rows, w: draws
  counts: vec4<u32>,
  // x: workgroups per row (blocks), y: pyramid levels (0: no occlusion
  // test), zw: the screen's size
  sizes: vec4<u32>,
  // xyz: the camera's position, w: the view's height in pixels
  camera: vec4<f32>,
  // x: 1 for a perspective camera, y: the tangent of half its vertical
  // field of view (orthographic: the view's height in units), z: 0, w: the
  // first member's previous transform's palette slot plus one (0: none)
  lod: vec4<f32>,
  view_proj: mat4x4<f32>,
};

struct Row {
  planes: array<vec4<f32>, 12>,
  // x: kind, y: planes, z: the row's layer mask
  info: vec4<u32>,
};

struct CrowdGpu {
  // x: first member, y: members, z: levels, w: first draw
  members: vec4<u32>,
  // x: parts, y: casts shadows, z: may be occluded, w: first clip box
  info: vec4<u32>,
  // x: its layer mask
  layers: vec4<u32>,
  errors: vec4<f32>,
  sphere: vec4<f32>,
  // x: the scene's lod bias times its own
  lod: vec4<f32>,
};

struct ClipGpu {
  // x: first slot in the baked buffer, y: joints, z: frames
  info: vec4<u32>,
  lo: vec4<f32>,
  hi: vec4<f32>,
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> members: array<f32>;
@group(0) @binding(2) var<storage, read> crowds: array<CrowdGpu>;
@group(0) @binding(3) var<storage, read> clips: array<ClipGpu>;
// Per draw: its mesh's index count.
@group(0) @binding(4) var<storage, read> draws: array<u32>;
@group(0) @binding(5) var<storage, read> rows: array<Row>;
@group(0) @binding(6) var pyramid: texture_2d<f32>;
// Per row: N + 1 prefix entries, then `blocks` workgroup sums, then
// `blocks` workgroup counts of the hidden (`visibility`'s `hidden`).
@group(0) @binding(7) var<storage, read_write> scan: array<vec4<u32>>;
// Row r's records from r * N.
@group(0) @binding(8) var<storage, read_write> records: array<f32>;
// Per (row, draw) five words of DrawIndexedIndirect, then the frame's
// counts (`totals`).
@group(0) @binding(9) var<storage, read_write> args: array<u32>;

var<workgroup> shared_sum: array<vec4<u32>, 256>;

fn n_members() -> u32 { return params.counts.x; }
fn prefix_at(r: u32) -> u32 { return r * (n_members() + 1u + 2u * params.sizes.x); }
fn blocks_at(r: u32) -> u32 { return prefix_at(r) + n_members() + 1u; }
fn hidden_at(r: u32) -> u32 { return blocks_at(r) + params.sizes.x; }

fn member_matrix(k: u32) -> mat4x4<f32> {
  let a = k * MEMBER_FLOATS;
  return instance_matrix(
    vec3<f32>(members[a], members[a + 1u], members[a + 2u]),
    vec3<f32>(members[a + 3u], members[a + 4u], members[a + 5u]),
    vec3<f32>(members[a + 6u], members[a + 7u], members[a + 8u]),
    vec3<f32>(members[a + 9u], members[a + 10u], members[a + 11u]));
}

// The crowd holding member k (crowds cover 0..N in order; an empty one
// shares its first member with the next).
fn crowd_of(k: u32) -> u32 {
  var lo = 0u;
  var hi = params.counts.y - 1u;
  loop {
    if (lo >= hi) { break; }
    let mid = (lo + hi + 1u) / 2u;
    if (crowds[mid].members.x <= k) { lo = mid; } else { hi = mid - 1u; }
  }
  return lo;
}

// crowd.rs `member_level`: the coarsest level whose error, projected at
// the member's distance, stays under a pixel (lod.rs `select`, no
// hysteresis).
fn member_level(c: CrowdGpu, m: mat4x4<f32>) -> u32 {
  let levels = c.members.z;
  let bias = c.lod.x;
  if (levels < 2u || bias <= 0.0) { return 0u; }
  let scale = max(length(m[0].xyz), max(length(m[1].xyz), length(m[2].xyz)));
  let centre = (m * vec4<f32>(c.sphere.xyz, 1.0)).xyz;
  var per_unit: f32;
  if (params.lod.x > 0.5) {
    let d = length(centre - params.camera.xyz) - c.sphere.w * scale;
    if (d <= 1e-6) { return 0u; }
    per_unit = params.camera.w / (2.0 * d * max(params.lod.y, 1e-6));
  } else {
    per_unit = params.camera.w / max(params.lod.y, 1e-6);
  }
  var level = 0u;
  for (var l = 1u; l < levels; l++) {
    if (c.errors[l] * per_unit * scale * bias < 1.0) { level = l; } else { break; }
  }
  return level;
}

fn one_hot(l: u32) -> vec4<u32> {
  return vec4<u32>(vec4<bool>(l == 0u, l == 1u, l == 2u, l == 3u));
}

@compute @workgroup_size(256)
fn visibility(@builtin(global_invocation_id) gid: vec3<u32>,
              @builtin(local_invocation_index) li: u32,
              @builtin(workgroup_id) wid: vec3<u32>) {
  let r = wid.y;
  let k = gid.x;
  var v = vec4<u32>(0u);
  // The camera row's hidden records, as GPU culling counts its instances
  // (cull_gpu.rs `GpuCounts`): outside its planes, occluded, and the
  // occluded ones' triangles.
  var hidden = vec4<u32>(0u);
  if (k < n_members()) {
    let c = crowds[crowd_of(k)];
    let row = rows[r];
    let kind = row.info.x;
    let m = member_matrix(k);
    let clip = clips[c.info.w + u32(members[k * MEMBER_FLOATS + 12u])];
    let center = (clip.lo.xyz + clip.hi.xyz) * 0.5;
    let extent = max(clip.hi.xyz - clip.lo.xyz, vec3<f32>(0.0)) * 0.5;
    let on_layers = (row.info.z & c.layers.x) != 0u;
    var inside = kind != OCCLUDERS && on_layers && (kind != SHADOW || c.info.y != 0u);
    // The world box around the member's (skin.rs `moved_box`), as the CPU
    // path culls its records (Frustum::intersects_box): the two paths cull
    // the same members.
    let wc = (m * vec4<f32>(center, 1.0)).xyz;
    let we = abs(m[0].xyz) * extent.x + abs(m[1].xyz) * extent.y + abs(m[2].xyz) * extent.z;
    for (var p = 0u; p < row.info.y && inside; p++) {
      let plane = row.planes[p];
      // The box's corner farthest inside the plane.
      let reach = dot(abs(plane.xyz), we) * (1.0 + 1e-5);
      inside = dot(plane.xyz, wc) + plane.w >= -reach;
    }
    let parts = c.info.x;
    if (kind == MAIN && on_layers && !inside) { hidden.x = parts; }
    if (inside && kind == MAIN && params.sizes.y > 0u && c.info.z != 0u
        && occluded(m, center, extent * (1.0 + 1e-5) + vec3<f32>(1e-6))) {
      inside = false;
      hidden.y = parts;
      let l = member_level(c, m);
      for (var p = 0u; p < parts; p++) {
        hidden.z += draws[c.members.w + l * parts + p] / 3u;
      }
    }
    if (inside) { v = one_hot(member_level(c, m)); }
    scan[prefix_at(r) + k] = v;
  }
  shared_sum[li] = v;
  workgroupBarrier();
  for (var s = WG / 2u; s > 0u; s >>= 1u) {
    if (li < s) { shared_sum[li] += shared_sum[li + s]; }
    workgroupBarrier();
  }
  if (li == 0u) { scan[blocks_at(r) + wid.x] = shared_sum[0]; }
  workgroupBarrier();
  shared_sum[li] = hidden;
  workgroupBarrier();
  for (var s = WG / 2u; s > 0u; s >>= 1u) {
    if (li < s) { shared_sum[li] += shared_sum[li + s]; }
    workgroupBarrier();
  }
  if (li == 0u) { scan[hidden_at(r) + wid.x] = shared_sum[0]; }
}

// An inclusive scan of `shared_sum` (Hillis-Steele).
fn scan_shared(li: u32) {
  for (var s = 1u; s < WG; s <<= 1u) {
    var v = shared_sum[li];
    if (li >= s) { v += shared_sum[li - s]; }
    workgroupBarrier();
    shared_sum[li] = v;
    workgroupBarrier();
  }
}

@compute @workgroup_size(256)
fn scan_blocks(@builtin(local_invocation_index) li: u32,
               @builtin(workgroup_id) wid: vec3<u32>) {
  let at = blocks_at(wid.y);
  var carry = vec4<u32>(0u);
  for (var base = 0u; base < params.sizes.x; base += WG) {
    let i = base + li;
    var v = vec4<u32>(0u);
    if (i < params.sizes.x) { v = scan[at + i]; }
    shared_sum[li] = v;
    workgroupBarrier();
    scan_shared(li);
    if (i < params.sizes.x) { scan[at + i] = carry + shared_sum[li] - v; }
    carry += shared_sum[WG - 1u];
    workgroupBarrier();
  }
}

@compute @workgroup_size(256)
fn prefix(@builtin(global_invocation_id) gid: vec3<u32>,
          @builtin(local_invocation_index) li: u32,
          @builtin(workgroup_id) wid: vec3<u32>) {
  let r = wid.y;
  let n = n_members();
  let k = gid.x;
  var v = vec4<u32>(0u);
  if (k < n) { v = scan[prefix_at(r) + k]; }
  shared_sum[li] = v;
  workgroupBarrier();
  scan_shared(li);
  let before = scan[blocks_at(r) + wid.x];
  if (k < n) {
    scan[prefix_at(r) + k] = before + shared_sum[li] - v;
    if (k == n - 1u) { scan[prefix_at(r) + n] = before + shared_sum[li]; }
  }
}

// Where clip `clip`'s fractional frame `frame` starts in the baked buffer,
// and how far towards the next frame it is (skin.rs `BakedClip::at`).
fn frame_slot(c: CrowdGpu, clip: f32, frame: f32) -> vec2<f32> {
  let info = clips[c.info.w + u32(clip)].info;
  let last = max(info.z, 2u) - 2u;
  // A frame that is not finite plays from the start.
  let f = select(0.0, max(frame, 0.0), frame == frame && abs(frame) < 3.0e38);
  let k = min(u32(floor(f)), last);
  return vec2<f32>(f32(info.x + k * info.y), clamp(f - f32(k), 0.0, 1.0));
}

@compute @workgroup_size(256)
fn scatter(@builtin(global_invocation_id) gid: vec3<u32>,
           @builtin(workgroup_id) wid: vec3<u32>) {
  let r = wid.y;
  let n = n_members();
  let k = gid.x;
  if (k >= n) { return; }
  let at = prefix_at(r);
  // The row's region holds level 0's members, then level 1's, ...: level
  // l starts at the members of the levels before it.
  let total = scan[at + n];
  let starts = vec4<u32>(0u, total.x, total.x + total.y, total.x + total.y + total.z);
  let c = crowds[crowd_of(k)];
  let here = scan[at + k];
  let mine = scan[at + k + 1u] - here;
  if (any(mine != vec4<u32>(0u))) {
    let l = dot(mine, vec4<u32>(0u, 1u, 2u, 3u));
    let slot = r * n + starts[l] + here[l];
    let a = k * MEMBER_FLOATS;
    let o = slot * RECORD_FLOATS;
    for (var j = 0u; j < 12u; j++) { records[o + j] = members[a + j]; }
    let clip = clips[c.info.w + u32(members[a + 12u])];
    let now = frame_slot(c, members[a + 12u], members[a + 13u]);
    let then = frame_slot(c, members[a + 14u], members[a + 15u]);
    records[o + 12u] = now.x;
    records[o + 13u] = f32(clip.info.y);
    records[o + 14u] = 0.0;
    records[o + 15u] = -1.0 - now.y;
    for (var j = 16u; j < 20u; j++) { records[o + j] = 0.0; }
    var previous = 0.0;
    if (params.lod.w > 0.5) { previous = params.lod.w + f32(k); }
    records[o + 20u] = previous;
    records[o + 21u] = then.x;
    records[o + 22u] = then.y;
    records[o + 23u] = 0.0;
    for (var j = 0u; j < 4u; j++) { records[o + 24u + j] = members[a + 16u + j]; }
  }
  if (k == c.members.x) {
    // The crowd's draws: each level's members, for every part.
    let first = here;
    let last = scan[at + c.members.x + c.members.y];
    let parts = c.info.x;
    for (var l = 0u; l < c.members.z; l++) {
      for (var p = 0u; p < parts; p++) {
        let d = c.members.w + l * parts + p;
        let w = (r * params.counts.w + d) * 5u;
        args[w] = draws[d];
        args[w + 1u] = last[l] - first[l];
        args[w + 2u] = 0u;
        args[w + 3u] = 0u;
        args[w + 4u] = starts[l] + first[l];
      }
    }
  }
}

// The frame's counts after `scatter`, at the words after the draws: the
// camera row's records outside its planes, occluded and the occluded ones'
// triangles, its drawn triangles, and the shadow rows' drawn records. One
// invocation: the rows' draws and the camera row's workgroup counts are
// few. Read back with GPU culling's (cull_gpu.rs `CROWD_COUNT_WORDS`).
@compute @workgroup_size(1)
fn totals() {
  var out = array<u32, 5>(0u, 0u, 0u, 0u, 0u);
  let d = params.counts.w;
  for (var r = 0u; r < params.counts.z; r++) {
    let kind = rows[r].info.x;
    if (kind == MAIN) {
      for (var b = 0u; b < params.sizes.x; b++) {
        let h = scan[hidden_at(r) + b];
        out[0] += h.x;
        out[1] += h.y;
        out[2] += h.z;
      }
    }
    for (var i = 0u; i < d; i++) {
      let count = args[(r * d + i) * 5u + 1u];
      if (kind == MAIN) { out[3] += count * (draws[i] / 3u); }
      if (kind == SHADOW) { out[4] += count; }
    }
  }
  let at = params.counts.z * d * 5u;
  for (var j = 0u; j < 5u; j++) { args[at + j] = out[j]; }
}
