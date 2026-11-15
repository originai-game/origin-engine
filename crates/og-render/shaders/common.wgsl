// What any shader may call, after the hashes (hash.wgsl): one copy each of
// pi, the octahedral map of unit vectors, the triangle that covers the
// screen, the view-space point under a pixel and interleaved gradient
// noise. Pure functions: a module that reads its own uniform passes it in.
// The library's files are LIBRARY_WGSL, which the frame group's text
// starts with (FRAME_WGSL) and a module without the frame group puts first.

// Pi as an f32 (every shorter spelling of it rounds to this one); write a
// turn as 2.0 * PI and other multiples from it.
const PI: f32 = 3.141592653589793;

// Octahedral encoding of a unit vector into -1..1 (two channels): the
// G-buffer's normals, the probes' maps, a compact vertex's normal
// (compact.rs `oct_encode` is the same map).
fn oct_encode(n: vec3<f32>) -> vec2<f32> {
  let p = n.xy / (abs(n.x) + abs(n.y) + abs(n.z));
  if (n.z >= 0.0) { return p; }
  return (1.0 - abs(p.yx)) * select(vec2<f32>(-1.0), vec2<f32>(1.0), p >= vec2<f32>(0.0));
}

// The unit vector `oct_encode` mapped to `e`.
fn oct_decode(e: vec2<f32>) -> vec3<f32> {
  var n = vec3<f32>(e, 1.0 - abs(e.x) - abs(e.y));
  if (n.z < 0.0) {
    n = vec3<f32>((1.0 - abs(n.yx)) * select(vec2<f32>(-1.0), vec2<f32>(1.0), n.xy >= vec2<f32>(0.0)), n.z);
  }
  return normalize(n);
}

// Vertex `i` (0 to 2) of the one triangle that covers the screen, in clip
// space at depth 0: (-1, -1), (3, -1), (-1, 3).
fn fullscreen_clip(i: u32) -> vec4<f32> {
  return vec4<f32>(f32(i32(i & 1u) * 4 - 1), f32(i32(i & 2u) * 2 - 1), 0.0, 1.0);
}

// The view-space point at `uv` whose view depth is `d`, through the
// projection's inverse (perspective or orthographic).
fn view_point(inv_proj: mat4x4<f32>, uv: vec2<f32>, d: f32) -> vec3<f32> {
  let ndc = vec2<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0);
  let a = inv_proj * vec4<f32>(ndc, 0.0, 1.0);
  let b = inv_proj * vec4<f32>(ndc, 1.0, 1.0);
  let p0 = a.xyz / a.w;
  let p1 = b.xyz / b.w;
  let t = (d + p0.z) / (p0.z - p1.z);
  return mix(p0, p1, t);
}

// Interleaved gradient noise (Jimenez 2014), 0..1: a fixed pattern over
// the pixels, blue-noise-like without a texture.
fn ign(px: vec2<f32>) -> f32 {
  return fract(52.9829189 * fract(dot(px, vec2<f32>(0.06711056, 0.00583715))));
}

// The view depth the prepass clears to where it drew nothing (post.rs
// FAR_DEPTH): the sky. A camera's far plane stays far under it.
const SKY_VIEW_DEPTH: f32 = 1.0e9;

// Whether a prepass view depth `d` is the sky's: past half the cleared
// value, so filtering and half-float rounding never decide it.
fn is_sky_depth(d: f32) -> bool {
  return d >= 0.5 * SKY_VIEW_DEPTH;
}

// Corner `i` (mod 6) of a unit quad drawn as two counter-clockwise
// triangles, (0,0) (1,0) (1,1) / (0,0) (1,1) (0,1): the quad every
// vertex-pulled sprite, card and billboard draws (2 * c - 1 for -1..1;
// .yx for the clockwise order in the xz plane).
fn quad_corner(i: u32) -> vec2<f32> {
  let k = i % 6u;
  return vec2<f32>(select(0.0, 1.0, k == 1u || k == 2u || k == 4u), select(0.0, 1.0, k == 2u || k == 4u || k == 5u));
}

// An orthonormal basis about the unit vector `n`: columns (t, b, n),
// right-handed, continuous everywhere but n.z = -1 (Duff et al. 2017,
// "Building an Orthonormal Basis, Revisited"). For anything whose answer
// does not depend on which way the tangents turn about `n`: a plane's
// span, a sampling frame, a fallback for a degenerate direction.
fn orthonormal_basis(n: vec3<f32>) -> mat3x3<f32> {
  let s = select(-1.0, 1.0, n.z >= 0.0);
  let a = -1.0 / (s + n.z);
  let b = n.x * n.y * a;
  let t = vec3<f32>(1.0 + s * n.x * n.x * a, s * b, -s * n.x);
  let bt = vec3<f32>(b, s + n.y * n.y * a, -n.y);
  return mat3x3<f32>(t, bt, n);
}

// The direction of cube face `face`'s point `uv` (NDC, -1..1): the IBL's
// cube maps and the probe grid's captures.
fn face_dir(face: u32, uv: vec2<f32>) -> vec3<f32> {
  // uv is the fragment's NDC position in [-1,1] (y up; texture row 0 is NDC
  // y = +1). Faces in wgpu's order +x,-x,+y,-y,+z,-z with the WebGPU/Vulkan
  // cube convention (sc, tc) = per-face axes, tc = -uv.y. (The spike flipped
  // v a second time, which mirrored the sky vertically; it never drew it.)
  let u = uv.x;
  let v = uv.y;
  switch face {
    case 0u: { return normalize(vec3<f32>(1.0, v, -u)); }
    case 1u: { return normalize(vec3<f32>(-1.0, v, u)); }
    case 2u: { return normalize(vec3<f32>(u, 1.0, -v)); }
    case 3u: { return normalize(vec3<f32>(u, -1.0, v)); }
    case 4u: { return normalize(vec3<f32>(u, v, 1.0)); }
    default: { return normalize(vec3<f32>(-u, v, -1.0)); }
  }
}
