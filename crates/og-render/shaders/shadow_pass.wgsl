// The shadow pass's slot (group 0) and what a caster does with it, shared
// by every caster's shader (shadow.wgsl, custom_shadow.wgsl,
// terrain_shadow.wgsl, scatter_shadow.wgsl, impostor_shadow.wgsl,
// crowd_card_shadow.wgsl): a caster projects with `shadow_clip` or
// `shadow_project` and ends its fragment stage in `shadow_depth` or
// `shadow_card`. (The grass casts into cascades only and writes its
// depth, grass_shadow.wgsl.) The slot is a cascade's
// view, a spot light's or a cube face's, or one hemisphere of a point
// light's dual-paraboloid map (shadows.rs). Selected with a dynamic offset.
struct ShadowPass {
  view_proj: mat4x4<f32>,
  light: vec4<f32>,       // xyz: light position, w: range (0: directional, write depth)
  time: vec4<f32>,        // x: the scene clock in seconds (the hooks' h.time), yzw: the scene's origin (h.origin)
  paraboloid: vec4<f32>,  // x: 0 flat, +1 paraboloid below light, -1 above; y: dither turn (turn % 64); z: map side; w: projection scale
  foliage_wind: array<vec4f, 9>, // current deformation only; FoliageWind
};
@group(0) @binding(0) var<uniform> shadow_pass: ShadowPass;

fn shadow_dither_turn() -> u32 {
  return u32(shadow_pass.paraboloid.y);
}

// A caster draws no motion: `previous_shader_params()` (hooks.rs) reads the current parameters (frame.wgsl's
// `params_moved` in the modules that draw motion).
fn params_moved(stamp: u32) -> bool {
  return false;
}

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
  return shadow_project(world, shadow_pass.view_proj * world);
}

// `shadow_clip` given the point through the pass's view projection,
// `flat` (a point an instance's matrix moved is projected with
// `mul_point`, instance.wgsl).
fn shadow_project(world: vec4<f32>, flat: vec4<f32>) -> vec4<f32> {
  if (abs(shadow_pass.paraboloid.x) < 0.5) { return flat; }
  let l = paraboloid_local(world.xyz - shadow_pass.light.xyz, shadow_pass.paraboloid.x);
  let d = max(length(l), 1e-6);
  let q = l / d;
  let xy = vec2<f32>(-q.x, q.y) * (shadow_pass.paraboloid.w / max(1.0 + q.z, 0.05));
  return vec4<f32>(xy, clamp(d / shadow_pass.light.w, 0.0, 1.0), 1.0);
}

// Whether `world` lies behind the active paraboloid hemisphere and should be
// cut. Folded into each shadow fragment entry's single discard for Metal.
fn shadow_paraboloid_cut(world: vec3<f32>) -> bool {
  if (shadow_pass.light.w <= 0.0 || abs(shadow_pass.paraboloid.x) <= 0.5) { return false; }
  let to = world - shadow_pass.light.xyz;
  let dist = length(to);
  return paraboloid_local(to, shadow_pass.paraboloid.x).z < -PARABOLOID_MARGIN * dist;
}

struct ShadowGrad {
  dx: vec3<f32>,
  dy: vec3<f32>,
};

// Compute screen-space world derivatives only for dual-paraboloid local point
// lights (uniform branch), avoiding 6 fine derivative instructions per pixel
// across directional cascades and perspective spot/cube shadow passes.
fn shadow_world_grad(world: vec3<f32>) -> ShadowGrad {
  if (abs(shadow_pass.paraboloid.x) > 0.5) {
    return ShadowGrad(dpdxFine(world), dpdyFine(world));
  }
  return ShadowGrad(vec3<f32>(0.0), vec3<f32>(0.0));
}

// What a local light's pass stores: distance / range. In a paraboloid
// hemisphere a triangle's interpolated points are not where the curved
// projection puts them (its edges stay straight), so the distance is
// measured along the texel's own ray to the triangle's plane (`dx`, `dy`:
// the world position's screen derivatives, taken before any discard),
// which is exact for flat triangles of any size.
fn local_distance(world: vec3<f32>, frag: vec4<f32>, dx: vec3<f32>, dy: vec3<f32>) -> f32 {
  let to = world - shadow_pass.light.xyz;
  var dist = length(to);
  if (abs(shadow_pass.paraboloid.x) > 0.5) {
    let sigma = shadow_pass.paraboloid.x;
    let size = shadow_pass.paraboloid.z;
    let ndc = vec2<f32>(1.0 - frag.x / size * 2.0, 1.0 - frag.y / size * 2.0) / shadow_pass.paraboloid.w;
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

// What a local light's caster fragment stores (shadow_target.rs): its radial
// distance over range in red (back faces projected in a single pass, matching
// cascades).
fn local_value(world: vec3<f32>, frag: vec4<f32>, dx: vec3<f32>, dy: vec3<f32>) -> vec4<f32> {
  return vec4<f32>(local_distance(world, frag, dx, dy), 0.0, 0.0, 1.0);
}

// What a caster's fragment at `world` stores (`frag`: its position
// builtin, `front`: its front_facing; `dx`, `dy`: the world position's
// derivatives, taken before any discard), folding `cut` and the paraboloid
// hemisphere clip into a single discard.
fn shadow_depth_cut(world: vec3<f32>, frag: vec4<f32>, dx: vec3<f32>, dy: vec3<f32>, front: bool, cut: bool) -> vec4<f32> {
  if (cut || shadow_paraboloid_cut(world)) { discard; }
  if (shadow_pass.light.w > 0.0) {
    return local_value(world, frag, dx, dy);
  }
  return vec4<f32>(frag.z, 0.0, 0.0, 1.0);
}

fn shadow_depth(world: vec3<f32>, frag: vec4<f32>, dx: vec3<f32>, dy: vec3<f32>, front: bool) -> vec4<f32> {
  return shadow_depth_cut(world, frag, dx, dy, front, false);
}

// A card's fragment, which stands for a surface off its own plane: the
// depth it writes and what it stores.
struct CardShadow {
  depth: f32,
  value: vec4<f32>,
};

// `shadow_depth_cut` for a card: `surface` is the point the fragment stands
// for and `flat` that point through the pass's view projection (as
// `shadow_project` takes it).
fn shadow_card_cut(surface: vec3<f32>, flat: vec4<f32>, frag: vec4<f32>, dx: vec3<f32>, dy: vec3<f32>, front: bool, cut: bool) -> CardShadow {
  if (cut || shadow_paraboloid_cut(surface)) { discard; }
  var o: CardShadow;
  if (abs(shadow_pass.paraboloid.x) > 0.5) {
    o.depth = clamp(length(surface - shadow_pass.light.xyz) / shadow_pass.light.w, 0.0, 1.0);
  } else {
    o.depth = clamp(flat.z / flat.w, 0.0, 1.0);
  }
  if (shadow_pass.light.w > 0.0) {
    o.value = local_value(surface, frag, dx, dy);
  } else {
    o.value = vec4<f32>(o.depth, 0.0, 0.0, 1.0);
  }
  return o;
}

fn shadow_card(surface: vec3<f32>, flat: vec4<f32>, frag: vec4<f32>, dx: vec3<f32>, dy: vec3<f32>, front: bool) -> CardShadow {
  return shadow_card_cut(surface, flat, frag, dx, dy, front, false);
}
