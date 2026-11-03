// The meshlet path's shadow casters (meshlet_gpu.rs), after shadow.wgsl in
// the standard shadow module: one draw per clustered batch and shadow pass
// of 64 x 3 vertices per cluster meshlet_cull.wgsl kept for the pass's
// view, pulled as meshlet.wgsl pulls them (the same group 3). A corner
// past the cluster's triangles is degenerate (w 0: clipped).

// The culled instances' transforms (instances.rs `GpuTransform`, 4 x vec4).
@group(3) @binding(9) var<storage, read> meshlet_instances: array<vec4<f32>>;
// The batch's work items in this pass (bound at its first): instance, cluster.
@group(3) @binding(10) var<storage, read> meshlet_items: array<vec2<u32>>;
// Per cluster 3 x vec4: (first index, triangles, level, 0), sphere, cone.
@group(3) @binding(11) var<storage, read> meshlet_clusters: array<vec4<u32>>;
@group(3) @binding(12) var<storage, read> meshlet_indices: array<u32>;
// The mesh's vertices (scene Vertex: 16 floats).
@group(3) @binding(13) var<storage, read> meshlet_vertices: array<vec4<f32>>;
// meshlet_sway.wgsl's `MeshletDraw`, and the vertex pool for its foliage
// regions: a swaying caster's shadow sways with it.
@group(3) @binding(8) var<uniform> meshlet_draw: MeshletDraw;
@group(3) @binding(17) var<storage, read> meshlet_foliage: array<vec4<f32>>;

// Work item `ii`'s transform, and its corner `vi`'s vertex index (none
// past the cluster's triangles).
struct MeshletCorner {
  ok: bool,
  index: u32,
  model: mat4x4<f32>,
  // The corner's position, swayed (meshlet_sway.wgsl) for a foliage mesh.
  position: vec3<f32>,
};

fn meshlet_corner(vi: u32, ii: u32) -> MeshletCorner {
  var r: MeshletCorner;
  let item = meshlet_items[ii];
  let c = meshlet_clusters[item.y * 3u];
  if (vi / 3u >= c.y) {
    r.ok = false;
    return r;
  }
  r.ok = true;
  r.index = meshlet_indices[c.x + vi];
  let t0 = meshlet_instances[item.x * 4u];
  let t1 = meshlet_instances[item.x * 4u + 1u];
  let t2 = meshlet_instances[item.x * 4u + 2u];
  r.model = instance_matrix(t0.xyz, vec3<f32>(t0.w, t1.xy), vec3<f32>(t1.zw, t2.x), t2.yzw);
  r.position = meshlet_vertices[r.index * 4u].xyz;
  if (c.w != 0u) {
    let at = meshlet_sway_row(c.w, r.index);
    let sway = foliage_sway(meshlet_instances[item.x * 4u + 3u].x);
    r.position += meshlet_sway(r.position, meshlet_foliage[c.w - 1u], meshlet_foliage[at], meshlet_foliage[at + 1u], r.model, meshlet_draw.wind[0], sway);
  }
  return r;
}

// As `vs_lean`: the position alone (casters drawn with `fs_lean`).
@vertex
fn vs_meshlet_lean(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> LeanVsOut {
  let p = meshlet_corner(vi, ii);
  var o: LeanVsOut;
  if (!p.ok) {
    o.clip = vec4<f32>(0.0);
    return o;
  }
  let world = p.model * vec4<f32>(p.position, 1.0);
  o.clip = shadow_project(world, mul_point(shadow_pass.view_proj, world));
  o.world = world.xyz;
  return o;
}

// As `vs`: the whole vertex, for the alpha-tested and dissolving casters
// (`fs_dissolve`, the material at group 1).
@vertex
fn vs_meshlet(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VsOut {
  let p = meshlet_corner(vi, ii);
  if (!p.ok) {
    var o: VsOut;
    o.clip = vec4<f32>(0.0);
    return o;
  }
  let a = meshlet_vertices[p.index * 4u];
  let b = meshlet_vertices[p.index * 4u + 1u];
  var v: VertexIn;
  v.position = p.position;
  v.normal = vec3<f32>(a.w, b.xy);
  v.uv = b.zw;
  v.color = meshlet_vertices[p.index * 4u + 2u];
  return shadow_vertex(p.model, v);
}
