// Linear blend skinning, as skin_storage.wgsl, with an arbitrary number
// of bone-local influences (2D rigs may deform each influence separately).
struct SpriteInfluence {
  position: vec2<f32>,
  weight: f32,
  bone: u32,
};
// Data textures (data_texture.rs): a bone four texels, an influence one;
// a vertex stage reads no storage under WebGPU's compatibility mode.
@group(3) @binding(0) var sprite_bones: texture_2d<f32>;
@group(3) @binding(1) var sprite_influences: texture_2d<u32>;

fn sprite_bone(i: u32) -> mat4x4<f32> {
  return mat4x4<f32>(
    textureLoad(sprite_bones, data_at(i * 4u), 0),
    textureLoad(sprite_bones, data_at(i * 4u + 1u), 0),
    textureLoad(sprite_bones, data_at(i * 4u + 2u), 0),
    textureLoad(sprite_bones, data_at(i * 4u + 3u), 0),
  );
}

fn sprite_influence(i: u32) -> SpriteInfluence {
  let t = textureLoad(sprite_influences, data_at(i), 0);
  return SpriteInfluence(bitcast<vec2<f32>>(t.xy), bitcast<f32>(t.z), t.w);
}

@vertex
fn vs_mesh_skin(m: MeshIn) -> MeshOut {
  var position = m.position;
  if (m.skin.y != 0u) {
    position = vec3<f32>(0.0);
    for (var k = 0u; k < m.skin.y; k += 1u) {
      let influence = sprite_influence(m.skin.x + k);
      position += (sprite_bone(influence.bone) * vec4<f32>(influence.position, 0.0, 1.0)).xyz * influence.weight;
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
