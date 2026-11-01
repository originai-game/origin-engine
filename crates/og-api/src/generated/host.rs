// Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate.
//! The host contract, generated from the IDL: one trait per module with
//! callables, each method a binding symbol with its wire signature (see
//! og-idl's `gen/host.rs` for the wire forms). `og-host-web` and
//! `og-host-node` implement every trait, and their generated glue calls
//! the methods, so a member missing on either host is a build error.
//!
//! The IDL's prose is not repeated here: a method names its TypeScript
//! path, documented in api.ogt and packages/knowledge/wiki/<module>.md.
//! An edit to the docs alone then leaves this file, and so the hosts
//! and the wasm, unchanged; field docs stay in RECORDS, which the
//! runtime's reflection serves.

/// What aliases engine memory, chosen per host.
pub trait Wire {
    /// A typed array the host hands out (`ArchetypeView.entities`).
    type Array;
    /// A component's column views (`ArchetypeView.column`).
    type Views;
    /// A JavaScript function the engine calls back.
    type Function;
    /// Bytes copied across, either way (a resource's packed record).
    type Bytes;
}

/// Engine lifecycle: what the TypeScript loaders drive.
pub trait Lifecycle: Wire {
    /// Install the function the engine calls for systems, commands and facts: `dispatcher(kind, engine, id, query, tick, dt, elapsed, payload)` (payload: a command's arguments as a JSON list) returns `""` on success, the error otherwise (`kind`: 0 system, 1 command, 2 facts, whose answer is `"="` then JSON).
    fn og_host_set_dispatcher(&self, dispatcher: Self::Function);
    /// Assemble a world from a declared game (`og.game`) and the declarations made so far. `options` is JSON `{ seed?, root? }`: a replay seed, and the project directory (native) or URL base (web) whose `cooked/` bundle is opened and whose GameDesc.scene is spawned before the first tick. The answer is a Result of the engine id, which is also the world handle.
    fn og_engine_create(&self, game: u32, options: String) -> String;
    /// Drop an engine and everything it owns. False for an unknown engine.
    fn og_engine_destroy(&self, engine: u32) -> bool;
    /// Take fixed steps regardless of the wall clock: a Result of the tick reached, or the first failing system with its tick.
    fn og_engine_advance(&self, engine: u32, steps: u32) -> String;
    /// Run a registered command action now, between ticks (`args` is a JSON list of strings). The scheduled channel is `og.commands.schedule`.
    fn og_engine_command(&self, engine: u32, action: String, args: String) -> String;
    /// One display frame: feed `elapsed_ms` of wall clock to the fixed-tick accumulator (unless paused), run the command steps due, take the ticks it allows up to `GameDesc.maxCatchUp` through the probe, then render when a renderer is attached. JSON `{ ok, ticks, tick, rendered, paused, quit }` or a refusal; `quit` is true once a `quit` command ran.
    fn og_engine_frame(&self, engine: u32, elapsed_ms: f64) -> String;
    /// A new game bundle is about to be imported (hot reload): its component, resource and command declarations replace the previous bundle's; systems and games keep their handles. The probe's `reload` then assembles the latest `og.game` and restores the world into it.
    fn og_host_begin_bundle(&self);
    /// The probe's JSON-RPC 2.0 entry: one request frame in, its answer frame out (og-probe `Probe::call`). The native loopback WebSocket and the web request ring carry exactly these frames. Call it between ticks, never from a system.
    fn og_engine_call(&self, engine: u32, frame: String) -> String;
    /// The files the command channel produced since the last call (`screenshot`, `dump_*`, `help`), as JSON `[{ tick, action, file, kind, data }]`: `file` is og-probe's `Output::file_name()`, `kind` is `png`, `json`, `text` or `wav`, `data` is base64 for a PNG or a WAV and the text otherwise. The host writes them (og run) or posts them.
    fn og_engine_take_outputs(&self, engine: u32) -> String;
    /// Log lines since the last call, as JSON `[{ tick, warning, text }]`: the command channel's and the probe's warnings and the loader's, each naming the fix.
    fn og_engine_take_log(&self, engine: u32) -> String;
    /// Write some fields of a resource as JSON: the runtime's path for a write a publish build refuses (logged once per system). A Result.
    fn og_engine_write_resource(&self, world: u32, resource: String, value: String) -> String;
    /// A resource's packed layout, how the runtime reads and writes its fields as bytes: a Result of `{ epoch, size, fields: [{ name, kind, offset, size, readonly, capacity, variants, element }] }` (`kind`: f32, f64, u32, i32, bool, Vector2, Vector3, Quaternion, Color, Matrix4, handle, str, enum, or list: `element` one of f32, f64, u32, i32, handle and `capacity` its elements, a count lane first, each lane the element's size). Refused for og.Time and og.Rng, whose handles are computed, and for what is not a registered resource.
    fn og_engine_resource_layout(&self, world: u32, resource: String) -> String;
    /// A resource's packed bytes, then the layout epoch as a little-endian u32 (it moves when any layout is declared again: a new engine, a reload). Empty when refused (undeclared in the running system, not a resource): the runtime then asks og_world_resource for the reason.
    fn og_engine_read_resource(&self, world: u32, resource: String) -> Self::Bytes;
    /// Overwrite a resource with packed bytes followed by the layout epoch they were made with (as og_engine_read_resource answers): how the runtime flushes a handle's field writes, once before the next engine call and at the end of every system. Read-only fields keep their value. A Result; refused for a stale epoch, a wrong size, an invalid value or a resource the running system did not declare among its writes.
    fn og_engine_write_resource_bytes(&self, world: u32, resource: String, bytes: Self::Bytes) -> String;
    /// The platform calls (IDL `platform`) made since the last call, oldest first, as JSON `[{ request, method, args, tick }]`. The host answers each but `telemetry` (one-way) with og_platform_answer.
    fn og_platform_take(&self, engine: u32) -> String;
    /// Answer one platform call: JSON `{ ok: true, data }` or `{ ok: false, error: { code, message, retryable } }`. It is applied at the start of the next tick (og.platform sends PlatformAnswered). False for an unknown, one-way or answered request, or a malformed answer.
    fn og_platform_answer(&self, engine: u32, request: u32, answer: String) -> bool;
    /// Say what answers this engine's platform calls: JSON `{ host, capabilities }` (`player`, `local` or `none`; the methods it answers), as og.platform.info reports.
    fn og_platform_set_host(&self, engine: u32, info: String);
    /// The room ops (IDL `platform` rooms, og.net's lockstep) made since the last call, oldest first, as JSON `[op]`: `{ op: "create", name, capacity }`, `{ op: "join", room, name }`, `{ op: "leave" }`, `{ op: "send", to?, channel?, data? | bytes? }` (bytes base64; a channel starting `og.` is the engine's own). The host carries each in order (docs/design.md §2.8).
    fn og_room_take(&self, engine: u32) -> String;
    /// One room event from the host, JSON: `{ type: "joined", room, peer, peers: [{ peer, name }] }`, `{ type: "peer_joined", peer, name }`, `{ type: "peer_left", peer }`, `{ type: "message", from, channel?, data? | bytes? }`, `{ type: "left", reason }` or `{ type: "error", op, code, message }`. Lockstep's messages act between ticks at once; the game's events are sent at the start of the next tick. False for a malformed event.
    fn og_room_deliver(&self, engine: u32, event: String) -> bool;
    /// Say what carries this engine's rooms: JSON `{ host, join? }` (`relay`, `player` or `none`), as og.platform.roomInfo reports; with `none` roomCreate and roomJoin are refused. `join` is the room a Play link named (`?room=<id>`): the engine joins it once, itself, and og.RoomJoined answers.
    fn og_room_set_host(&self, engine: u32, info: String);
}

/// `reflect`: packages/knowledge/wiki/reflect.md.
pub trait ReflectApi: Wire {
    /// `defineComponent`
    fn og_reflect_define_component(&self, name: String, fields: String) -> String;
    /// `defineResource`
    fn og_reflect_define_resource(&self, name: String, fields: String) -> String;
}

/// `world`: packages/knowledge/wiki/world.md.
pub trait WorldApi: Wire {
    /// `SaveEdit.version`
    fn og_save_edit_get_version(&self, save_edit: u32) -> u32;
    /// `SaveEdit.types`
    fn og_save_edit_types(&self, save_edit: u32) -> String;
    /// `SaveEdit.records`
    fn og_save_edit_records(&self, save_edit: u32, name: String) -> String;
    /// `SaveEdit.setRecords`
    fn og_save_edit_set_records(&self, save_edit: u32, name: String, desc: Option<String>) -> String;
    /// `SaveEdit.renameType`
    fn og_save_edit_rename_type(&self, save_edit: u32, from: String, to: String) -> String;
    /// `SaveEdit.renameField`
    fn og_save_edit_rename_field(&self, save_edit: u32, name: String, from: String, to: String) -> String;
    /// `SaveEdit.dropType`
    fn og_save_edit_drop_type(&self, save_edit: u32, name: String) -> String;
    /// `SaveEdit.spawn`
    fn og_save_edit_spawn(&self, save_edit: u32, desc: Option<String>) -> String;
    /// `SaveEdit.text`
    fn og_save_edit_text(&self, save_edit: u32) -> String;
    /// `SaveEdit.close`
    fn og_save_edit_close(&self, save_edit: u32);
    /// `SpatialIndex.count`
    fn og_spatial_index_get_count(&self, spatial_index: u32) -> u32;
    /// `SpatialIndex.within`
    fn og_spatial_index_within(&self, spatial_index: u32, center: String, radius: f64) -> String;
    /// `SpatialIndex.nearest`
    fn og_spatial_index_nearest(&self, spatial_index: u32, center: String, count: u32) -> String;
    /// `World.tick`
    fn og_world_get_tick(&self, world: u32) -> u32;
    /// `World.spawn`
    fn og_world_spawn(&self, world: u32, desc: Option<String>) -> String;
    /// `World.despawn`
    fn og_world_despawn(&self, world: u32, entity: f64) -> String;
    /// `World.alive`
    fn og_world_alive(&self, world: u32, entity: f64) -> bool;
    /// `World.has`
    fn og_world_has(&self, world: u32, entity: f64, component: String) -> bool;
    /// `World.insert`
    fn og_world_insert(&self, world: u32, entity: f64, component: String, value: Option<String>) -> String;
    /// `World.remove`
    fn og_world_remove(&self, world: u32, entity: f64, component: String) -> String;
    /// `World.get`
    fn og_world_get(&self, world: u32, entity: f64, component: String) -> String;
    /// `World.set`
    fn og_world_set(&self, world: u32, entity: f64, component: String, value: String) -> String;
    /// `World.resource`
    fn og_world_resource(&self, world: u32, resource: String) -> String;
    /// `World.children`
    fn og_world_children(&self, world: u32, entity: f64) -> String;
    /// `World.removed`
    fn og_world_removed(&self, world: u32, component: String) -> String;
    /// `World.spatial`
    fn og_world_spatial(&self, world: u32, desc: Option<String>) -> String;
    /// `World.state`
    fn og_world_state(&self, world: u32, name: String) -> String;
    /// `World.setState`
    fn og_world_set_state(&self, world: u32, name: String, value: String) -> String;
    /// `World.find`
    fn og_world_find(&self, world: u32, name: String) -> String;
    /// `World.prefabOf`
    fn og_world_prefab_of(&self, world: u32, entity: f64) -> String;
    /// `World.nameOf`
    fn og_world_name_of(&self, world: u32, entity: f64) -> String;
    /// `World.query`
    fn og_world_query(&self, world: u32, desc: Option<String>) -> String;
    /// `World.events`
    fn og_world_events(&self, world: u32, event: String) -> String;
    /// `World.emit`
    fn og_world_emit(&self, world: u32, event: String, payload: String) -> String;
    /// `World.instantiate`
    fn og_world_instantiate(&self, world: u32, asset: f64, desc: Option<String>) -> String;
    /// `World.snapshot`
    fn og_world_snapshot(&self, world: u32, desc: Option<String>) -> String;
    /// `World.openSave`
    fn og_world_open_save(&self, world: u32, save: String) -> String;
    /// `World.restore`
    fn og_world_restore(&self, world: u32, save: String, desc: Option<String>) -> String;
    /// `Query.archetypes`
    fn og_query_archetypes(&self, query: u32) -> String;
    /// `Query.count`
    fn og_query_count(&self, query: u32) -> u32;
    /// `ArchetypeView.count`
    fn og_archetype_view_get_count(&self, archetype_view: u32) -> u32;
    /// `ArchetypeView.tick`
    fn og_archetype_view_get_tick(&self, archetype_view: u32) -> u32;
    /// `ArchetypeView.entities`
    fn og_archetype_view_get_entities(&self, archetype_view: u32) -> Self::Array;
    /// `ArchetypeView.rows`
    fn og_archetype_view_get_rows(&self, archetype_view: u32) -> Self::Array;
    /// `ArchetypeView.entity`
    fn og_archetype_view_entity(&self, archetype_view: u32, index: u32) -> f64;
    /// `ArchetypeView.has`
    fn og_archetype_view_has(&self, archetype_view: u32, component: String) -> bool;
    /// `ArchetypeView.column`
    fn og_archetype_view_column(&self, archetype_view: u32, component: String) -> Self::Views;
}

/// `system`: packages/knowledge/wiki/system.md.
pub trait SystemApi: Wire {
    /// `Hook.name`
    fn og_hook_get_name(&self, hook: u32) -> String;
    /// `defineHook`
    fn og_system_define_hook(&self, name: String, component: String, desc: String, run: u32) -> u32;
    /// `System.name`
    fn og_system_get_name(&self, system: u32) -> String;
    /// `System.reads`
    fn og_system_get_reads(&self, system: u32) -> String;
    /// `System.writes`
    fn og_system_get_writes(&self, system: u32) -> String;
    /// `defineSystem`
    fn og_system_define_system(&self, name: String, desc: String, run: u32) -> u32;
}

/// `game`: packages/knowledge/wiki/game.md.
pub trait GameApi: Wire {
    /// `Game.tick`
    fn og_game_get_tick(&self, game: u32) -> u32;
    /// `Game.systems`
    fn og_game_get_systems(&self, game: u32) -> String;
    /// `og.game`
    fn og_game_game(&self, desc: Option<String>) -> u32;
}

/// `transform`: packages/knowledge/wiki/transform.md.
pub trait TransformApi: Wire {
    /// `og.transform.compose`
    fn og_transform_compose(&self, position: String, quaternion: String, scale: String) -> String;
    /// `og.transform.decompose`
    fn og_transform_decompose(&self, matrix: String) -> String;
    /// `og.transform.lookAt`
    fn og_transform_look_at(&self, eye: String, target: String, up: Option<String>) -> String;
    /// `og.transform.fromAxisAngle`
    fn og_transform_from_axis_angle(&self, axis: String, angle: f64) -> String;
}

/// `time`: packages/knowledge/wiki/time.md.
pub trait TimeApi: Wire {
    /// `og.time.setScale`
    fn og_time_set_scale(&self, world: u32, scale: f64) -> String;
    /// `og.time.hitStop`
    fn og_time_hit_stop(&self, world: u32, ticks: u32) -> String;
}

/// `rng`: packages/knowledge/wiki/rng.md.
pub trait RngApi: Wire {
    /// `RngStream.seed`
    fn og_rng_stream_get_seed(&self, rng_stream: u32) -> u32;
    /// `RngStream.next`
    fn og_rng_stream_next(&self, rng_stream: u32) -> f64;
    /// `RngStream.range`
    fn og_rng_stream_range(&self, rng_stream: u32, low: f64, high: f64) -> f64;
    /// `RngStream.integer`
    fn og_rng_stream_integer(&self, rng_stream: u32, low: i32, high: i32) -> i32;
    /// `RngStream.chance`
    fn og_rng_stream_chance(&self, rng_stream: u32, p: f64) -> bool;
    /// `RngStream.fork`
    fn og_rng_stream_fork(&self, rng_stream: u32, label: String) -> u32;
    /// `Rng.next`
    fn og_rng_next(&self, world: u32) -> f64;
    /// `Rng.range`
    fn og_rng_range(&self, world: u32, low: f64, high: f64) -> f64;
    /// `Rng.integer`
    fn og_rng_integer(&self, world: u32, low: i32, high: i32) -> i32;
    /// `Rng.chance`
    fn og_rng_chance(&self, world: u32, p: f64) -> bool;
    /// `Rng.fork`
    fn og_rng_fork(&self, world: u32, label: String) -> u32;
}

/// `input`: packages/knowledge/wiki/input.md.
pub trait InputApi: Wire {
    /// `og.input.defineAction`
    fn og_input_define_action(&self, world: u32, desc: Option<String>) -> String;
    /// `og.input.actions`
    fn og_input_actions(&self, world: u32) -> String;
    /// `og.input.action`
    fn og_input_action(&self, world: u32, name: String) -> String;
    /// `og.input.actionDown`
    fn og_input_action_down(&self, world: u32, name: String) -> bool;
    /// `og.input.actionPressed`
    fn og_input_action_pressed(&self, world: u32, name: String) -> bool;
    /// `og.input.actionReleased`
    fn og_input_action_released(&self, world: u32, name: String) -> bool;
    /// `og.input.actionHeld`
    fn og_input_action_held(&self, world: u32, name: String) -> bool;
    /// `og.input.actionValue`
    fn og_input_action_value(&self, world: u32, name: String) -> f64;
    /// `og.input.actionVector`
    fn og_input_action_vector(&self, world: u32, name: String) -> String;
    /// `og.input.setContexts`
    fn og_input_set_contexts(&self, world: u32, contexts: String);
    /// `og.input.contexts`
    fn og_input_contexts(&self, world: u32) -> String;
    /// `og.input.rebind`
    fn og_input_rebind(&self, world: u32, desc: Option<String>) -> String;
    /// `og.input.conflicts`
    fn og_input_conflicts(&self, world: u32, desc: Option<String>) -> String;
    /// `og.input.resetBindings`
    fn og_input_reset_bindings(&self, world: u32, action: Option<String>) -> String;
    /// `og.input.setToggle`
    fn og_input_set_toggle(&self, world: u32, action: String, on: bool) -> String;
    /// `og.input.saveBindings`
    fn og_input_save_bindings(&self, world: u32) -> String;
    /// `og.input.loadBindings`
    fn og_input_load_bindings(&self, world: u32, text: String) -> String;
    /// `og.input.startCapture`
    fn og_input_start_capture(&self, world: u32, desc: Option<String>) -> String;
    /// `og.input.capture`
    fn og_input_capture(&self, world: u32) -> String;
    /// `og.input.cancelCapture`
    fn og_input_cancel_capture(&self, world: u32);
    /// `og.input.applyCapture`
    fn og_input_apply_capture(&self, world: u32, policy: Option<String>) -> String;
    /// `og.input.glyph`
    fn og_input_glyph(&self, world: u32, action: String, device: Option<String>) -> String;
    /// `og.input.glyphs`
    fn og_input_glyphs(&self, world: u32, action: String, device: Option<String>) -> String;
    /// `og.input.controlGlyph`
    fn og_input_control_glyph(&self, control: String, device: Option<String>) -> String;
    /// `og.input.device`
    fn og_input_device(&self, world: u32) -> String;
    /// `og.input.padDevice`
    fn og_input_pad_device(&self, world: u32, pad: u32) -> String;
    /// `og.input.setDeadzones`
    fn og_input_set_deadzones(&self, world: u32, desc: Option<String>) -> String;
    /// `og.input.setSensitivity`
    fn og_input_set_sensitivity(&self, world: u32, desc: Option<String>) -> String;
    /// `og.input.sensitivity`
    fn og_input_sensitivity(&self, world: u32, action: String, device: Option<String>) -> String;
    /// `og.input.setKeyboardLayout`
    fn og_input_set_keyboard_layout(&self, labels: String);
    /// `og.input.keyboardLayout`
    fn og_input_keyboard_layout(&self) -> String;
    /// `og.input.setMouse`
    fn og_input_set_mouse(&self, world: u32, desc: Option<String>) -> String;
    /// `og.input.mouse`
    fn og_input_mouse(&self, world: u32) -> String;
    /// `og.input.rumble`
    fn og_input_rumble(&self, world: u32, desc: Option<String>) -> String;
    /// `Input.down`
    fn og_input_down(&self, world: u32, key: String) -> bool;
    /// `Input.pressed`
    fn og_input_pressed(&self, world: u32, key: String) -> bool;
    /// `Input.released`
    fn og_input_released(&self, world: u32, key: String) -> bool;
    /// `Input.axis`
    fn og_input_axis(&self, world: u32, negative: String, positive: String) -> f64;
    /// `Input.axis2`
    fn og_input_axis2(&self, world: u32, left: String, right: String, down: String, up: String) -> String;
    /// `Input.buttonDown`
    fn og_input_button_down(&self, world: u32, button: String) -> bool;
    /// `Input.buttonPressed`
    fn og_input_button_pressed(&self, world: u32, button: String) -> bool;
    /// `Input.buttonReleased`
    fn og_input_button_released(&self, world: u32, button: String) -> bool;
    /// `Pointers.down`
    fn og_pointers_down(&self, world: u32, id: u32) -> bool;
    /// `Pointers.pressed`
    fn og_pointers_pressed(&self, world: u32, id: u32) -> bool;
    /// `Pointers.released`
    fn og_pointers_released(&self, world: u32, id: u32) -> bool;
    /// `Pointers.position`
    fn og_pointers_position(&self, world: u32, id: u32) -> String;
    /// `Pointers.start`
    fn og_pointers_start(&self, world: u32, id: u32) -> String;
    /// `Pointers.count`
    fn og_pointers_count(&self, world: u32) -> u32;
    /// `Gamepad.connected`
    fn og_gamepad_connected(&self, world: u32, pad: Option<u32>) -> bool;
    /// `Gamepad.down`
    fn og_gamepad_down(&self, world: u32, button: String, pad: Option<u32>) -> bool;
    /// `Gamepad.pressed`
    fn og_gamepad_pressed(&self, world: u32, button: String, pad: Option<u32>) -> bool;
    /// `Gamepad.released`
    fn og_gamepad_released(&self, world: u32, button: String, pad: Option<u32>) -> bool;
    /// `Gamepad.axis`
    fn og_gamepad_axis(&self, world: u32, axis: String, pad: Option<u32>) -> f64;
}

/// `camera`: packages/knowledge/wiki/camera.md.
pub trait CameraApi: Wire {
    /// `Camera.lookAt`
    fn og_camera_look_at(&self, world: u32, target: String);
    /// `Camera.worldToScreen`
    fn og_camera_world_to_screen(&self, world: u32, point: String) -> String;
    /// `Camera.screenToRay`
    fn og_camera_screen_to_ray(&self, world: u32, x: f64, y: f64) -> String;
    /// `og.camera.shake`
    fn og_camera_shake(&self, world: u32, trauma: f64) -> String;
    /// `og.camera.punch`
    fn og_camera_punch(&self, world: u32, amount: f64) -> String;
    /// `og.camera.kick`
    fn og_camera_kick(&self, world: u32, direction: String, distance: f64) -> String;
}

/// `render`: packages/knowledge/wiki/render.md.
pub trait RenderApi: Wire {
    /// `og.render.snap`
    fn og_render_snap(&self, world: u32, entity: f64) -> String;
    /// `og.render.snapCamera`
    fn og_render_snap_camera(&self, world: u32);
    /// `og.render.ribbon`
    fn og_render_ribbon(&self, world: u32, desc: Option<String>) -> String;
    /// `og.render.sprite`
    fn og_render_sprite(&self, world: u32, desc: Option<String>) -> String;
    /// `og.render.floatingText`
    fn og_render_floating_text(&self, world: u32, desc: Option<String>) -> String;
    /// `og.render.measureText`
    fn og_render_measure_text(&self, world: u32, desc: Option<String>) -> String;
    /// `og.render.boxGeometry`
    fn og_render_box_geometry(&self, world: u32, desc: Option<String>) -> String;
    /// `og.render.planeGeometry`
    fn og_render_plane_geometry(&self, world: u32, desc: Option<String>) -> String;
    /// `og.render.sphereGeometry`
    fn og_render_sphere_geometry(&self, world: u32, desc: Option<String>) -> String;
    /// `og.render.standardMaterial`
    fn og_render_standard_material(&self, world: u32, desc: Option<String>) -> String;
    /// `og.render.updateMaterial`
    fn og_render_update_material(&self, world: u32, material: f64, desc: Option<String>) -> String;
    /// `og.render.shaderMaterial`
    fn og_render_shader_material(&self, world: u32, shader: f64, desc: Option<String>) -> String;
    /// `og.render.setMaterialShader`
    fn og_render_set_material_shader(&self, world: u32, material: f64, shader: String) -> String;
    /// `og.render.setShaderParam`
    fn og_render_set_shader_param(&self, world: u32, material: f64, name: String, value: String) -> String;
    /// `og.render.applyLook`
    fn og_render_apply_look(&self, world: u32, look: f64) -> String;
    /// `og.render.applyQuality`
    fn og_render_apply_quality(&self, world: u32, preset: String) -> String;
    /// `og.render.setDisplay`
    fn og_render_set_display(&self, world: u32, settings: Option<String>) -> String;
    /// `og.render.resetDisplay`
    fn og_render_reset_display(&self, world: u32) -> String;
    /// `og.render.display`
    fn og_render_display(&self, world: u32) -> String;
    /// `og.render.look`
    fn og_render_look(&self, world: u32) -> String;
    /// `og.render.capabilities`
    fn og_render_capabilities(&self, world: u32) -> String;
    /// `og.render.text`
    fn og_render_text(&self, world: u32, desc: Option<String>) -> String;
    /// `og.render.bufferGeometry`
    fn og_render_buffer_geometry(&self, world: u32, desc: Option<String>) -> String;
    /// `og.render.updateGeometry`
    fn og_render_update_geometry(&self, world: u32, geometry: f64, desc: Option<String>) -> String;
    /// `og.render.geometryInfo`
    fn og_render_geometry_info(&self, world: u32, geometry: f64) -> String;
    /// `og.render.cylinderGeometry`
    fn og_render_cylinder_geometry(&self, world: u32, desc: Option<String>) -> String;
    /// `og.render.coneGeometry`
    fn og_render_cone_geometry(&self, world: u32, desc: Option<String>) -> String;
    /// `og.render.capsuleGeometry`
    fn og_render_capsule_geometry(&self, world: u32, desc: Option<String>) -> String;
    /// `og.render.torusGeometry`
    fn og_render_torus_geometry(&self, world: u32, desc: Option<String>) -> String;
    /// `og.render.ringGeometry`
    fn og_render_ring_geometry(&self, world: u32, desc: Option<String>) -> String;
    /// `og.render.circleGeometry`
    fn og_render_circle_geometry(&self, world: u32, desc: Option<String>) -> String;
    /// `og.render.tubeGeometry`
    fn og_render_tube_geometry(&self, world: u32, desc: Option<String>) -> String;
    /// `og.render.shapeGeometry`
    fn og_render_shape_geometry(&self, world: u32, desc: Option<String>) -> String;
    /// `og.render.extrudeGeometry`
    fn og_render_extrude_geometry(&self, world: u32, desc: Option<String>) -> String;
    /// `og.render.latheGeometry`
    fn og_render_lathe_geometry(&self, world: u32, desc: Option<String>) -> String;
    /// `og.render.polyhedronGeometry`
    fn og_render_polyhedron_geometry(&self, world: u32, desc: Option<String>) -> String;
    /// `og.render.icosahedronGeometry`
    fn og_render_icosahedron_geometry(&self, world: u32, desc: Option<String>) -> String;
    /// `og.render.dodecahedronGeometry`
    fn og_render_dodecahedron_geometry(&self, world: u32, desc: Option<String>) -> String;
    /// `og.render.octahedronGeometry`
    fn og_render_octahedron_geometry(&self, world: u32, desc: Option<String>) -> String;
    /// `og.render.tetrahedronGeometry`
    fn og_render_tetrahedron_geometry(&self, world: u32, desc: Option<String>) -> String;
    /// `og.render.sweepGeometry`
    fn og_render_sweep_geometry(&self, world: u32, desc: Option<String>) -> String;
    /// `og.render.csgGeometry`
    fn og_render_csg_geometry(&self, world: u32, desc: Option<String>) -> String;
    /// `og.render.mergeGeometries`
    fn og_render_merge_geometries(&self, world: u32, desc: Option<String>) -> String;
    /// `og.render.lines`
    fn og_render_lines(&self, world: u32, desc: Option<String>) -> String;
    /// `og.render.points`
    fn og_render_points(&self, world: u32, desc: Option<String>) -> String;
    /// `og.render.raycast`
    fn og_render_raycast(&self, world: u32, ray: String, desc: Option<String>) -> String;
    /// `og.render.renderTarget`
    fn og_render_render_target(&self, world: u32, desc: Option<String>) -> String;
    /// `og.render.setPostParam`
    fn og_render_set_post_param(&self, world: u32, post: f64, name: String, value: String) -> String;
    /// `og.render.postErrors`
    fn og_render_post_errors(&self, world: u32) -> String;
    /// `og.render.bakedLighting`
    fn og_render_baked_lighting(&self, world: u32) -> String;
}

/// `load`: packages/knowledge/wiki/load.md.
pub trait LoadApi: Wire {
    /// `og.load.bundle`
    fn og_load_bundle(&self, world: u32, path: String) -> String;
    /// `og.load.bundleInfo`
    fn og_load_bundle_info(&self, world: u32, asset: f64) -> String;
    /// `og.load.defaultBundle`
    fn og_load_default_bundle(&self, world: u32) -> String;
    /// `og.load.gltf`
    fn og_load_gltf(&self, world: u32, path: String, desc: Option<String>) -> String;
    /// `og.load.scene`
    fn og_load_scene(&self, world: u32, path: String) -> String;
    /// `og.load.prefab`
    fn og_load_prefab(&self, world: u32, path: String) -> String;
    /// `og.load.font`
    fn og_load_font(&self, world: u32, path: String) -> String;
    /// `og.load.fontInfo`
    fn og_load_font_info(&self, world: u32, font: f64) -> String;
    /// `og.load.sound`
    fn og_load_sound(&self, world: u32, path: String) -> String;
    /// `og.load.lut`
    fn og_load_lut(&self, world: u32, path: String) -> String;
    /// `og.load.environment`
    fn og_load_environment(&self, world: u32, path: String) -> String;
    /// `og.load.emitter`
    fn og_load_emitter(&self, world: u32, path: String) -> String;
    /// `og.load.shader`
    fn og_load_shader(&self, world: u32, path: String) -> String;
    /// `og.load.post`
    fn og_load_post(&self, world: u32, path: String) -> String;
    /// `og.load.vector`
    fn og_load_vector(&self, world: u32, path: String) -> String;
    /// `og.load.uiMaterial`
    fn og_load_ui_material(&self, world: u32, path: String) -> String;
    /// `og.load.texture`
    fn og_load_texture(&self, world: u32, path: String) -> String;
    /// `og.load.textureArray`
    fn og_load_texture_array(&self, world: u32, path: String) -> String;
    /// `og.load.style`
    fn og_load_style(&self, world: u32, path: String) -> String;
    /// `og.load.document`
    fn og_load_document(&self, world: u32, path: String) -> String;
    /// `og.load.look`
    fn og_load_look(&self, world: u32, path: String) -> String;
    /// `og.load.lookInfo`
    fn og_load_look_info(&self, world: u32, look: f64) -> String;
    /// `og.load.palette`
    fn og_load_palette(&self, world: u32, path: String) -> String;
    /// `og.load.soundInfo`
    fn og_load_sound_info(&self, world: u32, sound: f64) -> String;
    /// `og.load.status`
    fn og_load_status(&self, world: u32, asset: f64) -> String;
    /// `og.load.error`
    fn og_load_error(&self, world: u32, asset: f64) -> String;
    /// `og.load.gltfInfo`
    fn og_load_gltf_info(&self, world: u32, asset: f64) -> String;
    /// `og.load.animation`
    fn og_load_animation(&self, world: u32, path: String) -> String;
    /// `og.load.rig`
    fn og_load_rig(&self, world: u32, path: String) -> String;
    /// `og.load.flipbook`
    fn og_load_flipbook(&self, world: u32, path: String) -> String;
    /// `og.load.tilemap`
    fn og_load_tilemap(&self, world: u32, path: String) -> String;
    /// `og.load.video`
    fn og_load_video(&self, world: u32, path: String) -> String;
    /// `og.load.videoInfo`
    fn og_load_video_info(&self, world: u32, video: f64) -> String;
    /// `og.load.timeline`
    fn og_load_timeline(&self, world: u32, path: String) -> String;
    /// `og.load.rig2d`
    fn og_load_rig2d(&self, world: u32, path: String) -> String;
}

/// `physics`: packages/knowledge/wiki/physics.md.
pub trait PhysicsApi: Wire {
    /// `og.physics.raycast`
    fn og_physics_raycast(&self, world: u32, desc: Option<String>) -> String;
    /// `og.physics.shapeCast`
    fn og_physics_shape_cast(&self, world: u32, desc: Option<String>) -> String;
    /// `og.physics.overlap`
    fn og_physics_overlap(&self, world: u32, desc: Option<String>) -> String;
    /// `og.physics.applyForce`
    fn og_physics_apply_force(&self, world: u32, entity: f64, force: String, point: Option<String>) -> String;
    /// `og.physics.applyImpulse`
    fn og_physics_apply_impulse(&self, world: u32, entity: f64, impulse: String, point: Option<String>) -> String;
    /// `og.physics.applyTorque`
    fn og_physics_apply_torque(&self, world: u32, entity: f64, torque: String) -> String;
    /// `og.physics.applyTorqueImpulse`
    fn og_physics_apply_torque_impulse(&self, world: u32, entity: f64, torque: String) -> String;
    /// `og.physics.setMesh`
    fn og_physics_set_mesh(&self, world: u32, mesh: f64, desc: Option<String>) -> String;
    /// `og.physics.ropePoints`
    fn og_physics_rope_points(&self, world: u32, entity: f64) -> String;
    /// `og.physics.clothPoints`
    fn og_physics_cloth_points(&self, world: u32, entity: f64) -> String;
    /// `og.physics.fluidParticles`
    fn og_physics_fluid_particles(&self, world: u32, entity: f64) -> String;
    /// `og.physics.fluid3dParticles`
    fn og_physics_fluid3d_particles(&self, world: u32, entity: f64) -> String;
    /// `og.physics.waterHeight`
    fn og_physics_water_height(&self, world: u32, entity: f64, x: f64, z: f64) -> String;
    /// `og.physics.vehicleWheels`
    fn og_physics_vehicle_wheels(&self, world: u32, entity: f64) -> String;
    /// `og.physics.ragdollRigFromSkeleton`
    fn og_physics_ragdoll_rig_from_skeleton(&self, world: u32, skeleton: f64) -> String;
    /// `og.physics.setRagdollRig`
    fn og_physics_set_ragdoll_rig(&self, world: u32, entity: f64, desc: Option<String>) -> String;
    /// `og.physics.ragdollPose`
    fn og_physics_ragdoll_pose(&self, world: u32, entity: f64) -> String;
    /// `og.physics.stats`
    fn og_physics_stats(&self, world: u32) -> String;
}

/// `atmosphere`: packages/knowledge/wiki/atmosphere.md.
pub trait AtmosphereApi: Wire {
    /// `og.atmosphere.timeOfDay`
    fn og_atmosphere_time_of_day(&self, world: u32) -> f64;
}

/// `anim`: packages/knowledge/wiki/anim.md.
pub trait AnimApi: Wire {
    /// `og.anim.graph`
    fn og_anim_graph(&self, world: u32, desc: Option<String>) -> String;
    /// `og.anim.humanoid`
    fn og_anim_humanoid(&self, world: u32, desc: Option<String>) -> String;
    /// `og.anim.retarget`
    fn og_anim_retarget(&self, world: u32, desc: Option<String>) -> String;
    /// `og.anim.mirror`
    fn og_anim_mirror(&self, world: u32, desc: Option<String>) -> String;
    /// `og.anim.rig2dInfo`
    fn og_anim_rig2d_info(&self, world: u32, rig: f64) -> String;
    /// `og.anim.skeleton2dBoxes`
    fn og_anim_skeleton2d_boxes(&self, world: u32, entity: f64) -> String;
}

/// `tween`: packages/knowledge/wiki/tween.md.
pub trait TweenApi: Wire {
    /// `og.tween.to`
    fn og_tween_to(&self, world: u32, desc: Option<String>) -> String;
    /// `og.tween.sequence`
    fn og_tween_sequence(&self, world: u32, steps: String, desc: Option<String>) -> String;
    /// `og.tween.parallel`
    fn og_tween_parallel(&self, world: u32, tweens: String, desc: Option<String>) -> String;
    /// `og.tween.cancel`
    fn og_tween_cancel(&self, world: u32, tween: f64) -> bool;
    /// `og.tween.cancelEntity`
    fn og_tween_cancel_entity(&self, world: u32, entity: f64) -> u32;
    /// `og.tween.active`
    fn og_tween_active(&self, world: u32, tween: f64) -> bool;
    /// `og.tween.squash`
    fn og_tween_squash(&self, world: u32, desc: Option<String>) -> String;
    /// `og.tween.flash`
    fn og_tween_flash(&self, world: u32, desc: Option<String>) -> String;
}

/// `tilemap`: packages/knowledge/wiki/tilemap.md.
pub trait TilemapApi: Wire {
    /// `og.tilemap.setTile`
    fn og_tilemap_set_tile(&self, world: u32, desc: Option<String>) -> String;
    /// `og.tilemap.getTile`
    fn og_tilemap_get_tile(&self, world: u32, entity: f64, layer: String, x: u32, y: u32) -> i32;
    /// `og.tilemap.cellAt`
    fn og_tilemap_cell_at(&self, world: u32, entity: f64, point: String) -> String;
    /// `og.tilemap.cellCenter`
    fn og_tilemap_cell_center(&self, world: u32, entity: f64, x: u32, y: u32) -> String;
}

/// `audio`: packages/knowledge/wiki/audio.md.
pub trait AudioApi: Wire {
    /// `og.audio.createSound`
    fn og_audio_create_sound(&self, world: u32, desc: Option<String>) -> String;
    /// `og.audio.outputLatency`
    fn og_audio_output_latency(&self, world: u32) -> f64;
}

/// `nav`: packages/knowledge/wiki/nav.md.
pub trait NavApi: Wire {
    /// `og.nav.grid`
    fn og_nav_grid(&self, world: u32, desc: Option<String>) -> String;
    /// `og.nav.gridFromTilemap`
    fn og_nav_grid_from_tilemap(&self, world: u32, desc: Option<String>) -> String;
    /// `og.nav.setCost`
    fn og_nav_set_cost(&self, world: u32, grid: f64, x: u32, y: u32, cost: u32) -> String;
    /// `og.nav.cellAt`
    fn og_nav_cell_at(&self, world: u32, grid: f64, point: String) -> String;
    /// `og.nav.findPath`
    fn og_nav_find_path(&self, world: u32, desc: Option<String>) -> String;
    /// `og.nav.flowField`
    fn og_nav_flow_field(&self, world: u32, desc: Option<String>) -> String;
    /// `og.nav.setFlowTargets`
    fn og_nav_set_flow_targets(&self, world: u32, field: f64, targets: String) -> String;
    /// `og.nav.flowAt`
    fn og_nav_flow_at(&self, world: u32, field: f64, point: String) -> String;
    /// `og.nav.load`
    fn og_nav_load(&self, world: u32, path: String) -> String;
    /// `og.nav.build`
    fn og_nav_build(&self, world: u32, desc: Option<String>) -> String;
    /// `og.nav.setObstacle`
    fn og_nav_set_obstacle(&self, world: u32, mesh: f64, id: u32, obstacle: Option<String>) -> String;
    /// `og.nav.nearest`
    fn og_nav_nearest(&self, world: u32, mesh: f64, point: String, extent: Option<String>) -> String;
    /// `og.nav.pathOf`
    fn og_nav_path_of(&self, world: u32, entity: f64) -> String;
}

/// `ai`: packages/knowledge/wiki/ai.md.
pub trait AiApi: Wire {
    /// `og.ai.load`
    fn og_ai_load(&self, world: u32, path: String) -> String;
    /// `og.ai.parse`
    fn og_ai_parse(&self, world: u32, text: String) -> String;
    /// `og.ai.get`
    fn og_ai_get(&self, world: u32, entity: f64, key: String) -> String;
    /// `og.ai.set`
    fn og_ai_set(&self, world: u32, entity: f64, key: String, value: String) -> String;
    /// `og.ai.restart`
    fn og_ai_restart(&self, world: u32, entity: f64) -> String;
    /// `og.ai.noise`
    fn og_ai_noise(&self, world: u32, desc: Option<String>) -> String;
    /// `og.ai.describe`
    fn og_ai_describe(&self, world: u32, entity: f64) -> String;
}

/// `platform`: packages/knowledge/wiki/platform.md.
pub trait PlatformApi: Wire {
    /// `og.platform.storeGet`
    fn og_platform_store_get(&self, world: u32, key: String) -> String;
    /// `og.platform.storeSet`
    fn og_platform_store_set(&self, world: u32, desc: Option<String>) -> String;
    /// `og.platform.storeList`
    fn og_platform_store_list(&self, world: u32) -> u32;
    /// `og.platform.leaderboardSubmit`
    fn og_platform_leaderboard_submit(&self, world: u32, desc: Option<String>) -> String;
    /// `og.platform.leaderboardTop`
    fn og_platform_leaderboard_top(&self, world: u32, desc: Option<String>) -> u32;
    /// `og.platform.leaderboardAround`
    fn og_platform_leaderboard_around(&self, world: u32, desc: Option<String>) -> u32;
    /// `og.platform.telemetry`
    fn og_platform_telemetry(&self, world: u32, name: String, attrs: Option<String>) -> String;
    /// `og.platform.point`
    fn og_platform_point(&self, world: u32, desc: Option<String>) -> String;
    /// `og.platform.aiMark`
    fn og_platform_ai_mark(&self, world: u32, desc: Option<String>) -> String;
    /// `og.platform.loadSettings`
    fn og_platform_load_settings(&self, world: u32) -> u32;
    /// `og.platform.saveSettings`
    fn og_platform_save_settings(&self, world: u32, value: String) -> String;
    /// `og.platform.saveSlot`
    fn og_platform_save_slot(&self, world: u32, desc: Option<String>) -> String;
    /// `og.platform.loadSlot`
    fn og_platform_load_slot(&self, world: u32, desc: Option<String>) -> String;
    /// `og.platform.result`
    fn og_platform_result(&self, world: u32, request: u32) -> String;
    /// `og.platform.info`
    fn og_platform_info(&self, world: u32) -> String;
    /// `og.platform.now`
    fn og_platform_now(&self, world: u32) -> f64;
    /// `og.platform.roomCreate`
    fn og_platform_room_create(&self, world: u32, desc: Option<String>) -> String;
    /// `og.platform.roomJoin`
    fn og_platform_room_join(&self, world: u32, room: String) -> String;
    /// `og.platform.roomLeave`
    fn og_platform_room_leave(&self, world: u32) -> String;
    /// `og.platform.roomSend`
    fn og_platform_room_send(&self, world: u32, data: String, to: Option<u32>) -> String;
    /// `og.platform.roomPeers`
    fn og_platform_room_peers(&self, world: u32) -> String;
    /// `og.platform.roomInfo`
    fn og_platform_room_info(&self, world: u32) -> String;
    /// `og.platform.roomMessage`
    fn og_platform_room_message(&self, world: u32, message: u32) -> String;
}

/// `i18n`: packages/knowledge/wiki/i18n.md.
pub trait I18nApi: Wire {
    /// `og.i18n.load`
    fn og_i18n_load(&self, world: u32, path: String) -> String;
    /// `og.i18n.setLocale`
    fn og_i18n_set_locale(&self, world: u32, locale: String) -> String;
    /// `og.i18n.locale`
    fn og_i18n_locale(&self, world: u32) -> String;
    /// `og.i18n.t`
    fn og_i18n_t(&self, world: u32, key: String, args: Option<String>) -> String;
    /// `og.i18n.has`
    fn og_i18n_has(&self, world: u32, key: String) -> bool;
}

/// `ui`: packages/knowledge/wiki/ui.md.
pub trait UiApi: Wire {
    /// `og.ui.configure`
    fn og_ui_configure(&self, world: u32, desc: Option<String>) -> String;
    /// `og.ui.viewport`
    fn og_ui_viewport(&self, world: u32) -> String;
    /// `og.ui.theme`
    fn og_ui_theme(&self, world: u32) -> String;
    /// `og.ui.open`
    fn og_ui_open(&self, id: String, desc: Option<String>);
    /// `og.ui.openPanel`
    fn og_ui_open_panel(&self, id: String, desc: Option<String>);
    /// `og.ui.openScroll`
    fn og_ui_open_scroll(&self, id: String, desc: Option<String>);
    /// `og.ui.close`
    fn og_ui_close(&self) -> String;
    /// `og.ui.richText`
    fn og_ui_rich_text(&self, id: String, markup: String, desc: Option<String>) -> String;
    /// `og.ui.label`
    fn og_ui_label(&self, id: String, text: String, desc: Option<String>);
    /// `og.ui.button`
    fn og_ui_button(&self, id: String, text: String, desc: Option<String>) -> String;
    /// `og.ui.image`
    fn og_ui_image(&self, id: String, desc: Option<String>);
    /// `og.ui.zoomView`
    fn og_ui_zoom_view(&self, id: String, desc: Option<String>) -> String;
    /// `og.ui.progress`
    fn og_ui_progress(&self, id: String, value: f64, desc: Option<String>);
    /// `og.ui.slider`
    fn og_ui_slider(&self, id: String, value: f64, desc: Option<String>) -> String;
    /// `og.ui.toggle`
    fn og_ui_toggle(&self, id: String, on: bool, label: String) -> String;
    /// `og.ui.textField`
    fn og_ui_text_field(&self, id: String, value: String, desc: Option<String>) -> String;
    /// `og.ui.textArea`
    fn og_ui_text_area(&self, id: String, value: String, desc: Option<String>) -> String;
    /// `og.ui.tooltip`
    fn og_ui_tooltip(&self, text: String);
    /// `og.ui.breakpoint`
    fn og_ui_breakpoint(&self) -> String;
    /// `og.ui.framePointer`
    fn og_ui_frame_pointer(&self) -> String;
    /// `og.ui.frameViewport`
    fn og_ui_frame_viewport(&self) -> String;
    /// `og.ui.hasIcon`
    fn og_ui_has_icon(&self, name: String) -> bool;
    /// `og.ui.seconds`
    fn og_ui_seconds(&self) -> f64;
    /// `og.ui.lastPath`
    fn og_ui_last_path(&self) -> String;
    /// `og.ui.setFocus`
    fn og_ui_set_focus(&self, path: String);
    /// `og.ui.frameDirection`
    fn og_ui_frame_direction(&self) -> String;
    /// `og.ui.frameTheme`
    fn og_ui_frame_theme(&self) -> String;
    /// `og.ui.openAnchor`
    fn og_ui_open_anchor(&self, id: String, desc: Option<String>);
    /// `og.ui.openTransition`
    fn og_ui_open_transition(&self, id: String, show: bool, desc: Option<String>) -> String;
    /// `og.ui.openModal`
    fn og_ui_open_modal(&self, id: String, open: bool, desc: Option<String>) -> String;
    /// `og.ui.closeModal`
    fn og_ui_close_modal(&self);
    /// `og.ui.spotlight`
    fn og_ui_spotlight(&self, id: String, hole: String, desc: Option<String>) -> String;
    /// `og.ui.bounds`
    fn og_ui_bounds(&self, path: String) -> String;
    /// `og.ui.openList`
    fn og_ui_open_list(&self, id: String, desc: Option<String>) -> String;
    /// `og.ui.openGrid`
    fn og_ui_open_grid(&self, id: String, desc: Option<String>) -> String;
    /// `og.ui.listItem`
    fn og_ui_list_item(&self, index: u32, selected: bool) -> String;
    /// `og.ui.gridItem`
    fn og_ui_grid_item(&self, index: u32, selected: bool) -> String;
    /// `og.ui.closeList`
    fn og_ui_close_list(&self) -> String;
    /// `og.ui.dragging`
    fn og_ui_dragging(&self, id: String) -> String;
    /// `og.ui.tabs`
    fn og_ui_tabs(&self, id: String, items: String, selected: u32, desc: Option<String>) -> String;
    /// `og.ui.select`
    fn og_ui_select(&self, id: String, options: String, selected: u32, desc: Option<String>) -> String;
    /// `og.ui.stepper`
    fn og_ui_stepper(&self, id: String, value: f64, desc: Option<String>) -> String;
    /// `og.ui.radio`
    fn og_ui_radio(&self, id: String, options: String, selected: u32, desc: Option<String>) -> String;
    /// `og.ui.checkbox`
    fn og_ui_checkbox(&self, id: String, on: bool, label: String) -> String;
    /// `og.ui.checkboxList`
    fn og_ui_checkbox_list(&self, id: String, labels: String, values: String) -> String;
    /// `og.ui.chip`
    fn og_ui_chip(&self, id: String, text: String, desc: Option<String>) -> String;
    /// `og.ui.badge`
    fn og_ui_badge(&self, id: String, text: String, tone: String);
    /// `og.ui.divider`
    fn og_ui_divider(&self, id: String, vertical: bool);
    /// `og.ui.spacer`
    fn og_ui_spacer(&self, id: String, size: String);
    /// `og.ui.keyValue`
    fn og_ui_key_value(&self, id: String, key: String, value: String, mono: bool);
    /// `og.ui.skeleton`
    fn og_ui_skeleton(&self, id: String, desc: Option<String>);
    /// `og.ui.spinner`
    fn og_ui_spinner(&self, id: String, size: f64);
    /// `og.ui.icon`
    fn og_ui_icon(&self, id: String, name: String, desc: Option<String>);
    /// `og.ui.meter`
    fn og_ui_meter(&self, id: String, value: f64, max: f64, desc: Option<String>);
    /// `og.ui.cooldown`
    fn og_ui_cooldown(&self, id: String, remaining: f64, desc: Option<String>) -> String;
    /// `og.ui.minimap`
    fn og_ui_minimap(&self, id: String, desc: Option<String>);
    /// `og.ui.dialogue`
    fn og_ui_dialogue(&self, id: String, desc: Option<String>) -> String;
    /// `og.ui.joystick`
    fn og_ui_joystick(&self, id: String, desc: Option<String>) -> String;
    /// `og.ui.dpad`
    fn og_ui_dpad(&self, id: String, desc: Option<String>) -> String;
    /// `og.ui.actionButton`
    fn og_ui_action_button(&self, id: String, text: String, desc: Option<String>) -> String;
    /// `og.ui.toast`
    fn og_ui_toast(&self, world: u32, text: String, desc: Option<String>) -> u32;
    /// `og.ui.dismissToast`
    fn og_ui_dismiss_toast(&self, world: u32, toast_id: u32);
    /// `og.ui.navigate`
    fn og_ui_navigate(&self, world: u32, intent: String);
    /// `og.ui.back`
    fn og_ui_back(&self) -> bool;
    /// `og.ui.setPlayers`
    fn og_ui_set_players(&self, count: u32);
    /// `og.ui.focus`
    fn og_ui_focus(&self, pad: u32) -> String;
    /// `og.ui.a11y`
    fn og_ui_a11y(&self) -> String;
    /// `og.ui.feedback`
    fn og_ui_feedback(&self, event: String, rise: f64);
    /// `og.ui.setHaptics`
    fn og_ui_set_haptics(&self, intensity: f64);
    /// `og.ui.setNavigation`
    fn og_ui_set_navigation(&self, on: bool);
    /// `og.ui.memory`
    fn og_ui_memory(&self, id: String, initial: f64) -> f64;
    /// `og.ui.remember`
    fn og_ui_remember(&self, id: String, value: f64);
    /// `og.ui.tick`
    fn og_ui_tick(&self) -> u32;
    /// `og.ui.counter`
    fn og_ui_counter(&self, id: String, value: f64, desc: Option<String>) -> f64;
    /// `og.ui.burst`
    fn og_ui_burst(&self, desc: Option<String>);
    /// `og.ui.shake`
    fn og_ui_shake(&self, desc: Option<String>);
    /// `og.ui.screenTransition`
    fn og_ui_screen_transition(&self, id: String, screen: String, desc: Option<String>) -> String;
    /// `og.ui.previewViewport`
    fn og_ui_preview_viewport(&self, world: u32, viewport: String);
    /// `og.ui.props`
    fn og_ui_props(&self, desc: Option<String>) -> bool;
    /// `og.ui.defineTimeline`
    fn og_ui_define_timeline(&self, name: String, timeline: Option<String>);
    /// `og.ui.play`
    fn og_ui_play(&self, path: String, choreo: String, desc: Option<String>) -> u32;
    /// `og.ui.playTimeline`
    fn og_ui_play_timeline(&self, path: String, timeline: String, desc: Option<String>) -> u32;
    /// `og.ui.seek`
    fn og_ui_seek(&self, play: u32, ms: f64) -> bool;
    /// `og.ui.pause`
    fn og_ui_pause(&self, play: u32) -> bool;
    /// `og.ui.resume`
    fn og_ui_resume(&self, play: u32) -> bool;
    /// `og.ui.reverse`
    fn og_ui_reverse(&self, play: u32) -> bool;
    /// `og.ui.stop`
    fn og_ui_stop(&self, play: u32) -> bool;
    /// `og.ui.playInfo`
    fn og_ui_play_info(&self, play: u32) -> String;
    /// `og.ui.forceState`
    fn og_ui_force_state(&self, path: String, group: String, state: String);
    /// `og.ui.openPath`
    fn og_ui_open_path(&self) -> String;
    /// `og.ui.visualState`
    fn og_ui_visual_state(&self, path: String, group: String) -> String;
    /// `og.ui.animate`
    fn og_ui_animate(&self, id: String, target: f64, transition: String) -> f64;
    /// `og.ui.gesture`
    fn og_ui_gesture(&self, path: String) -> String;
    /// `og.ui.motionEvents`
    fn og_ui_motion_events(&self) -> String;
    /// `og.ui.overlaps`
    fn og_ui_overlaps(&self) -> String;
    /// `og.ui.layoutStats`
    fn og_ui_layout_stats(&self) -> String;
    /// `og.ui.semantics`
    fn og_ui_semantics(&self, desc: Option<String>) -> bool;
    /// `og.ui.setDraw`
    fn og_ui_set_draw(&self, desc: Option<String>);
    /// `og.ui.openWorldAnchor`
    fn og_ui_open_world_anchor(&self, id: String, desc: Option<String>) -> String;
    /// `og.ui.openWorldPanel`
    fn og_ui_open_world_panel(&self, id: String, desc: Option<String>) -> String;
    /// `og.ui.defineMass`
    fn og_ui_define_mass(&self, name: String, desc: Option<String>) -> u32;
    /// `og.ui.spawnMass`
    fn og_ui_spawn_mass(&self, template: u32, instances: String);
    /// `og.ui.pushMass`
    fn og_ui_push_mass(&self, template: u32, instances: String);
    /// `og.ui.massCount`
    fn og_ui_mass_count(&self, template: u32) -> u32;
    /// `og.ui.particles`
    fn og_ui_particles(&self, desc: Option<String>);
    /// `og.ui.meta`
    fn og_ui_meta(&self, id: String, desc: Option<String>);
    /// `og.ui.sceneViewport`
    fn og_ui_scene_viewport(&self, id: String, desc: Option<String>) -> String;
    /// `og.ui.screen`
    fn og_ui_screen(&self, name: String, vm: Option<String>) -> String;
    /// `og.ui.component`
    fn og_ui_component(&self, id: String, name: String, props: Option<String>) -> String;
    /// `og.ui.openSlot`
    fn og_ui_open_slot(&self, path: String) -> bool;
    /// `og.ui.push`
    fn og_ui_push(&self, world: u32, screen: String, vm: Option<String>) -> String;
    /// `og.ui.pop`
    fn og_ui_pop(&self, world: u32) -> String;
    /// `og.ui.replace`
    fn og_ui_replace(&self, world: u32, screen: String, vm: Option<String>) -> String;
    /// `og.ui.clear`
    fn og_ui_clear(&self, world: u32);
    /// `og.ui.layers`
    fn og_ui_layers(&self, world: u32) -> String;
    /// `og.ui.screenVm`
    fn og_ui_screen_vm(&self, world: u32, screen: String) -> String;
    /// `og.ui.docEvents`
    fn og_ui_doc_events(&self, world: u32) -> String;
    /// `og.ui.gameInput`
    fn og_ui_game_input(&self, world: u32) -> bool;
    /// `og.ui.paused`
    fn og_ui_paused(&self, world: u32) -> bool;
    /// `og.ui.defineAction`
    fn og_ui_define_action(&self, world: u32, desc: String) -> String;
    /// `og.ui.action`
    fn og_ui_action(&self, name: String) -> bool;
    /// `og.ui.actionHeld`
    fn og_ui_action_held(&self, name: String) -> bool;
    /// `og.ui.glyph`
    fn og_ui_glyph(&self, world: u32, action: String) -> String;
    /// `og.ui.frameGlyph`
    fn og_ui_frame_glyph(&self, action: String) -> String;
    /// `og.ui.device`
    fn og_ui_device(&self, world: u32) -> String;
    /// `og.ui.setDevice`
    fn og_ui_set_device(&self, world: u32, device: String);
    /// `og.ui.actionBar`
    fn og_ui_action_bar(&self, world: u32) -> String;
    /// `og.ui.documentSchema`
    fn og_ui_document_schema(&self) -> String;
    /// `og.ui.setProp`
    fn og_ui_set_prop(&self, world: u32, path: String, prop: String, value: String) -> String;
    /// `og.ui.saveEdits`
    fn og_ui_save_edits(&self, world: u32) -> String;
}

/// `commands`: packages/knowledge/wiki/commands.md.
pub trait CommandsApi: Wire {
    /// `og.commands.register`
    fn og_commands_register(&self, name: String, handler: u32, desc: Option<String>) -> String;
    /// `og.commands.parse`
    fn og_commands_parse(&self, script: String) -> String;
    /// `og.commands.help`
    fn og_commands_help(&self, world: u32) -> String;
    /// `og.commands.schedule`
    fn og_commands_schedule(&self, world: u32, script: String) -> String;
}

/// `probe`: packages/knowledge/wiki/probe.md.
pub trait ProbeApi: Wire {
    /// `RuntimeProbe.version`
    fn og_runtime_probe_get_version(&self, runtime_probe: u32) -> u32;
    /// `RuntimeProbe.tick`
    fn og_runtime_probe_tick(&self, runtime_probe: u32) -> u32;
    /// `RuntimeProbe.state`
    fn og_runtime_probe_state(&self, runtime_probe: u32) -> String;
    /// `RuntimeProbe.advance`
    fn og_runtime_probe_advance(&self, runtime_probe: u32, steps: u32) -> u32;
    /// `RuntimeProbe.pause`
    fn og_runtime_probe_pause(&self, runtime_probe: u32) -> bool;
    /// `RuntimeProbe.resume`
    fn og_runtime_probe_resume(&self, runtime_probe: u32) -> bool;
    /// `RuntimeProbe.input`
    fn og_runtime_probe_input(&self, runtime_probe: u32, event: String) -> bool;
    /// `RuntimeProbe.accepted`
    fn og_runtime_probe_accepted(&self, runtime_probe: u32) -> u32;
    /// `RuntimeProbe.pick`
    fn og_runtime_probe_pick(&self, runtime_probe: u32, x: f64, y: f64) -> String;
    /// `RuntimeProbe.entity`
    fn og_runtime_probe_entity(&self, runtime_probe: u32, id: String) -> String;
    /// `RuntimeProbe.bounds`
    fn og_runtime_probe_bounds(&self, runtime_probe: u32, id: String) -> String;
    /// `RuntimeProbe.patch`
    fn og_runtime_probe_patch(&self, runtime_probe: u32, id: String, parameter: String, value: String) -> bool;
    /// `RuntimeProbe.run`
    fn og_runtime_probe_run(&self, runtime_probe: u32, program: String) -> String;
    /// `EngineProbe.hello`
    fn og_engine_probe_hello(&self, engine_probe: u32) -> String;
    /// `EngineProbe.capture`
    fn og_engine_probe_capture(&self, engine_probe: u32, width: Option<u32>, height: Option<u32>) -> String;
    /// `EngineProbe.sweep`
    fn og_engine_probe_sweep(&self, engine_probe: u32, desc: Option<String>) -> String;
    /// `EngineProbe.reload`
    fn og_engine_probe_reload(&self, engine_probe: u32) -> String;
    /// `EngineProbe.snapshot`
    fn og_engine_probe_snapshot(&self, engine_probe: u32) -> String;
    /// `EngineProbe.restore`
    fn og_engine_probe_restore(&self, engine_probe: u32, id: String) -> bool;
    /// `EngineProbe.dump`
    fn og_engine_probe_dump(&self, engine_probe: u32, desc: Option<String>) -> String;
    /// `EngineProbe.commands`
    fn og_engine_probe_commands(&self, engine_probe: u32, script: String) -> String;
    /// `EngineProbe.listen`
    fn og_engine_probe_listen(&self, engine_probe: u32, seconds: Option<f64>) -> String;
    /// `EngineProbe.memory`
    fn og_engine_probe_memory(&self, engine_probe: u32) -> String;
    /// `EngineProbe.describe`
    fn og_engine_probe_describe(&self, engine_probe: u32) -> String;
    /// `EngineProbe.expect`
    fn og_engine_probe_expect(&self, engine_probe: u32, what: String) -> String;
    /// `EngineProbe.expectations`
    fn og_engine_probe_expectations(&self, engine_probe: u32) -> String;
    /// `EngineProbe.saveScene`
    fn og_engine_probe_save_scene(&self, engine_probe: u32, write: Option<bool>, spawned: Option<bool>) -> String;
}

/// `terrain`: packages/knowledge/wiki/terrain.md.
pub trait TerrainApi: Wire {
    /// `og.terrain.load`
    fn og_terrain_load(&self, world: u32, path: String) -> String;
    /// `og.terrain.sample`
    fn og_terrain_sample(&self, world: u32, desc: Option<String>) -> String;
    /// `og.terrain.bounds`
    fn og_terrain_bounds(&self, world: u32, data: f64, origin: Option<String>, quaternion: Option<String>, scale: Option<String>) -> String;
    /// `og.terrain.edit`
    fn og_terrain_edit(&self, world: u32, data: f64, brush: Option<String>) -> String;
    /// `og.terrain.layer`
    fn og_terrain_layer(&self, world: u32, data: f64, index: u32) -> String;
    /// `og.terrain.setLayer`
    fn og_terrain_set_layer(&self, world: u32, data: f64, index: u32, layer: Option<String>) -> String;
}

/// `stream`: packages/knowledge/wiki/stream.md.
pub trait StreamApi: Wire {
    /// `og.stream.status`
    fn og_stream_status(&self, world: u32, group: String) -> String;
    /// `og.stream.request`
    fn og_stream_request(&self, world: u32, group: String) -> String;
    /// `og.stream.release`
    fn og_stream_release(&self, world: u32, group: String) -> String;
    /// `og.stream.budget`
    fn og_stream_budget(&self, world: u32) -> String;
    /// `og.stream.setBudget`
    fn og_stream_set_budget(&self, world: u32, bytes: f64) -> String;
}

/// `debug`: packages/knowledge/wiki/debug.md.
pub trait DebugApi: Wire {
    /// `og.debug.line`
    fn og_debug_line(&self, world: u32, from: String, to: String, desc: Option<String>);
    /// `og.debug.box`
    fn og_debug_box(&self, world: u32, center: String, size: String, desc: Option<String>);
    /// `og.debug.sphere`
    fn og_debug_sphere(&self, world: u32, center: String, radius: f64, desc: Option<String>);
    /// `og.debug.arrow`
    fn og_debug_arrow(&self, world: u32, from: String, to: String, desc: Option<String>);
    /// `og.debug.axes`
    fn og_debug_axes(&self, world: u32, position: String, size: f64, desc: Option<String>);
    /// `og.debug.text`
    fn og_debug_text(&self, world: u32, position: String, text: String, desc: Option<String>);
    /// `og.debug.clear`
    fn og_debug_clear(&self, world: u32);
    /// `og.debug.setView`
    fn og_debug_set_view(&self, world: u32, view: String);
    /// `og.debug.view`
    fn og_debug_view(&self, world: u32) -> String;
    /// `og.debug.setEnabled`
    fn og_debug_set_enabled(&self, world: u32, enabled: bool);
    /// `og.debug.enabled`
    fn og_debug_enabled(&self, world: u32) -> bool;
}

/// `path`: packages/knowledge/wiki/path.md.
pub trait PathApi: Wire {
    /// `og.path.create`
    fn og_path_create(&self, world: u32, desc: Option<String>) -> String;
    /// `og.path.find`
    fn og_path_find(&self, world: u32, name: String) -> String;
    /// `og.path.sample`
    fn og_path_sample(&self, world: u32, path: f64, t: f64) -> String;
    /// `og.path.length`
    fn og_path_length(&self, world: u32, path: f64) -> String;
    /// `og.path.nearest`
    fn og_path_nearest(&self, world: u32, path: f64, point: String) -> String;
}

/// `timeline`: packages/knowledge/wiki/timeline.md.
pub trait TimelineApi: Wire {
    /// `og.timeline.info`
    fn og_timeline_info(&self, world: u32, asset: f64) -> String;
    /// `og.timeline.play`
    fn og_timeline_play(&self, world: u32, entity: f64) -> String;
    /// `og.timeline.pause`
    fn og_timeline_pause(&self, world: u32, entity: f64) -> String;
    /// `og.timeline.seek`
    fn og_timeline_seek(&self, world: u32, entity: f64, time: f64) -> String;
    /// `og.timeline.stop`
    fn og_timeline_stop(&self, world: u32, entity: f64) -> String;
}

/// `net`: packages/knowledge/wiki/net.md.
pub trait NetApi: Wire {
    /// `og.net.submit`
    fn og_net_submit(&self, world: u32, input: Option<String>) -> String;
    /// `og.net.status`
    fn og_net_status(&self, world: u32) -> String;
    /// `og.net.hash`
    fn og_net_hash(&self, world: u32) -> String;
}

/// Every trait: a host is a type that implements all of them.
pub trait Host: Lifecycle + ReflectApi + WorldApi + SystemApi + GameApi + TransformApi + TimeApi + RngApi + InputApi + CameraApi + RenderApi + LoadApi + PhysicsApi + AtmosphereApi + AnimApi + TweenApi + TilemapApi + AudioApi + NavApi + AiApi + PlatformApi + I18nApi + UiApi + CommandsApi + ProbeApi + TerrainApi + StreamApi + DebugApi + PathApi + TimelineApi + NetApi {}

impl<T: Lifecycle + ReflectApi + WorldApi + SystemApi + GameApi + TransformApi + TimeApi + RngApi + InputApi + CameraApi + RenderApi + LoadApi + PhysicsApi + AtmosphereApi + AnimApi + TweenApi + TilemapApi + AudioApi + NavApi + AiApi + PlatformApi + I18nApi + UiApi + CommandsApi + ProbeApi + TerrainApi + StreamApi + DebugApi + PathApi + TimelineApi + NetApi> Host for T {}

/// Every binding symbol, in binding-table order (`bindings.json` ids).
pub const SYMBOLS: &[&str] = &[
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
    "og_physics_shape_cast",
    "og_physics_overlap",
    "og_physics_apply_force",
    "og_physics_apply_impulse",
    "og_physics_apply_torque",
    "og_physics_apply_torque_impulse",
    "og_physics_set_mesh",
    "og_physics_rope_points",
    "og_physics_cloth_points",
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
    "og_nav_nearest",
    "og_nav_path_of",
    "og_ai_load",
    "og_ai_parse",
    "og_ai_get",
    "og_ai_set",
    "og_ai_restart",
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
    "og_platform_save_slot",
    "og_platform_load_slot",
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
    "og_net_hash",
];

pub use og_records::{COLUMNS, ColumnDecl, FieldDecl, RECORDS, RecordDecl};
