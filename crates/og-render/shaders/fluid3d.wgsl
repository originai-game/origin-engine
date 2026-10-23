// 3D particle fluids drawn as a screen-space surface (og-physics'
// og.Fluid3D, fluid3d.rs): the particles, uploaded by the CPU each frame
// as instances, are drawn as sphere imposters twice, once for the nearest
// view depth (a depth test, its own target) and once added up for the
// thickness and colour (each sphere's chord through the pixel, weighted
// by the fluid's optical density). fluid3d_blur.wgsl smooths the depth
// with a separable bilateral filter, and the composite, drawn in the main
// pass after the scene is copied, rebuilds the surface from it: normals
// from the smoothed depth, the copied scene refracted through it and
// absorbed by its thickness, the sun (with its shadows), the image-based
// light and the clustered local lights scattered by its colour, the
// environment reflected by Schlick's Fresnel, and fog. It writes its
// depth, so what is drawn after it sorts against it. The same code on
// every tier: no storage buffer, no compute. Prepended by frame.wgsl,
// lighting.wgsl and the light tables (clusters.rs).

struct FluidUniform {
  right: vec4<f32>,    // xyz: the camera's right, w: pixels per metre at view depth 1 (perspective) or per metre (orthographic)
  up: vec4<f32>,       // xyz: the camera's up, w: reflectivity
  screen: vec4<f32>,   // w, h, 1/w, 1/h
  look: vec4<f32>,     // x: refraction, y: 1 orthographic, zw: unused
};

@group(1) @binding(0) var<uniform> fluid: FluidUniform;
@group(1) @binding(1) var fluid_depth: texture_2d<f32>;   // the smoothed view depth (FAR_DEPTH: none)
@group(1) @binding(2) var fluid_thick: texture_2d<f32>;   // rgb: colour x density, a: optical depth
@group(1) @binding(3) var scene_color: texture_2d<f32>;   // the scene drawn so far
@group(1) @binding(4) var fluid_sampler: sampler;
@group(1) @binding(5) var scene_depth: texture_2d<f32>;   // the prepass's view depth

const NONE: f32 = 1.0e8;

struct SphereOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) corner: vec2<f32>,
  @location(1) center: vec3<f32>,
  @location(2) radius: f32,
  @location(3) color_k: vec4<f32>,
};

@vertex
fn vs_sphere(@builtin(vertex_index) vi: u32, @location(0) center_radius: vec4<f32>, @location(1) color_k: vec4<f32>) -> SphereOut {
  var corners = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0),
    vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, 1.0), vec2<f32>(-1.0, 1.0));
  let c = corners[vi];
  let r = center_radius.w;
  let world = center_radius.xyz + (fluid.right.xyz * c.x + fluid.up.xyz * c.y) * r;
  var o: SphereOut;
  o.clip = frame.view_proj * vec4<f32>(world, 1.0);
  o.corner = c;
  o.center = center_radius.xyz;
  o.radius = r;
  o.color_k = color_k;
  return o;
}

fn view_depth(p: vec3<f32>) -> f32 {
  return dot(p - frame.view_pos.xyz, frame.view_forward.xyz);
}

struct DepthOut {
  @location(0) depth: vec4<f32>,
  @builtin(frag_depth) frag: f32,
};

// The nearest point of the spheres: its view depth, tested in depth.
@fragment
fn fs_depth(in: SphereOut) -> DepthOut {
  let rr = dot(in.corner, in.corner);
  if (rr > 1.0) { discard; }
  let z = sqrt(1.0 - rr);
  let on_quad = in.center + (fluid.right.xyz * in.corner.x + fluid.up.xyz * in.corner.y) * in.radius;
  let front = on_quad - frame.view_forward.xyz * (in.radius * z);
  let d = view_depth(front);
  if (d > textureLoad(scene_depth, vec2<i32>(in.clip.xy), 0).r) { discard; }
  let c = frame.view_proj * vec4<f32>(front, 1.0);
  var o: DepthOut;
  o.depth = vec4<f32>(d, 0.0, 0.0, 0.0);
  o.frag = clamp(c.z / c.w, 0.0, 1.0);
  return o;
}

// Each sphere's chord through the pixel, cut where it passes behind the
// scene, weighted by its optical density (color_k.w: -ln(1 - opacity) per
// metre times the particle's share of the volume), added up.
@fragment
fn fs_thick(in: SphereOut) -> @location(0) vec4<f32> {
  let rr = dot(in.corner, in.corner);
  if (rr > 1.0) { discard; }
  let z = sqrt(1.0 - rr);
  let center_depth = view_depth(in.center);
  let front = center_depth - in.radius * z;
  let scene = textureLoad(scene_depth, vec2<i32>(in.clip.xy), 0).r;
  let chord = clamp(scene - front, 0.0, 2.0 * in.radius * z);
  if (chord <= 0.0) { discard; }
  let w = chord * in.color_k.w;
  return vec4<f32>(in.color_k.rgb * w, w);
}

struct FullOut { @builtin(position) pos: vec4<f32> };

@vertex
fn vs_full(@builtin(vertex_index) i: u32) -> FullOut {
  var o: FullOut;
  let x = f32(i32(i & 1u) * 4 - 1);
  let y = f32(i32(i & 2u) * 2 - 1);
  o.pos = vec4<f32>(x, y, 0.0, 1.0);
  return o;
}

// The world point at pixel `px` (its centre) and view depth `d`.
fn world_at(px: vec2<f32>, d: f32) -> vec3<f32> {
  let uv = px * fluid.screen.zw;
  let ndc = vec2<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0);
  let a4 = frame.inv_view_proj * vec4<f32>(ndc, 0.0, 1.0);
  let b4 = frame.inv_view_proj * vec4<f32>(ndc, 1.0, 1.0);
  let a = a4.xyz / a4.w;
  let dir = normalize(b4.xyz / b4.w - a);
  return a + dir * ((d - view_depth(a)) / max(dot(dir, frame.view_forward.xyz), 1e-6));
}

fn depth_at(px: vec2<i32>) -> f32 {
  let size = vec2<i32>(fluid.screen.xy);
  return textureLoad(fluid_depth, clamp(px, vec2<i32>(0), size - vec2<i32>(1)), 0).r;
}

// The surface's tangent along `step` (a pixel offset): the side whose
// depth differs least, so silhouettes keep their own slope.
fn tangent(px: vec2<i32>, p: vec3<f32>, d: f32, step: vec2<i32>) -> vec3<f32> {
  let df = depth_at(px + step);
  let db = depth_at(px - step);
  let pc = vec2<f32>(px) + vec2<f32>(0.5);
  if (abs(df - d) <= abs(db - d) && df < NONE) {
    return world_at(pc + vec2<f32>(step), df) - p;
  }
  if (db < NONE) {
    return p - world_at(pc - vec2<f32>(step), db);
  }
  return world_at(pc + vec2<f32>(step), d) - p;
}

fn cluster_lights(frag: vec2<f32>, s: Surface, p: vec3<f32>, n: vec3<f32>, v: vec3<f32>) -> vec3<f32> {
  var color = vec3<f32>(0.0);
  if (frame.params.z < 0.5) { return color; }
  let word = cluster_word(cluster_at(frag, p));
  let first = word & 0xffffffu;
  let count = word >> 24u;
  for (var k = 0u; k < count; k++) {
    // Rect lights through LTC, points and spots with cookies (area.wgsl).
    color += surface_local_light(light_list[cluster_ref(first + k)], s, p, n, v);
  }
  return color;
}

struct CompositeOut {
  @location(0) color: vec4<f32>,
  @builtin(frag_depth) depth: f32,
};

@fragment
fn fs_composite(in: FullOut) -> CompositeOut {
  let px = vec2<i32>(in.pos.xy);
  let d = depth_at(px);
  if (d >= NONE) { discard; }
  let p = world_at(in.pos.xy, d);
  let tx = tangent(px, p, d, vec2<i32>(1, 0));
  let ty = tangent(px, p, d, vec2<i32>(0, 1));
  var n = normalize(cross(ty, tx));
  let v = select(normalize(frame.view_pos.xyz - p), -frame.view_forward.xyz, frame.view_pos.w > 0.5);
  if (dot(n, v) < 0.0) { n = -n; }

  let thick = textureLoad(fluid_thick, px, 0);
  let a = max(thick.a, 1e-5);
  let color = clamp(thick.rgb / a, vec3<f32>(0.0), vec3<f32>(1.0));
  let transmit = exp(-a);

  // What lies behind, shifted by the normal across the screen, unless the
  // shifted sample is in front of the surface.
  let uv = in.pos.xy * fluid.screen.zw;
  let bend = vec2<f32>(dot(n, fluid.right.xyz), -dot(n, fluid.up.xyz));
  var ruv = clamp(uv + bend * fluid.look.x * 0.03 * vec2<f32>(fluid.screen.y * fluid.screen.z, 1.0), vec2<f32>(0.0), vec2<f32>(1.0));
  let behind = textureLoad(scene_depth, vec2<i32>(ruv * fluid.screen.xy), 0).r;
  if (behind < d) { ruv = uv; }
  let background = textureSampleLevel(scene_color, fluid_sampler, ruv, 0.0).rgb;
  // Light passing through is tinted by the fluid's colour as it thins out.
  let tint = mix(color / max(max(color.r, max(color.g, color.b)), 1e-3), vec3<f32>(1.0), transmit);
  let through = background * tint * transmit;

  // Light scattered by the body: the sun, the image-based light, the local lights.
  var s: Surface;
  s.albedo = color;
  s.metallic = 0.0;
  s.roughness = 0.12;
  s.f0 = vec3<f32>(0.02);
  s.coat = 0.0;
  s.coat_roughness = 0.0;
  s.coat_normal = n;
  let l = normalize(frame.light_dir.xyz);
  let n_dot_l = max(dot(n, l), 0.0);
  var shadow = 1.0;
  if (frame.light_dir.w > 0.5) { shadow = cascade_shadow(p, n, n_dot_l); }
  var body = brdf(s, n, v, l) * frame.light_color.rgb * shadow;
  body += ambient_light_level(n, p) * color * frame.params.x;
  body += cluster_lights(in.pos.xy, s, p, n, v);
  var out = through + body * (1.0 - transmit);

  // The environment reflected (Schlick, water's reflectance).
  let n_dot_v = max(dot(n, v), 0.0);
  let fresnel = clamp((0.02 + 0.98 * pow(1.0 - n_dot_v, 5.0)) * fluid.up.w, 0.0, 1.0);
  // The environment, or the reflection probe holding the fluid (probes.rs).
  let reflected = reflected_light(reflect(-v, n), p, 0.5) * frame.params.x;
  out = mix(out, reflected, fresnel);
  let h = normalize(l + v);
  out += frame.light_color.rgb * pow(max(dot(n, h), 0.0), 200.0) * 2.0 * fresnel * fluid.up.w * shadow;

  let c = frame.view_proj * vec4<f32>(p, 1.0);
  var o: CompositeOut;
  o.color = vec4<f32>(apply_fog(out, p), 1.0);
  o.depth = clamp(c.z / c.w, 0.0, 1.0);
  return o;
}
