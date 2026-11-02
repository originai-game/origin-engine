// The meshlet path's early phase into GPU culling's occluder depth
// (meshlet_gpu.rs): the clusters visible last frame, drawn at this frame's
// camera before the pyramid is built again, pulled as meshlet.wgsl pulls
// them (group 1 is its draw group). After instance.wgsl.

// meshlet_cull.wgsl's `Params`, of which this reads the camera.
struct DepthView {
  view_proj: mat4x4<f32>,
};
@group(0) @binding(0) var<uniform> depth_view: DepthView;
@group(1) @binding(9) var<storage, read> meshlet_instances: array<vec4<f32>>;
@group(1) @binding(10) var<storage, read> meshlet_items: array<vec2<u32>>;
@group(1) @binding(11) var<storage, read> meshlet_clusters: array<vec4<u32>>;
@group(1) @binding(12) var<storage, read> meshlet_indices: array<u32>;
@group(1) @binding(13) var<storage, read> meshlet_vertices: array<vec4<f32>>;

@vertex
fn vs_depth(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> @builtin(position) vec4<f32> {
  let item = meshlet_items[ii];
  let c = meshlet_clusters[item.y * 3u];
  // A corner past the cluster's triangles: degenerate (w 0, clipped).
  if (vi / 3u >= c.y) { return vec4<f32>(0.0); }
  let index = meshlet_indices[c.x + vi];
  let t0 = meshlet_instances[item.x * 4u];
  let t1 = meshlet_instances[item.x * 4u + 1u];
  let t2 = meshlet_instances[item.x * 4u + 2u];
  let model = instance_matrix(t0.xyz, vec3<f32>(t0.w, t1.xy), vec3<f32>(t1.zw, t2.x), t2.yzw);
  let world = model * vec4<f32>(meshlet_vertices[index * 4u].xyz, 1.0);
  return mul_point(depth_view.view_proj, world);
}
