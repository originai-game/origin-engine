// The sprite batch's triangles (RenderScene::sprite_meshes: a 2D
// skeleton's region and mesh attachments, sprite_mesh.rs): one vertex per
// triangle corner in world space, with its uv, its colour and whether 2D
// lighting applies. Appended to sprite.wgsl (after frame.wgsl and
// lighting.wgsl), whose `fs` it shares, so a mesh is textured, fogged and
// lit from the light map exactly as a sprite of its layer is.

struct MeshIn {
  @location(0) position: vec3<f32>,
  @location(1) uv: vec2<f32>,
  @location(2) color: vec4<f32>,
  @location(3) lit: f32,
};

@vertex
fn vs_mesh(m: MeshIn) -> SpriteOut {
  var o: SpriteOut;
  o.clip = frame.view_proj * vec4<f32>(m.position, 1.0);
  o.uv = m.uv;
  o.color = m.color;
  o.flash = vec4<f32>(0.0);
  o.fx = vec3<f32>(0.0, m.lit, 0.0);
  o.corner = vec2<f32>(0.5);
  o.world = m.position;
  o.shadow = 0.0;
  return o;
}
