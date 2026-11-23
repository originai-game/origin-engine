// Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate.
// Every host member as a game reaches it: og's namespaces, the interfaces'
// classes over their handles, and the resources' methods. Each is its
// binding symbol's call (src/generated/abi.ts) with the wire's conversions;
// a member the IDL marks `custom` calls src/runtime/custom.ts instead.

import { abi } from "../runtime/host.ts";
import { custom } from "../runtime/custom.ts";
import { json } from "../runtime/result.ts";

export class SaveEdit {
  readonly #handle: number;
  constructor(handle: number) {
    this.#handle = handle;
  }
  /** The handle the host knows it by, for the members src/runtime/custom.ts implements. */
  static handleOf(o: SaveEdit): number {
    return o.#handle;
  }
  get version(): number {
    return abi().og_save_edit_get_version(this.#handle);
  }
  types(): unknown {
    return JSON.parse(abi().og_save_edit_types(this.#handle)) as unknown;
  }
  has(name: string): unknown {
    return custom.SaveEdit.has(this, name);
  }
  records(name: string): unknown {
    return JSON.parse(abi().og_save_edit_records(this.#handle, name)) as unknown;
  }
  setRecords(name: string, desc?: unknown): unknown {
    return JSON.parse(abi().og_save_edit_set_records(this.#handle, name, json(desc))) as unknown;
  }
  transform(name: string, map: Function, desc?: unknown): unknown {
    return custom.SaveEdit.transform(this, name, map, desc);
  }
  split(name: string, map: Function): unknown {
    return custom.SaveEdit.split(this, name, map);
  }
  renameType(from: string, to: string): unknown {
    return JSON.parse(abi().og_save_edit_rename_type(this.#handle, from, to)) as unknown;
  }
  renameField(name: string, from: string, to: string): unknown {
    return JSON.parse(abi().og_save_edit_rename_field(this.#handle, name, from, to)) as unknown;
  }
  dropType(name: string): unknown {
    return JSON.parse(abi().og_save_edit_drop_type(this.#handle, name)) as unknown;
  }
  spawn(desc?: unknown): unknown {
    return JSON.parse(abi().og_save_edit_spawn(this.#handle, json(desc))) as unknown;
  }
  text(): unknown {
    return JSON.parse(abi().og_save_edit_text(this.#handle)) as unknown;
  }
  close(): void {
    return abi().og_save_edit_close(this.#handle);
  }
}

export class SpatialIndex {
  readonly #handle: number;
  constructor(handle: number) {
    this.#handle = handle;
  }
  /** The handle the host knows it by, for the members src/runtime/custom.ts implements. */
  static handleOf(o: SpatialIndex): number {
    return o.#handle;
  }
  get count(): number {
    return abi().og_spatial_index_get_count(this.#handle);
  }
  within(center: unknown, radius: number): unknown {
    return JSON.parse(abi().og_spatial_index_within(this.#handle, JSON.stringify(center), radius)) as unknown;
  }
  nearest(center: unknown, count: number): unknown {
    return JSON.parse(abi().og_spatial_index_nearest(this.#handle, JSON.stringify(center), count)) as unknown;
  }
}

export class World {
  /** Its handle, which the host functions taking one read. */
  readonly id: number;
  constructor(handle: number) {
    this.id = handle;
  }
  /** The handle the host knows it by, for the members src/runtime/custom.ts implements. */
  static handleOf(o: World): number {
    return o.id;
  }
  get tick(): number {
    return abi().og_world_get_tick(this.id);
  }
  spawn(desc?: unknown): unknown {
    return custom.World.spawn(this, desc);
  }
  despawn(entity: number): unknown {
    return custom.World.despawn(this, entity);
  }
  alive(entity: number): boolean {
    return abi().og_world_alive(this.id, entity);
  }
  has(entity: number, component: { readonly name: string }): boolean {
    return abi().og_world_has(this.id, entity, component.name);
  }
  insert(entity: number, component: { readonly name: string }, value?: unknown): unknown {
    return custom.World.insert(this, entity, component, value);
  }
  remove(entity: number, component: { readonly name: string }): unknown {
    return custom.World.remove(this, entity, component);
  }
  get(entity: number, component: { readonly name: string }): unknown {
    return custom.World.get(this, entity, component);
  }
  set(entity: number, component: { readonly name: string }, value: unknown): unknown {
    return custom.World.set(this, entity, component, value);
  }
  resource(resource: { readonly name: string }): unknown {
    return custom.World.resource(this, resource);
  }
  children(entity: number): unknown {
    return JSON.parse(abi().og_world_children(this.id, entity)) as unknown;
  }
  removed(component: { readonly name: string }): unknown {
    return custom.World.removed(this, component);
  }
  spatial(desc?: unknown): unknown {
    return custom.World.spatial(this, desc);
  }
  state(name: string): unknown {
    return JSON.parse(abi().og_world_state(this.id, name)) as unknown;
  }
  setState(name: string, value: string): unknown {
    return JSON.parse(abi().og_world_set_state(this.id, name, value)) as unknown;
  }
  find(name: string): unknown {
    return JSON.parse(abi().og_world_find(this.id, name)) as unknown;
  }
  prefabOf(entity: number): unknown {
    return JSON.parse(abi().og_world_prefab_of(this.id, entity)) as unknown;
  }
  nameOf(entity: number): unknown {
    return JSON.parse(abi().og_world_name_of(this.id, entity)) as unknown;
  }
  query(desc?: unknown): unknown {
    return custom.World.query(this, desc);
  }
  events(event: { readonly name: string }): unknown {
    return custom.World.events(this, event);
  }
  emit(event: { readonly name: string }, payload: unknown): unknown {
    return custom.World.emit(this, event, payload);
  }
  instantiate(asset: number, desc?: unknown): unknown {
    return custom.World.instantiate(this, asset, desc);
  }
  snapshot(desc?: unknown): unknown {
    return custom.World.snapshot(this, desc);
  }
  openSave(save: string): unknown {
    return custom.World.openSave(this, save);
  }
  restore(save: string, desc?: unknown): unknown {
    return custom.World.restore(this, save, desc);
  }
}

export class RngStream {
  readonly #handle: number;
  constructor(handle: number) {
    this.#handle = handle;
  }
  /** The handle the host knows it by, for the members src/runtime/custom.ts implements. */
  static handleOf(o: RngStream): number {
    return o.#handle;
  }
  get seed(): number {
    return abi().og_rng_stream_get_seed(this.#handle);
  }
  next(): number {
    return abi().og_rng_stream_next(this.#handle);
  }
  range(low: number, high: number): number {
    return abi().og_rng_stream_range(this.#handle, low, high);
  }
  integer(low: number, high: number): number {
    return abi().og_rng_stream_integer(this.#handle, low, high);
  }
  chance(p: number): boolean {
    return abi().og_rng_stream_chance(this.#handle, p);
  }
  weighted(weights: unknown): number {
    return abi().og_rng_stream_weighted(this.#handle, JSON.stringify(weights ?? []));
  }
  permutation(n: number): unknown {
    return JSON.parse(abi().og_rng_stream_permutation(this.#handle, n)) as unknown;
  }
  fork(label: string): RngStream {
    const r = abi().og_rng_stream_fork(this.#handle, label);
    return r === this.#handle ? this : new RngStream(r);
  }
}

export class TweenTimeline {
  readonly #handle: number;
  constructor(handle: number) {
    this.#handle = handle;
  }
  /** The handle the host knows it by, for the members src/runtime/custom.ts implements. */
  static handleOf(o: TweenTimeline): number {
    return o.#handle;
  }
  get tween(): number {
    return abi().og_tween_timeline_get_tween(this.#handle);
  }
  get error(): unknown {
    return JSON.parse(abi().og_tween_timeline_get_error(this.#handle)) as unknown;
  }
  to(targets: unknown, vars?: unknown, position?: unknown): TweenTimeline {
    const r = abi().og_tween_timeline_to(this.#handle, JSON.stringify(targets), json(vars), json(position));
    return r === this.#handle ? this : new TweenTimeline(r);
  }
  from(targets: unknown, vars?: unknown, position?: unknown): TweenTimeline {
    const r = abi().og_tween_timeline_from(this.#handle, JSON.stringify(targets), json(vars), json(position));
    return r === this.#handle ? this : new TweenTimeline(r);
  }
  fromTo(targets: unknown, vars?: unknown, position?: unknown): TweenTimeline {
    const r = abi().og_tween_timeline_from_to(this.#handle, JSON.stringify(targets), json(vars), json(position));
    return r === this.#handle ? this : new TweenTimeline(r);
  }
  set(targets: unknown, vars?: unknown, position?: unknown): TweenTimeline {
    const r = abi().og_tween_timeline_set(this.#handle, JSON.stringify(targets), json(vars), json(position));
    return r === this.#handle ? this : new TweenTimeline(r);
  }
  call(name: string, position?: unknown): TweenTimeline {
    const r = abi().og_tween_timeline_call(this.#handle, name, json(position));
    return r === this.#handle ? this : new TweenTimeline(r);
  }
  addLabel(label: string, position?: unknown): TweenTimeline {
    const r = abi().og_tween_timeline_add_label(this.#handle, label, json(position));
    return r === this.#handle ? this : new TweenTimeline(r);
  }
  add(child: unknown, position?: unknown): unknown {
    return custom.TweenTimeline.add(this, child, position);
  }
  play(from?: unknown): TweenTimeline {
    const r = abi().og_tween_timeline_play(this.#handle, json(from));
    return r === this.#handle ? this : new TweenTimeline(r);
  }
  pause(at?: unknown): TweenTimeline {
    const r = abi().og_tween_timeline_pause(this.#handle, json(at));
    return r === this.#handle ? this : new TweenTimeline(r);
  }
  resume(): TweenTimeline {
    const r = abi().og_tween_timeline_resume(this.#handle);
    return r === this.#handle ? this : new TweenTimeline(r);
  }
  reverse(from?: unknown): TweenTimeline {
    const r = abi().og_tween_timeline_reverse(this.#handle, json(from));
    return r === this.#handle ? this : new TweenTimeline(r);
  }
  restart(): TweenTimeline {
    const r = abi().og_tween_timeline_restart(this.#handle);
    return r === this.#handle ? this : new TweenTimeline(r);
  }
  seek(position: unknown): TweenTimeline {
    const r = abi().og_tween_timeline_seek(this.#handle, JSON.stringify(position));
    return r === this.#handle ? this : new TweenTimeline(r);
  }
  progress(value?: number): number {
    return abi().og_tween_timeline_progress(this.#handle, value);
  }
  time(): number {
    return abi().og_tween_timeline_time(this.#handle);
  }
  duration(): number {
    return abi().og_tween_timeline_duration(this.#handle);
  }
  timeScale(value?: number): number {
    return abi().og_tween_timeline_time_scale(this.#handle, value);
  }
  isActive(): boolean {
    return abi().og_tween_timeline_is_active(this.#handle);
  }
  kill(): boolean {
    return abi().og_tween_timeline_kill(this.#handle);
  }
}

export class RuntimeProbe {
  readonly #handle: number;
  constructor(handle: number) {
    this.#handle = handle;
  }
  /** The handle the host knows it by, for the members src/runtime/custom.ts implements. */
  static handleOf(o: RuntimeProbe): number {
    return o.#handle;
  }
  get version(): number {
    return abi().og_runtime_probe_get_version(this.#handle);
  }
  tick(): number {
    return abi().og_runtime_probe_tick(this.#handle);
  }
  state(): unknown {
    return JSON.parse(abi().og_runtime_probe_state(this.#handle)) as unknown;
  }
  advance(steps: number): unknown {
    return custom.RuntimeProbe.advance(this, steps);
  }
  pause(): boolean {
    return abi().og_runtime_probe_pause(this.#handle);
  }
  resume(): boolean {
    return abi().og_runtime_probe_resume(this.#handle);
  }
  input(event: unknown): boolean {
    return abi().og_runtime_probe_input(this.#handle, JSON.stringify(event));
  }
  accepted(): number {
    return abi().og_runtime_probe_accepted(this.#handle);
  }
  pick(x: number, y: number): unknown {
    return JSON.parse(abi().og_runtime_probe_pick(this.#handle, x, y)) as unknown;
  }
  entity(id: string): unknown {
    return JSON.parse(abi().og_runtime_probe_entity(this.#handle, id)) as unknown;
  }
  bounds(id: string): unknown {
    return JSON.parse(abi().og_runtime_probe_bounds(this.#handle, id)) as unknown;
  }
  patch(id: string, parameter: string, value: unknown): boolean {
    return abi().og_runtime_probe_patch(this.#handle, id, parameter, JSON.stringify(value ?? null));
  }
  run(program: unknown): unknown {
    return custom.RuntimeProbe.run(this, program);
  }
}

export class EngineProbe extends RuntimeProbe {
  readonly #handle: number;
  constructor(handle: number) {
    super(handle);
    this.#handle = handle;
  }
  /** The handle the host knows it by, for the members src/runtime/custom.ts implements. */
  static override handleOf(o: EngineProbe): number {
    return o.#handle;
  }
  hello(): unknown {
    return JSON.parse(abi().og_engine_probe_hello(this.#handle)) as unknown;
  }
  capture(width?: number, height?: number, fit?: number, region?: unknown): unknown {
    return JSON.parse(abi().og_engine_probe_capture(this.#handle, width, height, fit, json(region))) as unknown;
  }
  captureViews(views: unknown, width?: number, height?: number, fit?: number): unknown {
    return JSON.parse(abi().og_engine_probe_capture_views(this.#handle, JSON.stringify(views ?? []), width, height, fit)) as unknown;
  }
  visible(id: string): unknown {
    return JSON.parse(abi().og_engine_probe_visible(this.#handle, id)) as unknown;
  }
  marks(desc?: unknown): unknown {
    return JSON.parse(abi().og_engine_probe_marks(this.#handle, json(desc))) as unknown;
  }
  timeline(): unknown {
    return JSON.parse(abi().og_engine_probe_timeline(this.#handle)) as unknown;
  }
  sweep(desc?: unknown): unknown {
    return JSON.parse(abi().og_engine_probe_sweep(this.#handle, json(desc))) as unknown;
  }
  reload(): unknown {
    return JSON.parse(abi().og_engine_probe_reload(this.#handle)) as unknown;
  }
  snapshot(): unknown {
    return JSON.parse(abi().og_engine_probe_snapshot(this.#handle)) as unknown;
  }
  restore(id: string): unknown {
    return custom.EngineProbe.restore(this, id);
  }
  dump(desc?: unknown): unknown {
    return JSON.parse(abi().og_engine_probe_dump(this.#handle, json(desc))) as unknown;
  }
  commands(script: string): unknown {
    return JSON.parse(abi().og_engine_probe_commands(this.#handle, script)) as unknown;
  }
  listen(seconds?: number): unknown {
    return JSON.parse(abi().og_engine_probe_listen(this.#handle, seconds)) as unknown;
  }
  memory(): unknown {
    return JSON.parse(abi().og_engine_probe_memory(this.#handle)) as unknown;
  }
  describe(joints?: string): unknown {
    return JSON.parse(abi().og_engine_probe_describe(this.#handle, joints)) as unknown;
  }
  expect(what: string): unknown {
    return JSON.parse(abi().og_engine_probe_expect(this.#handle, what)) as unknown;
  }
  expectations(): unknown {
    return JSON.parse(abi().og_engine_probe_expectations(this.#handle)) as unknown;
  }
  assertions(take?: boolean): unknown {
    return JSON.parse(abi().og_engine_probe_assertions(this.#handle, take)) as unknown;
  }
  assertionEvents(): unknown {
    return JSON.parse(abi().og_engine_probe_assertion_events(this.#handle)) as unknown;
  }
  observe(records?: unknown, types?: boolean): unknown {
    return JSON.parse(abi().og_engine_probe_observe(this.#handle, json(records), types)) as unknown;
  }
  saveScene(write?: boolean, spawned?: boolean): unknown {
    return JSON.parse(abi().og_engine_probe_save_scene(this.#handle, write, spawned)) as unknown;
  }
  pickUi(x: number, y: number): unknown {
    return JSON.parse(abi().og_engine_probe_pick_ui(this.#handle, x, y)) as unknown;
  }
}

/** The methods each engine resource's object carries beside its fields, made for one world. */
export const resourceMethods: Readonly<Record<string, (world: number) => object>> = Object.freeze({
  "og.Rng": (world: number) => ({
    next: (): unknown => custom.Rng.next(world),
    range: (low: number, high: number): unknown => custom.Rng.range(world, low, high),
    integer: (low: number, high: number): unknown => custom.Rng.integer(world, low, high),
    chance: (p: number): unknown => custom.Rng.chance(world, p),
    weighted: (weights: unknown): unknown => custom.Rng.weighted(world, weights),
    permutation: (n: number): unknown => custom.Rng.permutation(world, n),
    fork: (label: string): unknown => custom.Rng.fork(world, label),
    reset: (): unknown => custom.Rng.reset(world),
  }),
  "og.Input": (world: number) => ({
    down: (key: string): boolean => abi().og_input_down(world, key),
    justPressed: (key: string): boolean => abi().og_input_just_pressed(world, key),
    justReleased: (key: string): boolean => abi().og_input_just_released(world, key),
    axis: (negative: string, positive: string): number => abi().og_input_axis(world, negative, positive),
    axis2: (left: string, right: string, down: string, up: string): unknown => JSON.parse(abi().og_input_axis2(world, left, right, down, up)) as unknown,
    buttonDown: (button: string): boolean => abi().og_input_button_down(world, button),
    buttonJustPressed: (button: string): boolean => abi().og_input_button_just_pressed(world, button),
    buttonJustReleased: (button: string): boolean => abi().og_input_button_just_released(world, button),
  }),
  "og.Pointers": (world: number) => ({
    down: (id: number): boolean => abi().og_pointers_down(world, id),
    justPressed: (id: number): boolean => abi().og_pointers_just_pressed(world, id),
    justReleased: (id: number): boolean => abi().og_pointers_just_released(world, id),
    position: (id: number): unknown => JSON.parse(abi().og_pointers_position(world, id)) as unknown,
    start: (id: number): unknown => JSON.parse(abi().og_pointers_start(world, id)) as unknown,
    count: (): number => abi().og_pointers_count(world),
  }),
  "og.Gamepad": (world: number) => ({
    connected: (pad?: number): boolean => abi().og_gamepad_connected(world, pad),
    down: (button: string, pad?: number): boolean => abi().og_gamepad_down(world, button, pad),
    justPressed: (button: string, pad?: number): boolean => abi().og_gamepad_just_pressed(world, button, pad),
    justReleased: (button: string, pad?: number): boolean => abi().og_gamepad_just_released(world, button, pad),
    axis: (axis: string, pad?: number): number => abi().og_gamepad_axis(world, axis, pad),
  }),
  "og.Camera": (world: number) => ({
    lookAt: (target: unknown): void => abi().og_camera_look_at(world, JSON.stringify(target)),
    worldToScreen: (point: unknown): unknown => JSON.parse(abi().og_camera_world_to_screen(world, JSON.stringify(point))) as unknown,
    screenToRay: (x: number, y: number): unknown => JSON.parse(abi().og_camera_screen_to_ray(world, x, y)) as unknown,
  }),
});

/** The package's own functions (`defineComponent`). */
export const top = Object.freeze({
  defineComponent: (name: string, fields: unknown): unknown => custom.reflect.defineComponent(name, fields),
  defineResource: (name: string, fields: unknown): unknown => custom.reflect.defineResource(name, fields),
  defineEvent: (name: string, fields: unknown): unknown => custom.reflect.defineEvent(name, fields),
  defineHook: (name: string, component: { readonly name: string }, desc: unknown, run: Function): unknown => custom.system.defineHook(name, component, desc, run),
  defineSystem: (name: string, desc: unknown, run: Function): unknown => custom.system.defineSystem(name, desc, run),
});

/** og's functions and namespaces. */
export const og = Object.freeze({
  game: (desc?: unknown): unknown => custom.game.game(desc),
  transform: Object.freeze({
    compose: (position: unknown, quaternion: unknown, scale: unknown): unknown => JSON.parse(abi().og_transform_compose(JSON.stringify(position), JSON.stringify(quaternion), JSON.stringify(scale))) as unknown,
    decompose: (matrix: unknown): unknown => JSON.parse(abi().og_transform_decompose(JSON.stringify(matrix))) as unknown,
    matrixWorld: (world: { readonly id: number }, entity: number): unknown => JSON.parse(abi().og_transform_matrix_world(world.id, entity)) as unknown,
    lookAt: (eye: unknown, target: unknown, up?: unknown): unknown => JSON.parse(abi().og_transform_look_at(JSON.stringify(eye), JSON.stringify(target), json(up))) as unknown,
    face: (position: unknown, target: unknown, up?: unknown): unknown => JSON.parse(abi().og_transform_face(JSON.stringify(position), JSON.stringify(target), json(up))) as unknown,
    forward: (quaternion: unknown): unknown => JSON.parse(abi().og_transform_forward(JSON.stringify(quaternion))) as unknown,
    right: (quaternion: unknown): unknown => JSON.parse(abi().og_transform_right(JSON.stringify(quaternion))) as unknown,
    up: (quaternion: unknown): unknown => JSON.parse(abi().og_transform_up(JSON.stringify(quaternion))) as unknown,
    cameraForward: (quaternion: unknown): unknown => JSON.parse(abi().og_transform_camera_forward(JSON.stringify(quaternion))) as unknown,
    cameraRight: (quaternion: unknown): unknown => JSON.parse(abi().og_transform_camera_right(JSON.stringify(quaternion))) as unknown,
    heading: (quaternion: unknown): number => abi().og_transform_heading(JSON.stringify(quaternion)),
    fromHeading: (heading: number): unknown => JSON.parse(abi().og_transform_from_heading(heading)) as unknown,
    face2d: (position: unknown, target: unknown): unknown => JSON.parse(abi().og_transform_face2d(JSON.stringify(position), JSON.stringify(target))) as unknown,
    forward2d: (quaternion: unknown): unknown => JSON.parse(abi().og_transform_forward2d(JSON.stringify(quaternion))) as unknown,
    heading2d: (quaternion: unknown): number => abi().og_transform_heading2d(JSON.stringify(quaternion)),
    fromHeading2d: (heading: number): unknown => JSON.parse(abi().og_transform_from_heading2d(heading)) as unknown,
    fromAxisAngle: (axis: unknown, angle: number): unknown => JSON.parse(abi().og_transform_from_axis_angle(JSON.stringify(axis), angle)) as unknown,
  }),
  time: Object.freeze({
    setScale: (world: { readonly id: number }, scale: number): unknown => JSON.parse(abi().og_time_set_scale(world.id, scale)) as unknown,
    hitStop: (world: { readonly id: number }, ticks: number): unknown => JSON.parse(abi().og_time_hit_stop(world.id, ticks)) as unknown,
    setGroupScale: (world: { readonly id: number }, group: number, scale: number): unknown => JSON.parse(abi().og_time_set_group_scale(world.id, group, scale)) as unknown,
    groupDt: (world: { readonly id: number }, entity: number): number => abi().og_time_group_dt(world.id, entity),
  }),
  procgen: Object.freeze({
    noise: (desc: unknown, x: number, y: number, z: number): number => abi().og_procgen_noise(JSON.stringify(desc ?? {}), x, y, z),
    noiseGrid: (desc?: unknown): unknown => JSON.parse(abi().og_procgen_noise_grid(json(desc))) as unknown,
    poissonDisk: (desc?: unknown): unknown => JSON.parse(abi().og_procgen_poisson_disk(json(desc))) as unknown,
    bsp: (desc?: unknown): unknown => JSON.parse(abi().og_procgen_bsp(json(desc))) as unknown,
    wfc: (desc?: unknown): unknown => JSON.parse(abi().og_procgen_wfc(json(desc))) as unknown,
  }),
  input: Object.freeze({
    intent: (world: { readonly id: number }, event: { readonly name: string }, payload: unknown): unknown => JSON.parse(abi().og_input_intent(world.id, event.name, JSON.stringify(payload))) as unknown,
    takeIntents: (world: { readonly id: number }): unknown => JSON.parse(abi().og_input_take_intents(world.id)) as unknown,
    replayIntent: (world: { readonly id: number }, intent: unknown): unknown => JSON.parse(abi().og_input_replay_intent(world.id, JSON.stringify(intent))) as unknown,
    defineAction: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_input_define_action(world.id, json(desc))) as unknown,
    actions: (world: { readonly id: number }): unknown => JSON.parse(abi().og_input_actions(world.id)) as unknown,
    action: (world: { readonly id: number }, name: string): unknown => JSON.parse(abi().og_input_action(world.id, name)) as unknown,
    consume: (world: { readonly id: number }, name: string): boolean => abi().og_input_consume(world.id, name),
    isActionDown: (world: { readonly id: number }, name: string): boolean => abi().og_input_is_action_down(world.id, name),
    isActionJustPressed: (world: { readonly id: number }, name: string): boolean => abi().og_input_is_action_just_pressed(world.id, name),
    isActionJustReleased: (world: { readonly id: number }, name: string): boolean => abi().og_input_is_action_just_released(world.id, name),
    isActionHeld: (world: { readonly id: number }, name: string): boolean => abi().og_input_is_action_held(world.id, name),
    actionValue: (world: { readonly id: number }, name: string): number => abi().og_input_action_value(world.id, name),
    actionVector: (world: { readonly id: number }, name: string): unknown => JSON.parse(abi().og_input_action_vector(world.id, name)) as unknown,
    moveRelative: (world: { readonly id: number }, name: string, camera: unknown): unknown => JSON.parse(abi().og_input_move_relative(world.id, name, JSON.stringify(camera))) as unknown,
    setActionMaps: (world: { readonly id: number }, actionMaps: unknown): void => abi().og_input_set_action_maps(world.id, JSON.stringify(actionMaps ?? [])),
    actionMaps: (world: { readonly id: number }): unknown => JSON.parse(abi().og_input_action_maps(world.id)) as unknown,
    rebind: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_input_rebind(world.id, json(desc))) as unknown,
    conflicts: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_input_conflicts(world.id, json(desc))) as unknown,
    resetBindings: (world: { readonly id: number }, action?: string): unknown => JSON.parse(abi().og_input_reset_bindings(world.id, action)) as unknown,
    setToggle: (world: { readonly id: number }, action: string, on: boolean): unknown => JSON.parse(abi().og_input_set_toggle(world.id, action, on)) as unknown,
    saveBindings: (world: { readonly id: number }): string => abi().og_input_save_bindings(world.id),
    loadBindings: (world: { readonly id: number }, text: string): unknown => JSON.parse(abi().og_input_load_bindings(world.id, text)) as unknown,
    startCapture: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_input_start_capture(world.id, json(desc))) as unknown,
    capture: (world: { readonly id: number }): unknown => JSON.parse(abi().og_input_capture(world.id)) as unknown,
    cancelCapture: (world: { readonly id: number }): void => abi().og_input_cancel_capture(world.id),
    applyCapture: (world: { readonly id: number }, policy?: string): unknown => JSON.parse(abi().og_input_apply_capture(world.id, policy)) as unknown,
    glyph: (world: { readonly id: number }, action: string, device?: string): unknown => JSON.parse(abi().og_input_glyph(world.id, action, device)) as unknown,
    glyphs: (world: { readonly id: number }, action: string, device?: string): unknown => JSON.parse(abi().og_input_glyphs(world.id, action, device)) as unknown,
    controlGlyph: (control: string, device?: string): unknown => JSON.parse(abi().og_input_control_glyph(control, device)) as unknown,
    device: (world: { readonly id: number }): string => abi().og_input_device(world.id),
    padDevice: (world: { readonly id: number }, pad: number): unknown => JSON.parse(abi().og_input_pad_device(world.id, pad)) as unknown,
    setDeadzones: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_input_set_deadzones(world.id, json(desc))) as unknown,
    setSensitivity: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_input_set_sensitivity(world.id, json(desc))) as unknown,
    sensitivity: (world: { readonly id: number }, action: string, device?: string): unknown => JSON.parse(abi().og_input_sensitivity(world.id, action, device)) as unknown,
    setKeyboardLayout: (labels: unknown): void => abi().og_input_set_keyboard_layout(JSON.stringify(labels ?? [])),
    keyboardLayout: (): unknown => JSON.parse(abi().og_input_keyboard_layout()) as unknown,
    setMouse: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_input_set_mouse(world.id, json(desc))) as unknown,
    mouse: (world: { readonly id: number }): unknown => JSON.parse(abi().og_input_mouse(world.id)) as unknown,
    rumble: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_input_rumble(world.id, json(desc))) as unknown,
  }),
  camera: Object.freeze({
    shake: (world: { readonly id: number }, trauma: number): unknown => JSON.parse(abi().og_camera_shake(world.id, trauma)) as unknown,
    punch: (world: { readonly id: number }, amount: number): unknown => JSON.parse(abi().og_camera_punch(world.id, amount)) as unknown,
    kick: (world: { readonly id: number }, direction: unknown, distance: number): unknown => JSON.parse(abi().og_camera_kick(world.id, JSON.stringify(direction), distance)) as unknown,
  }),
  render: Object.freeze({
    snap: (world: { readonly id: number }, entity: number): unknown => JSON.parse(abi().og_render_snap(world.id, entity)) as unknown,
    snapCamera: (world: { readonly id: number }): void => abi().og_render_snap_camera(world.id),
    compile: (world: { readonly id: number }): void => abi().og_render_compile(world.id),
    reveal: (world: { readonly id: number }, entity: number, seconds: number): unknown => JSON.parse(abi().og_render_reveal(world.id, entity, seconds)) as unknown,
    ribbon: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_render_ribbon(world.id, json(desc))) as unknown,
    sprite: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_render_sprite(world.id, json(desc))) as unknown,
    floatingText: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_render_floating_text(world.id, json(desc))) as unknown,
    measureText: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_render_measure_text(world.id, json(desc))) as unknown,
    boxGeometry: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_render_box_geometry(world.id, json(desc))) as unknown,
    planeGeometry: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_render_plane_geometry(world.id, json(desc))) as unknown,
    sphereGeometry: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_render_sphere_geometry(world.id, json(desc))) as unknown,
    standardMaterial: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_render_standard_material(world.id, json(desc))) as unknown,
    updateMaterial: (world: { readonly id: number }, material: number, desc?: unknown): unknown => JSON.parse(abi().og_render_update_material(world.id, material, json(desc))) as unknown,
    shaderMaterial: (world: { readonly id: number }, shader: number, desc?: unknown): unknown => JSON.parse(abi().og_render_shader_material(world.id, shader, json(desc))) as unknown,
    setMaterialShader: (world: { readonly id: number }, material: number, shader: unknown): unknown => JSON.parse(abi().og_render_set_material_shader(world.id, material, JSON.stringify(shader ?? null))) as unknown,
    setShaderParam: (world: { readonly id: number }, material: number, name: string, value: unknown): unknown => JSON.parse(abi().og_render_set_shader_param(world.id, material, name, JSON.stringify(value ?? []))) as unknown,
    applyLook: (world: { readonly id: number }, look: number): unknown => JSON.parse(abi().og_render_apply_look(world.id, look)) as unknown,
    applyQuality: (world: { readonly id: number }, preset: string): unknown => JSON.parse(abi().og_render_apply_quality(world.id, preset)) as unknown,
    setDisplay: (world: { readonly id: number }, settings?: unknown): unknown => JSON.parse(abi().og_render_set_display(world.id, json(settings))) as unknown,
    resetDisplay: (world: { readonly id: number }): unknown => JSON.parse(abi().og_render_reset_display(world.id)) as unknown,
    display: (world: { readonly id: number }): unknown => JSON.parse(abi().og_render_display(world.id)) as unknown,
    look: (world: { readonly id: number }): unknown => JSON.parse(abi().og_render_look(world.id)) as unknown,
    capabilities: (world: { readonly id: number }): unknown => JSON.parse(abi().og_render_capabilities(world.id)) as unknown,
    text: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_render_text(world.id, json(desc))) as unknown,
    bufferGeometry: (world: { readonly id: number }, desc?: unknown): unknown => custom.render.bufferGeometry(world, desc),
    updateGeometry: (world: { readonly id: number }, geometry: number, desc?: unknown): unknown => custom.render.updateGeometry(world, geometry, desc),
    geometryInfo: (world: { readonly id: number }, geometry: number): unknown => JSON.parse(abi().og_render_geometry_info(world.id, geometry)) as unknown,
    cylinderGeometry: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_render_cylinder_geometry(world.id, json(desc))) as unknown,
    coneGeometry: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_render_cone_geometry(world.id, json(desc))) as unknown,
    capsuleGeometry: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_render_capsule_geometry(world.id, json(desc))) as unknown,
    torusGeometry: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_render_torus_geometry(world.id, json(desc))) as unknown,
    ringGeometry: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_render_ring_geometry(world.id, json(desc))) as unknown,
    circleGeometry: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_render_circle_geometry(world.id, json(desc))) as unknown,
    tubeGeometry: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_render_tube_geometry(world.id, json(desc))) as unknown,
    shapeGeometry: (world: { readonly id: number }, desc?: unknown): unknown => custom.render.shapeGeometry(world, desc),
    extrudeGeometry: (world: { readonly id: number }, desc?: unknown): unknown => custom.render.extrudeGeometry(world, desc),
    latheGeometry: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_render_lathe_geometry(world.id, json(desc))) as unknown,
    polyhedronGeometry: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_render_polyhedron_geometry(world.id, json(desc))) as unknown,
    icosahedronGeometry: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_render_icosahedron_geometry(world.id, json(desc))) as unknown,
    dodecahedronGeometry: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_render_dodecahedron_geometry(world.id, json(desc))) as unknown,
    octahedronGeometry: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_render_octahedron_geometry(world.id, json(desc))) as unknown,
    tetrahedronGeometry: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_render_tetrahedron_geometry(world.id, json(desc))) as unknown,
    sweepGeometry: (world: { readonly id: number }, desc?: unknown): unknown => custom.render.sweepGeometry(world, desc),
    csgGeometry: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_render_csg_geometry(world.id, json(desc))) as unknown,
    mergeGeometries: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_render_merge_geometries(world.id, json(desc))) as unknown,
    lines: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_render_lines(world.id, json(desc))) as unknown,
    points: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_render_points(world.id, json(desc))) as unknown,
    raycast: (world: { readonly id: number }, ray: unknown, desc?: unknown): unknown => JSON.parse(abi().og_render_raycast(world.id, JSON.stringify(ray), json(desc))) as unknown,
    light2dAt: (world: { readonly id: number }, point: unknown, layers?: number): unknown => JSON.parse(abi().og_render_light2d_at(world.id, JSON.stringify(point), layers)) as unknown,
    renderTarget: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_render_render_target(world.id, json(desc))) as unknown,
    setPostParam: (world: { readonly id: number }, post: number, name: string, value: unknown): unknown => JSON.parse(abi().og_render_set_post_param(world.id, post, name, JSON.stringify(value ?? []))) as unknown,
    postErrors: (world: { readonly id: number }): unknown => JSON.parse(abi().og_render_post_errors(world.id)) as unknown,
    bakedLighting: (world: { readonly id: number }): unknown => JSON.parse(abi().og_render_baked_lighting(world.id)) as unknown,
  }),
  load: Object.freeze({
    bundle: (world: { readonly id: number }, path: string): unknown => JSON.parse(abi().og_load_bundle(world.id, path)) as unknown,
    bundleInfo: (world: { readonly id: number }, asset: number): unknown => JSON.parse(abi().og_load_bundle_info(world.id, asset)) as unknown,
    progress: (world: { readonly id: number }, asset: number): unknown => JSON.parse(abi().og_load_progress(world.id, asset)) as unknown,
    defaultBundle: (world: { readonly id: number }): unknown => JSON.parse(abi().og_load_default_bundle(world.id)) as unknown,
    gltf: (world: { readonly id: number }, path: string, desc?: unknown): unknown => JSON.parse(abi().og_load_gltf(world.id, path, json(desc))) as unknown,
    scene: (world: { readonly id: number }, path: string): unknown => JSON.parse(abi().og_load_scene(world.id, path)) as unknown,
    prefab: (world: { readonly id: number }, path: string): unknown => JSON.parse(abi().og_load_prefab(world.id, path)) as unknown,
    font: (world: { readonly id: number }, path: string): unknown => JSON.parse(abi().og_load_font(world.id, path)) as unknown,
    fontInfo: (world: { readonly id: number }, font: number): unknown => JSON.parse(abi().og_load_font_info(world.id, font)) as unknown,
    sound: (world: { readonly id: number }, path: string): unknown => JSON.parse(abi().og_load_sound(world.id, path)) as unknown,
    lut: (world: { readonly id: number }, path: string): unknown => JSON.parse(abi().og_load_lut(world.id, path)) as unknown,
    environment: (world: { readonly id: number }, path: string): unknown => JSON.parse(abi().og_load_environment(world.id, path)) as unknown,
    environmentSun: (world: { readonly id: number }, map: number, rotation?: number, intensity?: number): unknown => JSON.parse(abi().og_load_environment_sun(world.id, map, rotation, intensity)) as unknown,
    emitter: (world: { readonly id: number }, path: string): unknown => JSON.parse(abi().og_load_emitter(world.id, path)) as unknown,
    effect: (world: { readonly id: number }, path: string): unknown => JSON.parse(abi().og_load_effect(world.id, path)) as unknown,
    shader: (world: { readonly id: number }, path: string): unknown => JSON.parse(abi().og_load_shader(world.id, path)) as unknown,
    post: (world: { readonly id: number }, path: string): unknown => JSON.parse(abi().og_load_post(world.id, path)) as unknown,
    vector: (world: { readonly id: number }, path: string): unknown => JSON.parse(abi().og_load_vector(world.id, path)) as unknown,
    uiMaterial: (world: { readonly id: number }, path: string): unknown => JSON.parse(abi().og_load_ui_material(world.id, path)) as unknown,
    texture: (world: { readonly id: number }, path: string): unknown => JSON.parse(abi().og_load_texture(world.id, path)) as unknown,
    textureArray: (world: { readonly id: number }, path: string): unknown => JSON.parse(abi().og_load_texture_array(world.id, path)) as unknown,
    style: (world: { readonly id: number }, path: string): unknown => JSON.parse(abi().og_load_style(world.id, path)) as unknown,
    look: (world: { readonly id: number }, path: string): unknown => JSON.parse(abi().og_load_look(world.id, path)) as unknown,
    lookInfo: (world: { readonly id: number }, look: number): unknown => JSON.parse(abi().og_load_look_info(world.id, look)) as unknown,
    palette: (world: { readonly id: number }, path: string): unknown => JSON.parse(abi().og_load_palette(world.id, path)) as unknown,
    soundInfo: (world: { readonly id: number }, sound: number): unknown => JSON.parse(abi().og_load_sound_info(world.id, sound)) as unknown,
    status: (world: { readonly id: number }, asset: number): string => abi().og_load_status(world.id, asset),
    error: (world: { readonly id: number }, asset: number): unknown => JSON.parse(abi().og_load_error(world.id, asset)) as unknown,
    gltfInfo: (world: { readonly id: number }, asset: number): unknown => JSON.parse(abi().og_load_gltf_info(world.id, asset)) as unknown,
    animation: (world: { readonly id: number }, path: string): unknown => JSON.parse(abi().og_load_animation(world.id, path)) as unknown,
    rig: (world: { readonly id: number }, path: string): unknown => JSON.parse(abi().og_load_rig(world.id, path)) as unknown,
    flipbook: (world: { readonly id: number }, path: string): unknown => JSON.parse(abi().og_load_flipbook(world.id, path)) as unknown,
    tilemap: (world: { readonly id: number }, path: string): unknown => JSON.parse(abi().og_load_tilemap(world.id, path)) as unknown,
    video: (world: { readonly id: number }, path: string): unknown => JSON.parse(abi().og_load_video(world.id, path)) as unknown,
    videoInfo: (world: { readonly id: number }, video: number): unknown => JSON.parse(abi().og_load_video_info(world.id, video)) as unknown,
    timeline: (world: { readonly id: number }, path: string): unknown => JSON.parse(abi().og_load_timeline(world.id, path)) as unknown,
    vfx: (world: { readonly id: number }, path: string): unknown => JSON.parse(abi().og_load_vfx(world.id, path)) as unknown,
    rig2d: (world: { readonly id: number }, path: string): unknown => JSON.parse(abi().og_load_rig2d(world.id, path)) as unknown,
    imageData: (world: { readonly id: number }, name: string, bytes: Uint8Array, desc?: unknown): unknown => JSON.parse(abi().og_load_image_data(world.id, name, bytes, json(desc))) as unknown,
    gltfData: (world: { readonly id: number }, name: string, bytes: Uint8Array, desc?: unknown): unknown => JSON.parse(abi().og_load_gltf_data(world.id, name, bytes, json(desc))) as unknown,
    environmentData: (world: { readonly id: number }, name: string, bytes: Uint8Array, desc?: unknown): unknown => JSON.parse(abi().og_load_environment_data(world.id, name, bytes, json(desc))) as unknown,
    data: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_load_data(world.id, json(desc))) as unknown,
    dataJson: (world: { readonly id: number }, asset: number): unknown => JSON.parse(abi().og_load_data_json(world.id, asset)) as unknown,
    dataBytes: (world: { readonly id: number }, asset: number): unknown => custom.load.dataBytes(world, asset),
    pack: (world: { readonly id: number }, entry?: unknown): unknown => JSON.parse(abi().og_load_pack(world.id, json(entry))) as unknown,
    environmentInfo: (world: { readonly id: number }, asset: number): unknown => JSON.parse(abi().og_load_environment_info(world.id, asset)) as unknown,
    release: (world: { readonly id: number }, asset: number): unknown => JSON.parse(abi().og_load_release(world.id, asset)) as unknown,
    imageInfo: (world: { readonly id: number }, asset: number): unknown => JSON.parse(abi().og_load_image_info(world.id, asset)) as unknown,
    completions: (world: { readonly id: number }): unknown => JSON.parse(abi().og_load_completions(world.id)) as unknown,
    replayCompletions: (world: { readonly id: number }, completions: unknown): unknown => JSON.parse(abi().og_load_replay_completions(world.id, JSON.stringify(completions ?? []))) as unknown,
  }),
  physics: Object.freeze({
    water: (world: { readonly id: number }, point: unknown): unknown => JSON.parse(abi().og_physics_water(world.id, JSON.stringify(point))) as unknown,
    waterHeight: (world: { readonly id: number }, entity: number, x: number, z: number): unknown => JSON.parse(abi().og_physics_water_height(world.id, entity, x, z)) as unknown,
    castRay: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_physics_cast_ray(world.id, json(desc))) as unknown,
    castRayAll: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_physics_cast_ray_all(world.id, json(desc))) as unknown,
    projectPoint: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_physics_project_point(world.id, json(desc))) as unknown,
    castShape: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_physics_cast_shape(world.id, json(desc))) as unknown,
    overlap: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_physics_overlap(world.id, json(desc))) as unknown,
    sensorContents: (world: { readonly id: number }, sensor: number): unknown => JSON.parse(abi().og_physics_sensor_contents(world.id, sensor)) as unknown,
    applyForce: (world: { readonly id: number }, entity: number, force: unknown, point?: unknown): unknown => JSON.parse(abi().og_physics_apply_force(world.id, entity, JSON.stringify(force), json(point))) as unknown,
    applyImpulse: (world: { readonly id: number }, entity: number, impulse: unknown, point?: unknown): unknown => JSON.parse(abi().og_physics_apply_impulse(world.id, entity, JSON.stringify(impulse), json(point))) as unknown,
    applyTorque: (world: { readonly id: number }, entity: number, torque: unknown): unknown => JSON.parse(abi().og_physics_apply_torque(world.id, entity, JSON.stringify(torque))) as unknown,
    applyTorqueImpulse: (world: { readonly id: number }, entity: number, torque: unknown): unknown => JSON.parse(abi().og_physics_apply_torque_impulse(world.id, entity, JSON.stringify(torque))) as unknown,
    setMesh: (world: { readonly id: number }, mesh: number, desc?: unknown): unknown => JSON.parse(abi().og_physics_set_mesh(world.id, mesh, json(desc))) as unknown,
    ropePoints: (world: { readonly id: number }, entity: number): unknown => JSON.parse(abi().og_physics_rope_points(world.id, entity)) as unknown,
    garmentPoints: (world: { readonly id: number }, entity: number): unknown => JSON.parse(abi().og_physics_garment_points(world.id, entity)) as unknown,
    clothPoints: (world: { readonly id: number }, entity: number): unknown => JSON.parse(abi().og_physics_cloth_points(world.id, entity)) as unknown,
    softBodyPoints: (world: { readonly id: number }, entity: number): unknown => JSON.parse(abi().og_physics_soft_body_points(world.id, entity)) as unknown,
    fluidParticles: (world: { readonly id: number }, entity: number): unknown => JSON.parse(abi().og_physics_fluid_particles(world.id, entity)) as unknown,
    fluid3dParticles: (world: { readonly id: number }, entity: number): unknown => JSON.parse(abi().og_physics_fluid3d_particles(world.id, entity)) as unknown,
    vehicleWheels: (world: { readonly id: number }, entity: number): unknown => JSON.parse(abi().og_physics_vehicle_wheels(world.id, entity)) as unknown,
    ragdollRigFromSkeleton: (world: { readonly id: number }, skeleton: number): unknown => JSON.parse(abi().og_physics_ragdoll_rig_from_skeleton(world.id, skeleton)) as unknown,
    setRagdollRig: (world: { readonly id: number }, entity: number, desc?: unknown): unknown => JSON.parse(abi().og_physics_set_ragdoll_rig(world.id, entity, json(desc))) as unknown,
    ragdollPose: (world: { readonly id: number }, entity: number): unknown => JSON.parse(abi().og_physics_ragdoll_pose(world.id, entity)) as unknown,
    stats: (world: { readonly id: number }): unknown => JSON.parse(abi().og_physics_stats(world.id)) as unknown,
  }),
  atmosphere: Object.freeze({
    timeOfDay: (world: { readonly id: number }): number => abi().og_atmosphere_time_of_day(world.id),
    season: (world: { readonly id: number }): unknown => JSON.parse(abi().og_atmosphere_season(world.id)) as unknown,
  }),
  anim: Object.freeze({
    getNodeMatrix: (world: { readonly id: number }, entity: number, node: number): unknown => JSON.parse(abi().og_anim_get_node_matrix(world.id, entity, node)) as unknown,
    graph: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_anim_graph(world.id, json(desc))) as unknown,
    humanoid: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_anim_humanoid(world.id, json(desc))) as unknown,
    retarget: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_anim_retarget(world.id, json(desc))) as unknown,
    mirror: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_anim_mirror(world.id, json(desc))) as unknown,
    rig2dInfo: (world: { readonly id: number }, rig: number): unknown => JSON.parse(abi().og_anim_rig2d_info(world.id, rig)) as unknown,
    skeleton2dBoxes: (world: { readonly id: number }, entity: number): unknown => JSON.parse(abi().og_anim_skeleton2d_boxes(world.id, entity)) as unknown,
  }),
  tween: Object.freeze({
    to: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_tween_to(world.id, json(desc))) as unknown,
    from: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_tween_from(world.id, json(desc))) as unknown,
    fromTo: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_tween_from_to(world.id, json(desc))) as unknown,
    sequence: (world: { readonly id: number }, steps: unknown, desc?: unknown): unknown => JSON.parse(abi().og_tween_sequence(world.id, JSON.stringify(steps ?? []), json(desc))) as unknown,
    parallel: (world: { readonly id: number }, tweens: unknown, desc?: unknown): unknown => JSON.parse(abi().og_tween_parallel(world.id, JSON.stringify(tweens ?? []), json(desc))) as unknown,
    timeline: (world: { readonly id: number }, desc?: unknown): TweenTimeline => new TweenTimeline(abi().og_tween_timeline(world.id, json(desc))),
    cancel: (world: { readonly id: number }, tween: number): boolean => abi().og_tween_cancel(world.id, tween),
    cancelEntity: (world: { readonly id: number }, entity: number): number => abi().og_tween_cancel_entity(world.id, entity),
    active: (world: { readonly id: number }, tween: number): boolean => abi().og_tween_active(world.id, tween),
    squash: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_tween_squash(world.id, json(desc))) as unknown,
    flash: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_tween_flash(world.id, json(desc))) as unknown,
  }),
  tilemap: Object.freeze({
    setTile: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_tilemap_set_tile(world.id, json(desc))) as unknown,
    fill: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_tilemap_fill(world.id, json(desc))) as unknown,
    create: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_tilemap_create(world.id, json(desc))) as unknown,
    resize: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_tilemap_resize(world.id, json(desc))) as unknown,
    neighbours: (world: { readonly id: number }, map: number, x: number, y: number): unknown => JSON.parse(abi().og_tilemap_neighbours(world.id, map, x, y)) as unknown,
    getTile: (world: { readonly id: number }, entity: number, layer: string, x: number, y: number): number => abi().og_tilemap_get_tile(world.id, entity, layer, x, y),
    cellAt: (world: { readonly id: number }, entity: number, point: unknown): unknown => JSON.parse(abi().og_tilemap_cell_at(world.id, entity, JSON.stringify(point))) as unknown,
    cellCenter: (world: { readonly id: number }, entity: number, x: number, y: number): unknown => JSON.parse(abi().og_tilemap_cell_center(world.id, entity, x, y)) as unknown,
  }),
  audio: Object.freeze({
    playFeedback: (world: { readonly id: number }, sound: number, desc?: unknown): unknown => JSON.parse(abi().og_audio_play_feedback(world.id, sound, json(desc))) as unknown,
    createSound: (world: { readonly id: number }, desc?: unknown): unknown => custom.audio.createSound(world, desc),
    outputLatency: (world: { readonly id: number }): unknown => custom.audio.outputLatency(world),
  }),
  nav: Object.freeze({
    grid: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_nav_grid(world.id, json(desc))) as unknown,
    gridFromTilemap: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_nav_grid_from_tilemap(world.id, json(desc))) as unknown,
    setCost: (world: { readonly id: number }, grid: number, x: number, y: number, cost: number): unknown => JSON.parse(abi().og_nav_set_cost(world.id, grid, x, y, cost)) as unknown,
    cellAt: (world: { readonly id: number }, grid: number, point: unknown): unknown => JSON.parse(abi().og_nav_cell_at(world.id, grid, JSON.stringify(point))) as unknown,
    findPath: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_nav_find_path(world.id, json(desc))) as unknown,
    flowField: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_nav_flow_field(world.id, json(desc))) as unknown,
    setFlowTargets: (world: { readonly id: number }, field: number, targets: unknown): unknown => JSON.parse(abi().og_nav_set_flow_targets(world.id, field, JSON.stringify(targets ?? []))) as unknown,
    flowAt: (world: { readonly id: number }, field: number, point: unknown): unknown => JSON.parse(abi().og_nav_flow_at(world.id, field, JSON.stringify(point))) as unknown,
    load: (world: { readonly id: number }, path: string, profile?: string): unknown => JSON.parse(abi().og_nav_load(world.id, path, profile)) as unknown,
    build: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_nav_build(world.id, json(desc))) as unknown,
    setObstacle: (world: { readonly id: number }, mesh: number, id: number, obstacle?: unknown): unknown => JSON.parse(abi().og_nav_set_obstacle(world.id, mesh, id, json(obstacle))) as unknown,
    carve: (world: { readonly id: number }, mesh: number, id: number, shape?: unknown): unknown => JSON.parse(abi().og_nav_carve(world.id, mesh, id, json(shape))) as unknown,
    setArea: (world: { readonly id: number }, mesh: number, id: number, desc?: unknown): unknown => JSON.parse(abi().og_nav_set_area(world.id, mesh, id, json(desc))) as unknown,
    setLink: (world: { readonly id: number }, mesh: number, id: number, desc?: unknown): unknown => JSON.parse(abi().og_nav_set_link(world.id, mesh, id, json(desc))) as unknown,
    setLinkOpen: (world: { readonly id: number }, mesh: number, id: number, open: boolean): unknown => JSON.parse(abi().og_nav_set_link_open(world.id, mesh, id, open)) as unknown,
    finishLink: (world: { readonly id: number }, entity: number): boolean => abi().og_nav_finish_link(world.id, entity),
    nearest: (world: { readonly id: number }, mesh: number, point: unknown, extent?: unknown): unknown => JSON.parse(abi().og_nav_nearest(world.id, mesh, JSON.stringify(point), json(extent))) as unknown,
    pathOf: (world: { readonly id: number }, entity: number): unknown => JSON.parse(abi().og_nav_path_of(world.id, entity)) as unknown,
  }),
  ai: Object.freeze({
    load: (world: { readonly id: number }, path: string): unknown => JSON.parse(abi().og_ai_load(world.id, path)) as unknown,
    parse: (world: { readonly id: number }, text: string): unknown => JSON.parse(abi().og_ai_parse(world.id, text)) as unknown,
    get: (world: { readonly id: number }, entity: number, key: string): unknown => JSON.parse(abi().og_ai_get(world.id, entity, key)) as unknown,
    set: (world: { readonly id: number }, entity: number, key: string, value: unknown): unknown => JSON.parse(abi().og_ai_set(world.id, entity, key, JSON.stringify(value ?? null))) as unknown,
    restart: (world: { readonly id: number }, entity: number): unknown => JSON.parse(abi().og_ai_restart(world.id, entity)) as unknown,
    finish: (world: { readonly id: number }, entity: number, success: boolean, action?: string): boolean => abi().og_ai_finish(world.id, entity, success, action),
    noise: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_ai_noise(world.id, json(desc))) as unknown,
    describe: (world: { readonly id: number }, entity: number): unknown => JSON.parse(abi().og_ai_describe(world.id, entity)) as unknown,
  }),
  story: Object.freeze({
    setSaliency: (world: { readonly id: number }, strategy: string): unknown => JSON.parse(abi().og_story_set_saliency(world.id, strategy)) as unknown,
    load: (world: { readonly id: number }, path: string): unknown => JSON.parse(abi().og_story_load(world.id, path)) as unknown,
    parse: (world: { readonly id: number }, text: string): unknown => JSON.parse(abi().og_story_parse(world.id, text)) as unknown,
    start: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_story_start(world.id, json(desc))) as unknown,
    view: (world: { readonly id: number }, conversation: number): unknown => JSON.parse(abi().og_story_view(world.id, conversation)) as unknown,
    graph: (world: { readonly id: number }, dialogue: string): unknown => JSON.parse(abi().og_story_graph(world.id, dialogue)) as unknown,
    history: (world: { readonly id: number }, conversation: number): unknown => JSON.parse(abi().og_story_history(world.id, conversation)) as unknown,
    conversations: (world: { readonly id: number }): unknown => JSON.parse(abi().og_story_conversations(world.id)) as unknown,
    advance: (world: { readonly id: number }, conversation: number): unknown => JSON.parse(abi().og_story_advance(world.id, conversation)) as unknown,
    choose: (world: { readonly id: number }, conversation: number, index: number): unknown => JSON.parse(abi().og_story_choose(world.id, conversation, index)) as unknown,
    stop: (world: { readonly id: number }, conversation: number): boolean => abi().og_story_stop(world.id, conversation),
    bark: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_story_bark(world.id, json(desc))) as unknown,
    ambient: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_story_ambient(world.id, json(desc))) as unknown,
    barks: (world: { readonly id: number }): unknown => JSON.parse(abi().og_story_barks(world.id)) as unknown,
    get: (world: { readonly id: number }, name: string): unknown => JSON.parse(abi().og_story_get(world.id, name)) as unknown,
    set: (world: { readonly id: number }, name: string, value: unknown): unknown => JSON.parse(abi().og_story_set(world.id, name, JSON.stringify(value ?? null))) as unknown,
    reset: (world: { readonly id: number }): void => abi().og_story_reset(world.id),
    startQuest: (world: { readonly id: number }, id: string): unknown => JSON.parse(abi().og_story_start_quest(world.id, id)) as unknown,
    progress: (world: { readonly id: number }, quest: string, objective: string, amount?: number): boolean => abi().og_story_progress(world.id, quest, objective, amount),
    setStage: (world: { readonly id: number }, quest: string, stage: string): unknown => JSON.parse(abi().og_story_set_stage(world.id, quest, stage)) as unknown,
    completeQuest: (world: { readonly id: number }, quest: string): unknown => JSON.parse(abi().og_story_complete_quest(world.id, quest)) as unknown,
    failQuest: (world: { readonly id: number }, quest: string): unknown => JSON.parse(abi().og_story_fail_quest(world.id, quest)) as unknown,
    quest: (world: { readonly id: number }, id: string): unknown => JSON.parse(abi().og_story_quest(world.id, id)) as unknown,
    quests: (world: { readonly id: number }): unknown => JSON.parse(abi().og_story_quests(world.id)) as unknown,
    track: (world: { readonly id: number }, id: string, on: boolean): boolean => abi().og_story_track(world.id, id, on),
    markSeen: (world: { readonly id: number }, id: string): boolean => abi().og_story_mark_seen(world.id, id),
    mark: (world: { readonly id: number }, id: string, desc?: unknown): boolean => abi().og_story_mark(world.id, id, json(desc)),
  }),
  platform: Object.freeze({
    storeGet: (world: { readonly id: number }, key: string): unknown => JSON.parse(abi().og_platform_store_get(world.id, key)) as unknown,
    storeSet: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_platform_store_set(world.id, json(desc))) as unknown,
    catalogHead: (world: { readonly id: number }, catalog: string): unknown => JSON.parse(abi().og_platform_catalog_head(world.id, catalog)) as unknown,
    storeList: (world: { readonly id: number }): number => abi().og_platform_store_list(world.id),
    leaderboardSubmit: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_platform_leaderboard_submit(world.id, json(desc))) as unknown,
    leaderboardTop: (world: { readonly id: number }, desc?: unknown): number => abi().og_platform_leaderboard_top(world.id, json(desc)),
    leaderboardAround: (world: { readonly id: number }, desc?: unknown): number => abi().og_platform_leaderboard_around(world.id, json(desc)),
    telemetry: (world: { readonly id: number }, name: string, attrs?: unknown): unknown => JSON.parse(abi().og_platform_telemetry(world.id, name, json(attrs))) as unknown,
    point: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_platform_point(world.id, json(desc))) as unknown,
    aiMark: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_platform_ai_mark(world.id, json(desc))) as unknown,
    loadSettings: (world: { readonly id: number }): number => abi().og_platform_load_settings(world.id),
    saveSettings: (world: { readonly id: number }, value: unknown): unknown => JSON.parse(abi().og_platform_save_settings(world.id, JSON.stringify(value ?? null))) as unknown,
    saveSlot: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_platform_save_slot(world.id, json(desc))) as unknown,
    loadSlot: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_platform_load_slot(world.id, json(desc))) as unknown,
    products: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_platform_products(world.id, json(desc))) as unknown,
    purchase: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_platform_purchase(world.id, json(desc))) as unknown,
    entitlements: (world: { readonly id: number }): number => abi().og_platform_entitlements(world.id),
    consume: (world: { readonly id: number }, purchase: string): unknown => JSON.parse(abi().og_platform_consume(world.id, purchase)) as unknown,
    grant: (world: { readonly id: number }, purchase: string): unknown => JSON.parse(abi().og_platform_grant(world.id, purchase)) as unknown,
    granted: (world: { readonly id: number }, purchase: string): boolean => abi().og_platform_granted(world.id, purchase),
    adReady: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_platform_ad_ready(world.id, json(desc))) as unknown,
    showAd: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_platform_show_ad(world.id, json(desc))) as unknown,
    achievementUnlock: (world: { readonly id: number }, id: string): unknown => JSON.parse(abi().og_platform_achievement_unlock(world.id, id)) as unknown,
    achievementProgress: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_platform_achievement_progress(world.id, json(desc))) as unknown,
    achievements: (world: { readonly id: number }): number => abi().og_platform_achievements(world.id),
    player: (world: { readonly id: number }): number => abi().og_platform_player(world.id),
    friends: (world: { readonly id: number }, desc?: unknown): number => abi().og_platform_friends(world.id, json(desc)),
    inviteSend: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_platform_invite_send(world.id, json(desc))) as unknown,
    invites: (world: { readonly id: number }): number => abi().og_platform_invites(world.id),
    share: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_platform_share(world.id, json(desc))) as unknown,
    supports: (world: { readonly id: number }, method: string): boolean => abi().og_platform_supports(world.id, method),
    notice: (world: { readonly id: number }, notice: number): unknown => JSON.parse(abi().og_platform_notice(world.id, notice)) as unknown,
    result: (world: { readonly id: number }, request: number): unknown => JSON.parse(abi().og_platform_result(world.id, request)) as unknown,
    info: (world: { readonly id: number }): unknown => JSON.parse(abi().og_platform_info(world.id)) as unknown,
    journal: (world: { readonly id: number }): unknown => JSON.parse(abi().og_platform_journal(world.id)) as unknown,
    now: (world: { readonly id: number }): number => abi().og_platform_now(world.id),
    roomCreate: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_platform_room_create(world.id, json(desc))) as unknown,
    roomJoin: (world: { readonly id: number }, room: string): unknown => JSON.parse(abi().og_platform_room_join(world.id, room)) as unknown,
    quickPlay: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_platform_quick_play(world.id, json(desc))) as unknown,
    roomLeave: (world: { readonly id: number }): unknown => JSON.parse(abi().og_platform_room_leave(world.id)) as unknown,
    roomSend: (world: { readonly id: number }, data: unknown, to?: number): unknown => custom.platform.roomSend(world, data, to),
    roomPeers: (world: { readonly id: number }): unknown => JSON.parse(abi().og_platform_room_peers(world.id)) as unknown,
    roomInfo: (world: { readonly id: number }): unknown => JSON.parse(abi().og_platform_room_info(world.id)) as unknown,
    roomMessage: (world: { readonly id: number }, message: number): unknown => custom.platform.roomMessage(world, message),
  }),
  i18n: Object.freeze({
    load: (world: { readonly id: number }, path: string): unknown => JSON.parse(abi().og_i18n_load(world.id, path)) as unknown,
    setLocale: (world: { readonly id: number }, locale: string): unknown => JSON.parse(abi().og_i18n_set_locale(world.id, locale)) as unknown,
    locale: (world: { readonly id: number }): unknown => JSON.parse(abi().og_i18n_locale(world.id)) as unknown,
    t: (world: { readonly id: number }, key: string, args?: unknown): string => abi().og_i18n_t(world.id, key, json(args)),
    has: (world: { readonly id: number }, key: string): boolean => abi().og_i18n_has(world.id, key),
    formatNumber: (world: { readonly id: number }, value: number, format?: unknown): string => abi().og_i18n_format_number(world.id, value, json(format)),
    formatCompact: (world: { readonly id: number }, value: number, decimals?: number): string => abi().og_i18n_format_compact(world.id, value, decimals),
    formatCurrency: (world: { readonly id: number }, amount: number, currency: string): string => abi().og_i18n_format_currency(world.id, amount, currency),
    formatDate: (world: { readonly id: number }, timeMs: number, format?: unknown): string => abi().og_i18n_format_date(world.id, timeMs, json(format)),
    formatTime: (world: { readonly id: number }, timeMs: number, format?: unknown): string => abi().og_i18n_format_time(world.id, timeMs, json(format)),
    formatDuration: (world: { readonly id: number }, seconds: number, style?: string): string => abi().og_i18n_format_duration(world.id, seconds, style),
  }),
  ui: Object.freeze({
    configure: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_ui_configure(world.id, json(desc))) as unknown,
    viewport: (world: { readonly id: number }): unknown => JSON.parse(abi().og_ui_viewport(world.id)) as unknown,
    theme: (world: { readonly id: number }): unknown => JSON.parse(abi().og_ui_theme(world.id)) as unknown,
    framePointer: (): unknown => JSON.parse(abi().og_ui_frame_pointer()) as unknown,
    frameCursor: (): string => abi().og_ui_frame_cursor(),
    frameViewport: (): unknown => JSON.parse(abi().og_ui_frame_viewport()) as unknown,
    hasIcon: (name: string): boolean => abi().og_ui_has_icon(name),
    seconds: (): number => abi().og_ui_seconds(),
    frameDirection: (): string => abi().og_ui_frame_direction(),
    frameTheme: (): unknown => JSON.parse(abi().og_ui_frame_theme()) as unknown,
    setVirtualCursor: (desc: unknown): void => abi().og_ui_set_virtual_cursor(JSON.stringify(desc ?? null)),
    virtualCursor: (): unknown => JSON.parse(abi().og_ui_virtual_cursor()) as unknown,
    feedback: (event: string, rise: number): void => abi().og_ui_feedback(event, rise),
    setHaptics: (intensity: number): void => abi().og_ui_set_haptics(intensity),
    tick: (): number => abi().og_ui_tick(),
    previewViewport: (world: { readonly id: number }, viewport: unknown): void => abi().og_ui_preview_viewport(world.id, JSON.stringify(viewport ?? null)),
  }),
  commands: Object.freeze({
    register: (name: string, handler: Function, desc?: unknown): unknown => custom.commands.register(name, handler, desc),
    parse: (script: string): unknown => JSON.parse(abi().og_commands_parse(script)) as unknown,
    help: (world: { readonly id: number }): unknown => JSON.parse(abi().og_commands_help(world.id)) as unknown,
    schedule: (world: { readonly id: number }, script: string): unknown => JSON.parse(abi().og_commands_schedule(world.id, script)) as unknown,
  }),
  terrain: Object.freeze({
    load: (world: { readonly id: number }, path: string): unknown => JSON.parse(abi().og_terrain_load(world.id, path)) as unknown,
    sample: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_terrain_sample(world.id, json(desc))) as unknown,
    bounds: (world: { readonly id: number }, data: number, origin?: unknown, quaternion?: unknown, scale?: unknown): unknown => JSON.parse(abi().og_terrain_bounds(world.id, data, json(origin), json(quaternion), json(scale))) as unknown,
    edit: (world: { readonly id: number }, data: number, brush?: unknown): unknown => JSON.parse(abi().og_terrain_edit(world.id, data, json(brush))) as unknown,
    stamp: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_terrain_stamp(world.id, json(desc))) as unknown,
    unstamp: (world: { readonly id: number }, data: number, stamp: number): unknown => JSON.parse(abi().og_terrain_unstamp(world.id, data, stamp)) as unknown,
    layer: (world: { readonly id: number }, data: number, index: number): unknown => JSON.parse(abi().og_terrain_layer(world.id, data, index)) as unknown,
    setLayer: (world: { readonly id: number }, data: number, index: number, layer?: unknown): unknown => JSON.parse(abi().og_terrain_set_layer(world.id, data, index, json(layer))) as unknown,
  }),
  stream: Object.freeze({
    status: (world: { readonly id: number }, group: string): unknown => JSON.parse(abi().og_stream_status(world.id, group)) as unknown,
    request: (world: { readonly id: number }, group: string): unknown => JSON.parse(abi().og_stream_request(world.id, group)) as unknown,
    release: (world: { readonly id: number }, group: string): unknown => JSON.parse(abi().og_stream_release(world.id, group)) as unknown,
    budget: (world: { readonly id: number }): unknown => JSON.parse(abi().og_stream_budget(world.id)) as unknown,
    setBudget: (world: { readonly id: number }, bytes: number): unknown => JSON.parse(abi().og_stream_set_budget(world.id, bytes)) as unknown,
  }),
  video: Object.freeze({
    capabilities: (world: { readonly id: number }): unknown => JSON.parse(abi().og_video_capabilities(world.id)) as unknown,
  }),
  debug: Object.freeze({
    line: (world: { readonly id: number }, from: unknown, to: unknown, desc?: unknown): void => abi().og_debug_line(world.id, JSON.stringify(from), JSON.stringify(to), json(desc)),
    box: (world: { readonly id: number }, center: unknown, size: unknown, desc?: unknown): void => abi().og_debug_box(world.id, JSON.stringify(center), JSON.stringify(size), json(desc)),
    sphere: (world: { readonly id: number }, center: unknown, radius: number, desc?: unknown): void => abi().og_debug_sphere(world.id, JSON.stringify(center), radius, json(desc)),
    arrow: (world: { readonly id: number }, from: unknown, to: unknown, desc?: unknown): void => abi().og_debug_arrow(world.id, JSON.stringify(from), JSON.stringify(to), json(desc)),
    axes: (world: { readonly id: number }, position: unknown, size: number, desc?: unknown): void => abi().og_debug_axes(world.id, JSON.stringify(position), size, json(desc)),
    text: (world: { readonly id: number }, position: unknown, text: string, desc?: unknown): void => abi().og_debug_text(world.id, JSON.stringify(position), text, json(desc)),
    clear: (world: { readonly id: number }): void => abi().og_debug_clear(world.id),
    setView: (world: { readonly id: number }, view: string): void => abi().og_debug_set_view(world.id, view),
    view: (world: { readonly id: number }): unknown => custom.debug.view(world),
    setEnabled: (world: { readonly id: number }, enabled: boolean): void => abi().og_debug_set_enabled(world.id, enabled),
    enabled: (world: { readonly id: number }): boolean => abi().og_debug_enabled(world.id),
  }),
  assert: Object.freeze({
    record: (world: { readonly id: number }, record: unknown): number => abi().og_assert_record(world.id, JSON.stringify(record ?? null)),
  }),
  path: Object.freeze({
    create: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_path_create(world.id, json(desc))) as unknown,
    find: (world: { readonly id: number }, name: string): unknown => JSON.parse(abi().og_path_find(world.id, name)) as unknown,
    sample: (world: { readonly id: number }, path: number, t: number): unknown => JSON.parse(abi().og_path_sample(world.id, path, t)) as unknown,
    length: (world: { readonly id: number }, path: number): unknown => JSON.parse(abi().og_path_length(world.id, path)) as unknown,
    nearest: (world: { readonly id: number }, path: number, point: unknown): unknown => JSON.parse(abi().og_path_nearest(world.id, path, JSON.stringify(point))) as unknown,
  }),
  timeline: Object.freeze({
    captions: (world: { readonly id: number }): unknown => JSON.parse(abi().og_timeline_captions(world.id)) as unknown,
    info: (world: { readonly id: number }, asset: number): unknown => JSON.parse(abi().og_timeline_info(world.id, asset)) as unknown,
    play: (world: { readonly id: number }, entity: number): unknown => JSON.parse(abi().og_timeline_play(world.id, entity)) as unknown,
    pause: (world: { readonly id: number }, entity: number): unknown => JSON.parse(abi().og_timeline_pause(world.id, entity)) as unknown,
    seek: (world: { readonly id: number }, entity: number, time: number): unknown => JSON.parse(abi().og_timeline_seek(world.id, entity, time)) as unknown,
    stop: (world: { readonly id: number }, entity: number): unknown => JSON.parse(abi().og_timeline_stop(world.id, entity)) as unknown,
  }),
  net: Object.freeze({
    submit: (world: { readonly id: number }, input?: unknown): unknown => JSON.parse(abi().og_net_submit(world.id, json(input))) as unknown,
    shape: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_net_shape(world.id, json(desc))) as unknown,
    submitInput: (world: { readonly id: number }, component: { readonly name: string }, value: unknown): unknown => JSON.parse(abi().og_net_submit_input(world.id, component.name, JSON.stringify(value ?? {}))) as unknown,
    command: (world: { readonly id: number }, component: { readonly name: string }, value: unknown): unknown => JSON.parse(abi().og_net_command(world.id, component.name, JSON.stringify(value ?? {}))) as unknown,
    commands: (world: { readonly id: number }, component: { readonly name: string }): unknown => JSON.parse(abi().og_net_commands(world.id, component.name)) as unknown,
    status: (world: { readonly id: number }): unknown => JSON.parse(abi().og_net_status(world.id)) as unknown,
    resimulating: (world: { readonly id: number }): boolean => abi().og_net_resimulating(world.id),
    finalHash: (world: { readonly id: number }, tick: number): unknown => JSON.parse(abi().og_net_final_hash(world.id, tick)) as unknown,
    hash: (world: { readonly id: number }): string => abi().og_net_hash(world.id),
  }),
  shared: Object.freeze({
    shape: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_shared_shape(world.id, json(desc))) as unknown,
    join: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_shared_join(world.id, json(desc))) as unknown,
    leave: (world: { readonly id: number }): unknown => JSON.parse(abi().og_shared_leave(world.id)) as unknown,
    teleport: (world: { readonly id: number }, position: unknown): unknown => JSON.parse(abi().og_shared_teleport(world.id, JSON.stringify(position))) as unknown,
    send: (world: { readonly id: number }, component: { readonly name: string }, value: unknown): unknown => JSON.parse(abi().og_shared_send(world.id, component.name, JSON.stringify(value ?? {}))) as unknown,
    events: (world: { readonly id: number }, component: { readonly name: string }): unknown => JSON.parse(abi().og_shared_events(world.id, component.name)) as unknown,
    put: (world: { readonly id: number }, key: string, component: { readonly name: string }, value: unknown, desc?: unknown): unknown => JSON.parse(abi().og_shared_put(world.id, key, component.name, JSON.stringify(value ?? {}), json(desc))) as unknown,
    remove: (world: { readonly id: number }, key: string, expect: unknown): unknown => JSON.parse(abi().og_shared_remove(world.id, key, JSON.stringify(expect ?? null))) as unknown,
    object: (world: { readonly id: number }, entity: number): unknown => JSON.parse(abi().og_shared_object(world.id, entity)) as unknown,
    remote: (world: { readonly id: number }, entity: number): unknown => JSON.parse(abi().og_shared_remote(world.id, entity)) as unknown,
    status: (world: { readonly id: number }): unknown => JSON.parse(abi().og_shared_status(world.id)) as unknown,
  }),
  motion: Object.freeze({
    pose: (world: { readonly id: number }, entity: number, desc?: unknown): unknown => JSON.parse(abi().og_motion_pose(world.id, entity, json(desc))) as unknown,
    place: (world: { readonly id: number }, entity: number, desc?: unknown): unknown => JSON.parse(abi().og_motion_place(world.id, entity, json(desc))) as unknown,
    face: (world: { readonly id: number }, entity: number, desc?: unknown): unknown => JSON.parse(abi().og_motion_face(world.id, entity, json(desc))) as unknown,
    walkTo: (world: { readonly id: number }, entity: number, desc?: unknown): unknown => JSON.parse(abi().og_motion_walk_to(world.id, entity, json(desc))) as unknown,
    jump: (world: { readonly id: number }, entity: number, desc?: unknown): unknown => JSON.parse(abi().og_motion_jump(world.id, entity, json(desc))) as unknown,
    clear: (world: { readonly id: number }, entity: number): unknown => JSON.parse(abi().og_motion_clear(world.id, entity)) as unknown,
    lookAt: (world: { readonly id: number }, entity: number, desc?: unknown): unknown => JSON.parse(abi().og_motion_look_at(world.id, entity, json(desc))) as unknown,
    react: (world: { readonly id: number }, entity: number, desc?: unknown): unknown => JSON.parse(abi().og_motion_react(world.id, entity, json(desc))) as unknown,
    status: (world: { readonly id: number }, entity: number): unknown => JSON.parse(abi().og_motion_status(world.id, entity)) as unknown,
    capabilities: (world: { readonly id: number }): unknown => JSON.parse(abi().og_motion_capabilities(world.id)) as unknown,
    setBackend: (world: { readonly id: number }, backend: string): unknown => JSON.parse(abi().og_motion_set_backend(world.id, backend)) as unknown,
    presentation: (world: { readonly id: number }, entity: number): unknown => JSON.parse(abi().og_motion_presentation(world.id, entity)) as unknown,
    poses: (): unknown => JSON.parse(abi().og_motion_poses()) as unknown,
  }),
  destruction: Object.freeze({
    damage: (world: { readonly id: number }, entity: number, desc?: unknown): unknown => JSON.parse(abi().og_destruction_damage(world.id, entity, json(desc))) as unknown,
    explode: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_destruction_explode(world.id, json(desc))) as unknown,
    fracture: (world: { readonly id: number }, entity: number): unknown => JSON.parse(abi().og_destruction_fracture(world.id, entity)) as unknown,
    info: (world: { readonly id: number }, entity: number): unknown => JSON.parse(abi().og_destruction_info(world.id, entity)) as unknown,
    material: (world: { readonly id: number }, kind: number): unknown => JSON.parse(abi().og_destruction_material(world.id, kind)) as unknown,
    capabilities: (world: { readonly id: number }): unknown => JSON.parse(abi().og_destruction_capabilities(world.id)) as unknown,
    carve2d: (world: { readonly id: number }, entity: number, desc?: unknown): unknown => JSON.parse(abi().og_destruction_carve2d(world.id, entity, json(desc))) as unknown,
    fill2d: (world: { readonly id: number }, entity: number, desc?: unknown): unknown => JSON.parse(abi().og_destruction_fill2d(world.id, entity, json(desc))) as unknown,
    explode2d: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_destruction_explode2d(world.id, json(desc))) as unknown,
    fracture2d: (world: { readonly id: number }, entity: number, desc?: unknown): unknown => JSON.parse(abi().og_destruction_fracture2d(world.id, entity, json(desc))) as unknown,
    cell2d: (world: { readonly id: number }, entity: number, point: unknown): unknown => JSON.parse(abi().og_destruction_cell2d(world.id, entity, JSON.stringify(point))) as unknown,
  }),
  vfx: Object.freeze({
    spawn: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_vfx_spawn(world.id, json(desc))) as unknown,
    info: (world: { readonly id: number }, asset: number): unknown => JSON.parse(abi().og_vfx_info(world.id, asset)) as unknown,
  }),
  sight: Object.freeze({
    visible: (world: { readonly id: number }, team: number, point: unknown): unknown => JSON.parse(abi().og_sight_visible(world.id, team, JSON.stringify(point))) as unknown,
    explored: (world: { readonly id: number }, team: number, point: unknown): unknown => JSON.parse(abi().og_sight_explored(world.id, team, JSON.stringify(point))) as unknown,
    reveal: (world: { readonly id: number }, team: number, point: unknown, radius: number): unknown => JSON.parse(abi().og_sight_reveal(world.id, team, JSON.stringify(point), radius)) as unknown,
    forget: (world: { readonly id: number }, team: number): unknown => JSON.parse(abi().og_sight_forget(world.id, team)) as unknown,
  }),
  flow: Object.freeze({
    load: (world: { readonly id: number }, dialogue: string): unknown => JSON.parse(abi().og_flow_load(world.id, dialogue)) as unknown,
    start: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_flow_start(world.id, json(desc))) as unknown,
    current: (world: { readonly id: number }): unknown => JSON.parse(abi().og_flow_current(world.id)) as unknown,
    view: (world: { readonly id: number }, flow: number): unknown => JSON.parse(abi().og_flow_view(world.id, flow)) as unknown,
    choose: (world: { readonly id: number }, flow: number, index: number): unknown => JSON.parse(abi().og_flow_choose(world.id, flow, index)) as unknown,
    advance: (world: { readonly id: number }, flow: number, outcome?: string): unknown => JSON.parse(abi().og_flow_advance(world.id, flow, outcome)) as unknown,
    play: (world: { readonly id: number }, flow: number): unknown => JSON.parse(abi().og_flow_play(world.id, flow)) as unknown,
    pause: (world: { readonly id: number }, flow: number): unknown => JSON.parse(abi().og_flow_pause(world.id, flow)) as unknown,
    skip: (world: { readonly id: number }, flow: number): unknown => JSON.parse(abi().og_flow_skip(world.id, flow)) as unknown,
    chart: (world: { readonly id: number }, dialogue: string): unknown => JSON.parse(abi().og_flow_chart(world.id, dialogue)) as unknown,
    stop: (world: { readonly id: number }, flow: number): unknown => JSON.parse(abi().og_flow_stop(world.id, flow)) as unknown,
  }),
  scene: Object.freeze({
    preload: (world: { readonly id: number }, scene: string): unknown => JSON.parse(abi().og_scene_preload(world.id, scene)) as unknown,
    progress: (world: { readonly id: number }, scene: string): unknown => JSON.parse(abi().og_scene_progress(world.id, scene)) as unknown,
    release: (world: { readonly id: number }, scene: string): unknown => JSON.parse(abi().og_scene_release(world.id, scene)) as unknown,
    change: (world: { readonly id: number }, scene: string): unknown => JSON.parse(abi().og_scene_change(world.id, scene)) as unknown,
    current: (world: { readonly id: number }): unknown => JSON.parse(abi().og_scene_current(world.id)) as unknown,
  }),
  loading: Object.freeze({
    stage: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_loading_stage(world.id, json(desc))) as unknown,
    done: (world: { readonly id: number }): unknown => JSON.parse(abi().og_loading_done(world.id)) as unknown,
    idle: (world: { readonly id: number }): boolean => abi().og_loading_idle(world.id),
    status: (world: { readonly id: number }): unknown => JSON.parse(abi().og_loading_status(world.id)) as unknown,
  }),
  play: Object.freeze({
    candidates: (world: { readonly id: number }, desc?: unknown): unknown => JSON.parse(abi().og_play_candidates(world.id, json(desc))) as unknown,
  }),
  table: Object.freeze({
    load: (world: { readonly id: number }, path: string, schema: { readonly name: string }): unknown => JSON.parse(abi().og_table_load(world.id, path, schema.name)) as unknown,
    parse: (world: { readonly id: number }, path: string, text: string, schema: { readonly name: string }): unknown => JSON.parse(abi().og_table_parse(world.id, path, text, schema.name)) as unknown,
    row: (world: { readonly id: number }, table: string, id: string): unknown => JSON.parse(abi().og_table_row(world.id, table, id)) as unknown,
    rows: (world: { readonly id: number }, table: string): unknown => JSON.parse(abi().og_table_rows(world.id, table)) as unknown,
    get: (world: { readonly id: number }, row: number, schema: { readonly name: string }): unknown => JSON.parse(abi().og_table_get(world.id, row, schema.name)) as unknown,
    name: (world: { readonly id: number }, row: number): unknown => JSON.parse(abi().og_table_name(world.id, row)) as unknown,
    missing: (world: { readonly id: number }): unknown => JSON.parse(abi().og_table_missing(world.id)) as unknown,
  }),
});

/** The members the IDL marks `custom`, by module or owner: src/runtime/custom.ts implements each. */
export interface Custom {
  readonly EngineProbe: {
    restore(self: EngineProbe, id: string): unknown;
  };
  readonly Rng: {
    next(world: number): unknown;
    range(world: number, low: number, high: number): unknown;
    integer(world: number, low: number, high: number): unknown;
    chance(world: number, p: number): unknown;
    weighted(world: number, weights: unknown): unknown;
    permutation(world: number, n: number): unknown;
    fork(world: number, label: string): unknown;
    reset(world: number): unknown;
  };
  readonly RuntimeProbe: {
    advance(self: RuntimeProbe, steps: number): unknown;
    run(self: RuntimeProbe, program: unknown): unknown;
  };
  readonly SaveEdit: {
    has(self: SaveEdit, name: string): unknown;
    transform(self: SaveEdit, name: string, map: Function, desc?: unknown): unknown;
    split(self: SaveEdit, name: string, map: Function): unknown;
  };
  readonly TweenTimeline: {
    add(self: TweenTimeline, child: unknown, position?: unknown): unknown;
  };
  readonly World: {
    spawn(self: World, desc?: unknown): unknown;
    despawn(self: World, entity: number): unknown;
    insert(self: World, entity: number, component: { readonly name: string }, value?: unknown): unknown;
    remove(self: World, entity: number, component: { readonly name: string }): unknown;
    get(self: World, entity: number, component: { readonly name: string }): unknown;
    set(self: World, entity: number, component: { readonly name: string }, value: unknown): unknown;
    resource(self: World, resource: { readonly name: string }): unknown;
    removed(self: World, component: { readonly name: string }): unknown;
    spatial(self: World, desc?: unknown): unknown;
    query(self: World, desc?: unknown): unknown;
    events(self: World, event: { readonly name: string }): unknown;
    emit(self: World, event: { readonly name: string }, payload: unknown): unknown;
    instantiate(self: World, asset: number, desc?: unknown): unknown;
    snapshot(self: World, desc?: unknown): unknown;
    openSave(self: World, save: string): unknown;
    restore(self: World, save: string, desc?: unknown): unknown;
  };
  readonly audio: {
    createSound(world: { readonly id: number }, desc?: unknown): unknown;
    outputLatency(world: { readonly id: number }): unknown;
  };
  readonly commands: {
    register(name: string, handler: Function, desc?: unknown): unknown;
  };
  readonly debug: {
    view(world: { readonly id: number }): unknown;
  };
  readonly game: {
    game(desc?: unknown): unknown;
  };
  readonly load: {
    dataBytes(world: { readonly id: number }, asset: number): unknown;
  };
  readonly platform: {
    roomSend(world: { readonly id: number }, data: unknown, to?: number): unknown;
    roomMessage(world: { readonly id: number }, message: number): unknown;
  };
  readonly reflect: {
    defineComponent(name: string, fields: unknown): unknown;
    defineResource(name: string, fields: unknown): unknown;
    defineEvent(name: string, fields: unknown): unknown;
  };
  readonly render: {
    bufferGeometry(world: { readonly id: number }, desc?: unknown): unknown;
    updateGeometry(world: { readonly id: number }, geometry: number, desc?: unknown): unknown;
    shapeGeometry(world: { readonly id: number }, desc?: unknown): unknown;
    extrudeGeometry(world: { readonly id: number }, desc?: unknown): unknown;
    sweepGeometry(world: { readonly id: number }, desc?: unknown): unknown;
  };
  readonly system: {
    defineHook(name: string, component: { readonly name: string }, desc: unknown, run: Function): unknown;
    defineSystem(name: string, desc: unknown, run: Function): unknown;
  };
}
