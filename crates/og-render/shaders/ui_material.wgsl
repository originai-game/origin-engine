// A layer's composite (src/ui/pass.rs), after ui_common.wgsl and
// ui_blend.wgsl: the layer's
// offscreen target read through a material, masked, laid over the frame
// by the compositing shape's coverage. The built-in materials (ui_builtin
// .wgsl) and a game's `type ui-material` (ui/mod.rs) follow this prelude
// with their `Params` and `fn material(m: MaterialInput, p: Params)`.

struct LayerParams {
  // The layer's rectangle: x, y, width, height in CSS px.
  rect: vec4<f32>,
  params: array<vec4<f32>, 4>,
  // x: the built-in material, y: mask (0 none, 1 gradient, 2 image), z:
  // the mask's gradient, w: 1 for a layer laid onto a quad (`quad`), 2
  // for one laid on its plane only (a CSS 3D transform: no scene read).
  info: vec4<f32>,
  // The mask image's uv rectangle.
  mask_uv: vec4<f32>,
  // A projected layer's drape (UiDrape): rows a (xyz) with the scale's x,
  // y and the reach in w, then rows b. A screen point (x, y) at view
  // depth d lies at a·(x, y, 1) + d·b·(x, y, 1): the rect's CSS px, and
  // world units off its plane.
  drape: array<vec4<f32>, 6>,
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

// A lattice point's value in [0, 1), from integer operations only: PCG
// nested per axis, `pcg(x + pcg(y))`. The float hash this replaces
// (`fract(p * vec2(123.34, 456.21))`, a dot and `fract(q.x * q.y)`) kept the
// last bits of products that each GPU's compiler fuses and rounds its own
// way, so a dissolve's cells differed between DX12 and Metal (ui-layers,
// studio-win 2026-11-05). Mixing the axes with one XOR before a single PCG
// left the dissolve with thin vertical slivers; nesting keeps the blobs.
// Every caller passes whole numbers (the value noise's lattice, the
// glitch's slice and tick), so `p` is read as its integer cell.
fn hash12(p: vec2<f32>) -> f32 {
  let c = bitcast<vec2<u32>>(vec2<i32>(floor(p)));
  return f32(ui_pcg(c.x + ui_pcg(c.y)) >> 8u) / 16777216.0;
}

fn ui_pcg(v: u32) -> u32 {
  let state = v * 747796405u + 2891336453u;
  let word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
  return (word >> 22u) ^ word;
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

// The depth prepass's value where it has none (post.rs FAR_DEPTH).
const FAR_DEPTH: f32 = 1.0e9;

// The scene's view depth under the screen point `css` (the overlay's CSS
// px): the prepass's, or FAR_DEPTH where there is none.
fn scene_depth_at(css: vec2<f32>) -> f32 {
  let dims = vec2<f32>(textureDimensions(scene_depth));
  let size = frame.output.xy / max(frame.output.z, 1e-6);
  let q = clamp(css * dims / max(size, vec2<f32>(1.0)), vec2<f32>(0.0), dims - vec2<f32>(1.0));
  return textureLoad(scene_depth, vec2<i32>(q), 0).r;
}

struct Draped {
  // Over the layer's rectangle.
  uv: vec2<f32>,
  // The plane's own uv there, smooth across the scene's edges.
  plane: vec2<f32>,
  // How much of the layer the point takes.
  weight: f32,
};

// A projected layer at the screen point `css`, laid as a decal: on the
// scene's surface under it while that is within reach of the plane
// (faded as the surface turns away from the plane's normal and as it
// nears the reach on the camera's side), on the plane where the surface
// is farther behind it or there is none, and nowhere where something
// nearer hides it.
fn drape(css: vec2<f32>) -> Draped {
  let g = layer_params.drape;
  let h = vec3<f32>(css, 1.0);
  let a = vec3<f32>(dot(g[0].xyz, h), dot(g[1].xyz, h), dot(g[2].xyz, h));
  let b = vec3<f32>(dot(g[3].xyz, h), dot(g[4].xyz, h), dot(g[5].xyz, h));
  let scale = vec2<f32>(g[0].w, g[1].w);
  let reach = g[2].w;
  let r = layer_params.rect;
  // Where the ray meets the plane (none behind the eye).
  let dp = select(-1.0, -a.z / b.z, abs(b.z) > 1e-12);
  let p = a + max(dp, 0.0) * b;
  var out: Draped;
  out.plane = (p.xy - r.xy) / max(r.zw, vec2<f32>(1e-4));
  out.uv = out.plane;
  out.weight = select(0.0, 1.0, dp > 0.0);
  if (layer_params.info.w > 1.5) {
    // On its plane only: a CSS 3D transform's layer.
    return out;
  }
  let d = scene_depth_at(css);
  let s = a + d * b;
  // The surface's facing, from its place's change across the pixel in
  // world units: 1 lying along the plane, 0 standing across it.
  let m = vec3<f32>(s.xy * scale, s.z);
  let nrm = cross(dpdx(m), dpdy(m));
  let facing = abs(nrm.z) / max(length(nrm), 1e-12);
  out.weight = 0.0;
  let surface = d < 0.5 * FAR_DEPTH;
  if (surface && abs(s.z) <= reach) {
    out.uv = (s.xy - r.xy) / max(r.zw, vec2<f32>(1e-4));
    // Toward the camera it fades out over the reach's last quarter; on
    // the far side it meets the plane's fallback.
    let near_side = dp <= 0.0 || d < dp;
    let edge = select(1.0, 1.0 - smoothstep(0.75 * reach, reach, abs(s.z)), near_side);
    out.weight = smoothstep(0.2, 0.45, facing) * edge;
  } else if (dp > 0.0 && (!surface || d > dp)) {
    out.weight = 1.0;
  }
  return out;
}

// The uv over the layer's rectangle of the screen point `css`: straight,
// or draped for a projected layer.
fn layer_uv(css: vec2<f32>) -> vec2<f32> {
  if (layer_params.info.w > 0.5) {
    return drape(css).uv;
  }
  let r = layer_params.rect;
  return (css - r.xy) / max(r.zw, vec2<f32>(1e-4));
}

fn composite(in: UiOut, c: vec4<f32>) -> vec4<f32> {
  let s = instances[in.instance];
  let px = local_px(in.local);
  var cov = clamp(0.5 - shape_distance(s, in.local).x / px, 0.0, 1.0);
  let r = layer_params.rect;
  let css = css_of(in.position.xy);
  var uv = (css - r.xy) / max(r.zw, vec2<f32>(1e-4));
  if (layer_params.info.w > 0.5) {
    // Inside the rect, its edges smoothed over a device pixel: the uv's
    // distance to the unit square's border over the plane's change per
    // pixel (the surface's jumps at the scene's edges would blur them).
    let dr = drape(css);
    uv = dr.uv;
    let edge = min(uv, vec2<f32>(1.0) - uv) / max(fwidth(dr.plane), vec2<f32>(1e-6));
    cov = cov * clamp(0.5 + min(edge.x, edge.y), 0.0, 1.0) * dr.weight;
  }
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
  let op = (s.kinds.z >> 4u) & 31u;
  // A Porter-Duff composite (src/ui/build.rs codes 17 to 20) blends premultiplied.
  if (op >= 17u && op <= 20u) {
    let out = finish_color(c.rgb, a);
    return vec4<f32>(out.rgb * a, a);
  }
  return blend_over(c.rgb, a, op, in.position.xy);
}

fn material_input(in: UiOut) -> MaterialInput {
  let r = layer_params.rect;
  let css = css_of(in.position.xy);
  var m: MaterialInput;
  m.pixel = css - r.xy;
  m.size = r.zw;
  m.uv = m.pixel / max(r.zw, vec2<f32>(1e-4));
  if (layer_params.info.w > 0.5) {
    m.uv = layer_uv(css);
    m.pixel = m.uv * r.zw;
  }
  m.time = frame.origin.z;
  return m;
}

@fragment
fn fs_layer(in: UiOut) -> @location(0) vec4<f32> {
  let c = material(material_input(in), material_params());
  return composite(in, c);
}
