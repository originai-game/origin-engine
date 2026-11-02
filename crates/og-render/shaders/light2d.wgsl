// 2D lighting: the light map the lit sprite layers multiply with. Cleared to
// the ambient light; each light adds an instanced quad over its radius,
// `(1 - d/r)^falloff` (faded to zero over its last tenth), times its visibility from its row of the polar
// shadow map (light2d_shadow.wgsl): the fraction of a few rays about the
// pixel's angle that reach it, spread over the penumbra a light of radius
// `source` gives at the blocker's distance (PCSS), so penumbrae widen away
// from the occluder. A pixel inside the first occluder its ray meets is not
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
};

struct LightOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world: vec2<f32>,
  @location(1) light: vec4<f32>,
  @location(2) color_source: vec4<f32>,
  @location(3) flags: vec4<f32>,
  @location(4) direction: vec4<f32>,
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
  return o;
}

const TAU: f32 = 6.283185307179586;

// Whether the ray at column `c` (wrapped) of `row` reaches distance `d`:
// short of the first occluder, or inside it.
fn reaches(row: i32, c: i32, d: f32) -> f32 {
  let n = i32(lights2d.info.x);
  let v = textureLoad(shadow_map, vec2<i32>(((c % n) + n) % n, row), 0).xy;
  let bias = 0.002 * d;
  return select(0.0, 1.0, d <= v.x + bias || d <= v.y);
}

// The same at a fractional column, between its two texels.
fn reaches_at(row: i32, x: f32, d: f32) -> f32 {
  let c = floor(x - 0.5);
  let t = x - 0.5 - c;
  return mix(reaches(row, i32(c), d), reaches(row, i32(c) + 1, d), t);
}

fn blocker(row: i32, c: i32) -> f32 {
  let n = i32(lights2d.info.x);
  return textureLoad(shadow_map, vec2<i32>(((c % n) + n) % n, row), 0).x;
}

// The fraction of the light of radius `source` at angle `theta` from the
// pixel at distance `d` that reaches it.
fn visibility(row: i32, theta: f32, d: f32, source: f32) -> f32 {
  let n = lights2d.info.x;
  let per_radian = n / TAU;
  let x = fract(theta / TAU) * n;
  // Blockers: the occluders nearer than the pixel within the light's
  // angular size as seen from there.
  let search = clamp(source / d * per_radian, 0.5, n / 8.0);
  var sum = 0.0;
  var found = 0.0;
  for (var k = 0; k < 8; k++) {
    let b = blocker(row, i32(floor(x + (f32(k) / 7.0 * 2.0 - 1.0) * search)));
    if (b < d) {
      sum += b;
      found += 1.0;
    }
  }
  if (found == 0.0) { return reaches_at(row, x, d); }
  let db = max(sum / found, 1e-3);
  // The penumbra's half-angle at the pixel (PCSS), at least half a texel.
  let half = clamp(source * (d - db) / (db * d) * per_radian, 0.5, n / 8.0);
  var lit = 0.0;
  for (var k = 0; k < 9; k++) {
    lit += reaches_at(row, x + (f32(k) / 8.0 * 2.0 - 1.0) * half, d);
  }
  return lit / 9.0;
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
    atten *= visibility(i32(in.direction.w), theta, dist, max(in.color_source.w, 1e-3));
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
