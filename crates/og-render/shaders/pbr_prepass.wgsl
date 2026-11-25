// The depth-normal prepass's fragment shader, after pbr_fs.wgsl.

struct PrepassTargets {
  // xy: world normal (octahedral), zw: screen-space velocity (uv per frame).
  @location(0) gbuffer: vec4<f32>,
  // View depth along the camera's forward axis (world units).
  @location(1) depth: vec4<f32>,
  // Bytes (`outline_bytes`): the outline colour and clump bit over r and g, the
  // width in eighths of a pixel (0: the scene's outline), the object's id.
  @location(2) outline: vec4<f32>,
  @location(3) motion_depth: f32,
};

@fragment
fn fs_prepass(in: PrepassOut, @builtin(front_facing) front_facing: bool) -> PrepassTargets {
  hook_front = select(-1.0, 1.0, front_facing);
  return prepass_targets(in);
}

// What the prepass writes for a fragment (`fs_prepass`, and
// `fs_prepass_lean`, which may drop the discard: DISCARD in pbr_fs.wgsl).
fn prepass_targets(in: PrepassOut) -> PrepassTargets {
  let mode = u32(material.flags.w);
  // The lit pass's uv (pbr.wgsl `stylised_uv`), so both cut the same pixels.
  let uv = stylised_uv(in.uv, in.affine);
  // A material shader's hooks (identities with the built-in hooks).
  var hook: Hook;
  hook.world = in.world_pos;
  hook.local = in.local_pos;
  hook.normal = in.normal;
  hook.uv = uv;
  hook.color = in.color;
  hook.time = frame.time.x;
  hook.origin = frame.origin.xyz;
  hook.eye = frame.view_pos.xyz;
  hook.front = hook_front;
  hook.wind = frame.foliage_wind[0][0];
  let params = shader_params();
  if (DISCARD) {
    var base_sample = vec4<f32>(1.0);
    if ((mode & 64u) != 0u) {
      let base_uv = material_uv(UV_MAP, uv);
      if (ANALYTIC) {
        let gx = analytic_grad(UV_MAP, uv, analytic_uv.xy);
        let gy = analytic_grad(UV_MAP, uv, analytic_uv.zw);
        base_sample = base_texel(base_uv, gx, gy);
      } else {
        base_sample = base_texel(base_uv, dpdxFine(base_uv), dpdyFine(base_uv));
      }
    }
    let base = surface_color(hook, params, material.base_color * base_sample * in.color);
    let dissolved = dissolve_at(uv, in.local_pos).w > 0.5;
    // An instance fading in: the pixels the lit pass cuts (instance.wgsl
    // `dither_cut`), from the bits `outline_object` put above its id.
    let hidden = dither_cut(hidden_share((in.object >> 16u) & 511u), in.clip.xy, frame.drawn.w);
    if ((material.emissive.w >= 0.0 && base.a < material.emissive.w) || dissolved || hidden) {
      discard;
    }
  }
  var n = normalize(in.normal);
  if ((mode & 1u) != 0u) {
    n = -normalize(cross(world_dx(in.world_pos), world_dy(in.world_pos))) * hook_front;
  }
  n *= material_face(hook_front);
  let cur = in.cur_clip.xy / max(in.cur_clip.w, 1e-4);
  let prev = in.prev_clip.xy / max(in.prev_clip.w, 1e-4);
  let valid_vel = in.cur_clip.w > 1e-4 && in.prev_clip.w > 1e-4;
  let velocity = select(vec2<f32>(0.0), (cur - prev) * vec2<f32>(0.5, -0.5), valid_vel);
  var o: PrepassTargets;
  o.gbuffer = vec4<f32>(oct_encode(n), velocity);
  o.motion_depth = select(in.cur_clip.z / max(in.cur_clip.w, 1e-4), in.prev_clip.z / max(in.prev_clip.w, 1e-4), valid_vel);
  // y: the reflectivity screen-space reflections read (ssr.rs): the gloss
  // and metalness as the lit pass shades them (the factors times their
  // maps' b and g), none for an unlit material (mode bit 2).
  let unlit = (mode & 2u) != 0u;
  var reflectivity = 0.0;
  // Wet ground, streaks and puddles as the lit pass shades them
  // (weather_surface.wgsl weather_gloss): reflections see the puddles as
  // mirrors and their ripples, and the normal is the water's.
  if (!unlit) {
    var metalness_factor = material.mr.x;
    if ((mode & 128u) != 0u && metalness_factor > 0.0) {
      metalness_factor *= material_sample(metalness_tex, UV_METALNESS, uv, 1u).b;
    }
    var roughness_factor = material.mr.y;
    if ((mode & 256u) != 0u) {
      roughness_factor *= material_sample(roughness_tex, UV_METALNESS, uv, 2u).g;
    }
    var metalness = clamp(surface_metalness(hook, params, metalness_factor), 0.0, 1.0);
    var roughness = clamp(surface_roughness(hook, params, roughness_factor), 0.04, 1.0);
    var surface: Surface;
    surface.metallic = metalness;
    surface.roughness = roughness;
    let dpx = world_dx(in.world_pos);
    let dpy = world_dy(in.world_pos);
    let footprint = length(dpx) + length(dpy);
    let aged = material_wear(surface, in.uv, n, in.world_pos, footprint);
    metalness = aged.metallic;
    roughness = aged.roughness;
    let wet = wet_decal_gloss_grad(in.clip.xy, in.world_pos, n, n, clamp(roughness, 0.04, 1.0), dpx, dpy);
    var g: Weathered;
    if (ANALYTIC) {
      g = weather_gloss_grad(wet.w, metalness, wet.xyz, n, in.world_pos, weather_cavity(uv), material.uv[3].w, select(WEATHER_STANDS, WEATHER_MOVES, (in.object & 256u) != 0u), footprint, 0.0);
    } else {
      g = weather_gloss(wet.w, metalness, wet.xyz, n, in.world_pos, weather_cavity(uv), material.uv[3].w, select(WEATHER_STANDS, WEATHER_MOVES, (in.object & 256u) != 0u));
    }
    roughness = g.s.roughness;
    metalness = g.s.metallic;
    o.gbuffer = vec4<f32>(oct_encode(g.n), velocity);
    reflectivity = reflectivity_bits(roughness, metalness);
  }
  // Above the reflectivity's 16 bits, the painterly filter's detail byte
  // (stylise.wgsl paint_detail): the material's paintDetail in 7 bits, bit 7
  // a skinned instance (a character). At most 2^24, exact in an f32.
  let detail = u32(round(clamp(material.uv[3].z, 0.0, 1.0) * 127.0)) | (((in.object >> 8u) & 1u) << 7u);
  o.depth = vec4<f32>(dot(in.world_pos - frame.view_pos.xyz, frame.view_forward.xyz), reflectivity + 65536.0 * f32(detail), 0.0, 1.0);
  // The material style's outline distance (style_line_width), never for
  // an unstyled material: mode bit 8.
  var width = material.outline.w;
  if ((u32(material.flags.w) & 8u) == 0u) {
    width = style_line_width(width, in.world_pos, false);
  }
  o.outline = outline_bytes(material.outline.rgb, width, in.object & 255u, false);
  return o;
}
