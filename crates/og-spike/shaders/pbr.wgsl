// Origin Engine spike: forward PBR (metallic-roughness), one directional light with
// PCF shadows, split-sum IBL. Linear HDR output; the post pass tonemaps.

struct Camera {
  view_proj: mat4x4<f32>,
  shadow_view_proj: mat4x4<f32>,
  view_pos: vec4<f32>,
  light_dir: vec4<f32>,   // xyz: direction towards the light, w: unused
  light_color: vec4<f32>, // rgb: radiance, w: shadow texel size
  params: vec4<f32>,      // x: ibl intensity, y: prefiltered mip count - 1, z: time, w: unused
};

struct MaterialFactors {
  base_color: vec4<f32>,
  mr: vec4<f32>,          // x: metallic, y: roughness, z: normal scale, w: has_normal_map
  emissive: vec4<f32>,    // rgb: emissive, w: alpha cutoff (<0 = opaque)
};

@group(0) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(1) var shadow_map: texture_2d<f32>;
@group(0) @binding(2) var shadow_sampler: sampler;
@group(0) @binding(3) var irradiance_map: texture_cube<f32>;
@group(0) @binding(4) var prefiltered_map: texture_cube<f32>;
@group(0) @binding(5) var brdf_lut: texture_2d<f32>;
@group(0) @binding(6) var ibl_sampler: sampler;

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
  @location(4) shadow_pos: vec3<f32>,
};

@vertex
fn vs(v: VertexIn, i: InstanceIn) -> VertexOut {
  let model = mat4x4<f32>(i.m0, i.m1, i.m2, i.m3);
  let world = model * vec4<f32>(v.position, 1.0);
  // Normal through the model's upper 3x3; instances carry uniform scale, so no inverse-transpose.
  let n = normalize((model * vec4<f32>(v.normal, 0.0)).xyz);
  var o: VertexOut;
  o.clip = camera.view_proj * world;
  o.world_pos = world.xyz;
  o.normal = n;
  o.uv = v.uv;
  o.color = v.color;
  let sp = camera.shadow_view_proj * world;
  // NDC to texture space; wgpu NDC y is up, texture y is down, depth is 0..1.
  o.shadow_pos = vec3<f32>(sp.x * 0.5 + 0.5, 0.5 - sp.y * 0.5, sp.z);
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

fn shadow_factor(sp: vec3<f32>, n_dot_l: f32) -> f32 {
  if (sp.x < 0.0 || sp.x > 1.0 || sp.y < 0.0 || sp.y > 1.0 || sp.z > 1.0) {
    return 1.0;
  }
  // 3x3 PCF with explicit compares (ref <= stored means lit): the same result on
  // every backend regardless of how a comparison sampler orders its operands.
  let bias = max(0.0025 * (1.0 - n_dot_l), 0.0006);
  let ref_depth = sp.z - bias;
  let size = vec2<i32>(textureDimensions(shadow_map));
  let base = vec2<i32>(sp.xy * vec2<f32>(size));
  var sum = 0.0;
  for (var y = -1; y <= 1; y++) {
    for (var x = -1; x <= 1; x++) {
      let at = clamp(base + vec2<i32>(x, y), vec2<i32>(0), size - 1);
      let stored = textureLoad(shadow_map, at, 0).r;
      sum += select(0.0, 1.0, ref_depth <= stored);
    }
  }
  return sum / 9.0;
}

@fragment
fn fs(in: VertexOut) -> @location(0) vec4<f32> {
  let base_sample = textureSample(base_color_tex, material_sampler, in.uv);
  let base = material.base_color * base_sample * in.color;
  if (material.emissive.w >= 0.0 && base.a < material.emissive.w) {
    discard;
  }
  let mr_sample = textureSample(mr_tex, material_sampler, in.uv);
  let metallic = clamp(material.mr.x * mr_sample.b, 0.0, 1.0);
  let roughness = clamp(material.mr.y * mr_sample.g, 0.04, 1.0);

  var n = normalize(in.normal);
  let v = normalize(camera.view_pos.xyz - in.world_pos);
  if (dot(n, v) < 0.0) { n = -n; }
  if (material.mr.w > 0.5) {
    var map = textureSample(normal_tex, material_sampler, in.uv).xyz * 2.0 - 1.0;
    map = vec3<f32>(map.xy * material.mr.z, map.z);
    n = perturb_normal(n, in.world_pos, in.uv, map);
  }

  let l = normalize(camera.light_dir.xyz);
  let h = normalize(v + l);
  let n_dot_l = max(dot(n, l), 0.0);
  let n_dot_v = max(dot(n, v), 1e-4);
  let n_dot_h = max(dot(n, h), 0.0);
  let h_dot_v = max(dot(h, v), 0.0);

  let albedo = base.rgb;
  let f0 = mix(vec3<f32>(0.04), albedo, metallic);
  let a = roughness * roughness;

  // Direct light.
  let d = d_ggx(n_dot_h, a);
  let g = g_smith(n_dot_v, n_dot_l, roughness);
  let f = f_schlick(h_dot_v, f0);
  let spec = (d * g * f) / (4.0 * n_dot_v * n_dot_l + 1e-4);
  let kd = (vec3<f32>(1.0) - f) * (1.0 - metallic);
  let shadow = shadow_factor(in.shadow_pos, n_dot_l);
  let direct = (kd * albedo / PI + spec) * camera.light_color.rgb * n_dot_l * shadow;

  // Image-based light (split sum).
  let f_ibl = f_schlick_roughness(n_dot_v, f0, roughness);
  let kd_ibl = (vec3<f32>(1.0) - f_ibl) * (1.0 - metallic);
  let irradiance = textureSample(irradiance_map, ibl_sampler, n).rgb;
  let diffuse_ibl = irradiance * albedo;
  let r = reflect(-v, n);
  let mip = roughness * camera.params.y;
  let prefiltered = textureSampleLevel(prefiltered_map, ibl_sampler, r, mip).rgb;
  let lut = textureSample(brdf_lut, ibl_sampler, vec2<f32>(n_dot_v, roughness)).rg;
  let specular_ibl = prefiltered * (f_ibl * lut.x + lut.y);
  let ambient = (kd_ibl * diffuse_ibl + specular_ibl) * camera.params.x;

  let emissive = material.emissive.rgb * textureSample(emissive_tex, material_sampler, in.uv).rgb;
  // Debug views (camera.params.w): 1 = shadow factor, 2 = shadow-space xy, 3 = normals.
  if (camera.params.w > 5.5) {
    let stored = textureLoad(shadow_map, vec2<i32>(in.shadow_pos.xy * vec2<f32>(textureDimensions(shadow_map))), 0).r;
    return vec4<f32>(vec3<f32>(select(0.0, 1.0, in.shadow_pos.z - 0.001 <= stored)), 1.0);
  }
  if (camera.params.w > 4.5) { return vec4<f32>(vec3<f32>(in.shadow_pos.z), 1.0); }
  if (camera.params.w > 3.5) { return vec4<f32>(vec3<f32>(textureLoad(shadow_map, vec2<i32>(in.shadow_pos.xy * vec2<f32>(textureDimensions(shadow_map))), 0).r), 1.0); }
  if (camera.params.w > 2.5) { return vec4<f32>(n * 0.5 + 0.5, 1.0); }
  if (camera.params.w > 1.5) { return vec4<f32>(in.shadow_pos.xy, in.shadow_pos.z, 1.0); }
  if (camera.params.w > 0.5) { return vec4<f32>(vec3<f32>(shadow), 1.0); }
  return vec4<f32>(direct + ambient + emissive, 1.0);
}
