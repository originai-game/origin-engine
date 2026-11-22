// The overlay: sprites and text in screen space (CSS pixels, origin top
// left, y down), drawn after every post effect straight into the output with
// no depth test, alpha blended over the finished frame. Colours are linear;
// an sRGB output encodes them in hardware, any other output (a WebGPU
// canvas's Bgra8Unorm) here. Each instance carries its clip rectangle
// (overlay space, x0 y0 x1 y1); fragments outside it are discarded.

// The frame uniform and the clip shapes are ui_common.wgsl's (prepended):
// a clip with a shape (UiClip) cuts sprites and text as it cuts shapes, and
// a layer (UiLayer) draws them offset by its origin.

struct SpriteParams {
  // x: a glyph atlas page's distance range in its pixels (0: a plain
  // sprite, -1: a page of colour glyph pairs, foreground_mix), yz: its
  // size in pixels; w:
  // the glass pyramid's mip levels (src/glass.rs).
  msdf: vec4<f32>,
};
@group(1) @binding(0) var sprite_tex: texture_2d<f32>;
@group(1) @binding(1) var sprite_sampler: sampler;
@group(1) @binding(2) var<uniform> sprite_params: SpriteParams;

struct OverlayIn {
  @location(0) position_rotation: vec4<f32>,  // xy: anchor position (CSS px), w: rotation, clockwise
  @location(1) size_anchor: vec4<f32>,        // xy: size, zw: anchor (fraction from the top left)
  @location(2) uv_rect: vec4<f32>,            // u0, v0, u1, v1 (v down)
  @location(3) color: vec4<f32>,
  @location(4) clip: vec4<f32>,               // x0, y0, x1, y1 in CSS px
  @location(5) effect: vec4<f32>,             // glass: its corner and blur radii and size; an array sprite: w its layer
  @location(6) effect_color: vec4<f32>,       // glass: a its opacity
  @location(7) clip_shape: u32,               // the clip's shape (UiClip) + 1, 0: none
};

struct OverlayOut {
  @builtin(position) position: vec4<f32>,
  @location(0) uv: vec2<f32>,
  @location(1) color: vec4<f32>,
  @location(2) clip: vec4<f32>,
  @location(3) effect: vec4<f32>,
  @location(4) effect_color: vec4<f32>,
  @location(5) @interpolate(flat, either) clip_shape: u32,
};

@vertex
fn vs(@builtin(vertex_index) vi: u32, s: OverlayIn) -> OverlayOut {
  var corners = array<vec2<f32>, 6>(
    vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0),
    vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0));
  let c = corners[vi];
  let quad = (c - s.size_anchor.zw) * s.size_anchor.xy;
  let local = quad;
  let cs = cos(s.position_rotation.w);
  let sn = sin(s.position_rotation.w);
  // y grows downwards, so this turns clockwise on screen.
  let rotated = vec2<f32>(local.x * cs - local.y * sn, local.x * sn + local.y * cs);
  let device = (s.position_rotation.xy + rotated - frame.origin.xy) * frame.output.z;
  var o: OverlayOut;
  o.position = vec4<f32>(device.x / frame.output.x * 2.0 - 1.0, 1.0 - device.y / frame.output.y * 2.0, 0.0, 1.0);
  // The top of the quad (c.y = 0) samples the top of the region.
  o.uv = vec2<f32>(mix(s.uv_rect.x, s.uv_rect.z, c.x), mix(s.uv_rect.y, s.uv_rect.w, c.y));
  o.color = s.color;
  o.clip = (s.clip - frame.origin.xyxy) * frame.output.z;
  // Lengths in device pixels, as the fragment measures distance.
  o.effect = vec4<f32>(s.effect.xy * frame.output.z, s.effect.zw);
  o.effect_color = s.effect_color;
  o.clip_shape = s.clip_shape;
  return o;
}

fn finish(rgb: vec3<f32>, a0: f32, at: vec2<f32>, clip: vec4<f32>, shape: u32) -> vec4<f32> {
  var a = a0;
  if (shape != 0u) {
    a *= clip_coverage(shape, css_of(at));
  }
  if (at.x < clip.x || at.y < clip.y || at.x >= clip.z || at.y >= clip.w || a <= 0.0) {
    discard;
  }
  return finish_color(rgb, a);
}

// A colour glyph in the text's colour (MsdfFont::foreground): the glyph
// painted with a black foreground and its white-foreground twin half the
// page to the right, mixed channel by channel by the text's colour `fg`
// in sRGB, where the cook painted them: exact wherever the paint is
// affine in the foreground (fills, gradients, Porter-Duff composites).
fn foreground_mix(black: vec4<f32>, uv: vec2<f32>, fg: vec3<f32>) -> vec4<f32> {
  let white = textureSample(sprite_tex, sprite_sampler, uv + vec2<f32>(0.5, 0.0));
  return vec4<f32>(decode(mix(encode(black.rgb), encode(white.rgb), encode(fg))), black.a);
}

@fragment
fn fs(in: OverlayOut) -> @location(0) vec4<f32> {
  var c = textureSample(sprite_tex, sprite_sampler, in.uv);
  if (sprite_params.msdf.x < 0.0) {
    c = foreground_mix(c, in.uv, in.color.rgb) * vec4<f32>(1.0, 1.0, 1.0, in.color.a);
  } else {
    c *= in.color;
  }
  return finish(c.rgb, c.a, in.position.xy, in.clip, in.clip_shape);
}

fn median(r: f32, g: f32, b: f32) -> f32 {
  return max(min(r, g), min(max(r, g), b));
}

// A text's glyph from its distance field in the glyph atlas: the field's
// range scaled to the screen, the edge where the median crosses half.
@fragment
fn fs_msdf(in: OverlayOut) -> @location(0) vec4<f32> {
  let m = textureSample(sprite_tex, sprite_sampler, in.uv);
  let unit_range = vec2<f32>(sprite_params.msdf.x) / sprite_params.msdf.yz;
  let screen_tex_size = vec2<f32>(1.0) / fwidthFine(in.uv);
  let screen_px_range = max(0.5 * dot(unit_range, screen_tex_size), 1.0);
  let fill = clamp(screen_px_range * (median(m.r, m.g, m.b) - 0.5) + 0.5, 0.0, 1.0);
  return finish(in.color.rgb, ink_cover(fill, in.color.rgb) * in.color.a, in.position.xy, in.clip, in.clip_shape);
}

@fragment
fn fs_glass(in: OverlayOut) -> @location(0) vec4<f32> {
  // A glass panel (src/glass.rs): the blurred frame under it, tinted by its
  // colour (rgb, mixed in by alpha), masked to its rounded rectangle. uv
  // spans the quad 0..1; effect: x the corner radius and y the blur radius
  // in device pixels, zw the quad's size in CSS pixels; effect_color.a the
  // panel's opacity.
  let size = in.effect.zw * frame.output.z;
  let p = (in.uv - vec2<f32>(0.5)) * size;
  let r = min(in.effect.x, 0.5 * min(size.x, size.y));
  let q = abs(p) - (0.5 * size - vec2<f32>(r));
  let d = length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - r;
  let mask = clamp(0.5 - d, 0.0, 1.0);
  // Level k of the pyramid is a blur of about 2^(k+1) device pixels.
  let levels = sprite_params.msdf.w;
  let lod = clamp(log2(max(in.effect.y, 1.0)) - 1.0, 0.0, levels - 1.0);
  let at = in.position.xy / frame.output.xy;
  let behind = textureSampleLevel(sprite_tex, sprite_sampler, at, lod).rgb;
  let rgb = mix(behind, in.color.rgb, in.color.a);
  return finish(rgb, mask * in.effect_color.a, in.position.xy, in.clip, in.clip_shape);
}
