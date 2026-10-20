// Sprites drawn from a layer of a texture array (RenderScene::sprite_arrays,
// and 2D particles of an emitter with an array): sprite.wgsl's quads with
// the instance's layer in location 7, sampling a texture_2d_array, so every
// sprite of one array is one instanced draw whichever layer (frame) each
// shows. Hit flash, 2D lighting, drop shadows (location 6, as sprite.wgsl),
// the soft dot (a particle's round edge over its layer) and the glow
// (`fs_glow` adds glowing array sprites into the glow buffer) as
// sprite.wgsl; no MSDF text. On WebGL2 the array arrives as a
// sampler2DArray: og-render pads a one-layer or multiple-of-six array
// (gl_safe_array_layers) so the GL backend keeps it a 2D array.
// Prepended by frame.wgsl and lighting.wgsl.

@group(1) @binding(0) var sprite_array: texture_2d_array<f32>;
@group(1) @binding(1) var sprite_array_sampler: sampler;

struct SpriteArrayFrame {
  screen: vec4<f32>,  // w, h, 1/w, 1/h
};
@group(2) @binding(0) var light_map: texture_2d<f32>;
@group(2) @binding(1) var light_sampler: sampler;
@group(2) @binding(2) var<uniform> sprite_fx: SpriteArrayFrame;

struct SpriteIn {
  @location(0) position_rotation: vec4<f32>,  // xyz: anchor position, w: rotation (radians)
  @location(1) size_anchor: vec4<f32>,        // xy: size, zw: anchor (fraction from bottom-left)
  @location(2) uv_rect: vec4<f32>,            // u0, v0, u1, v1 (v down)
  @location(3) color: vec4<f32>,
  @location(4) flash: vec4<f32>,              // rgb: flash colour, a: amount
  @location(5) fx: vec3<f32>,                 // x: glow, y: lit, z: soft dot
  @location(6) ground: vec4<f32>,             // a drop shadow's projection (sprite.wgsl)
  @location(7) layer: u32,
};

struct SpriteOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) uv: vec2<f32>,
  @location(1) color: vec4<f32>,
  @location(2) flash: vec4<f32>,
  @location(3) @interpolate(flat) layer: u32,
  @location(4) fx: vec3<f32>,
  @location(5) world: vec3<f32>,
  @location(6) corner: vec2<f32>,
  @location(7) shadow: f32,                   // 1 for a drop shadow
};

@vertex
fn vs(@builtin(vertex_index) vi: u32, s: SpriteIn) -> SpriteOut {
  var corners = array<vec2<f32>, 6>(
    vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0),
    vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0));
  let c = corners[vi];
  let local = (c - s.size_anchor.zw) * s.size_anchor.xy;
  let cs = cos(s.position_rotation.w);
  let sn = sin(s.position_rotation.w);
  let rotated = vec2<f32>(local.x * cs - local.y * sn, local.x * sn + local.y * cs);
  var world = vec3<f32>(s.position_rotation.xy + rotated, s.position_rotation.z);
  if (s.ground.x > 0.5) {
    // A drop shadow: down the sun to the level ground or the plane of a
    // slope (sprite.wgsl ground_shadow and sloped_shadow).
    let rise = s.fx.yz;
    if (rise.x == 0.0 && rise.y == 0.0) {
      let h = world.y - s.ground.w;
      world = vec3<f32>(world.x + s.ground.y * h, s.ground.w, world.z + s.ground.z * h);
    } else {
      let a = s.position_rotation.xyz;
      let above = world.y - s.ground.w - rise.x * (world.x - a.x) - rise.y * (world.z - a.z);
      let t = above / max(1.0 + rise.x * s.ground.y + rise.y * s.ground.z, 0.2);
      world = vec3<f32>(world.x + s.ground.y * t, world.y - t, world.z + s.ground.z * t);
    }
  }
  var o: SpriteOut;
  o.clip = frame.view_proj * vec4<f32>(world, 1.0);
  o.uv = vec2<f32>(mix(s.uv_rect.x, s.uv_rect.z, c.x), mix(s.uv_rect.w, s.uv_rect.y, c.y));
  o.color = s.color;
  o.flash = s.flash;
  o.layer = s.layer;
  o.fx = select(s.fx, vec3<f32>(0.0), s.ground.x > 0.5);
  o.world = world;
  o.corner = c;
  o.shadow = select(0.0, 1.0, s.ground.x > 0.5);
  return o;
}

// The sprite's colour before lighting: its layer, tint, soft dot, flash.
fn array_color(in: SpriteOut) -> vec4<f32> {
  var c = textureSample(sprite_array, sprite_array_sampler, in.uv, in.layer) * in.color;
  if (in.fx.z > 0.5) {
    let d = clamp(1.0 - length(in.corner * 2.0 - 1.0), 0.0, 1.0);
    c.a *= d * d;
  }
  if (in.flash.a > 0.0) {
    c = vec4<f32>(mix(c.rgb, in.flash.rgb, in.flash.a), c.a);
  }
  return c;
}

@fragment
fn fs(in: SpriteOut) -> @location(0) vec4<f32> {
  var c = array_color(in);
  if (in.shadow > 0.5) {
    // A drop shadow inside the cascades' shadow (sprite.wgsl).
    let l = normalize(frame.light_dir.xyz);
    c.a *= cascade_shadow(in.world, vec3<f32>(0.0, 1.0, 0.0), max(l.y, 0.05));
  }
  if (c.a <= 0.0) { discard; }
  if (in.fx.y > 0.5) {
    let light = textureSampleLevel(light_map, light_sampler, in.clip.xy * sprite_fx.screen.zw, 0.0).rgb;
    c = vec4<f32>(c.rgb * light, c.a);
  }
  return vec4<f32>(apply_fog(c.rgb, in.world), c.a);
}

// Into the glow buffer: the sprite's light, whatever covers it.
@fragment
fn fs_glow(in: SpriteOut) -> @location(0) vec4<f32> {
  let c = array_color(in);
  return vec4<f32>(c.rgb * c.a * in.fx.x, 0.0);
}
