// Linear blend skinning, as skin_storage.wgsl, with an arbitrary number
// of bone-local influences (2D rigs may deform each influence separately).
struct SpriteInfluence {
  position: vec2<f32>,
  weight: f32,
  bone: u32,
};
@group(3) @binding(0) var<storage, read> sprite_bones: array<mat4x4<f32>>;
@group(3) @binding(1) var<storage, read> sprite_influences: array<SpriteInfluence>;

@vertex
fn vs_mesh_skin(m: MeshIn) -> MeshOut {
  var position = m.position;
  if (m.skin.y != 0u) {
    position = vec3<f32>(0.0);
    for (var k = 0u; k < m.skin.y; k += 1u) {
      let influence = sprite_influences[m.skin.x + k];
      position += (sprite_bones[influence.bone] * vec4<f32>(influence.position, 0.0, 1.0)).xyz * influence.weight;
    }
  }
  var o: MeshOut;
  o.clip = frame.view_proj * vec4<f32>(position, 1.0);
  o.uv = m.uv;
  o.color = m.color;
  o.offset = m.offset;
  o.world = position;
  o.lit = m.lit;
  return o;
}
