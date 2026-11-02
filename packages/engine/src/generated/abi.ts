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
  /** Write some fields of a resource as JSON: the runtime's path for a write a publish build refuses (logged once per system). A Result. */
  og_engine_write_resource(world: number, resource: string, value: string): string;
  /** A resource's packed layout, how the runtime reads and writes its fields as bytes: a Result of `{ epoch, size, fields: [{ name, kind, offset, size, readonly, capacity, variants, element }] }` (`kind`: f32, f64, u32, i32, bool, Vector2, Vector3, Quaternion, Color, Matrix4, handle, str, enum, or list: `element` one of f32, f64, u32, i32, handle and `capacity` its elements, a count lane first, each lane the element's size). Refused for og.Time and og.Rng, whose handles are computed, and for what is not a registered resource. */
  og_engine_resource_layout(world: number, resource: string): string;
  /** A resource's packed bytes, then the layout epoch as a little-endian u32 (it moves when any layout is declared again: a new engine, a reload). Empty when refused (undeclared in the running system, not a resource): the runtime then asks og_world_resource for the reason. */
  og_engine_read_resource(world: number, resource: string): Uint8Array;
  /** Overwrite a resource with packed bytes followed by the layout epoch they were made with (as og_engine_read_resource answers): how the runtime flushes a handle's field writes, once before the next engine call and at the end of every system. Read-only fields keep their value. A Result; refused for a stale epoch, a wrong size, an invalid value or a resource the running system did not declare among its writes. */
  og_engine_write_resource_bytes(world: number, resource: string, bytes: Uint8Array): string;
  /** The platform calls (IDL `platform`) made since the last call, oldest first, as JSON `[{ request, method, args, tick }]`. The host answers each but `telemetry` (one-way) with og_platform_answer. */
  og_platform_take(engine: number): string;
  /** Answer one platform call: JSON `{ ok: true, data }` or `{ ok: false, error: { code, message, retryable } }`. It is applied at the start of the next tick (og.platform sends PlatformAnswered). False for an unknown, one-way or answered request, or a malformed answer. */
  og_platform_answer(engine: number, request: number, answer: string): boolean;
  /** Say what answers this engine's platform calls: JSON `{ host, capabilities }` (`player`, `local` or `none`; the methods it answers), as og.platform.info reports. */
  og_platform_set_host(engine: number, info: string): void;
  /** Unasked host input for og.platform (docs/platform-asks.md #18): JSON `{ kind, data, time? }`, kind `purchase` (a purchase settled: `{ purchase, product, quantity, state }`), `entitlements` (`{ entitlements }`, changed without a call) or `invite` (`{ invite, from, payload, time }`). Applied at the start of the next tick (og.PlatformNotice) and logged with the answers. False for a malformed notice or an unknown kind. */
  og_platform_notify(engine: number, notice: string): boolean;
  /** The room ops (IDL `platform` rooms, og.net's lockstep) made since the last call, oldest first, as JSON `[op]`: `{ op: "create", name, capacity }`, `{ op: "join", room, name }`, `{ op: "leave" }`, `{ op: "send", to?, channel?, data? | bytes? }` (bytes base64; a channel starting `og.` is the engine's own). The host carries each in order (docs/design.md §2.8). */
  og_room_take(engine: number): string;
  /** One room event from the host, JSON: `{ type: "joined", room, peer, peers: [{ peer, name }] }`, `{ type: "peer_joined", peer, name }`, `{ type: "peer_left", peer }`, `{ type: "message", from, channel?, data? | bytes? }`, `{ type: "left", reason }` or `{ type: "error", op, code, message }`. Lockstep's messages act between ticks at once; the game's events are sent at the start of the next tick. False for a malformed event. */
  og_room_deliver(engine: number, event: string): boolean;
  /** Say what carries this engine's rooms: JSON `{ host, join? }` (`relay`, `player` or `none`), as og.platform.roomInfo reports; with `none` roomCreate and roomJoin are refused. `join` is the room a Play link named (`?room=<id>`): the engine joins it once, itself, and og.RoomJoined answers. */
  og_room_set_host(engine: number, info: string): void;
  /** `defineComponent` */
  og_reflect_define_component(name: string, fields: string): string;
  /** `defineResource` */
  og_reflect_define_resource(name: string, fields: string): string;
  /** `SaveEdit.version` */
  og_save_edit_get_version(save_edit: number): number;
  /** `SaveEdit.types` */
  og_save_edit_types(save_edit: number): string;
  /** `SaveEdit.records` */
  og_save_edit_records(save_edit: number, name: string): string;
  /** `SaveEdit.setRecords` */
  og_save_edit_set_records(save_edit: number, name: string, desc: string | undefined): string;
  /** `SaveEdit.renameType` */
  og_save_edit_rename_type(save_edit: number, from: string, to: string): string;
  /** `SaveEdit.renameField` */
  og_save_edit_rename_field(save_edit: number, name: string, from: string, to: string): string;
  /** `SaveEdit.dropType` */
  og_save_edit_drop_type(save_edit: number, name: string): string;
  /** `SaveEdit.spawn` */
  og_save_edit_spawn(save_edit: number, desc: string | undefined): string;
  /** `SaveEdit.text` */
  og_save_edit_text(save_edit: number): string;
  /** `SaveEdit.close` */
  og_save_edit_close(save_edit: number): void;
  /** `SpatialIndex.count` */
  og_spatial_index_get_count(spatial_index: number): number;
  /** `SpatialIndex.within` */
  og_spatial_index_within(spatial_index: number, center: string, radius: number): string;
  /** `SpatialIndex.nearest` */
  og_spatial_index_nearest(spatial_index: number, center: string, count: number): string;
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
  /** `World.children` */
  og_world_children(world: number, entity: number): string;
  /** `World.removed` */
  og_world_removed(world: number, component: string): string;
  /** `World.spatial` */
  og_world_spatial(world: number, desc: string | undefined): string;
  /** `World.state` */
  og_world_state(world: number, name: string): string;
  /** `World.setState` */
  og_world_set_state(world: number, name: string, value: string): string;
  /** `World.find` */
  og_world_find(world: number, name: string): string;
  /** `World.prefabOf` */
  og_world_prefab_of(world: number, entity: number): string;
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
  /** `World.snapshot` */
  og_world_snapshot(world: number, desc: string | undefined): string;
  /** `World.openSave` */
  og_world_open_save(world: number, save: string): string;
  /** `World.restore` */
  og_world_restore(world: number, save: string, desc: string | undefined): string;
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
  /** `ArchetypeView.rows` */
  og_archetype_view_get_rows(archetype_view: number): A;
  /** `ArchetypeView.entity` */
  og_archetype_view_entity(archetype_view: number, index: number): number;
  /** `ArchetypeView.has` */
  og_archetype_view_has(archetype_view: number, component: string): boolean;
  /** `ArchetypeView.column` */
  og_archetype_view_column(archetype_view: number, component: string): V;
  /** `Hook.name` */
  og_hook_get_name(hook: number): string;
  /** `defineHook` */
  og_system_define_hook(name: string, component: string, desc: string, run: number): number;
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
  /** `og.time.setGroupScale` */
  og_time_set_group_scale(world: number, group: number, scale: number): string;
  /** `og.time.groupDt` */
  og_time_group_dt(world: number, entity: number): number;
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
  /** `RngStream.weighted` */
  og_rng_stream_weighted(rng_stream: number, weights: string): number;
  /** `RngStream.permutation` */
  og_rng_stream_permutation(rng_stream: number, n: number): string;
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
  /** `Rng.weighted` */
  og_rng_weighted(world: number, weights: string): number;
  /** `Rng.permutation` */
  og_rng_permutation(world: number, n: number): string;
  /** `Rng.fork` */
  og_rng_fork(world: number, label: string): number;
  /** `og.procgen.noise` */
  og_procgen_noise(desc: string, x: number, y: number, z: number): number;
  /** `og.procgen.noiseGrid` */
  og_procgen_noise_grid(desc: string | undefined): string;
  /** `og.procgen.poissonDisk` */
  og_procgen_poisson_disk(desc: string | undefined): string;
  /** `og.procgen.bsp` */
  og_procgen_bsp(desc: string | undefined): string;
  /** `og.procgen.wfc` */
  og_procgen_wfc(desc: string | undefined): string;
  /** `og.input.defineAction` */
  og_input_define_action(world: number, desc: string | undefined): string;
  /** `og.input.actions` */
  og_input_actions(world: number): string;
  /** `og.input.action` */
  og_input_action(world: number, name: string): string;
  /** `og.input.actionDown` */
  og_input_action_down(world: number, name: string): boolean;
  /** `og.input.actionPressed` */
  og_input_action_pressed(world: number, name: string): boolean;
  /** `og.input.actionReleased` */
  og_input_action_released(world: number, name: string): boolean;
  /** `og.input.actionHeld` */
  og_input_action_held(world: number, name: string): boolean;
  /** `og.input.actionValue` */
  og_input_action_value(world: number, name: string): number;
  /** `og.input.actionVector` */
  og_input_action_vector(world: number, name: string): string;
  /** `og.input.setContexts` */
  og_input_set_contexts(world: number, contexts: string): void;
  /** `og.input.contexts` */
  og_input_contexts(world: number): string;
  /** `og.input.rebind` */
  og_input_rebind(world: number, desc: string | undefined): string;
  /** `og.input.conflicts` */
  og_input_conflicts(world: number, desc: string | undefined): string;
  /** `og.input.resetBindings` */
  og_input_reset_bindings(world: number, action: string | undefined): string;
  /** `og.input.setToggle` */
  og_input_set_toggle(world: number, action: string, on: boolean): string;
  /** `og.input.saveBindings` */
  og_input_save_bindings(world: number): string;
  /** `og.input.loadBindings` */
  og_input_load_bindings(world: number, text: string): string;
  /** `og.input.startCapture` */
  og_input_start_capture(world: number, desc: string | undefined): string;
  /** `og.input.capture` */
  og_input_capture(world: number): string;
  /** `og.input.cancelCapture` */
  og_input_cancel_capture(world: number): void;
  /** `og.input.applyCapture` */
  og_input_apply_capture(world: number, policy: string | undefined): string;
  /** `og.input.glyph` */
  og_input_glyph(world: number, action: string, device: string | undefined): string;
  /** `og.input.glyphs` */
  og_input_glyphs(world: number, action: string, device: string | undefined): string;
  /** `og.input.controlGlyph` */
  og_input_control_glyph(control: string, device: string | undefined): string;
  /** `og.input.device` */
  og_input_device(world: number): string;
  /** `og.input.padDevice` */
  og_input_pad_device(world: number, pad: number): string;
  /** `og.input.setDeadzones` */
  og_input_set_deadzones(world: number, desc: string | undefined): string;
  /** `og.input.setSensitivity` */
  og_input_set_sensitivity(world: number, desc: string | undefined): string;
  /** `og.input.sensitivity` */
  og_input_sensitivity(world: number, action: string, device: string | undefined): string;
  /** `og.input.setKeyboardLayout` */
  og_input_set_keyboard_layout(labels: string): void;
  /** `og.input.keyboardLayout` */
  og_input_keyboard_layout(): string;
  /** `og.input.setMouse` */
  og_input_set_mouse(world: number, desc: string | undefined): string;
  /** `og.input.mouse` */
  og_input_mouse(world: number): string;
  /** `og.input.rumble` */
  og_input_rumble(world: number, desc: string | undefined): string;
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
  /** `og.render.snap` */
  og_render_snap(world: number, entity: number): string;
  /** `og.render.snapCamera` */
  og_render_snap_camera(world: number): void;
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
  /** `og.render.setMaterialShader` */
  og_render_set_material_shader(world: number, material: number, shader: string): string;
  /** `og.render.setShaderParam` */
  og_render_set_shader_param(world: number, material: number, name: string, value: string): string;
  /** `og.render.applyLook` */
  og_render_apply_look(world: number, look: number): string;
  /** `og.render.applyQuality` */
  og_render_apply_quality(world: number, preset: string): string;
  /** `og.render.setDisplay` */
  og_render_set_display(world: number, settings: string | undefined): string;
  /** `og.render.resetDisplay` */
  og_render_reset_display(world: number): string;
  /** `og.render.display` */
  og_render_display(world: number): string;
  /** `og.render.look` */
  og_render_look(world: number): string;
  /** `og.render.capabilities` */
  og_render_capabilities(world: number): string;
  /** `og.render.text` */
  og_render_text(world: number, desc: string | undefined): string;
  /** `og.render.bufferGeometry` */
  og_render_buffer_geometry(world: number, desc: string | undefined): string;
  /** `og.render.updateGeometry` */
  og_render_update_geometry(world: number, geometry: number, desc: string | undefined): string;
  /** `og.render.geometryInfo` */
  og_render_geometry_info(world: number, geometry: number): string;
  /** `og.render.cylinderGeometry` */
  og_render_cylinder_geometry(world: number, desc: string | undefined): string;
  /** `og.render.coneGeometry` */
  og_render_cone_geometry(world: number, desc: string | undefined): string;
  /** `og.render.capsuleGeometry` */
  og_render_capsule_geometry(world: number, desc: string | undefined): string;
  /** `og.render.torusGeometry` */
  og_render_torus_geometry(world: number, desc: string | undefined): string;
  /** `og.render.ringGeometry` */
  og_render_ring_geometry(world: number, desc: string | undefined): string;
  /** `og.render.circleGeometry` */
  og_render_circle_geometry(world: number, desc: string | undefined): string;
  /** `og.render.tubeGeometry` */
  og_render_tube_geometry(world: number, desc: string | undefined): string;
  /** `og.render.shapeGeometry` */
  og_render_shape_geometry(world: number, desc: string | undefined): string;
  /** `og.render.extrudeGeometry` */
  og_render_extrude_geometry(world: number, desc: string | undefined): string;
  /** `og.render.latheGeometry` */
  og_render_lathe_geometry(world: number, desc: string | undefined): string;
  /** `og.render.polyhedronGeometry` */
  og_render_polyhedron_geometry(world: number, desc: string | undefined): string;
  /** `og.render.icosahedronGeometry` */
  og_render_icosahedron_geometry(world: number, desc: string | undefined): string;
  /** `og.render.dodecahedronGeometry` */
  og_render_dodecahedron_geometry(world: number, desc: string | undefined): string;
  /** `og.render.octahedronGeometry` */
  og_render_octahedron_geometry(world: number, desc: string | undefined): string;
  /** `og.render.tetrahedronGeometry` */
  og_render_tetrahedron_geometry(world: number, desc: string | undefined): string;
  /** `og.render.sweepGeometry` */
  og_render_sweep_geometry(world: number, desc: string | undefined): string;
  /** `og.render.csgGeometry` */
  og_render_csg_geometry(world: number, desc: string | undefined): string;
  /** `og.render.mergeGeometries` */
  og_render_merge_geometries(world: number, desc: string | undefined): string;
  /** `og.render.lines` */
  og_render_lines(world: number, desc: string | undefined): string;
  /** `og.render.points` */
  og_render_points(world: number, desc: string | undefined): string;
  /** `og.render.raycast` */
  og_render_raycast(world: number, ray: string, desc: string | undefined): string;
  /** `og.render.renderTarget` */
  og_render_render_target(world: number, desc: string | undefined): string;
  /** `og.render.setPostParam` */
  og_render_set_post_param(world: number, post: number, name: string, value: string): string;
  /** `og.render.postErrors` */
  og_render_post_errors(world: number): string;
  /** `og.render.bakedLighting` */
  og_render_baked_lighting(world: number): string;
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
  /** `og.load.prefab` */
  og_load_prefab(world: number, path: string): string;
  /** `og.load.font` */
  og_load_font(world: number, path: string): string;
  /** `og.load.fontInfo` */
  og_load_font_info(world: number, font: number): string;
  /** `og.load.sound` */
  og_load_sound(world: number, path: string): string;
  /** `og.load.lut` */
  og_load_lut(world: number, path: string): string;
  /** `og.load.environment` */
  og_load_environment(world: number, path: string): string;
  /** `og.load.emitter` */
  og_load_emitter(world: number, path: string): string;
  /** `og.load.shader` */
  og_load_shader(world: number, path: string): string;
  /** `og.load.post` */
  og_load_post(world: number, path: string): string;
  /** `og.load.vector` */
  og_load_vector(world: number, path: string): string;
  /** `og.load.uiMaterial` */
  og_load_ui_material(world: number, path: string): string;
  /** `og.load.texture` */
  og_load_texture(world: number, path: string): string;
  /** `og.load.textureArray` */
  og_load_texture_array(world: number, path: string): string;
  /** `og.load.style` */
  og_load_style(world: number, path: string): string;
  /** `og.load.document` */
  og_load_document(world: number, path: string): string;
  /** `og.load.look` */
  og_load_look(world: number, path: string): string;
  /** `og.load.lookInfo` */
  og_load_look_info(world: number, look: number): string;
  /** `og.load.palette` */
  og_load_palette(world: number, path: string): string;
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
  /** `og.load.rig` */
  og_load_rig(world: number, path: string): string;
  /** `og.load.flipbook` */
  og_load_flipbook(world: number, path: string): string;
  /** `og.load.tilemap` */
  og_load_tilemap(world: number, path: string): string;
  /** `og.load.video` */
  og_load_video(world: number, path: string): string;
  /** `og.load.videoInfo` */
  og_load_video_info(world: number, video: number): string;
  /** `og.load.timeline` */
  og_load_timeline(world: number, path: string): string;
  /** `og.load.rig2d` */
  og_load_rig2d(world: number, path: string): string;
  /** `og.physics.raycast` */
  og_physics_raycast(world: number, desc: string | undefined): string;
  /** `og.physics.raycastAll` */
  og_physics_raycast_all(world: number, desc: string | undefined): string;
  /** `og.physics.projectPoint` */
  og_physics_project_point(world: number, desc: string | undefined): string;
  /** `og.physics.shapeCast` */
  og_physics_shape_cast(world: number, desc: string | undefined): string;
  /** `og.physics.overlap` */
  og_physics_overlap(world: number, desc: string | undefined): string;
  /** `og.physics.applyForce` */
  og_physics_apply_force(world: number, entity: number, force: string, point: string | undefined): string;
  /** `og.physics.applyImpulse` */
  og_physics_apply_impulse(world: number, entity: number, impulse: string, point: string | undefined): string;
  /** `og.physics.applyTorque` */
  og_physics_apply_torque(world: number, entity: number, torque: string): string;
  /** `og.physics.applyTorqueImpulse` */
  og_physics_apply_torque_impulse(world: number, entity: number, torque: string): string;
  /** `og.physics.setMesh` */
  og_physics_set_mesh(world: number, mesh: number, desc: string | undefined): string;
  /** `og.physics.ropePoints` */
  og_physics_rope_points(world: number, entity: number): string;
  /** `og.physics.clothPoints` */
  og_physics_cloth_points(world: number, entity: number): string;
  /** `og.physics.softBodyPoints` */
  og_physics_soft_body_points(world: number, entity: number): string;
  /** `og.physics.fluidParticles` */
  og_physics_fluid_particles(world: number, entity: number): string;
  /** `og.physics.fluid3dParticles` */
  og_physics_fluid3d_particles(world: number, entity: number): string;
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
  /** `og.physics.stats` */
  og_physics_stats(world: number): string;
  /** `og.atmosphere.timeOfDay` */
  og_atmosphere_time_of_day(world: number): number;
  /** `og.anim.graph` */
  og_anim_graph(world: number, desc: string | undefined): string;
  /** `og.anim.humanoid` */
  og_anim_humanoid(world: number, desc: string | undefined): string;
  /** `og.anim.retarget` */
  og_anim_retarget(world: number, desc: string | undefined): string;
  /** `og.anim.mirror` */
  og_anim_mirror(world: number, desc: string | undefined): string;
  /** `og.anim.rig2dInfo` */
  og_anim_rig2d_info(world: number, rig: number): string;
  /** `og.anim.skeleton2dBoxes` */
  og_anim_skeleton2d_boxes(world: number, entity: number): string;
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
  /** `og.tilemap.fill` */
  og_tilemap_fill(world: number, desc: string | undefined): string;
  /** `og.tilemap.create` */
  og_tilemap_create(world: number, desc: string | undefined): string;
  /** `og.tilemap.resize` */
  og_tilemap_resize(world: number, desc: string | undefined): string;
  /** `og.tilemap.neighbours` */
  og_tilemap_neighbours(world: number, map: number, x: number, y: number): string;
  /** `og.tilemap.getTile` */
  og_tilemap_get_tile(world: number, entity: number, layer: string, x: number, y: number): number;
  /** `og.tilemap.cellAt` */
  og_tilemap_cell_at(world: number, entity: number, point: string): string;
  /** `og.tilemap.cellCenter` */
  og_tilemap_cell_center(world: number, entity: number, x: number, y: number): string;
  /** `og.audio.createSound` */
  og_audio_create_sound(world: number, desc: string | undefined): string;
  /** `og.audio.outputLatency` */
  og_audio_output_latency(world: number): number;
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
  og_nav_load(world: number, path: string, profile: string | undefined): string;
  /** `og.nav.build` */
  og_nav_build(world: number, desc: string | undefined): string;
  /** `og.nav.setObstacle` */
  og_nav_set_obstacle(world: number, mesh: number, id: number, obstacle: string | undefined): string;
  /** `og.nav.carve` */
  og_nav_carve(world: number, mesh: number, id: number, shape: string | undefined): string;
  /** `og.nav.setArea` */
  og_nav_set_area(world: number, mesh: number, id: number, desc: string | undefined): string;
  /** `og.nav.setLink` */
  og_nav_set_link(world: number, mesh: number, id: number, desc: string | undefined): string;
  /** `og.nav.setLinkOpen` */
  og_nav_set_link_open(world: number, mesh: number, id: number, open: boolean): string;
  /** `og.nav.finishLink` */
  og_nav_finish_link(world: number, entity: number): boolean;
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
  /** `og.ai.restart` */
  og_ai_restart(world: number, entity: number): string;
  /** `og.ai.finish` */
  og_ai_finish(world: number, entity: number, success: boolean, action: string | undefined): boolean;
  /** `og.ai.noise` */
  og_ai_noise(world: number, desc: string | undefined): string;
  /** `og.ai.describe` */
  og_ai_describe(world: number, entity: number): string;
  /** `og.story.load` */
  og_story_load(world: number, path: string): string;
  /** `og.story.parse` */
  og_story_parse(world: number, text: string): string;
  /** `og.story.start` */
  og_story_start(world: number, desc: string | undefined): string;
  /** `og.story.view` */
  og_story_view(world: number, conversation: number): string;
  /** `og.story.history` */
  og_story_history(world: number, conversation: number): string;
  /** `og.story.conversations` */
  og_story_conversations(world: number): string;
  /** `og.story.advance` */
  og_story_advance(world: number, conversation: number): string;
  /** `og.story.choose` */
  og_story_choose(world: number, conversation: number, index: number): string;
  /** `og.story.stop` */
  og_story_stop(world: number, conversation: number): boolean;
  /** `og.story.bark` */
  og_story_bark(world: number, desc: string | undefined): string;
  /** `og.story.ambient` */
  og_story_ambient(world: number, desc: string | undefined): string;
  /** `og.story.barks` */
  og_story_barks(world: number): string;
  /** `og.story.get` */
  og_story_get(world: number, name: string): string;
  /** `og.story.set` */
  og_story_set(world: number, name: string, value: string): string;
  /** `og.story.reset` */
  og_story_reset(world: number): void;
  /** `og.story.startQuest` */
  og_story_start_quest(world: number, id: string): string;
  /** `og.story.progress` */
  og_story_progress(world: number, quest: string, objective: string, amount: number | undefined): boolean;
  /** `og.story.setStage` */
  og_story_set_stage(world: number, quest: string, stage: string): string;
  /** `og.story.completeQuest` */
  og_story_complete_quest(world: number, quest: string): string;
  /** `og.story.failQuest` */
  og_story_fail_quest(world: number, quest: string): string;
  /** `og.story.quest` */
  og_story_quest(world: number, id: string): string;
  /** `og.story.quests` */
  og_story_quests(world: number): string;
  /** `og.story.track` */
  og_story_track(world: number, id: string, on: boolean): boolean;
  /** `og.story.markSeen` */
  og_story_mark_seen(world: number, id: string): boolean;
  /** `og.story.mark` */
  og_story_mark(world: number, id: string, desc: string | undefined): boolean;
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
  /** `og.platform.saveSlot` */
  og_platform_save_slot(world: number, desc: string | undefined): string;
  /** `og.platform.loadSlot` */
  og_platform_load_slot(world: number, desc: string | undefined): string;
  /** `og.platform.products` */
  og_platform_products(world: number, desc: string | undefined): string;
  /** `og.platform.purchase` */
  og_platform_purchase(world: number, desc: string | undefined): string;
  /** `og.platform.entitlements` */
  og_platform_entitlements(world: number): number;
  /** `og.platform.consume` */
  og_platform_consume(world: number, purchase: string): string;
  /** `og.platform.grant` */
  og_platform_grant(world: number, purchase: string): string;
  /** `og.platform.granted` */
  og_platform_granted(world: number, purchase: string): boolean;
  /** `og.platform.adReady` */
  og_platform_ad_ready(world: number, desc: string | undefined): string;
  /** `og.platform.showAd` */
  og_platform_show_ad(world: number, desc: string | undefined): string;
  /** `og.platform.achievementUnlock` */
  og_platform_achievement_unlock(world: number, id: string): string;
  /** `og.platform.achievementProgress` */
  og_platform_achievement_progress(world: number, desc: string | undefined): string;
  /** `og.platform.achievements` */
  og_platform_achievements(world: number): number;
  /** `og.platform.player` */
  og_platform_player(world: number): number;
  /** `og.platform.friends` */
  og_platform_friends(world: number, desc: string | undefined): number;
  /** `og.platform.inviteSend` */
  og_platform_invite_send(world: number, desc: string | undefined): string;
  /** `og.platform.invites` */
  og_platform_invites(world: number): number;
  /** `og.platform.share` */
  og_platform_share(world: number, desc: string | undefined): string;
  /** `og.platform.supports` */
  og_platform_supports(world: number, method: string): boolean;
  /** `og.platform.notice` */
  og_platform_notice(world: number, notice: number): string;
  /** `og.platform.result` */
  og_platform_result(world: number, request: number): string;
  /** `og.platform.info` */
  og_platform_info(world: number): string;
  /** `og.platform.now` */
  og_platform_now(world: number): number;
  /** `og.platform.roomCreate` */
  og_platform_room_create(world: number, desc: string | undefined): string;
  /** `og.platform.roomJoin` */
  og_platform_room_join(world: number, room: string): string;
  /** `og.platform.roomLeave` */
  og_platform_room_leave(world: number): string;
  /** `og.platform.roomSend` */
  og_platform_room_send(world: number, data: string, to: number | undefined): string;
  /** `og.platform.roomPeers` */
  og_platform_room_peers(world: number): string;
  /** `og.platform.roomInfo` */
  og_platform_room_info(world: number): string;
  /** `og.platform.roomMessage` */
  og_platform_room_message(world: number, message: number): string;
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
  /** `og.i18n.formatCompact` */
  og_i18n_format_compact(world: number, value: number): string;
  /** `og.i18n.formatCurrency` */
  og_i18n_format_currency(world: number, amount: number, currency: string): string;
  /** `og.i18n.formatDate` */
  og_i18n_format_date(world: number, seconds: number, format: string | undefined): string;
  /** `og.i18n.formatTime` */
  og_i18n_format_time(world: number, seconds: number, format: string | undefined): string;
  /** `og.i18n.formatDuration` */
  og_i18n_format_duration(world: number, seconds: number): string;
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
  /** `og.ui.richText` */
  og_ui_rich_text(id: string, markup: string, desc: string | undefined): string;
  /** `og.ui.label` */
  og_ui_label(id: string, text: string, desc: string | undefined): void;
  /** `og.ui.button` */
  og_ui_button(id: string, text: string, desc: string | undefined): string;
  /** `og.ui.image` */
  og_ui_image(id: string, desc: string | undefined): void;
  /** `og.ui.zoomView` */
  og_ui_zoom_view(id: string, desc: string | undefined): string;
  /** `og.ui.progress` */
  og_ui_progress(id: string, value: number, desc: string | undefined): void;
  /** `og.ui.slider` */
  og_ui_slider(id: string, value: number, desc: string | undefined): string;
  /** `og.ui.toggle` */
  og_ui_toggle(id: string, on: boolean, label: string): string;
  /** `og.ui.textField` */
  og_ui_text_field(id: string, value: string, desc: string | undefined): string;
  /** `og.ui.textArea` */
  og_ui_text_area(id: string, value: string, desc: string | undefined): string;
  /** `og.ui.tooltip` */
  og_ui_tooltip(text: string): void;
  /** `og.ui.breakpoint` */
  og_ui_breakpoint(): string;
  /** `og.ui.framePointer` */
  og_ui_frame_pointer(): string;
  /** `og.ui.frameViewport` */
  og_ui_frame_viewport(): string;
  /** `og.ui.hasIcon` */
  og_ui_has_icon(name: string): boolean;
  /** `og.ui.seconds` */
  og_ui_seconds(): number;
  /** `og.ui.lastPath` */
  og_ui_last_path(): string;
  /** `og.ui.setFocus` */
  og_ui_set_focus(path: string): void;
  /** `og.ui.scrollIntoView` */
  og_ui_scroll_into_view(path: string, center: boolean): void;
  /** `og.ui.frameDirection` */
  og_ui_frame_direction(): string;
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
  /** `og.ui.setPlayers` */
  og_ui_set_players(count: number): void;
  /** `og.ui.focus` */
  og_ui_focus(pad: number): string;
  /** `og.ui.a11y` */
  og_ui_a11y(): string;
  /** `og.ui.feedback` */
  og_ui_feedback(event: string, rise: number): void;
  /** `og.ui.setHaptics` */
  og_ui_set_haptics(intensity: number): void;
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
  /** `og.ui.props` */
  og_ui_props(desc: string | undefined): boolean;
  /** `og.ui.defineTimeline` */
  og_ui_define_timeline(name: string, timeline: string | undefined): void;
  /** `og.ui.play` */
  og_ui_play(path: string, choreo: string, desc: string | undefined): number;
  /** `og.ui.playTimeline` */
  og_ui_play_timeline(path: string, timeline: string, desc: string | undefined): number;
  /** `og.ui.seek` */
  og_ui_seek(play: number, ms: number): boolean;
  /** `og.ui.pause` */
  og_ui_pause(play: number): boolean;
  /** `og.ui.resume` */
  og_ui_resume(play: number): boolean;
  /** `og.ui.reverse` */
  og_ui_reverse(play: number): boolean;
  /** `og.ui.stop` */
  og_ui_stop(play: number): boolean;
  /** `og.ui.playInfo` */
  og_ui_play_info(play: number): string;
  /** `og.ui.forceState` */
  og_ui_force_state(path: string, group: string, state: string): void;
  /** `og.ui.openPath` */
  og_ui_open_path(): string;
  /** `og.ui.visualState` */
  og_ui_visual_state(path: string, group: string): string;
  /** `og.ui.animate` */
  og_ui_animate(id: string, target: number, transition: string): number;
  /** `og.ui.gesture` */
  og_ui_gesture(path: string): string;
  /** `og.ui.motionEvents` */
  og_ui_motion_events(): string;
  /** `og.ui.overlaps` */
  og_ui_overlaps(): string;
  /** `og.ui.layoutStats` */
  og_ui_layout_stats(): string;
  /** `og.ui.semantics` */
  og_ui_semantics(desc: string | undefined): boolean;
  /** `og.ui.partAttr` */
  og_ui_part_attr(key: string, value: string): boolean;
  /** `og.ui.setDraw` */
  og_ui_set_draw(desc: string | undefined): void;
  /** `og.ui.openWorldAnchor` */
  og_ui_open_world_anchor(id: string, desc: string | undefined): string;
  /** `og.ui.openWorldPanel` */
  og_ui_open_world_panel(id: string, desc: string | undefined): string;
  /** `og.ui.defineMass` */
  og_ui_define_mass(name: string, desc: string | undefined): number;
  /** `og.ui.spawnMass` */
  og_ui_spawn_mass(template: number, instances: string): void;
  /** `og.ui.pushMass` */
  og_ui_push_mass(template: number, instances: string): void;
  /** `og.ui.massCount` */
  og_ui_mass_count(template: number): number;
  /** `og.ui.particles` */
  og_ui_particles(desc: string | undefined): void;
  /** `og.ui.meta` */
  og_ui_meta(id: string, desc: string | undefined): void;
  /** `og.ui.sceneViewport` */
  og_ui_scene_viewport(id: string, desc: string | undefined): string;
  /** `og.ui.screen` */
  og_ui_screen(name: string, vm: string | undefined): string;
  /** `og.ui.component` */
  og_ui_component(id: string, name: string, props: string | undefined): string;
  /** `og.ui.openSlot` */
  og_ui_open_slot(path: string): boolean;
  /** `og.ui.push` */
  og_ui_push(world: number, screen: string, vm: string | undefined): string;
  /** `og.ui.pop` */
  og_ui_pop(world: number): string;
  /** `og.ui.replace` */
  og_ui_replace(world: number, screen: string, vm: string | undefined): string;
  /** `og.ui.clear` */
  og_ui_clear(world: number): void;
  /** `og.ui.layers` */
  og_ui_layers(world: number): string;
  /** `og.ui.screenVm` */
  og_ui_screen_vm(world: number, screen: string): string;
  /** `og.ui.docEvents` */
  og_ui_doc_events(world: number): string;
  /** `og.ui.gameInput` */
  og_ui_game_input(world: number): boolean;
  /** `og.ui.paused` */
  og_ui_paused(world: number): boolean;
  /** `og.ui.defineAction` */
  og_ui_define_action(world: number, desc: string): string;
  /** `og.ui.action` */
  og_ui_action(name: string): boolean;
  /** `og.ui.actionHeld` */
  og_ui_action_held(name: string): boolean;
  /** `og.ui.glyph` */
  og_ui_glyph(world: number, action: string): string;
  /** `og.ui.frameGlyph` */
  og_ui_frame_glyph(action: string): string;
  /** `og.ui.device` */
  og_ui_device(world: number): string;
  /** `og.ui.setDevice` */
  og_ui_set_device(world: number, device: string): void;
  /** `og.ui.actionBar` */
  og_ui_action_bar(world: number): string;
  /** `og.ui.documentSchema` */
  og_ui_document_schema(): string;
  /** `og.ui.setProp` */
  og_ui_set_prop(world: number, path: string, prop: string, value: string): string;
  /** `og.ui.saveEdits` */
  og_ui_save_edits(world: number): string;
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
  /** `EngineProbe.memory` */
  og_engine_probe_memory(engine_probe: number): string;
  /** `EngineProbe.describe` */
  og_engine_probe_describe(engine_probe: number): string;
  /** `EngineProbe.expect` */
  og_engine_probe_expect(engine_probe: number, what: string): string;
  /** `EngineProbe.expectations` */
  og_engine_probe_expectations(engine_probe: number): string;
  /** `EngineProbe.saveScene` */
  og_engine_probe_save_scene(engine_probe: number, write: boolean | undefined, spawned: boolean | undefined): string;
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
  /** `og.stream.status` */
  og_stream_status(world: number, group: string): string;
  /** `og.stream.request` */
  og_stream_request(world: number, group: string): string;
  /** `og.stream.release` */
  og_stream_release(world: number, group: string): string;
  /** `og.stream.budget` */
  og_stream_budget(world: number): string;
  /** `og.stream.setBudget` */
  og_stream_set_budget(world: number, bytes: number): string;
  /** `og.video.capabilities` */
  og_video_capabilities(world: number): string;
  /** `og.debug.line` */
  og_debug_line(world: number, from: string, to: string, desc: string | undefined): void;
  /** `og.debug.box` */
  og_debug_box(world: number, center: string, size: string, desc: string | undefined): void;
  /** `og.debug.sphere` */
  og_debug_sphere(world: number, center: string, radius: number, desc: string | undefined): void;
  /** `og.debug.arrow` */
  og_debug_arrow(world: number, from: string, to: string, desc: string | undefined): void;
  /** `og.debug.axes` */
  og_debug_axes(world: number, position: string, size: number, desc: string | undefined): void;
  /** `og.debug.text` */
  og_debug_text(world: number, position: string, text: string, desc: string | undefined): void;
  /** `og.debug.clear` */
  og_debug_clear(world: number): void;
  /** `og.debug.setView` */
  og_debug_set_view(world: number, view: string): void;
  /** `og.debug.view` */
  og_debug_view(world: number): string;
  /** `og.debug.setEnabled` */
  og_debug_set_enabled(world: number, enabled: boolean): void;
  /** `og.debug.enabled` */
  og_debug_enabled(world: number): boolean;
  /** `og.path.create` */
  og_path_create(world: number, desc: string | undefined): string;
  /** `og.path.find` */
  og_path_find(world: number, name: string): string;
  /** `og.path.sample` */
  og_path_sample(world: number, path: number, t: number): string;
  /** `og.path.length` */
  og_path_length(world: number, path: number): string;
  /** `og.path.nearest` */
  og_path_nearest(world: number, path: number, point: string): string;
  /** `og.timeline.info` */
  og_timeline_info(world: number, asset: number): string;
  /** `og.timeline.play` */
  og_timeline_play(world: number, entity: number): string;
  /** `og.timeline.pause` */
  og_timeline_pause(world: number, entity: number): string;
  /** `og.timeline.seek` */
  og_timeline_seek(world: number, entity: number, time: number): string;
  /** `og.timeline.stop` */
  og_timeline_stop(world: number, entity: number): string;
  /** `og.net.submit` */
  og_net_submit(world: number, input: string | undefined): string;
  /** `og.net.status` */
  og_net_status(world: number): string;
  /** `og.net.resimulating` */
  og_net_resimulating(world: number): boolean;
  /** `og.net.finalHash` */
  og_net_final_hash(world: number, tick: number): string;
  /** `og.net.hash` */
  og_net_hash(world: number): string;
  /** `og.motion.pose` */
  og_motion_pose(world: number, entity: number, desc: string | undefined): string;
  /** `og.motion.place` */
  og_motion_place(world: number, entity: number, desc: string | undefined): string;
  /** `og.motion.face` */
  og_motion_face(world: number, entity: number, desc: string | undefined): string;
  /** `og.motion.walkTo` */
  og_motion_walk_to(world: number, entity: number, desc: string | undefined): string;
  /** `og.motion.clear` */
  og_motion_clear(world: number, entity: number): string;
  /** `og.motion.status` */
  og_motion_status(world: number, entity: number): string;
  /** `og.motion.capabilities` */
  og_motion_capabilities(world: number): string;
  /** `og.motion.poses` */
  og_motion_poses(): string;
}

/** Every binding symbol, in binding-table order, then the lifecycle members. */
export const SYMBOLS: readonly string[] = [
  "og_reflect_define_component",
  "og_reflect_define_resource",
  "og_save_edit_get_version",
  "og_save_edit_types",
  "og_save_edit_records",
  "og_save_edit_set_records",
  "og_save_edit_rename_type",
  "og_save_edit_rename_field",
  "og_save_edit_drop_type",
  "og_save_edit_spawn",
  "og_save_edit_text",
  "og_save_edit_close",
  "og_spatial_index_get_count",
  "og_spatial_index_within",
  "og_spatial_index_nearest",
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
  "og_world_children",
  "og_world_removed",
  "og_world_spatial",
  "og_world_state",
  "og_world_set_state",
  "og_world_find",
  "og_world_prefab_of",
  "og_world_name_of",
  "og_world_query",
  "og_world_events",
  "og_world_emit",
  "og_world_instantiate",
  "og_world_snapshot",
  "og_world_open_save",
  "og_world_restore",
  "og_query_archetypes",
  "og_query_count",
  "og_archetype_view_get_count",
  "og_archetype_view_get_tick",
  "og_archetype_view_get_entities",
  "og_archetype_view_get_rows",
  "og_archetype_view_entity",
  "og_archetype_view_has",
  "og_archetype_view_column",
  "og_hook_get_name",
  "og_system_define_hook",
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
  "og_time_set_group_scale",
  "og_time_group_dt",
  "og_rng_stream_get_seed",
  "og_rng_stream_next",
  "og_rng_stream_range",
  "og_rng_stream_integer",
  "og_rng_stream_chance",
  "og_rng_stream_weighted",
  "og_rng_stream_permutation",
  "og_rng_stream_fork",
  "og_rng_next",
  "og_rng_range",
  "og_rng_integer",
  "og_rng_chance",
  "og_rng_weighted",
  "og_rng_permutation",
  "og_rng_fork",
  "og_procgen_noise",
  "og_procgen_noise_grid",
  "og_procgen_poisson_disk",
  "og_procgen_bsp",
  "og_procgen_wfc",
  "og_input_define_action",
  "og_input_actions",
  "og_input_action",
  "og_input_action_down",
  "og_input_action_pressed",
  "og_input_action_released",
  "og_input_action_held",
  "og_input_action_value",
  "og_input_action_vector",
  "og_input_set_contexts",
  "og_input_contexts",
  "og_input_rebind",
  "og_input_conflicts",
  "og_input_reset_bindings",
  "og_input_set_toggle",
  "og_input_save_bindings",
  "og_input_load_bindings",
  "og_input_start_capture",
  "og_input_capture",
  "og_input_cancel_capture",
  "og_input_apply_capture",
  "og_input_glyph",
  "og_input_glyphs",
  "og_input_control_glyph",
  "og_input_device",
  "og_input_pad_device",
  "og_input_set_deadzones",
  "og_input_set_sensitivity",
  "og_input_sensitivity",
  "og_input_set_keyboard_layout",
  "og_input_keyboard_layout",
  "og_input_set_mouse",
  "og_input_mouse",
  "og_input_rumble",
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
  "og_render_snap",
  "og_render_snap_camera",
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
  "og_render_set_material_shader",
  "og_render_set_shader_param",
  "og_render_apply_look",
  "og_render_apply_quality",
  "og_render_set_display",
  "og_render_reset_display",
  "og_render_display",
  "og_render_look",
  "og_render_capabilities",
  "og_render_text",
  "og_render_buffer_geometry",
  "og_render_update_geometry",
  "og_render_geometry_info",
  "og_render_cylinder_geometry",
  "og_render_cone_geometry",
  "og_render_capsule_geometry",
  "og_render_torus_geometry",
  "og_render_ring_geometry",
  "og_render_circle_geometry",
  "og_render_tube_geometry",
  "og_render_shape_geometry",
  "og_render_extrude_geometry",
  "og_render_lathe_geometry",
  "og_render_polyhedron_geometry",
  "og_render_icosahedron_geometry",
  "og_render_dodecahedron_geometry",
  "og_render_octahedron_geometry",
  "og_render_tetrahedron_geometry",
  "og_render_sweep_geometry",
  "og_render_csg_geometry",
  "og_render_merge_geometries",
  "og_render_lines",
  "og_render_points",
  "og_render_raycast",
  "og_render_render_target",
  "og_render_set_post_param",
  "og_render_post_errors",
  "og_render_baked_lighting",
  "og_load_bundle",
  "og_load_bundle_info",
  "og_load_default_bundle",
  "og_load_gltf",
  "og_load_scene",
  "og_load_prefab",
  "og_load_font",
  "og_load_font_info",
  "og_load_sound",
  "og_load_lut",
  "og_load_environment",
  "og_load_emitter",
  "og_load_shader",
  "og_load_post",
  "og_load_vector",
  "og_load_ui_material",
  "og_load_texture",
  "og_load_texture_array",
  "og_load_style",
  "og_load_document",
  "og_load_look",
  "og_load_look_info",
  "og_load_palette",
  "og_load_sound_info",
  "og_load_status",
  "og_load_error",
  "og_load_gltf_info",
  "og_load_animation",
  "og_load_rig",
  "og_load_flipbook",
  "og_load_tilemap",
  "og_load_video",
  "og_load_video_info",
  "og_load_timeline",
  "og_load_rig2d",
  "og_physics_raycast",
  "og_physics_raycast_all",
  "og_physics_project_point",
  "og_physics_shape_cast",
  "og_physics_overlap",
  "og_physics_apply_force",
  "og_physics_apply_impulse",
  "og_physics_apply_torque",
  "og_physics_apply_torque_impulse",
  "og_physics_set_mesh",
  "og_physics_rope_points",
  "og_physics_cloth_points",
  "og_physics_soft_body_points",
  "og_physics_fluid_particles",
  "og_physics_fluid3d_particles",
  "og_physics_water_height",
  "og_physics_vehicle_wheels",
  "og_physics_ragdoll_rig_from_skeleton",
  "og_physics_set_ragdoll_rig",
  "og_physics_ragdoll_pose",
  "og_physics_stats",
  "og_atmosphere_time_of_day",
  "og_anim_graph",
  "og_anim_humanoid",
  "og_anim_retarget",
  "og_anim_mirror",
  "og_anim_rig2d_info",
  "og_anim_skeleton2d_boxes",
  "og_tween_to",
  "og_tween_sequence",
  "og_tween_parallel",
  "og_tween_cancel",
  "og_tween_cancel_entity",
  "og_tween_active",
  "og_tween_squash",
  "og_tween_flash",
  "og_tilemap_set_tile",
  "og_tilemap_fill",
  "og_tilemap_create",
  "og_tilemap_resize",
  "og_tilemap_neighbours",
  "og_tilemap_get_tile",
  "og_tilemap_cell_at",
  "og_tilemap_cell_center",
  "og_audio_create_sound",
  "og_audio_output_latency",
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
  "og_nav_carve",
  "og_nav_set_area",
  "og_nav_set_link",
  "og_nav_set_link_open",
  "og_nav_finish_link",
  "og_nav_nearest",
  "og_nav_path_of",
  "og_ai_load",
  "og_ai_parse",
  "og_ai_get",
  "og_ai_set",
  "og_ai_restart",
  "og_ai_finish",
  "og_ai_noise",
  "og_ai_describe",
  "og_story_load",
  "og_story_parse",
  "og_story_start",
  "og_story_view",
  "og_story_history",
  "og_story_conversations",
  "og_story_advance",
  "og_story_choose",
  "og_story_stop",
  "og_story_bark",
  "og_story_ambient",
  "og_story_barks",
  "og_story_get",
  "og_story_set",
  "og_story_reset",
  "og_story_start_quest",
  "og_story_progress",
  "og_story_set_stage",
  "og_story_complete_quest",
  "og_story_fail_quest",
  "og_story_quest",
  "og_story_quests",
  "og_story_track",
  "og_story_mark_seen",
  "og_story_mark",
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
  "og_platform_save_slot",
  "og_platform_load_slot",
  "og_platform_products",
  "og_platform_purchase",
  "og_platform_entitlements",
  "og_platform_consume",
  "og_platform_grant",
  "og_platform_granted",
  "og_platform_ad_ready",
  "og_platform_show_ad",
  "og_platform_achievement_unlock",
  "og_platform_achievement_progress",
  "og_platform_achievements",
  "og_platform_player",
  "og_platform_friends",
  "og_platform_invite_send",
  "og_platform_invites",
  "og_platform_share",
  "og_platform_supports",
  "og_platform_notice",
  "og_platform_result",
  "og_platform_info",
  "og_platform_now",
  "og_platform_room_create",
  "og_platform_room_join",
  "og_platform_room_leave",
  "og_platform_room_send",
  "og_platform_room_peers",
  "og_platform_room_info",
  "og_platform_room_message",
  "og_i18n_load",
  "og_i18n_set_locale",
  "og_i18n_locale",
  "og_i18n_t",
  "og_i18n_has",
  "og_i18n_format_compact",
  "og_i18n_format_currency",
  "og_i18n_format_date",
  "og_i18n_format_time",
  "og_i18n_format_duration",
  "og_ui_configure",
  "og_ui_viewport",
  "og_ui_theme",
  "og_ui_open",
  "og_ui_open_panel",
  "og_ui_open_scroll",
  "og_ui_close",
  "og_ui_rich_text",
  "og_ui_label",
  "og_ui_button",
  "og_ui_image",
  "og_ui_zoom_view",
  "og_ui_progress",
  "og_ui_slider",
  "og_ui_toggle",
  "og_ui_text_field",
  "og_ui_text_area",
  "og_ui_tooltip",
  "og_ui_breakpoint",
  "og_ui_frame_pointer",
  "og_ui_frame_viewport",
  "og_ui_has_icon",
  "og_ui_seconds",
  "og_ui_last_path",
  "og_ui_set_focus",
  "og_ui_scroll_into_view",
  "og_ui_frame_direction",
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
  "og_ui_set_players",
  "og_ui_focus",
  "og_ui_a11y",
  "og_ui_feedback",
  "og_ui_set_haptics",
  "og_ui_set_navigation",
  "og_ui_memory",
  "og_ui_remember",
  "og_ui_tick",
  "og_ui_counter",
  "og_ui_burst",
  "og_ui_shake",
  "og_ui_screen_transition",
  "og_ui_preview_viewport",
  "og_ui_props",
  "og_ui_define_timeline",
  "og_ui_play",
  "og_ui_play_timeline",
  "og_ui_seek",
  "og_ui_pause",
  "og_ui_resume",
  "og_ui_reverse",
  "og_ui_stop",
  "og_ui_play_info",
  "og_ui_force_state",
  "og_ui_open_path",
  "og_ui_visual_state",
  "og_ui_animate",
  "og_ui_gesture",
  "og_ui_motion_events",
  "og_ui_overlaps",
  "og_ui_layout_stats",
  "og_ui_semantics",
  "og_ui_part_attr",
  "og_ui_set_draw",
  "og_ui_open_world_anchor",
  "og_ui_open_world_panel",
  "og_ui_define_mass",
  "og_ui_spawn_mass",
  "og_ui_push_mass",
  "og_ui_mass_count",
  "og_ui_particles",
  "og_ui_meta",
  "og_ui_scene_viewport",
  "og_ui_screen",
  "og_ui_component",
  "og_ui_open_slot",
  "og_ui_push",
  "og_ui_pop",
  "og_ui_replace",
  "og_ui_clear",
  "og_ui_layers",
  "og_ui_screen_vm",
  "og_ui_doc_events",
  "og_ui_game_input",
  "og_ui_paused",
  "og_ui_define_action",
  "og_ui_action",
  "og_ui_action_held",
  "og_ui_glyph",
  "og_ui_frame_glyph",
  "og_ui_device",
  "og_ui_set_device",
  "og_ui_action_bar",
  "og_ui_document_schema",
  "og_ui_set_prop",
  "og_ui_save_edits",
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
  "og_engine_probe_memory",
  "og_engine_probe_describe",
  "og_engine_probe_expect",
  "og_engine_probe_expectations",
  "og_engine_probe_save_scene",
  "og_terrain_load",
  "og_terrain_sample",
  "og_terrain_bounds",
  "og_terrain_edit",
  "og_terrain_layer",
  "og_terrain_set_layer",
  "og_stream_status",
  "og_stream_request",
  "og_stream_release",
  "og_stream_budget",
  "og_stream_set_budget",
  "og_video_capabilities",
  "og_debug_line",
  "og_debug_box",
  "og_debug_sphere",
  "og_debug_arrow",
  "og_debug_axes",
  "og_debug_text",
  "og_debug_clear",
  "og_debug_set_view",
  "og_debug_view",
  "og_debug_set_enabled",
  "og_debug_enabled",
  "og_path_create",
  "og_path_find",
  "og_path_sample",
  "og_path_length",
  "og_path_nearest",
  "og_timeline_info",
  "og_timeline_play",
  "og_timeline_pause",
  "og_timeline_seek",
  "og_timeline_stop",
  "og_net_submit",
  "og_net_status",
  "og_net_resimulating",
  "og_net_final_hash",
  "og_net_hash",
  "og_motion_pose",
  "og_motion_place",
  "og_motion_face",
  "og_motion_walk_to",
  "og_motion_clear",
  "og_motion_status",
  "og_motion_capabilities",
  "og_motion_poses",
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
  "og_engine_resource_layout",
  "og_engine_read_resource",
  "og_engine_write_resource_bytes",
  "og_platform_take",
  "og_platform_answer",
  "og_platform_set_host",
  "og_platform_notify",
  "og_room_take",
  "og_room_deliver",
  "og_room_set_host",
];

/** Why `value`, passed as `param` of `call` where an entity is expected, is not one. */
function notEntity(call: string, param: string, value: unknown): never {
  const pass = "pass the number world.spawn, world.find or a.entity(i) returned";
  const what =
    value === null ? "null (world.find returns null when no entity has the name)"
    : value === undefined ? `undefined (${pass})`
    : typeof value === "object" && "ok" in value ? "a Result, not an entity (world.spawn answers { ok, value }: pass its value)"
    : `${/^[aeiou]/.test(typeof value) ? "an" : "a"} ${typeof value}, not an entity (${pass})`;
  throw new Error(`${call}: ${param} is ${what}`);
}

/** `abi` with every binding that takes an `Entity` checking it is a number first (`notEntity`). */
export function checkEntities<A, V>(abi: HostAbi<A, V>): HostAbi<A, V> {
  return {
    ...abi,
    og_world_despawn(world: number, entity: number) {
      if (typeof entity !== "number") notEntity("world.despawn", "entity", entity);
      return abi.og_world_despawn(world, entity);
    },
    og_world_alive(world: number, entity: number) {
      if (typeof entity !== "number") notEntity("world.alive", "entity", entity);
      return abi.og_world_alive(world, entity);
    },
    og_world_has(world: number, entity: number, component: string) {
      if (typeof entity !== "number") notEntity("world.has", "entity", entity);
      return abi.og_world_has(world, entity, component);
    },
    og_world_insert(world: number, entity: number, component: string, value: string | undefined) {
      if (typeof entity !== "number") notEntity("world.insert", "entity", entity);
      return abi.og_world_insert(world, entity, component, value);
    },
    og_world_remove(world: number, entity: number, component: string) {
      if (typeof entity !== "number") notEntity("world.remove", "entity", entity);
      return abi.og_world_remove(world, entity, component);
    },
    og_world_get(world: number, entity: number, component: string) {
      if (typeof entity !== "number") notEntity("world.get", "entity", entity);
      return abi.og_world_get(world, entity, component);
    },
    og_world_set(world: number, entity: number, component: string, value: string) {
      if (typeof entity !== "number") notEntity("world.set", "entity", entity);
      return abi.og_world_set(world, entity, component, value);
    },
    og_world_children(world: number, entity: number) {
      if (typeof entity !== "number") notEntity("world.children", "entity", entity);
      return abi.og_world_children(world, entity);
    },
    og_world_prefab_of(world: number, entity: number) {
      if (typeof entity !== "number") notEntity("world.prefabOf", "entity", entity);
      return abi.og_world_prefab_of(world, entity);
    },
    og_world_name_of(world: number, entity: number) {
      if (typeof entity !== "number") notEntity("world.nameOf", "entity", entity);
      return abi.og_world_name_of(world, entity);
    },
    og_time_group_dt(world: number, entity: number) {
      if (typeof entity !== "number") notEntity("og.time.groupDt", "entity", entity);
      return abi.og_time_group_dt(world, entity);
    },
    og_render_snap(world: number, entity: number) {
      if (typeof entity !== "number") notEntity("og.render.snap", "entity", entity);
      return abi.og_render_snap(world, entity);
    },
    og_physics_apply_force(world: number, entity: number, force: string, point: string | undefined) {
      if (typeof entity !== "number") notEntity("og.physics.applyForce", "entity", entity);
      return abi.og_physics_apply_force(world, entity, force, point);
    },
    og_physics_apply_impulse(world: number, entity: number, impulse: string, point: string | undefined) {
      if (typeof entity !== "number") notEntity("og.physics.applyImpulse", "entity", entity);
      return abi.og_physics_apply_impulse(world, entity, impulse, point);
    },
    og_physics_apply_torque(world: number, entity: number, torque: string) {
      if (typeof entity !== "number") notEntity("og.physics.applyTorque", "entity", entity);
      return abi.og_physics_apply_torque(world, entity, torque);
    },
    og_physics_apply_torque_impulse(world: number, entity: number, torque: string) {
      if (typeof entity !== "number") notEntity("og.physics.applyTorqueImpulse", "entity", entity);
      return abi.og_physics_apply_torque_impulse(world, entity, torque);
    },
    og_physics_rope_points(world: number, entity: number) {
      if (typeof entity !== "number") notEntity("og.physics.ropePoints", "entity", entity);
      return abi.og_physics_rope_points(world, entity);
    },
    og_physics_cloth_points(world: number, entity: number) {
      if (typeof entity !== "number") notEntity("og.physics.clothPoints", "entity", entity);
      return abi.og_physics_cloth_points(world, entity);
    },
    og_physics_soft_body_points(world: number, entity: number) {
      if (typeof entity !== "number") notEntity("og.physics.softBodyPoints", "entity", entity);
      return abi.og_physics_soft_body_points(world, entity);
    },
    og_physics_fluid_particles(world: number, entity: number) {
      if (typeof entity !== "number") notEntity("og.physics.fluidParticles", "entity", entity);
      return abi.og_physics_fluid_particles(world, entity);
    },
    og_physics_fluid3d_particles(world: number, entity: number) {
      if (typeof entity !== "number") notEntity("og.physics.fluid3dParticles", "entity", entity);
      return abi.og_physics_fluid3d_particles(world, entity);
    },
    og_physics_water_height(world: number, entity: number, x: number, z: number) {
      if (typeof entity !== "number") notEntity("og.physics.waterHeight", "entity", entity);
      return abi.og_physics_water_height(world, entity, x, z);
    },
    og_physics_vehicle_wheels(world: number, entity: number) {
      if (typeof entity !== "number") notEntity("og.physics.vehicleWheels", "entity", entity);
      return abi.og_physics_vehicle_wheels(world, entity);
    },
    og_physics_set_ragdoll_rig(world: number, entity: number, desc: string | undefined) {
      if (typeof entity !== "number") notEntity("og.physics.setRagdollRig", "entity", entity);
      return abi.og_physics_set_ragdoll_rig(world, entity, desc);
    },
    og_physics_ragdoll_pose(world: number, entity: number) {
      if (typeof entity !== "number") notEntity("og.physics.ragdollPose", "entity", entity);
      return abi.og_physics_ragdoll_pose(world, entity);
    },
    og_anim_skeleton2d_boxes(world: number, entity: number) {
      if (typeof entity !== "number") notEntity("og.anim.skeleton2dBoxes", "entity", entity);
      return abi.og_anim_skeleton2d_boxes(world, entity);
    },
    og_tween_cancel_entity(world: number, entity: number) {
      if (typeof entity !== "number") notEntity("og.tween.cancelEntity", "entity", entity);
      return abi.og_tween_cancel_entity(world, entity);
    },
    og_tilemap_get_tile(world: number, entity: number, layer: string, x: number, y: number) {
      if (typeof entity !== "number") notEntity("og.tilemap.getTile", "entity", entity);
      return abi.og_tilemap_get_tile(world, entity, layer, x, y);
    },
    og_tilemap_cell_at(world: number, entity: number, point: string) {
      if (typeof entity !== "number") notEntity("og.tilemap.cellAt", "entity", entity);
      return abi.og_tilemap_cell_at(world, entity, point);
    },
    og_tilemap_cell_center(world: number, entity: number, x: number, y: number) {
      if (typeof entity !== "number") notEntity("og.tilemap.cellCenter", "entity", entity);
      return abi.og_tilemap_cell_center(world, entity, x, y);
    },
    og_nav_finish_link(world: number, entity: number) {
      if (typeof entity !== "number") notEntity("og.nav.finishLink", "entity", entity);
      return abi.og_nav_finish_link(world, entity);
    },
    og_nav_path_of(world: number, entity: number) {
      if (typeof entity !== "number") notEntity("og.nav.pathOf", "entity", entity);
      return abi.og_nav_path_of(world, entity);
    },
    og_ai_get(world: number, entity: number, key: string) {
      if (typeof entity !== "number") notEntity("og.ai.get", "entity", entity);
      return abi.og_ai_get(world, entity, key);
    },
    og_ai_set(world: number, entity: number, key: string, value: string) {
      if (typeof entity !== "number") notEntity("og.ai.set", "entity", entity);
      return abi.og_ai_set(world, entity, key, value);
    },
    og_ai_restart(world: number, entity: number) {
      if (typeof entity !== "number") notEntity("og.ai.restart", "entity", entity);
      return abi.og_ai_restart(world, entity);
    },
    og_ai_finish(world: number, entity: number, success: boolean, action: string | undefined) {
      if (typeof entity !== "number") notEntity("og.ai.finish", "entity", entity);
      return abi.og_ai_finish(world, entity, success, action);
    },
    og_ai_describe(world: number, entity: number) {
      if (typeof entity !== "number") notEntity("og.ai.describe", "entity", entity);
      return abi.og_ai_describe(world, entity);
    },
    og_timeline_play(world: number, entity: number) {
      if (typeof entity !== "number") notEntity("og.timeline.play", "entity", entity);
      return abi.og_timeline_play(world, entity);
    },
    og_timeline_pause(world: number, entity: number) {
      if (typeof entity !== "number") notEntity("og.timeline.pause", "entity", entity);
      return abi.og_timeline_pause(world, entity);
    },
    og_timeline_seek(world: number, entity: number, time: number) {
      if (typeof entity !== "number") notEntity("og.timeline.seek", "entity", entity);
      return abi.og_timeline_seek(world, entity, time);
    },
    og_timeline_stop(world: number, entity: number) {
      if (typeof entity !== "number") notEntity("og.timeline.stop", "entity", entity);
      return abi.og_timeline_stop(world, entity);
    },
    og_motion_pose(world: number, entity: number, desc: string | undefined) {
      if (typeof entity !== "number") notEntity("og.motion.pose", "entity", entity);
      return abi.og_motion_pose(world, entity, desc);
    },
    og_motion_place(world: number, entity: number, desc: string | undefined) {
      if (typeof entity !== "number") notEntity("og.motion.place", "entity", entity);
      return abi.og_motion_place(world, entity, desc);
    },
    og_motion_face(world: number, entity: number, desc: string | undefined) {
      if (typeof entity !== "number") notEntity("og.motion.face", "entity", entity);
      return abi.og_motion_face(world, entity, desc);
    },
    og_motion_walk_to(world: number, entity: number, desc: string | undefined) {
      if (typeof entity !== "number") notEntity("og.motion.walkTo", "entity", entity);
      return abi.og_motion_walk_to(world, entity, desc);
    },
    og_motion_clear(world: number, entity: number) {
      if (typeof entity !== "number") notEntity("og.motion.clear", "entity", entity);
      return abi.og_motion_clear(world, entity);
    },
    og_motion_status(world: number, entity: number) {
      if (typeof entity !== "number") notEntity("og.motion.status", "entity", entity);
      return abi.og_motion_status(world, entity);
    },
  };
}
