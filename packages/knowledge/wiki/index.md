<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The Origin Engine API

`@origingame/engine`, API version 1: 14 modules, generated from `crates/og-api/api.ogt`. Each module has one page; every signature on it is quoted from the IDL.

The Origin Engine API. Game code declares components and systems, and reads the world through typed views over engine memory.

Every failure is a returned Result; desc objects may omit any field; nothing reads the wall clock inside a tick.

## Modules

| Module | TypeScript | Publish builds | Summary |
|---|---|---|---|
| [`math`](math.md) | types and tokens | kept | Plain math values in the three.js vocabulary. They are data: the engine owns the operations, so results are identical on every host. |
| [`reflect`](reflect.md) | named exports | kept | Reflection: components and resources are declared once with their field layout and metadata, and registered by name. |
| [`world`](world.md) | types and tokens | kept | The world: an archetype ECS. Entities are generational handles; components are columns in engine memory. |
| [`system`](system.md) | named exports | kept | Systems: functions the engine calls once per tick with their matched archetypes. |
| [`game`](game.md) | `og.*` | kept | The game entry: what the bundle's default export declares. |
| [`transform`](transform.md) | `og.transform.*` | kept | Transforms in the three.js vocabulary: position, quaternion and scale, composed into matrixWorld down the parent chain. |
| [`time`](time.md) | types and tokens | kept | Simulation time. Game code never reads the wall clock; it reads Time. |
| [`rng`](rng.md) | types and tokens | kept | Deterministic randomness. Every random draw comes from an Rng stream; the sequence for a seed is identical on every host and in the Web scaffold's rng.ts. |
| [`input`](input.md) | types and tokens | kept | Input as the game sees it: the Input resource for polling, events for edges. |
| [`camera`](camera.md) | types and tokens | kept | The main camera, as a resource with three.js PerspectiveCamera and OrthographicCamera parameter names. |
| [`render`](render.md) | `og.render.*` | kept | What is drawn: meshes with materials. Draws are instanced by (geometry, material) automatically; there is no per-object draw call. |
| [`load`](load.md) | `og.load.*` | kept | Asset loading from cooked output: `og cook` turns assets/ into cooked/ (.ogb blobs and the manifest cooked/.cook.ogt), and the runtime reads only cooked/. |
| [`commands`](commands.md) | `og.commands.*` | dropped | The command channel: actions scheduled on simulation time, from `og run --commands "t:action[:args];..."` or the probe's commands(). |
| [`probe`](probe.md) | types and tokens | kept | The probe: RuntimeProbe protocol 5 (origin/packages/contracts/src/probe.ts) served natively, plus the engine's wire members. |

## Units

Field units the docs and Tune use.

| Unit | Means |
|---|---|
| `m` | metres |
| `s` | seconds of simulation time |
| `ms` | milliseconds |
| `rad` | radians |
| `deg` | degrees |
| `hz` | ticks or cycles per second |
| `ticks` | fixed simulation steps |
| `cssPx` | CSS pixels of the play surface, origin at the top left |
| `ratio` | a fraction between 0 and 1 |
| `bytes` | bytes |

## Every name

226 entries: modules' functions, types, records, column kinds and constants, and every interface member and resource method.

| Name | Kind | Page | Binding symbol |
|---|---|---|---|
| [`AdvanceOperation`](probe.md#type-AdvanceOperation) | struct | probe | — |
| [`Antialias`](render.md#type-Antialias) | enum | render | — |
| [`ArchetypeDump`](probe.md#type-ArchetypeDump) | struct | probe | — |
| [`ArchetypeView`](world.md#type-ArchetypeView) | interface | world | — |
| [`ArchetypeView.column`](world.md#member-ArchetypeView-column) | method | world | `og_archetype_view_column` |
| [`ArchetypeView.count`](world.md#member-ArchetypeView-count) | property | world | `og_archetype_view_get_count` |
| [`ArchetypeView.entities`](world.md#member-ArchetypeView-entities) | property | world | `og_archetype_view_get_entities` |
| [`ArchetypeView.entity`](world.md#member-ArchetypeView-entity) | method | world | `og_archetype_view_entity` |
| [`ArchetypeView.has`](world.md#member-ArchetypeView-has) | method | world | `og_archetype_view_has` |
| [`ArchetypeView.tick`](world.md#member-ArchetypeView-tick) | property | world | `og_archetype_view_get_tick` |
| [`Asset`](load.md#type-Asset) | handle | load | — |
| [`Asset()`](load.md#column-Asset) | column kind | load | — |
| [`bool()`](reflect.md#column-bool) | column kind | reflect | — |
| [`BuildProfile`](probe.md#type-BuildProfile) | enum | probe | — |
| [`BundleInfo`](load.md#type-BundleInfo) | struct | load | — |
| [`Camera.lookAt`](camera.md#member-Camera-lookAt) | method | camera | `og_camera_look_at` |
| [`Camera.screenToRay`](camera.md#member-Camera-screenToRay) | method | camera | `og_camera_screen_to_ray` |
| [`Camera.worldToScreen`](camera.md#member-Camera-worldToScreen) | method | camera | `og_camera_world_to_screen` |
| [`Capture`](probe.md#type-Capture) | struct | probe | — |
| [`Color`](math.md#type-Color) | struct | math | — |
| [`Color()`](reflect.md#column-Color) | column kind | reflect | — |
| [`CommandDesc`](commands.md#type-CommandDesc) | desc struct | commands | — |
| [`CommandHandler`](commands.md#type-CommandHandler) | callback | commands | — |
| [`CommandInfo`](commands.md#type-CommandInfo) | struct | commands | — |
| [`CommandsReport`](commands.md#type-CommandsReport) | struct | commands | — |
| [`CommandStep`](commands.md#type-CommandStep) | struct | commands | — |
| [`CoreAction`](commands.md#type-CoreAction) | enum | commands | — |
| [`defineComponent`](reflect.md#fn-defineComponent) | function | reflect | `og_reflect_define_component` |
| [`defineResource`](reflect.md#fn-defineResource) | function | reflect | `og_reflect_define_resource` |
| [`defineSystem`](system.md#fn-defineSystem) | function | system | `og_system_define_system` |
| [`DumpDesc`](probe.md#type-DumpDesc) | desc struct | probe | — |
| [`DumpKind`](probe.md#type-DumpKind) | enum | probe | — |
| [`DumpResult`](probe.md#type-DumpResult) | union | probe | — |
| [`EngineProbe`](probe.md#type-EngineProbe) | interface | probe | — |
| [`EngineProbe.capture`](probe.md#member-EngineProbe-capture) | method | probe | `og_engine_probe_capture` |
| [`EngineProbe.commands`](probe.md#member-EngineProbe-commands) | method | probe | `og_engine_probe_commands` |
| [`EngineProbe.dump`](probe.md#member-EngineProbe-dump) | method | probe | `og_engine_probe_dump` |
| [`EngineProbe.hello`](probe.md#member-EngineProbe-hello) | method | probe | `og_engine_probe_hello` |
| [`EngineProbe.reload`](probe.md#member-EngineProbe-reload) | method | probe | `og_engine_probe_reload` |
| [`EngineProbe.restore`](probe.md#member-EngineProbe-restore) | method | probe | `og_engine_probe_restore` |
| [`EngineProbe.snapshot`](probe.md#member-EngineProbe-snapshot) | method | probe | `og_engine_probe_snapshot` |
| [`EngineProbe.sweep`](probe.md#member-EngineProbe-sweep) | method | probe | `og_engine_probe_sweep` |
| [`Entity`](world.md#type-Entity) | handle | world | — |
| [`Entity()`](world.md#column-Entity) | column kind | world | — |
| [`EntityDump`](probe.md#type-EntityDump) | struct | probe | — |
| [`EntityOperation`](probe.md#type-EntityOperation) | struct | probe | — |
| [`f32()`](reflect.md#column-f32) | column kind | reflect | — |
| [`f64()`](reflect.md#column-f64) | column kind | reflect | — |
| [`FactsFn`](game.md#type-FactsFn) | callback | game | — |
| [`FieldOptions`](reflect.md#type-FieldOptions) | desc struct | reflect | — |
| [`Font`](load.md#type-Font) | handle | load | — |
| [`FontInfo`](load.md#type-FontInfo) | struct | load | — |
| [`Game`](game.md#type-Game) | interface | game | — |
| [`Game.systems`](game.md#member-Game-systems) | property | game | `og_game_get_systems` |
| [`Game.tick`](game.md#member-Game-tick) | property | game | `og_game_get_tick` |
| [`GameDesc`](game.md#type-GameDesc) | desc struct | game | — |
| [`Geometry`](render.md#type-Geometry) | handle | render | — |
| [`Geometry()`](render.md#column-Geometry) | column kind | render | — |
| [`GltfDesc`](load.md#type-GltfDesc) | desc struct | load | — |
| [`GltfInfo`](load.md#type-GltfInfo) | struct | load | — |
| [`Hello`](probe.md#type-Hello) | struct | probe | — |
| [`i32()`](reflect.md#column-i32) | column kind | reflect | — |
| [`InjectedInput`](input.md#type-InjectedInput) | union | input | — |
| [`Input.axis`](input.md#member-Input-axis) | method | input | `og_input_axis` |
| [`Input.axis2`](input.md#member-Input-axis2) | method | input | `og_input_axis2` |
| [`Input.buttonDown`](input.md#member-Input-buttonDown) | method | input | `og_input_button_down` |
| [`Input.buttonPressed`](input.md#member-Input-buttonPressed) | method | input | `og_input_button_pressed` |
| [`Input.buttonReleased`](input.md#member-Input-buttonReleased) | method | input | `og_input_button_released` |
| [`Input.down`](input.md#member-Input-down) | method | input | `og_input_down` |
| [`Input.pressed`](input.md#member-Input-pressed) | method | input | `og_input_pressed` |
| [`Input.released`](input.md#member-Input-released) | method | input | `og_input_released` |
| [`InputOperation`](probe.md#type-InputOperation) | struct | probe | — |
| [`KeyAction`](input.md#type-KeyAction) | enum | input | — |
| [`KeyInput`](input.md#type-KeyInput) | struct | input | — |
| [`LoadStatus`](load.md#type-LoadStatus) | enum | load | — |
| [`Material`](render.md#type-Material) | handle | render | — |
| [`Material()`](render.md#column-Material) | column kind | render | — |
| [`Matrix4`](math.md#type-Matrix4) | struct | math | — |
| [`Matrix4()`](reflect.md#column-Matrix4) | column kind | reflect | — |
| [`MAX_LIGHTS`](render.md#const-MAX_LIGHTS) | constant | render | — |
| [`Modifiers`](input.md#type-Modifiers) | struct | input | — |
| [`og.AssetFailed`](load.md#record-AssetFailed) | event | load | — |
| [`og.AssetReady`](load.md#record-AssetReady) | event | load | — |
| [`og.Camera`](camera.md#record-Camera) | resource | camera | — |
| [`og.commands.help`](commands.md#fn-help) | function | commands | `og_commands_help` |
| [`og.commands.parse`](commands.md#fn-parse) | function | commands | `og_commands_parse` |
| [`og.commands.register`](commands.md#fn-register) | function | commands | `og_commands_register` |
| [`og.commands.schedule`](commands.md#fn-schedule) | function | commands | `og_commands_schedule` |
| [`og.DirectionalLight`](render.md#record-DirectionalLight) | component | render | — |
| [`og.game`](game.md#fn-game) | function | game | `og_game_game` |
| [`og.Input`](input.md#record-Input) | resource | input | — |
| [`og.KeyDown`](input.md#record-KeyDown) | event | input | — |
| [`og.KeyUp`](input.md#record-KeyUp) | event | input | — |
| [`og.load.bundle`](load.md#fn-bundle) | function | load | `og_load_bundle` |
| [`og.load.bundleInfo`](load.md#fn-bundleInfo) | function | load | `og_load_bundle_info` |
| [`og.load.defaultBundle`](load.md#fn-defaultBundle) | function | load | `og_load_default_bundle` |
| [`og.load.error`](load.md#fn-error) | function | load | `og_load_error` |
| [`og.load.font`](load.md#fn-font) | function | load | `og_load_font` |
| [`og.load.fontInfo`](load.md#fn-fontInfo) | function | load | `og_load_font_info` |
| [`og.load.gltf`](load.md#fn-gltf) | function | load | `og_load_gltf` |
| [`og.load.gltfInfo`](load.md#fn-gltfInfo) | function | load | `og_load_gltf_info` |
| [`og.load.scene`](load.md#fn-scene) | function | load | `og_load_scene` |
| [`og.load.sound`](load.md#fn-sound) | function | load | `og_load_sound` |
| [`og.load.soundInfo`](load.md#fn-soundInfo) | function | load | `og_load_sound_info` |
| [`og.load.status`](load.md#fn-status) | function | load | `og_load_status` |
| [`og.MatrixWorld`](transform.md#record-MatrixWorld) | component | transform | — |
| [`og.Mesh`](render.md#record-Mesh) | component | render | — |
| [`og.Model`](load.md#record-Model) | component | load | — |
| [`og.Parent`](transform.md#record-Parent) | component | transform | — |
| [`og.PointerDown`](input.md#record-PointerDown) | event | input | — |
| [`og.PointerMove`](input.md#record-PointerMove) | event | input | — |
| [`og.PointerUp`](input.md#record-PointerUp) | event | input | — |
| [`og.PointLight`](render.md#record-PointLight) | component | render | — |
| [`og.render.capabilities`](render.md#fn-capabilities) | function | render | `og_render_capabilities` |
| [`og.render.standardMaterial`](render.md#fn-standardMaterial) | function | render | `og_render_standard_material` |
| [`og.render.text`](render.md#fn-text) | function | render | `og_render_text` |
| [`og.render.updateMaterial`](render.md#fn-updateMaterial) | function | render | `og_render_update_material` |
| [`og.RenderSettings`](render.md#record-RenderSettings) | resource | render | — |
| [`og.Rng`](rng.md#record-Rng) | resource | rng | — |
| [`og.SpotLight`](render.md#record-SpotLight) | component | render | — |
| [`og.Time`](time.md#record-Time) | resource | time | — |
| [`og.Transform`](transform.md#record-Transform) | component | transform | — |
| [`og.transform.compose`](transform.md#fn-compose) | function | transform | `og_transform_compose` |
| [`og.transform.decompose`](transform.md#fn-decompose) | function | transform | `og_transform_decompose` |
| [`og.transform.fromAxisAngle`](transform.md#fn-fromAxisAngle) | function | transform | `og_transform_from_axis_angle` |
| [`og.transform.lookAt`](transform.md#fn-lookAt) | function | transform | `og_transform_look_at` |
| [`og.Wheel`](input.md#record-Wheel) | event | input | — |
| [`PickOperation`](probe.md#type-PickOperation) | struct | probe | — |
| [`PickResult`](probe.md#type-PickResult) | struct | probe | — |
| [`PLAYTEST_MAX_TICKS`](probe.md#const-PLAYTEST_MAX_TICKS) | constant | probe | — |
| [`PointerAction`](input.md#type-PointerAction) | enum | input | — |
| [`PointerButton`](input.md#type-PointerButton) | enum | input | — |
| [`PointerInput`](input.md#type-PointerInput) | struct | input | — |
| [`PROBE_GLOBAL`](probe.md#const-PROBE_GLOBAL) | constant | probe | — |
| [`PROBE_PROGRAM_MAX_OPERATIONS`](probe.md#const-PROBE_PROGRAM_MAX_OPERATIONS) | constant | probe | — |
| [`PROBE_VERSION`](probe.md#const-PROBE_VERSION) | constant | probe | — |
| [`PROBE_WIRE_VERSION`](probe.md#const-PROBE_WIRE_VERSION) | constant | probe | — |
| [`ProbeEntityRef`](probe.md#type-ProbeEntityRef) | struct | probe | — |
| [`ProbeOperation`](probe.md#type-ProbeOperation) | union | probe | — |
| [`ProbeProgramResult`](probe.md#type-ProbeProgramResult) | struct | probe | — |
| [`ProbeState`](probe.md#type-ProbeState) | struct | probe | — |
| [`ProbeTimings`](probe.md#type-ProbeTimings) | struct | probe | — |
| [`ProbeTunable`](probe.md#type-ProbeTunable) | struct | probe | — |
| [`Quaternion`](math.md#type-Quaternion) | struct | math | — |
| [`Quaternion()`](reflect.md#column-Quaternion) | column kind | reflect | — |
| [`Query`](world.md#type-Query) | interface | world | — |
| [`Query.archetypes`](world.md#member-Query-archetypes) | method | world | `og_query_archetypes` |
| [`Query.count`](world.md#member-Query-count) | method | world | `og_query_count` |
| [`QueryDesc`](world.md#type-QueryDesc) | desc struct | world | — |
| [`Ray`](math.md#type-Ray) | struct | math | — |
| [`ReloadReport`](probe.md#type-ReloadReport) | struct | probe | — |
| [`RenderCapabilities`](render.md#type-RenderCapabilities) | struct | render | — |
| [`RenderOptions`](render.md#type-RenderOptions) | desc struct | render | — |
| [`RenderTier`](render.md#type-RenderTier) | enum | render | — |
| [`REPLAY_SEED_PARAM`](probe.md#const-REPLAY_SEED_PARAM) | constant | probe | — |
| [`Rng.chance`](rng.md#member-Rng-chance) | method | rng | `og_rng_chance` |
| [`Rng.fork`](rng.md#member-Rng-fork) | method | rng | `og_rng_fork` |
| [`Rng.integer`](rng.md#member-Rng-integer) | method | rng | `og_rng_integer` |
| [`Rng.next`](rng.md#member-Rng-next) | method | rng | `og_rng_next` |
| [`Rng.range`](rng.md#member-Rng-range) | method | rng | `og_rng_range` |
| [`RngStream`](rng.md#type-RngStream) | interface | rng | — |
| [`RngStream.chance`](rng.md#member-RngStream-chance) | method | rng | `og_rng_stream_chance` |
| [`RngStream.fork`](rng.md#member-RngStream-fork) | method | rng | `og_rng_stream_fork` |
| [`RngStream.integer`](rng.md#member-RngStream-integer) | method | rng | `og_rng_stream_integer` |
| [`RngStream.next`](rng.md#member-RngStream-next) | method | rng | `og_rng_stream_next` |
| [`RngStream.range`](rng.md#member-RngStream-range) | method | rng | `og_rng_stream_range` |
| [`RngStream.seed`](rng.md#member-RngStream-seed) | property | rng | `og_rng_stream_get_seed` |
| [`RuntimeProbe`](probe.md#type-RuntimeProbe) | interface | probe | — |
| [`RuntimeProbe.accepted`](probe.md#member-RuntimeProbe-accepted) | method | probe | `og_runtime_probe_accepted` |
| [`RuntimeProbe.advance`](probe.md#member-RuntimeProbe-advance) | method | probe | `og_runtime_probe_advance` |
| [`RuntimeProbe.bounds`](probe.md#member-RuntimeProbe-bounds) | method | probe | `og_runtime_probe_bounds` |
| [`RuntimeProbe.entity`](probe.md#member-RuntimeProbe-entity) | method | probe | `og_runtime_probe_entity` |
| [`RuntimeProbe.input`](probe.md#member-RuntimeProbe-input) | method | probe | `og_runtime_probe_input` |
| [`RuntimeProbe.patch`](probe.md#member-RuntimeProbe-patch) | method | probe | `og_runtime_probe_patch` |
| [`RuntimeProbe.pause`](probe.md#member-RuntimeProbe-pause) | method | probe | `og_runtime_probe_pause` |
| [`RuntimeProbe.pick`](probe.md#member-RuntimeProbe-pick) | method | probe | `og_runtime_probe_pick` |
| [`RuntimeProbe.resume`](probe.md#member-RuntimeProbe-resume) | method | probe | `og_runtime_probe_resume` |
| [`RuntimeProbe.run`](probe.md#member-RuntimeProbe-run) | method | probe | `og_runtime_probe_run` |
| [`RuntimeProbe.state`](probe.md#member-RuntimeProbe-state) | method | probe | `og_runtime_probe_state` |
| [`RuntimeProbe.tick`](probe.md#member-RuntimeProbe-tick) | method | probe | `og_runtime_probe_tick` |
| [`RuntimeProbe.version`](probe.md#member-RuntimeProbe-version) | property | probe | `og_runtime_probe_get_version` |
| [`Side`](render.md#type-Side) | enum | render | — |
| [`SnapshotInfo`](probe.md#type-SnapshotInfo) | struct | probe | — |
| [`Sound`](load.md#type-Sound) | handle | load | — |
| [`SoundInfo`](load.md#type-SoundInfo) | struct | load | — |
| [`SpawnDesc`](world.md#type-SpawnDesc) | desc struct | world | — |
| [`StandardMaterialDesc`](render.md#type-StandardMaterialDesc) | desc struct | render | — |
| [`StartFn`](game.md#type-StartFn) | callback | game | — |
| [`SweepDesc`](probe.md#type-SweepDesc) | desc struct | probe | — |
| [`SweepFrame`](probe.md#type-SweepFrame) | struct | probe | — |
| [`SweepResult`](probe.md#type-SweepResult) | struct | probe | — |
| [`System`](system.md#type-System) | interface | system | — |
| [`System.name`](system.md#member-System-name) | property | system | `og_system_get_name` |
| [`System.reads`](system.md#member-System-reads) | property | system | `og_system_get_reads` |
| [`System.writes`](system.md#member-System-writes) | property | system | `og_system_get_writes` |
| [`SystemDesc`](system.md#type-SystemDesc) | desc struct | system | — |
| [`SystemDump`](probe.md#type-SystemDump) | struct | probe | — |
| [`SystemRun`](system.md#type-SystemRun) | callback | system | — |
| [`TextDesc`](render.md#type-TextDesc) | desc struct | render | — |
| [`Texture`](render.md#type-Texture) | handle | render | — |
| [`u32()`](reflect.md#column-u32) | column kind | reflect | — |
| [`UiDump`](probe.md#type-UiDump) | struct | probe | — |
| [`UiElement`](probe.md#type-UiElement) | struct | probe | — |
| [`Vector2`](math.md#type-Vector2) | struct | math | — |
| [`Vector2()`](reflect.md#column-Vector2) | column kind | reflect | — |
| [`Vector3`](math.md#type-Vector3) | struct | math | — |
| [`Vector3()`](reflect.md#column-Vector3) | column kind | reflect | — |
| [`WheelInput`](input.md#type-WheelInput) | struct | input | — |
| [`World`](world.md#type-World) | interface | world | — |
| [`World.alive`](world.md#member-World-alive) | method | world | `og_world_alive` |
| [`World.despawn`](world.md#member-World-despawn) | method | world | `og_world_despawn` |
| [`World.emit`](world.md#member-World-emit) | method | world | `og_world_emit` |
| [`World.events`](world.md#member-World-events) | method | world | `og_world_events` |
| [`World.find`](world.md#member-World-find) | method | world | `og_world_find` |
| [`World.get`](world.md#member-World-get) | method | world | `og_world_get` |
| [`World.has`](world.md#member-World-has) | method | world | `og_world_has` |
| [`World.insert`](world.md#member-World-insert) | method | world | `og_world_insert` |
| [`World.instantiate`](world.md#member-World-instantiate) | method | world | `og_world_instantiate` |
| [`World.nameOf`](world.md#member-World-nameOf) | method | world | `og_world_name_of` |
| [`World.query`](world.md#member-World-query) | method | world | `og_world_query` |
| [`World.remove`](world.md#member-World-remove) | method | world | `og_world_remove` |
| [`World.resource`](world.md#member-World-resource) | method | world | `og_world_resource` |
| [`World.set`](world.md#member-World-set) | method | world | `og_world_set` |
| [`World.spawn`](world.md#member-World-spawn) | method | world | `og_world_spawn` |
| [`World.tick`](world.md#member-World-tick) | property | world | `og_world_get_tick` |
| [`WorldDump`](probe.md#type-WorldDump) | struct | probe | — |
