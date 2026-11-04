// The depth-normal prepass's fragment shader, after pbr_fs.wgsl.

struct PrepassTargets {
  // xy: world normal (octahedral), zw: screen-space velocity (uv per frame).
  @location(0) gbuffer: vec4<f32>,
  // View depth along the camera's forward axis (world units).
  @location(1) depth: vec4<f32>,
  // Bytes (`outline_bytes`): the outline colour in 5:6:5 over r and g, the
  // width in eighths of a pixel (0: the scene's outline), the object's id.
  @location(2) outline: vec4<f32>,
};

@fragment
fn fs_prepass(in: PrepassOut, @builtin(front_facing) front_facing: bool) -> PrepassTargets {
  hook_front = select(-1.0, 1.0, front_facing);
  return prepass_targets(in);
}

// What the prepass writes for a fragment (`fs_prepass`, and
// `fs_prepass_lean`, which may drop the discard: DISCARD in pbr_fs.wgsl).
fn prepass_targets(in: PrepassOut) -> PrepassTargets {
  let base_uv = material_uv(UV_MAP, in.uv);
  var base_sample: vec4<f32>;
  if (ANALYTIC) {
    let gx = analytic_grad(UV_MAP, in.uv, analytic_uv.xy);
    let gy = analytic_grad(UV_MAP, in.uv, analytic_uv.zw);
    base_sample = textureSampleGrad(base_color_tex, material_sampler, base_uv, gx, gy);
  } else {
    base_sample = textureSample(base_color_tex, material_sampler, base_uv);
  }
  // A material shader's colour (and alpha cut) as in the lit pass.
  var hook: Hook;
  hook.world = in.world_pos;
  hook.local = in.local_pos;
  hook.normal = in.normal;
  hook.uv = in.uv;
  hook.color = in.color;
  hook.time = frame.time.x;
  hook.origin = frame.origin.xyz;
  hook.eye = frame.view_pos.xyz;
  hook.front = hook_front;
  let base = surface_color(hook, shader_params(), material.base_color * base_sample * in.color);
  let dissolved = dissolve_at(in.uv, in.local_pos).w > 0.5;
  if (DISCARD && ((material.emissive.w >= 0.0 && base.a < material.emissive.w) || dissolved)) {
    discard;
  }
  var n = normalize(in.normal);
  if ((u32(material.flags.w) & 1u) != 0u) {
    n = normalize(cross(world_dx(in.world_pos), world_dy(in.world_pos)));
  }
  let v = select(normalize(frame.view_pos.xyz - in.world_pos), -frame.view_forward.xyz, frame.view_pos.w > 0.5);
  if (dot(n, v) < 0.0) { n = -n; }
  let cur = in.cur_clip.xy / in.cur_clip.w;
  let prev = in.prev_clip.xy / max(in.prev_clip.w, 1e-6);
  let velocity = (cur - prev) * vec2<f32>(0.5, -0.5);
  var o: PrepassTargets;
  o.gbuffer = vec4<f32>(oct_encode(n), velocity);
  // y: the reflectivity screen-space reflections read (ssr.rs): the gloss
  // and metalness as the lit pass shades them (the factors times their
  // maps' b and g), none for an unlit material (mode bit 2).
  let unlit = (u32(material.flags.w) & 2u) != 0u;
  var metalness = material.mr.x * material_sample(metalness_tex, UV_METALNESS, in.uv).b;
  var roughness = material.mr.y * material_sample(roughness_tex, UV_METALNESS, in.uv).g;
  // Wet ground, streaks and puddles as the lit pass shades them
  // (weather_surface.wgsl weather_gloss): reflections see the puddles as
  // mirrors and their ripples, and the normal is the water's.
  if (!unlit) {
    let g = weather_gloss(clamp(roughness, 0.04, 1.0), metalness, n, n, in.world_pos, weather_cavity(in.uv), material.uv[3].w);
    roughness = g.s.roughness;
    metalness = g.s.metallic;
    o.gbuffer = vec4<f32>(oct_encode(g.n), velocity);
  }
  let reflectivity = select(reflectivity_bits(roughness, metalness), 0.0, unlit);
  // Above the reflectivity's 16 bits, the painterly filter's detail byte
  // (stylise.wgsl paint_detail): the material's paintDetail in 7 bits, bit 7
  // a skinned instance (a character). At most 2^24, exact in an f32.
  let detail = u32(round(clamp(material.uv[3].z, 0.0, 1.0) * 127.0)) | (((in.object >> 8u) & 1u) << 7u);
  o.depth = vec4<f32>(dot(in.world_pos - frame.view_pos.xyz, frame.view_forward.xyz), reflectivity + 65536.0 * f32(detail), 0.0, 1.0);
  // The material style's outline distance: nearer than it the line grows,
  // up to four times its width (a ground a metre under the camera is not
  // lined 30 px wide), farther it thins in proportion (never for an
  // orthographic camera, nor an unstyled material: mode bit 8).
  var width = material.outline.w;
  let unstyled = (u32(material.flags.w) & 8u) != 0u;
  if (frame.outline_style.x > 0.0 && frame.view_pos.w < 0.5 && width > 0.0 && !unstyled) {
    let depth = max(dot(in.world_pos - frame.view_pos.xyz, frame.view_forward.xyz), 0.01);
    // Never under a pixel: a thinner line is drawn at its coverage
    // (fxpost.wgsl fs_outline, fs_outline_wide) and read as a faint grey
    // where a far silhouette wants ink (inked-comic-3d's hills).
    width = clamp(width * frame.outline_style.x / depth, 1.0, max(min(31.0, width * 4.0), 1.0));
  }
  o.outline = outline_bytes(material.outline.rgb, width, in.object & 255u);
  return o;
}

// The outline target's texel (Rgba8Unorm, read back as bytes by
// fxpost.wgsl `fs_outline`): the colour in 5:6:5 (r its high byte, g its
// low), the width in eighths of a pixel, the object's id.
fn outline_bytes(color: vec3<f32>, width: f32, object: u32) -> vec4<f32> {
  let c = vec3<u32>(round(clamp(color, vec3<f32>(0.0), vec3<f32>(1.0)) * vec3<f32>(31.0, 63.0, 31.0)));
  let packed = (c.r << 11u) | (c.g << 5u) | c.b;
  let w = u32(round(clamp(width, 0.0, 31.875) * 8.0));
  return vec4<f32>(f32(packed >> 8u), f32(packed & 255u), f32(w), f32(object)) / 255.0;
}
