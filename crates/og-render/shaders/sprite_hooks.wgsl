// Sprites drawn with a material shader (sprite_hooks.rs): the sprite
// module, then the material's parameters (group 3), the shader's hooks
// (hooks.rs `hooks_section`) and these entries. A hooked sprite is the
// same quad, texture, outline, flash and 2D light as any sprite; its
// corners move by `vertex_offset`, its colour (texture times tint, before
// the light) goes through `surface_color`, and `surface_emissive` adds
// light after the 2D light. `surface_normal` is not called.
//
// For a sprite the hooks see: `world` the point in the world, `local` the
// point on the quad (x and y 0 to 1 from its bottom-left corner, z 0),
// `normal` +z, `uv` the texture coordinate (v down), `color` the
// sprite's tint, `time` the scene clock, `eye` the camera, `front` 1.

// The material's parameters (`MaterialUniform::custom` of the lit
// shader's material, the same values og.render.setShaderParam writes).
// Sprites draw no motion: `previous_shader_params()` reads these too.
struct SpriteMaterial {
  custom: array<vec4<f32>, 8>,
};
@group(3) @binding(0) var<uniform> material: SpriteMaterial;

// The sprite's texture at uv (mip 0), for hooks that read it elsewhere
// than the fragment's own texel: distortion, chromatic splits, outlines.
fn sprite_texel(uv: vec2<f32>) -> vec4<f32> {
  return textureSampleLevel(sprite_tex, sprite_sampler, uv, 0.0);
}

fn sprite_hook(world: vec3<f32>, corner: vec2<f32>, uv: vec2<f32>, color: vec4<f32>) -> Hook {
  var h: Hook;
  h.world = world;
  h.local = vec3<f32>(corner, 0.0);
  h.normal = vec3<f32>(0.0, 0.0, 1.0);
  h.uv = uv;
  h.color = color;
  h.time = frame.time.x;
  h.eye = frame.view_pos.xyz;
  h.front = 1.0;
  h.origin = frame.origin.xyz;
  return h;
}

@vertex
fn vs_hooked(@builtin(vertex_index) vi: u32, s: SpriteIn) -> SpriteOut {
  var o = sprite_vertex(vi, s);
  let h = sprite_hook(o.world, o.corner, o.uv, s.color);
  o.world = o.world + vertex_offset(h, shader_params());
  o.clip = frame.view_proj * vec4<f32>(o.world, 1.0);
  return o;
}

fn fragment_hook(in: SpriteOut) -> Hook {
  return sprite_hook(in.world, in.corner, in.uv, in.color);
}

// The sprite's colour through surface_color. A drop shadow keeps its
// black and takes only the alpha (a dissolve cuts the shadow too).
fn hooked_color(in: SpriteOut) -> vec4<f32> {
  let c = surface_color(fragment_hook(in), shader_params(), sprite_color(in));
  if (in.shadow > 0.5) { return vec4<f32>(0.0, 0.0, 0.0, c.a); }
  return c;
}

// The light it gives off (none for a drop shadow).
fn hooked_emissive(in: SpriteOut) -> vec3<f32> {
  if (in.shadow > 0.5) { return vec3<f32>(0.0); }
  return surface_emissive(fragment_hook(in), shader_params(), vec3<f32>(0.0));
}

// sprite.wgsl `fs` with the hooks.
@fragment
fn fs_hooked(in: SpriteOut) -> @location(0) vec4<f32> {
  var c = hooked_color(in);
  if (in.shadow > 0.5) {
    c.a *= drop_shadow_visible(in.world);
  }
  if (c.a <= 0.0) { discard; }
  if (in.fx.y > 0.5) {
    c = vec4<f32>(c.rgb * light_at(in.clip.xy, in.fx.y), c.a);
  }
  c = vec4<f32>(c.rgb + hooked_emissive(in), c.a);
  return vec4<f32>(apply_fog(c.rgb, in.world), c.a);
}

// sprite.wgsl `fs_untonemapped` with the hooks.
@fragment
fn fs_hooked_untonemapped(in: SpriteOut) -> @location(0) vec4<f32> {
  var c = hooked_color(in);
  if (c.a <= 0.0) { discard; }
  if (in.fx.y > 0.5) {
    c = vec4<f32>(c.rgb * light_at(in.clip.xy, in.fx.y), c.a);
  }
  return vec4<f32>(c.rgb + hooked_emissive(in), c.a);
}
