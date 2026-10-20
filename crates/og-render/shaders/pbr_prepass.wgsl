// The depth-normal prepass's fragment shader, after pbr_fs.wgsl.

struct PrepassTargets {
  // xy: world normal (octahedral), zw: screen-space velocity (uv per frame).
  @location(0) gbuffer: vec4<f32>,
  // View depth along the camera's forward axis (world units).
  @location(1) depth: vec4<f32>,
  // rgb: outline colour, a: outline width / 32 (0: the scene's outline).
  @location(2) outline: vec4<f32>,
};

@fragment
fn fs_prepass(in: PrepassOut) -> PrepassTargets {
  let base_sample = textureSample(base_color_tex, material_sampler, in.uv);
  // A material shader's colour (and alpha cut) as in the lit pass.
  var hook: Hook;
  hook.world = in.world_pos;
  hook.local = in.local_pos;
  hook.normal = in.normal;
  hook.uv = in.uv;
  hook.color = in.color;
  hook.time = frame.time.x;
  let base = surface_color(hook, shader_params(), material.base_color * base_sample * in.color);
  let dissolved = dissolve_at(in.uv, in.local_pos).w > 0.5;
  if ((material.emissive.w >= 0.0 && base.a < material.emissive.w) || dissolved) {
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
  // farther it thins in proportion (never for an orthographic camera).
  var width = material.outline.w;
  if (frame.outline_style.x > 0.0 && frame.view_pos.w < 0.5 && width > 0.0) {
    let depth = max(dot(in.world_pos - frame.view_pos.xyz, frame.view_forward.xyz), 0.01);
    width = clamp(width * frame.outline_style.x / depth, 0.5, 31.0);
  }
  o.outline = vec4<f32>(clamp(material.outline.rgb, vec3<f32>(0.0), vec3<f32>(1.0)), width / 32.0);
  return o;
}
