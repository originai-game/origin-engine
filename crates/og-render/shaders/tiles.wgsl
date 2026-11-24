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
struct TileParams {
  origin_time: vec4<f32>, color: vec4<f32>, light: vec4<f32>,
  axis_x: vec4<f32>, axis_y: vec4<f32>,
};
// Data textures (data_texture.rs) of the quads (three texels each) and the
// frames (two each): a vertex stage reads no storage under WebGPU's
// compatibility mode.
@group(3) @binding(0) var tile_quads: texture_2d<u32>;
@group(3) @binding(1) var tile_frames: texture_2d<u32>;

fn tile_quad(i: u32) -> TileQuad {
  let a = textureLoad(tile_quads, data_at(i * 3u), 0);
  let b = textureLoad(tile_quads, data_at(i * 3u + 1u), 0);
  let c = textureLoad(tile_quads, data_at(i * 3u + 2u), 0);
  return TileQuad(bitcast<vec4<f32>>(a), bitcast<vec4<f32>>(b), c.xy, bitcast<f32>(c.z), 0.0);
}

fn tile_frame(i: u32) -> TileFrame {
  let a = textureLoad(tile_frames, data_at(i * 2u), 0);
  let b = textureLoad(tile_frames, data_at(i * 2u + 1u), 0);
  return TileFrame(bitcast<vec4<f32>>(a), bitcast<vec4<f32>>(b));
}
@group(3) @binding(2) var<uniform> tile_params: TileParams;

@vertex
fn vs_tile(@builtin(vertex_index) vi: u32, @builtin(instance_index) qi: u32) -> SpriteOut {
  let q = tile_quad(qi);
  var uv = q.uv;
  if (q.frames.y != 0u) {
    var t = tile_params.origin_time.w % q.total;
    if (t < 0.0) { t += q.total; }
    for (var k = 0u; k < q.frames.y; k += 1u) {
      let f = tile_frame(q.frames.x + k);
      uv = f.uv;
      if (t < f.duration.x) { break; }
      t -= f.duration.x;
    }
  }
  let c = quad_corner(vi);
  let local = q.min_size.xy + c * q.min_size.zw;
  let world = tile_params.origin_time.xyz + tile_params.axis_x.xyz * local.x + tile_params.axis_y.xyz * local.y;
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
