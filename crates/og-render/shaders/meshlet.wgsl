// The meshlet path's vertex stages (meshlet.rs, meshlet_gpu.rs), after
// pbr.wgsl and pbr_prepass.wgsl in the standard lit module. A batch is one
// draw of 64 x 3 vertices per visible cluster: the instance index names a
// work item (an instance and a cluster, written by meshlet_cull.wgsl), the
// vertex index a corner of one of the cluster's triangles, read from the
// cluster pool's indices and the mesh's own f32 vertices (vertex pulling).
// A corner past the cluster's triangles is degenerate (w 0: clipped).

struct MeshletDraw {
  // x: 1 colours each instance by its level (RenderScene::lod_debug).
  flags: vec4<u32>,
};
@group(3) @binding(8) var<uniform> meshlet_draw: MeshletDraw;
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

struct MeshletVertex {
  ok: bool,
  v: VertexIn,
  model: mat4x4<f32>,
  look: vec4<f32>,
  // The instance's sorted slot.
  slot: u32,
};

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
  var o = lit_vertex(p.model, p.v);
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
  return prepass_vertex(p.model, prev, p.v);
}
