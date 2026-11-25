// Compact 8 B/px visibility buffer software rasteriser and 8x8 tile classifier
// (meshlet_gpu/vis.rs):
//   - `raster_depth` / `raster_id`: dispatched indirectly for only the active
//     soft clusters, using incremental edge stepping and an early depth pre-test
//     before `atomicMax` into `vbuf` (8 B/px: depth[wh] + id[wh]).
//   - `classify_count` / `classify_prefix` / `classify_emit`: bins every active
//     8x8 screen tile into compact per-batch tile lists (`tile_list`) and fills
//     per-batch `DrawIndirect` args (`tile_draws`), eliminating the old 148 B/px
//     `attrs` G-buffer, the full-screen `setup` pass, and full-screen overdraw.
// After instance.wgsl and meshlet_sway.wgsl.

struct RasterParams {
  // The camera's (jittered, as the hardware draws it).
  view_proj: mat4x4<f32>,
  // x: width, y: height, z: soft_rows, w: max tile_list capacity in u32 words.
  size: vec4<u32>,
  // x: soft list's first item, y: capacity in clusters, z: count (copied from
  // compaction), w: unused.
  soft: vec4<u32>,
  // Retro 3D (frame.wgsl `stylise`, stylise.rs `Retro3d::uniform`): the
  // snap grid in cells, the affine share, the depth bits.
  stylise: vec4<f32>,
  // The foliage's wind this frame and the last drawn (meshlet_sway.wgsl).
  wind: array<FoliageWind, 2>,
  // The leaves' year (foliage.wgsl `LeafYear`).
  leaf_year: LeafYear,
};

@group(0) @binding(0) var<uniform> raster: RasterParams;
// The sorted instances (instances.rs `GpuTransform`, 4 x vec4).
@group(0) @binding(1) var<storage, read> instances: array<vec4<f32>>;
// The soft list: per cluster (instance, cluster), (row | side << 28,
// the mesh's first vertex in the vertex pool).
@group(0) @binding(2) var<storage, read> items: array<vec2<u32>>;
@group(0) @binding(3) var<storage, read> clusters: array<vec4<u32>>;
@group(0) @binding(4) var<storage, read> indices: array<u32>;
// Every clustered mesh's vertices (scene Vertex: 4 x vec4) + foliage regions.
@group(0) @binding(5) var<storage, read> vertices: array<vec4<f32>>;
// Compact 8 B/px visibility buffer: depth words [0..wh), id words [wh..2*wh).
@group(0) @binding(6) var<storage, read_write> vbuf: array<atomic<u32>>;
// Per soft batch DrawIndirectArgs (4 words: 6, tile_count, 0, 0).
@group(0) @binding(7) var<storage, read_write> tile_draws: array<atomic<u32>>;
// Front [0..2*soft_rows): (start_offset, write_cursor) per soft row;
// tail [2*soft_rows..max_tiles): packed 8x8 tile coords (tx | (ty << 16)).
@group(0) @binding(8) var<storage, read_write> tile_list: array<atomic<u32>>;
// The wind field's lean (wind_field.rs; meshlet_sway.wgsl).
@group(0) @binding(9) var meshlet_wind_tex: texture_2d<f32>;

const WORKGROUPS_PER_ROW: u32 = 65535u;
const SUBPIXEL: i32 = 256;
const RASTER_MAX: f32 = 64.0;
const INVALID_ROW: u32 = 0xffffffffu;

var<workgroup> wg_tile_rows: array<u32, 64>;

fn raster_retro(clip: vec4<f32>) -> vec4<f32> {
  var c = clip;
  if (c.w <= 0.0) { return c; }
  if (raster.stylise.x > 0.0) {
    let half = raster.stylise.xy * 0.5;
    let ndc = c.xy / c.w;
    c = vec4<f32>(round(ndc * half) / half * c.w, c.zw);
  }
  if (raster.stylise.w > 0.0) {
    let steps = exp2(raster.stylise.w);
    c.z = floor(c.z / c.w * steps) / steps * c.w;
  }
  return c;
}

fn raster_leaf(local: u32, region: u32, now: mat4x4<f32>) -> MeshletLeaf {
  let at = meshlet_sway_row(region, local);
  return meshlet_leaf(vertices[region], vertices[at + 1u], now, raster.wind[0].origin.xyz, raster.leaf_year);
}

fn raster_position(base: u32, local: u32, region: u32, level: u32, inst: u32, model: mat4x4<f32>) -> vec3<f32> {
  var p = vertices[(base + local) * 4u].xyz;
  if (region == 0u) { return p; }
  let at = meshlet_sway_row(region, local);
  let scale = meshlet_level_scale(vertices[meshlet_scale_row(region, level)], level);
  p = meshlet_grown(p, vertices[at + 1u], raster_leaf(local, region, model).leaf.kept * scale);
  let layer = instances[inst * 4u + 3u].x;
  return p + meshlet_sway(p, vertices[region - 1u], vertices[at], vertices[at + 1u], model, raster.wind[0], foliage_sway(layer), false);
}

fn transform_at(buf_index: u32) -> mat4x4<f32> {
  let t0 = instances[buf_index * 4u];
  let t1 = instances[buf_index * 4u + 1u];
  let t2 = instances[buf_index * 4u + 2u];
  return instance_matrix(t0.xyz, vec3<f32>(t0.w, t1.xy), vec3<f32>(t1.zw, t2.x), t2.yzw);
}

struct Tri {
  ok: bool,
  p: array<vec3<f32>, 3>,
  lo: vec2<i32>,
  hi: vec2<i32>,
};

fn soft_cluster(wg: vec3<u32>) -> u32 {
  return wg.x + wg.y * WORKGROUPS_PER_ROW;
}

fn triangle(k: u32, tri: u32) -> Tri {
  var t: Tri;
  t.ok = false;
  let e = items[raster.soft.x + k * 2u];
  let f = items[raster.soft.x + k * 2u + 1u];
  let c = clusters[e.y * 3u];
  if (tri >= c.y) { return t; }
  let model = transform_at(e.x);
  let size = vec2<f32>(raster.size.xy);
  for (var j = 0u; j < 3u; j++) {
    let local = indices[c.x + tri * 3u + j];
    let world = model * vec4<f32>(raster_position(f.y, local, c.w, c.z, e.x, model), 1.0);
    let clip = raster_retro(mul_point(raster.view_proj, world));
    if (clip.w <= 1e-6 || clip.z > clip.w) { return t; }
    let ndc = clip.xyz / clip.w;
    t.p[j] = vec3<f32>((ndc.x * 0.5 + 0.5) * size.x, (0.5 - ndc.y * 0.5) * size.y, ndc.z);
  }
  let a = t.p[0];
  let b = t.p[1];
  let d = t.p[2];
  let area = (b.x - a.x) * (d.y - a.y) - (b.y - a.y) * (d.x - a.x);
  let front = area < 0.0;
  let side = f.x >> 28u;
  if (area == 0.0 || (side == 0u && !front) || (side == 1u && front)) { return t; }
  let lo = min(a, min(b, d));
  let hi = max(a, max(b, d));
  if (hi.z < 0.0 || hi.x < 0.0 || hi.y < 0.0 || lo.x > size.x || lo.y > size.y) { return t; }
  if (hi.x - lo.x > RASTER_MAX || hi.y - lo.y > RASTER_MAX) { return t; }
  t.lo = max(vec2<i32>(ceil(lo.xy - 0.5)), vec2<i32>(0));
  t.hi = min(vec2<i32>(floor(hi.xy - 0.5)), vec2<i32>(raster.size.xy) - 1);
  t.ok = all(t.lo <= t.hi);
  return t;
}

fn edge(a: vec2<i32>, b: vec2<i32>, p: vec2<i32>) -> i32 {
  return (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x);
}

fn owns(a: vec2<i32>, b: vec2<i32>) -> bool {
  let d = b - a;
  return d.y < 0 || (d.y == 0 && d.x > 0);
}

// Incremental half-space rasterisation with top-left ownership bias and early
// atomicLoad pre-test before atomicMax to cut coherent L2 atomic contention.
fn cover(t: Tri, id: u32) {
  let o = floor(min(t.p[0].xy, min(t.p[1].xy, t.p[2].xy)));
  var v = array<vec2<i32>, 3>(
    vec2<i32>(round((t.p[0].xy - o) * f32(SUBPIXEL))),
    vec2<i32>(round((t.p[1].xy - o) * f32(SUBPIXEL))),
    vec2<i32>(round((t.p[2].xy - o) * f32(SUBPIXEL))),
  );
  var z = vec3<f32>(t.p[0].z, t.p[1].z, t.p[2].z);
  var area = edge(v[0], v[1], v[2]);
  if (area == 0) { return; }
  if (area < 0) {
    let s = v[1];
    v[1] = v[2];
    v[2] = s;
    z = z.xzy;
    area = -area;
  }
  let inv_area = 1.0 / f32(area);
  let w = raster.size.x;
  let wh = raster.size.x * raster.size.y;
  let oi = vec2<i32>(o);
  let half = SUBPIXEL / 2;
  let c0 = (t.lo - oi) * SUBPIXEL + vec2<i32>(half);
  let bias = vec3<i32>(
    select(-1, 0, owns(v[1], v[2])),
    select(-1, 0, owns(v[2], v[0])),
    select(-1, 0, owns(v[0], v[1])),
  );
  let step_x = vec3<i32>(v[1].y - v[2].y, v[2].y - v[0].y, v[0].y - v[1].y) * SUBPIXEL;
  let step_y = vec3<i32>(v[2].x - v[1].x, v[0].x - v[2].x, v[1].x - v[0].x) * SUBPIXEL;
  var row_e = vec3<i32>(
    edge(v[1], v[2], c0),
    edge(v[2], v[0], c0),
    edge(v[0], v[1], c0),
  );
  for (var y = t.lo.y; y <= t.hi.y; y++) {
    var e = row_e;
    let row_px = u32(y) * w;
    for (var x = t.lo.x; x <= t.hi.x; x++) {
      let eb = e + bias;
      if ((eb.x | eb.y | eb.z) >= 0) {
        let depth = (f32(e.x) * z.x + f32(e.y) * z.y + f32(e.z) * z.z) * inv_area;
        if (depth >= 0.0 && depth <= 1.0) {
          let word = bitcast<u32>(max(depth, 0.0));
          let pixel = row_px + u32(x);
          if (id == 0u) {
            if (atomicLoad(&vbuf[pixel]) < word) {
              atomicMax(&vbuf[pixel], word);
            }
          } else if (atomicLoad(&vbuf[pixel]) == word) {
            atomicMax(&vbuf[wh + pixel], id);
          }
        }
      }
      e += step_x;
    }
    row_e += step_y;
  }
}

@compute @workgroup_size(64)
fn raster_depth(@builtin(workgroup_id) wg: vec3<u32>, @builtin(local_invocation_index) li: u32) {
  let k = soft_cluster(wg);
  if (k >= min(raster.soft.z, raster.soft.y)) { return; }
  let t = triangle(k, li);
  if (t.ok) { cover(t, 0u); }
}

@compute @workgroup_size(64)
fn raster_id(@builtin(workgroup_id) wg: vec3<u32>, @builtin(local_invocation_index) li: u32) {
  let k = soft_cluster(wg);
  if (k >= min(raster.soft.z, raster.soft.y)) { return; }
  let t = triangle(k, li);
  if (t.ok) { cover(t, ((k << 6u) | li) + 1u); }
}

fn pixel_batch_row(gid: vec2<u32>) -> u32 {
  if (gid.x >= raster.size.x || gid.y >= raster.size.y) { return INVALID_ROW; }
  let wh = raster.size.x * raster.size.y;
  let id = atomicLoad(&vbuf[wh + gid.y * raster.size.x + gid.x]);
  if (id == 0u) { return INVALID_ROW; }
  let k = (id - 1u) >> 6u;
  let row = items[raster.soft.x + k * 2u + 1u].x & 0x0fffffffu;
  if (row >= raster.size.z) { return INVALID_ROW; }
  return row;
}

// Pass 1: count unique active batches per 8x8 tile into tile_draws[row].instance_count.
@compute @workgroup_size(8, 8)
fn classify_count(@builtin(global_invocation_id) gid: vec3<u32>, @builtin(local_invocation_index) li: u32) {
  let row = pixel_batch_row(gid.xy);
  wg_tile_rows[li] = row;
  workgroupBarrier();
  if (row != INVALID_ROW) {
    var first = true;
    for (var j = 0u; j < li; j++) {
      if (wg_tile_rows[j] == row) {
        first = false;
        break;
      }
    }
    if (first) {
      atomicAdd(&tile_draws[row * 4u + 1u], 1u);
    }
  }
}

// Pass 2: exclusive prefix sum over soft batch tile counts and initialize DrawIndirect args.
@compute @workgroup_size(1)
fn classify_prefix() {
  let soft_rows = raster.size.z;
  let max_tiles = raster.size.w;
  var cursor = soft_rows * 2u;
  for (var r = 0u; r < soft_rows; r++) {
    let raw = atomicLoad(&tile_draws[r * 4u + 1u]);
    let avail = select(0u, max_tiles - cursor, max_tiles > cursor);
    let n = min(raw, avail);
    atomicStore(&tile_draws[r * 4u + 0u], 6u);
    atomicStore(&tile_draws[r * 4u + 1u], n);
    atomicStore(&tile_draws[r * 4u + 2u], 0u);
    atomicStore(&tile_draws[r * 4u + 3u], 0u);
    atomicStore(&tile_list[r * 2u], cursor);
    atomicStore(&tile_list[r * 2u + 1u], 0u);
    cursor += n;
  }
}

// Pass 3: scatter each 8x8 tile coordinate into its active batches' compact tile slices.
@compute @workgroup_size(8, 8)
fn classify_emit(@builtin(global_invocation_id) gid: vec3<u32>,
                 @builtin(workgroup_id) wg: vec3<u32>,
                 @builtin(local_invocation_index) li: u32) {
  let row = pixel_batch_row(gid.xy);
  wg_tile_rows[li] = row;
  workgroupBarrier();
  if (row != INVALID_ROW) {
    var first = true;
    for (var j = 0u; j < li; j++) {
      if (wg_tile_rows[j] == row) {
        first = false;
        break;
      }
    }
    if (first) {
      let base = atomicLoad(&tile_list[row * 2u]);
      let limit = atomicLoad(&tile_draws[row * 4u + 1u]);
      let slot = atomicAdd(&tile_list[row * 2u + 1u], 1u);
      if (slot < limit) {
        atomicStore(&tile_list[base + slot], (wg.x & 0xffffu) | ((wg.y & 0xffffu) << 16u));
      }
    }
  }
}
