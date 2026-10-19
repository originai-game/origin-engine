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
  /** The files the command channel produced since the last call (`screenshot`, `dump_*`, `help`), as JSON `[{ tick, action, file, kind, data }]`: `file` is og-probe's `Output::file_name()`, `kind` is `png`, `json`, `text` or `wav`, `data` is base64 for a PNG or a WAV and the text otherwise. The host writes them (og run) or posts them. */
  og_engine_take_outputs(engine: number): string;
  /** Log lines since the last call, as JSON `[{ tick, warning, text }]`: the command channel's and the probe's warnings and the loader's, each naming the fix. */
  og_engine_take_log(engine: number): string;
  /** Write some fields of a resource: how the runtime makes `world.resource(R).value` a live value. A Result. */
  og_engine_write_resource(world: number, resource: string, value: string): string;
  /** The platform calls (IDL `platform`) made since the last call, oldest first, as JSON `[{ request, method, args, tick }]`. The host answers each but `telemetry` (one-way) with og_platform_answer. */
  og_platform_take(engine: number): string;
  /** Answer one platform call: JSON `{ ok: true, data }` or `{ ok: false, error: { code, message, retryable } }`. It is applied at the start of the next tick (og.platform sends PlatformAnswered). False for an unknown, one-way or answered request, or a malformed answer. */
  og_platform_answer(engine: number, request: number, answer: string): boolean;
  /** Say what answers this engine's platform calls: JSON `{ host, capabilities }` (`player`, `local` or `none`; the methods it answers), as og.platform.info reports. */
  og_platform_set_host(engine: number, info: string): void;
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
  /** `og.time.setScale` */
  og_time_set_scale(world: number, scale: number): string;
  /** `og.time.hitStop` */
  og_time_hit_stop(world: number, ticks: number): string;
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
  /** `Pointers.down` */
  og_pointers_down(world: number, id: number): boolean;
  /** `Pointers.pressed` */
  og_pointers_pressed(world: number, id: number): boolean;
  /** `Pointers.released` */
  og_pointers_released(world: number, id: number): boolean;
  /** `Pointers.position` */
  og_pointers_position(world: number, id: number): string;
  /** `Pointers.start` */
  og_pointers_start(world: number, id: number): string;
  /** `Pointers.count` */
  og_pointers_count(world: number): number;
  /** `Gamepad.connected` */
  og_gamepad_connected(world: number, pad: number | undefined): boolean;
  /** `Gamepad.down` */
  og_gamepad_down(world: number, button: string, pad: number | undefined): boolean;
  /** `Gamepad.pressed` */
  og_gamepad_pressed(world: number, button: string, pad: number | undefined): boolean;
  /** `Gamepad.released` */
  og_gamepad_released(world: number, button: string, pad: number | undefined): boolean;
  /** `Gamepad.axis` */
  og_gamepad_axis(world: number, axis: string, pad: number | undefined): number;
  /** `Camera.lookAt` */
  og_camera_look_at(world: number, target: string): void;
  /** `Camera.worldToScreen` */
  og_camera_world_to_screen(world: number, point: string): string;
  /** `Camera.screenToRay` */
  og_camera_screen_to_ray(world: number, x: number, y: number): string;
  /** `og.camera.shake` */
  og_camera_shake(world: number, trauma: number): string;
  /** `og.camera.punch` */
  og_camera_punch(world: number, amount: number): string;
  /** `og.camera.kick` */
  og_camera_kick(world: number, direction: string, distance: number): string;
  /** `og.render.ribbon` */
  og_render_ribbon(world: number, desc: string | undefined): string;
  /** `og.render.sprite` */
  og_render_sprite(world: number, desc: string | undefined): string;
  /** `og.render.floatingText` */
  og_render_floating_text(world: number, desc: string | undefined): string;
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
  /** `og.render.shaderMaterial` */
  og_render_shader_material(world: number, shader: number, desc: string | undefined): string;
  /** `og.render.setShaderParam` */
  og_render_set_shader_param(world: number, material: number, name: string, value: string): string;
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
  /** `og.load.lut` */
  og_load_lut(world: number, path: string): string;
  /** `og.load.emitter` */
  og_load_emitter(world: number, path: string): string;
  /** `og.load.shader` */
  og_load_shader(world: number, path: string): string;
  /** `og.load.texture` */
  og_load_texture(world: number, path: string): string;
  /** `og.load.skin` */
  og_load_skin(world: number, path: string): string;
  /** `og.load.soundInfo` */
  og_load_sound_info(world: number, sound: number): string;
  /** `og.load.status` */
  og_load_status(world: number, asset: number): string;
  /** `og.load.error` */
  og_load_error(world: number, asset: number): string;
  /** `og.load.gltfInfo` */
  og_load_gltf_info(world: number, asset: number): string;
  /** `og.load.animation` */
  og_load_animation(world: number, path: string): string;
  /** `og.load.flipbook` */
  og_load_flipbook(world: number, path: string): string;
  /** `og.load.tilemap` */
  og_load_tilemap(world: number, path: string): string;
  /** `og.physics.raycast` */
  og_physics_raycast(world: number, desc: string | undefined): string;
  /** `og.physics.shapeCast` */
  og_physics_shape_cast(world: number, desc: string | undefined): string;
  /** `og.physics.overlap` */
  og_physics_overlap(world: number, desc: string | undefined): string;
  /** `og.physics.setMesh` */
  og_physics_set_mesh(world: number, mesh: number, desc: string | undefined): string;
  /** `og.physics.ropePoints` */
  og_physics_rope_points(world: number, entity: number): string;
  /** `og.physics.clothPoints` */
  og_physics_cloth_points(world: number, entity: number): string;
  /** `og.physics.fluidParticles` */
  og_physics_fluid_particles(world: number, entity: number): string;
  /** `og.physics.waterHeight` */
  og_physics_water_height(world: number, entity: number, x: number, z: number): string;
  /** `og.physics.vehicleWheels` */
  og_physics_vehicle_wheels(world: number, entity: number): string;
  /** `og.physics.ragdollRigFromSkeleton` */
  og_physics_ragdoll_rig_from_skeleton(world: number, skeleton: number): string;
  /** `og.physics.setRagdollRig` */
  og_physics_set_ragdoll_rig(world: number, entity: number, desc: string | undefined): string;
  /** `og.physics.ragdollPose` */
  og_physics_ragdoll_pose(world: number, entity: number): string;
  /** `og.atmosphere.timeOfDay` */
  og_atmosphere_time_of_day(world: number): number;
  /** `og.anim.graph` */
  og_anim_graph(world: number, desc: string | undefined): string;
  /** `og.tween.to` */
  og_tween_to(world: number, desc: string | undefined): string;
  /** `og.tween.sequence` */
  og_tween_sequence(world: number, steps: string, desc: string | undefined): string;
  /** `og.tween.parallel` */
  og_tween_parallel(world: number, tweens: string, desc: string | undefined): string;
  /** `og.tween.cancel` */
  og_tween_cancel(world: number, tween: number): boolean;
  /** `og.tween.cancelEntity` */
  og_tween_cancel_entity(world: number, entity: number): number;
  /** `og.tween.active` */
  og_tween_active(world: number, tween: number): boolean;
  /** `og.tween.squash` */
  og_tween_squash(world: number, desc: string | undefined): string;
  /** `og.tween.flash` */
  og_tween_flash(world: number, desc: string | undefined): string;
  /** `og.tilemap.setTile` */
  og_tilemap_set_tile(world: number, desc: string | undefined): string;
  /** `og.tilemap.getTile` */
  og_tilemap_get_tile(world: number, entity: number, layer: string, x: number, y: number): number;
  /** `og.tilemap.cellAt` */
  og_tilemap_cell_at(world: number, entity: number, point: string): string;
  /** `og.tilemap.cellCenter` */
  og_tilemap_cell_center(world: number, entity: number, x: number, y: number): string;
  /** `og.nav.grid` */
  og_nav_grid(world: number, desc: string | undefined): string;
  /** `og.nav.gridFromTilemap` */
  og_nav_grid_from_tilemap(world: number, desc: string | undefined): string;
  /** `og.nav.setCost` */
  og_nav_set_cost(world: number, grid: number, x: number, y: number, cost: number): string;
  /** `og.nav.cellAt` */
  og_nav_cell_at(world: number, grid: number, point: string): string;
  /** `og.nav.findPath` */
  og_nav_find_path(world: number, desc: string | undefined): string;
  /** `og.nav.flowField` */
  og_nav_flow_field(world: number, desc: string | undefined): string;
  /** `og.nav.setFlowTargets` */
  og_nav_set_flow_targets(world: number, field: number, targets: string): string;
  /** `og.nav.flowAt` */
  og_nav_flow_at(world: number, field: number, point: string): string;
  /** `og.nav.load` */
  og_nav_load(world: number, path: string): string;
  /** `og.nav.build` */
  og_nav_build(world: number, desc: string | undefined): string;
  /** `og.nav.setObstacle` */
  og_nav_set_obstacle(world: number, mesh: number, id: number, obstacle: string | undefined): string;
  /** `og.nav.nearest` */
  og_nav_nearest(world: number, mesh: number, point: string, extent: string | undefined): string;
  /** `og.nav.pathOf` */
  og_nav_path_of(world: number, entity: number): string;
  /** `og.ai.load` */
  og_ai_load(world: number, path: string): string;
  /** `og.ai.parse` */
  og_ai_parse(world: number, text: string): string;
  /** `og.ai.get` */
  og_ai_get(world: number, entity: number, key: string): string;
  /** `og.ai.set` */
  og_ai_set(world: number, entity: number, key: string, value: string): string;
  /** `og.ai.noise` */
  og_ai_noise(world: number, desc: string | undefined): string;
  /** `og.ai.describe` */
  og_ai_describe(world: number, entity: number): string;
  /** `og.platform.storeGet` */
  og_platform_store_get(world: number, key: string): string;
  /** `og.platform.storeSet` */
  og_platform_store_set(world: number, desc: string | undefined): string;
  /** `og.platform.storeList` */
  og_platform_store_list(world: number): number;
  /** `og.platform.leaderboardSubmit` */
  og_platform_leaderboard_submit(world: number, desc: string | undefined): string;
  /** `og.platform.leaderboardTop` */
  og_platform_leaderboard_top(world: number, desc: string | undefined): number;
  /** `og.platform.leaderboardAround` */
  og_platform_leaderboard_around(world: number, desc: string | undefined): number;
  /** `og.platform.telemetry` */
  og_platform_telemetry(world: number, name: string, attrs: string | undefined): string;
  /** `og.platform.point` */
  og_platform_point(world: number, desc: string | undefined): string;
  /** `og.platform.aiMark` */
  og_platform_ai_mark(world: number, desc: string | undefined): string;
  /** `og.platform.loadSettings` */
  og_platform_load_settings(world: number): number;
  /** `og.platform.saveSettings` */
  og_platform_save_settings(world: number, value: string): string;
  /** `og.platform.result` */
  og_platform_result(world: number, request: number): string;
  /** `og.platform.info` */
  og_platform_info(world: number): string;
  /** `og.i18n.load` */
  og_i18n_load(world: number, path: string): string;
  /** `og.i18n.setLocale` */
  og_i18n_set_locale(world: number, locale: string): string;
  /** `og.i18n.locale` */
  og_i18n_locale(world: number): string;
  /** `og.i18n.t` */
  og_i18n_t(world: number, key: string, args: string | undefined): string;
  /** `og.i18n.has` */
  og_i18n_has(world: number, key: string): boolean;
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
  /** `og.ui.breakpoint` */
  og_ui_breakpoint(): string;
  /** `og.ui.frameViewport` */
  og_ui_frame_viewport(): string;
  /** `og.ui.hasIcon` */
  og_ui_has_icon(name: string): boolean;
  /** `og.ui.seconds` */
  og_ui_seconds(): number;
  /** `og.ui.frameTheme` */
  og_ui_frame_theme(): string;
  /** `og.ui.openAnchor` */
  og_ui_open_anchor(id: string, desc: string | undefined): void;
  /** `og.ui.openTransition` */
  og_ui_open_transition(id: string, show: boolean, desc: string | undefined): string;
  /** `og.ui.openModal` */
  og_ui_open_modal(id: string, open: boolean, desc: string | undefined): string;
  /** `og.ui.closeModal` */
  og_ui_close_modal(): void;
  /** `og.ui.spotlight` */
  og_ui_spotlight(id: string, hole: string, desc: string | undefined): string;
  /** `og.ui.bounds` */
  og_ui_bounds(path: string): string;
  /** `og.ui.openList` */
  og_ui_open_list(id: string, desc: string | undefined): string;
  /** `og.ui.openGrid` */
  og_ui_open_grid(id: string, desc: string | undefined): string;
  /** `og.ui.listItem` */
  og_ui_list_item(index: number, selected: boolean): string;
  /** `og.ui.gridItem` */
  og_ui_grid_item(index: number, selected: boolean): string;
  /** `og.ui.closeList` */
  og_ui_close_list(): string;
  /** `og.ui.dragging` */
  og_ui_dragging(id: string): string;
  /** `og.ui.tabs` */
  og_ui_tabs(id: string, items: string, selected: number, desc: string | undefined): string;
  /** `og.ui.select` */
  og_ui_select(id: string, options: string, selected: number, desc: string | undefined): string;
  /** `og.ui.stepper` */
  og_ui_stepper(id: string, value: number, desc: string | undefined): string;
  /** `og.ui.radio` */
  og_ui_radio(id: string, options: string, selected: number, desc: string | undefined): string;
  /** `og.ui.checkbox` */
  og_ui_checkbox(id: string, on: boolean, label: string): string;
  /** `og.ui.checkboxList` */
  og_ui_checkbox_list(id: string, labels: string, values: string): string;
  /** `og.ui.chip` */
  og_ui_chip(id: string, text: string, desc: string | undefined): string;
  /** `og.ui.badge` */
  og_ui_badge(id: string, text: string, tone: string): void;
  /** `og.ui.divider` */
  og_ui_divider(id: string, vertical: boolean): void;
  /** `og.ui.spacer` */
  og_ui_spacer(id: string, size: string): void;
  /** `og.ui.keyValue` */
  og_ui_key_value(id: string, key: string, value: string, mono: boolean): void;
  /** `og.ui.skeleton` */
  og_ui_skeleton(id: string, desc: string | undefined): void;
  /** `og.ui.spinner` */
  og_ui_spinner(id: string, size: number): void;
  /** `og.ui.icon` */
  og_ui_icon(id: string, name: string, desc: string | undefined): void;
  /** `og.ui.meter` */
  og_ui_meter(id: string, value: number, max: number, desc: string | undefined): void;
  /** `og.ui.cooldown` */
  og_ui_cooldown(id: string, remaining: number, desc: string | undefined): string;
  /** `og.ui.minimap` */
  og_ui_minimap(id: string, desc: string | undefined): void;
  /** `og.ui.dialogue` */
  og_ui_dialogue(id: string, desc: string | undefined): string;
  /** `og.ui.joystick` */
  og_ui_joystick(id: string, desc: string | undefined): string;
  /** `og.ui.dpad` */
  og_ui_dpad(id: string, desc: string | undefined): string;
  /** `og.ui.actionButton` */
  og_ui_action_button(id: string, text: string, desc: string | undefined): string;
  /** `og.ui.toast` */
  og_ui_toast(world: number, text: string, desc: string | undefined): number;
  /** `og.ui.dismissToast` */
  og_ui_dismiss_toast(world: number, toast_id: number): void;
  /** `og.ui.navigate` */
  og_ui_navigate(world: number, intent: string): void;
  /** `og.ui.back` */
  og_ui_back(): boolean;
  /** `og.ui.setNavigation` */
  og_ui_set_navigation(on: boolean): void;
  /** `og.ui.memory` */
  og_ui_memory(id: string, initial: number): number;
  /** `og.ui.remember` */
  og_ui_remember(id: string, value: number): void;
  /** `og.ui.tick` */
  og_ui_tick(): number;
  /** `og.ui.counter` */
  og_ui_counter(id: string, value: number, desc: string | undefined): number;
  /** `og.ui.burst` */
  og_ui_burst(desc: string | undefined): void;
  /** `og.ui.shake` */
  og_ui_shake(desc: string | undefined): void;
  /** `og.ui.screenTransition` */
  og_ui_screen_transition(id: string, screen: string, desc: string | undefined): string;
  /** `og.ui.previewViewport` */
  og_ui_preview_viewport(world: number, viewport: string): void;
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
  /** `EngineProbe.listen` */
  og_engine_probe_listen(engine_probe: number, seconds: number | undefined): string;
  /** `og.terrain.load` */
  og_terrain_load(world: number, path: string): string;
  /** `og.terrain.sample` */
  og_terrain_sample(world: number, desc: string | undefined): string;
  /** `og.terrain.bounds` */
  og_terrain_bounds(world: number, data: number, origin: string | undefined, quaternion: string | undefined, scale: string | undefined): string;
  /** `og.terrain.edit` */
  og_terrain_edit(world: number, data: number, brush: string | undefined): string;
  /** `og.terrain.layer` */
  og_terrain_layer(world: number, data: number, index: number): string;
  /** `og.terrain.setLayer` */
  og_terrain_set_layer(world: number, data: number, index: number, layer: string | undefined): string;
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
  "og_time_set_scale",
  "og_time_hit_stop",
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
  "og_pointers_down",
  "og_pointers_pressed",
  "og_pointers_released",
  "og_pointers_position",
  "og_pointers_start",
  "og_pointers_count",
  "og_gamepad_connected",
  "og_gamepad_down",
  "og_gamepad_pressed",
  "og_gamepad_released",
  "og_gamepad_axis",
  "og_camera_look_at",
  "og_camera_world_to_screen",
  "og_camera_screen_to_ray",
  "og_camera_shake",
  "og_camera_punch",
  "og_camera_kick",
  "og_render_ribbon",
  "og_render_sprite",
  "og_render_floating_text",
  "og_render_measure_text",
  "og_render_box_geometry",
  "og_render_plane_geometry",
  "og_render_sphere_geometry",
  "og_render_standard_material",
  "og_render_update_material",
  "og_render_shader_material",
  "og_render_set_shader_param",
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
  "og_load_lut",
  "og_load_emitter",
  "og_load_shader",
  "og_load_texture",
  "og_load_skin",
  "og_load_sound_info",
  "og_load_status",
  "og_load_error",
  "og_load_gltf_info",
  "og_load_animation",
  "og_load_flipbook",
  "og_load_tilemap",
  "og_physics_raycast",
  "og_physics_shape_cast",
  "og_physics_overlap",
  "og_physics_set_mesh",
  "og_physics_rope_points",
  "og_physics_cloth_points",
  "og_physics_fluid_particles",
  "og_physics_water_height",
  "og_physics_vehicle_wheels",
  "og_physics_ragdoll_rig_from_skeleton",
  "og_physics_set_ragdoll_rig",
  "og_physics_ragdoll_pose",
  "og_atmosphere_time_of_day",
  "og_anim_graph",
  "og_tween_to",
  "og_tween_sequence",
  "og_tween_parallel",
  "og_tween_cancel",
  "og_tween_cancel_entity",
  "og_tween_active",
  "og_tween_squash",
  "og_tween_flash",
  "og_tilemap_set_tile",
  "og_tilemap_get_tile",
  "og_tilemap_cell_at",
  "og_tilemap_cell_center",
  "og_nav_grid",
  "og_nav_grid_from_tilemap",
  "og_nav_set_cost",
  "og_nav_cell_at",
  "og_nav_find_path",
  "og_nav_flow_field",
  "og_nav_set_flow_targets",
  "og_nav_flow_at",
  "og_nav_load",
  "og_nav_build",
  "og_nav_set_obstacle",
  "og_nav_nearest",
  "og_nav_path_of",
  "og_ai_load",
  "og_ai_parse",
  "og_ai_get",
  "og_ai_set",
  "og_ai_noise",
  "og_ai_describe",
  "og_platform_store_get",
  "og_platform_store_set",
  "og_platform_store_list",
  "og_platform_leaderboard_submit",
  "og_platform_leaderboard_top",
  "og_platform_leaderboard_around",
  "og_platform_telemetry",
  "og_platform_point",
  "og_platform_ai_mark",
  "og_platform_load_settings",
  "og_platform_save_settings",
  "og_platform_result",
  "og_platform_info",
  "og_i18n_load",
  "og_i18n_set_locale",
  "og_i18n_locale",
  "og_i18n_t",
  "og_i18n_has",
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
  "og_ui_breakpoint",
  "og_ui_frame_viewport",
  "og_ui_has_icon",
  "og_ui_seconds",
  "og_ui_frame_theme",
  "og_ui_open_anchor",
  "og_ui_open_transition",
  "og_ui_open_modal",
  "og_ui_close_modal",
  "og_ui_spotlight",
  "og_ui_bounds",
  "og_ui_open_list",
  "og_ui_open_grid",
  "og_ui_list_item",
  "og_ui_grid_item",
  "og_ui_close_list",
  "og_ui_dragging",
  "og_ui_tabs",
  "og_ui_select",
  "og_ui_stepper",
  "og_ui_radio",
  "og_ui_checkbox",
  "og_ui_checkbox_list",
  "og_ui_chip",
  "og_ui_badge",
  "og_ui_divider",
  "og_ui_spacer",
  "og_ui_key_value",
  "og_ui_skeleton",
  "og_ui_spinner",
  "og_ui_icon",
  "og_ui_meter",
  "og_ui_cooldown",
  "og_ui_minimap",
  "og_ui_dialogue",
  "og_ui_joystick",
  "og_ui_dpad",
  "og_ui_action_button",
  "og_ui_toast",
  "og_ui_dismiss_toast",
  "og_ui_navigate",
  "og_ui_back",
  "og_ui_set_navigation",
  "og_ui_memory",
  "og_ui_remember",
  "og_ui_tick",
  "og_ui_counter",
  "og_ui_burst",
  "og_ui_shake",
  "og_ui_screen_transition",
  "og_ui_preview_viewport",
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
  "og_engine_probe_listen",
  "og_terrain_load",
  "og_terrain_sample",
  "og_terrain_bounds",
  "og_terrain_edit",
  "og_terrain_layer",
  "og_terrain_set_layer",
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
  "og_platform_take",
  "og_platform_answer",
  "og_platform_set_host",
];
