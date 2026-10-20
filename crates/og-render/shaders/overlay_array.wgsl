// Overlay sprites drawn from a layer of a texture array (Overlay::sprite_arrays):
// overlay.wgsl's screen-space quads (CSS pixels, origin top left, y down,
// clipped, alpha blended over the finished frame) sampling a
// texture_2d_array, the layer in the instance's effect.w (which overlay
// sprites leave unused), so every overlay sprite of one array is one
// instanced draw whichever layer each shows. On WebGL2 the array arrives as
// a sampler2DArray (padded by gl_safe_array_layers, as the world's).

struct OverlayFrame {
  // x, y: the output in device pixels; z: device pixels per CSS pixel;
  // w: 1 when the shader encodes sRGB (a non-sRGB output).
  output: vec4<f32>,
};
@group(0) @binding(0) var<uniform> overlay: OverlayFrame;

@group(1) @binding(0) var sprite_array: texture_2d_array<f32>;
@group(1) @binding(1) var sprite_array_sampler: sampler;

struct OverlayIn {
  @location(0) position_rotation: vec4<f32>,  // xy: anchor position (CSS px), w: rotation, clockwise
  @location(1) size_anchor: vec4<f32>,        // xy: size, zw: anchor (fraction from the top left)
  @location(2) uv_rect: vec4<f32>,            // u0, v0, u1, v1 (v down)
  @location(3) color: vec4<f32>,
  @location(4) clip: vec4<f32>,               // x0, y0, x1, y1 in CSS px
  @location(5) effect: vec4<f32>,             // w: the layer
  @location(6) effect_color: vec4<f32>,       // unused
};

struct OverlayOut {
  @builtin(position) position: vec4<f32>,
  @location(0) uv: vec2<f32>,
  @location(1) color: vec4<f32>,
  @location(2) clip: vec4<f32>,
  @location(3) @interpolate(flat) layer: u32,
};

@vertex
fn vs(@builtin(vertex_index) vi: u32, s: OverlayIn) -> OverlayOut {
  var corners = array<vec2<f32>, 6>(
    vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0),
    vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0));
  let c = corners[vi];
  let local = (c - s.size_anchor.zw) * s.size_anchor.xy;
  let cs = cos(s.position_rotation.w);
  let sn = sin(s.position_rotation.w);
  let rotated = vec2<f32>(local.x * cs - local.y * sn, local.x * sn + local.y * cs);
  let device = (s.position_rotation.xy + rotated) * overlay.output.z;
  var o: OverlayOut;
  o.position = vec4<f32>(device.x / overlay.output.x * 2.0 - 1.0, 1.0 - device.y / overlay.output.y * 2.0, 0.0, 1.0);
  o.uv = vec2<f32>(mix(s.uv_rect.x, s.uv_rect.z, c.x), mix(s.uv_rect.y, s.uv_rect.w, c.y));
  o.color = s.color;
  o.clip = s.clip * overlay.output.z;
  o.layer = u32(max(s.effect.w, 0.0) + 0.5);
  return o;
}

fn encode(c: vec3<f32>) -> vec3<f32> {
  let x = clamp(c, vec3<f32>(0.0), vec3<f32>(1.0));
  let lo = x * 12.92;
  let hi = 1.055 * pow(x, vec3<f32>(1.0 / 2.4)) - 0.055;
  return select(hi, lo, x <= vec3<f32>(0.0031308));
}

@fragment
fn fs(in: OverlayOut) -> @location(0) vec4<f32> {
  let c = textureSample(sprite_array, sprite_array_sampler, in.uv, in.layer) * in.color;
  let at = in.position.xy;
  if (at.x < in.clip.x || at.y < in.clip.y || at.x >= in.clip.z || at.y >= in.clip.w || c.a <= 0.0) {
    discard;
  }
  if (overlay.output.w > 0.5) {
    return vec4<f32>(encode(c.rgb), c.a);
  }
  return c;
}
