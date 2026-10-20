// The shadow pass's slot (group 0), shared by the casters' shaders
// (shadow.wgsl, terrain_shadow.wgsl, scatter_shadow.wgsl): a cascade's
// view, a spot light's or a cube face's, or one hemisphere of a point
// light's dual-paraboloid map (shadows.rs). Selected with a dynamic offset.
struct ShadowPass {
  view_proj: mat4x4<f32>,
  light: vec4<f32>,       // xyz: light position, w: range (0: directional, write depth)
  time: vec4<f32>,        // x: the scene clock in seconds (the hooks' h.time)
  paraboloid: vec4<f32>,  // x: 1 for a paraboloid hemisphere, y: +1 the one below the light, -1 above, z: the map's side, w: the projection's scale
};
@group(0) @binding(0) var<uniform> shadow_pass: ShadowPass;

// How far behind its hemisphere (as a fraction of the distance) a point is
// still drawn: the map holds up to 96° from the axis.
const PARABOLOID_MARGIN: f32 = 0.1;

// An offset from the light in hemisphere `sigma`'s frame: its axis (straight
// down for +1, up for -1) along +z.
fn paraboloid_local(v: vec3<f32>, sigma: f32) -> vec3<f32> {
  return vec3<f32>(v.x, sigma * v.z, -sigma * v.y);
}

// The clip position of a world point in this pass. A paraboloid hemisphere
// is not a linear projection: each vertex is projected (the direction's xy
// over 1 + its z), and the depth is distance / range. Points far behind the
// hemisphere keep a finite position; the fragment stage drops them.
fn shadow_clip(world: vec4<f32>) -> vec4<f32> {
  if (shadow_pass.paraboloid.x < 0.5) { return shadow_pass.view_proj * world; }
  let l = paraboloid_local(world.xyz - shadow_pass.light.xyz, shadow_pass.paraboloid.y);
  let d = max(length(l), 1e-6);
  let q = l / d;
  let xy = q.xy * (shadow_pass.paraboloid.w / max(1.0 + q.z, 0.05));
  return vec4<f32>(xy, clamp(d / shadow_pass.light.w, 0.0, 1.0), 1.0);
}

// What a local light's pass stores: distance / range. In a paraboloid
// hemisphere a triangle's interpolated points are not where the curved
// projection puts them (its edges stay straight), so the distance is
// measured along the texel's own ray to the triangle's plane (`dx`, `dy`:
// the world position's screen derivatives, taken before any discard),
// which is exact for flat triangles of any size. Fragments behind the
// hemisphere are discarded.
fn local_distance(world: vec3<f32>, frag: vec4<f32>, dx: vec3<f32>, dy: vec3<f32>) -> f32 {
  let to = world - shadow_pass.light.xyz;
  var dist = length(to);
  if (shadow_pass.paraboloid.x > 0.5) {
    let sigma = shadow_pass.paraboloid.y;
    if (paraboloid_local(to, sigma).z < -PARABOLOID_MARGIN * dist) { discard; }
    let size = shadow_pass.paraboloid.z;
    let ndc = vec2<f32>(frag.x / size * 2.0 - 1.0, 1.0 - frag.y / size * 2.0) / shadow_pass.paraboloid.w;
    let r2 = dot(ndc, ndc);
    let q = vec3<f32>(2.0 * ndc, 1.0 - r2) / (1.0 + r2);
    let ray = vec3<f32>(q.x, -sigma * q.z, sigma * q.y);
    let nrm = cross(dx, dy);
    let len = length(nrm);
    let facing = dot(ray, nrm);
    if (len > 0.0 && abs(facing) > 1e-3 * len) {
      let t = dot(to, nrm) / facing;
      if (t > 0.5 * dist && t < 2.0 * dist) { dist = t; }
    }
  }
  return dist / shadow_pass.light.w;
}
