// The sprite batch's triangles (RenderScene::sprite_meshes: a 2D
// skeleton's region and mesh attachments, sprite_mesh.rs): one vertex per
// triangle corner in world space, with its uv, its colour, its colour
// offset and whether 2D lighting applies. Appended to sprite.wgsl (after
// frame.wgsl and lighting.wgsl): `fs_mesh` colours a texel as its `fs`
// colours a sprite's (no flash, round mask or drop shadow) with the
// offset added after the multiplier, so a mesh is textured, fogged and
// lit from the light map exactly as a sprite of its layer is.

struct MeshIn {
  @location(0) position: vec3<f32>,
  @location(1) uv: vec2<f32>,
  @location(2) color: vec4<f32>,
  @location(3) lit: f32,
  @location(4) offset: vec4<f32>,
  @location(5) skin: vec2<u32>,
};

struct MeshOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) uv: vec2<f32>,
  @location(1) color: vec4<f32>,
  @location(2) offset: vec4<f32>,
  @location(3) world: vec3<f32>,
  @location(4) lit: f32,
};

@vertex
fn vs_mesh(m: MeshIn) -> MeshOut {
  var o: MeshOut;
  o.clip = frame.view_proj * vec4<f32>(m.position, 1.0);
  o.uv = m.uv;
  o.color = m.color;
  o.offset = m.offset;
  o.world = m.position;
  o.lit = m.lit;
  return o;
}

// texel * color + offset (DragonBones' colour transform), never below 0,
// its alpha at most 1.
@fragment
fn fs_mesh(in: MeshOut) -> @location(0) vec4<f32> {
  var c = max(textureSample(sprite_tex, sprite_sampler, in.uv) * in.color + in.offset, vec4<f32>(0.0));
  c.a = min(c.a, 1.0);
  if (c.a <= 0.0) { discard; }
  if (in.lit > 0.5) {
    let light = light_at(in.clip.xy, in.lit);
    c = vec4<f32>(c.rgb * light, c.a);
  }
  return vec4<f32>(apply_fog(c.rgb, in.world), c.a);
}

// `fs_mesh` with the texture's normal map (sprite_normals.rs): the tangent
// frame from the screen derivatives follows the bent mesh.
@fragment
fn fs_mesh_normal(in: MeshOut) -> @location(0) vec4<f32> {
  let n = normal_at(in.uv, in.world.xy);
  var c = max(textureSample(sprite_tex, sprite_sampler, in.uv) * in.color + in.offset, vec4<f32>(0.0));
  c.a = min(c.a, 1.0);
  if (c.a <= 0.0) { discard; }
  if (in.lit > 0.5) {
    c = vec4<f32>(c.rgb * normal_light(in.clip.xy, in.lit, n), c.a);
  }
  return vec4<f32>(apply_fog(c.rgb, in.world), c.a);
}
