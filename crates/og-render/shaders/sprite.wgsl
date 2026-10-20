// Sprites and MSDF text: instanced quads in the world's xy plane, alpha
// blended (or added) into the HDR target after the 3D scene. One instance
// per sprite or glyph; the quad's corners come from the vertex index (no
// vertex buffer). Per instance: a hit flash, 2D lighting from the light map
// (lit layers), a soft round dot for untextured particles, and a glow
// strength (`fs_glow` adds glowing sprites into the glow buffer).
// Prepended by frame.wgsl.

struct SpriteParams {
  // x: MSDF distance range in atlas pixels (0: a plain sprite), yz: atlas size in pixels.
  msdf: vec4<f32>,
};
@group(1) @binding(0) var sprite_tex: texture_2d<f32>;
@group(1) @binding(1) var sprite_sampler: sampler;
@group(1) @binding(2) var<uniform> sprite_params: SpriteParams;

struct SpriteFxFrame {
  screen: vec4<f32>,  // w, h, 1/w, 1/h
};
@group(2) @binding(0) var light_map: texture_2d<f32>;
@group(2) @binding(1) var light_sampler: sampler;
@group(2) @binding(2) var<uniform> sprite_fx: SpriteFxFrame;

struct SpriteIn {
  @location(0) position_rotation: vec4<f32>,  // xyz: anchor position, w: rotation (radians)
  @location(1) size_anchor: vec4<f32>,        // xy: size, zw: anchor (fraction from bottom-left)
  @location(2) uv_rect: vec4<f32>,            // u0, v0, u1, v1 (v down)
  @location(3) color: vec4<f32>,
  @location(4) flash: vec4<f32>,              // rgb: flash colour, a: amount
  @location(5) fx: vec3<f32>,                 // x: glow, y: lit, z: soft dot
  @location(6) ground: vec4<f32>,             // a drop shadow: x 1, yz: x and z moved per metre of height, w: the ground's height
};

struct SpriteOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) uv: vec2<f32>,
  @location(1) color: vec4<f32>,
  @location(2) flash: vec4<f32>,
  @location(3) fx: vec3<f32>,
  @location(4) corner: vec2<f32>,
  @location(5) world: vec3<f32>,
};

// A drop shadow's corner: the standing quad's corner carried along the sun
// down to the ground plane (sprites.rs shadow_ground); others unchanged.
fn ground_shadow(p: vec3<f32>, g: vec4<f32>) -> vec3<f32> {
  if (g.x < 0.5) { return p; }
  let h = p.y - g.w;
  return vec3<f32>(p.x + g.y * h, g.w, p.z + g.z * h);
}

@vertex
fn vs(@builtin(vertex_index) vi: u32, s: SpriteIn) -> SpriteOut {
  // Two triangles: (0,0) (1,0) (1,1) / (0,0) (1,1) (0,1), counter-clockwise.
  var corners = array<vec2<f32>, 6>(
    vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0),
    vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0));
  let c = corners[vi];
  let local = (c - s.size_anchor.zw) * s.size_anchor.xy;
  let cs = cos(s.position_rotation.w);
  let sn = sin(s.position_rotation.w);
  let rotated = vec2<f32>(local.x * cs - local.y * sn, local.x * sn + local.y * cs);
  let world = ground_shadow(vec3<f32>(s.position_rotation.xy + rotated, s.position_rotation.z), s.ground);
  var o: SpriteOut;
  o.clip = frame.view_proj * vec4<f32>(world, 1.0);
  // Bottom of the quad samples the bottom of the region (v grows downwards).
  o.uv = vec2<f32>(mix(s.uv_rect.x, s.uv_rect.z, c.x), mix(s.uv_rect.w, s.uv_rect.y, c.y));
  o.color = s.color;
  o.flash = s.flash;
  o.fx = s.fx;
  o.corner = c;
  o.world = world;
  return o;
}

// The sprite's colour before lighting: texture, tint, soft dot, flash.
fn sprite_color(in: SpriteOut) -> vec4<f32> {
  var c = textureSample(sprite_tex, sprite_sampler, in.uv) * in.color;
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
  var c = sprite_color(in);
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
  let c = sprite_color(in);
  return vec4<f32>(c.rgb * c.a * in.fx.x, 0.0);
}

fn median(r: f32, g: f32, b: f32) -> f32 {
  return max(min(r, g), min(max(r, g), b));
}

@fragment
fn fs_msdf(in: SpriteOut) -> @location(0) vec4<f32> {
  let m = textureSample(sprite_tex, sprite_sampler, in.uv).rgb;
  // The distance range in screen pixels: atlas range scaled by how many
  // screen pixels one atlas pixel covers here.
  let unit_range = vec2<f32>(sprite_params.msdf.x) / sprite_params.msdf.yz;
  let screen_tex_size = vec2<f32>(1.0) / fwidth(in.uv);
  let screen_px_range = max(0.5 * dot(unit_range, screen_tex_size), 1.0);
  let sd = median(m.r, m.g, m.b) - 0.5;
  let fill = clamp(screen_px_range * sd + 0.5, 0.0, 1.0);
  // An outline (fx.x: its width as a fraction of half the distance range,
  // flash.rgb its colour) grows the shape outwards and shows behind the fill.
  let w = clamp(in.fx.x, 0.0, 1.0) * 0.5;
  let outer = clamp(screen_px_range * (sd + w) + 0.5, 0.0, 1.0);
  let opacity = select(fill, outer, w > 0.0) * in.color.a;
  if (opacity <= 0.0) { discard; }
  let rgb = select(in.color.rgb, mix(in.flash.rgb, in.color.rgb, fill / max(outer, 1e-4)), w > 0.0);
  return vec4<f32>(apply_fog(rgb, in.world), opacity);
}
