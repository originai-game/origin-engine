// Mass elements (src/ui/mass.rs, docs/ui.md §1): one instance per
// element, `quads` quads each. The vertex stage projects the element's
// point, evaluates its template's timeline at its age, formats its number
// or picks its string, and places the quad; the fragment stage draws a
// glyph from the font's distance field (with an outline), an icon or a bar.
// After ui_common.wgsl (the frame uniform, clips, occluders, scene depth).

struct SpriteParams {
  msdf: vec4<f32>,
};
@group(1) @binding(0) var sprite_tex: texture_2d<f32>;
@group(1) @binding(1) var sprite_sampler: sampler;
@group(1) @binding(2) var<uniform> sprite_params: SpriteParams;

struct MassCamera {
  view_proj: mat4x4<f32>,
  // xyz: the eye; w: 1 orthographic.
  eye: vec4<f32>,
  forward: vec4<f32>,
  // xy: the overlay's CSS size; z: the tick's fraction; w: unused.
  css: vec4<f32>,
  // x: the tick; y: the table's first glyph entry (past the templates).
  tick: vec4<u32>,
};
@group(2) @binding(0) var<uniform> camera: MassCamera;

// One batch's template (src/ui/mass.rs `TemplateGpu`).
struct MassTemplate {
  // kind (0 number, 1 text, 2 bar, 3 icon), quads, table base, flags.
  head: vec4<u32>,
  // lifetime (ticks), keys, decimals, entries (strings or frames).
  head2: vec4<u32>,
  // size xy (CSS px), pivot xy.
  size: vec4<f32>,
  // reference distance, min scale, max scale, drift.
  dist: vec4<f32>,
  // alpha behind the scene, bias, softness, gravity.
  occl: vec4<f32>,
  outline: vec4<f32>,
  // x: outline width (CSS px), y: bar radius, z: particle shape.
  misc: vec4<f32>,
  back: vec4<f32>,
  ghost: vec4<f32>,
  // The batch's clip rectangle x0 y0 x1 y1, and its clip head in w of
  // `clip_head`.
  clip: vec4<f32>,
  clip_head: vec4<u32>,
  // t, scale, alpha, ease.
  keys: array<vec4<f32>, 8>,
  // offset x, y.
  offsets: array<vec4<f32>, 8>,
};
// The table texture: each batch's template (32 texels, integers as exact
// floats), then the glyphs (uv, plane, advance), strings' runs (start,
// count, width) and icon frames (uv) at each template's base past them.
// Texel i is at (i % 256, i / 256).
@group(2) @binding(1) var table_tex: texture_2d<f32>;

fn texel(i: u32) -> vec4<f32> {
  return textureLoad(table_tex, vec2<i32>(i32(i % 256u), i32(i / 256u)), 0);
}

fn table(i: u32) -> vec4<f32> {
  return texel(camera.tick.y + i);
}

fn mass_template(b: u32) -> MassTemplate {
  let at = b * 32u;
  var t: MassTemplate;
  t.head = vec4<u32>(texel(at));
  t.head2 = vec4<u32>(texel(at + 1u));
  t.size = texel(at + 2u);
  t.dist = texel(at + 3u);
  t.occl = texel(at + 4u);
  t.outline = texel(at + 5u);
  t.misc = texel(at + 6u);
  t.back = texel(at + 7u);
  t.ghost = texel(at + 8u);
  t.clip = texel(at + 9u);
  t.clip_head = vec4<u32>(texel(at + 10u));
  for (var i = 0u; i < 8u; i++) {
    t.keys[i] = texel(at + 11u + i);
    t.offsets[i] = texel(at + 19u + i);
  }
  return t;
}

// One element: an instance of the vertex buffer (mass.rs `UiMassInstance`).
struct MassInstance {
  @location(0) position_value: vec4<f32>,
  @location(1) color: vec4<f32>,
  @location(2) start_seed: vec2<u32>,
  @location(3) scale: f32,
  @location(4) batch: u32,
  @location(5) aux: vec4<f32>,
  @location(6) velocity: vec4<f32>,
};

const KIND_NUMBER: u32 = 0u;
const KIND_TEXT: u32 = 1u;
const KIND_BAR: u32 = 2u;
const KIND_ICON: u32 = 3u;
const KIND_PARTICLE: u32 = 4u;
const FLAG_WORLD: u32 = 1u;
const FLAG_OCCLUDE: u32 = 2u;
const FLAG_HIDE: u32 = 4u;
const FLAG_OUTLINE: u32 = 8u;
const FLAG_PLUS: u32 = 16u;
const FLAG_COMPACT: u32 = 32u;
const FLAG_DISTANCE: u32 = 64u;
const FLAG_DRAG: u32 = 128u;

// Number symbols: 0-9, then these.
const SYM_PLUS: u32 = 10u;
const SYM_MINUS: u32 = 11u;
const SYM_POINT: u32 = 12u;
const SYM_K: u32 = 13u;
const SYM_M: u32 = 14u;

struct MassOut {
  @builtin(position) position: vec4<f32>,
  @location(0) uv: vec2<f32>,
  @location(1) color: vec4<f32>,
  // A bar's position from its centre and its half size (CSS px).
  @location(2) local: vec2<f32>,
  @location(3) half: vec2<f32>,
  @location(4) @interpolate(flat) batch: u32,
};

fn ease_lut(base: u32, k: f32) -> f32 {
  let info = table(base);
  if (k <= 0.0) { return info.y; }
  if (k >= 1.0) { return info.z; }
  if (info.x == 0.0) {
    let x = k * 4096.0;
    let i = min(u32(x), 4095u);
    return mix(table(base + 1u + i).y, table(base + 2u + i).y, x - f32(i));
  }
  // Quantize time first: interpolation can only reconstruct discrete
  // step ordinates, never ramp between the times on either side of a jump.
  let product = k * info.x;
  var ordinal = floor(product);
  if (product == ordinal && fma(k, info.x, -product) < 0.0) { ordinal -= 1.0; }
  var lo = 0u;
  var hi = 4096u;
  for (var j = 0u; j < 13u; j++) {
    let mid = (lo + hi) / 2u;
    if (table(base + 1u + mid).x <= ordinal) { lo = mid; } else { hi = mid; }
  }
  let a = table(base + 1u + lo);
  let b = table(base + 1u + hi);
  return mix(a.y, b.y, clamp((ordinal - a.x) / max(b.x - a.x, 1.0), 0.0, 1.0));
}

fn ease(code: f32, k0: f32, lut: f32) -> f32 {
  let k = clamp(k0, 0.0, 1.0);
  let c = u32(code + 0.5);
  if (c == 1u) { return 1.0 - (1.0 - k) * (1.0 - k); }
  if (c == 2u) { return k * k; }
  if (c == 3u) {
    let u = k - 1.0;
    return 1.0 + 2.70158 * u * u * u + 1.70158 * u * u;
  }
  if (c == 4u) { return select(0.0, 1.0, k >= 1.0); }
  if (c == 5u) { return ease_lut(u32(lut), k); }
  return k;
}

// The timeline at life fraction `t`: offset xy, scale, alpha (mass.rs's
// `sample_keys`).
fn sample_keys(tp: MassTemplate, t: f32) -> vec4<f32> {
  let n = max(tp.head2.y, 1u);
  let first = tp.keys[0];
  if (t <= first.x) {
    return vec4<f32>(tp.offsets[0].xy, first.y, first.z);
  }
  for (var i = 1u; i < n; i++) {
    let a = tp.keys[i - 1u];
    let k = tp.keys[i];
    if (t <= k.x) {
      let e = ease(k.w, (t - a.x) / max(k.x - a.x, 1e-6), tp.offsets[i].z);
      let oa = tp.offsets[i - 1u].xy;
      let ob = tp.offsets[i].xy;
      return vec4<f32>(mix(oa, ob, e), mix(a.y, k.y, e), mix(a.z, k.z, e));
    }
  }
  let last = tp.keys[n - 1u];
  return vec4<f32>(tp.offsets[n - 1u].xy, last.y, last.z);
}

fn hash01(x: u32) -> f32 {
  return f32(pcg(x) & 0xffffu) / 65535.0;
}

// The symbols of a number (mass.rs's `format_number`), at most 16.
struct Number {
  syms: array<u32, 16>,
  count: u32,
};

fn format_number(value: f32, decimals: u32, plus: bool, compact: bool) -> Number {
  var out: Number;
  out.count = 0u;
  var a = abs(value);
  var dec = min(decimals, 3u);
  var suffix = 99u;
  if (compact && a >= 999950.0) {
    a = a / 1000000.0;
    suffix = SYM_M;
    dec = 1u;
  } else if (compact && a >= 10000.0) {
    a = a / 1000.0;
    suffix = SYM_K;
    dec = 1u;
  }
  var scales = array<f32, 4>(1.0, 10.0, 100.0, 1000.0);
  let scale = scales[dec];
  let fixed = u32(min(floor(a * scale + 0.5), 4.0e9));
  let su = u32(scale);
  var int_part = fixed / su;
  let frac = fixed % su;
  if (value < 0.0 && fixed > 0u) {
    out.syms[out.count] = SYM_MINUS;
    out.count += 1u;
  } else if (plus && fixed > 0u) {
    out.syms[out.count] = SYM_PLUS;
    out.count += 1u;
  }
  // The integer's digits, most significant first.
  var digits = array<u32, 10>(0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u);
  var nd = 0u;
  loop {
    digits[nd] = int_part % 10u;
    nd += 1u;
    int_part = int_part / 10u;
    if (int_part == 0u || nd >= 10u) { break; }
  }
  for (var i = 0u; i < nd; i++) {
    out.syms[out.count] = digits[nd - 1u - i];
    out.count += 1u;
  }
  if (dec > 0u) {
    out.syms[out.count] = SYM_POINT;
    out.count += 1u;
    var p = su / 10u;
    var f = frac;
    for (var i = 0u; i < dec; i++) {
      out.syms[out.count] = f / p;
      out.count += 1u;
      f = f % p;
      p = max(p / 10u, 1u);
    }
  }
  if (suffix != 99u) {
    out.syms[out.count] = suffix;
    out.count += 1u;
  }
  return out;
}

fn hidden() -> MassOut {
  var o: MassOut;
  o.position = vec4<f32>(-2.0, -2.0, 0.0, 1.0);
  o.uv = vec2<f32>(0.0);
  o.color = vec4<f32>(0.0);
  o.local = vec2<f32>(0.0);
  o.half = vec2<f32>(1.0);
  o.batch = 0u;
  return o;
}

@vertex
fn vs_mass(@builtin(vertex_index) vi: u32, m: MassInstance) -> MassOut {
  var corners = array<vec2<f32>, 6>(
    vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0),
    vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0), vec2<f32>(0.0, 1.0));
  let c = corners[vi % 6u];
  let q = vi / 6u;
  let tp = mass_template(m.batch);
  let kind = tp.head.x;
  let flags = tp.head.w;
  // Its age and life.
  if (camera.tick.x < m.start_seed.x) {
    return hidden();
  }
  let age = f32(camera.tick.x - m.start_seed.x) + camera.css.z;
  var t = 0.0;
  if (tp.head2.x > 0u) {
    t = age / f32(tp.head2.x);
    if (t > 1.0) {
      return hidden();
    }
  }
  let key = sample_keys(tp, t);
  var travel = t;
  if ((flags & FLAG_DRAG) != 0u) {
    travel = 1.0 - (1.0 - t) * (1.0 - t);
  }
  var offset = key.xy + m.aux.xy + m.velocity.xy * travel;
  offset.x += ((hash01(m.start_seed.y) * 2.0 - 1.0) * tp.dist.w + m.aux.w) * t;
  offset.y += tp.occl.w * t * t;
  var scale = key.z * m.scale;
  var alpha = key.w * m.color.a;
  // A world element's offsets and size follow its distance scale.
  var dscale = 1.0;
  // Its point on the overlay.
  var at = m.position_value.xy;
  if ((flags & FLAG_WORLD) != 0u) {
    let clip = camera.view_proj * vec4<f32>(m.position_value.xyz, 1.0);
    if (clip.w <= 1e-5) {
      return hidden();
    }
    let ndc = clip.xyz / clip.w;
    if (ndc.z < 0.0 || ndc.z > 1.0) {
      return hidden();
    }
    at = vec2<f32>((ndc.x + 1.0) * 0.5 * camera.css.x, (1.0 - ndc.y) * 0.5 * camera.css.y);
    let depth = dot(m.position_value.xyz - camera.eye.xyz, camera.forward.xyz);
    if ((flags & FLAG_DISTANCE) != 0u && tp.dist.x > 0.0) {
      dscale = clamp(tp.dist.x / max(depth, 1e-4), tp.dist.y, tp.dist.z);
      scale *= dscale;
    }
    if ((flags & (FLAG_OCCLUDE | FLAG_HIDE)) != 0u) {
      let v = scene_visibility(at, camera.css.xy, depth, tp.occl.y, tp.occl.z);
      if ((flags & FLAG_HIDE) != 0u && v < 0.5) {
        return hidden();
      }
      alpha *= mix(tp.occl.x, 1.0, v);
    }
  }
  if (alpha <= 0.002 || scale <= 0.0) {
    return hidden();
  }
  at += offset * dscale;
  let em = tp.size.x * scale;
  var o: MassOut;
  o.batch = m.batch;
  o.color = vec4<f32>(m.color.rgb, alpha);
  o.local = vec2<f32>(0.0);
  o.half = vec2<f32>(1.0);
  var rect = vec4<f32>(0.0); // x0 y0 x1 y1, CSS px
  var uv = vec4<f32>(0.0, 0.0, 1.0, 1.0);
  let base = tp.head.z;
  if (kind == KIND_NUMBER || kind == KIND_TEXT) {
    // The glyph entry of quad q, the pen before it and the line's width (em).
    var entry = 0u;
    var pen = 0.0;
    var width = 0.0;
    var count = 0u;
    if (kind == KIND_NUMBER) {
      let n = format_number(m.position_value.w, tp.head2.z, (flags & FLAG_PLUS) != 0u, (flags & FLAG_COMPACT) != 0u);
      count = n.count;
      for (var i = 0u; i < n.count; i++) {
        let adv = table(base + n.syms[i] * 3u + 2u).x;
        if (i == q) {
          entry = base + n.syms[i] * 3u;
          pen = width;
        }
        width += adv;
      }
    } else {
      let s = u32(max(m.position_value.w, 0.0));
      if (s >= tp.head2.w) {
        return hidden();
      }
      let run = table(base + s);
      count = u32(run.y);
      width = run.z;
      entry = base + tp.head2.w + (u32(run.x) + q) * 3u;
    }
    if (q >= count) {
      return hidden();
    }
    let plane = table(entry + 1u);
    if (plane.z <= plane.x) {
      return hidden();
    }
    uv = table(entry);
    let left = at.x - tp.size.z * width * em;
    let top = at.y - tp.size.w * em;
    let baseline = top + 0.8 * em;
    rect = vec4<f32>(left + (pen + plane.x) * em, baseline - plane.w * em,
                     left + (pen + plane.z) * em, baseline - plane.y * em);
  } else if (kind == KIND_BAR) {
    if (q >= 3u) {
      return hidden();
    }
    let size = tp.size.xy * scale;
    let x0 = at.x - tp.size.z * size.x;
    let y0 = at.y - tp.size.w * size.y;
    var frac = 1.0;
    if (q == 1u) {
      frac = clamp(max(m.aux.z, m.position_value.w), 0.0, 1.0);
      o.color = vec4<f32>(tp.ghost.rgb, tp.ghost.a * alpha);
    } else if (q == 2u) {
      frac = clamp(m.position_value.w, 0.0, 1.0);
    } else {
      o.color = vec4<f32>(tp.back.rgb, tp.back.a * alpha);
    }
    if (frac <= 0.0) {
      return hidden();
    }
    rect = vec4<f32>(x0, y0, x0 + size.x * frac, y0 + size.y);
    o.half = 0.5 * size;
    // The position from the whole bar's centre (every quad shares its ends).
    o.local = (vec2<f32>(mix(rect.x, rect.z, c.x), mix(rect.y, rect.w, c.y)) - vec2<f32>(x0, y0)) - o.half;
  } else {
    // An icon or a particle: one quad about its pivot, turned.
    let i = u32(max(m.position_value.w, 0.0));
    if (q >= 1u || (kind == KIND_ICON && i >= tp.head2.w)) {
      return hidden();
    }
    if (kind == KIND_ICON) {
      uv = table(base + i);
    }
    let size = tp.size.xy * scale;
    let pivot = tp.size.zw;
    let local = (c - pivot) * size;
    let turn = m.velocity.w + m.velocity.z * TAU * t;
    let cs = cos(turn);
    let sn = sin(turn);
    let css = at + vec2<f32>(local.x * cs - local.y * sn, local.x * sn + local.y * cs);
    o.half = 0.5 * size;
    o.local = (c - vec2<f32>(0.5)) * size;
    o.position = device_position(css);
    o.uv = vec2<f32>(mix(uv.x, uv.z, c.x), mix(uv.y, uv.w, c.y));
    return o;
  }
  let css = vec2<f32>(mix(rect.x, rect.z, c.x), mix(rect.y, rect.w, c.y));
  o.position = device_position(css);
  o.uv = vec2<f32>(mix(uv.x, uv.z, c.x), mix(uv.y, uv.w, c.y));
  return o;
}

fn median3(r: f32, g: f32, b: f32) -> f32 {
  return max(min(r, g), min(max(r, g), b));
}

@fragment
fn fs_mass(in: MassOut) -> @location(0) vec4<f32> {
  let tp = mass_template(in.batch);
  let kind = tp.head.x;
  let css = css_of(in.position.xy);
  var cover = 1.0;
  if (css.x < tp.clip.x || css.y < tp.clip.y || css.x >= tp.clip.z || css.y >= tp.clip.w) {
    discard;
  }
  if (tp.clip_head.w != 0u) {
    cover = clip_coverage(tp.clip_head.w, css);
  }
  var rgb = in.color.rgb;
  var a = in.color.a;
  let sample = textureSample(sprite_tex, sprite_sampler, in.uv);
  let unit_range = vec2<f32>(sprite_params.msdf.x) / max(sprite_params.msdf.yz, vec2<f32>(1.0));
  let screen_tex_size = vec2<f32>(1.0) / max(fwidthFine(in.uv), vec2<f32>(1e-6));
  let px_range = max(0.5 * dot(unit_range, screen_tex_size), 1.0);
  if (kind == KIND_NUMBER || kind == KIND_TEXT) {
    let reach = 0.5 * px_range - 0.5;
    let d = px_range * (median3(sample.r, sample.g, sample.b) - 0.5);
    let true_d = px_range * (min(median3(sample.r, sample.g, sample.b), sample.a) - 0.5);
    let fill = ink_cover(clamp(d + 0.5, 0.0, 1.0), rgb);
    if ((tp.head.w & FLAG_OUTLINE) != 0u) {
      let w = min(tp.misc.x * frame.output.z, reach);
      let od = select(d, true_d, d < 0.0);
      let band = clamp(od + w + 0.5, 0.0, 1.0);
      let oa = band * tp.outline.a;
      let fa = fill * a;
      let out_a = fa + oa * a * (1.0 - fill);
      rgb = mix(tp.outline.rgb, rgb, select(fill, fa / max(out_a, 1e-5), out_a > 0.0));
      a = out_a;
    } else {
      a *= fill;
    }
  } else if (kind == KIND_BAR) {
    // The whole bar's rounded rectangle.
    let r = min(tp.misc.y, min(in.half.x, in.half.y));
    let qd = abs(in.local) - (in.half - vec2<f32>(r));
    let sd = length(max(qd, vec2<f32>(0.0))) + min(max(qd.x, qd.y), 0.0) - r;
    a *= clamp(0.5 - sd * frame.output.z, 0.0, 1.0);
  } else if (kind == KIND_PARTICLE) {
    let shape = u32(tp.misc.z + 0.5);
    var sd = 0.0;
    if (shape == 1u) {
      sd = length(in.local / max(in.half, vec2<f32>(1e-3))) - 1.0;
      sd *= min(in.half.x, in.half.y);
    } else if (shape == 2u) {
      let p = abs(in.local) / max(in.half, vec2<f32>(1e-3));
      sd = (p.x + p.y - 1.0) * min(in.half.x, in.half.y) * 0.7071;
    } else {
      let qd = abs(in.local) - in.half;
      sd = length(max(qd, vec2<f32>(0.0))) + min(max(qd.x, qd.y), 0.0);
    }
    a *= clamp(0.5 - sd * frame.output.z, 0.0, 1.0);
  } else {
    rgb *= sample.rgb;
    a *= sample.a;
  }
  a *= cover;
  if (a <= 0.0) {
    discard;
  }
  return finish_color(rgb, a);
}
