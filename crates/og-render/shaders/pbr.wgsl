// Forward PBR (metallic-roughness): the directional light with 3 cascaded
// shadow maps, up to 8 point/spot lights (the first two shadowed) in a
// per-fragment loop, split-sum IBL. Linear HDR output; the post passes
// tonemap. Prepended by frame.wgsl.

struct MaterialFactors {
  base_color: vec4<f32>,
  mr: vec4<f32>,          // x: metallic, y: roughness, z: normal scale, w: has_normal_map
  emissive: vec4<f32>,    // rgb: emissive, w: alpha cutoff (<0 = opaque)
};

@group(1) @binding(0) var<uniform> material: MaterialFactors;
@group(1) @binding(1) var base_color_tex: texture_2d<f32>;
@group(1) @binding(2) var mr_tex: texture_2d<f32>;
@group(1) @binding(3) var normal_tex: texture_2d<f32>;
@group(1) @binding(4) var emissive_tex: texture_2d<f32>;
@group(1) @binding(5) var material_sampler: sampler;

struct VertexIn {
  @location(0) position: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
  @location(3) color: vec4<f32>,
};

struct InstanceIn {
  @location(4) m0: vec4<f32>,
  @location(5) m1: vec4<f32>,
  @location(6) m2: vec4<f32>,
  @location(7) m3: vec4<f32>,
};

struct VertexOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) world_pos: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
  @location(3) color: vec4<f32>,
};

@vertex
fn vs(v: VertexIn, i: InstanceIn) -> VertexOut {
  let model = mat4x4<f32>(i.m0, i.m1, i.m2, i.m3);
  let world = model * vec4<f32>(v.position, 1.0);
  // Normal through the model's upper 3x3; instances carry uniform scale, so no inverse-transpose.
  let n = normalize((model * vec4<f32>(v.normal, 0.0)).xyz);
  var o: VertexOut;
  o.clip = frame.view_proj * world;
  o.world_pos = world.xyz;
  o.normal = n;
  o.uv = v.uv;
  o.color = v.color;
  return o;
}

const PI: f32 = 3.14159265359;

fn d_ggx(n_dot_h: f32, a: f32) -> f32 {
  let a2 = a * a;
  let d = n_dot_h * n_dot_h * (a2 - 1.0) + 1.0;
  return a2 / (PI * d * d + 1e-7);
}

fn g_smith(n_dot_v: f32, n_dot_l: f32, roughness: f32) -> f32 {
  let r = roughness + 1.0;
  let k = (r * r) / 8.0;
  let gv = n_dot_v / (n_dot_v * (1.0 - k) + k);
  let gl = n_dot_l / (n_dot_l * (1.0 - k) + k);
  return gv * gl;
}

fn f_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
  return f0 + (1.0 - f0) * pow(1.0 - cos_theta, 5.0);
}

fn f_schlick_roughness(cos_theta: f32, f0: vec3<f32>, roughness: f32) -> vec3<f32> {
  return f0 + (max(vec3<f32>(1.0 - roughness), f0) - f0) * pow(1.0 - cos_theta, 5.0);
}

// Cotangent frame from derivatives (no vertex tangents needed).
fn perturb_normal(n: vec3<f32>, p: vec3<f32>, uv: vec2<f32>, map: vec3<f32>) -> vec3<f32> {
  let dp1 = dpdx(p);
  let dp2 = dpdy(p);
  let duv1 = dpdx(uv);
  let duv2 = dpdy(uv);
  let dp2perp = cross(dp2, n);
  let dp1perp = cross(n, dp1);
  let t = dp2perp * duv1.x + dp1perp * duv2.x;
  let b = dp2perp * duv1.y + dp1perp * duv2.y;
  let invmax = inverseSqrt(max(dot(t, t), dot(b, b)) + 1e-12);
  let tbn = mat3x3<f32>(t * invmax, b * invmax, n);
  return normalize(tbn * map);
}

// Bilinear PCF over one layer of an R32Float array: the four texels around
// the sample compared explicitly (ref <= stored means lit) and blended by the
// sample's position between them, as a hardware comparison sampler with
// linear filtering would, so edges ramp over one texel instead of stepping.
// textureLoad and explicit compares give the same result on every backend
// (WebGL2 can neither filter a depth texture nor gather).
fn pcf(map: texture_2d_array<f32>, uv: vec2<f32>, layer: u32, ref_depth: f32) -> f32 {
  let size = vec2<i32>(textureDimensions(map));
  let p = uv * vec2<f32>(size) - 0.5;
  let base = vec2<i32>(floor(p));
  let f = fract(p);
  var lit: array<f32, 4>;
  for (var i = 0; i < 4; i++) {
    let at = clamp(base + vec2<i32>(i & 1, i >> 1u), vec2<i32>(0), size - 1);
    lit[i] = select(0.0, 1.0, ref_depth <= textureLoad(map, at, i32(layer), 0).r);
  }
  return mix(mix(lit[0], lit[1], f.x), mix(lit[2], lit[3], f.x), f.y);
}

// The cascade whose camera slice holds the point (by view depth, the same
// slices its casters were culled to); CASCADES beyond the last one.
fn cascade_index(p: vec3<f32>) -> u32 {
  let count = u32(frame.light_dir.w);
  let depth = dot(p - frame.view_pos.xyz, frame.view_forward.xyz);
  for (var c = 0u; c < count; c++) {
    if (depth <= frame.cascade_bias[c].z) { return c; }
  }
  return CASCADES;
}

// The bias is in world units scaled by the cascade's texel size (a normal
// offset growing with the light's angle to the surface, plus a depth bias
// converted to the cascade's normalized depth), so it holds at any scene
// scale. Shadows fade out over the last tenth of the shadow distance.
fn cascade_shadow(p: vec3<f32>, n: vec3<f32>, n_dot_l: f32) -> f32 {
  let c = cascade_index(p);
  if (c >= CASCADES) { return 1.0; }
  let bias = frame.cascade_bias[c];
  let sin_l = sqrt(max(1.0 - n_dot_l * n_dot_l, 0.0));
  let offset = p + n * bias.x * (0.25 + sin_l);
  let sp = frame.cascade_view_proj[c] * vec4<f32>(offset, 1.0);
  let uv = vec2<f32>(sp.x * 0.5 + 0.5, 0.5 - sp.y * 0.5);
  if (sp.z > 1.0) { return 1.0; }
  let shadow = pcf(cascade_map, uv, c, sp.z - bias.y);
  let last = frame.cascade_bias[u32(frame.light_dir.w) - 1u].z;
  let depth = dot(p - frame.view_pos.xyz, frame.view_forward.xyz);
  return mix(shadow, 1.0, smoothstep(0.9 * last, last, depth));
}

// A point light's cube face (+x, -x, +y, -y, +z, -z) for a direction from it.
fn cube_face(d: vec3<f32>) -> u32 {
  let a = abs(d);
  if (a.x >= a.y && a.x >= a.z) { return select(1u, 0u, d.x > 0.0); }
  if (a.y >= a.z) { return select(3u, 2u, d.y > 0.0); }
  return select(5u, 4u, d.z > 0.0);
}

// Local shadows store distance / range; the fragment compares its own.
fn local_shadow(light: LocalLight, p: vec3<f32>, n: vec3<f32>) -> f32 {
  let slot = i32(light.direction_slot.w);
  if (slot < 0) { return 1.0; }
  let range = light.position_range.w;
  let to_frag = p - light.position_range.xyz;
  let dist = length(to_frag);
  // Normal offset grows with distance (texels widen with it).
  let offset = p + n * light.cone.z * dist;
  var layer = u32(slot) * 6u;
  if (light.color_kind.w < 0.5) { layer += cube_face(to_frag); }
  let clip = frame.local_view_proj[layer] * vec4<f32>(offset, 1.0);
  if (clip.w <= 0.0) { return 1.0; }
  let ndc = clip.xy / clip.w;
  let uv = vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
  if (light.color_kind.w > 0.5 && (any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0)))) { return 1.0; }
  let ref_depth = length(offset - light.position_range.xyz) / range - light.cone.w;
  return pcf(local_shadow_map, clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0)), layer, ref_depth);
}

struct Surface {
  albedo: vec3<f32>,
  metallic: f32,
  roughness: f32,
  f0: vec3<f32>,
};

fn brdf(s: Surface, n: vec3<f32>, v: vec3<f32>, l: vec3<f32>) -> vec3<f32> {
  let h = normalize(v + l);
  let n_dot_l = max(dot(n, l), 0.0);
  let n_dot_v = max(dot(n, v), 1e-4);
  let n_dot_h = max(dot(n, h), 0.0);
  let h_dot_v = max(dot(h, v), 0.0);
  let a = s.roughness * s.roughness;
  let d = d_ggx(n_dot_h, a);
  let g = g_smith(n_dot_v, n_dot_l, s.roughness);
  let f = f_schlick(h_dot_v, s.f0);
  let spec = (d * g * f) / (4.0 * n_dot_v * n_dot_l + 1e-4);
  let kd = (vec3<f32>(1.0) - f) * (1.0 - s.metallic);
  return (kd * s.albedo / PI + spec) * n_dot_l;
}

fn local_light(light: LocalLight, s: Surface, p: vec3<f32>, n: vec3<f32>, v: vec3<f32>) -> vec3<f32> {
  let to_light = light.position_range.xyz - p;
  let d2 = max(dot(to_light, to_light), 1e-4);
  let range = light.position_range.w;
  let ratio = d2 / (range * range);
  // Inverse square, windowed to zero at the range (UE4's falloff).
  let window = clamp(1.0 - ratio * ratio, 0.0, 1.0);
  var atten = window * window / d2;
  let l = to_light * inverseSqrt(d2);
  if (light.color_kind.w > 0.5) {
    let cos_angle = dot(-l, light.direction_slot.xyz);
    let t = clamp((cos_angle - light.cone.x) * light.cone.y, 0.0, 1.0);
    atten *= t * t;
  }
  if (atten <= 0.0) { return vec3<f32>(0.0); }
  if (dot(n, l) <= 0.0) { return vec3<f32>(0.0); }
  let shadow = local_shadow(light, p, n);
  return brdf(s, n, v, l) * light.color_kind.rgb * atten * shadow;
}

@fragment
fn fs(in: VertexOut) -> @location(0) vec4<f32> {
  let base_sample = textureSample(base_color_tex, material_sampler, in.uv);
  let base = material.base_color * base_sample * in.color;
  if (material.emissive.w >= 0.0 && base.a < material.emissive.w) {
    discard;
  }
  let mr_sample = textureSample(mr_tex, material_sampler, in.uv);
  var s: Surface;
  s.metallic = clamp(material.mr.x * mr_sample.b, 0.0, 1.0);
  s.roughness = clamp(material.mr.y * mr_sample.g, 0.04, 1.0);
  s.albedo = base.rgb;
  s.f0 = mix(vec3<f32>(0.04), s.albedo, s.metallic);

  var n = normalize(in.normal);
  // Perspective: towards the eye; orthographic: against the view direction.
  let v = select(normalize(frame.view_pos.xyz - in.world_pos), -frame.view_forward.xyz, frame.view_pos.w > 0.5);
  if (dot(n, v) < 0.0) { n = -n; }
  if (material.mr.w > 0.5) {
    var map = textureSample(normal_tex, material_sampler, in.uv).xyz * 2.0 - 1.0;
    map = vec3<f32>(map.xy * material.mr.z, map.z);
    n = perturb_normal(n, in.world_pos, in.uv, map);
  }

  // The directional light.
  let l = normalize(frame.light_dir.xyz);
  let n_dot_l = max(dot(n, l), 0.0);
  var shadow = 1.0;
  if (frame.light_dir.w > 0.5 && n_dot_l > 0.0) { shadow = cascade_shadow(in.world_pos, normalize(in.normal), n_dot_l); }
  var color = brdf(s, n, v, l) * frame.light_color.rgb * shadow;

  // Point and spot lights.
  let light_count = u32(frame.params.z);
  for (var i = 0u; i < light_count; i++) {
    color += local_light(frame.lights[i], s, in.world_pos, n, v);
  }

  // Image-based light (split sum).
  let n_dot_v = max(dot(n, v), 1e-4);
  let f_ibl = f_schlick_roughness(n_dot_v, s.f0, s.roughness);
  let kd_ibl = (vec3<f32>(1.0) - f_ibl) * (1.0 - s.metallic);
  let irradiance = textureSample(irradiance_map, ibl_sampler, n).rgb;
  let diffuse_ibl = irradiance * s.albedo;
  let r = reflect(-v, n);
  let mip = s.roughness * frame.params.y;
  let prefiltered = textureSampleLevel(prefiltered_map, ibl_sampler, r, mip).rgb;
  let lut = textureSample(brdf_lut, ibl_sampler, vec2<f32>(n_dot_v, s.roughness)).rg;
  let specular_ibl = prefiltered * (f_ibl * lut.x + lut.y);
  color += (kd_ibl * diffuse_ibl + specular_ibl) * frame.params.x;

  color += material.emissive.rgb * textureSample(emissive_tex, material_sampler, in.uv).rgb;

  if (frame.light_color.w > 0.5) {
    // Debug: the cascade covering this pixel.
    let c = cascade_index(in.world_pos);
    var tint = vec3<f32>(0.0);
    if (c == 0u) { tint = vec3<f32>(1.0, 0.0, 0.0); }
    if (c == 1u) { tint = vec3<f32>(0.0, 1.0, 0.0); }
    if (c == 2u) { tint = vec3<f32>(0.0, 0.0, 1.0); }
    return vec4<f32>(tint * 4.0, 1.0);
  }
  return vec4<f32>(color, 1.0);
}
