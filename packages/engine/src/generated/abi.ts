// Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate.
// The host ABI: every binding symbol with its wire signature. Both hosts
// export exactly these functions; `A` is how a host hands out typed arrays
// and `V` how it hands out a component's column views.

/// The engine's call back into the runtime (`og_host_set_dispatcher`).
export type Dispatcher = (kind: number, engine: number, id: number, query: number, tick: number, dt: number, elapsed: number, payload: string) => string;

export interface HostAbi<A, V> {
  /** Install the function the engine calls for systems, commands and facts: `dispatcher(kind, engine, id, query, tick, dt, elapsed, payload)` (payload: a command's arguments as a JSON list) returns `""` on success, the error otherwise (`kind`: 0 system, 1 command, 2 facts, whose answer is `"="` then JSON). */
  og_host_set_dispatcher(dispatcher: Dispatcher): void;
  /** Assemble a world from a declared game (`og.game`) and the declarations made so far. `options` is JSON `{ seed?, root? }`: a replay seed, and the project directory (native) or URL base (web) whose `cooked/` bundle is opened and whose GameDesc.scene is spawned before the first tick. The answer is a Result of the engine id, which is also the world handle. */
  og_engine_create(game: number, options: string): string;
  /** Drop an engine and everything it owns. False for an unknown engine. */
  og_engine_destroy(engine: number): boolean;
  /** Take fixed steps regardless of the wall clock: a Result of the tick reached, or the first failing system with its tick. */
  og_engine_advance(engine: number, steps: number): string;
  /** Run a registered command action now, between ticks (`args` is a JSON list of strings). The scheduled channel is `og.commands.schedule`. */
  og_engine_command(engine: number, action: string, args: string): string;
  /** One display frame: feed `elapsed_ms` of wall clock to the fixed-tick accumulator (unless paused), run the command steps due, take the ticks it allows up to `GameDesc.maxCatchUp` through the probe, then render when a renderer is attached. JSON `{ ok, ticks, tick, rendered, paused, quit }` or a refusal; `quit` is true once a `quit` command ran. */
  og_engine_frame(engine: number, elapsed_ms: number): string;
  /** A new game bundle is about to be imported (hot reload): its component, resource and command declarations replace the previous bundle's; systems and games keep their handles. The probe's `reload` then assembles the latest `og.game` and restores the world into it. */
  og_host_begin_bundle(): void;
  /** The probe's JSON-RPC 2.0 entry: one request frame in, its answer frame out (og-probe `Probe::call`). The native loopback WebSocket and the web request ring carry exactly these frames. Call it between ticks, never from a system. */
  og_engine_call(engine: number, frame: string): string;
  /** The files the command channel produced since the last call (`screenshot`, `dump_*`, `help`), as JSON `[{ tick, action, file, kind, data }]`: `file` is og-probe's `Output::file_name()`, `kind` is `png`, `json` or `text`, `data` is base64 for a PNG and the text otherwise. The host writes them (og run) or posts them. */
  og_engine_take_outputs(engine: number): string;
  /** Log lines since the last call, as JSON `[{ tick, warning, text }]`: the command channel's and the probe's warnings and the loader's, each naming the fix. */
  og_engine_take_log(engine: number): string;
  /** Write some fields of a resource: how the runtime makes `world.resource(R).value` a live value. A Result. */
  og_engine_write_resource(world: number, resource: string, value: string): string;
  /** `defineComponent` */
  og_reflect_define_component(name: string, fields: string): string;
  /** `defineResource` */
  og_reflect_define_resource(name: string, fields: string): string;
  /** `World.tick` */
  og_world_get_tick(world: number): number;
  /** `World.spawn` */
  og_world_spawn(world: number, desc: string | undefined): string;
  /** `World.despawn` */
  og_world_despawn(world: number, entity: number): string;
  /** `World.alive` */
  og_world_alive(world: number, entity: number): boolean;
  /** `World.has` */
  og_world_has(world: number, entity: number, component: string): boolean;
  /** `World.insert` */
  og_world_insert(world: number, entity: number, component: string, value: string | undefined): string;
  /** `World.remove` */
  og_world_remove(world: number, entity: number, component: string): string;
  /** `World.get` */
  og_world_get(world: number, entity: number, component: string): string;
  /** `World.set` */
  og_world_set(world: number, entity: number, component: string, value: string): string;
  /** `World.resource` */
  og_world_resource(world: number, resource: string): string;
  /** `World.find` */
  og_world_find(world: number, name: string): string;
  /** `World.nameOf` */
  og_world_name_of(world: number, entity: number): string;
  /** `World.query` */
  og_world_query(world: number, desc: string | undefined): string;
  /** `World.events` */
  og_world_events(world: number, event: string): string;
  /** `World.emit` */
  og_world_emit(world: number, event: string, payload: string): string;
  /** `World.instantiate` */
  og_world_instantiate(world: number, asset: number, desc: string | undefined): string;
  /** `Query.archetypes` */
  og_query_archetypes(query: number): string;
  /** `Query.count` */
  og_query_count(query: number): number;
  /** `ArchetypeView.count` */
  og_archetype_view_get_count(archetype_view: number): number;
  /** `ArchetypeView.tick` */
  og_archetype_view_get_tick(archetype_view: number): number;
  /** `ArchetypeView.entities` */
  og_archetype_view_get_entities(archetype_view: number): A;
  /** `ArchetypeView.entity` */
  og_archetype_view_entity(archetype_view: number, index: number): number;
  /** `ArchetypeView.has` */
  og_archetype_view_has(archetype_view: number, component: string): boolean;
  /** `ArchetypeView.column` */
  og_archetype_view_column(archetype_view: number, component: string): V;
  /** `System.name` */
  og_system_get_name(system: number): string;
  /** `System.reads` */
  og_system_get_reads(system: number): string;
  /** `System.writes` */
  og_system_get_writes(system: number): string;
  /** `defineSystem` */
  og_system_define_system(name: string, desc: string, run: number): number;
  /** `Game.tick` */
  og_game_get_tick(game: number): number;
  /** `Game.systems` */
  og_game_get_systems(game: number): string;
  /** `og.game` */
  og_game_game(desc: string | undefined): number;
  /** `og.transform.compose` */
  og_transform_compose(position: string, quaternion: string, scale: string): string;
  /** `og.transform.decompose` */
  og_transform_decompose(matrix: string): string;
  /** `og.transform.lookAt` */
  og_transform_look_at(eye: string, target: string, up: string | undefined): string;
  /** `og.transform.fromAxisAngle` */
  og_transform_from_axis_angle(axis: string, angle: number): string;
  /** `RngStream.seed` */
  og_rng_stream_get_seed(rng_stream: number): number;
  /** `RngStream.next` */
  og_rng_stream_next(rng_stream: number): number;
  /** `RngStream.range` */
  og_rng_stream_range(rng_stream: number, low: number, high: number): number;
  /** `RngStream.integer` */
  og_rng_stream_integer(rng_stream: number, low: number, high: number): number;
  /** `RngStream.chance` */
  og_rng_stream_chance(rng_stream: number, p: number): boolean;
  /** `RngStream.fork` */
  og_rng_stream_fork(rng_stream: number, label: string): number;
  /** `Rng.next` */
  og_rng_next(world: number): number;
  /** `Rng.range` */
  og_rng_range(world: number, low: number, high: number): number;
  /** `Rng.integer` */
  og_rng_integer(world: number, low: number, high: number): number;
  /** `Rng.chance` */
  og_rng_chance(world: number, p: number): boolean;
  /** `Rng.fork` */
  og_rng_fork(world: number, label: string): number;
  /** `Input.down` */
  og_input_down(world: number, key: string): boolean;
  /** `Input.pressed` */
  og_input_pressed(world: number, key: string): boolean;
  /** `Input.released` */
  og_input_released(world: number, key: string): boolean;
  /** `Input.axis` */
  og_input_axis(world: number, negative: string, positive: string): number;
  /** `Input.axis2` */
  og_input_axis2(world: number, left: string, right: string, down: string, up: string): string;
  /** `Input.buttonDown` */
  og_input_button_down(world: number, button: string): boolean;
  /** `Input.buttonPressed` */
  og_input_button_pressed(world: number, button: string): boolean;
  /** `Input.buttonReleased` */
  og_input_button_released(world: number, button: string): boolean;
  /** `Camera.lookAt` */
  og_camera_look_at(world: number, target: string): void;
  /** `Camera.worldToScreen` */
  og_camera_world_to_screen(world: number, point: string): string;
  /** `Camera.screenToRay` */
  og_camera_screen_to_ray(world: number, x: number, y: number): string;
  /** `og.render.sprite` */
  og_render_sprite(world: number, desc: string | undefined): string;
  /** `og.render.measureText` */
  og_render_measure_text(world: number, desc: string | undefined): string;
  /** `og.render.boxGeometry` */
  og_render_box_geometry(world: number, desc: string | undefined): string;
  /** `og.render.planeGeometry` */
  og_render_plane_geometry(world: number, desc: string | undefined): string;
  /** `og.render.sphereGeometry` */
  og_render_sphere_geometry(world: number, desc: string | undefined): string;
  /** `og.render.standardMaterial` */
  og_render_standard_material(world: number, desc: string | undefined): string;
  /** `og.render.updateMaterial` */
  og_render_update_material(world: number, material: number, desc: string | undefined): string;
  /** `og.render.capabilities` */
  og_render_capabilities(world: number): string;
  /** `og.render.text` */
  og_render_text(world: number, desc: string | undefined): string;
  /** `og.load.bundle` */
  og_load_bundle(world: number, path: string): string;
  /** `og.load.bundleInfo` */
  og_load_bundle_info(world: number, asset: number): string;
  /** `og.load.defaultBundle` */
  og_load_default_bundle(world: number): string;
  /** `og.load.gltf` */
  og_load_gltf(world: number, path: string, desc: string | undefined): string;
  /** `og.load.scene` */
  og_load_scene(world: number, path: string): string;
  /** `og.load.font` */
  og_load_font(world: number, path: string): string;
  /** `og.load.fontInfo` */
  og_load_font_info(world: number, font: number): string;
  /** `og.load.sound` */
  og_load_sound(world: number, path: string): string;
  /** `og.load.texture` */
  og_load_texture(world: number, path: string): string;
  /** `og.load.soundInfo` */
  og_load_sound_info(world: number, sound: number): string;
  /** `og.load.status` */
  og_load_status(world: number, asset: number): string;
  /** `og.load.error` */
  og_load_error(world: number, asset: number): string;
  /** `og.load.gltfInfo` */
  og_load_gltf_info(world: number, asset: number): string;
  /** `og.physics.raycast` */
  og_physics_raycast(world: number, desc: string | undefined): string;
  /** `og.physics.shapeCast` */
  og_physics_shape_cast(world: number, desc: string | undefined): string;
  /** `og.physics.overlap` */
  og_physics_overlap(world: number, desc: string | undefined): string;
  /** `og.physics.setMesh` */
  og_physics_set_mesh(world: number, mesh: number, desc: string | undefined): string;
  /** `og.ui.configure` */
  og_ui_configure(world: number, desc: string | undefined): string;
  /** `og.ui.viewport` */
  og_ui_viewport(world: number): string;
  /** `og.ui.theme` */
  og_ui_theme(world: number): string;
  /** `og.ui.open` */
  og_ui_open(id: string, desc: string | undefined): void;
  /** `og.ui.openPanel` */
  og_ui_open_panel(id: string, desc: string | undefined): void;
  /** `og.ui.openScroll` */
  og_ui_open_scroll(id: string, desc: string | undefined): void;
  /** `og.ui.close` */
  og_ui_close(): string;
  /** `og.ui.label` */
  og_ui_label(id: string, text: string, desc: string | undefined): void;
  /** `og.ui.button` */
  og_ui_button(id: string, text: string, desc: string | undefined): string;
  /** `og.ui.image` */
  og_ui_image(id: string, desc: string | undefined): void;
  /** `og.ui.progress` */
  og_ui_progress(id: string, value: number, desc: string | undefined): void;
  /** `og.ui.slider` */
  og_ui_slider(id: string, value: number, desc: string | undefined): string;
  /** `og.ui.toggle` */
  og_ui_toggle(id: string, on: boolean, label: string): string;
  /** `og.ui.textField` */
  og_ui_text_field(id: string, value: string, desc: string | undefined): string;
  /** `og.ui.tooltip` */
  og_ui_tooltip(text: string): void;
  /** `og.commands.register` */
  og_commands_register(name: string, handler: number, desc: string | undefined): string;
  /** `og.commands.parse` */
  og_commands_parse(script: string): string;
  /** `og.commands.help` */
  og_commands_help(world: number): string;
  /** `og.commands.schedule` */
  og_commands_schedule(world: number, script: string): string;
  /** `RuntimeProbe.version` */
  og_runtime_probe_get_version(runtime_probe: number): number;
  /** `RuntimeProbe.tick` */
  og_runtime_probe_tick(runtime_probe: number): number;
  /** `RuntimeProbe.state` */
  og_runtime_probe_state(runtime_probe: number): string;
  /** `RuntimeProbe.advance` */
  og_runtime_probe_advance(runtime_probe: number, steps: number): number;
  /** `RuntimeProbe.pause` */
  og_runtime_probe_pause(runtime_probe: number): boolean;
  /** `RuntimeProbe.resume` */
  og_runtime_probe_resume(runtime_probe: number): boolean;
  /** `RuntimeProbe.input` */
  og_runtime_probe_input(runtime_probe: number, event: string): boolean;
  /** `RuntimeProbe.accepted` */
  og_runtime_probe_accepted(runtime_probe: number): number;
  /** `RuntimeProbe.pick` */
  og_runtime_probe_pick(runtime_probe: number, x: number, y: number): string;
  /** `RuntimeProbe.entity` */
  og_runtime_probe_entity(runtime_probe: number, id: string): string;
  /** `RuntimeProbe.bounds` */
  og_runtime_probe_bounds(runtime_probe: number, id: string): string;
  /** `RuntimeProbe.patch` */
  og_runtime_probe_patch(runtime_probe: number, id: string, parameter: string, value: string): boolean;
  /** `RuntimeProbe.run` */
  og_runtime_probe_run(runtime_probe: number, program: string): string;
  /** `EngineProbe.hello` */
  og_engine_probe_hello(engine_probe: number): string;
  /** `EngineProbe.capture` */
  og_engine_probe_capture(engine_probe: number, width: number | undefined, height: number | undefined): string;
  /** `EngineProbe.sweep` */
  og_engine_probe_sweep(engine_probe: number, desc: string | undefined): string;
  /** `EngineProbe.reload` */
  og_engine_probe_reload(engine_probe: number): string;
  /** `EngineProbe.snapshot` */
  og_engine_probe_snapshot(engine_probe: number): string;
  /** `EngineProbe.restore` */
  og_engine_probe_restore(engine_probe: number, id: string): boolean;
  /** `EngineProbe.dump` */
  og_engine_probe_dump(engine_probe: number, desc: string | undefined): string;
  /** `EngineProbe.commands` */
  og_engine_probe_commands(engine_probe: number, script: string): string;
}

/** Every binding symbol, in binding-table order, then the lifecycle members. */
export const SYMBOLS: readonly string[] = [
  "og_reflect_define_component",
  "og_reflect_define_resource",
  "og_world_get_tick",
  "og_world_spawn",
  "og_world_despawn",
  "og_world_alive",
  "og_world_has",
  "og_world_insert",
  "og_world_remove",
  "og_world_get",
  "og_world_set",
  "og_world_resource",
  "og_world_find",
  "og_world_name_of",
  "og_world_query",
  "og_world_events",
  "og_world_emit",
  "og_world_instantiate",
  "og_query_archetypes",
  "og_query_count",
  "og_archetype_view_get_count",
  "og_archetype_view_get_tick",
  "og_archetype_view_get_entities",
  "og_archetype_view_entity",
  "og_archetype_view_has",
  "og_archetype_view_column",
  "og_system_get_name",
  "og_system_get_reads",
  "og_system_get_writes",
  "og_system_define_system",
  "og_game_get_tick",
  "og_game_get_systems",
  "og_game_game",
  "og_transform_compose",
  "og_transform_decompose",
  "og_transform_look_at",
  "og_transform_from_axis_angle",
  "og_rng_stream_get_seed",
  "og_rng_stream_next",
  "og_rng_stream_range",
  "og_rng_stream_integer",
  "og_rng_stream_chance",
  "og_rng_stream_fork",
  "og_rng_next",
  "og_rng_range",
  "og_rng_integer",
  "og_rng_chance",
  "og_rng_fork",
  "og_input_down",
  "og_input_pressed",
  "og_input_released",
  "og_input_axis",
  "og_input_axis2",
  "og_input_button_down",
  "og_input_button_pressed",
  "og_input_button_released",
  "og_camera_look_at",
  "og_camera_world_to_screen",
  "og_camera_screen_to_ray",
  "og_render_sprite",
  "og_render_measure_text",
  "og_render_box_geometry",
  "og_render_plane_geometry",
  "og_render_sphere_geometry",
  "og_render_standard_material",
  "og_render_update_material",
  "og_render_capabilities",
  "og_render_text",
  "og_load_bundle",
  "og_load_bundle_info",
  "og_load_default_bundle",
  "og_load_gltf",
  "og_load_scene",
  "og_load_font",
  "og_load_font_info",
  "og_load_sound",
  "og_load_texture",
  "og_load_sound_info",
  "og_load_status",
  "og_load_error",
  "og_load_gltf_info",
  "og_physics_raycast",
  "og_physics_shape_cast",
  "og_physics_overlap",
  "og_physics_set_mesh",
  "og_ui_configure",
  "og_ui_viewport",
  "og_ui_theme",
  "og_ui_open",
  "og_ui_open_panel",
  "og_ui_open_scroll",
  "og_ui_close",
  "og_ui_label",
  "og_ui_button",
  "og_ui_image",
  "og_ui_progress",
  "og_ui_slider",
  "og_ui_toggle",
  "og_ui_text_field",
  "og_ui_tooltip",
  "og_commands_register",
  "og_commands_parse",
  "og_commands_help",
  "og_commands_schedule",
  "og_runtime_probe_get_version",
  "og_runtime_probe_tick",
  "og_runtime_probe_state",
  "og_runtime_probe_advance",
  "og_runtime_probe_pause",
  "og_runtime_probe_resume",
  "og_runtime_probe_input",
  "og_runtime_probe_accepted",
  "og_runtime_probe_pick",
  "og_runtime_probe_entity",
  "og_runtime_probe_bounds",
  "og_runtime_probe_patch",
  "og_runtime_probe_run",
  "og_engine_probe_hello",
  "og_engine_probe_capture",
  "og_engine_probe_sweep",
  "og_engine_probe_reload",
  "og_engine_probe_snapshot",
  "og_engine_probe_restore",
  "og_engine_probe_dump",
  "og_engine_probe_commands",
  "og_host_set_dispatcher",
  "og_engine_create",
  "og_engine_destroy",
  "og_engine_advance",
  "og_engine_command",
  "og_engine_frame",
  "og_host_begin_bundle",
  "og_engine_call",
  "og_engine_take_outputs",
  "og_engine_take_log",
  "og_engine_write_resource",
];
