// 2D lighting: the light map the lit sprite layers multiply with. Cleared to
// the ambient light; each light adds an instanced quad over its radius,
// `(1 - d/r)^falloff`, times its visibility past the occluders: for each
// rotated-rectangle occluder between the pixel and the light, the fraction
// of the light's disc (radius `source`) the rectangle hides as seen from
// the pixel, so penumbrae widen away from the occluder.

struct Lights2d {
  view_proj: mat4x4<f32>,
  info: vec4<f32>,                 // x: occluders
  occluders: array<vec4<f32>, 256>,  // per occluder: (center, half size), (cos, sin, 0, 0)
};
@group(0) @binding(0) var<uniform> lights2d: Lights2d;

struct LightIn {
  @location(0) pos_radius: vec4<f32>,   // xy: position, z: radius, w: falloff
  @location(1) color_source: vec4<f32>, // rgb: colour x intensity, w: source radius
  @location(2) flags: vec4<f32>,        // x: casts shadows
};

struct LightOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world: vec2<f32>,
  @location(1) light: vec4<f32>,
  @location(2) color_source: vec4<f32>,
  @location(3) flags: vec4<f32>,
};

@vertex
fn vs(@builtin(vertex_index) vi: u32, l: LightIn) -> LightOut {
  var corners = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0),
    vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, 1.0), vec2<f32>(-1.0, 1.0));
  let p = l.pos_radius.xy + corners[vi] * l.pos_radius.z;
  var o: LightOut;
  o.clip = lights2d.view_proj * vec4<f32>(p, 0.0, 1.0);
  o.world = p;
  o.light = l.pos_radius;
  o.color_source = l.color_source;
  o.flags = l.flags;
  return o;
}

fn cross2(a: vec2<f32>, b: vec2<f32>) -> f32 {
  return a.x * b.y - a.y * b.x;
}

fn angle_from(base: vec2<f32>, v: vec2<f32>) -> f32 {
  return atan2(cross2(base, v), dot(base, v));
}

@fragment
fn fs(in: LightOut) -> @location(0) vec4<f32> {
  let to_light = in.light.xy - in.world;
  let dist = length(to_light);
  let r = in.light.z;
  if (dist >= r) { discard; }
  var atten = pow(1.0 - dist / r, max(in.light.w, 0.01));
  if (in.flags.x > 0.5 && dist > 1e-4) {
    let alpha = atan(max(in.color_source.w, 1e-3) / dist);
    let count = u32(lights2d.info.x);
    for (var i = 0u; i < count; i++) {
      let a = lights2d.occluders[i * 2u];
      let b = lights2d.occluders[i * 2u + 1u];
      let center = a.xy;
      let half = a.zw;
      let cs = b.x;
      let sn = b.y;
      // Into the occluder's frame.
      let dp = in.world - center;
      let q = vec2<f32>(dp.x * cs + dp.y * sn, -dp.x * sn + dp.y * cs);
      if (all(abs(q) <= half)) { continue; }
      let dl = in.light.xy - center;
      let ql = vec2<f32>(dl.x * cs + dl.y * sn, -dl.x * sn + dl.y * cs);
      if (all(abs(ql) <= half)) { continue; }
      let nearest = length(max(abs(q) - half, vec2<f32>(0.0)));
      if (nearest >= dist) { continue; }
      // The rectangle's angular span from the pixel, around its centre's direction.
      let base = normalize(center - in.world);
      var lo = 1e9;
      var hi = -1e9;
      for (var k = 0u; k < 4u; k++) {
        let s = vec2<f32>(select(-1.0, 1.0, (k & 1u) != 0u), select(-1.0, 1.0, (k & 2u) != 0u)) * half;
        let corner = center + vec2<f32>(s.x * cs - s.y * sn, s.x * sn + s.y * cs);
        let ang = angle_from(base, corner - in.world);
        lo = min(lo, ang);
        hi = max(hi, ang);
      }
      let al = angle_from(base, to_light);
      let covered = clamp(min(hi, al + alpha) - max(lo, al - alpha), 0.0, 2.0 * alpha);
      atten *= 1.0 - covered / (2.0 * alpha);
      if (atten <= 0.0) { break; }
    }
  }
  return vec4<f32>(in.color_source.rgb * atten, 0.0);
}
