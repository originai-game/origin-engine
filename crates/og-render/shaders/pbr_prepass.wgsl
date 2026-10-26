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
  let base_sample = textureSample(base_color_tex, material_sampler, in.uv);
  // A material shader's colour (and alpha cut) as in the lit pass.
  var hook: Hook;
  hook.world = in.world_pos;
  hook.local = in.local_pos;
  hook.normal = in.normal;
  hook.uv = in.uv;
  hook.color = in.color;
  hook.time = frame.time.x;
  hook.eye = frame.view_pos.xyz;
  hook.front = hook_front;
  let base = surface_color(hook, shader_params(), material.base_color * base_sample * in.color);
  let dissolved = dissolve_at(in.uv, in.local_pos).w > 0.5;
  if (DISCARD && ((material.emissive.w >= 0.0 && base.a < material.emissive.w) || dissolved)) {
    discard;
  }
  var n = normalize(in.normal);
  if ((u32(material.flags.w) & 1u) != 0u) {
    n = normalize(cross(dpdx(in.world_pos), dpdy(in.world_pos)));
  }
  let v = select(normalize(frame.view_pos.xyz - in.world_pos), -frame.view_forward.xyz, frame.view_pos.w > 0.5);
  if (dot(n, v) < 0.0) { n = -n; }
  let cur = in.cur_clip.xy / in.cur_clip.w;
  let prev = in.prev_clip.xy / max(in.prev_clip.w, 1e-6);
  let velocity = (cur - prev) * vec2<f32>(0.5, -0.5);
  var o: PrepassTargets;
  o.gbuffer = vec4<f32>(oct_encode(n), velocity);
  o.depth = vec4<f32>(dot(in.world_pos - frame.view_pos.xyz, frame.view_forward.xyz), 0.0, 0.0, 1.0);
  // The material style's outline distance: nearer than it the line grows,
  // up to four times its width (a ground a metre under the camera is not
  // lined 30 px wide), farther it thins in proportion (never for an
  // orthographic camera, nor an unstyled material: mode bit 8).
  var width = material.outline.w;
  let unstyled = (u32(material.flags.w) & 8u) != 0u;
  if (frame.outline_style.x > 0.0 && frame.view_pos.w < 0.5 && width > 0.0 && !unstyled) {
    let depth = max(dot(in.world_pos - frame.view_pos.xyz, frame.view_forward.xyz), 0.01);
    width = clamp(width * frame.outline_style.x / depth, 0.5, min(31.0, width * 4.0));
  }
  o.outline = outline_bytes(material.outline.rgb, width, in.object);
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
