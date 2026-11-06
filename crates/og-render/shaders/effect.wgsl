// Effect materials (effect.rs): one parameter set for particles, trails,
// ribbons, beams and effect meshes. `EffectGpu` in effect.rs is `Effect`;
// group 3 of the particle and strip draws holds it (a dynamic entry per
// material, flags 0 for a surface without one) and the material's
// textures. Prepended after frame.wgsl and before the draw's own shader,
// with the hook contract (hooks.rs `hooks_section_with`) after it.
//
// The vertex stage evaluates the driven values (`effect_driven`): they
// depend on the surface's life and random value only, so a particle's are
// constant over it. The fragment stage shades (`effect_shade`).

struct EffectDriver {
  head: vec4<f32>,               // x: 0 constant (y), 1 random (y to z), 2 curve (w keys)
  t: array<vec4<f32>, 2>,
  v: array<vec4<f32>, 2>,
};

struct Effect {
  flags: vec4<u32>,              // x: bits (effect.rs FLAG_*), y: main channel, z: mask channel, w: noise channel
  uv: array<vec4<f32>, 8>,       // main, mask, noise, flow: [tile.xy, offset.xy], [scroll.xy, rotation (rad/s), polar]
  erosion: vec4<f32>,            // x: softness, y: edge, z: flow strength, w: flow speed
  edge_color: vec4<f32>,
  fresnel: vec4<f32>,            // x: power, y: strength, z: 1 inverted (the soft depth is the surface's: effect.rs)
  color: vec4<f32>,
  counts: vec4<f32>,             // x: ramp keys, y: flow channel
  ramp_t: array<vec4<f32>, 2>,
  ramp_v: array<vec4<f32>, 8>,
  drivers: array<EffectDriver, 13>,  // offset u, v, erosion, intensity, alpha, custom 1 xyzw, custom 2 xyzw
  params: array<vec4<f32>, 8>,   // the hook's parameters
};

@group(3) @binding(0) var<uniform> effect: Effect;
@group(3) @binding(1) var effect_main: texture_2d<f32>;
@group(3) @binding(2) var effect_mask: texture_2d<f32>;
@group(3) @binding(3) var effect_noise: texture_2d<f32>;
@group(3) @binding(4) var effect_flow: texture_2d<f32>;
@group(3) @binding(5) var effect_sampler: sampler;

const FX_ON: u32 = 1u;
const FX_MAIN: u32 = 2u;
const FX_MASK: u32 = 4u;
const FX_EROSION: u32 = 8u;
const FX_NOISE: u32 = 16u;
const FX_FLOW: u32 = 32u;
const FX_FRESNEL: u32 = 64u;
const FX_RAMP: u32 = 128u;

fn effect_on() -> bool {
  return (effect.flags.x & FX_ON) != 0u;
}

// The values a surface's life and random value drive: a (offset u, offset
// v, erosion, intensity), b (alpha, life, random), the custom data.
struct EffectDriven {
  a: vec4<f32>,
  b: vec4<f32>,
  c1: vec4<f32>,
  c2: vec4<f32>,
};

fn fx_key(a: array<vec4<f32>, 2>, i: u32) -> f32 {
  let v = a[i / 4u];
  return v[i % 4u];
}

fn fx_hash(v: u32) -> u32 {
  let state = v * 747796405u + 2891336453u;
  let word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
  return (word >> 22u) ^ word;
}

fn fx_unit(v: u32) -> f32 {
  return f32(fx_hash(v) >> 8u) / 16777216.0;
}

// Driver `k` at life `t`; a random driver draws its own value from `seed`.
fn effect_drive(k: u32, t: f32, seed: u32) -> f32 {
  let d = effect.drivers[k];
  let kind = u32(d.head.x);
  if (kind == 0u) { return d.head.y; }
  if (kind == 1u) { return mix(d.head.y, d.head.z, fx_unit(seed ^ (k * 2654435769u))); }
  let n = u32(d.head.w);
  var v = fx_key(d.v, 0u);
  for (var i = 1u; i < n; i++) {
    let t0 = fx_key(d.t, i - 1u);
    let t1 = fx_key(d.t, i);
    if (t >= t1) {
      v = fx_key(d.v, i);
    } else if (t > t0) {
      v = mix(fx_key(d.v, i - 1u), fx_key(d.v, i), (t - t0) / max(t1 - t0, 1e-6));
      break;
    } else {
      break;
    }
  }
  return v;
}

// `life` 0 to 1; `seed` the surface's random bits (its random value is
// the unit float of them).
fn effect_driven(life: f32, seed: u32) -> EffectDriven {
  var o: EffectDriven;
  let t = clamp(life, 0.0, 1.0);
  o.a = vec4<f32>(effect_drive(0u, t, seed), effect_drive(1u, t, seed), effect_drive(2u, t, seed), effect_drive(3u, t, seed));
  o.b = vec4<f32>(effect_drive(4u, t, seed), t, fx_unit(seed), 0.0);
  o.c1 = vec4<f32>(effect_drive(5u, t, seed), effect_drive(6u, t, seed), effect_drive(7u, t, seed), effect_drive(8u, t, seed));
  o.c2 = vec4<f32>(effect_drive(9u, t, seed), effect_drive(10u, t, seed), effect_drive(11u, t, seed), effect_drive(12u, t, seed));
  return o;
}

// Texture `k`'s coordinates (0 main, 1 mask, 2 noise, 3 flow) from the
// surface's `uv`: turned, polar, then tiled, offset and scrolled.
fn effect_uv(k: u32, uv: vec2<f32>) -> vec2<f32> {
  let a = effect.uv[k * 2u];
  let b = effect.uv[k * 2u + 1u];
  let time = frame.time.x;
  var p = uv;
  if (b.z != 0.0) {
    let r = b.z * time;
    let c = cos(r);
    let s = sin(r);
    let d = p - 0.5;
    p = vec2<f32>(d.x * c - d.y * s, d.x * s + d.y * c) + 0.5;
  }
  if (b.w > 0.5) {
    let d = p - 0.5;
    p = vec2<f32>(fract(atan2(d.y, d.x) * 0.15915494 + 1.0), length(d) * 2.0);
  }
  return p * a.xy + a.zw + b.xy * time;
}

// Texture `k` sampled at `uv` (from `effect_uv`) with derivatives that
// skip a polar map's seam (where the angle wraps a whole tile).
fn effect_grads(k: u32, uv: vec2<f32>) -> mat2x2<f32> {
  var dx = dpdx(uv);
  var dy = dpdy(uv);
  if (effect.uv[k * 2u + 1u].w > 0.5) {
    let tile = max(abs(effect.uv[k * 2u].x), 1e-6);
    dx.x -= tile * round(dx.x / tile);
    dy.x -= tile * round(dy.x / tile);
  }
  return mat2x2<f32>(dx, dy);
}

// A texel through a channel: 0 rgba, 1 to 4 that channel as the alpha with white.
fn effect_channel(c: vec4<f32>, ch: u32) -> vec4<f32> {
  switch ch {
    case 1u: { return vec4<f32>(1.0, 1.0, 1.0, c.r); }
    case 2u: { return vec4<f32>(1.0, 1.0, 1.0, c.g); }
    case 3u: { return vec4<f32>(1.0, 1.0, 1.0, c.b); }
    case 4u: { return vec4<f32>(1.0, 1.0, 1.0, c.a); }
    default: { return c; }
  }
}

// A texel as one number: the channel, or luminance times alpha for rgba.
fn effect_scalar(c: vec4<f32>, ch: u32) -> f32 {
  if (ch == 0u) { return dot(c.rgb, vec3<f32>(0.2126, 0.7152, 0.0722)) * c.a; }
  return effect_channel(c, ch).a;
}

fn effect_ramp(x: f32) -> vec4<f32> {
  let n = u32(effect.counts.x);
  var v = effect.ramp_v[0];
  for (var i = 1u; i < n; i++) {
    let t0 = fx_key(effect.ramp_t, i - 1u);
    let t1 = fx_key(effect.ramp_t, i);
    if (x >= t1) {
      v = effect.ramp_v[i];
    } else if (x > t0) {
      v = mix(effect.ramp_v[i - 1u], effect.ramp_v[i], (x - t0) / max(t1 - t0, 1e-6));
      break;
    } else {
      break;
    }
  }
  return v;
}

fn fx_lattice(c: vec2<i32>) -> f32 {
  return fx_unit((bitcast<u32>(c.x) * 73856093u) ^ (bitcast<u32>(c.y) * 19349663u));
}

// Two octaves of 2D value noise in [0, 1): the erosion's noise without a
// texture.
fn effect_value_noise(p: vec2<f32>) -> f32 {
  var sum = 0.0;
  var q = p;
  var w = 0.65;
  for (var o = 0u; o < 2u; o++) {
    let i = vec2<i32>(floor(q));
    let f = fract(q);
    let u = f * f * (3.0 - 2.0 * f);
    let a = mix(fx_lattice(i), fx_lattice(i + vec2<i32>(1, 0)), u.x);
    let b = mix(fx_lattice(i + vec2<i32>(0, 1)), fx_lattice(i + vec2<i32>(1, 1)), u.x);
    sum += w * mix(a, b, u.y);
    q = q * 2.03 + vec2<f32>(17.0, 31.0);
    w = 0.35;
  }
  return sum;
}

// What a surface hands the effect at a fragment.
struct EffectFragment {
  // The main texture's base coordinates (a flipbook frame's on a sheet)
  // and the surface's own 0 to 1 coordinates (mask, noise, flow).
  uv: vec2<f32>,
  shape_uv: vec2<f32>,
  // The surface's colour (vertex colour, curves, tint).
  color: vec4<f32>,
  // The surface's fixed shape when it has no texture (a soft dot, a soft
  // edge; 1 otherwise), applied when the material has no main texture.
  shape: f32,
  // The surface's own cross-section, applied unless the material has a
  // mask (which is then the cross-section): a strip's soft edge across its
  // width; 1 for particles and meshes.
  profile: f32,
  // 1 when the surface has a texture of its own (group 1).
  textured: f32,
  world: vec3<f32>,
  normal: vec3<f32>,
  driven: EffectDriven,
};

// The effect's colour at a fragment. `surface` is the surface's own
// texture (group 1, white without one), sampled with the effect's
// repeating sampler. Every texture is sampled in uniform control flow;
// only the flags (uniform) choose what is used.
fn effect_shade(f: EffectFragment, surface: texture_2d<f32>) -> vec4<f32> {
  let bits = effect.flags.x;
  let d = f.driven;
  let time = frame.time.x;
  // Flow: the map's red and green push the main UV in two crossfaded phases.
  let uv_flow = effect_uv(3u, f.shape_uv);
  let gf = effect_grads(3u, uv_flow);
  let flow_tex = textureSampleGrad(effect_flow, effect_sampler, uv_flow, gf[0], gf[1]);
  var push = vec2<f32>(0.0);
  var phase = 0.0;
  if ((bits & FX_FLOW) != 0u) {
    push = (flow_tex.rg * 2.0 - 1.0) * effect.erosion.z;
    phase = fract(time * effect.erosion.w);
  }
  let uv_main = effect_uv(0u, f.uv) + d.a.xy;
  let gm = effect_grads(0u, uv_main);
  let uv_a = uv_main + push * phase;
  let own_a = textureSampleGrad(effect_main, effect_sampler, uv_a, gm[0], gm[1]);
  let surf_a = textureSampleGrad(surface, effect_sampler, uv_a, gm[0], gm[1]);
  var tex = select(surf_a, own_a, (bits & FX_MAIN) != 0u);
  if ((bits & FX_FLOW) != 0u) {
    let uv_b = uv_main + push * fract(phase + 0.5);
    let own_b = textureSampleGrad(effect_main, effect_sampler, uv_b, gm[0], gm[1]);
    let surf_b = textureSampleGrad(surface, effect_sampler, uv_b, gm[0], gm[1]);
    let w = 1.0 - abs(phase * 2.0 - 1.0);
    tex = mix(select(surf_b, own_b, (bits & FX_MAIN) != 0u), tex, w);
  }
  let has_texture = (bits & FX_MAIN) != 0u || f.textured > 0.5;
  let ch = effect.flags.y;
  var main = effect_channel(tex, ch);
  if ((bits & FX_RAMP) != 0u) {
    let x = select(effect_scalar(tex, 0u) / max(tex.a, 1e-4), main.a, ch != 0u);
    let r = effect_ramp(clamp(x, 0.0, 1.0));
    main = vec4<f32>(r.rgb, r.a * select(tex.a, x, ch != 0u));
  }
  if (!has_texture) {
    main = vec4<f32>(main.rgb, f.shape);
  }
  var c = f.color * main * effect.color;
  c = vec4<f32>(c.rgb * d.a.w, c.a * d.b.x);
  // The mask.
  let uv_mask = effect_uv(1u, f.shape_uv);
  let gk = effect_grads(1u, uv_mask);
  let mask_tex = textureSampleGrad(effect_mask, effect_sampler, uv_mask, gk[0], gk[1]);
  if ((bits & FX_MASK) != 0u) {
    c.a *= effect_scalar(mask_tex, effect.flags.z);
  } else {
    c.a *= f.profile;
  }
  // Erosion: cut under the amount, a soft border, a coloured edge.
  let uv_noise = effect_uv(2u, f.shape_uv);
  let gn = effect_grads(2u, uv_noise);
  let noise_tex = textureSampleGrad(effect_noise, effect_sampler, uv_noise, gn[0], gn[1]);
  if ((bits & FX_EROSION) != 0u) {
    var n = effect_value_noise(uv_noise * 4.0);
    if ((bits & FX_NOISE) != 0u) { n = effect_scalar(noise_tex, effect.flags.w); }
    let soft = max(effect.erosion.x, 1e-4);
    let edge = effect.erosion.y;
    let cut = d.a.z * (1.0 + soft + edge) - soft - edge;
    c.a *= clamp((n - cut) / soft, 0.0, 1.0);
    if (edge > 0.0) {
      let e = 1.0 - clamp((n - cut - soft) / edge, 0.0, 1.0);
      c = vec4<f32>(mix(c.rgb, effect.edge_color.rgb, e * effect.edge_color.a), c.a);
    }
  }
  // Fresnel: by the angle to the eye.
  if ((bits & FX_FRESNEL) != 0u) {
    let v = select(normalize(frame.view_pos.xyz - f.world), -frame.view_forward.xyz, frame.view_pos.w > 0.5);
    let rim = pow(1.0 - clamp(abs(dot(normalize(f.normal), v)), 0.0, 1.0), max(effect.fresnel.x, 1e-3));
    let k = select(rim, 1.0 - rim, effect.fresnel.z > 0.5);
    c.a *= mix(1.0, k, clamp(effect.fresnel.y, 0.0, 1.0));
  }
  // The hook (hooks.rs: surface_color, then surface_emissive added).
  hook_effect = d.b.yz;
  hook_custom1 = d.c1;
  hook_custom2 = d.c2;
  var h: Hook;
  h.world = f.world;
  h.local = vec3<f32>(f.shape_uv, 0.0);
  h.normal = normalize(f.normal);
  h.uv = uv_main;
  h.color = c;
  h.time = time;
  h.eye = frame.view_pos.xyz;
  h.front = 1.0;
  h.origin = frame.origin.xyz;
  let p = shader_params();
  c = surface_color(h, p, c);
  c = vec4<f32>(c.rgb + surface_emissive(h, p, vec3<f32>(0.0)), c.a);
  return c;
}
