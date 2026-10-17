// Sprites and MSDF text: instanced quads in the world's xy plane, alpha
// blended into the HDR target after the 3D scene. One instance per sprite or
// glyph; the quad's corners come from the vertex index (no vertex buffer).
// Prepended by frame.wgsl.

struct SpriteParams {
  // x: MSDF distance range in atlas pixels (0: a plain sprite), yz: atlas size in pixels.
  msdf: vec4<f32>,
};
@group(1) @binding(0) var sprite_tex: texture_2d<f32>;
@group(1) @binding(1) var sprite_sampler: sampler;
@group(1) @binding(2) var<uniform> sprite_params: SpriteParams;

struct SpriteIn {
  @location(0) position_rotation: vec4<f32>,  // xyz: anchor position, w: rotation (radians)
  @location(1) size_anchor: vec4<f32>,        // xy: size, zw: anchor (fraction from bottom-left)
  @location(2) uv_rect: vec4<f32>,            // u0, v0, u1, v1 (v down)
  @location(3) color: vec4<f32>,
};

struct SpriteOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) uv: vec2<f32>,
  @location(1) color: vec4<f32>,
};

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
  let world = vec3<f32>(s.position_rotation.xy + rotated, s.position_rotation.z);
  var o: SpriteOut;
  o.clip = frame.view_proj * vec4<f32>(world, 1.0);
  // Bottom of the quad samples the bottom of the region (v grows downwards).
  o.uv = vec2<f32>(mix(s.uv_rect.x, s.uv_rect.z, c.x), mix(s.uv_rect.w, s.uv_rect.y, c.y));
  o.color = s.color;
  return o;
}

@fragment
fn fs(in: SpriteOut) -> @location(0) vec4<f32> {
  let c = textureSample(sprite_tex, sprite_sampler, in.uv) * in.color;
  if (c.a <= 0.0) { discard; }
  return c;
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
  let distance = screen_px_range * (median(m.r, m.g, m.b) - 0.5);
  let opacity = clamp(distance + 0.5, 0.0, 1.0) * in.color.a;
  if (opacity <= 0.0) { discard; }
  return vec4<f32>(in.color.rgb, opacity);
}
