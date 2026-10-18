// Screen-space effects over the depth-normal prepass: ambient occlusion
// (GTAO and SSAO, then a depth-aware blur), outlines, bokeh depth of field
// (circle of confusion, a half-resolution gather, the composite), motion
// blur along the prepass velocity, and god rays (a sky mask around the sun
// blurred radially). Prepended by fullscreen.wgsl. Every pass binds the same
// layout; slots a pass does not use hold placeholders.

struct FxPost {
  proj: mat4x4<f32>,
  inv_proj: mat4x4<f32>,
  view: mat4x4<f32>,          // world -> view (normals)
  screen: vec4<f32>,          // full-resolution w, h, 1/w, 1/h
  cam: vec4<f32>,             // x: far, y: 1 orthographic
  ao: vec4<f32>,              // x: radius (world), y: intensity
  outline_color: vec4<f32>,   // the scene's outline (linear rgb, coverage)
  outline: vec4<f32>,         // x: scene width (0: none), y: depth threshold, z: normal threshold
  dof: vec4<f32>,             // x: focus distance, y: focus range, z: max blur (full-res px), w: falloff
  dof2: vec4<f32>,            // x: taps
  mb: vec4<f32>,              // x: shutter, y: samples, z: max px
  god: vec4<f32>,             // xy: sun uv, z: decay, w: density
  god2: vec4<f32>,            // x: samples, y: 1 when the sun is in front, z: intensity, w: sky threshold
  god_tint: vec4<f32>,
};
@group(0) @binding(0) var<uniform> fxp: FxPost;
@group(0) @binding(1) var fx_linear: sampler;
@group(0) @binding(2) var tex_a: texture_2d<f32>;   // filterable: the lit image, a half-resolution input
@group(0) @binding(3) var tex_depth: texture_2d<f32>;  // the prepass view depth (R32Float)
@group(0) @binding(4) var tex_gbuffer: texture_2d<f32>;  // oct normal, velocity
@group(0) @binding(5) var tex_b: texture_2d<f32>;   // filterable: outline ids, AO, the blurred half

const FX_PI: f32 = 3.14159265359;

fn depth_at(px: vec2<i32>) -> f32 {
  let size = vec2<i32>(textureDimensions(tex_depth));
  return textureLoad(tex_depth, clamp(px, vec2<i32>(0), size - 1), 0).r;
}

fn depth_uv(uv: vec2<f32>) -> f32 {
  return depth_at(vec2<i32>(uv * vec2<f32>(textureDimensions(tex_depth))));
}

fn is_sky(d: f32) -> bool {
  return d >= fxp.cam.x * 0.999;
}

fn oct_decode(e: vec2<f32>) -> vec3<f32> {
  var n = vec3<f32>(e, 1.0 - abs(e.x) - abs(e.y));
  if (n.z < 0.0) {
    n = vec3<f32>((1.0 - abs(n.yx)) * select(vec2<f32>(-1.0), vec2<f32>(1.0), n.xy >= vec2<f32>(0.0)), n.z);
  }
  return normalize(n);
}

fn world_normal(px: vec2<i32>) -> vec3<f32> {
  let size = vec2<i32>(textureDimensions(tex_gbuffer));
  return oct_decode(textureLoad(tex_gbuffer, clamp(px, vec2<i32>(0), size - 1), 0).xy);
}

// The view-space point at `uv` whose view depth is `d`.
fn view_pos(uv: vec2<f32>, d: f32) -> vec3<f32> {
  let ndc = vec2<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0);
  let a = fxp.inv_proj * vec4<f32>(ndc, 0.0, 1.0);
  let b = fxp.inv_proj * vec4<f32>(ndc, 1.0, 1.0);
  let p0 = a.xyz / a.w;
  let p1 = b.xyz / b.w;
  let t = (d + p0.z) / (p0.z - p1.z);
  return mix(p0, p1, t);
}

// Interleaved gradient noise (Jimenez): a fixed per-pixel pattern.
fn ign(px: vec2<f32>) -> f32 {
  return fract(52.9829189 * fract(dot(px, vec2<f32>(0.06711056, 0.00583715))));
}

// Pixels per world unit at view depth `d` (vertically).
fn pixels_per_unit(d: f32) -> f32 {
  let s = fxp.proj[1][1] * 0.5 * fxp.screen.y;
  return select(s / max(d, 1e-4), s, fxp.cam.y > 0.5);
}

@fragment
fn fs_gtao(in: VsOut) -> @location(0) vec4<f32> {
  let px = vec2<i32>(in.pos.xy);
  let d = depth_at(px);
  if (is_sky(d)) { return vec4<f32>(1.0); }
  let p = view_pos(in.uv, d);
  let n = normalize((fxp.view * vec4<f32>(world_normal(px), 0.0)).xyz);
  let v = select(normalize(-p), vec3<f32>(0.0, 0.0, 1.0), fxp.cam.y > 0.5);
  let radius = fxp.ao.x;
  let radius_px = radius * pixels_per_unit(d);
  if (radius_px < 1.0) { return vec4<f32>(1.0); }
  let noise = ign(in.pos.xy);
  let slices = 3u;
  let steps = 6u;
  var vis = 0.0;
  for (var s = 0u; s < slices; s++) {
    let phi = (f32(s) + noise) * FX_PI / f32(slices);
    let omega = vec2<f32>(cos(phi), -sin(phi));
    let dir = vec3<f32>(cos(phi), sin(phi), 0.0);
    let ortho = dir - dot(dir, v) * v;
    let axis = normalize(cross(ortho, v));
    let proj_n = n - axis * dot(n, axis);
    let proj_len = length(proj_n);
    let sign_n = sign(dot(ortho, proj_n));
    let cos_n = clamp(dot(proj_n, v) / max(proj_len, 1e-5), 0.0, 1.0);
    let ang_n = sign_n * acos(cos_n);
    let low0 = cos(ang_n + FX_PI * 0.5);
    let low1 = cos(ang_n - FX_PI * 0.5);
    var h0 = low0;
    var h1 = low1;
    for (var k = 1u; k <= steps; k++) {
      let t = (f32(k) - 0.5 + fract(noise * 7.13 + f32(s) * 0.37)) / f32(steps);
      let offset = omega * (t * t * radius_px) * fxp.screen.zw;
      let uv0 = in.uv + offset;
      let uv1 = in.uv - offset;
      let s0 = view_pos(uv0, depth_uv(uv0)) - p;
      let s1 = view_pos(uv1, depth_uv(uv1)) - p;
      let l0 = length(s0);
      let l1 = length(s1);
      let w0 = clamp(2.0 - 2.0 * l0 / radius, 0.0, 1.0);
      let w1 = clamp(2.0 - 2.0 * l1 / radius, 0.0, 1.0);
      h0 = max(h0, mix(low0, dot(s0 / max(l0, 1e-5), v), w0));
      h1 = max(h1, mix(low1, dot(s1 / max(l1, 1e-5), v), w1));
    }
    var a0 = -acos(clamp(h1, -1.0, 1.0));
    var a1 = acos(clamp(h0, -1.0, 1.0));
    a0 = ang_n + clamp(a0 - ang_n, -FX_PI * 0.5, FX_PI * 0.5);
    a1 = ang_n + clamp(a1 - ang_n, -FX_PI * 0.5, FX_PI * 0.5);
    let arc0 = (cos_n + 2.0 * a0 * sin(ang_n) - cos(2.0 * a0 - ang_n)) * 0.25;
    let arc1 = (cos_n + 2.0 * a1 * sin(ang_n) - cos(2.0 * a1 - ang_n)) * 0.25;
    vis += proj_len * (arc0 + arc1);
  }
  vis = clamp(vis / f32(slices), 0.0, 1.0);
  return vec4<f32>(pow(vis, max(fxp.ao.y, 0.0)), 0.0, 0.0, 1.0);
}

// Normal-oriented hemisphere SSAO: 8 fixed samples, rotated per pixel.
@fragment
fn fs_ssao(in: VsOut) -> @location(0) vec4<f32> {
  let px = vec2<i32>(in.pos.xy);
  let d = depth_at(px);
  if (is_sky(d)) { return vec4<f32>(1.0); }
  let p = view_pos(in.uv, d);
  let n = normalize((fxp.view * vec4<f32>(world_normal(px), 0.0)).xyz);
  let radius = fxp.ao.x;
  let a = ign(in.pos.xy) * 6.2831853;
  var helper = vec3<f32>(cos(a), sin(a), 0.0);
  if (abs(dot(helper, n)) > 0.95) { helper = vec3<f32>(0.0, 0.0, 1.0); }
  let t = normalize(helper - n * dot(helper, n));
  let b = cross(n, t);
  var kernel = array<vec3<f32>, 8>(
    vec3<f32>(0.53, 0.18, 0.42), vec3<f32>(-0.38, 0.47, 0.30), vec3<f32>(-0.20, -0.55, 0.52),
    vec3<f32>(0.62, -0.31, 0.21), vec3<f32>(0.10, 0.78, 0.55), vec3<f32>(-0.71, -0.12, 0.40),
    vec3<f32>(0.27, -0.83, 0.38), vec3<f32>(-0.14, 0.09, 0.95));
  var occlusion = 0.0;
  for (var i = 0u; i < 8u; i++) {
    let k = kernel[i] * (0.35 + 0.65 * f32(i + 1u) / 8.0);
    let s = p + (t * k.x + b * k.y + n * k.z) * radius;
    let clip = fxp.proj * vec4<f32>(s, 1.0);
    let ndc = clip.xy / clip.w;
    let uv = vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
    let scene = depth_uv(uv);
    let sample_depth = -s.z;
    let range = clamp(radius / max(abs(d - scene), 1e-4), 0.0, 1.0);
    if (scene < sample_depth - 0.02 * radius) { occlusion += range; }
  }
  let ao = 1.0 - occlusion / 8.0;
  return vec4<f32>(pow(ao, max(fxp.ao.y, 0.0)), 0.0, 0.0, 1.0);
}

// A 4x4 depth-aware blur of the raw AO (tex_b) over its noise pattern.
@fragment
fn fs_ao_blur(in: VsOut) -> @location(0) vec4<f32> {
  let px = vec2<i32>(in.pos.xy);
  let d0 = depth_at(px);
  let size = vec2<i32>(textureDimensions(tex_b));
  var sum = 0.0;
  var weight = 0.0;
  for (var y = -2; y <= 1; y++) {
    for (var x = -2; x <= 1; x++) {
      let q = clamp(px + vec2<i32>(x, y), vec2<i32>(0), size - 1);
      let w = exp(-abs(depth_at(q) - d0) / max(d0 * 0.04, 1e-3));
      sum += textureLoad(tex_b, q, 0).r * w;
      weight += w;
    }
  }
  return vec4<f32>(sum / max(weight, 1e-5), 0.0, 0.0, 1.0);
}

// Outlines inside silhouettes (a farther neighbour at the line width) and
// along creases, in each object's own colour and width (tex_b: the prepass
// outline target) or the scene's.
@fragment
fn fs_outline(in: VsOut) -> @location(0) vec4<f32> {
  let px = vec2<i32>(in.pos.xy);
  let c = textureLoad(tex_a, px, 0);
  let id = textureLoad(tex_b, px, 0);
  var width = id.a * 32.0;
  var color = vec4<f32>(id.rgb, 1.0);
  if (width <= 0.0) {
    width = fxp.outline.x;
    color = fxp.outline_color;
  }
  if (width <= 0.0) { return c; }
  let d0 = depth_at(px);
  if (is_sky(d0)) { return c; }
  let n0 = world_normal(px);
  var edges = 0.0;
  for (var i = 0u; i < 8u; i++) {
    let a = f32(i) * FX_PI * 0.25;
    let q = px + vec2<i32>(round(vec2<f32>(cos(a), sin(a)) * width));
    let d = depth_at(q);
    if (d > d0 * (1.0 + fxp.outline.y)) {
      edges += 1.0;
    } else if (abs(d - d0) < d0 * fxp.outline.y * 4.0 && 1.0 - dot(world_normal(q), n0) > fxp.outline.z) {
      edges += 1.0;
    }
  }
  let k = clamp(edges, 0.0, 1.0) * color.a;
  return vec4<f32>(mix(c.rgb, color.rgb, k), c.a);
}

// The signed circle of confusion in half-resolution pixels (near < 0).
fn coc(d: f32) -> f32 {
  let off = d - fxp.dof.x;
  let blur = clamp((abs(off) - fxp.dof.y) / max(fxp.dof.y * fxp.dof.w, 1e-3), 0.0, 1.0) * fxp.dof.z * 0.5;
  return select(blur, -blur, off < 0.0);
}

// Half resolution: the lit image (2x2 average) and its circle of confusion.
@fragment
fn fs_dof_coc(in: VsOut) -> @location(0) vec4<f32> {
  let full = vec2<i32>(in.pos.xy) * 2;
  var c = vec3<f32>(0.0);
  var nearest = 1e30;
  for (var i = 0; i < 4; i++) {
    let q = full + vec2<i32>(i & 1, i >> 1u);
    c += textureLoad(tex_a, clamp(q, vec2<i32>(0), vec2<i32>(textureDimensions(tex_a)) - 1), 0).rgb;
    nearest = min(nearest, depth_at(q));
  }
  return vec4<f32>(c * 0.25, coc(nearest));
}

// The gather: a golden-angle disc of taps; a tap counts when its own
// circle covers this pixel (near taps bleed over sharp ones, far taps do not).
@fragment
fn fs_dof_blur(in: VsOut) -> @location(0) vec4<f32> {
  let size = vec2<f32>(textureDimensions(tex_a));
  let center = textureSampleLevel(tex_a, fx_linear, in.uv, 0.0);
  let c0 = center.a;
  let taps = u32(fxp.dof2.x);
  let max_r = fxp.dof.z * 0.5;
  var sum = center.rgb;
  var weight = 1.0;
  var near = 0.0;
  for (var k = 0u; k < taps; k++) {
    let r = sqrt((f32(k) + 0.5) / f32(taps)) * max_r;
    let th = f32(k) * 2.39996323;
    let uv = in.uv + vec2<f32>(cos(th), sin(th)) * r / size;
    let s = textureSampleLevel(tex_a, fx_linear, uv, 0.0);
    let cs = s.a;
    var reach = abs(cs);
    if (cs >= 0.0) { reach = min(cs, abs(c0)); }
    let w = clamp(reach - r + 1.0, 0.0, 1.0);
    // Bright taps weigh more: visible bokeh.
    let lum = dot(s.rgb, LUMA);
    let boost = 1.0 + 2.0 * clamp(lum - 1.0, 0.0, 4.0);
    sum += s.rgb * w * boost;
    weight += w * boost;
    if (cs < 0.0) { near += w; }
  }
  let near_cover = clamp(near / f32(max(taps, 1u)) * 3.0, 0.0, 1.0);
  let a = max(clamp(abs(c0) - 0.5, 0.0, 1.0), near_cover);
  return vec4<f32>(sum / weight, a);
}

@fragment
fn fs_dof_composite(in: VsOut) -> @location(0) vec4<f32> {
  let sharp = textureLoad(tex_a, vec2<i32>(in.pos.xy), 0);
  let b = textureSampleLevel(tex_b, fx_linear, in.uv, 0.0);
  return vec4<f32>(mix(sharp.rgb, b.rgb, b.a), sharp.a);
}

// Taps along the fastest velocity around this pixel (a cheap dilation, so a
// moving object smears over what is behind it), scaled by the shutter and
// clamped.
@fragment
fn fs_motion_blur(in: VsOut) -> @location(0) vec4<f32> {
  let px = vec2<i32>(in.pos.xy);
  let c = textureLoad(tex_a, px, 0);
  let size = vec2<i32>(textureDimensions(tex_gbuffer));
  var v = textureLoad(tex_gbuffer, px, 0).zw;
  let reach = i32(fxp.mb.z * 0.5);
  for (var i = 0u; i < 8u; i++) {
    let a = f32(i) * FX_PI * 0.25;
    let q = clamp(px + vec2<i32>(round(vec2<f32>(cos(a), sin(a)) * f32(reach))), vec2<i32>(0), size - 1);
    let w = textureLoad(tex_gbuffer, q, 0).zw;
    if (dot(w, w) > dot(v, v)) { v = w; }
  }
  v *= fxp.mb.x;
  let len_px = length(v * fxp.screen.xy);
  if (len_px < 0.5) { return c; }
  if (len_px > fxp.mb.z) { v *= fxp.mb.z / len_px; }
  let n = u32(fxp.mb.y);
  var sum = c.rgb;
  for (var i = 0u; i < n; i++) {
    let t = (f32(i) + 0.5) / f32(n) - 0.5;
    sum += textureSampleLevel(tex_a, fx_linear, in.uv + v * t, 0.0).rgb;
  }
  return vec4<f32>(sum / f32(n + 1u), c.a);
}

// Half resolution: the sky's light around the sun (geometry masks it out).
@fragment
fn fs_godrays_mask(in: VsOut) -> @location(0) vec4<f32> {
  let full = vec2<i32>(in.pos.xy) * 2;
  if (!is_sky(depth_at(full))) { return vec4<f32>(0.0, 0.0, 0.0, 1.0); }
  let aspect = fxp.screen.x * fxp.screen.w;
  let to_sun = (in.uv - fxp.god.xy) * vec2<f32>(aspect, 1.0);
  let fall = clamp(1.0 - length(to_sun) / 0.5, 0.0, 1.0);
  // Only the sky brighter than its surroundings carries rays: the sun and its glow.
  let c = textureSampleLevel(tex_a, fx_linear, in.uv, 0.0).rgb;
  let lum = dot(c, LUMA);
  let bright = max(lum - fxp.god2.w, 0.0) / max(lum, 1e-4);
  return vec4<f32>(min(c * bright, vec3<f32>(32.0)) * pow(fall, 3.0), 1.0);
}

// Half resolution: the mask (tex_b) blurred radially towards the sun.
@fragment
fn fs_godrays_blur(in: VsOut) -> @location(0) vec4<f32> {
  if (fxp.god2.y < 0.5) { return vec4<f32>(0.0, 0.0, 0.0, 1.0); }
  let n = u32(fxp.god2.x);
  let step = (in.uv - fxp.god.xy) * fxp.god.w / f32(max(n, 1u));
  var uv = in.uv;
  var decay = 1.0;
  var sum = vec3<f32>(0.0);
  for (var i = 0u; i < n; i++) {
    sum += textureSampleLevel(tex_b, fx_linear, uv, 0.0).rgb * decay;
    decay *= fxp.god.z;
    uv -= step;
  }
  // Normalized to 48 taps: the brightness does not depend on the tap count.
  return vec4<f32>(sum * (4.0 / f32(max(n, 1u))) * fxp.god_tint.rgb * fxp.god2.z, 1.0);
}
