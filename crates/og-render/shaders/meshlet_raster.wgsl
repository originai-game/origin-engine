// The visibility buffer (meshlet.rs states each step on the CPU,
// meshlet_gpu.rs drives it; three.js#35191's software rasteriser). The
// camera's small clusters, which meshlet_cull.wgsl appended to the soft
// list, are rasterised here, one workgroup a cluster and one thread a
// triangle: `raster_depth` takes each covered pixel's nearest depth
// (an atomic max of meshlet.rs `vis_depth`), `raster_id` writes the
// triangle's id where its depth is the pixel's (`vis_id`, an atomic max:
// equal depths keep one), then `setup`, one thread a pixel, rebuilds the
// seen triangle's attributes there and appends them to `attrs` (its slot
// in the pixel's word at the front), for the resolve (meshlet.wgsl `fs_meshlet_resolve`) to
// shade. After instance.wgsl.

struct RasterParams {
  // The camera's (jittered, as the hardware draws it).
  view_proj: mat4x4<f32>,
  // xy: the screen's size; z: the most records `attrs` holds; w: 1 tints
  // each instance by its level (RenderScene::lod_debug).
  size: vec4<u32>,
  // x: the soft list's first item, y: its capacity in clusters, z: its
  // count (copied from the cull's), w: vec4 rows per record (8, or 12
  // with the extra rows).
  soft: vec4<u32>,
  // Retro 3D (frame.wgsl `stylise`, stylise.rs `Retro3d::uniform`): the
  // snap grid in cells, the affine share, the depth bits.
  stylise: vec4<f32>,
  // The foliage's wind this frame and the last drawn (meshlet_sway.wgsl).
  wind: array<FoliageWind, 2>,
};

@group(0) @binding(0) var<uniform> raster: RasterParams;
// The sorted instances (instances.rs `GpuTransform`, 4 x vec4).
@group(0) @binding(1) var<storage, read> instances: array<vec4<f32>>;
// The soft list: per cluster (instance, cluster), (row | side << 28,
// the mesh's first vertex in the vertex pool).
@group(0) @binding(2) var<storage, read> items: array<vec2<u32>>;
@group(0) @binding(3) var<storage, read> clusters: array<vec4<u32>>;
@group(0) @binding(4) var<storage, read> indices: array<u32>;
// Every clustered mesh's vertices (scene Vertex: 4 x vec4).
@group(0) @binding(5) var<storage, read> vertices: array<vec4<f32>>;
// The depth words (one a pixel), the id words, then the records' count.
@group(0) @binding(6) var<storage, read_write> vbuf: array<atomic<u32>>;
// Per pixel its record's slot plus one (0: none of the soft list's),
// then per record RECORD_WORDS words (meshlet.wgsl `meshlet_record`).
@group(0) @binding(7) var<storage, read_write> attrs: array<u32>;
// The instances' previous transforms, at the same slots.
@group(0) @binding(8) var<storage, read> previous: array<vec4<f32>>;
// The wind field's lean (wind_field.rs; meshlet_sway.wgsl).
@group(0) @binding(9) var meshlet_wind_tex: texture_2d<f32>;

const WG: u32 = 64u;
const WORKGROUPS_PER_ROW: u32 = 65535u;
// meshlet.rs SUBPIXEL and RASTER_MAX.
const SUBPIXEL: i32 = 256;
const RASTER_MAX: f32 = 64.0;

// pbr.wgsl `retro_clip` (meshlet.rs `retro_clip`) over the raster's
// stylise: the hardware path's clip position, snapped and stepped.
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

// Vertex `local` (the mesh's own index) of a cluster whose first row's w
// is `region` (meshlet_sway.wgsl; 0: no foliage), in a mesh at `base` of
// the pool: where instance slot `inst` puts it in the mesh's own space,
// swayed by this frame's wind, or the last drawn frame's at `model` (the
// previous transform) with `from_previous`.
fn raster_position(base: u32, local: u32, region: u32, inst: u32, model: mat4x4<f32>, from_previous: bool) -> vec3<f32> {
  let p = vertices[(base + local) * 4u].xyz;
  if (region == 0u) { return p; }
  let at = meshlet_sway_row(region, local);
  var layer = instances[inst * 4u + 3u].x;
  var wind = raster.wind[0];
  if (from_previous) {
    layer = previous[inst * 4u + 3u].x;
    wind = raster.wind[1];
  }
  return p + meshlet_sway(p, vertices[region - 1u], vertices[at], vertices[at + 1u], model, wind, foliage_sway(layer), from_previous);
}

fn transform_at(buf_index: u32, from_previous: bool) -> mat4x4<f32> {
  var t0: vec4<f32>;
  var t1: vec4<f32>;
  var t2: vec4<f32>;
  if (from_previous) {
    t0 = previous[buf_index * 4u];
    t1 = previous[buf_index * 4u + 1u];
    t2 = previous[buf_index * 4u + 2u];
  } else {
    t0 = instances[buf_index * 4u];
    t1 = instances[buf_index * 4u + 1u];
    t2 = instances[buf_index * 4u + 2u];
  }
  return instance_matrix(t0.xyz, vec3<f32>(t0.w, t1.xy), vec3<f32>(t1.zw, t2.x), t2.yzw);
}

// One soft cluster's triangle in pixels and depth (meshlet.rs
// `to_screen`, `raster_class`): ok false when it covers nothing here.
struct Tri {
  ok: bool,
  p: array<vec3<f32>, 3>,
  // The pixel centres of its box, inclusive.
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
  let model = transform_at(e.x, false);
  let size = vec2<f32>(raster.size.xy);
  for (var j = 0u; j < 3u; j++) {
    let local = indices[c.x + tri * 3u + j];
    let world = model * vec4<f32>(raster_position(f.y, local, c.w, e.x, model, false), 1.0);
    let clip = raster_retro(mul_point(raster.view_proj, world));
    // Never for a soft cluster (it lies past the near plane): skipped.
    if (clip.w <= 1e-6 || clip.z < 0.0) { return t; }
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
  if (lo.z > 1.0 || hi.x < 0.0 || hi.y < 0.0 || lo.x > size.x || lo.y > size.y) { return t; }
  if (hi.x - lo.x > RASTER_MAX || hi.y - lo.y > RASTER_MAX) { return t; }
  t.lo = max(vec2<i32>(ceil(lo.xy - 0.5)), vec2<i32>(0));
  t.hi = min(vec2<i32>(floor(hi.xy - 0.5)), vec2<i32>(raster.size.xy) - 1);
  t.ok = all(t.lo <= t.hi);
  return t;
}

fn edge(a: vec2<i32>, b: vec2<i32>, p: vec2<i32>) -> i32 {
  return (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x);
}

// meshlet.rs `owns`.
fn owns(a: vec2<i32>, b: vec2<i32>) -> bool {
  let d = b - a;
  return d.y < 0 || (d.y == 0 && d.x > 0);
}

fn inside(e: i32, a: vec2<i32>, b: vec2<i32>) -> bool {
  return e > 0 || (e == 0 && owns(a, b));
}

// meshlet.rs `covers` over the triangle's box: per covered pixel, the
// depth word, then `id` (0: the depth pass).
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
  let w = raster.size.x;
  let wh = raster.size.x * raster.size.y;
  let oi = vec2<i32>(o);
  let half = SUBPIXEL / 2;
  for (var y = t.lo.y; y <= t.hi.y; y++) {
    for (var x = t.lo.x; x <= t.hi.x; x++) {
      let c = (vec2<i32>(x, y) - oi) * SUBPIXEL + vec2<i32>(half);
      let e0 = edge(v[1], v[2], c);
      let e1 = edge(v[2], v[0], c);
      let e2 = edge(v[0], v[1], c);
      if (!(inside(e0, v[1], v[2]) && inside(e1, v[2], v[0]) && inside(e2, v[0], v[1]))) { continue; }
      let depth = (f32(e0) * z.x + f32(e1) * z.y + f32(e2) * z.z) / f32(area);
      if (depth < 0.0 || depth > 1.0) { continue; }
      // meshlet.rs `vis_depth`.
      let word = ~bitcast<u32>(max(depth, 0.0));
      let pixel = u32(y) * w + u32(x);
      if (id == 0u) {
        atomicMax(&vbuf[pixel], word);
      } else if (atomicLoad(&vbuf[pixel]) == word) {
        atomicMax(&vbuf[wh + pixel], id);
      }
    }
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
  // meshlet.rs `vis_id`.
  if (t.ok) { cover(t, ((k << 6u) | li) + 1u); }
}

// lighting.wgsl `oct_encode`.
fn raster_oct(n: vec3<f32>) -> vec2<f32> {
  let p = n.xy / (abs(n.x) + abs(n.y) + abs(n.z));
  if (n.z >= 0.0) { return p; }
  return (1.0 - abs(p.yx)) * select(vec2<f32>(-1.0), vec2<f32>(1.0), p >= vec2<f32>(0.0));
}

// pbr.wgsl `world_normal`.
fn raster_normal(model: mat4x4<f32>, n: vec3<f32>) -> vec3<f32> {
  let a = model[0].xyz;
  let b = model[1].xyz;
  let c = model[2].xyz;
  let cofactor = mat3x3<f32>(cross(b, c), cross(c, a), cross(a, b));
  return normalize(cofactor * n) * select(1.0, -1.0, dot(a, cross(b, c)) < 0.0);
}

// pbr.wgsl `object_id`.
fn raster_object(model: mat4x4<f32>) -> u32 {
  var h = 2166136261u;
  for (var c = 0u; c < 3u; c++) {
    h = (h ^ bitcast<u32>(model[3][c])) * 16777619u;
    h = (h ^ bitcast<u32>(model[0][c] + model[1][c] * 3.0 + model[2][c] * 7.0)) * 16777619u;
  }
  h = h ^ (h >> 16u);
  return 1u + h % 255u;
}

// lod.rs `debug_color` packed 8:8:8 (meshlet.wgsl `meshlet_level_color`).
fn raster_level_color(level: u32) -> f32 {
  var c = vec3<f32>(0.9, 0.08, 0.08);
  if (level == 0u) { c = vec3<f32>(0.1, 0.85, 0.15); }
  else if (level == 1u) { c = vec3<f32>(0.95, 0.85, 0.1); }
  else if (level == 2u) { c = vec3<f32>(1.0, 0.45, 0.05); }
  let q = vec3<u32>(c * 255.0 + 0.5);
  return f32((q.x << 16u) | (q.y << 8u) | q.z);
}

// meshlet.rs `screen_weights`: the screen-linear weights at `ndc` (affine
// texturing's), extrapolated past the triangle's edges as a quad's helper
// pixels are.
fn raster_screen(clip: array<vec4<f32>, 3>, ndc: vec2<f32>) -> vec3<f32> {
  let p0 = clip[0].xy / clip[0].w;
  let p1 = clip[1].xy / clip[1].w;
  let p2 = clip[2].xy / clip[2].w;
  let area = (p1.x - p0.x) * (p2.y - p0.y) - (p1.y - p0.y) * (p2.x - p0.x);
  return vec3<f32>(
    (p1.x - ndc.x) * (p2.y - ndc.y) - (p1.y - ndc.y) * (p2.x - ndc.x),
    (p2.x - ndc.x) * (p0.y - ndc.y) - (p2.y - ndc.y) * (p0.x - ndc.x),
    (p0.x - ndc.x) * (p1.y - ndc.y) - (p0.y - ndc.y) * (p1.x - ndc.x),
  ) / area;
}

// meshlet.rs `barycentrics`: the perspective-correct weights.
fn raster_bary(clip: array<vec4<f32>, 3>, l: vec3<f32>) -> vec3<f32> {
  let q = l / vec3<f32>(clip[0].w, clip[1].w, clip[2].w);
  return q / (q.x + q.y + q.z);
}

// One thread a pixel: the seen soft triangle's attributes at the pixel's
// centre, perspective-correct, as a record: (world, depth), (normal oct,
// uv, colour), (tangent oct, tangent w, its batch row, object id), the
// instance's look, (last frame's world, local x), (d(uv)/dx, d(uv)/dy),
// (d(world)/dx, local y), (d(world)/dy, local z); with the extra rows
// (`raster.soft.w` 12) then (d(local)/dx, affine u), (d(local)/dy,
// affine v), (d(lit uv)/dx, d(lit uv)/dy), the lit uv being pbr.wgsl
// `stylised_uv`'s mix of the two, and (the canopy dome's oct, 1 with
// one, 0, 0). A foliage mesh's corners are swayed (`raster_position`):
// now for the pixel, at the last drawn frame for last frame's world. The gradients are the differences to
// the next pixel right and down on the same triangle's plane, which the
// hardware's quads take and the resolve's full-screen quads cannot
// (pbr_fs.wgsl `ANALYTIC`). The triangle's corners are the hardware's,
// retro 3D's snap and depth steps included (`raster_retro`).
@compute @workgroup_size(8, 8)
fn setup(@builtin(global_invocation_id) gid: vec3<u32>) {
  if (gid.x >= raster.size.x || gid.y >= raster.size.y) { return; }
  let wh = raster.size.x * raster.size.y;
  let pixel = gid.y * raster.size.x + gid.x;
  let id = atomicLoad(&vbuf[wh + pixel]);
  if (id == 0u) {
    attrs[pixel] = 0u;
    return;
  }
  let k = (id - 1u) >> 6u;
  let tri = (id - 1u) & 63u;
  let e = items[raster.soft.x + k * 2u];
  let f = items[raster.soft.x + k * 2u + 1u];
  let c = clusters[e.y * 3u];
  let model = transform_at(e.x, false);
  let prev = transform_at(e.x, true);
  let size = vec2<f32>(raster.size.xy);
  let ndc = vec2<f32>((f32(gid.x) + 0.5) / size.x * 2.0 - 1.0, 1.0 - (f32(gid.y) + 0.5) / size.y * 2.0);
  var vi: array<u32, 3>;
  var clip: array<vec4<f32>, 3>;
  // Each corner where it is now and where it was (swayed for foliage).
  var at: array<vec3<f32>, 3>;
  var was: array<vec3<f32>, 3>;
  for (var j = 0u; j < 3u; j++) {
    let local = indices[c.x + tri * 3u + j];
    vi[j] = f.y + local;
    at[j] = raster_position(f.y, local, c.w, e.x, model, false);
    was[j] = raster_position(f.y, local, c.w, e.x, prev, true);
    clip[j] = raster_retro(mul_point(raster.view_proj, model * vec4<f32>(at[j], 1.0)));
  }
  let l = raster_screen(clip, ndc);
  // The next pixel right and down (framebuffer y runs down, ndc y up).
  let lx = raster_screen(clip, ndc + vec2<f32>(2.0 / size.x, 0.0));
  let ly = raster_screen(clip, ndc - vec2<f32>(0.0, 2.0 / size.y));
  let b = raster_bary(clip, l);
  let bx = raster_bary(clip, lx);
  let by = raster_bary(clip, ly);
  var position = vec3<f32>(0.0);
  var normal = vec3<f32>(0.0);
  var uv = vec2<f32>(0.0);
  var color = vec4<f32>(0.0);
  var tangent = vec4<f32>(0.0);
  var dpos_x = vec3<f32>(0.0);
  var dpos_y = vec3<f32>(0.0);
  var duv_x = vec2<f32>(0.0);
  var duv_y = vec2<f32>(0.0);
  var affine = vec2<f32>(0.0);
  var daff_x = vec2<f32>(0.0);
  var daff_y = vec2<f32>(0.0);
  var before = vec3<f32>(0.0);
  // A foliage mesh's canopy dome (meshlet_foliage.wgsl `foliage_normal` at
  // full strength), which the resolve blends in by its material's.
  var dome = vec3<f32>(0.0);
  for (var j = 0u; j < 3u; j++) {
    let a = vertices[vi[j] * 4u];
    let v = vertices[vi[j] * 4u + 1u];
    dpos_x += at[j] * (bx[j] - b[j]);
    dpos_y += at[j] * (by[j] - b[j]);
    before += was[j] * b[j];
    if (c.w != 0u) {
      let k = meshlet_sway_row(c.w, vi[j] - f.y);
      dome += foliage_normal(vec3<f32>(a.w, v.xy), vertices[k + 1u].xyz, vertices[c.w - 1u].xyz, 1.0) * b[j];
    }
    duv_x += v.zw * (bx[j] - b[j]);
    duv_y += v.zw * (by[j] - b[j]);
    affine += v.zw * l[j];
    daff_x += v.zw * (lx[j] - l[j]);
    daff_y += v.zw * (ly[j] - l[j]);
    position += at[j] * b[j];
    normal += vec3<f32>(a.w, v.xy) * b[j];
    uv += v.zw * b[j];
    color += vertices[vi[j] * 4u + 2u] * b[j];
    tangent += vertices[vi[j] * 4u + 3u] * b[j];
  }
  let slot = atomicAdd(&vbuf[2u * wh], 1u);
  if (slot >= raster.size.z) {
    attrs[pixel] = 0u;
    return;
  }
  let world = (model * vec4<f32>(position, 1.0)).xyz;
  let n = raster_normal(model, normal);
  let tw = (model * vec4<f32>(tangent.xyz, 0.0)).xyz;
  let t = select(vec3<f32>(0.0, 0.0, 1.0), normalize(tw), dot(tw, tw) > 1e-20);
  var look = instances[e.x * 4u + 3u];
  // Its outline group (pbr.wgsl `look_outline`), read before the level
  // view below writes over the flash word.
  let group = u32(max(floor(look.y * 0.5), 0.0));
  if (raster.size.w != 0u) {
    look.y = 0.8;
    look.z = raster_level_color(c.z);
  }
  let depth = bitcast<f32>(~atomicLoad(&vbuf[pixel]));
  let dwx = (model * vec4<f32>(dpos_x, 0.0)).xyz;
  let dwy = (model * vec4<f32>(dpos_y, 0.0)).xyz;
  // A tangent of w 0 is none (pbr.wgsl `lit_vertex`).
  let rows = array<vec4<u32>, 8>(
    vec4<u32>(bitcast<vec3<u32>>(world), bitcast<u32>(depth)),
    vec4<u32>(pack2x16snorm(raster_oct(n)), bitcast<vec2<u32>>(uv), pack4x8unorm(color)),
    vec4<u32>(
      pack2x16snorm(raster_oct(t)),
      bitcast<u32>(select(0.0, tangent.w, dot(tw, tw) > 1e-20)),
      f.x & 0x0fffffffu,
      select(raster_object(model), group, group > 0u),
    ),
    bitcast<vec4<u32>>(look),
    vec4<u32>(bitcast<vec3<u32>>((prev * vec4<f32>(before, 1.0)).xyz), bitcast<u32>(position.x)),
    bitcast<vec4<u32>>(vec4<f32>(duv_x, duv_y)),
    vec4<u32>(bitcast<vec3<u32>>(dwx), bitcast<u32>(position.y)),
    vec4<u32>(bitcast<vec3<u32>>(dwy), bitcast<u32>(position.z)),
  );
  // The records from the slots' end, rounded up to a vec4.
  let stride = raster.soft.w;
  let r = ((wh + 3u) & ~3u) + slot * stride * 4u;
  for (var i = 0u; i < 8u; i++) {
    for (var j = 0u; j < 4u; j++) { attrs[r + i * 4u + j] = rows[i][j]; }
  }
  if (stride > 8u) {
    let k = raster.stylise.z;
    // The dome in the world (0: none).
    var dome_world = 0u;
    if (c.w != 0u) { dome_world = pack2x16snorm(raster_oct(raster_normal(model, dome))); }
    let more = array<vec4<u32>, 4>(
      bitcast<vec4<u32>>(vec4<f32>(dpos_x, affine.x)),
      bitcast<vec4<u32>>(vec4<f32>(dpos_y, affine.y)),
      bitcast<vec4<u32>>(vec4<f32>(mix(duv_x, daff_x, k), mix(duv_y, daff_y, k))),
      vec4<u32>(dome_world, u32(c.w != 0u), 0u, 0u),
    );
    for (var i = 0u; i < 4u; i++) {
      for (var j = 0u; j < 4u; j++) { attrs[r + (8u + i) * 4u + j] = more[i][j]; }
    }
  }
  attrs[pixel] = slot + 1u;
}
