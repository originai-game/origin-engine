// A layer's composite (src/ui/pass.rs), after ui_common.wgsl: the layer's
// offscreen target read through a material, masked, laid over the frame
// by the compositing shape's coverage. The built-in materials (ui_builtin
// .wgsl) and a game's `type ui-material` (ui/mod.rs) follow this prelude
// with their `Params` and `fn material(m: MaterialInput, p: Params)`.

struct LayerParams {
  // The layer's rectangle: x, y, width, height in CSS px.
  rect: vec4<f32>,
  params: array<vec4<f32>, 4>,
  // x: the built-in material, y: mask (0 none, 1 gradient, 2 image), z:
  // the mask's gradient.
  info: vec4<f32>,
  // The mask image's uv rectangle.
  mask_uv: vec4<f32>,
};
@group(1) @binding(0) var layer_tex: texture_2d<f32>;
@group(1) @binding(1) var layer_sampler: sampler;
@group(1) @binding(2) var<uniform> layer_params: LayerParams;
@group(1) @binding(3) var mask_tex: texture_2d<f32>;

struct MaterialInput {
  // 0..1 over the layer's rectangle from its top left.
  uv: vec2<f32>,
  // CSS px from the rectangle's top left.
  pixel: vec2<f32>,
  // The rectangle's size, CSS px.
  size: vec2<f32>,
  // The UI clock, seconds.
  time: f32,
};

// The layer at `uv`, premultiplied (what was drawn into it).
fn layer_premultiplied(uv: vec2<f32>) -> vec4<f32> {
  return textureSampleLevel(layer_tex, layer_sampler, uv, 0.0);
}

// The layer at `uv`: linear colour, straight alpha.
fn layer(uv: vec2<f32>) -> vec4<f32> {
  let c = layer_premultiplied(uv);
  return vec4<f32>(c.rgb / max(c.a, 1e-5), c.a);
}

// The uv of one CSS px of the layer.
fn layer_texel() -> vec2<f32> {
  return 1.0 / max(layer_params.rect.zw, vec2<f32>(1.0));
}

fn hash12(p: vec2<f32>) -> f32 {
  var q = fract(p * vec2<f32>(123.34, 456.21));
  q += dot(q, q + 45.32);
  return fract(q.x * q.y);
}

// Smooth value noise, 0..1.
fn value_noise(p: vec2<f32>) -> f32 {
  let i = floor(p);
  let f = fract(p);
  let u = f * f * (3.0 - 2.0 * f);
  let a = hash12(i);
  let b = hash12(i + vec2<f32>(1.0, 0.0));
  let c = hash12(i + vec2<f32>(0.0, 1.0));
  let d = hash12(i + vec2<f32>(1.0, 1.0));
  return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

// A mask's fraction `m` restated for linear-light blending of the layer's
// colour `rgb`: the alpha drawing the luminance CSS's sRGB blend draws
// with `m` over the colour's opposite (sRGB 1 − its own: white under a
// dark scrim, black under light text), the ground a fade shows against.
// Dark content is close to exact over any ground; mid grey has no
// contrast to fade and keeps `m`. Applying `m` directly fades in linear
// light, where a dark scrim barely darkens until the mask is nearly
// whole and light text stays bright until its very edge: either fade
// reads as a line, not a fade.
fn mask_weight(m: f32, rgb: vec3<f32>) -> f32 {
  let lc = dot(rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
  let cs = encode(vec3<f32>(lc)).x;
  let gs = 1.0 - cs;
  let lg = decode(vec3<f32>(gs)).x;
  if (abs(lc - lg) < 1e-3) {
    return m;
  }
  let drawn = decode(vec3<f32>(mix(gs, cs, m))).x;
  return clamp((drawn - lg) / (lc - lg), 0.0, 1.0);
}

fn composite(in: UiOut, c: vec4<f32>) -> vec4<f32> {
  let s = instances[in.instance];
  let px = local_px(in.local);
  let cov = clamp(0.5 - shape_distance(s, in.local).x / px, 0.0, 1.0);
  let r = layer_params.rect;
  let css = css_of(in.position.xy);
  let uv = (css - r.xy) / max(r.zw, vec2<f32>(1e-4));
  var mask = 1.0;
  let mk = u32(layer_params.info.y);
  if (mk == 1u) {
    mask = gradient_color(u32(layer_params.info.z), uv, r.zw, in.position.xy).a;
  } else if (mk == 2u) {
    let m = mix(layer_params.mask_uv.xy, layer_params.mask_uv.zw, clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0)));
    mask = textureSampleLevel(mask_tex, layer_sampler, m, 0.0).a;
  }
  if (mk != 0u) {
    mask = mask_weight(mask, c.rgb);
  }
  let a = clamp(cov * c.a * mask * s.extra.x * instance_clip(s, in.position.xy), 0.0, 1.0);
  return finish_color(c.rgb, a);
}

fn material_input(in: UiOut) -> MaterialInput {
  let r = layer_params.rect;
  let css = css_of(in.position.xy);
  var m: MaterialInput;
  m.pixel = css - r.xy;
  m.size = r.zw;
  m.uv = m.pixel / max(r.zw, vec2<f32>(1e-4));
  m.time = frame.origin.z;
  return m;
}

@fragment
fn fs_layer(in: UiOut) -> @location(0) vec4<f32> {
  let c = material(material_input(in), material_params());
  return composite(in, c);
}
