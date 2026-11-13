// The meshlet path's vertex stages (meshlet.rs, meshlet_gpu.rs), after
// pbr.wgsl and pbr_prepass.wgsl in the standard lit module. A batch is one
// draw of 64 x 3 vertices per visible cluster: the instance index names a
// work item (an instance and a cluster, written by meshlet_cull.wgsl), the
// vertex index a corner of one of the cluster's triangles, read from the
// cluster pool's indices and the mesh's own f32 vertices (vertex pulling).
// A corner past the cluster's triangles is degenerate (w 0: clipped).

// meshlet_sway.wgsl's `MeshletDraw`.
@group(3) @binding(8) var<uniform> meshlet_draw: MeshletDraw;
// The wind field's lean (wind_field.rs; meshlet_sway.wgsl).
@group(3) @binding(18) var meshlet_wind_tex: texture_2d<f32>;
// The culled instances' transforms (instances.rs `GpuTransform`, 4 x vec4).
@group(3) @binding(9) var<storage, read> meshlet_instances: array<vec4<f32>>;
// The batch's work items (bound at its first): instance, cluster.
@group(3) @binding(10) var<storage, read> meshlet_items: array<vec2<u32>>;
// Per cluster 3 x vec4: (first index, triangles, level, 0), sphere, cone.
@group(3) @binding(11) var<storage, read> meshlet_clusters: array<vec4<u32>>;
@group(3) @binding(12) var<storage, read> meshlet_indices: array<u32>;
// The mesh's vertices (scene Vertex: 16 floats).
@group(3) @binding(13) var<storage, read> meshlet_vertices: array<vec4<f32>>;
// The instances' previous transforms, at the same slots (the current ones
// without object motion).
@group(3) @binding(14) var<storage, read> meshlet_previous: array<vec4<f32>>;
// The vertex pool, read for its foliage regions (meshlet_sway.wgsl).
@group(3) @binding(17) var<storage, read> meshlet_foliage: array<vec4<f32>>;

struct MeshletVertex {
  ok: bool,
  v: VertexIn,
  model: mat4x4<f32>,
  look: vec4<f32>,
  // The instance's sorted slot.
  slot: u32,
  // Its foliage region plus one (0: none) and the vertex's own index.
  region: u32,
  index: u32,
  // Where the vertex rests (the swayed position is v.position).
  rest: vec3<f32>,
};

// The vertex's foliage rows (meshlet_sway.wgsl).
fn meshlet_rows(region: u32, index: u32) -> array<vec4<f32>, 3> {
  let at = meshlet_sway_row(region, index);
  return array<vec4<f32>, 3>(meshlet_foliage[region - 1u], meshlet_foliage[at], meshlet_foliage[at + 1u]);
}

fn meshlet_transform(t0: vec4<f32>, t1: vec4<f32>, t2: vec4<f32>) -> mat4x4<f32> {
  return instance_matrix(t0.xyz, vec3<f32>(t0.w, t1.xy), vec3<f32>(t1.zw, t2.x), t2.yzw);
}

// lod.rs `debug_color`, packed 8:8:8 as pbr.wgsl's `look_rgb` reads it.
fn meshlet_level_color(level: u32) -> f32 {
  var c = vec3<f32>(0.9, 0.08, 0.08);
  if (level == 0u) { c = vec3<f32>(0.1, 0.85, 0.15); }
  else if (level == 1u) { c = vec3<f32>(0.95, 0.85, 0.1); }
  else if (level == 2u) { c = vec3<f32>(1.0, 0.45, 0.05); }
  let q = vec3<u32>(c * 255.0 + 0.5);
  return f32((q.x << 16u) | (q.y << 8u) | q.z);
}

fn meshlet_vertex(vi: u32, ii: u32) -> MeshletVertex {
  var r: MeshletVertex;
  let item = meshlet_items[ii];
  let c = meshlet_clusters[item.y * 3u];
  if (vi / 3u >= c.y) {
    r.ok = false;
    return r;
  }
  r.ok = true;
  let index = meshlet_indices[c.x + vi];
  let a = meshlet_vertices[index * 4u];
  let b = meshlet_vertices[index * 4u + 1u];
  r.v.position = a.xyz;
  r.v.normal = vec3<f32>(a.w, b.xy);
  r.v.uv = b.zw;
  r.v.color = meshlet_vertices[index * 4u + 2u];
  r.v.tangent = meshlet_vertices[index * 4u + 3u];
  let t0 = meshlet_instances[item.x * 4u];
  let t1 = meshlet_instances[item.x * 4u + 1u];
  let t2 = meshlet_instances[item.x * 4u + 2u];
  r.model = meshlet_transform(t0, t1, t2);
  r.slot = item.x;
  r.look = meshlet_instances[item.x * 4u + 3u];
  r.region = c.w;
  r.index = index;
  r.rest = r.v.position;
  if (c.w != 0u) {
    let f = meshlet_rows(c.w, index);
    r.v.position += meshlet_sway(r.rest, f[0], f[1], f[2], r.model, meshlet_draw.wind[0], foliage_sway(r.look.x), false);
  }
  if (meshlet_draw.flags.x != 0u) {
    r.look.y = 0.8;
    r.look.z = meshlet_level_color(c.z);
  }
  return r;
}

@vertex
fn vs_meshlet(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VertexOut {
  let p = meshlet_vertex(vi, ii);
  if (!p.ok) {
    var o: VertexOut;
    o.clip = vec4<f32>(0.0);
    return o;
  }
  var v = p.v;
  if (p.region != 0u) {
    // The leaves' canopy dome (the material's foliage, coat.w), on the lit
    // pass alone: the shadow casters and the prepass keep the mesh's own.
    let f = meshlet_rows(p.region, p.index);
    v.normal = foliage_normal(v.normal, f[2].xyz, f[0].xyz, material.coat.w);
  }
  var o = lit_vertex(p.model, v);
  o.look = p.look;
  return o;
}

@vertex
fn vs_prepass_meshlet(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> PrepassOut {
  let p = meshlet_vertex(vi, ii);
  if (!p.ok) {
    var o: PrepassOut;
    o.clip = vec4<f32>(0.0);
    return o;
  }
  // Last frame's transform for the velocity (TAA, motion blur).
  let q = p.slot * 4u;
  let prev = meshlet_transform(meshlet_previous[q], meshlet_previous[q + 1u], meshlet_previous[q + 2u]);
  // A foliage vertex was where the last drawn frame's wind and sway (the
  // previous record's layer word) put it.
  var before = p.rest;
  if (p.region != 0u) {
    let f = meshlet_rows(p.region, p.index);
    before += meshlet_sway(p.rest, f[0], f[1], f[2], prev, meshlet_draw.wind[1], foliage_sway(meshlet_previous[q + 3u].x), true);
  }
  return prepass_vertex_from(p.model, prev, p.v, before);
}

// The visibility buffer's resolve (meshlet_raster.wgsl): one full-screen
// triangle per soft batch, its row the instance index. A pixel whose
// record is the batch's is shaded as the hardware path shades a fragment
// (`lit_fragment`, `prepass_targets`) at the rasteriser's depth; any
// other is discarded.
// Per pixel its record's slot plus one, then the records
// (meshlet_raster.wgsl `setup`); binding 15 its screen width.
@group(3) @binding(15) var<uniform> meshlet_screen: vec4<u32>;
@group(3) @binding(16) var<storage, read> meshlet_attrs: array<vec4<u32>>;

struct MeshletResolveOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) @interpolate(flat) row: u32,
};

@vertex
fn vs_meshlet_resolve(@builtin(vertex_index) vi: u32, @builtin(instance_index) row: u32) -> MeshletResolveOut {
  var o: MeshletResolveOut;
  o.clip = fullscreen_clip(vi);
  o.row = row;
  return o;
}

struct MeshletRecord {
  // The pixel is this batch's.
  ok: bool,
  depth: f32,
  v: VertexOut,
  prev_world: vec3<f32>,
  object: u32,
  // The lit pass's uv gradients: pbr.wgsl `stylised_uv`'s (affine
  // texturing's mix), the raw uv's without the extra rows.
  lit_uv: vec4<f32>,
};

fn meshlet_record(pos: vec4<f32>, row: u32) -> MeshletRecord {
  var r: MeshletRecord;
  // meshlet_screen: (width, height, its pixels rounded up to a row, vec4
  // rows per record).
  let pixel = u32(pos.y) * meshlet_screen.x + u32(pos.x);
  let slot = meshlet_attrs[pixel / 4u][pixel % 4u];
  r.ok = false;
  r.depth = 1.0;
  if (slot == 0u) { return r; }
  let at = meshlet_screen.z / 4u + (slot - 1u) * meshlet_screen.w;
  let a = meshlet_attrs[at];
  let b = meshlet_attrs[at + 1u];
  let c = meshlet_attrs[at + 2u];
  if (c.z != row) { return r; }
  r.ok = true;
  r.depth = bitcast<f32>(a.w);
  r.v.clip = pos;
  r.v.world_pos = bitcast<vec3<f32>>(a.xyz);
  r.v.normal = oct_decode(unpack2x16snorm(b.x));
  r.v.uv = bitcast<vec2<f32>>(b.yz);
  r.v.color = unpack4x8unorm(b.w);
  // The material style's surface pattern lies in the mesh's own space.
  r.v.local_pos = vec3<f32>(
    bitcast<f32>(meshlet_attrs[at + 4u].w),
    bitcast<f32>(meshlet_attrs[at + 6u].w),
    bitcast<f32>(meshlet_attrs[at + 7u].w),
  );
  r.v.tangent = vec4<f32>(oct_decode(unpack2x16snorm(c.x)), bitcast<f32>(c.y));
  if (r.v.tangent.w == 0.0) { r.v.tangent = vec4<f32>(0.0); }
  r.v.affine = vec3<f32>(r.v.uv, 1.0);
  r.v.look = bitcast<vec4<f32>>(meshlet_attrs[at + 3u]);
  r.prev_world = bitcast<vec3<f32>>(meshlet_attrs[at + 4u].xyz);
  // The pixel's gradients (pbr_fs.wgsl `ANALYTIC`).
  analytic_uv = bitcast<vec4<f32>>(meshlet_attrs[at + 5u]);
  analytic_dx = bitcast<vec3<f32>>(meshlet_attrs[at + 6u].xyz);
  analytic_dy = bitcast<vec3<f32>>(meshlet_attrs[at + 7u].xyz);
  r.lit_uv = analytic_uv;
  if (meshlet_screen.w > 8u) {
    // The extra rows (meshlet_raster.wgsl `setup`).
    let lx = bitcast<vec4<f32>>(meshlet_attrs[at + 8u]);
    let ly = bitcast<vec4<f32>>(meshlet_attrs[at + 9u]);
    analytic_local_dx = lx.xyz;
    analytic_local_dy = ly.xyz;
    // Affine texturing's uv, which `stylised_uv` mixes in.
    r.v.affine = vec3<f32>(lx.w, ly.w, 1.0);
    r.lit_uv = bitcast<vec4<f32>>(meshlet_attrs[at + 10u]);
    // A foliage mesh's canopy dome, by its material's foliage (coat.w),
    // as `vs_meshlet` blends it per vertex.
    let d = meshlet_attrs[at + 11u];
    if (d.y != 0u && material.coat.w > 0.0) {
      r.v.normal = normalize(mix(normalize(r.v.normal), oct_decode(unpack2x16snorm(d.x)), clamp(material.coat.w, 0.0, 1.0)));
    }
  }
  r.object = c.w;
  return r;
}

struct MeshletResolveLit {
  @location(0) color: vec4<f32>,
  @builtin(frag_depth) depth: f32,
};

@fragment
fn fs_meshlet_resolve(in: MeshletResolveOut) -> MeshletResolveLit {
  let r = meshlet_record(in.clip, in.row);
  if (!r.ok) { discard; }
  var o: MeshletResolveLit;
  o.depth = r.depth;
  // Read only by a material shader's hooks, which no soft batch has.
  hook_front = 1.0;
  analytic_uv = r.lit_uv;
  o.color = lit_fragment(r.v);
  return o;
}

struct MeshletResolvePrepass {
  @location(0) gbuffer: vec4<f32>,
  @location(1) depth: vec4<f32>,
  @location(2) outline: vec4<f32>,
  @builtin(frag_depth) frag_depth: f32,
};

@fragment
fn fs_prepass_meshlet_resolve(in: MeshletResolveOut) -> MeshletResolvePrepass {
  let r = meshlet_record(in.clip, in.row);
  if (!r.ok) { discard; }
  var p: PrepassOut;
  p.clip = in.clip;
  p.world_pos = r.v.world_pos;
  p.normal = r.v.normal;
  p.uv = r.v.uv;
  p.color = r.v.color;
  p.local_pos = r.v.local_pos;
  p.cur_clip = fx.view_proj_unjittered * vec4<f32>(r.v.world_pos, 1.0);
  p.prev_clip = mul_point(fx.prev_view_proj, vec4<f32>(r.prev_world, 1.0));
  p.object = r.object;
  hook_front = 1.0;
  let t = prepass_targets(p);
  var o: MeshletResolvePrepass;
  o.gbuffer = t.gbuffer;
  o.depth = t.depth;
  o.outline = t.outline;
  o.frag_depth = r.depth;
  return o;
}
