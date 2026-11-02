// The overlay: sprites and MSDF text in screen space (CSS pixels, origin top
// left, y down), drawn after every post effect straight into the output with
// no depth test, alpha blended over the finished frame. Colours are linear;
// an sRGB output encodes them in hardware, any other output (a WebGPU
// canvas's Bgra8Unorm) here. Each instance carries its clip rectangle
// (overlay space, x0 y0 x1 y1); fragments outside it are discarded.

// The frame uniform and the clip shapes are ui_common.wgsl's (prepended):
// a clip with a shape (UiClip) cuts sprites and text as it cuts shapes, and
// a layer (UiLayer) draws them offset by its origin.

struct SpriteParams {
  // x: MSDF distance range in atlas pixels (0: a plain sprite), yz: atlas
  // size in pixels; w: the glass pyramid's mip levels (src/glass.rs).
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
  @location(5) effect: vec4<f32>,             // MSDF text: x glow radius (CSS px), y weight (CSS px), z 1: the glow pass
  @location(6) effect_color: vec4<f32>,       // the glow's colour; glass: a its opacity
  @location(7) effect2: vec4<f32>,            // MSDF text: x outline inside, y outline outside or shadow softness (CSS px), z gradient + 1
  @location(8) clip_shape: u32,               // the clip's shape (UiClip) + 1, 0: none
  @location(9) shear: vec3<f32>,              // MSDF text under a map: x scale - 1, skew, y scale - 1 before the turn
};

struct OverlayOut {
  @builtin(position) position: vec4<f32>,
  @location(0) uv: vec2<f32>,
  @location(1) color: vec4<f32>,
  @location(2) clip: vec4<f32>,
  @location(3) effect: vec4<f32>,
  @location(4) effect_color: vec4<f32>,
  @location(5) effect2: vec4<f32>,
  @location(6) @interpolate(flat) clip_shape: u32,
};

@vertex
fn vs(@builtin(vertex_index) vi: u32, s: OverlayIn) -> OverlayOut {
  var corners = array<vec2<f32>, 6>(
    vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0),
    vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0));
  let c = corners[vi];
  let quad = (c - s.size_anchor.zw) * s.size_anchor.xy;
  let local = vec2<f32>((1.0 + s.shear.x) * quad.x + s.shear.y * quad.y, (1.0 + s.shear.z) * quad.y);
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
  o.effect2 = vec4<f32>(s.effect2.xy * frame.output.z, s.effect2.zw);
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

@fragment
fn fs(in: OverlayOut) -> @location(0) vec4<f32> {
  let c = textureSample(sprite_tex, sprite_sampler, in.uv) * in.color;
  return finish(c.rgb, c.a, in.position.xy, in.clip, in.clip_shape);
}

fn median(r: f32, g: f32, b: f32) -> f32 {
  return max(min(r, g), min(max(r, g), b));
}

@fragment
fn fs_msdf(in: OverlayOut) -> @location(0) vec4<f32> {
  let m = textureSample(sprite_tex, sprite_sampler, in.uv);
  let unit_range = vec2<f32>(sprite_params.msdf.x) / sprite_params.msdf.yz;
  let screen_tex_size = vec2<f32>(1.0) / fwidth(in.uv);
  let screen_px_range = max(0.5 * dot(unit_range, screen_tex_size), 1.0);
  // Signed distance to the glyph's edge in device pixels, inside positive,
  // thickened by the weight. The field is flat past half its range, so the
  // weight stays inside that reach (a heavier one would fill the glyph's
  // whole quad where the field is thin: small text from a narrow range).
  let reach = 0.5 * screen_px_range - 0.5;
  let weight = min(in.effect.y, max(0.5 * screen_px_range - 0.75, 0.0));
  let distance = screen_px_range * (median(m.r, m.g, m.b) - 0.5) + weight;
  let fill = clamp(distance + 0.5, 0.0, 1.0);
  // An MTSDF's alpha is the true distance (round past corners, where the
  // median runs on along the edges); a plain MSDF's alpha is 1, so the
  // smaller of the two is the median there.
  let true_distance = screen_px_range * (min(median(m.r, m.g, m.b), m.a) - 0.5) + weight;
  let kind = u32(in.effect.z + 0.5);
  switch kind {
    case 1u: {
      // The glow pass: the distance outside the ink, falling off
      // quadratically to the radius. The field holds distances up to half
      // its range: past that it is flat, so the glow ends inside it (never
      // at the glyph quad's hard edge).
      let outside = max(-true_distance, 0.0);
      let k = clamp(1.0 - outside / max(min(in.effect.x, reach), 1e-3), 0.0, 1.0);
      let a = in.effect_color.a * k * k * (1.0 - fill) * in.color.a;
      return finish(in.effect_color.rgb, a, in.position.xy, in.clip, in.clip_shape);
    }
    case 2u: {
      // A shadow (UiTextShadow): the moved glyph, its edge softened over
      // the blur, which the field's reach bounds.
      let soft = clamp(in.effect2.y, 1.0, max(2.0 * reach, 1.0));
      let k = clamp(true_distance / soft + 0.5, 0.0, 1.0);
      return finish(in.effect_color.rgb, in.effect_color.a * k * in.color.a, in.position.xy, in.clip, in.clip_shape);
    }
    case 3u, 4u: {
      // An outline (UiTextOutline): the band from `inside` within the ink
      // to `outside` beyond it, each bounded by the field's reach. Drawn
      // under the fill (3: outside) or over it (4: inside or centred).
      let outer = min(in.effect2.y, reach);
      let inner = min(in.effect2.x, reach);
      let d = select(distance, true_distance, distance < 0.0);
      var a = clamp(d + outer + 0.5, 0.0, 1.0);
      if (kind == 4u) {
        a *= 1.0 - clamp(d - inner + 0.5, 0.0, 1.0);
      }
      return finish(in.effect_color.rgb, in.effect_color.a * a * in.color.a, in.position.xy, in.clip, in.clip_shape);
    }
    default: {
      var color = in.color;
      if (in.effect2.z > 0.5) {
        // A gradient fill over the text's box.
        let gi = u32(in.effect2.z - 0.5);
        let g = gradients[gi];
        let css = css_of(in.position.xy);
        let c = gradient_color(gi, (css - g.box.xy) / max(g.box.zw, vec2<f32>(1e-4)), g.box.zw, in.position.xy);
        color = vec4<f32>(c.rgb, c.a * in.color.a);
      }
      return finish(color.rgb, ink_cover(fill, color.rgb) * color.a, in.position.xy, in.clip, in.clip_shape);
    }
  }
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
