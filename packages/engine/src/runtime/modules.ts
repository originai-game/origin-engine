import type * as Api from "@origingame/engine";
import { registerCommand } from "./declare.ts";
import { abi } from "./host.ts";
import { json, parse } from "./result.ts";
import { checkAccess } from "./views.ts";
import { WorldImpl, uiReactor } from "./world.ts";

// ── free functions ────────────────────────────────────────────────────────

export const transform = Object.freeze({
  compose: (position: unknown, quaternion: unknown, scale: unknown) =>
    parse(abi().og_transform_compose(JSON.stringify(position), JSON.stringify(quaternion), JSON.stringify(scale))),
  decompose: (matrix: unknown) => parse(abi().og_transform_decompose(JSON.stringify(matrix))),
  lookAt: (eye: unknown, target: unknown, up?: unknown) =>
    parse(abi().og_transform_look_at(JSON.stringify(eye), JSON.stringify(target), json(up))),
  face: (position: unknown, target: unknown, up?: unknown) =>
    parse(abi().og_transform_face(JSON.stringify(position), JSON.stringify(target), json(up))),
  forward: (quaternion: unknown) => parse(abi().og_transform_forward(JSON.stringify(quaternion))),
  right: (quaternion: unknown) => parse(abi().og_transform_right(JSON.stringify(quaternion))),
  up: (quaternion: unknown) => parse(abi().og_transform_up(JSON.stringify(quaternion))),
  cameraForward: (quaternion: unknown) => parse(abi().og_transform_camera_forward(JSON.stringify(quaternion))),
  cameraRight: (quaternion: unknown) => parse(abi().og_transform_camera_right(JSON.stringify(quaternion))),
  heading: (quaternion: unknown) => abi().og_transform_heading(JSON.stringify(quaternion)),
  fromHeading: (heading: number) => parse(abi().og_transform_from_heading(heading)),
  face2d: (position: unknown, target: unknown) =>
    parse(abi().og_transform_face2d(JSON.stringify(position), JSON.stringify(target))),
  forward2d: (quaternion: unknown) => parse(abi().og_transform_forward2d(JSON.stringify(quaternion))),
  heading2d: (quaternion: unknown) => abi().og_transform_heading2d(JSON.stringify(quaternion)),
  fromHeading2d: (heading: number) => parse(abi().og_transform_from_heading2d(heading)),
  fromAxisAngle: (axis: unknown, angle: number) => parse(abi().og_transform_from_axis_angle(JSON.stringify(axis), angle)),
});

export const render = Object.freeze({
  standardMaterial: (world: WorldImpl, desc?: unknown) => parse(abi().og_render_standard_material(world.id, json(desc))),
  updateMaterial: (world: WorldImpl, material: number, desc?: unknown) =>
    parse(abi().og_render_update_material(world.id, material, json(desc))),
  capabilities: (world: WorldImpl) => parse(abi().og_render_capabilities(world.id)),
  applyLook: (world: WorldImpl, look: number) => parse(abi().og_render_apply_look(world.id, look)),
  applyQuality: (world: WorldImpl, preset: string) => parse(abi().og_render_apply_quality(world.id, preset)),
  setDisplay: (world: WorldImpl, settings?: unknown) => parse(abi().og_render_set_display(world.id, json(settings))),
  resetDisplay: (world: WorldImpl) => parse(abi().og_render_reset_display(world.id)),
  display: (world: WorldImpl) => parse(abi().og_render_display(world.id)),
  look: (world: WorldImpl) => parse(abi().og_render_look(world.id)),
  snap: (world: WorldImpl, entity: number) => parse(abi().og_render_snap(world.id, entity)),
  snapCamera: (world: WorldImpl) => abi().og_render_snap_camera(world.id),
  text: (world: WorldImpl, desc?: unknown) => parse(abi().og_render_text(world.id, json(desc))),
  sprite: (world: WorldImpl, desc?: unknown) => parse(abi().og_render_sprite(world.id, json(desc))),
  ribbon: (world: WorldImpl, desc?: unknown) => parse(abi().og_render_ribbon(world.id, json(desc))),
  measureText: (world: WorldImpl, desc?: unknown) => parse(abi().og_render_measure_text(world.id, json(desc))),
  floatingText: (world: WorldImpl, desc?: unknown) => parse(abi().og_render_floating_text(world.id, json(desc))),
  boxGeometry: (world: WorldImpl, desc?: unknown) => parse(abi().og_render_box_geometry(world.id, json(desc))),
  planeGeometry: (world: WorldImpl, desc?: unknown) => parse(abi().og_render_plane_geometry(world.id, json(desc))),
  sphereGeometry: (world: WorldImpl, desc?: unknown) => parse(abi().og_render_sphere_geometry(world.id, json(desc))),
  shaderMaterial: (world: WorldImpl, shader: number, desc?: unknown) =>
    parse(abi().og_render_shader_material(world.id, shader, json(desc))),
  setMaterialShader: (world: WorldImpl, material: number, shader: number | null) =>
    parse(abi().og_render_set_material_shader(world.id, material, JSON.stringify(shader ?? null))),
  setShaderParam: (world: WorldImpl, material: number, name: string, value: ReadonlyArray<number>) =>
    parse(abi().og_render_set_shader_param(world.id, material, name, JSON.stringify(value))),
});

export const load = Object.freeze({
  bundle: (world: WorldImpl, path: string) => parse(abi().og_load_bundle(world.id, path)),
  bundleInfo: (world: WorldImpl, asset: number) => parse(abi().og_load_bundle_info(world.id, asset)),
  defaultBundle: (world: WorldImpl) => parse(abi().og_load_default_bundle(world.id)),
  gltf: (world: WorldImpl, path: string, desc?: unknown) => parse(abi().og_load_gltf(world.id, path, json(desc))),
  imageData: (world: WorldImpl, name: string, bytes: Uint8Array, desc?: unknown) => parse(abi().og_load_image_data(world.id, name, bytes, json(desc))),
  gltfData: (world: WorldImpl, name: string, bytes: Uint8Array, desc?: unknown) => parse(abi().og_load_gltf_data(world.id, name, bytes, json(desc))),
  environmentData: (world: WorldImpl, name: string, bytes: Uint8Array, desc?: unknown) => parse(abi().og_load_environment_data(world.id, name, bytes, json(desc))),
  environmentInfo: (world: WorldImpl, asset: number) => parse(abi().og_load_environment_info(world.id, asset)),
  release: (world: WorldImpl, asset: number) => parse(abi().og_load_release(world.id, asset)),
  imageInfo: (world: WorldImpl, asset: number) => parse(abi().og_load_image_info(world.id, asset)),
  completions: (world: WorldImpl) => parse(abi().og_load_completions(world.id)),
  replayCompletions: (world: WorldImpl, completions: unknown) => parse(abi().og_load_replay_completions(world.id, JSON.stringify(completions))),
  scene: (world: WorldImpl, path: string) => parse(abi().og_load_scene(world.id, path)),
  prefab: (world: WorldImpl, path: string) => parse(abi().og_load_prefab(world.id, path)),
  font: (world: WorldImpl, path: string) => parse(abi().og_load_font(world.id, path)),
  fontInfo: (world: WorldImpl, font: number) => parse(abi().og_load_font_info(world.id, font)),
  sound: (world: WorldImpl, path: string) => parse(abi().og_load_sound(world.id, path)),
  texture: (world: WorldImpl, path: string) => parse(abi().og_load_texture(world.id, path)),
  textureArray: (world: WorldImpl, path: string) => parse(abi().og_load_texture_array(world.id, path)),
  lut: (world: WorldImpl, path: string) => parse(abi().og_load_lut(world.id, path)),
  environment: (world: WorldImpl, path: string) => parse(abi().og_load_environment(world.id, path)),
  environmentSun: (world: WorldImpl, map: number) => parse(abi().og_load_environment_sun(world.id, map)),
  emitter: (world: WorldImpl, path: string) => parse(abi().og_load_emitter(world.id, path)),
  shader: (world: WorldImpl, path: string) => parse(abi().og_load_shader(world.id, path)),
  vector: (world: WorldImpl, path: string) => parse(abi().og_load_vector(world.id, path)),
  uiMaterial: (world: WorldImpl, path: string) => parse(abi().og_load_ui_material(world.id, path)),
  document: (world: WorldImpl, path: string) => parse(abi().og_load_document(world.id, path)),
  style: (world: WorldImpl, path: string) => parse(abi().og_load_style(world.id, path)),
  look: (world: WorldImpl, path: string) => parse(abi().og_load_look(world.id, path)),
  lookInfo: (world: WorldImpl, look: number) => parse(abi().og_load_look_info(world.id, look)),
  palette: (world: WorldImpl, path: string) => parse(abi().og_load_palette(world.id, path)),
  soundInfo: (world: WorldImpl, sound: number) => parse(abi().og_load_sound_info(world.id, sound)),
  video: (world: WorldImpl, path: string) => parse(abi().og_load_video(world.id, path)),
  videoInfo: (world: WorldImpl, video: number) => parse(abi().og_load_video_info(world.id, video)),
  status: (world: WorldImpl, asset: number) => abi().og_load_status(world.id, asset),
  error: (world: WorldImpl, asset: number) => parse(abi().og_load_error(world.id, asset)),
  gltfInfo: (world: WorldImpl, asset: number) => parse(abi().og_load_gltf_info(world.id, asset)),
  animation: (world: WorldImpl, path: string) => parse(abi().og_load_animation(world.id, path)),
  timeline: (world: WorldImpl, path: string) => parse(abi().og_load_timeline(world.id, path)),
  vfx: (world: WorldImpl, path: string) => parse(abi().og_load_vfx(world.id, path)),
  flipbook: (world: WorldImpl, path: string) => parse(abi().og_load_flipbook(world.id, path)),
  rig: (world: WorldImpl, path: string) => parse(abi().og_load_rig(world.id, path)),
  rig2d: (world: WorldImpl, path: string) => parse(abi().og_load_rig2d(world.id, path)),
  tilemap: (world: WorldImpl, path: string) => parse(abi().og_load_tilemap(world.id, path)),
});

export const anim = Object.freeze({
  graph: (world: WorldImpl, desc?: unknown) => parse(abi().og_anim_graph(world.id, json(desc))),
  humanoid: (world: WorldImpl, desc?: unknown) => parse(abi().og_anim_humanoid(world.id, json(desc))),
  retarget: (world: WorldImpl, desc?: unknown) => parse(abi().og_anim_retarget(world.id, json(desc))),
  mirror: (world: WorldImpl, desc?: unknown) => parse(abi().og_anim_mirror(world.id, json(desc))),
  rig2dInfo: (world: WorldImpl, rig: number) => parse(abi().og_anim_rig2d_info(world.id, rig)),
  skeleton2dBoxes: (world: WorldImpl, entity: number) => parse(abi().og_anim_skeleton2d_boxes(world.id, entity)),
  // One lookup for a flipbook's or a 2D rig's clips (an index, -1 when
  // missing) and a loaded model's (og.load.animation: a ClipAsset, 0 when missing).
  clip: (info: { clips?: ReadonlyArray<{ clip: number; name: string }>; skeleton?: number }, name: string): number =>
    info?.clips?.find((c) => c.name === name)?.clip ?? (info?.skeleton !== undefined ? 0 : -1),
});

// ── E4: time scale, camera feel, tweens, tilemaps ───────────────────────────

export const time = Object.freeze({
  setScale: (world: WorldImpl, scale: number) => parse(abi().og_time_set_scale(world.id, scale)),
  hitStop: (world: WorldImpl, ticks: number) => parse(abi().og_time_hit_stop(world.id, ticks)),
  setGroupScale: (world: WorldImpl, group: number, scale: number) => parse(abi().og_time_set_group_scale(world.id, group, scale)),
  groupDt: (world: WorldImpl, entity: number) => abi().og_time_group_dt(world.id, entity),
});

export const camera = Object.freeze({
  shake: (world: WorldImpl, trauma: number) => parse(abi().og_camera_shake(world.id, trauma)),
  punch: (world: WorldImpl, amount: number) => parse(abi().og_camera_punch(world.id, amount)),
  kick: (world: WorldImpl, direction: unknown, distance: number) =>
    parse(abi().og_camera_kick(world.id, JSON.stringify(direction), distance)),
});

export const tween = Object.freeze({
  to: (world: WorldImpl, desc?: unknown) => parse(abi().og_tween_to(world.id, json(desc))),
  from: (world: WorldImpl, desc?: unknown) => parse(abi().og_tween_from(world.id, json(desc))),
  fromTo: (world: WorldImpl, desc?: unknown) => parse(abi().og_tween_from_to(world.id, json(desc))),
  sequence: (world: WorldImpl, steps: unknown, desc?: unknown) =>
    parse(abi().og_tween_sequence(world.id, JSON.stringify(steps), json(desc))),
  parallel: (world: WorldImpl, tweens: unknown, desc?: unknown) =>
    parse(abi().og_tween_parallel(world.id, JSON.stringify(tweens), json(desc))),
  cancel: (world: WorldImpl, t: number) => abi().og_tween_cancel(world.id, t),
  cancelEntity: (world: WorldImpl, entity: number) => abi().og_tween_cancel_entity(world.id, entity),
  active: (world: WorldImpl, t: number) => abi().og_tween_active(world.id, t),
  squash: (world: WorldImpl, desc?: unknown) => parse(abi().og_tween_squash(world.id, json(desc))),
  flash: (world: WorldImpl, desc?: unknown) => parse(abi().og_tween_flash(world.id, json(desc))),
  timeline: (world: WorldImpl, desc?: unknown) => tweenTimeline(abi().og_tween_timeline(world.id, json(desc))),
});

/** A TweenTimeline: its builders answer the timeline itself, so calls chain as GSAP's. */
function tweenTimeline(h: number): object {
  const a = abi;
  const at = (position: unknown) => json(position);
  const tl: Record<string, unknown> = {};
  const chain = (_: number) => tl;
  Object.defineProperties(tl, {
    tween: { enumerable: true, get: () => a().og_tween_timeline_get_tween(h) },
    error: { enumerable: true, get: () => parse<string | null>(a().og_tween_timeline_get_error(h)) },
  });
  Object.assign(tl, {
    to: (targets: unknown, vars?: unknown, position?: unknown) =>
      chain(a().og_tween_timeline_to(h, JSON.stringify(targets), json(vars), at(position))),
    from: (targets: unknown, vars?: unknown, position?: unknown) =>
      chain(a().og_tween_timeline_from(h, JSON.stringify(targets), json(vars), at(position))),
    fromTo: (targets: unknown, vars?: unknown, position?: unknown) =>
      chain(a().og_tween_timeline_from_to(h, JSON.stringify(targets), json(vars), at(position))),
    set: (targets: unknown, vars?: unknown, position?: unknown) =>
      chain(a().og_tween_timeline_set(h, JSON.stringify(targets), json(vars), at(position))),
    call: (name: string, position?: unknown) => chain(a().og_tween_timeline_call(h, name, at(position))),
    addLabel: (label: string, position?: unknown) => chain(a().og_tween_timeline_add_label(h, label, at(position))),
    // A timeline object nests by its handle, as GSAP's tl.add(child) does.
    add: (child: unknown, position?: unknown) => {
      const c = typeof child === "object" && child !== null && "tween" in child ? (child as { tween: number }).tween : child;
      return chain(a().og_tween_timeline_add(h, JSON.stringify(c), at(position)));
    },
    play: (from?: unknown) => chain(a().og_tween_timeline_play(h, at(from))),
    pause: (atTime?: unknown) => chain(a().og_tween_timeline_pause(h, at(atTime))),
    resume: () => chain(a().og_tween_timeline_resume(h)),
    reverse: (from?: unknown) => chain(a().og_tween_timeline_reverse(h, at(from))),
    restart: () => chain(a().og_tween_timeline_restart(h)),
    seek: (position: unknown) => chain(a().og_tween_timeline_seek(h, JSON.stringify(position))),
    progress: (value?: number) => a().og_tween_timeline_progress(h, value),
    time: () => a().og_tween_timeline_time(h),
    duration: () => a().og_tween_timeline_duration(h),
    timeScale: (value?: number) => a().og_tween_timeline_time_scale(h, value),
    isActive: () => a().og_tween_timeline_is_active(h),
    kill: () => a().og_tween_timeline_kill(h),
  });
  return Object.freeze(tl);
}

export const tilemap = Object.freeze({
  setTile: (world: WorldImpl, desc?: unknown) => parse(abi().og_tilemap_set_tile(world.id, json(desc))),
  fill: (world: WorldImpl, desc?: unknown) => parse(abi().og_tilemap_fill(world.id, json(desc))),
  create: (world: WorldImpl, desc?: unknown) => parse(abi().og_tilemap_create(world.id, json(desc))),
  resize: (world: WorldImpl, desc?: unknown) => parse(abi().og_tilemap_resize(world.id, json(desc))),
  neighbours: (world: WorldImpl, map: number, x: number, y: number) =>
    parse(abi().og_tilemap_neighbours(world.id, map, x, y)),
  getTile: (world: WorldImpl, entity: number, layer: string, x: number, y: number) =>
    abi().og_tilemap_get_tile(world.id, entity, layer, x, y),
  cellAt: (world: WorldImpl, entity: number, point: unknown) =>
    parse(abi().og_tilemap_cell_at(world.id, entity, JSON.stringify(point))),
  cellCenter: (world: WorldImpl, entity: number, x: number, y: number) =>
    parse(abi().og_tilemap_cell_center(world.id, entity, x, y)),
});

export const commands = Object.freeze({
  register: registerCommand,
  parse: (script: string) => parse(abi().og_commands_parse(script)),
  schedule: (world: WorldImpl, script: string) => parse(abi().og_commands_schedule(world.id, script)),
  help: (world: WorldImpl) => parse(abi().og_commands_help(world.id)),
});

export const physics = Object.freeze({
  castRay: (world: WorldImpl, desc?: unknown) => parse(abi().og_physics_cast_ray(world.id, json(desc))),
  castRayAll: (world: WorldImpl, desc?: unknown) => parse(abi().og_physics_cast_ray_all(world.id, json(desc))),
  projectPoint: (world: WorldImpl, desc?: unknown) => parse(abi().og_physics_project_point(world.id, json(desc))),
  castShape: (world: WorldImpl, desc?: unknown) => parse(abi().og_physics_cast_shape(world.id, json(desc))),
  overlap: (world: WorldImpl, desc?: unknown) => parse(abi().og_physics_overlap(world.id, json(desc))),
  applyForce: (world: WorldImpl, entity: number, force: unknown, point?: unknown) =>
    parse(abi().og_physics_apply_force(world.id, entity, JSON.stringify(force), json(point))),
  applyImpulse: (world: WorldImpl, entity: number, impulse: unknown, point?: unknown) =>
    parse(abi().og_physics_apply_impulse(world.id, entity, JSON.stringify(impulse), json(point))),
  applyTorque: (world: WorldImpl, entity: number, torque: unknown) =>
    parse(abi().og_physics_apply_torque(world.id, entity, JSON.stringify(torque))),
  applyTorqueImpulse: (world: WorldImpl, entity: number, torque: unknown) =>
    parse(abi().og_physics_apply_torque_impulse(world.id, entity, JSON.stringify(torque))),
  setMesh: (world: WorldImpl, mesh: number, desc?: unknown) => parse(abi().og_physics_set_mesh(world.id, mesh, json(desc))),
  ropePoints: (world: WorldImpl, entity: number) => parse(abi().og_physics_rope_points(world.id, entity)),
  clothPoints: (world: WorldImpl, entity: number) => parse(abi().og_physics_cloth_points(world.id, entity)),
  garmentPoints: (world: WorldImpl, entity: number) => parse(abi().og_physics_garment_points(world.id, entity)),
  fluidParticles: (world: WorldImpl, entity: number) => parse(abi().og_physics_fluid_particles(world.id, entity)),
  fluid3dParticles: (world: WorldImpl, entity: number) => parse(abi().og_physics_fluid3d_particles(world.id, entity)),
  water: (world: WorldImpl, point: unknown) => parse(abi().og_physics_water(world.id, JSON.stringify(point))),
  waterHeight: (world: WorldImpl, entity: number, x: number, z: number) =>
    parse(abi().og_physics_water_height(world.id, entity, x, z)),
  vehicleWheels: (world: WorldImpl, entity: number) => parse(abi().og_physics_vehicle_wheels(world.id, entity)),
  softBodyPoints: (world: WorldImpl, entity: number) => parse(abi().og_physics_soft_body_points(world.id, entity)),
  ragdollRigFromSkeleton: (world: WorldImpl, skeleton: number) =>
    parse(abi().og_physics_ragdoll_rig_from_skeleton(world.id, skeleton)),
  setRagdollRig: (world: WorldImpl, entity: number, desc?: unknown) =>
    parse(abi().og_physics_set_ragdoll_rig(world.id, entity, json(desc))),
  ragdollPose: (world: WorldImpl, entity: number) => parse(abi().og_physics_ragdoll_pose(world.id, entity)),
  stats: (world: WorldImpl) => parse(abi().og_physics_stats(world.id)),
});

export const audio = Object.freeze({
  outputLatency: (world: WorldImpl) => (checkAccess("read", "og.Audio"), abi().og_audio_output_latency(world.id)),
  playFeedback: (world: WorldImpl, sound: number, desc?: unknown) =>
    parse(abi().og_audio_play_feedback(world.id, sound, json(desc))),
  createSound: (world: WorldImpl, desc?: unknown) => {
    const d = desc !== null && typeof desc === "object" ? { ...(desc as Record<string, unknown>) } : desc;
    if (d !== null && typeof d === "object" && ArrayBuffer.isView((d as Record<string, unknown>).samples)) {
      const r = d as Record<string, unknown>;
      r.samples = Array.from(r.samples as ArrayLike<number>);
    }
    return parse(abi().og_audio_create_sound(world.id, json(d)));
  },
});

export const atmosphere = Object.freeze({
  timeOfDay: (world: WorldImpl) => abi().og_atmosphere_time_of_day(world.id),
});

export const ui = Object.freeze({
  scope: (world: WorldImpl, id: string): Api.UiScope => uiReactor(world).scope(id),
  configure: (world: WorldImpl, desc?: unknown) => parse(abi().og_ui_configure(world.id, json(desc))),
  filePicker: (world: WorldImpl): boolean => abi().og_ui_file_picker(world.id),
  viewport: (world: WorldImpl) => parse(abi().og_ui_viewport(world.id)),
  theme: (world: WorldImpl) => parse(abi().og_ui_theme(world.id)),
  frameDirection: (): "ltr" | "rtl" => abi().og_ui_frame_direction() as "ltr" | "rtl",
  frameCursor: (): Api.Cursor => abi().og_ui_frame_cursor() as Api.Cursor,
  scrollIntoView: (path: string, center: boolean): void => abi().og_ui_scroll_into_view(path, center),
  feedback: (event: string, rise: number) => abi().og_ui_feedback(event, rise),
  setHaptics: (intensity: number) => abi().og_ui_set_haptics(intensity),
});
