// 2D lighting: the light map the lit sprite layers multiply with. Cleared to
// the ambient light; each light adds an instanced quad over its radius,
// `(1 - d/r)^falloff` (faded to zero over its last tenth), times its visibility from its row of the polar
// shadow map (light2d_shadow.wgsl): the fraction of the rays about the
// pixel's angle that reach it, over the penumbra a light of radius
// `source` gives at the blocker's distance (PCSS), integrated exactly
// (`visibility`), so penumbrae widen away from the occluder as a smooth
// ramp. A pixel inside the first occluder its ray meets is not
// shadowed by it. A spot light fades between the cosines of its outer and
// inner half-angles about its axis; a global light covers the whole map at
// full colour, unshadowed.
//
// With normal-mapped receivers (`fs_dir`) a second target keeps where the
// light comes from: each light adds its luminance-weighted unit direction
// to it (xyz, world space: towards the light at its height above the
// plane, a global light from its axis at its height) and the weight (w),
// so the receivers read the weighted mean direction (sprite.wgsl
// `normal_light`).

struct Lights2d {
  view_proj: mat4x4<f32>,
  info: vec4<f32>,                 // x: the shadow map's angles
};
@group(0) @binding(0) var<uniform> lights2d: Lights2d;
@group(0) @binding(1) var shadow_map: texture_2d<f32>;

struct LightIn {
  @location(0) pos_radius: vec4<f32>,   // xy: position, z: radius, w: falloff
  @location(1) color_source: vec4<f32>, // rgb: colour x intensity, w: source radius
  @location(2) flags: vec4<f32>,        // x: casts shadows, y: kind (0 point, 1 spot, 2 global), z, w: cos outer, cos inner
  @location(3) direction: vec4<f32>,    // xy: axis, z: height, w: shadow row (-1: none)
  @location(4) shadow: vec4<f32>,       // x: the distance to its nearest occluder
};

struct LightOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world: vec2<f32>,
  @location(1) light: vec4<f32>,
  @location(2) color_source: vec4<f32>,
  @location(3) flags: vec4<f32>,
  @location(4) direction: vec4<f32>,
  @location(5) shadow: vec4<f32>,
};

@vertex
fn vs(@builtin(vertex_index) vi: u32, l: LightIn) -> LightOut {
  var corners = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0),
    vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, 1.0), vec2<f32>(-1.0, 1.0));
  let p = l.pos_radius.xy + corners[vi] * l.pos_radius.z;
  var o: LightOut;
  o.clip = lights2d.view_proj * vec4<f32>(p, 0.0, 1.0);
  if (l.flags.y > 1.5) {
    // A global light: the whole strip.
    o.clip = vec4<f32>(corners[vi], 0.0, 1.0);
  }
  o.world = p;
  o.light = l.pos_radius;
  o.color_source = l.color_source;
  o.flags = l.flags;
  o.direction = l.direction;
  o.shadow = l.shadow;
  return o;
}

const TAU: f32 = 6.283185307179586;

// Texel `c` (wrapped) of `row`: the distance to the first occluder on
// its ray and where the ray leaves it.
fn texel(row: i32, c: i32) -> vec2<f32> {
  let n = i32(lights2d.info.x);
  return textureLoad(shadow_map, vec2<i32>(((c % n) + n) % n, row), 0).xy;
}

// Whether a ray of texel `v` reaches distance `d`: short of the first
// occluder, or inside it.
fn reach(v: vec2<f32>, d: f32) -> f32 {
  return select(0.0, 1.0, d <= v.x + 0.002 * d || d <= v.y);
}

// The mean of `reach` over columns x0..x1 (texel c spans c..c+1),
// integrated exactly rather than point-sampled: eight spans, and in a span
// whose ends differ, its transition found by bisection. One transition a
// span (an eighth of the window) is assumed; a gap narrower than that
// between two blockers may be missed, as a point tap would miss it.
fn lit_between(row: i32, x0: f32, x1: f32, d: f32) -> f32 {
  let w = x1 - x0;
  var xa = x0;
  var ca = i32(floor(x0));
  var va = reach(texel(row, ca), d);
  var acc = 0.0;
  for (var k = 1; k <= 8; k++) {
    let xb = x0 + w * f32(k) / 8.0;
    let cb = i32(floor(xb));
    let vb = reach(texel(row, cb), d);
    var t = xb;
    if (vb != va) {
      // The first column after ca that reads vb: ca reads va, cb vb.
      var lo = ca;
      var hi = cb;
      loop {
        if (hi - lo <= 1) { break; }
        let m = lo + (hi - lo) / 2;
        if (reach(texel(row, m), d) == va) { lo = m; } else { hi = m; }
      }
      t = f32(hi);
    }
    acc += va * (t - xa) + vb * (xb - t);
    xa = xb;
    ca = cb;
    va = vb;
  }
  return acc / w;
}

// How far column c is from x, in columns (0 for x's own).
fn apart(c: i32, x: f32) -> f32 {
  return max(0.0, max(f32(c) - x, x - f32(c + 1)));
}

// The blockers the pixel at column x, distance d, sees within `w` columns
// either side, as (sum of depths, count): a blocked texel of depth b at k
// columns counts only if a ray from the pixel to the light's disk crosses
// it, k within source (1/b - 1/d) radians. The window is scanned as
// lit_between scans it, so a blocked run's nearest texel is found exactly
// (by bisection) and a blocker joins as the pixel's view of the disk
// first touches it, where the filter it sets still reads fully lit: the
// lit end of the penumbra has no jump.
fn blockers(row: i32, x: f32, d: f32, source: f32, w: f32) -> vec2<f32> {
  let per_radian = lights2d.info.x / TAU;
  var s = vec2<f32>(0.0);
  var ca = i32(floor(x - w));
  var ta = texel(row, ca);
  var va = reach(ta, d);
  if (va == 0.0 && apart(ca, x) <= source * (1.0 / ta.x - 1.0 / d) * per_radian) {
    s += vec2<f32>(ta.x, 1.0);
  }
  for (var k = 1; k <= 8; k++) {
    let cb = i32(floor(x - w + 2.0 * w * f32(k) / 8.0));
    let tb = texel(row, cb);
    let vb = reach(tb, d);
    if (vb == 0.0 && apart(cb, x) <= source * (1.0 / tb.x - 1.0 / d) * per_radian) {
      s += vec2<f32>(tb.x, 1.0);
    }
    // A transition: its blocked texel, unless the span is too far for a
    // blocker even at half its blocked end's depth (a far edge costs no
    // bisection).
    let te = select(tb, ta, va == 0.0);
    if (vb != va && min(apart(ca, x), apart(cb, x)) <= source * (2.0 / te.x - 1.0 / d) * per_radian) {
      var lo = ca;
      var hi = cb;
      var tl = ta;
      var th = tb;
      loop {
        if (hi - lo <= 1) { break; }
        let m = lo + (hi - lo) / 2;
        let tm = texel(row, m);
        if (reach(tm, d) == va) { lo = m; tl = tm; } else { hi = m; th = tm; }
      }
      // The blocked side's end was counted already.
      let c = select(hi, lo, va == 0.0);
      let t = select(th, tl, va == 0.0);
      if (c != select(cb, ca, va == 0.0) && apart(c, x) <= source * (1.0 / t.x - 1.0 / d) * per_radian) {
        s += vec2<f32>(t.x, 1.0);
      }
    }
    ca = cb;
    ta = tb;
    va = vb;
  }
  return s;
}

// The fraction of the light of radius `source` at angle `theta` from the
// pixel at distance `d` that reaches it, `near` the distance from the
// light to its nearest occluder. PCSS (Fernando 2005) on the polar map:
// the mean depth of the blockers the pixel's view of the disk crosses
// sets the penumbra's half-angle, and the visible fraction is the box
// filter of the map over it, integrated exactly (lit_between): a ramp at
// every source size, where a fixed set of point taps drew a step each.
fn visibility(row: i32, theta: f32, d: f32, source: f32, near: f32) -> f32 {
  if (d <= near) { return 1.0; }
  let n = lights2d.info.x;
  let per_radian = n / TAU;
  let x = fract(theta / TAU) * n;
  // As far as a blocker could block: one at `near`, the nearest there is.
  let w = clamp(source * (1.0 / near - 1.0 / d) * per_radian, 0.5, n / 8.0);
  let s = blockers(row, x, d, source, w);
  if (s.y == 0.0) {
    // No blocker: lit, but for the texel's own edge (between two texels).
    let c = floor(x - 0.5);
    let t = x - 0.5 - c;
    return mix(reach(texel(row, i32(c)), d), reach(texel(row, i32(c) + 1), d), t);
  }
  let db = max(s.x / s.y, 1e-3);
  // The penumbra's half-angle at the pixel, at least half a texel.
  let half = clamp(source * (d - db) / (db * d) * per_radian, 0.5, n / 8.0);
  return lit_between(row, x - half, x + half, d);
}

// The light this light adds at the pixel (discarded where it adds none).
fn light_of(in: LightOut) -> vec3<f32> {
  if (in.flags.y > 1.5) {
    return in.color_source.rgb;
  }
  let to_light = in.light.xy - in.world;
  let dist = length(to_light);
  let r = in.light.z;
  if (dist >= r) { discard; }
  // Faded to zero slope over the last tenth of the radius: a falloff
  // below 1 would otherwise end on a hard rim (light2d_at.rs does the same).
  var atten = pow(1.0 - dist / r, max(in.light.w, 0.01)) * (1.0 - smoothstep(0.9, 1.0, dist / r));
  if (in.flags.y > 0.5) {
    // A spot: the cosine of the pixel's angle off the axis.
    let c = select(1.0, dot(-to_light / dist, in.direction.xy), dist > 1e-4);
    atten *= smoothstep(in.flags.z, max(in.flags.w, in.flags.z + 1e-4), c);
    if (atten <= 0.0) { discard; }
  }
  if (in.flags.x > 0.5 && in.direction.w >= 0.0 && dist > 1e-4) {
    let theta = atan2(-to_light.y, -to_light.x);
    atten *= visibility(i32(in.direction.w), theta, dist, max(in.color_source.w, 1e-3), in.shadow.x);
  }
  return in.color_source.rgb * atten;
}

@fragment
fn fs(in: LightOut) -> @location(0) vec4<f32> {
  return vec4<f32>(light_of(in), 0.0);
}

struct LightDirOut {
  @location(0) light: vec4<f32>,
  @location(1) dir: vec4<f32>,
};

@fragment
fn fs_dir(in: LightOut) -> LightDirOut {
  let c = light_of(in);
  var to_light = in.direction.xy;
  if (in.flags.y < 1.5) { to_light = in.light.xy - in.world; }
  let d = normalize(vec3<f32>(to_light, in.direction.z) + vec3<f32>(0.0, 0.0, 1e-6));
  let w = dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
  var o: LightDirOut;
  o.light = vec4<f32>(c, 0.0);
  o.dir = vec4<f32>(d * w, w);
  return o;
}
