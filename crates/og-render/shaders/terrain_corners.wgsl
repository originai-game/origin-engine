// The corner cache's writer (terrain_vertex.wgsl): once a frame, before
// any pass draws the terrain, every clipmap level's (n + 1)² corners, one
// invocation each, dispatched per level with its level slot bound.
// Prepended by terrain.wgsl, terrain_layers.wgsl and terrain_vertex.wgsl.

@group(0) @binding(0) var<storage, read_write> corners_out: array<vec4<u32>>;

// A corner's height (the terrain's space) is the one a level of its step
// draws at its clamped grid point: the field's, displaced near the camera
// by its layers' heights (terrain_relief) and lifted by lying snow. The
// coarser level's corners are its own, so both levels draw a corner they
// share alike.
@compute @workgroup_size(64)
fn write_corners(@builtin(global_invocation_id) id: vec3<u32>) {
  let n = u32(level.grid.x) + 1u;
  if (id.x >= n * n) { return; }
  let g = vec2<f32>(f32(id.x % n), f32(id.x / n));
  let c = grid_clamp(grid_corner(g));
  let s = c * level.camera.z / terrain_cell();
  let h = field_height(s);
  let r = terrain_relief(c, s, h, level.center.z);
  let total = h + r.d + terrain_snow_lift(s, level.center.z);
  corners_out[u32(level.corners.z) + id.x] = vec4<u32>(bitcast<u32>(total), corner_pack(r.slope), corner_pack(r.bare), 0u);
}
