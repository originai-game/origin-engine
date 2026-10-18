<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The Origin Engine API

`@origingame/engine`, API version 1: 23 modules, generated from `crates/og-api/api.ogt`. Each module has one page; every signature on it is quoted from the IDL.

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
| [`time`](time.md) | `og.time.*` | kept | Simulation time. Game code never reads the wall clock; it reads Time. |
| [`rng`](rng.md) | types and tokens | kept | Deterministic randomness. Every random draw comes from an Rng stream; the sequence for a seed is identical on every host and in the Web scaffold's rng.ts. |
| [`input`](input.md) | types and tokens | kept | Input as the game sees it: the Input resource for polling, events for edges. |
| [`camera`](camera.md) | `og.camera.*` | kept | The main camera, as a resource with three.js PerspectiveCamera and OrthographicCamera parameter names. |
| [`render`](render.md) | `og.render.*` | kept | What is drawn: meshes with materials. Draws are instanced by (geometry, material) automatically; there is no per-object draw call. |
| [`load`](load.md) | `og.load.*` | kept | Asset loading from cooked output: `og cook` turns assets/ into cooked/ (.ogb blobs and the manifest cooked/.cook.ogt), and the runtime reads only cooked/. |
| [`physics`](physics.md) | `og.physics.*` | kept | Rigid bodies over rapier (rapier3d in the 3D profile, rapier2d in 2D, both with enhanced determinism): game code owns the components, the engine's og.physics system steps them after the game's systems and writes the dynamic bodies back. |
| [`anim`](anim.md) | `og.anim.*` | kept | Skeletal animation over og-anim: glTF skins and clips (cooked into the `<model>#anim` blob, loaded with og.load.animation), blend-tree graphs with layered state machines, root motion, two-bone and FABRIK IK and blendshapes, evaluated by the engine's og.anim system after the game's systems and before physics. |
| [`tween`](tween.md) | `og.tween.*` | kept | Tweens: animate any numeric field of a component (og.Transform.position, og.Sprite.color, og.Sprite.opacity, a game's own f32) from its value to a target over time with an easing curve, a delay, loops and yoyo; chain them in sequences or run them in parallel. Advanced by the engine's og.tween system after the game's systems and before og.anim and physics, by scaled time (slow motion, hit stop) unless unscaled. |
| [`tilemap`](tilemap.md) | `og.tilemap.*` | kept | Tilemaps: layers of tiles from a tileset atlas (an `.ogt` of `type tileset`: grid, solid tiles and collision rects, animated tiles, 47-blob and 16-bitmask auto-tiling terrains), loaded from a cooked map (a hand-written `.ogt` of `type tilemap`, a Tiled `.tmj` or an LDtk `.ldtk`) with og.load.tilemap and placed by an og.Tilemap component. |
| [`audio`](audio.md) | types and tokens | kept | Sound: inside the tick, the engine's og.audio system (after the game's systems and physics) turns AudioSource changes into a deterministic, time-stamped command stream; outside the tick the host's mixer renders it (an AudioWorklet on the web, the output device, a capture or a null sink natively). |
| [`nav`](nav.md) | `og.nav.*` | kept | Navigation: grids (A* and jump point search with eight-way moves that never cut a blocked corner, line-of-sight smoothing), flow fields for crowds, and navmeshes (baked at cook time from a level's static colliders and models, or built at runtime for a flat procedural level) with funnel paths and box obstacles that rebuild only the tiles they touch. |
| [`ai`](ai.md) | `og.ai.*` | kept | Enemy and NPC AI as data: behaviour trees and state machines in `.ogt` files of `type behaviour` (crates/og-ai/README.md has the grammar), perception (sight cones checked against physics occlusion, hearing of spatial sounds and game noises) and a blackboard per agent. |
| [`ui`](ui.md) | `og.ui.*` | kept | The engine's UI: an immediate-mode toolkit. Systems describe the tree every tick with these calls; the engine lays it out Clay-style, themes it with the platform's design tokens, draws it over the frame and answers the probe (dump_ui, touch_move_to, move_to_text) from it. |
| [`systems`](systems.md) | types and tokens | kept | Systems the engine ships as TypeScript: game-level code, listed explicitly in GameDesc.systems like the game's own, so the schedule (and dump_world) shows them and nothing runs that the game did not ask for. |
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
| `kg` | kilograms |
| `mps` | metres per second |
| `mps2` | metres per second squared |
| `radps` | radians per second |
| `perSecond` | a rate per second: the fraction of a value that decays in one second |
| `kgPerM3` | kilograms per cubic metre (per square metre in the 2D profile) |

## Every name

689 entries: modules' functions, types, records, column kinds and constants, and every interface member and resource method.

| Name | Kind | Page | Binding symbol |
|---|---|---|---|
| [`Ability`](ui.md#type-Ability) | desc struct | ui | — |
| [`ActionButtonDesc`](ui.md#type-ActionButtonDesc) | desc struct | ui | — |
| [`AdvanceOperation`](probe.md#type-AdvanceOperation) | struct | probe | — |
| [`AI_FAILURE`](ai.md#const-AI_FAILURE) | constant | ai | — |
| [`AI_RUNNING`](ai.md#const-AI_RUNNING) | constant | ai | — |
| [`AI_SUCCESS`](ai.md#const-AI_SUCCESS) | constant | ai | — |
| [`AiDescription`](ai.md#type-AiDescription) | struct | ai | — |
| [`AlertDesc`](ui.md#type-AlertDesc) | desc struct | ui | — |
| [`Align`](ui.md#type-Align) | enum | ui | — |
| [`Anchor`](ui.md#type-Anchor) | enum | ui | — |
| [`AnchorDesc`](ui.md#type-AnchorDesc) | desc struct | ui | — |
| [`Animation`](anim.md#type-Animation) | struct | anim | — |
| [`Antialias`](render.md#type-Antialias) | enum | render | — |
| [`AoMethod`](render.md#type-AoMethod) | enum | render | — |
| [`AoMethod()`](render.md#column-AoMethod) | column kind | render | — |
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
| [`AudioBus`](audio.md#type-AudioBus) | enum | audio | — |
| [`AudioBus()`](audio.md#column-AudioBus) | column kind | audio | — |
| [`Behaviour`](ai.md#type-Behaviour) | handle | ai | — |
| [`Behaviour()`](ai.md#column-Behaviour) | column kind | ai | — |
| [`BehaviourInfo`](ai.md#type-BehaviourInfo) | struct | ai | — |
| [`BlendshapeSource`](anim.md#type-BlendshapeSource) | enum | anim | — |
| [`BlendshapeSource()`](anim.md#column-BlendshapeSource) | column kind | anim | — |
| [`BODY_DYNAMIC`](physics.md#const-BODY_DYNAMIC) | constant | physics | — |
| [`BODY_KINEMATIC`](physics.md#const-BODY_KINEMATIC) | constant | physics | — |
| [`BODY_STATIC`](physics.md#const-BODY_STATIC) | constant | physics | — |
| [`bool()`](reflect.md#column-bool) | column kind | reflect | — |
| [`BoxGeometryDesc`](render.md#type-BoxGeometryDesc) | desc struct | render | — |
| [`Breakpoint`](ui.md#type-Breakpoint) | enum | ui | — |
| [`BuildProfile`](probe.md#type-BuildProfile) | enum | probe | — |
| [`BundleInfo`](load.md#type-BundleInfo) | struct | load | — |
| [`BurstDesc`](ui.md#type-BurstDesc) | desc struct | ui | — |
| [`ButtonDesc`](ui.md#type-ButtonDesc) | desc struct | ui | — |
| [`ButtonVariant`](ui.md#type-ButtonVariant) | enum | ui | — |
| [`Camera.lookAt`](camera.md#member-Camera-lookAt) | method | camera | `og_camera_look_at` |
| [`Camera.screenToRay`](camera.md#member-Camera-screenToRay) | method | camera | `og_camera_screen_to_ray` |
| [`Camera.worldToScreen`](camera.md#member-Camera-worldToScreen) | method | camera | `og_camera_world_to_screen` |
| [`Capture`](probe.md#type-Capture) | struct | probe | — |
| [`CheckListResult`](ui.md#type-CheckListResult) | struct | ui | — |
| [`ChipDesc`](ui.md#type-ChipDesc) | desc struct | ui | — |
| [`ChoiceResult`](ui.md#type-ChoiceResult) | struct | ui | — |
| [`ClipAsset`](anim.md#type-ClipAsset) | handle | anim | — |
| [`ClipAsset()`](anim.md#column-ClipAsset) | column kind | anim | — |
| [`ClipInfo`](anim.md#type-ClipInfo) | struct | anim | — |
| [`Color`](math.md#type-Color) | struct | math | — |
| [`Color()`](reflect.md#column-Color) | column kind | reflect | — |
| [`CommandDesc`](commands.md#type-CommandDesc) | desc struct | commands | — |
| [`CommandHandler`](commands.md#type-CommandHandler) | callback | commands | — |
| [`CommandInfo`](commands.md#type-CommandInfo) | struct | commands | — |
| [`CommandsReport`](commands.md#type-CommandsReport) | struct | commands | — |
| [`CommandStep`](commands.md#type-CommandStep) | struct | commands | — |
| [`ConfirmDesc`](ui.md#type-ConfirmDesc) | desc struct | ui | — |
| [`ConfirmResult`](ui.md#type-ConfirmResult) | struct | ui | — |
| [`Contrast`](ui.md#type-Contrast) | enum | ui | — |
| [`ControlSize`](ui.md#type-ControlSize) | enum | ui | — |
| [`ControlSizes`](ui.md#type-ControlSizes) | struct | ui | — |
| [`CooldownDesc`](ui.md#type-CooldownDesc) | desc struct | ui | — |
| [`CoreAction`](commands.md#type-CoreAction) | enum | commands | — |
| [`CounterDesc`](ui.md#type-CounterDesc) | desc struct | ui | — |
| [`Cross`](ui.md#type-Cross) | enum | ui | — |
| [`Currency`](ui.md#type-Currency) | desc struct | ui | — |
| [`CutsceneDesc`](ui.md#type-CutsceneDesc) | desc struct | ui | — |
| [`CutsceneResult`](ui.md#type-CutsceneResult) | struct | ui | — |
| [`defineComponent`](reflect.md#fn-defineComponent) | function | reflect | `og_reflect_define_component` |
| [`defineResource`](reflect.md#fn-defineResource) | function | reflect | `og_reflect_define_resource` |
| [`defineSystem`](system.md#fn-defineSystem) | function | system | `og_system_define_system` |
| [`DensityMode`](ui.md#type-DensityMode) | enum | ui | — |
| [`DialogueDesc`](ui.md#type-DialogueDesc) | desc struct | ui | — |
| [`DialogueLine`](ui.md#type-DialogueLine) | desc struct | ui | — |
| [`DialogueResult`](ui.md#type-DialogueResult) | struct | ui | — |
| [`Direction`](ui.md#type-Direction) | enum | ui | — |
| [`DofQuality`](render.md#type-DofQuality) | enum | render | — |
| [`DofQuality()`](render.md#column-DofQuality) | column kind | render | — |
| [`DpadDesc`](ui.md#type-DpadDesc) | desc struct | ui | — |
| [`DpadResult`](ui.md#type-DpadResult) | struct | ui | — |
| [`DumpDesc`](probe.md#type-DumpDesc) | desc struct | probe | — |
| [`DumpKind`](probe.md#type-DumpKind) | enum | probe | — |
| [`DumpResult`](probe.md#type-DumpResult) | union | probe | — |
| [`Ease`](tween.md#type-Ease) | enum | tween | — |
| [`ElementDesc`](ui.md#type-ElementDesc) | desc struct | ui | — |
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
| [`FieldDesc`](ui.md#type-FieldDesc) | desc struct | ui | — |
| [`FieldOptions`](reflect.md#type-FieldOptions) | desc struct | reflect | — |
| [`FieldResult`](ui.md#type-FieldResult) | struct | ui | — |
| [`FixedSize`](ui.md#type-FixedSize) | struct | ui | — |
| [`FlashDesc`](tween.md#type-FlashDesc) | desc struct | tween | — |
| [`FlipbookClipInfo`](anim.md#type-FlipbookClipInfo) | struct | anim | — |
| [`FlipbookInfo`](anim.md#type-FlipbookInfo) | struct | anim | — |
| [`FlipbookSheet`](anim.md#type-FlipbookSheet) | handle | anim | — |
| [`FlipbookSheet()`](anim.md#column-FlipbookSheet) | column kind | anim | — |
| [`FlipMode`](anim.md#type-FlipMode) | enum | anim | — |
| [`FloatingTextDesc`](render.md#type-FloatingTextDesc) | desc struct | render | — |
| [`FlowDesc`](nav.md#type-FlowDesc) | desc struct | nav | — |
| [`Font`](load.md#type-Font) | handle | load | — |
| [`FontInfo`](load.md#type-FontInfo) | struct | load | — |
| [`FontRole`](ui.md#type-FontRole) | enum | ui | — |
| [`Game`](game.md#type-Game) | interface | game | — |
| [`Game.systems`](game.md#member-Game-systems) | property | game | `og_game_get_systems` |
| [`Game.tick`](game.md#member-Game-tick) | property | game | `og_game_get_tick` |
| [`GameDesc`](game.md#type-GameDesc) | desc struct | game | — |
| [`Gamepad.axis`](input.md#member-Gamepad-axis) | method | input | `og_gamepad_axis` |
| [`Gamepad.connected`](input.md#member-Gamepad-connected) | method | input | `og_gamepad_connected` |
| [`Gamepad.down`](input.md#member-Gamepad-down) | method | input | `og_gamepad_down` |
| [`Gamepad.pressed`](input.md#member-Gamepad-pressed) | method | input | `og_gamepad_pressed` |
| [`Gamepad.released`](input.md#member-Gamepad-released) | method | input | `og_gamepad_released` |
| [`GamepadAxis`](input.md#type-GamepadAxis) | enum | input | — |
| [`GamepadButton`](input.md#type-GamepadButton) | enum | input | — |
| [`GamepadInput`](input.md#type-GamepadInput) | struct | input | — |
| [`GameSettings`](ui.md#type-GameSettings) | desc struct | ui | — |
| [`Generator`](ui.md#type-Generator) | desc struct | ui | — |
| [`Geometry`](render.md#type-Geometry) | handle | render | — |
| [`Geometry()`](render.md#column-Geometry) | column kind | render | — |
| [`GltfDesc`](load.md#type-GltfDesc) | desc struct | load | — |
| [`GltfInfo`](load.md#type-GltfInfo) | struct | load | — |
| [`GraphAsset`](anim.md#type-GraphAsset) | handle | anim | — |
| [`GraphAsset()`](anim.md#column-GraphAsset) | column kind | anim | — |
| [`GraphDesc`](anim.md#type-GraphDesc) | desc struct | anim | — |
| [`GraphicsQuality`](ui.md#type-GraphicsQuality) | enum | ui | — |
| [`GridDesc`](ui.md#type-GridDesc) | desc struct | ui | — |
| [`Hello`](probe.md#type-Hello) | struct | probe | — |
| [`HudResult`](ui.md#type-HudResult) | struct | ui | — |
| [`HueName`](ui.md#type-HueName) | enum | ui | — |
| [`i32()`](reflect.md#column-i32) | column kind | reflect | — |
| [`IconAtlasDesc`](ui.md#type-IconAtlasDesc) | desc struct | ui | — |
| [`IconDesc`](ui.md#type-IconDesc) | desc struct | ui | — |
| [`IdleHudDesc`](ui.md#type-IdleHudDesc) | desc struct | ui | — |
| [`IkSolver`](anim.md#type-IkSolver) | enum | anim | — |
| [`IkSolver()`](anim.md#column-IkSolver) | column kind | anim | — |
| [`IkSpace`](anim.md#type-IkSpace) | enum | anim | — |
| [`IkSpace()`](anim.md#column-IkSpace) | column kind | anim | — |
| [`ImageDesc`](ui.md#type-ImageDesc) | desc struct | ui | — |
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
| [`Insets`](ui.md#type-Insets) | struct | ui | — |
| [`InventoryDesc`](ui.md#type-InventoryDesc) | desc struct | ui | — |
| [`InventoryItem`](ui.md#type-InventoryItem) | desc struct | ui | — |
| [`InventoryResult`](ui.md#type-InventoryResult) | struct | ui | — |
| [`JoystickDesc`](ui.md#type-JoystickDesc) | desc struct | ui | — |
| [`JoystickResult`](ui.md#type-JoystickResult) | struct | ui | — |
| [`KeyAction`](input.md#type-KeyAction) | enum | input | — |
| [`KeyBinding`](ui.md#type-KeyBinding) | desc struct | ui | — |
| [`KeyInput`](input.md#type-KeyInput) | struct | input | — |
| [`LabelDesc`](ui.md#type-LabelDesc) | desc struct | ui | — |
| [`LanguageOption`](ui.md#type-LanguageOption) | desc struct | ui | — |
| [`LeaderboardDesc`](ui.md#type-LeaderboardDesc) | desc struct | ui | — |
| [`LeaderboardEntry`](ui.md#type-LeaderboardEntry) | desc struct | ui | — |
| [`LeaderboardResult`](ui.md#type-LeaderboardResult) | struct | ui | — |
| [`LevelInfo`](ui.md#type-LevelInfo) | desc struct | ui | — |
| [`LevelSelectDesc`](ui.md#type-LevelSelectDesc) | desc struct | ui | — |
| [`LevelSelectResult`](ui.md#type-LevelSelectResult) | struct | ui | — |
| [`ListDesc`](ui.md#type-ListDesc) | desc struct | ui | — |
| [`ListRange`](ui.md#type-ListRange) | struct | ui | — |
| [`ListResult`](ui.md#type-ListResult) | struct | ui | — |
| [`LoadingDesc`](ui.md#type-LoadingDesc) | desc struct | ui | — |
| [`LoadStatus`](load.md#type-LoadStatus) | enum | load | — |
| [`LOCK_ROTATION_X`](physics.md#const-LOCK_ROTATION_X) | constant | physics | — |
| [`LOCK_ROTATION_Y`](physics.md#const-LOCK_ROTATION_Y) | constant | physics | — |
| [`LOCK_ROTATION_Z`](physics.md#const-LOCK_ROTATION_Z) | constant | physics | — |
| [`LOCK_TRANSLATION_X`](physics.md#const-LOCK_TRANSLATION_X) | constant | physics | — |
| [`LOCK_TRANSLATION_Y`](physics.md#const-LOCK_TRANSLATION_Y) | constant | physics | — |
| [`LOCK_TRANSLATION_Z`](physics.md#const-LOCK_TRANSLATION_Z) | constant | physics | — |
| [`Lut`](render.md#type-Lut) | handle | render | — |
| [`Lut()`](render.md#column-Lut) | column kind | render | — |
| [`MainMenuDesc`](ui.md#type-MainMenuDesc) | desc struct | ui | — |
| [`MapMarker`](ui.md#type-MapMarker) | desc struct | ui | — |
| [`Material`](render.md#type-Material) | handle | render | — |
| [`Material()`](render.md#column-Material) | column kind | render | — |
| [`Matrix4`](math.md#type-Matrix4) | struct | math | — |
| [`Matrix4()`](reflect.md#column-Matrix4) | column kind | reflect | — |
| [`MAX_GAMEPADS`](input.md#const-MAX_GAMEPADS) | constant | input | — |
| [`MAX_LIGHTS`](render.md#const-MAX_LIGHTS) | constant | render | — |
| [`MAX_POINTERS`](input.md#const-MAX_POINTERS) | constant | input | — |
| [`MenuItem`](ui.md#type-MenuItem) | desc struct | ui | — |
| [`MenuResult`](ui.md#type-MenuResult) | struct | ui | — |
| [`MeshDesc`](physics.md#type-MeshDesc) | desc struct | physics | — |
| [`MeterDesc`](ui.md#type-MeterDesc) | desc struct | ui | — |
| [`MeterKind`](ui.md#type-MeterKind) | enum | ui | — |
| [`MinimapDesc`](ui.md#type-MinimapDesc) | desc struct | ui | — |
| [`ModalDesc`](ui.md#type-ModalDesc) | desc struct | ui | — |
| [`ModalResult`](ui.md#type-ModalResult) | struct | ui | — |
| [`Modifiers`](input.md#type-Modifiers) | struct | input | — |
| [`NAV_ARRIVED`](nav.md#const-NAV_ARRIVED) | constant | nav | — |
| [`NAV_IDLE`](nav.md#const-NAV_IDLE) | constant | nav | — |
| [`NAV_MOVING`](nav.md#const-NAV_MOVING) | constant | nav | — |
| [`NAV_PARTIAL`](nav.md#const-NAV_PARTIAL) | constant | nav | — |
| [`NAV_UNREACHABLE`](nav.md#const-NAV_UNREACHABLE) | constant | nav | — |
| [`NavAlgorithm`](nav.md#type-NavAlgorithm) | enum | nav | — |
| [`NavBox`](nav.md#type-NavBox) | desc struct | nav | — |
| [`NavCell`](nav.md#type-NavCell) | struct | nav | — |
| [`NavGridDesc`](nav.md#type-NavGridDesc) | desc struct | nav | — |
| [`Navigation`](nav.md#type-Navigation) | handle | nav | — |
| [`Navigation()`](nav.md#column-Navigation) | column kind | nav | — |
| [`NavIntent`](ui.md#type-NavIntent) | enum | ui | — |
| [`NavMeshDesc`](nav.md#type-NavMeshDesc) | desc struct | nav | — |
| [`NavMeshInfo`](nav.md#type-NavMeshInfo) | struct | nav | — |
| [`NavPath`](nav.md#type-NavPath) | struct | nav | — |
| [`NavRebuild`](nav.md#type-NavRebuild) | struct | nav | — |
| [`NoiseDesc`](ai.md#type-NoiseDesc) | desc struct | ai | — |
| [`og.Afterimage`](render.md#record-Afterimage) | component | render | — |
| [`og.ai.describe`](ai.md#fn-describe) | function | ai | `og_ai_describe` |
| [`og.ai.get`](ai.md#fn-get) | function | ai | `og_ai_get` |
| [`og.ai.load`](ai.md#fn-load) | function | ai | `og_ai_load` |
| [`og.ai.noise`](ai.md#fn-noise) | function | ai | `og_ai_noise` |
| [`og.ai.parse`](ai.md#fn-parse) | function | ai | `og_ai_parse` |
| [`og.ai.set`](ai.md#fn-set) | function | ai | `og_ai_set` |
| [`og.AiAgent`](ai.md#record-AiAgent) | component | ai | — |
| [`og.AiChanged`](ai.md#record-AiChanged) | event | ai | — |
| [`og.anim.clip`](anim.md#fn-clip) | function | anim | — |
| [`og.anim.graph`](anim.md#fn-graph) | function | anim | `og_anim_graph` |
| [`og.AnimationEvent`](anim.md#record-AnimationEvent) | event | anim | — |
| [`og.AnimationGraph`](anim.md#record-AnimationGraph) | component | anim | — |
| [`og.AnimationPlayer`](anim.md#record-AnimationPlayer) | component | anim | — |
| [`og.AssetFailed`](load.md#record-AssetFailed) | event | load | — |
| [`og.AssetReady`](load.md#record-AssetReady) | event | load | — |
| [`og.Audio`](audio.md#record-Audio) | resource | audio | — |
| [`og.AudioListener`](audio.md#record-AudioListener) | component | audio | — |
| [`og.AudioSource`](audio.md#record-AudioSource) | component | audio | — |
| [`og.Beam`](render.md#record-Beam) | component | render | — |
| [`og.Blendshapes`](anim.md#record-Blendshapes) | component | anim | — |
| [`og.Camera`](camera.md#record-Camera) | resource | camera | — |
| [`og.camera.kick`](camera.md#fn-kick) | function | camera | `og_camera_kick` |
| [`og.camera.punch`](camera.md#fn-punch) | function | camera | `og_camera_punch` |
| [`og.camera.shake`](camera.md#fn-shake) | function | camera | `og_camera_shake` |
| [`og.CameraRail`](camera.md#record-CameraRail) | component | camera | — |
| [`og.CameraZone`](camera.md#record-CameraZone) | component | camera | — |
| [`og.Collider`](physics.md#record-Collider) | component | physics | — |
| [`og.CollisionEnded`](physics.md#record-CollisionEnded) | event | physics | — |
| [`og.CollisionStarted`](physics.md#record-CollisionStarted) | event | physics | — |
| [`og.commands.help`](commands.md#fn-help) | function | commands | `og_commands_help` |
| [`og.commands.parse`](commands.md#fn-parse) | function | commands | `og_commands_parse` |
| [`og.commands.register`](commands.md#fn-register) | function | commands | `og_commands_register` |
| [`og.commands.schedule`](commands.md#fn-schedule) | function | commands | `og_commands_schedule` |
| [`og.Contact`](physics.md#record-Contact) | event | physics | — |
| [`og.Decal`](render.md#record-Decal) | component | render | — |
| [`og.DirectionalLight`](render.md#record-DirectionalLight) | component | render | — |
| [`og.Flipbook`](anim.md#record-Flipbook) | component | anim | — |
| [`og.FlipbookEvent`](anim.md#record-FlipbookEvent) | event | anim | — |
| [`og.Follow2D`](camera.md#record-Follow2D) | component | camera | — |
| [`og.FollowCamera`](camera.md#record-FollowCamera) | component | camera | — |
| [`og.game`](game.md#fn-game) | function | game | `og_game_game` |
| [`og.Gamepad`](input.md#record-Gamepad) | resource | input | — |
| [`og.IkChain`](anim.md#record-IkChain) | component | anim | — |
| [`og.Input`](input.md#record-Input) | resource | input | — |
| [`og.KeyDown`](input.md#record-KeyDown) | event | input | — |
| [`og.KeyUp`](input.md#record-KeyUp) | event | input | — |
| [`og.Light2D`](render.md#record-Light2D) | component | render | — |
| [`og.Lighting2D`](render.md#record-Lighting2D) | resource | render | — |
| [`og.load.animation`](load.md#fn-animation) | function | load | `og_load_animation` |
| [`og.load.bundle`](load.md#fn-bundle) | function | load | `og_load_bundle` |
| [`og.load.bundleInfo`](load.md#fn-bundleInfo) | function | load | `og_load_bundle_info` |
| [`og.load.defaultBundle`](load.md#fn-defaultBundle) | function | load | `og_load_default_bundle` |
| [`og.load.emitter`](load.md#fn-emitter) | function | load | `og_load_emitter` |
| [`og.load.error`](load.md#fn-error) | function | load | `og_load_error` |
| [`og.load.flipbook`](load.md#fn-flipbook) | function | load | `og_load_flipbook` |
| [`og.load.font`](load.md#fn-font) | function | load | `og_load_font` |
| [`og.load.fontInfo`](load.md#fn-fontInfo) | function | load | `og_load_font_info` |
| [`og.load.gltf`](load.md#fn-gltf) | function | load | `og_load_gltf` |
| [`og.load.gltfInfo`](load.md#fn-gltfInfo) | function | load | `og_load_gltf_info` |
| [`og.load.lut`](load.md#fn-lut) | function | load | `og_load_lut` |
| [`og.load.scene`](load.md#fn-scene) | function | load | `og_load_scene` |
| [`og.load.skin`](load.md#fn-skin) | function | load | `og_load_skin` |
| [`og.load.sound`](load.md#fn-sound) | function | load | `og_load_sound` |
| [`og.load.soundInfo`](load.md#fn-soundInfo) | function | load | `og_load_sound_info` |
| [`og.load.status`](load.md#fn-status) | function | load | `og_load_status` |
| [`og.load.texture`](load.md#fn-texture) | function | load | `og_load_texture` |
| [`og.load.tilemap`](load.md#fn-tilemap) | function | load | `og_load_tilemap` |
| [`og.MatrixWorld`](transform.md#record-MatrixWorld) | component | transform | — |
| [`og.Mesh`](render.md#record-Mesh) | component | render | — |
| [`og.Model`](load.md#record-Model) | component | load | — |
| [`og.nav.build`](nav.md#fn-build) | function | nav | `og_nav_build` |
| [`og.nav.cellAt`](nav.md#fn-cellAt) | function | nav | `og_nav_cell_at` |
| [`og.nav.findPath`](nav.md#fn-findPath) | function | nav | `og_nav_find_path` |
| [`og.nav.flowAt`](nav.md#fn-flowAt) | function | nav | `og_nav_flow_at` |
| [`og.nav.flowField`](nav.md#fn-flowField) | function | nav | `og_nav_flow_field` |
| [`og.nav.grid`](nav.md#fn-grid) | function | nav | `og_nav_grid` |
| [`og.nav.gridFromTilemap`](nav.md#fn-gridFromTilemap) | function | nav | `og_nav_grid_from_tilemap` |
| [`og.nav.load`](nav.md#fn-load) | function | nav | `og_nav_load` |
| [`og.nav.nearest`](nav.md#fn-nearest) | function | nav | `og_nav_nearest` |
| [`og.nav.pathOf`](nav.md#fn-pathOf) | function | nav | `og_nav_path_of` |
| [`og.nav.setCost`](nav.md#fn-setCost) | function | nav | `og_nav_set_cost` |
| [`og.nav.setFlowTargets`](nav.md#fn-setFlowTargets) | function | nav | `og_nav_set_flow_targets` |
| [`og.nav.setObstacle`](nav.md#fn-setObstacle) | function | nav | `og_nav_set_obstacle` |
| [`og.NavAgent`](nav.md#record-NavAgent) | component | nav | — |
| [`og.Occluder2D`](render.md#record-Occluder2D) | component | render | — |
| [`og.OrbitCamera`](camera.md#record-OrbitCamera) | component | camera | — |
| [`og.Parent`](transform.md#record-Parent) | component | transform | — |
| [`og.ParticleSystem`](render.md#record-ParticleSystem) | component | render | — |
| [`og.Perceivable`](ai.md#record-Perceivable) | component | ai | — |
| [`og.Perception`](ai.md#record-Perception) | component | ai | — |
| [`og.Physics`](physics.md#record-Physics) | resource | physics | — |
| [`og.physics.overlap`](physics.md#fn-overlap) | function | physics | `og_physics_overlap` |
| [`og.physics.raycast`](physics.md#fn-raycast) | function | physics | `og_physics_raycast` |
| [`og.physics.setMesh`](physics.md#fn-setMesh) | function | physics | `og_physics_set_mesh` |
| [`og.physics.shapeCast`](physics.md#fn-shapeCast) | function | physics | `og_physics_shape_cast` |
| [`og.PointerDown`](input.md#record-PointerDown) | event | input | — |
| [`og.PointerMove`](input.md#record-PointerMove) | event | input | — |
| [`og.Pointers`](input.md#record-Pointers) | resource | input | — |
| [`og.PointerUp`](input.md#record-PointerUp) | event | input | — |
| [`og.PointLight`](render.md#record-PointLight) | component | render | — |
| [`og.PostEffects`](render.md#record-PostEffects) | resource | render | — |
| [`og.render.boxGeometry`](render.md#fn-boxGeometry) | function | render | `og_render_box_geometry` |
| [`og.render.capabilities`](render.md#fn-capabilities) | function | render | `og_render_capabilities` |
| [`og.render.floatingText`](render.md#fn-floatingText) | function | render | `og_render_floating_text` |
| [`og.render.measureText`](render.md#fn-measureText) | function | render | `og_render_measure_text` |
| [`og.render.planeGeometry`](render.md#fn-planeGeometry) | function | render | `og_render_plane_geometry` |
| [`og.render.ribbon`](render.md#fn-ribbon) | function | render | `og_render_ribbon` |
| [`og.render.sphereGeometry`](render.md#fn-sphereGeometry) | function | render | `og_render_sphere_geometry` |
| [`og.render.sprite`](render.md#fn-sprite) | function | render | `og_render_sprite` |
| [`og.render.standardMaterial`](render.md#fn-standardMaterial) | function | render | `og_render_standard_material` |
| [`og.render.text`](render.md#fn-text) | function | render | `og_render_text` |
| [`og.render.updateMaterial`](render.md#fn-updateMaterial) | function | render | `og_render_update_material` |
| [`og.RenderSettings`](render.md#record-RenderSettings) | resource | render | — |
| [`og.RigidBody`](physics.md#record-RigidBody) | component | physics | — |
| [`og.Rng`](rng.md#record-Rng) | resource | rng | — |
| [`og.SensorEntered`](physics.md#record-SensorEntered) | event | physics | — |
| [`og.SensorExited`](physics.md#record-SensorExited) | event | physics | — |
| [`og.Skeleton`](anim.md#record-Skeleton) | component | anim | — |
| [`og.SoundEnded`](audio.md#record-SoundEnded) | event | audio | — |
| [`og.SoundStarted`](audio.md#record-SoundStarted) | event | audio | — |
| [`og.SpotLight`](render.md#record-SpotLight) | component | render | — |
| [`og.Sprite`](render.md#record-Sprite) | component | render | — |
| [`og.Steering`](nav.md#record-Steering) | component | nav | — |
| [`og.systems.cameras`](systems.md#value-cameras) | value | systems | — |
| [`og.Tilemap`](tilemap.md#record-Tilemap) | component | tilemap | — |
| [`og.tilemap.cellAt`](tilemap.md#fn-cellAt) | function | tilemap | `og_tilemap_cell_at` |
| [`og.tilemap.cellCenter`](tilemap.md#fn-cellCenter) | function | tilemap | `og_tilemap_cell_center` |
| [`og.tilemap.getTile`](tilemap.md#fn-getTile) | function | tilemap | `og_tilemap_get_tile` |
| [`og.tilemap.setTile`](tilemap.md#fn-setTile) | function | tilemap | `og_tilemap_set_tile` |
| [`og.Time`](time.md#record-Time) | resource | time | — |
| [`og.time.hitStop`](time.md#fn-hitStop) | function | time | `og_time_hit_stop` |
| [`og.time.setScale`](time.md#fn-setScale) | function | time | `og_time_set_scale` |
| [`og.Trail`](render.md#record-Trail) | component | render | — |
| [`og.Transform`](transform.md#record-Transform) | component | transform | — |
| [`og.transform.compose`](transform.md#fn-compose) | function | transform | `og_transform_compose` |
| [`og.transform.decompose`](transform.md#fn-decompose) | function | transform | `og_transform_decompose` |
| [`og.transform.fromAxisAngle`](transform.md#fn-fromAxisAngle) | function | transform | `og_transform_from_axis_angle` |
| [`og.transform.lookAt`](transform.md#fn-lookAt) | function | transform | `og_transform_look_at` |
| [`og.tween.active`](tween.md#fn-active) | function | tween | `og_tween_active` |
| [`og.tween.cancel`](tween.md#fn-cancel) | function | tween | `og_tween_cancel` |
| [`og.tween.cancelEntity`](tween.md#fn-cancelEntity) | function | tween | `og_tween_cancel_entity` |
| [`og.tween.flash`](tween.md#fn-flash) | function | tween | `og_tween_flash` |
| [`og.tween.parallel`](tween.md#fn-parallel) | function | tween | `og_tween_parallel` |
| [`og.tween.sequence`](tween.md#fn-sequence) | function | tween | `og_tween_sequence` |
| [`og.tween.squash`](tween.md#fn-squash) | function | tween | `og_tween_squash` |
| [`og.tween.to`](tween.md#fn-to) | function | tween | `og_tween_to` |
| [`og.TweenCompleted`](tween.md#record-TweenCompleted) | event | tween | — |
| [`og.ui.actionButton`](ui.md#fn-actionButton) | function | ui | `og_ui_action_button` |
| [`og.ui.alertDialog`](ui.md#fn-alertDialog) | function | ui | — |
| [`og.ui.anchor`](ui.md#fn-anchor) | function | ui | — |
| [`og.ui.back`](ui.md#fn-back) | function | ui | `og_ui_back` |
| [`og.ui.badge`](ui.md#fn-badge) | function | ui | `og_ui_badge` |
| [`og.ui.bounds`](ui.md#fn-bounds) | function | ui | `og_ui_bounds` |
| [`og.ui.breakpoint`](ui.md#fn-breakpoint) | function | ui | `og_ui_breakpoint` |
| [`og.ui.burst`](ui.md#fn-burst) | function | ui | `og_ui_burst` |
| [`og.ui.button`](ui.md#fn-button) | function | ui | `og_ui_button` |
| [`og.ui.checkbox`](ui.md#fn-checkbox) | function | ui | `og_ui_checkbox` |
| [`og.ui.checkboxList`](ui.md#fn-checkboxList) | function | ui | `og_ui_checkbox_list` |
| [`og.ui.chip`](ui.md#fn-chip) | function | ui | `og_ui_chip` |
| [`og.ui.close`](ui.md#fn-close) | function | ui | `og_ui_close` |
| [`og.ui.closeList`](ui.md#fn-closeList) | function | ui | `og_ui_close_list` |
| [`og.ui.closeModal`](ui.md#fn-closeModal) | function | ui | `og_ui_close_modal` |
| [`og.ui.column`](ui.md#fn-column) | function | ui | — |
| [`og.ui.configure`](ui.md#fn-configure) | function | ui | `og_ui_configure` |
| [`og.ui.confirmDialog`](ui.md#fn-confirmDialog) | function | ui | — |
| [`og.ui.cooldown`](ui.md#fn-cooldown) | function | ui | `og_ui_cooldown` |
| [`og.ui.counter`](ui.md#fn-counter) | function | ui | `og_ui_counter` |
| [`og.ui.cutscene`](ui.md#fn-cutscene) | function | ui | — |
| [`og.ui.dialogue`](ui.md#fn-dialogue) | function | ui | `og_ui_dialogue` |
| [`og.ui.dismissToast`](ui.md#fn-dismissToast) | function | ui | `og_ui_dismiss_toast` |
| [`og.ui.divider`](ui.md#fn-divider) | function | ui | `og_ui_divider` |
| [`og.ui.dpad`](ui.md#fn-dpad) | function | ui | `og_ui_dpad` |
| [`og.ui.dragging`](ui.md#fn-dragging) | function | ui | `og_ui_dragging` |
| [`og.ui.frameTheme`](ui.md#fn-frameTheme) | function | ui | `og_ui_frame_theme` |
| [`og.ui.frameViewport`](ui.md#fn-frameViewport) | function | ui | `og_ui_frame_viewport` |
| [`og.ui.grid`](ui.md#fn-grid) | function | ui | — |
| [`og.ui.gridItem`](ui.md#fn-gridItem) | function | ui | `og_ui_grid_item` |
| [`og.ui.hasIcon`](ui.md#fn-hasIcon) | function | ui | `og_ui_has_icon` |
| [`og.ui.icon`](ui.md#fn-icon) | function | ui | `og_ui_icon` |
| [`og.ui.idi`](ui.md#fn-idi) | function | ui | — |
| [`og.ui.idleHud`](ui.md#fn-idleHud) | function | ui | — |
| [`og.ui.image`](ui.md#fn-image) | function | ui | `og_ui_image` |
| [`og.ui.inventoryScreen`](ui.md#fn-inventoryScreen) | function | ui | — |
| [`og.ui.joystick`](ui.md#fn-joystick) | function | ui | `og_ui_joystick` |
| [`og.ui.keyValue`](ui.md#fn-keyValue) | function | ui | `og_ui_key_value` |
| [`og.ui.label`](ui.md#fn-label) | function | ui | `og_ui_label` |
| [`og.ui.leaderboardScreen`](ui.md#fn-leaderboardScreen) | function | ui | — |
| [`og.ui.levelSelect`](ui.md#fn-levelSelect) | function | ui | — |
| [`og.ui.list`](ui.md#fn-list) | function | ui | — |
| [`og.ui.listItem`](ui.md#fn-listItem) | function | ui | `og_ui_list_item` |
| [`og.ui.loadingScreen`](ui.md#fn-loadingScreen) | function | ui | — |
| [`og.ui.mainMenu`](ui.md#fn-mainMenu) | function | ui | — |
| [`og.ui.memory`](ui.md#fn-memory) | function | ui | `og_ui_memory` |
| [`og.ui.meter`](ui.md#fn-meter) | function | ui | `og_ui_meter` |
| [`og.ui.minimap`](ui.md#fn-minimap) | function | ui | `og_ui_minimap` |
| [`og.ui.modal`](ui.md#fn-modal) | function | ui | — |
| [`og.ui.navigate`](ui.md#fn-navigate) | function | ui | `og_ui_navigate` |
| [`og.ui.open`](ui.md#fn-open) | function | ui | `og_ui_open` |
| [`og.ui.openAnchor`](ui.md#fn-openAnchor) | function | ui | `og_ui_open_anchor` |
| [`og.ui.openGrid`](ui.md#fn-openGrid) | function | ui | `og_ui_open_grid` |
| [`og.ui.openList`](ui.md#fn-openList) | function | ui | `og_ui_open_list` |
| [`og.ui.openModal`](ui.md#fn-openModal) | function | ui | `og_ui_open_modal` |
| [`og.ui.openPanel`](ui.md#fn-openPanel) | function | ui | `og_ui_open_panel` |
| [`og.ui.openScroll`](ui.md#fn-openScroll) | function | ui | `og_ui_open_scroll` |
| [`og.ui.openTransition`](ui.md#fn-openTransition) | function | ui | `og_ui_open_transition` |
| [`og.ui.panel`](ui.md#fn-panel) | function | ui | — |
| [`og.ui.pauseMenu`](ui.md#fn-pauseMenu) | function | ui | — |
| [`og.ui.platformerHud`](ui.md#fn-platformerHud) | function | ui | — |
| [`og.ui.previewViewport`](ui.md#fn-previewViewport) | function | ui | `og_ui_preview_viewport` |
| [`og.ui.progress`](ui.md#fn-progress) | function | ui | `og_ui_progress` |
| [`og.ui.puzzleHud`](ui.md#fn-puzzleHud) | function | ui | — |
| [`og.ui.racingHud`](ui.md#fn-racingHud) | function | ui | — |
| [`og.ui.radio`](ui.md#fn-radio) | function | ui | `og_ui_radio` |
| [`og.ui.remember`](ui.md#fn-remember) | function | ui | `og_ui_remember` |
| [`og.ui.resultsScreen`](ui.md#fn-resultsScreen) | function | ui | — |
| [`og.ui.row`](ui.md#fn-row) | function | ui | — |
| [`og.ui.screenTransition`](ui.md#fn-screenTransition) | function | ui | `og_ui_screen_transition` |
| [`og.ui.scroll`](ui.md#fn-scroll) | function | ui | — |
| [`og.ui.seconds`](ui.md#fn-seconds) | function | ui | `og_ui_seconds` |
| [`og.ui.select`](ui.md#fn-select) | function | ui | `og_ui_select` |
| [`og.ui.setNavigation`](ui.md#fn-setNavigation) | function | ui | `og_ui_set_navigation` |
| [`og.ui.settingsScreen`](ui.md#fn-settingsScreen) | function | ui | — |
| [`og.ui.shooterHud`](ui.md#fn-shooterHud) | function | ui | — |
| [`og.ui.shopScreen`](ui.md#fn-shopScreen) | function | ui | — |
| [`og.ui.skeleton`](ui.md#fn-skeleton) | function | ui | `og_ui_skeleton` |
| [`og.ui.slider`](ui.md#fn-slider) | function | ui | `og_ui_slider` |
| [`og.ui.spacer`](ui.md#fn-spacer) | function | ui | `og_ui_spacer` |
| [`og.ui.spinner`](ui.md#fn-spinner) | function | ui | `og_ui_spinner` |
| [`og.ui.spotlight`](ui.md#fn-spotlight) | function | ui | `og_ui_spotlight` |
| [`og.ui.stepper`](ui.md#fn-stepper) | function | ui | `og_ui_stepper` |
| [`og.ui.tabs`](ui.md#fn-tabs) | function | ui | `og_ui_tabs` |
| [`og.ui.textField`](ui.md#fn-textField) | function | ui | `og_ui_text_field` |
| [`og.ui.theme`](ui.md#fn-theme) | function | ui | `og_ui_theme` |
| [`og.ui.tick`](ui.md#fn-tick) | function | ui | `og_ui_tick` |
| [`og.ui.toast`](ui.md#fn-toast) | function | ui | `og_ui_toast` |
| [`og.ui.toggle`](ui.md#fn-toggle) | function | ui | `og_ui_toggle` |
| [`og.ui.tooltip`](ui.md#fn-tooltip) | function | ui | `og_ui_tooltip` |
| [`og.ui.transition`](ui.md#fn-transition) | function | ui | — |
| [`og.ui.tutorialOverlay`](ui.md#fn-tutorialOverlay) | function | ui | — |
| [`og.ui.viewport`](ui.md#fn-viewport) | function | ui | `og_ui_viewport` |
| [`og.Velocity`](physics.md#record-Velocity) | component | physics | — |
| [`og.Wheel`](input.md#record-Wheel) | event | input | — |
| [`OverlapDesc`](physics.md#type-OverlapDesc) | desc struct | physics | — |
| [`Paint`](ui.md#type-Paint) | desc struct | ui | — |
| [`PanelDesc`](ui.md#type-PanelDesc) | desc struct | ui | — |
| [`ParticleEmitter`](render.md#type-ParticleEmitter) | handle | render | — |
| [`ParticleEmitter()`](render.md#column-ParticleEmitter) | column kind | render | — |
| [`PathDesc`](nav.md#type-PathDesc) | desc struct | nav | — |
| [`PauseMenuDesc`](ui.md#type-PauseMenuDesc) | desc struct | ui | — |
| [`PercentSize`](ui.md#type-PercentSize) | struct | ui | — |
| [`PickOperation`](probe.md#type-PickOperation) | struct | probe | — |
| [`PickResult`](probe.md#type-PickResult) | struct | probe | — |
| [`PlaneGeometryDesc`](render.md#type-PlaneGeometryDesc) | desc struct | render | — |
| [`PlatformerHudDesc`](ui.md#type-PlatformerHudDesc) | desc struct | ui | — |
| [`PLAYTEST_MAX_TICKS`](probe.md#const-PLAYTEST_MAX_TICKS) | constant | probe | — |
| [`PointerAction`](input.md#type-PointerAction) | enum | input | — |
| [`PointerButton`](input.md#type-PointerButton) | enum | input | — |
| [`PointerInput`](input.md#type-PointerInput) | struct | input | — |
| [`Pointers.count`](input.md#member-Pointers-count) | method | input | `og_pointers_count` |
| [`Pointers.down`](input.md#member-Pointers-down) | method | input | `og_pointers_down` |
| [`Pointers.position`](input.md#member-Pointers-position) | method | input | `og_pointers_position` |
| [`Pointers.pressed`](input.md#member-Pointers-pressed) | method | input | `og_pointers_pressed` |
| [`Pointers.released`](input.md#member-Pointers-released) | method | input | `og_pointers_released` |
| [`Pointers.start`](input.md#member-Pointers-start) | method | input | `og_pointers_start` |
| [`Presence`](ui.md#type-Presence) | struct | ui | — |
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
| [`PROFILE_2D`](physics.md#const-PROFILE_2D) | constant | physics | — |
| [`PROFILE_3D`](physics.md#const-PROFILE_3D) | constant | physics | — |
| [`ProgressDesc`](ui.md#type-ProgressDesc) | desc struct | ui | — |
| [`PuzzleHudDesc`](ui.md#type-PuzzleHudDesc) | desc struct | ui | — |
| [`Quaternion`](math.md#type-Quaternion) | struct | math | — |
| [`Quaternion()`](reflect.md#column-Quaternion) | column kind | reflect | — |
| [`Query`](world.md#type-Query) | interface | world | — |
| [`Query.archetypes`](world.md#member-Query-archetypes) | method | world | `og_query_archetypes` |
| [`Query.count`](world.md#member-Query-count) | method | world | `og_query_count` |
| [`QueryDesc`](world.md#type-QueryDesc) | desc struct | world | — |
| [`QueryFilter`](physics.md#type-QueryFilter) | desc struct | physics | — |
| [`RacingHudDesc`](ui.md#type-RacingHudDesc) | desc struct | ui | — |
| [`RadioDesc`](ui.md#type-RadioDesc) | desc struct | ui | — |
| [`Ray`](math.md#type-Ray) | struct | math | — |
| [`RaycastDesc`](physics.md#type-RaycastDesc) | desc struct | physics | — |
| [`RaycastHit`](physics.md#type-RaycastHit) | struct | physics | — |
| [`ReloadReport`](probe.md#type-ReloadReport) | struct | probe | — |
| [`RenderCapabilities`](render.md#type-RenderCapabilities) | struct | render | — |
| [`RenderOptions`](render.md#type-RenderOptions) | desc struct | render | — |
| [`RenderTier`](render.md#type-RenderTier) | enum | render | — |
| [`REPLAY_SEED_PARAM`](probe.md#const-REPLAY_SEED_PARAM) | constant | probe | — |
| [`Response`](ui.md#type-Response) | struct | ui | — |
| [`ResultsDesc`](ui.md#type-ResultsDesc) | desc struct | ui | — |
| [`RibbonDesc`](render.md#type-RibbonDesc) | desc struct | render | — |
| [`RibbonPoint`](render.md#type-RibbonPoint) | desc struct | render | — |
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
| [`RootMotion`](anim.md#type-RootMotion) | enum | anim | — |
| [`RootMotion()`](anim.md#column-RootMotion) | column kind | anim | — |
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
| [`Scheme`](ui.md#type-Scheme) | enum | ui | — |
| [`ScreenTransitionDesc`](ui.md#type-ScreenTransitionDesc) | desc struct | ui | — |
| [`ScreenTransitionResult`](ui.md#type-ScreenTransitionResult) | struct | ui | — |
| [`ScreenWipe`](ui.md#type-ScreenWipe) | enum | ui | — |
| [`ScrollDesc`](ui.md#type-ScrollDesc) | desc struct | ui | — |
| [`SelectDesc`](ui.md#type-SelectDesc) | desc struct | ui | — |
| [`SetTileDesc`](tilemap.md#type-SetTileDesc) | desc struct | tilemap | — |
| [`SettingsChanged`](ui.md#type-SettingsChanged) | callback | ui | — |
| [`SettingsDesc`](ui.md#type-SettingsDesc) | desc struct | ui | — |
| [`SettingsResult`](ui.md#type-SettingsResult) | struct | ui | — |
| [`SHAPE_BOX`](physics.md#const-SHAPE_BOX) | constant | physics | — |
| [`SHAPE_CAPSULE`](physics.md#const-SHAPE_CAPSULE) | constant | physics | — |
| [`SHAPE_CONVEX`](physics.md#const-SHAPE_CONVEX) | constant | physics | — |
| [`SHAPE_CYLINDER`](physics.md#const-SHAPE_CYLINDER) | constant | physics | — |
| [`SHAPE_SPHERE`](physics.md#const-SHAPE_SPHERE) | constant | physics | — |
| [`SHAPE_TRIMESH`](physics.md#const-SHAPE_TRIMESH) | constant | physics | — |
| [`ShapeCastDesc`](physics.md#type-ShapeCastDesc) | desc struct | physics | — |
| [`ShapeCastHit`](physics.md#type-ShapeCastHit) | struct | physics | — |
| [`ShapeDesc`](physics.md#type-ShapeDesc) | desc struct | physics | — |
| [`ShooterHudDesc`](ui.md#type-ShooterHudDesc) | desc struct | ui | — |
| [`ShopDesc`](ui.md#type-ShopDesc) | desc struct | ui | — |
| [`ShopItem`](ui.md#type-ShopItem) | desc struct | ui | — |
| [`ShopResult`](ui.md#type-ShopResult) | struct | ui | — |
| [`Side`](render.md#type-Side) | enum | render | — |
| [`Size`](ui.md#type-Size) | union | ui | — |
| [`SizeRange`](ui.md#type-SizeRange) | struct | ui | — |
| [`SkeletonAsset`](anim.md#type-SkeletonAsset) | handle | anim | — |
| [`SkeletonAsset()`](anim.md#column-SkeletonAsset) | column kind | anim | — |
| [`SkeletonDesc`](ui.md#type-SkeletonDesc) | desc struct | ui | — |
| [`Skin`](load.md#type-Skin) | handle | load | — |
| [`SkinPart`](ui.md#type-SkinPart) | enum | ui | — |
| [`SliderDesc`](ui.md#type-SliderDesc) | desc struct | ui | — |
| [`SliderResult`](ui.md#type-SliderResult) | struct | ui | — |
| [`SlotMove`](ui.md#type-SlotMove) | struct | ui | — |
| [`SnapshotInfo`](probe.md#type-SnapshotInfo) | struct | probe | — |
| [`Sound`](load.md#type-Sound) | handle | load | — |
| [`Sound()`](load.md#column-Sound) | column kind | load | — |
| [`SOUND_FINISHED`](audio.md#const-SOUND_FINISHED) | constant | audio | — |
| [`SOUND_REMOVED`](audio.md#const-SOUND_REMOVED) | constant | audio | — |
| [`SOUND_REPLACED`](audio.md#const-SOUND_REPLACED) | constant | audio | — |
| [`SOUND_STOPPED`](audio.md#const-SOUND_STOPPED) | constant | audio | — |
| [`SoundInfo`](load.md#type-SoundInfo) | struct | load | — |
| [`SpawnDesc`](world.md#type-SpawnDesc) | desc struct | world | — |
| [`SphereGeometryDesc`](render.md#type-SphereGeometryDesc) | desc struct | render | — |
| [`SpotlightDesc`](ui.md#type-SpotlightDesc) | desc struct | ui | — |
| [`SpriteBlend`](render.md#type-SpriteBlend) | enum | render | — |
| [`SpriteBlend()`](render.md#column-SpriteBlend) | column kind | render | — |
| [`SpriteDesc`](render.md#type-SpriteDesc) | desc struct | render | — |
| [`SquashDesc`](tween.md#type-SquashDesc) | desc struct | tween | — |
| [`StandardMaterialDesc`](render.md#type-StandardMaterialDesc) | desc struct | render | — |
| [`StartFn`](game.md#type-StartFn) | callback | game | — |
| [`StatRow`](ui.md#type-StatRow) | desc struct | ui | — |
| [`StepperDesc`](ui.md#type-StepperDesc) | desc struct | ui | — |
| [`StepperResult`](ui.md#type-StepperResult) | struct | ui | — |
| [`StripBlend`](render.md#type-StripBlend) | enum | render | — |
| [`StripBlend()`](render.md#column-StripBlend) | column kind | render | — |
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
| [`TabsDesc`](ui.md#type-TabsDesc) | desc struct | ui | — |
| [`TextAlign`](ui.md#type-TextAlign) | enum | ui | — |
| [`TextDesc`](render.md#type-TextDesc) | desc struct | render | — |
| [`TextMeasureDesc`](render.md#type-TextMeasureDesc) | desc struct | render | — |
| [`TextMetrics`](render.md#type-TextMetrics) | struct | render | — |
| [`Texture`](render.md#type-Texture) | handle | render | — |
| [`Texture()`](render.md#column-Texture) | column kind | render | — |
| [`ThemeColor`](ui.md#type-ThemeColor) | enum | ui | — |
| [`TileCell`](tilemap.md#type-TileCell) | struct | tilemap | — |
| [`TileLayerInfo`](tilemap.md#type-TileLayerInfo) | struct | tilemap | — |
| [`TilemapAsset`](tilemap.md#type-TilemapAsset) | handle | tilemap | — |
| [`TilemapAsset()`](tilemap.md#column-TilemapAsset) | column kind | tilemap | — |
| [`TilemapGridDesc`](nav.md#type-TilemapGridDesc) | desc struct | nav | — |
| [`TilemapInfo`](tilemap.md#type-TilemapInfo) | struct | tilemap | — |
| [`TileObject`](tilemap.md#type-TileObject) | struct | tilemap | — |
| [`ToastDesc`](ui.md#type-ToastDesc) | desc struct | ui | — |
| [`ToggleResult`](ui.md#type-ToggleResult) | struct | ui | — |
| [`Tone`](ui.md#type-Tone) | enum | ui | — |
| [`TransitionDesc`](ui.md#type-TransitionDesc) | desc struct | ui | — |
| [`TransitionKind`](ui.md#type-TransitionKind) | enum | ui | — |
| [`TutorialDesc`](ui.md#type-TutorialDesc) | desc struct | ui | — |
| [`TutorialResult`](ui.md#type-TutorialResult) | struct | ui | — |
| [`TutorialStep`](ui.md#type-TutorialStep) | desc struct | ui | — |
| [`Tween`](tween.md#type-Tween) | handle | tween | — |
| [`Tween()`](tween.md#column-Tween) | column kind | tween | — |
| [`TweenDesc`](tween.md#type-TweenDesc) | desc struct | tween | — |
| [`TweenGroupDesc`](tween.md#type-TweenGroupDesc) | desc struct | tween | — |
| [`TypeStep`](ui.md#type-TypeStep) | struct | ui | — |
| [`TypeStepName`](ui.md#type-TypeStepName) | enum | ui | — |
| [`u32()`](reflect.md#column-u32) | column kind | reflect | — |
| [`UiAction`](ui.md#type-UiAction) | callback | ui | — |
| [`UiBuild`](ui.md#type-UiBuild) | callback | ui | — |
| [`UiDesc`](ui.md#type-UiDesc) | desc struct | ui | — |
| [`UiDump`](probe.md#type-UiDump) | struct | probe | — |
| [`UiElement`](probe.md#type-UiElement) | struct | probe | — |
| [`UiItemBuild`](ui.md#type-UiItemBuild) | callback | ui | — |
| [`UiRect`](ui.md#type-UiRect) | struct | ui | — |
| [`UiSounds`](ui.md#type-UiSounds) | desc struct | ui | — |
| [`UiTheme`](ui.md#type-UiTheme) | struct | ui | — |
| [`Vector2`](math.md#type-Vector2) | struct | math | — |
| [`Vector2()`](reflect.md#column-Vector2) | column kind | reflect | — |
| [`Vector3`](math.md#type-Vector3) | struct | math | — |
| [`Vector3()`](reflect.md#column-Vector3) | column kind | reflect | — |
| [`Viewport`](ui.md#type-Viewport) | struct | ui | — |
| [`WheelInput`](input.md#type-WheelInput) | struct | input | — |
| [`WipeSide`](ui.md#type-WipeSide) | enum | ui | — |
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
