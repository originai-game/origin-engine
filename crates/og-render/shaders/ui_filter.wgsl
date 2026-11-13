// One primitive of an SVG filter graph over a UI layer (src/ui/filter.rs),
// after ui_blend_modes.wgsl: a full-target pass that writes the
// primitive's result (premultiplied, linear unless the primitive asks for
// sRGB) from its inputs, which share the layer's target size. Pixels
// outside the primitive's subregion are transparent, and an input read
// outside its texture is transparent (Filter Effects 1 §9), or takes its
// edge for a backdrop (FLAG_CLAMP). Subregions, lengths and lights are in
// the filter's space (`space`, its map to CSS px): every primitive runs
// along that space's axes, so a turned element's blur turns with it.

struct FilterParams {
  // x: the op (FilterOp's code), y: flags (FLAG_*), z: x channel, w: y channel.
  op: vec4<u32>,
  // The layer target's top left (CSS px) and device px per CSS px.
  origin: vec4<f32>,
  // The subregion, in the filter's space.
  subregion: vec4<f32>,
  // The filter's space to CSS px: the linear part (a, b, c, d; x' =
  // a x + c y + e) and the translation (xy), then its inverse's.
  space: vec4<f32>,
  space_t: vec4<f32>,
  inverse: vec4<f32>,
  inverse_t: vec4<f32>,
  a: vec4<f32>,
  b: vec4<f32>,
  c: vec4<f32>,
  // A colour matrix's rows, or a component transfer's functions (type,
  // slope/amplitude, intercept/exponent, offset; per channel) and lengths.
  m: array<vec4<f32>, 5>,
  // Component transfer tables: 16 values per channel.
  table: array<vec4<f32>, 16>,
};

@group(0) @binding(0) var in1_tex: texture_2d<f32>;
@group(0) @binding(1) var in2_tex: texture_2d<f32>;
@group(0) @binding(2) var aux_tex: texture_2d<f32>;
@group(0) @binding(3) var<uniform> f: FilterParams;

const FLAG_SRGB: u32 = 1u;
const FLAG_IN1_ALPHA: u32 = 2u;
const FLAG_IN2_ALPHA: u32 = 4u;
const FLAG_STITCH: u32 = 16u;
const FLAG_FRACTAL: u32 = 32u;
const FLAG_CLAMP: u32 = 64u;

@vertex
fn vs_filter(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  return fullscreen_clip(i);
}

fn to_srgb(c: vec3<f32>) -> vec3<f32> {
  let x = clamp(c, vec3<f32>(0.0), vec3<f32>(1.0));
  return select(1.055 * pow(x, vec3<f32>(1.0 / 2.4)) - 0.055, x * 12.92, x <= vec3<f32>(0.0031308));
}

fn to_linear(c: vec3<f32>) -> vec3<f32> {
  let x = clamp(c, vec3<f32>(0.0), vec3<f32>(1.0));
  return select(pow((x + 0.055) / 1.055, vec3<f32>(2.4)), x / 12.92, x <= vec3<f32>(0.04045));
}

fn unpremul(c: vec4<f32>) -> vec4<f32> {
  if (c.a <= 0.0) {
    return vec4<f32>(0.0);
  }
  return vec4<f32>(c.rgb / c.a, c.a);
}

// A premultiplied colour in the primitive's colour space.
fn space_in(c: vec4<f32>) -> vec4<f32> {
  if ((f.op.y & FLAG_SRGB) == 0u) {
    return c;
  }
  let u = unpremul(c);
  return vec4<f32>(to_srgb(u.rgb) * u.a, u.a);
}

fn space_out(c: vec4<f32>) -> vec4<f32> {
  if ((f.op.y & FLAG_SRGB) == 0u) {
    return c;
  }
  let u = unpremul(c);
  return vec4<f32>(to_linear(u.rgb) * u.a, u.a);
}

fn texel(t: texture_2d<f32>, p: vec2<i32>) -> vec4<f32> {
  let size = vec2<i32>(textureDimensions(t));
  if ((f.op.y & FLAG_CLAMP) != 0u) {
    return textureLoad(t, clamp(p, vec2<i32>(0), size - 1), 0);
  }
  if (any(p < vec2<i32>(0)) || any(p >= size)) {
    return vec4<f32>(0.0);
  }
  return textureLoad(t, p, 0);
}

// Bilinear at device position `p` (pixel centres at .5), transparent
// outside.
fn bilinear(t: texture_2d<f32>, p: vec2<f32>) -> vec4<f32> {
  let q = p - 0.5;
  let i = vec2<i32>(floor(q));
  let w = q - floor(q);
  let a = mix(texel(t, i), texel(t, i + vec2<i32>(1, 0)), w.x);
  let b = mix(texel(t, i + vec2<i32>(0, 1)), texel(t, i + vec2<i32>(1, 1)), w.x);
  return mix(a, b, w.y);
}

fn in1_at(p: vec2<f32>) -> vec4<f32> {
  var c = bilinear(in1_tex, p);
  if ((f.op.y & FLAG_IN1_ALPHA) != 0u) {
    c = vec4<f32>(0.0, 0.0, 0.0, c.a);
  }
  return space_in(c);
}

fn in2_at(p: vec2<f32>) -> vec4<f32> {
  var c = bilinear(in2_tex, p);
  if ((f.op.y & FLAG_IN2_ALPHA) != 0u) {
    c = vec4<f32>(0.0, 0.0, 0.0, c.a);
  }
  return space_in(c);
}

// The CSS position of device position `p`.
fn css(p: vec2<f32>) -> vec2<f32> {
  return f.origin.xy + p / f.origin.z;
}

fn linear_part(m: vec4<f32>) -> mat2x2<f32> {
  return mat2x2<f32>(m.xy, m.zw);
}

// The point of the filter's space at CSS position `q`.
fn local(q: vec2<f32>) -> vec2<f32> {
  return linear_part(f.inverse) * q + f.inverse_t.xy;
}

// The device position of a point of the filter's space.
fn device_of(l: vec2<f32>) -> vec2<f32> {
  return (linear_part(f.space) * l + f.space_t.xy - f.origin.xy) * f.origin.z;
}

// A vector of the filter's space in device px.
fn device_vec(v: vec2<f32>) -> vec2<f32> {
  return linear_part(f.space) * v * f.origin.z;
}

// in1's texel `i` (a morphology keeps the values it finds).
fn in1_texel(i: vec2<i32>) -> vec4<f32> {
  var c = texel(in1_tex, i);
  if ((f.op.y & FLAG_IN1_ALPHA) != 0u) {
    c = vec4<f32>(0.0, 0.0, 0.0, c.a);
  }
  return space_in(c);
}

// ---- feTurbulence (Filter Effects 1 §15.24, the reference implementation)

// The lattice (aux_tex, 514 × 4): texel (i, k) holds channel k's gradient
// at i in xy and the lattice selector at i in z.
fn lattice(i: i32) -> i32 {
  return i32(textureLoad(aux_tex, vec2<i32>(i, 0), 0).z);
}

fn gradient_at(k: i32, i: i32) -> vec2<f32> {
  return textureLoad(aux_tex, vec2<i32>(i, k), 0).xy;
}

fn s_curve(t: f32) -> f32 {
  return t * t * (3.0 - 2.0 * t);
}

// The stitch tile: width, height, wrap x, wrap y.
fn noise2(k: i32, vx: f32, vy: f32, stitch: vec4<f32>, on: bool) -> f32 {
  let n = 4096.0;
  let tx = vx + n;
  var bx0 = i32(tx) & 255;
  var bx1 = (bx0 + 1) & 255;
  let rx0 = tx - floor(tx);
  let rx1 = rx0 - 1.0;
  let ty = vy + n;
  var by0 = i32(ty) & 255;
  var by1 = (by0 + 1) & 255;
  let ry0 = ty - floor(ty);
  let ry1 = ry0 - 1.0;
  if (on) {
    let w = i32(stitch.x);
    let h = i32(stitch.y);
    let wx = i32(stitch.z);
    let wy = i32(stitch.w);
    if (bx0 >= wx) { bx0 -= w; }
    if (bx1 >= wx) { bx1 -= w; }
    if (by0 >= wy) { by0 -= h; }
    if (by1 >= wy) { by1 -= h; }
    bx0 = bx0 & 255;
    bx1 = bx1 & 255;
    by0 = by0 & 255;
    by1 = by1 & 255;
  }
  let i = lattice(bx0);
  let j = lattice(bx1);
  let b00 = lattice(i + by0);
  let b10 = lattice(j + by0);
  let b01 = lattice(i + by1);
  let b11 = lattice(j + by1);
  let sx = s_curve(rx0);
  let sy = s_curve(ry0);
  let u0 = dot(gradient_at(k, b00), vec2<f32>(rx0, ry0));
  let v0 = dot(gradient_at(k, b10), vec2<f32>(rx1, ry0));
  let a = mix(u0, v0, sx);
  let u1 = dot(gradient_at(k, b01), vec2<f32>(rx0, ry1));
  let v1 = dot(gradient_at(k, b11), vec2<f32>(rx1, ry1));
  let b = mix(u1, v1, sx);
  return mix(a, b, sy);
}

fn turbulence(k: i32, point: vec2<f32>) -> f32 {
  let fractal = (f.op.y & FLAG_FRACTAL) != 0u;
  let on = (f.op.y & FLAG_STITCH) != 0u;
  var base = f.a.xy;
  // The stitch tile in user units: c = x, y, width, height.
  var stitch = vec4<f32>(0.0);
  if (on) {
    let t = f.c;
    if (base.x != 0.0) {
      let lo = floor(t.z * base.x) / t.z;
      let hi = ceil(t.z * base.x) / t.z;
      base.x = select(hi, lo, base.x / lo < hi / base.x);
    }
    if (base.y != 0.0) {
      let lo = floor(t.w * base.y) / t.w;
      let hi = ceil(t.w * base.y) / t.w;
      base.y = select(hi, lo, base.y / lo < hi / base.y);
    }
    let w = floor(t.z * base.x + 0.5);
    let h = floor(t.w * base.y + 0.5);
    stitch = vec4<f32>(w, h, t.x * base.x + 4096.0 + w, t.y * base.y + 4096.0 + h);
  }
  var sum = 0.0;
  var v = point * base;
  var ratio = 1.0;
  let octaves = i32(f.a.z);
  for (var o = 0; o < octaves; o++) {
    let n = noise2(k, v.x, v.y, stitch, on);
    if (fractal) {
      sum += n / ratio;
    } else {
      sum += abs(n) / ratio;
    }
    v *= 2.0;
    ratio *= 2.0;
    if (on) {
      stitch = vec4<f32>(2.0 * stitch.x, 2.0 * stitch.y, 2.0 * stitch.z - 4096.0, 2.0 * stitch.w - 4096.0);
    }
  }
  if (fractal) {
    return (sum + 1.0) / 2.0;
  }
  return sum;
}

// ---- lighting (Filter Effects 1 §15.13): the surface from in1's alpha,
// in the filter's space

fn alpha_at(p: vec2<f32>) -> f32 {
  return bilinear(in1_tex, p).a;
}

// The Sobel normal over cells whose edges are m[2] (device px), each
// step's slope scaled by m[3].xy (into slope per unit of the space).
fn surface_normal(p: vec2<f32>, scale: f32) -> vec3<f32> {
  let u = f.m[2].xy;
  let v = f.m[2].zw;
  let a00 = alpha_at(p - u - v);
  let a10 = alpha_at(p - v);
  let a20 = alpha_at(p + u - v);
  let a01 = alpha_at(p - u);
  let a21 = alpha_at(p + u);
  let a02 = alpha_at(p - u + v);
  let a12 = alpha_at(p + v);
  let a22 = alpha_at(p + u + v);
  let nx = -scale * 0.25 * f.m[3].x * ((a20 + 2.0 * a21 + a22) - (a00 + 2.0 * a01 + a02));
  let ny = -scale * 0.25 * f.m[3].y * ((a02 + 2.0 * a12 + a22) - (a00 + 2.0 * a10 + a20));
  return normalize(vec3<f32>(nx, ny, 1.0));
}

// a: surface scale, constant, exponent, light kind (0 distant, 1 point,
// 2 spot); b: azimuth, elevation (rad) or the light's x, y, z (the
// filter's space); c: spot's points-at x, y, z, specular exponent; m[0]:
// the colour (linear), m[1].x: the cone's cosine (-2: none), m[1].y: a
// cell's height in units; m[2], m[3]: the cells (surface_normal).
fn lighting(p: vec2<f32>, q: vec2<f32>, specular: bool) -> vec4<f32> {
  let surface = f.a.x;
  let n = surface_normal(p, surface);
  let z = surface * f.m[1].y * alpha_at(p);
  var l: vec3<f32>;
  var color = f.m[0].rgb;
  let kind = i32(f.a.w);
  if (kind == 0) {
    l = vec3<f32>(cos(f.b.x) * cos(f.b.y), sin(f.b.x) * cos(f.b.y), sin(f.b.y));
  } else {
    l = normalize(f.b.xyz - vec3<f32>(local(q), z));
    if (kind == 2) {
      let s = normalize(f.c.xyz - f.b.xyz);
      let minus_l_dot_s = -dot(l, s);
      let cone = f.m[1].x;
      if (minus_l_dot_s <= 0.0 || (cone > -1.5 && minus_l_dot_s < cone)) {
        color = vec3<f32>(0.0);
      } else {
        color = color * pow(minus_l_dot_s, f.c.w);
      }
    }
  }
  if (!specular) {
    let k = f.a.y * max(dot(n, l), 0.0);
    return vec4<f32>(clamp(k * color, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
  }
  let h = normalize(l + vec3<f32>(0.0, 0.0, 1.0));
  let k = f.a.y * pow(max(dot(n, h), 0.0), f.a.z);
  let rgb = clamp(k * color, vec3<f32>(0.0), vec3<f32>(1.0));
  return vec4<f32>(rgb, max(rgb.r, max(rgb.g, rgb.b)));
}

// ---- component transfer (§15.11), on unpremultiplied values

fn transfer(c: f32, ch: u32) -> f32 {
  let fun = f.m[ch];
  let kind = i32(fun.x);
  let n = i32(f.m[4][ch]);
  if (kind == 1 || kind == 2) {
    if (n < 1) {
      return c;
    }
    let base = i32(ch) * 16;
    if (kind == 1) {
      // table: linear between n values.
      if (n == 1) {
        return f.table[base / 4][0];
      }
      let x = clamp(c, 0.0, 1.0) * f32(n - 1);
      let k = min(i32(floor(x)), n - 2);
      let v0 = f.table[(base + k) / 4][(base + k) % 4];
      let v1 = f.table[(base + k + 1) / 4][(base + k + 1) % 4];
      return mix(v0, v1, x - f32(k));
    }
    // discrete: n steps.
    let k = min(i32(floor(clamp(c, 0.0, 1.0) * f32(n))), n - 1);
    return f.table[(base + k) / 4][(base + k) % 4];
  }
  if (kind == 3) {
    return fun.y * c + fun.z;
  }
  if (kind == 4) {
    return fun.y * pow(max(c, 0.0), fun.z) + fun.w;
  }
  return c;
}

@fragment
fn fs_filter(@builtin(position) at: vec4<f32>) -> @location(0) vec4<f32> {
  let p = at.xy;
  let q = css(p);
  let l = local(q);
  let r = f.subregion;
  if (l.x < r.x || l.y < r.y || l.x > r.x + r.z || l.y > r.y + r.w) {
    return vec4<f32>(0.0);
  }
  var out = vec4<f32>(0.0);
  switch f.op.x {
    case 0u: {
      // Blur: one axis of a Gaussian, a.xy the image of the space's axis
      // scaled by the standard deviation (device px), every device px
      // within 3σ (at most 128 a side: a wider blur is split into
      // passes, src/ui/filter.rs).
      let sigma = length(f.a.xy);
      let dir = f.a.xy / max(sigma, 1e-6);
      if (sigma <= 1e-4) {
        out = in1_at(p);
      } else {
        let reach = min(ceil(3.0 * sigma), 128.0);
        var sum = vec4<f32>(0.0);
        var total = 0.0;
        for (var i = -reach; i <= reach; i += 1.0) {
          let w = exp(-0.5 * i * i / (sigma * sigma));
          sum += w * in1_at(p + dir * i);
          total += w;
        }
        out = sum / total;
      }
    }
    case 1u: {
      // Offset by a.xy device px (the space's offset on the screen).
      out = in1_at(p - f.a.xy);
    }
    case 2u: {
      // Flood: a premultiplied colour, in the primitive's space.
      out = space_in(f.a);
    }
    case 3u: {
      // Composite of in1 (A) over in2 (B) by operator b.x; k in a.
      let a = in1_at(p);
      let b = in2_at(p);
      switch u32(f.b.x) {
        case 0u: { out = a + b * (1.0 - a.a); }
        case 1u: { out = a * b.a; }
        case 2u: { out = a * (1.0 - b.a); }
        case 3u: { out = a * b.a + b * (1.0 - a.a); }
        case 4u: { out = a * (1.0 - b.a) + b * (1.0 - a.a); }
        case 5u: { out = min(a + b, vec4<f32>(1.0)); }
        default: {
          let k = f.a;
          out = clamp(k.x * a * b + k.y * a + k.z * b + vec4<f32>(k.w), vec4<f32>(0.0), vec4<f32>(1.0));
          out = vec4<f32>(min(out.rgb, vec3<f32>(out.a)), out.a);
        }
      }
    }
    case 4u: {
      // Blend in1 (source) over in2 (backdrop) by mode b.x (UiBlend's
      // code): co = cs (1 - ab) + cb (1 - as) + as ab B(Cb, Cs).
      let s = in1_at(p);
      let b = in2_at(p);
      let cs = unpremul(s).rgb;
      let cb = unpremul(b).rgb;
      let mode = u32(f.b.x);
      var mixed: vec3<f32>;
      switch mode {
        case 0u: { mixed = cs; }
        case 3u: { mixed = min(cs + cb, vec3<f32>(1.0)); }
        case 13u: { mixed = set_lum(set_sat(cs, sat(cb)), lum(cb)); }
        case 14u: { mixed = set_lum(set_sat(cb, sat(cs)), lum(cb)); }
        case 15u: { mixed = set_lum(cs, lum(cb)); }
        case 16u: { mixed = set_lum(cb, lum(cs)); }
        default: {
          mixed = vec3<f32>(blend_channel(cb.r, cs.r, mode), blend_channel(cb.g, cs.g, mode), blend_channel(cb.b, cs.b, mode));
        }
      }
      let rgb = s.rgb * (1.0 - b.a) + b.rgb * (1.0 - s.a) + s.a * b.a * clamp(mixed, vec3<f32>(0.0), vec3<f32>(1.0));
      out = vec4<f32>(rgb, s.a + b.a - s.a * b.a);
    }
    case 5u: {
      // Colour matrix on unpremultiplied values.
      let c = unpremul(in1_at(p));
      let v = vec4<f32>(c.rgb, c.a);
      var o: vec4<f32>;
      for (var i = 0; i < 4; i++) {
        let row0 = f.m[i];
        o[i] = dot(row0, v) + f.table[0][i];
      }
      o = clamp(o, vec4<f32>(0.0), vec4<f32>(1.0));
      out = vec4<f32>(o.rgb * o.a, o.a);
    }
    case 6u: {
      // Component transfer on unpremultiplied values.
      let c = unpremul(in1_at(p));
      let o = clamp(vec4<f32>(transfer(c.r, 0u), transfer(c.g, 1u), transfer(c.b, 2u), transfer(c.a, 3u)), vec4<f32>(0.0), vec4<f32>(1.0));
      out = vec4<f32>(o.rgb * o.a, o.a);
    }
    case 7u: {
      // Morphology along one axis of the space: the min (erode) or max
      // (dilate) of each channel over every texel the segment of a.xy
      // (the radius along it, device px, whole) either side of the pixel
      // crosses: a grid walk (Amanatides and Woo), so a turned axis
      // misses no texel; along a device axis, the row or column itself.
      let radius = length(f.a.xy);
      let dilate = f.a.z > 0.5;
      let reach = min(floor(radius + 1e-3), 128.0);
      let d = f.a.xy / max(radius, 1e-6) * reach;
      let a = p - d;
      let b = p + d;
      var cell = vec2<i32>(floor(a));
      let last = vec2<i32>(floor(b));
      let span = b - a;
      let dir_step = vec2<i32>(sign(span));
      let inv = 1.0 / max(abs(span), vec2<f32>(1e-9));
      let first = select(a - floor(a), floor(a) + 1.0 - a, span > vec2<f32>(0.0));
      var t_max = select(vec2<f32>(2.0), first * inv, abs(span) > vec2<f32>(1e-6));
      var o = in1_texel(cell);
      for (var n = 0; n < 520; n++) {
        if (all(cell == last)) {
          break;
        }
        if (t_max.x < t_max.y) {
          cell.x += dir_step.x;
          t_max.x += inv.x;
        } else {
          cell.y += dir_step.y;
          t_max.y += inv.y;
        }
        let c = in1_texel(cell);
        o = select(min(o, c), max(o, c), dilate);
      }
      out = o;
    }
    case 8u: {
      // Displacement of in1 by in2's channels (unpremultiplied), a.x
      // units of the space per unit.
      let m = unpremul(bilinear(in2_tex, p));
      let d = vec2<f32>(m[f.op.z], m[f.op.w]) - 0.5;
      out = in1_at(p + device_vec(f.a.x * d));
    }
    case 9u: {
      // Turbulence at the noise point: b.xy its origin in the space, b.z
      // noise units per unit.
      let point = (l - f.b.xy) * f.b.z;
      var o = vec4<f32>(turbulence(0, point), turbulence(1, point), turbulence(2, point), turbulence(3, point));
      o = clamp(o, vec4<f32>(0.0), vec4<f32>(1.0));
      out = vec4<f32>(o.rgb * o.a, o.a);
    }
    case 10u: {
      // Image: aux_tex's uv rect b over rect a of the space, straight
      // sRGB-decoded colour as the sampler reads it, transparent outside.
      let t = (l - f.a.xy) / max(f.a.zw, vec2<f32>(1e-4));
      if (all(t >= vec2<f32>(0.0)) && all(t <= vec2<f32>(1.0))) {
        let size = vec2<f32>(textureDimensions(aux_tex));
        let c = bilinear(aux_tex, mix(f.b.xy, f.b.zw, t) * size);
        out = space_in(vec4<f32>(c.rgb * c.a, c.a));
      }
    }
    case 11u: {
      // Tile: in1's subregion a (the space) repeated.
      let t = f.a;
      let k = t.xy + (l - t.xy) - floor((l - t.xy) / max(t.zw, vec2<f32>(1e-4))) * t.zw;
      out = in1_at(device_of(k));
    }
    case 12u: { out = lighting(p, q, false); }
    case 13u: { out = lighting(p, q, true); }
    case 16u: {
      // An element's layer (aux_tex), its top left a.xy (CSS px) and a.z
      // device px per CSS px; transparent outside it.
      if (f.a.z > 0.0) {
        let at = (q - f.a.xy) * f.a.z;
        let size = vec2<f32>(textureDimensions(aux_tex));
        if (all(at >= vec2<f32>(0.0)) && all(at <= size)) {
          let c = textureLoad(aux_tex, clamp(vec2<i32>(floor(at)), vec2<i32>(0), vec2<i32>(size) - 1), 0);
          out = space_in(c);
        }
      }
    }
    case 17u: {
      // A backdrop's capture: in1 is the target under the layer, its top
      // left a.xy (CSS px) and a.z device px per CSS px, sRGB-encoded
      // when a.w is 1; premultiplied linear out.
      var c = bilinear(in1_tex, (q - f.a.xy) * f.a.z);
      if (f.a.w > 0.5) {
        let u = unpremul(c);
        c = vec4<f32>(to_linear(u.rgb) * u.a, u.a);
      }
      return c;
    }
    case 15u: {
      // Convolve matrix (§15.12): op.zw the order, a the divisor, bias and
      // target, b.x the edge mode (0 duplicate, 1 wrap, 2 none), b.y
      // preserve alpha, m[2] a cell's edges (device px); the kernel in
      // table, row-major.
      let order = vec2<i32>(i32(f.op.z), i32(f.op.w));
      let size = vec2<f32>(textureDimensions(in1_tex));
      let keep = f.b.y > 0.5;
      let edge = u32(f.b.x);
      var sum = vec4<f32>(0.0);
      for (var i = 0; i < order.y; i++) {
        for (var j = 0; j < order.x; j++) {
          var s = p + f32(j - i32(f.a.z)) * f.m[2].xy + f32(i - i32(f.a.w)) * f.m[2].zw;
          if (edge == 0u) {
            s = clamp(s, vec2<f32>(0.5), size - 0.5);
          } else if (edge == 1u) {
            s = s - floor(s / size) * size;
          }
          var c = bilinear(in1_tex, s);
          if ((f.op.y & FLAG_IN1_ALPHA) != 0u) {
            c = vec4<f32>(0.0, 0.0, 0.0, c.a);
          }
          c = space_in(c);
          if (keep) {
            c = unpremul(c);
          }
          let k = order.x * order.y - 1 - (i * order.x + j);
          sum += c * f.table[k / 4][k % 4];
        }
      }
      sum = sum / f.a.x;
      if (keep) {
        let a = in1_at(p).a;
        let rgb = clamp(sum.rgb + f.a.y, vec3<f32>(0.0), vec3<f32>(1.0));
        out = vec4<f32>(rgb * a, a);
      } else {
        let a = clamp(sum.a + f.a.y, 0.0, 1.0);
        out = vec4<f32>(clamp(sum.rgb + f.a.y * a, vec3<f32>(0.0), vec3<f32>(a)), a);
      }
    }
    default: {
      // Merge's step: in1 over in2.
      let a = in1_at(p);
      out = a + in2_at(p) * (1.0 - a.a);
    }
  }
  return space_out(out);
}
