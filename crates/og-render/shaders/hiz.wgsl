// The occluders' depth pyramid for GPU culling (cull_gpu.rs): level 0 is
// the farthest sample of each pixel of the occluder pass's depth, each
// level after it the farthest of the texels under each of its own (2 x 2,
// and 3 at an odd edge), so one texel bounds everything it covers.

@group(0) @binding(0) var level_in: texture_2d<f32>;
@group(0) @binding(1) var level_out: texture_storage_2d<r32float, write>;
// cull_gpu.rs prepends `depth_in` (single or multisampled) and
// `farthest_sample(xy)`.

@compute @workgroup_size(8, 8)
fn base(@builtin(global_invocation_id) gid: vec3<u32>) {
  let dims = textureDimensions(level_out);
  if (gid.x >= dims.x || gid.y >= dims.y) { return; }
  textureStore(level_out, gid.xy, vec4<f32>(farthest_sample(gid.xy), 0.0, 0.0, 1.0));
}

@compute @workgroup_size(8, 8)
fn reduce(@builtin(global_invocation_id) gid: vec3<u32>) {
  let dims = textureDimensions(level_out);
  if (gid.x >= dims.x || gid.y >= dims.y) { return; }
  let below = textureDimensions(level_in, 0);
  let x0 = gid.x * 2u;
  let y0 = gid.y * 2u;
  // An odd size below folds its last row or column into the last texel.
  let x1 = select(x0 + 1u, below.x - 1u, gid.x == dims.x - 1u);
  let y1 = select(y0 + 1u, below.y - 1u, gid.y == dims.y - 1u);
  var depth = 0.0;
  for (var y = y0; y <= y1; y++) {
    for (var x = x0; x <= x1; x++) {
      depth = max(depth, textureLoad(level_in, vec2<u32>(x, y), 0).r);
    }
  }
  textureStore(level_out, gid.xy, vec4<f32>(depth, 0.0, 0.0, 1.0));
}
