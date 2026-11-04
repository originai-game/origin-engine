// Immutable per-chunk geometry and animation; only placement, time, tint
// and the light strip change for a view. No simulation state is written.
struct TileQuad {
  min_size: vec4<f32>,
  uv: vec4<f32>,
  frames: vec2<u32>,
  total: f32,
  padding: f32,
};
struct TileFrame { uv: vec4<f32>, duration: vec4<f32> };
struct TileParams { origin_time: vec4<f32>, color: vec4<f32>, light: vec4<f32> };
@group(3) @binding(0) var<storage, read> tile_quads: array<TileQuad>;
@group(3) @binding(1) var<storage, read> tile_frames: array<TileFrame>;
@group(3) @binding(2) var<uniform> tile_params: TileParams;

@vertex
fn vs_tile(@builtin(vertex_index) vi: u32, @builtin(instance_index) qi: u32) -> SpriteOut {
  let q = tile_quads[qi];
  var uv = q.uv;
  if (q.frames.y != 0u) {
    var t = tile_params.origin_time.w % q.total;
    if (t < 0.0) { t += q.total; }
    for (var k = 0u; k < q.frames.y; k += 1u) {
      let f = tile_frames[q.frames.x + k];
      uv = f.uv;
      if (t < f.duration.x) { break; }
      t -= f.duration.x;
    }
  }
  var corners = array<vec2<f32>, 6>(vec2f(0, 0), vec2f(1, 0), vec2f(1, 1), vec2f(0, 0), vec2f(1, 1), vec2f(0, 1));
  let c = corners[vi];
  let world = vec3f(tile_params.origin_time.xy + q.min_size.xy + c * q.min_size.zw, tile_params.origin_time.z);
  var o: SpriteOut;
  o.clip = frame.view_proj * vec4f(world, 1.0);
  o.uv = vec2f(mix(uv.x, uv.z, c.x), mix(uv.w, uv.y, c.y));
  o.color = tile_params.color;
  o.fx = vec3f(0.0, tile_params.light.x, 0.0);
  o.corner = c;
  o.world = world;
  o.region = region_of(uv);
  return o;
}
