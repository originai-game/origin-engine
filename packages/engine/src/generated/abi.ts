// Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate.
// The host ABI: every binding symbol with its wire signature. Both hosts
// export exactly these functions; `A` is how a host hands out typed arrays
// and `V` how it hands out a component's column views (the shared buffer
// is `unknown`: each loader knows its host's form).

/// The engine's call back into the runtime (`og_host_set_dispatcher`).
export type Dispatcher = (kind: number, engine: number, id: number, query: number, tick: number, dt: number, elapsed: number, payload: string) => string;

/** A system call (`Dispatch.kind`): the shared buffer opens with the time scale's four doubles, then the system's archetypes from word `FRAME_QUERY_WORD`. */
export const DISPATCH_SYSTEM = 0;
/** A command handler's call (`Dispatch.kind`): the command's arguments are the payload's JSON list. */
export const DISPATCH_COMMAND = 1;
/** The facts function's call (`Dispatch.kind`): its answer is `=` then the facts' JSON. */
export const DISPATCH_FACTS = 2;
/** A system's run condition (`Dispatch.kind`): the answer is `=true` or `=false`. */
export const DISPATCH_CONDITION = 3;
/** A hook's call (`Dispatch.kind`): the payload is the hooked entity. */
export const DISPATCH_HOOK = 4;
/** The platform pump's call (`Dispatch.kind`): the host drains what it handed the engine. */
export const DISPATCH_PLATFORM = 5;
/** A named UI registration pass (`Dispatch.kind`): the world is read only, and a changed world hash refuses it. */
export const DISPATCH_UI = 6;
/** The shared buffer's size in bytes: what a host installs, and what both sides view. */
export const SHARED_BYTES = 65536;
/** Where a system call's archetypes start in the shared buffer, in words: after the time scale's four doubles. */
export const FRAME_QUERY_WORD = 8;

export interface HostAbi<A, V> {
  /** Install the function the engine calls for systems, commands and facts: `dispatcher(kind, engine, id, query, tick, dt, elapsed, payload)` (payload: a command's arguments as a JSON list) returns `""` on success, the error otherwise (`kind`: 0 system, 1 command, 2 facts, whose answer is `"="` then JSON). A system's call has an empty payload: its time scale and its query's archetypes are in the shared buffer (og_host_shared) when the dispatcher is entered. */
  og_host_set_dispatcher(dispatcher: Dispatcher): void;
  /** The shared buffer: 65536 bytes the engine and the runtime both see, where what crosses on every system call is written as numbers instead of text. Valid until the next engine call. A system's call: four f64 (the time scale, the hit stop's ticks left, the scaled dt and the scaled elapsed time) at bytes 0 to 31, then its query's archetypes from word 8 as og_engine_query_archetypes writes them. */
  og_host_shared(): unknown;
  /** Assemble a world from a declared game (`og.game`) and the declarations made so far. `options` is JSON `{ seed?, root? }`: a replay seed, and the project directory (native) or URL base (web) whose `cooked/` bundle is opened and whose GameDesc.scene is spawned before the first tick. The answer is a Result of the engine id, which is also the world handle. */
  og_engine_create(game: number, options: string): string;
  /** Drop an engine and everything it owns. False for an unknown engine. */
  og_engine_destroy(engine: number): boolean;
  /** Take fixed steps regardless of the wall clock: a Result of the tick reached, or the first failing system with its tick. */
  og_engine_advance(engine: number, steps: number): string;
  /** Run a registered command action now, between ticks (`args` is a JSON list of strings). The scheduled channel is `og.commands.schedule`. */
  og_engine_command(engine: number, action: string, args: string): string;
  /** One display frame: feed `elapsed_ms` of wall clock to the fixed-tick accumulator (unless paused), run the command steps due, take the ticks it allows up to `GameDesc.maxCatchUpTicks` through the probe, then render when a renderer is attached. JSON `{ ok, ticks, tick, rendered, paused, quit }` or a refusal; `quit` is true once a `quit` command ran. */
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
  /** A component's number in this world (its schema id index): what og_engine_query_archetypes lists as an archetype's types and og_engine_view_column takes. 4294967295 for a name this world has not registered as a component. It holds until the layout epoch moves. */
  og_engine_component_id(world: number, component: string): number;
  /** A component's packed layout, as og_engine_resource_layout answers a resource's, with its number (`id`, og_engine_component_id's) and its fields in the order world.get answers them (by name). A Result; refused for a name this world has not registered as a component. */
  og_engine_component_layout(world: number, component: string): string;
  /** world.get without text: write the packed bytes of a component (og_engine_component_id's number) of an entity to the start of the shared buffer, and answer the layout epoch they were read under. 0 when refused (no such entity or component, undeclared in the running system, a column a view holds): the runtime then asks og_world_get for the reason. */
  og_engine_read_component(world: number, entity: number, component: number): number;
  /** world.set without text: overwrite fields of a component the entity has from the shared buffer. The packed record is at the buffer's start; from the next word boundary after it come a count and that many (offset, size) word pairs, the byte ranges to take (the fields the value named); the rest keeps its value. `epoch` is the layout epoch the bytes were made with. The answer is the layout epoch now, and the write happened only when the two are equal. 0 when refused (as og_engine_read_component, or an invalid value): the runtime then calls og_world_set, which says why. */
  og_engine_write_component(world: number, entity: number, component: number, epoch: number): number;
  /** Write a query's archetypes, from its `from`th, to the shared buffer as u32 words from word 0, and answer how many were written: the query's archetype count, how many follow, their tick and the layout epoch; then per archetype its view handle, its entity count, its type count and its types (og_engine_component_id's numbers). What did not fit is asked for with a later `from`. `from` 0 makes a world query's views (Query.archetypes outside systems). */
  og_engine_query_archetypes(query: number, from: number): number;
  /** ArchetypeView.column: the columns of a component (og_engine_component_id's number) in a view, one per field in field order. A running system's view borrows them when first asked and holds them until the system returns. Empty when refused (a stale view, a component the system did not declare or the archetype lacks, a column borrowed otherwise). */
  og_engine_view_column(view: number, component: number): V;
  /** ArchetypeView.addScaled: target += source * scale over the view's rows (every row, or those its change filters pass), in place in the columns og_engine_view_column hands out, the fields given by their place in that call's answer. Each f32 element is computed in f64 and rounded once. A Result; refused, with nothing changed, as og_engine_view_column is, for a read-only target or for columns of different sizes. */
  og_engine_view_add_scaled(view: number, target: number, target_field: number, source: number, source_field: number, scale: number): string;
  /** Apply a binary UI source/computation patch batch. Replies are binary results in request order. Source values and the dependency graph belong to this engine; the TypeScript runtime executes requested computation IDs outside the engine lock. */
  og_engine_ui_patch(world: number, bytes: Uint8Array): Uint8Array;
  /** The development inspector's panel since the last call, for the TSX panel the runtime mounts: its values as JSON while it is open and they changed, `null` once when it closed, empty when nothing changed (always empty in a publish build). */
  og_engine_inspector_panel(engine: number): string;
  /** One event of the inspector's TSX panel, JSON `{ kind, name, row, value }`, done at the next frame: `click` and `changed` (a control's prop) as the panel names them, `input` and `commit` of a field part's text, `cancel`, `focus` and `blur` of a text box, `panel` (the panel's node). False for a malformed event or a publish build. */
  og_engine_inspector_event(engine: number, event: string): boolean;
  /** The platform calls (IDL `platform`) made since the last call, oldest first, as JSON `[{ request, method, args, tick }]`. The host answers each but `telemetry` (one-way) with og_platform_answer. */
  og_platform_take(engine: number): string;
  /** Answer one platform call: JSON `{ ok: true, data }` or `{ ok: false, error: { code, message, retryable } }`. It is applied at the start of the next tick (og.platform sends PlatformAnswered). False for an unknown, one-way or answered request, or a malformed answer. */
  og_platform_answer(engine: number, request: number, answer: string): boolean;
  /** Say what answers this engine's platform calls: JSON `{ host, capabilities }` (`player`, `local` or `none`; the methods it answers), as og.platform.info reports. */
  og_platform_set_host(engine: number, info: string): void;
  /** Unasked host input for og.platform (docs/platform-asks.md #19): JSON `{ kind, data, time? }`, kind `purchase` (a purchase settled: `{ purchase, product, quantity, state }`), `entitlements` (`{ entitlements }`, changed without a call) or `invite` (`{ invite, from, payload, time }`). Applied at the start of the next tick (og.PlatformNotice) and logged with the answers. False for a malformed notice or an unknown kind. */
  og_platform_notify(engine: number, notice: string): boolean;
  /** The room ops (IDL `platform` rooms, og.net's lockstep) made since the last call, oldest first, as JSON `[op]`: `{ op: "create", name, capacity }`, `{ op: "join", room, name }`, `{ op: "leave" }`, `{ op: "send", to?, channel?, data? | bytes? }` (bytes base64; a channel starting `og.` is the engine's own). The host carries each in order (docs/design.md §2.8). */
  og_room_take(engine: number): string;
  /** One room event from the host, JSON: `{ type: "joined", room, peer, peers: [{ peer, name }] }`, `{ type: "peer_joined", peer, name }`, `{ type: "peer_left", peer }`, `{ type: "message", from, channel?, data? | bytes? }`, `{ type: "left", reason }` or `{ type: "error", op, code, message }`. Lockstep's messages act between ticks at once; the game's events are sent at the start of the next tick. False for a malformed event. */
  og_room_deliver(engine: number, event: string): boolean;
  /** Say what carries this engine's rooms: JSON `{ host, join? }` (`relay`, `player` or `none`), as og.platform.roomInfo reports; with `none` roomCreate and roomJoin are refused. `join` is the room a Play link named (`?room=<id>`): the engine joins it once, itself, and og.RoomJoined answers. */
  og_room_set_host(engine: number, info: string): void;
  /** The shared-world ops (IDL `shared`) made since the last call, oldest first, as JSON `[op]`: `{ op: "connect", world, reconnect }` (ask the session provider for a connection and open it; `reconnect` after a lost one), `{ op: "send", bytes }` (one binary message, base64, on the open connection) or `{ op: "close" }`. The host carries each in order (docs/design.md §2.12). */
  og_shared_take(engine: number): string;
  /** One shared-world event from the host, JSON: `{ type: "host", host }` (what carries sessions here: the provider's name, or `none`), `{ type: "open", transport, token }` (connected; the token is the provider's, sent in hello as it is), `{ type: "message", bytes }` (base64), `{ type: "closed", reason }` or `{ type: "refused", code, message }` (the provider refused). Applied at the start of the next tick and journaled (og.platform.journal) without the token. False for a malformed event. */
  og_shared_deliver(engine: number, event: string): boolean;
  /** Name the content base og.load.data reads catalogs from (guide loading §14), for an engine created with `contentLater`: a page that learns it from its Player's hello. Empty for none: waiting data loads then fail `unavailable`. Until it is named they stay pending (30 s at most on the web). */
  og_engine_set_content(engine: number, base: string): void;
  /** Turn an engine's trace on or off (`og run --trace`, the shell's `trace()`). Engine 0xffffffff is the loads made before an engine exists: its boot scene's. */
  og_engine_trace(engine: number, on: boolean): void;
  /** The trace's spans since the last take, as JSON `{ now, events }`: Chrome trace events on the engine's clock. `last` waits for the GPU's frames in flight where the host can (natively; the web's arrive in a later take). */
  og_engine_trace_take(engine: number, last: boolean): string;
  /** Turn an engine's performance record on (a fresh one) or off (`og run --perf`, the shell's `perf()`, guide perf). */
  og_engine_perf(engine: number, on: boolean): void;
  /** The performance record's report so far, as JSON `{ json, text }`: each line's mean, median, p95, p99 and max, flagged against `budgets` (a JSON object of line to milliseconds), and the hitches with their causes. The record is the engine's only one of frame times: `frame interval` (a presenting host's, og_present_frame's) and `frame work` are its lines. */
  og_engine_perf_report(engine: number, budgets: string): string;
  /** The performance record's frames since the last call, binned (field telemetry's `frames`): JSON `{ count, ms, bins, interval, work, hitches, longHitches, max }`, `bins` the 11 upper edges in ms (8.4 to 250) and `interval` and `work` 12 counts each, the last above 250 ms. A hitch is a frame whose interval is more than twice the median of the 64 before it, a long one also 250 ms or more. `null` while the record is off. */
  og_engine_perf_frames(engine: number): string;
  /** One display frame of a presenting host at `now_ms`, the host's monotonic clock (requestAnimationFrame's timestamp, the desktop window loop's). The engine keeps the frame cap (og.render.setDisplay, DisplayInfo.maxFps) by drawing only some display frames, measures the display's interval from these timestamps unless og_present_refresh told it, and runs a drawn one as og_engine_frame with the time since the last drawn one, which the performance record takes as its interval. JSON: og_engine_frame's answer with `drawn: true`, or `{ ok: true, drawn: false }`; both with `nextMs`, the host time the next frame to draw is due, which a host without a display callback waits until (winit's ControlFlow::WaitUntil). A host calls it on every display frame, drawn or not, and names its locale at og_engine_create (`locale`). */
  og_present_frame(engine: number, now_ms: number): string;
  /** The clock stopped, held or went on (play paused or resumed, a probe capture's wait): the next og_present_frame draws, and is handed no time from before. */
  og_present_resume(engine: number): void;
  /** The display's refresh rate as the host reads it from the display (the desktop window's monitor), Hz; 0 or less is unknown. A host that does not tell it calls og_present_frame on every display frame (requestAnimationFrame), whose timestamps measure it. */
  og_present_refresh(engine: number, hz: number): void;
  /** The host's window went fullscreen or left it (the page's fullscreenchange, the desktop window): DisplayInfo.fullscreen, and the player's setting follows it, so a player who leaves with Escape finds Fullscreen off. The frame answer's `fullscreen` is what the player asks for, which the host carries out. */
  og_present_fullscreen(engine: number, on: boolean): void;
  /** `defineComponent` */
  og_reflect_define_component(name: string, fields: string): string;
  /** `defineResource` */
  og_reflect_define_resource(name: string, fields: string): string;
  /** `defineEvent` */
  og_reflect_define_event(name: string, fields: string): string;
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
  /** `og.transform.face` */
  og_transform_face(position: string, target: string, up: string | undefined): string;
  /** `og.transform.forward` */
  og_transform_forward(quaternion: string): string;
  /** `og.transform.right` */
  og_transform_right(quaternion: string): string;
  /** `og.transform.up` */
  og_transform_up(quaternion: string): string;
  /** `og.transform.cameraForward` */
  og_transform_camera_forward(quaternion: string): string;
  /** `og.transform.cameraRight` */
  og_transform_camera_right(quaternion: string): string;
  /** `og.transform.heading` */
  og_transform_heading(quaternion: string): number;
  /** `og.transform.fromHeading` */
  og_transform_from_heading(heading: number): string;
  /** `og.transform.face2d` */
  og_transform_face2d(position: string, target: string): string;
  /** `og.transform.forward2d` */
  og_transform_forward2d(quaternion: string): string;
  /** `og.transform.heading2d` */
  og_transform_heading2d(quaternion: string): number;
  /** `og.transform.fromHeading2d` */
  og_transform_from_heading2d(heading: number): string;
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
  /** `og.input.intent` */
  og_input_intent(world: number, event: string, payload: string): string;
  /** `og.input.takeIntents` */
  og_input_take_intents(world: number): string;
  /** `og.input.replayIntent` */
  og_input_replay_intent(world: number, intent: string): string;
  /** `og.input.defineAction` */
  og_input_define_action(world: number, desc: string | undefined): string;
  /** `og.input.actions` */
  og_input_actions(world: number): string;
  /** `og.input.action` */
  og_input_action(world: number, name: string): string;
  /** `og.input.isActionDown` */
  og_input_is_action_down(world: number, name: string): boolean;
  /** `og.input.isActionJustPressed` */
  og_input_is_action_just_pressed(world: number, name: string): boolean;
  /** `og.input.isActionJustReleased` */
  og_input_is_action_just_released(world: number, name: string): boolean;
  /** `og.input.isActionHeld` */
  og_input_is_action_held(world: number, name: string): boolean;
  /** `og.input.actionValue` */
  og_input_action_value(world: number, name: string): number;
  /** `og.input.actionVector` */
  og_input_action_vector(world: number, name: string): string;
  /** `og.input.moveRelative` */
  og_input_move_relative(world: number, name: string, camera: string): string;
  /** `og.input.setActionMaps` */
  og_input_set_action_maps(world: number, action_maps: string): void;
  /** `og.input.actionMaps` */
  og_input_action_maps(world: number): string;
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
  /** `Input.justPressed` */
  og_input_just_pressed(world: number, key: string): boolean;
  /** `Input.justReleased` */
  og_input_just_released(world: number, key: string): boolean;
  /** `Input.axis` */
  og_input_axis(world: number, negative: string, positive: string): number;
  /** `Input.axis2` */
  og_input_axis2(world: number, left: string, right: string, down: string, up: string): string;
  /** `Input.buttonDown` */
  og_input_button_down(world: number, button: string): boolean;
  /** `Input.buttonJustPressed` */
  og_input_button_just_pressed(world: number, button: string): boolean;
  /** `Input.buttonJustReleased` */
  og_input_button_just_released(world: number, button: string): boolean;
  /** `Pointers.down` */
  og_pointers_down(world: number, id: number): boolean;
  /** `Pointers.justPressed` */
  og_pointers_just_pressed(world: number, id: number): boolean;
  /** `Pointers.justReleased` */
  og_pointers_just_released(world: number, id: number): boolean;
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
  /** `Gamepad.justPressed` */
  og_gamepad_just_pressed(world: number, button: string, pad: number | undefined): boolean;
  /** `Gamepad.justReleased` */
  og_gamepad_just_released(world: number, button: string, pad: number | undefined): boolean;
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
  /** `og.render.compile` */
  og_render_compile(world: number): void;
  /** `og.render.reveal` */
  og_render_reveal(world: number, entity: number, seconds: number): string;
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
  /** `og.render.light2dAt` */
  og_render_light2d_at(world: number, point: string, layers: number | undefined): string;
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
  /** `og.load.progress` */
  og_load_progress(world: number, asset: number): string;
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
  /** `og.load.environmentSun` */
  og_load_environment_sun(world: number, map: number): string;
  /** `og.load.emitter` */
  og_load_emitter(world: number, path: string): string;
  /** `og.load.effect` */
  og_load_effect(world: number, path: string): string;
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
  /** `og.load.vfx` */
  og_load_vfx(world: number, path: string): string;
  /** `og.load.rig2d` */
  og_load_rig2d(world: number, path: string): string;
  /** `og.load.imageData` */
  og_load_image_data(world: number, name: string, bytes: Uint8Array, desc: string | undefined): string;
  /** `og.load.gltfData` */
  og_load_gltf_data(world: number, name: string, bytes: Uint8Array, desc: string | undefined): string;
  /** `og.load.environmentData` */
  og_load_environment_data(world: number, name: string, bytes: Uint8Array, desc: string | undefined): string;
  /** `og.load.data` */
  og_load_data(world: number, desc: string | undefined): string;
  /** `og.load.dataJson` */
  og_load_data_json(world: number, asset: number): string;
  /** `og.load.dataBytes` */
  og_load_data_bytes(world: number, asset: number): string;
  /** `og.load.pack` */
  og_load_pack(world: number, entry: string | undefined): string;
  /** `og.load.environmentInfo` */
  og_load_environment_info(world: number, asset: number): string;
  /** `og.load.release` */
  og_load_release(world: number, asset: number): string;
  /** `og.load.imageInfo` */
  og_load_image_info(world: number, asset: number): string;
  /** `og.load.completions` */
  og_load_completions(world: number): string;
  /** `og.load.replayCompletions` */
  og_load_replay_completions(world: number, completions: string): string;
  /** `og.physics.water` */
  og_physics_water(world: number, point: string): string;
  /** `og.physics.waterHeight` */
  og_physics_water_height(world: number, entity: number, x: number, z: number): string;
  /** `og.physics.castRay` */
  og_physics_cast_ray(world: number, desc: string | undefined): string;
  /** `og.physics.castRayAll` */
  og_physics_cast_ray_all(world: number, desc: string | undefined): string;
  /** `og.physics.projectPoint` */
  og_physics_project_point(world: number, desc: string | undefined): string;
  /** `og.physics.castShape` */
  og_physics_cast_shape(world: number, desc: string | undefined): string;
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
  /** `og.physics.garmentPoints` */
  og_physics_garment_points(world: number, entity: number): string;
  /** `og.physics.clothPoints` */
  og_physics_cloth_points(world: number, entity: number): string;
  /** `og.physics.softBodyPoints` */
  og_physics_soft_body_points(world: number, entity: number): string;
  /** `og.physics.fluidParticles` */
  og_physics_fluid_particles(world: number, entity: number): string;
  /** `og.physics.fluid3dParticles` */
  og_physics_fluid3d_particles(world: number, entity: number): string;
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
  /** `og.atmosphere.season` */
  og_atmosphere_season(world: number): string;
  /** `og.anim.getNodeMatrix` */
  og_anim_get_node_matrix(world: number, entity: number, node: number): string;
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
  /** `TweenTimeline.tween` */
  og_tween_timeline_get_tween(tween_timeline: number): number;
  /** `TweenTimeline.error` */
  og_tween_timeline_get_error(tween_timeline: number): string;
  /** `TweenTimeline.to` */
  og_tween_timeline_to(tween_timeline: number, targets: string, vars: string | undefined, position: string | undefined): number;
  /** `TweenTimeline.from` */
  og_tween_timeline_from(tween_timeline: number, targets: string, vars: string | undefined, position: string | undefined): number;
  /** `TweenTimeline.fromTo` */
  og_tween_timeline_from_to(tween_timeline: number, targets: string, vars: string | undefined, position: string | undefined): number;
  /** `TweenTimeline.set` */
  og_tween_timeline_set(tween_timeline: number, targets: string, vars: string | undefined, position: string | undefined): number;
  /** `TweenTimeline.call` */
  og_tween_timeline_call(tween_timeline: number, name: string, position: string | undefined): number;
  /** `TweenTimeline.addLabel` */
  og_tween_timeline_add_label(tween_timeline: number, label: string, position: string | undefined): number;
  /** `TweenTimeline.add` */
  og_tween_timeline_add(tween_timeline: number, child: string, position: string | undefined): number;
  /** `TweenTimeline.play` */
  og_tween_timeline_play(tween_timeline: number, from: string | undefined): number;
  /** `TweenTimeline.pause` */
  og_tween_timeline_pause(tween_timeline: number, at: string | undefined): number;
  /** `TweenTimeline.resume` */
  og_tween_timeline_resume(tween_timeline: number): number;
  /** `TweenTimeline.reverse` */
  og_tween_timeline_reverse(tween_timeline: number, from: string | undefined): number;
  /** `TweenTimeline.restart` */
  og_tween_timeline_restart(tween_timeline: number): number;
  /** `TweenTimeline.seek` */
  og_tween_timeline_seek(tween_timeline: number, position: string): number;
  /** `TweenTimeline.progress` */
  og_tween_timeline_progress(tween_timeline: number, value: number | undefined): number;
  /** `TweenTimeline.time` */
  og_tween_timeline_time(tween_timeline: number): number;
  /** `TweenTimeline.duration` */
  og_tween_timeline_duration(tween_timeline: number): number;
  /** `TweenTimeline.timeScale` */
  og_tween_timeline_time_scale(tween_timeline: number, value: number | undefined): number;
  /** `TweenTimeline.isActive` */
  og_tween_timeline_is_active(tween_timeline: number): boolean;
  /** `TweenTimeline.kill` */
  og_tween_timeline_kill(tween_timeline: number): boolean;
  /** `og.tween.to` */
  og_tween_to(world: number, desc: string | undefined): string;
  /** `og.tween.from` */
  og_tween_from(world: number, desc: string | undefined): string;
  /** `og.tween.fromTo` */
  og_tween_from_to(world: number, desc: string | undefined): string;
  /** `og.tween.sequence` */
  og_tween_sequence(world: number, steps: string, desc: string | undefined): string;
  /** `og.tween.parallel` */
  og_tween_parallel(world: number, tweens: string, desc: string | undefined): string;
  /** `og.tween.timeline` */
  og_tween_timeline(world: number, desc: string | undefined): number;
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
  /** `og.audio.playFeedback` */
  og_audio_play_feedback(world: number, sound: number, desc: string | undefined): string;
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
  /** `og.story.setSaliency` */
  og_story_set_saliency(world: number, strategy: string): string;
  /** `og.story.load` */
  og_story_load(world: number, path: string): string;
  /** `og.story.parse` */
  og_story_parse(world: number, text: string): string;
  /** `og.story.start` */
  og_story_start(world: number, desc: string | undefined): string;
  /** `og.story.view` */
  og_story_view(world: number, conversation: number): string;
  /** `og.story.graph` */
  og_story_graph(world: number, dialogue: string): string;
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
  /** `og.platform.catalogHead` */
  og_platform_catalog_head(world: number, catalog: string): string;
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
  /** `og.platform.journal` */
  og_platform_journal(world: number): string;
  /** `og.platform.now` */
  og_platform_now(world: number): number;
  /** `og.platform.roomCreate` */
  og_platform_room_create(world: number, desc: string | undefined): string;
  /** `og.platform.roomJoin` */
  og_platform_room_join(world: number, room: string): string;
  /** `og.platform.quickPlay` */
  og_platform_quick_play(world: number, desc: string | undefined): string;
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
  /** `og.i18n.formatNumber` */
  og_i18n_format_number(world: number, value: number, format: string | undefined): string;
  /** `og.i18n.formatCompact` */
  og_i18n_format_compact(world: number, value: number, decimals: number | undefined): string;
  /** `og.i18n.formatCurrency` */
  og_i18n_format_currency(world: number, amount: number, currency: string): string;
  /** `og.i18n.formatDate` */
  og_i18n_format_date(world: number, time_ms: number, format: string | undefined): string;
  /** `og.i18n.formatTime` */
  og_i18n_format_time(world: number, time_ms: number, format: string | undefined): string;
  /** `og.i18n.formatDuration` */
  og_i18n_format_duration(world: number, seconds: number, style: string | undefined): string;
  /** `og.ui.configure` */
  og_ui_configure(world: number, desc: string | undefined): string;
  /** `og.ui.viewport` */
  og_ui_viewport(world: number): string;
  /** `og.ui.theme` */
  og_ui_theme(world: number): string;
  /** `og.ui.framePointer` */
  og_ui_frame_pointer(): string;
  /** `og.ui.frameCursor` */
  og_ui_frame_cursor(): string;
  /** `og.ui.frameViewport` */
  og_ui_frame_viewport(): string;
  /** `og.ui.hasIcon` */
  og_ui_has_icon(name: string): boolean;
  /** `og.ui.seconds` */
  og_ui_seconds(): number;
  /** `og.ui.frameDirection` */
  og_ui_frame_direction(): string;
  /** `og.ui.frameTheme` */
  og_ui_frame_theme(): string;
  /** `og.ui.setVirtualCursor` */
  og_ui_set_virtual_cursor(desc: string): void;
  /** `og.ui.virtualCursor` */
  og_ui_virtual_cursor(): string;
  /** `og.ui.feedback` */
  og_ui_feedback(event: string, rise: number): void;
  /** `og.ui.setHaptics` */
  og_ui_set_haptics(intensity: number): void;
  /** `og.ui.tick` */
  og_ui_tick(): number;
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
  og_engine_probe_capture(engine_probe: number, width: number | undefined, height: number | undefined, fit: number | undefined, region: string | undefined): string;
  /** `EngineProbe.captureViews` */
  og_engine_probe_capture_views(engine_probe: number, views: string, width: number | undefined, height: number | undefined, fit: number | undefined): string;
  /** `EngineProbe.visible` */
  og_engine_probe_visible(engine_probe: number, id: string): string;
  /** `EngineProbe.marks` */
  og_engine_probe_marks(engine_probe: number, desc: string | undefined): string;
  /** `EngineProbe.timeline` */
  og_engine_probe_timeline(engine_probe: number): string;
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
  /** `EngineProbe.assertions` */
  og_engine_probe_assertions(engine_probe: number, take: boolean | undefined): string;
  /** `EngineProbe.assertionEvents` */
  og_engine_probe_assertion_events(engine_probe: number): string;
  /** `EngineProbe.observe` */
  og_engine_probe_observe(engine_probe: number, records: string | undefined, types: boolean | undefined): string;
  /** `EngineProbe.saveScene` */
  og_engine_probe_save_scene(engine_probe: number, write: boolean | undefined, spawned: boolean | undefined): string;
  /** `EngineProbe.pickUi` */
  og_engine_probe_pick_ui(engine_probe: number, x: number, y: number): string;
  /** `og.terrain.load` */
  og_terrain_load(world: number, path: string): string;
  /** `og.terrain.sample` */
  og_terrain_sample(world: number, desc: string | undefined): string;
  /** `og.terrain.bounds` */
  og_terrain_bounds(world: number, data: number, origin: string | undefined, quaternion: string | undefined, scale: string | undefined): string;
  /** `og.terrain.edit` */
  og_terrain_edit(world: number, data: number, brush: string | undefined): string;
  /** `og.terrain.stamp` */
  og_terrain_stamp(world: number, desc: string | undefined): string;
  /** `og.terrain.unstamp` */
  og_terrain_unstamp(world: number, data: number, stamp: number): string;
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
  /** `og.assert.record` */
  og_assert_record(world: number, record: string): number;
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
  /** `og.timeline.captions` */
  og_timeline_captions(world: number): string;
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
  /** `og.net.shape` */
  og_net_shape(world: number, desc: string | undefined): string;
  /** `og.net.submitInput` */
  og_net_submit_input(world: number, component: string, value: string): string;
  /** `og.net.command` */
  og_net_command(world: number, component: string, value: string): string;
  /** `og.net.commands` */
  og_net_commands(world: number, component: string): string;
  /** `og.net.status` */
  og_net_status(world: number): string;
  /** `og.net.resimulating` */
  og_net_resimulating(world: number): boolean;
  /** `og.net.finalHash` */
  og_net_final_hash(world: number, tick: number): string;
  /** `og.net.hash` */
  og_net_hash(world: number): string;
  /** `og.shared.shape` */
  og_shared_shape(world: number, desc: string | undefined): string;
  /** `og.shared.join` */
  og_shared_join(world: number, desc: string | undefined): string;
  /** `og.shared.leave` */
  og_shared_leave(world: number): string;
  /** `og.shared.teleport` */
  og_shared_teleport(world: number, position: string): string;
  /** `og.shared.send` */
  og_shared_send(world: number, component: string, value: string): string;
  /** `og.shared.events` */
  og_shared_events(world: number, component: string): string;
  /** `og.shared.put` */
  og_shared_put(world: number, key: string, component: string, value: string, desc: string | undefined): string;
  /** `og.shared.remove` */
  og_shared_remove(world: number, key: string, expect: string): string;
  /** `og.shared.object` */
  og_shared_object(world: number, entity: number): string;
  /** `og.shared.remote` */
  og_shared_remote(world: number, entity: number): string;
  /** `og.shared.status` */
  og_shared_status(world: number): string;
  /** `og.motion.pose` */
  og_motion_pose(world: number, entity: number, desc: string | undefined): string;
  /** `og.motion.place` */
  og_motion_place(world: number, entity: number, desc: string | undefined): string;
  /** `og.motion.face` */
  og_motion_face(world: number, entity: number, desc: string | undefined): string;
  /** `og.motion.walkTo` */
  og_motion_walk_to(world: number, entity: number, desc: string | undefined): string;
  /** `og.motion.jump` */
  og_motion_jump(world: number, entity: number, desc: string | undefined): string;
  /** `og.motion.clear` */
  og_motion_clear(world: number, entity: number): string;
  /** `og.motion.lookAt` */
  og_motion_look_at(world: number, entity: number, desc: string | undefined): string;
  /** `og.motion.react` */
  og_motion_react(world: number, entity: number, desc: string | undefined): string;
  /** `og.motion.status` */
  og_motion_status(world: number, entity: number): string;
  /** `og.motion.capabilities` */
  og_motion_capabilities(world: number): string;
  /** `og.motion.setBackend` */
  og_motion_set_backend(world: number, backend: string): string;
  /** `og.motion.presentation` */
  og_motion_presentation(world: number, entity: number): string;
  /** `og.motion.poses` */
  og_motion_poses(): string;
  /** `og.destruction.damage` */
  og_destruction_damage(world: number, entity: number, desc: string | undefined): string;
  /** `og.destruction.explode` */
  og_destruction_explode(world: number, desc: string | undefined): string;
  /** `og.destruction.fracture` */
  og_destruction_fracture(world: number, entity: number): string;
  /** `og.destruction.info` */
  og_destruction_info(world: number, entity: number): string;
  /** `og.destruction.material` */
  og_destruction_material(world: number, kind: number): string;
  /** `og.destruction.capabilities` */
  og_destruction_capabilities(world: number): string;
  /** `og.destruction.carve2d` */
  og_destruction_carve2d(world: number, entity: number, desc: string | undefined): string;
  /** `og.destruction.fill2d` */
  og_destruction_fill2d(world: number, entity: number, desc: string | undefined): string;
  /** `og.destruction.explode2d` */
  og_destruction_explode2d(world: number, desc: string | undefined): string;
  /** `og.destruction.fracture2d` */
  og_destruction_fracture2d(world: number, entity: number, desc: string | undefined): string;
  /** `og.destruction.cell2d` */
  og_destruction_cell2d(world: number, entity: number, point: string): string;
  /** `og.vfx.spawn` */
  og_vfx_spawn(world: number, desc: string | undefined): string;
  /** `og.vfx.info` */
  og_vfx_info(world: number, asset: number): string;
  /** `og.sight.visible` */
  og_sight_visible(world: number, team: number, point: string): string;
  /** `og.sight.explored` */
  og_sight_explored(world: number, team: number, point: string): string;
  /** `og.sight.reveal` */
  og_sight_reveal(world: number, team: number, point: string, radius: number): string;
  /** `og.sight.forget` */
  og_sight_forget(world: number, team: number): string;
  /** `og.flow.load` */
  og_flow_load(world: number, dialogue: string): string;
  /** `og.flow.start` */
  og_flow_start(world: number, desc: string | undefined): string;
  /** `og.flow.view` */
  og_flow_view(world: number, flow: number): string;
  /** `og.flow.choose` */
  og_flow_choose(world: number, flow: number, index: number): string;
  /** `og.flow.advance` */
  og_flow_advance(world: number, flow: number, outcome: string | undefined): string;
  /** `og.flow.play` */
  og_flow_play(world: number, flow: number): string;
  /** `og.flow.pause` */
  og_flow_pause(world: number, flow: number): string;
  /** `og.flow.skip` */
  og_flow_skip(world: number, flow: number): string;
  /** `og.flow.chart` */
  og_flow_chart(world: number, dialogue: string): string;
  /** `og.flow.stop` */
  og_flow_stop(world: number, flow: number): string;
  /** `og.scene.preload` */
  og_scene_preload(world: number, scene: string): string;
  /** `og.scene.progress` */
  og_scene_progress(world: number, scene: string): string;
  /** `og.scene.release` */
  og_scene_release(world: number, scene: string): string;
  /** `og.scene.change` */
  og_scene_change(world: number, scene: string): string;
  /** `og.scene.current` */
  og_scene_current(world: number): string;
  /** `og.loading.stage` */
  og_loading_stage(world: number, desc: string | undefined): string;
  /** `og.loading.done` */
  og_loading_done(world: number): string;
  /** `og.loading.idle` */
  og_loading_idle(world: number): boolean;
  /** `og.loading.status` */
  og_loading_status(world: number): string;
}

/** Every binding symbol, in binding-table order, then the lifecycle and present members. */
export const SYMBOLS: readonly string[] = [
  "og_reflect_define_component",
  "og_reflect_define_resource",
  "og_reflect_define_event",
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
  "og_query_count",
  "og_archetype_view_get_count",
  "og_archetype_view_get_tick",
  "og_archetype_view_get_entities",
  "og_archetype_view_get_rows",
  "og_archetype_view_entity",
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
  "og_transform_face",
  "og_transform_forward",
  "og_transform_right",
  "og_transform_up",
  "og_transform_camera_forward",
  "og_transform_camera_right",
  "og_transform_heading",
  "og_transform_from_heading",
  "og_transform_face2d",
  "og_transform_forward2d",
  "og_transform_heading2d",
  "og_transform_from_heading2d",
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
  "og_input_intent",
  "og_input_take_intents",
  "og_input_replay_intent",
  "og_input_define_action",
  "og_input_actions",
  "og_input_action",
  "og_input_is_action_down",
  "og_input_is_action_just_pressed",
  "og_input_is_action_just_released",
  "og_input_is_action_held",
  "og_input_action_value",
  "og_input_action_vector",
  "og_input_move_relative",
  "og_input_set_action_maps",
  "og_input_action_maps",
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
  "og_input_just_pressed",
  "og_input_just_released",
  "og_input_axis",
  "og_input_axis2",
  "og_input_button_down",
  "og_input_button_just_pressed",
  "og_input_button_just_released",
  "og_pointers_down",
  "og_pointers_just_pressed",
  "og_pointers_just_released",
  "og_pointers_position",
  "og_pointers_start",
  "og_pointers_count",
  "og_gamepad_connected",
  "og_gamepad_down",
  "og_gamepad_just_pressed",
  "og_gamepad_just_released",
  "og_gamepad_axis",
  "og_camera_look_at",
  "og_camera_world_to_screen",
  "og_camera_screen_to_ray",
  "og_camera_shake",
  "og_camera_punch",
  "og_camera_kick",
  "og_render_snap",
  "og_render_snap_camera",
  "og_render_compile",
  "og_render_reveal",
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
  "og_render_light2d_at",
  "og_render_render_target",
  "og_render_set_post_param",
  "og_render_post_errors",
  "og_render_baked_lighting",
  "og_load_bundle",
  "og_load_bundle_info",
  "og_load_progress",
  "og_load_default_bundle",
  "og_load_gltf",
  "og_load_scene",
  "og_load_prefab",
  "og_load_font",
  "og_load_font_info",
  "og_load_sound",
  "og_load_lut",
  "og_load_environment",
  "og_load_environment_sun",
  "og_load_emitter",
  "og_load_effect",
  "og_load_shader",
  "og_load_post",
  "og_load_vector",
  "og_load_ui_material",
  "og_load_texture",
  "og_load_texture_array",
  "og_load_style",
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
  "og_load_vfx",
  "og_load_rig2d",
  "og_load_image_data",
  "og_load_gltf_data",
  "og_load_environment_data",
  "og_load_data",
  "og_load_data_json",
  "og_load_data_bytes",
  "og_load_pack",
  "og_load_environment_info",
  "og_load_release",
  "og_load_image_info",
  "og_load_completions",
  "og_load_replay_completions",
  "og_physics_water",
  "og_physics_water_height",
  "og_physics_cast_ray",
  "og_physics_cast_ray_all",
  "og_physics_project_point",
  "og_physics_cast_shape",
  "og_physics_overlap",
  "og_physics_apply_force",
  "og_physics_apply_impulse",
  "og_physics_apply_torque",
  "og_physics_apply_torque_impulse",
  "og_physics_set_mesh",
  "og_physics_rope_points",
  "og_physics_garment_points",
  "og_physics_cloth_points",
  "og_physics_soft_body_points",
  "og_physics_fluid_particles",
  "og_physics_fluid3d_particles",
  "og_physics_vehicle_wheels",
  "og_physics_ragdoll_rig_from_skeleton",
  "og_physics_set_ragdoll_rig",
  "og_physics_ragdoll_pose",
  "og_physics_stats",
  "og_atmosphere_time_of_day",
  "og_atmosphere_season",
  "og_anim_get_node_matrix",
  "og_anim_graph",
  "og_anim_humanoid",
  "og_anim_retarget",
  "og_anim_mirror",
  "og_anim_rig2d_info",
  "og_anim_skeleton2d_boxes",
  "og_tween_timeline_get_tween",
  "og_tween_timeline_get_error",
  "og_tween_timeline_to",
  "og_tween_timeline_from",
  "og_tween_timeline_from_to",
  "og_tween_timeline_set",
  "og_tween_timeline_call",
  "og_tween_timeline_add_label",
  "og_tween_timeline_add",
  "og_tween_timeline_play",
  "og_tween_timeline_pause",
  "og_tween_timeline_resume",
  "og_tween_timeline_reverse",
  "og_tween_timeline_restart",
  "og_tween_timeline_seek",
  "og_tween_timeline_progress",
  "og_tween_timeline_time",
  "og_tween_timeline_duration",
  "og_tween_timeline_time_scale",
  "og_tween_timeline_is_active",
  "og_tween_timeline_kill",
  "og_tween_to",
  "og_tween_from",
  "og_tween_from_to",
  "og_tween_sequence",
  "og_tween_parallel",
  "og_tween_timeline",
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
  "og_audio_play_feedback",
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
  "og_story_set_saliency",
  "og_story_load",
  "og_story_parse",
  "og_story_start",
  "og_story_view",
  "og_story_graph",
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
  "og_platform_catalog_head",
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
  "og_platform_journal",
  "og_platform_now",
  "og_platform_room_create",
  "og_platform_room_join",
  "og_platform_quick_play",
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
  "og_i18n_format_number",
  "og_i18n_format_compact",
  "og_i18n_format_currency",
  "og_i18n_format_date",
  "og_i18n_format_time",
  "og_i18n_format_duration",
  "og_ui_configure",
  "og_ui_viewport",
  "og_ui_theme",
  "og_ui_frame_pointer",
  "og_ui_frame_cursor",
  "og_ui_frame_viewport",
  "og_ui_has_icon",
  "og_ui_seconds",
  "og_ui_frame_direction",
  "og_ui_frame_theme",
  "og_ui_set_virtual_cursor",
  "og_ui_virtual_cursor",
  "og_ui_feedback",
  "og_ui_set_haptics",
  "og_ui_tick",
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
  "og_engine_probe_capture_views",
  "og_engine_probe_visible",
  "og_engine_probe_marks",
  "og_engine_probe_timeline",
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
  "og_engine_probe_assertions",
  "og_engine_probe_assertion_events",
  "og_engine_probe_observe",
  "og_engine_probe_save_scene",
  "og_engine_probe_pick_ui",
  "og_terrain_load",
  "og_terrain_sample",
  "og_terrain_bounds",
  "og_terrain_edit",
  "og_terrain_stamp",
  "og_terrain_unstamp",
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
  "og_assert_record",
  "og_path_create",
  "og_path_find",
  "og_path_sample",
  "og_path_length",
  "og_path_nearest",
  "og_timeline_captions",
  "og_timeline_info",
  "og_timeline_play",
  "og_timeline_pause",
  "og_timeline_seek",
  "og_timeline_stop",
  "og_net_submit",
  "og_net_shape",
  "og_net_submit_input",
  "og_net_command",
  "og_net_commands",
  "og_net_status",
  "og_net_resimulating",
  "og_net_final_hash",
  "og_net_hash",
  "og_shared_shape",
  "og_shared_join",
  "og_shared_leave",
  "og_shared_teleport",
  "og_shared_send",
  "og_shared_events",
  "og_shared_put",
  "og_shared_remove",
  "og_shared_object",
  "og_shared_remote",
  "og_shared_status",
  "og_motion_pose",
  "og_motion_place",
  "og_motion_face",
  "og_motion_walk_to",
  "og_motion_jump",
  "og_motion_clear",
  "og_motion_look_at",
  "og_motion_react",
  "og_motion_status",
  "og_motion_capabilities",
  "og_motion_set_backend",
  "og_motion_presentation",
  "og_motion_poses",
  "og_destruction_damage",
  "og_destruction_explode",
  "og_destruction_fracture",
  "og_destruction_info",
  "og_destruction_material",
  "og_destruction_capabilities",
  "og_destruction_carve2d",
  "og_destruction_fill2d",
  "og_destruction_explode2d",
  "og_destruction_fracture2d",
  "og_destruction_cell2d",
  "og_vfx_spawn",
  "og_vfx_info",
  "og_sight_visible",
  "og_sight_explored",
  "og_sight_reveal",
  "og_sight_forget",
  "og_flow_load",
  "og_flow_start",
  "og_flow_view",
  "og_flow_choose",
  "og_flow_advance",
  "og_flow_play",
  "og_flow_pause",
  "og_flow_skip",
  "og_flow_chart",
  "og_flow_stop",
  "og_scene_preload",
  "og_scene_progress",
  "og_scene_release",
  "og_scene_change",
  "og_scene_current",
  "og_loading_stage",
  "og_loading_done",
  "og_loading_idle",
  "og_loading_status",
  "og_host_set_dispatcher",
  "og_host_shared",
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
  "og_engine_component_id",
  "og_engine_component_layout",
  "og_engine_read_component",
  "og_engine_write_component",
  "og_engine_query_archetypes",
  "og_engine_view_column",
  "og_engine_view_add_scaled",
  "og_engine_ui_patch",
  "og_engine_inspector_panel",
  "og_engine_inspector_event",
  "og_platform_take",
  "og_platform_answer",
  "og_platform_set_host",
  "og_platform_notify",
  "og_room_take",
  "og_room_deliver",
  "og_room_set_host",
  "og_shared_take",
  "og_shared_deliver",
  "og_engine_set_content",
  "og_engine_trace",
  "og_engine_trace_take",
  "og_engine_perf",
  "og_engine_perf_report",
  "og_engine_perf_frames",
  "og_present_frame",
  "og_present_resume",
  "og_present_refresh",
  "og_present_fullscreen",
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
    og_render_reveal(world: number, entity: number, seconds: number) {
      if (typeof entity !== "number") notEntity("og.render.reveal", "entity", entity);
      return abi.og_render_reveal(world, entity, seconds);
    },
    og_physics_water_height(world: number, entity: number, x: number, z: number) {
      if (typeof entity !== "number") notEntity("og.physics.waterHeight", "entity", entity);
      return abi.og_physics_water_height(world, entity, x, z);
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
    og_physics_garment_points(world: number, entity: number) {
      if (typeof entity !== "number") notEntity("og.physics.garmentPoints", "entity", entity);
      return abi.og_physics_garment_points(world, entity);
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
    og_anim_get_node_matrix(world: number, entity: number, node: number) {
      if (typeof entity !== "number") notEntity("og.anim.getNodeMatrix", "entity", entity);
      return abi.og_anim_get_node_matrix(world, entity, node);
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
    og_shared_object(world: number, entity: number) {
      if (typeof entity !== "number") notEntity("og.shared.object", "entity", entity);
      return abi.og_shared_object(world, entity);
    },
    og_shared_remote(world: number, entity: number) {
      if (typeof entity !== "number") notEntity("og.shared.remote", "entity", entity);
      return abi.og_shared_remote(world, entity);
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
    og_motion_jump(world: number, entity: number, desc: string | undefined) {
      if (typeof entity !== "number") notEntity("og.motion.jump", "entity", entity);
      return abi.og_motion_jump(world, entity, desc);
    },
    og_motion_clear(world: number, entity: number) {
      if (typeof entity !== "number") notEntity("og.motion.clear", "entity", entity);
      return abi.og_motion_clear(world, entity);
    },
    og_motion_look_at(world: number, entity: number, desc: string | undefined) {
      if (typeof entity !== "number") notEntity("og.motion.lookAt", "entity", entity);
      return abi.og_motion_look_at(world, entity, desc);
    },
    og_motion_react(world: number, entity: number, desc: string | undefined) {
      if (typeof entity !== "number") notEntity("og.motion.react", "entity", entity);
      return abi.og_motion_react(world, entity, desc);
    },
    og_motion_status(world: number, entity: number) {
      if (typeof entity !== "number") notEntity("og.motion.status", "entity", entity);
      return abi.og_motion_status(world, entity);
    },
    og_motion_presentation(world: number, entity: number) {
      if (typeof entity !== "number") notEntity("og.motion.presentation", "entity", entity);
      return abi.og_motion_presentation(world, entity);
    },
    og_destruction_damage(world: number, entity: number, desc: string | undefined) {
      if (typeof entity !== "number") notEntity("og.destruction.damage", "entity", entity);
      return abi.og_destruction_damage(world, entity, desc);
    },
    og_destruction_fracture(world: number, entity: number) {
      if (typeof entity !== "number") notEntity("og.destruction.fracture", "entity", entity);
      return abi.og_destruction_fracture(world, entity);
    },
    og_destruction_info(world: number, entity: number) {
      if (typeof entity !== "number") notEntity("og.destruction.info", "entity", entity);
      return abi.og_destruction_info(world, entity);
    },
    og_destruction_carve2d(world: number, entity: number, desc: string | undefined) {
      if (typeof entity !== "number") notEntity("og.destruction.carve2d", "entity", entity);
      return abi.og_destruction_carve2d(world, entity, desc);
    },
    og_destruction_fill2d(world: number, entity: number, desc: string | undefined) {
      if (typeof entity !== "number") notEntity("og.destruction.fill2d", "entity", entity);
      return abi.og_destruction_fill2d(world, entity, desc);
    },
    og_destruction_fracture2d(world: number, entity: number, desc: string | undefined) {
      if (typeof entity !== "number") notEntity("og.destruction.fracture2d", "entity", entity);
      return abi.og_destruction_fracture2d(world, entity, desc);
    },
    og_destruction_cell2d(world: number, entity: number, point: string) {
      if (typeof entity !== "number") notEntity("og.destruction.cell2d", "entity", entity);
      return abi.og_destruction_cell2d(world, entity, point);
    },
    og_flow_view(world: number, flow: number) {
      if (typeof flow !== "number") notEntity("og.flow.view", "flow", flow);
      return abi.og_flow_view(world, flow);
    },
    og_flow_choose(world: number, flow: number, index: number) {
      if (typeof flow !== "number") notEntity("og.flow.choose", "flow", flow);
      return abi.og_flow_choose(world, flow, index);
    },
    og_flow_advance(world: number, flow: number, outcome: string | undefined) {
      if (typeof flow !== "number") notEntity("og.flow.advance", "flow", flow);
      return abi.og_flow_advance(world, flow, outcome);
    },
    og_flow_play(world: number, flow: number) {
      if (typeof flow !== "number") notEntity("og.flow.play", "flow", flow);
      return abi.og_flow_play(world, flow);
    },
    og_flow_pause(world: number, flow: number) {
      if (typeof flow !== "number") notEntity("og.flow.pause", "flow", flow);
      return abi.og_flow_pause(world, flow);
    },
    og_flow_skip(world: number, flow: number) {
      if (typeof flow !== "number") notEntity("og.flow.skip", "flow", flow);
      return abi.og_flow_skip(world, flow);
    },
    og_flow_stop(world: number, flow: number) {
      if (typeof flow !== "number") notEntity("og.flow.stop", "flow", flow);
      return abi.og_flow_stop(world, flow);
    },
  };
}
