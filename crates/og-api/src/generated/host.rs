// Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate.
//! The host contract, generated from the IDL: one trait per module with
//! callables, each method a binding symbol with its wire signature (see
//! `og_api::gen::host` for the wire forms). `og-host-web` and
//! `og-host-node` implement every trait, and their generated glue calls
//! the methods, so a member missing on either host is a build error.

/// What aliases engine memory, chosen per host.
pub trait Wire {
    /// A typed array the host hands out (`ArchetypeView.entities`).
    type Array;
    /// A component's column views (`ArchetypeView.column`).
    type Views;
    /// A JavaScript function the engine calls back.
    type Function;
}

/// Engine lifecycle: what the TypeScript loaders drive.
pub trait Lifecycle: Wire {
    /// Install the function the engine calls for systems, commands and facts: `dispatcher(kind, engine, id, query, tick, dt, elapsed, payload)` (payload: a command's arguments as a JSON list) returns `""` on success, the error otherwise (`kind`: 0 system, 1 command, 2 facts, whose answer is `"="` then JSON).
    fn og_host_set_dispatcher(&self, dispatcher: Self::Function);
    /// Assemble a world from a declared game (`og.game`) and the declarations made so far. `options` is JSON `{ seed? }`; the answer is a Result of the engine id, which is also the world handle.
    fn og_engine_create(&self, game: u32, options: String) -> String;
    /// Drop an engine and everything it owns. False for an unknown engine.
    fn og_engine_destroy(&self, engine: u32) -> bool;
    /// Take fixed steps regardless of the wall clock: a Result of the tick reached, or the first failing system with its tick.
    fn og_engine_advance(&self, engine: u32, steps: u32) -> String;
    /// Run a registered command action now, between ticks (`args` is a JSON list of strings). The scheduled channel is `og.commands.schedule`.
    fn og_engine_command(&self, engine: u32, action: String, args: String) -> String;
    /// One display frame: feed `elapsed_ms` of wall clock to the fixed-tick accumulator (unless paused), take the ticks it allows up to `GameDesc.maxCatchUp`, then render when a renderer is attached. JSON `{ ok, ticks, tick, rendered }` or a refusal.
    fn og_engine_frame(&self, engine: u32, elapsed_ms: f64) -> String;
    /// Write some fields of a resource: how the runtime makes `world.resource(R).value` a live value. A Result.
    fn og_engine_write_resource(&self, world: u32, resource: String, value: String) -> String;
}

/// `reflect`: Reflection: components and resources are declared once with their field layout and metadata, and registered by name.
pub trait ReflectApi: Wire {
    /// `defineComponent`: Declare a component: plain data stored per entity in engine memory, one column per field.
    fn og_reflect_define_component(&self, name: String, fields: String) -> String;
    /// `defineResource`: Declare a resource: a reflected singleton, such as a tuning table. Its fields start at their defaults.
    fn og_reflect_define_resource(&self, name: String, fields: String) -> String;
}

/// `world`: The world: an archetype ECS. Entities are generational handles; components are columns in engine memory.
pub trait WorldApi: Wire {
    /// `World.tick`: Fixed steps taken since the world was created.
    fn og_world_get_tick(&self, world: u32) -> u32;
    /// `World.spawn`: Spawn an entity.
    fn og_world_spawn(&self, world: u32, desc: Option<String>) -> String;
    /// `World.despawn`: Despawn an entity and its descendants.
    fn og_world_despawn(&self, world: u32, entity: f64) -> String;
    /// `World.alive`: Whether an entity handle is live.
    fn og_world_alive(&self, world: u32, entity: f64) -> bool;
    /// `World.has`: Whether an entity has a component.
    fn og_world_has(&self, world: u32, entity: f64, component: String) -> bool;
    /// `World.insert`: Add a component to an entity, or overwrite it when present. A structural change.
    fn og_world_insert(&self, world: u32, entity: f64, component: String, value: Option<String>) -> String;
    /// `World.remove`: Remove a component from an entity. A structural change.
    fn og_world_remove(&self, world: u32, entity: f64, component: String) -> String;
    /// `World.get`: Copy one entity's component value. For bulk work use a system's typed views.
    fn og_world_get(&self, world: u32, entity: f64, component: String) -> String;
    /// `World.set`: Write some fields of one entity's component.
    fn og_world_set(&self, world: u32, entity: f64, component: String, value: String) -> String;
    /// `World.resource`: A resource's live value. Inside a system, the resource must be among the system's reads or writes.
    fn og_world_resource(&self, world: u32, resource: String) -> String;
    /// `World.find`: The entity with a name.
    fn og_world_find(&self, world: u32, name: String) -> String;
    /// `World.nameOf`: An entity's name.
    fn og_world_name_of(&self, world: u32, entity: f64) -> String;
    /// `World.query`: A query over the current archetypes, for commands and callbacks; systems receive theirs.
    fn og_world_query(&self, world: u32, desc: Option<String>) -> String;
    /// `World.events`: The events of one type emitted during the previous tick and this one, oldest first.
    fn og_world_events(&self, world: u32, event: String) -> String;
    /// `World.emit`: Emit an event; readers see it this tick and the next.
    fn og_world_emit(&self, world: u32, event: String, payload: String) -> String;
    /// `World.instantiate`: Spawn a loaded scene or glTF asset into the world as a tree of entities.
    fn og_world_instantiate(&self, world: u32, asset: f64, desc: Option<String>) -> String;
    /// `Query.archetypes`: The matched archetypes this tick.
    fn og_query_archetypes(&self, query: u32) -> String;
    /// `Query.count`: The number of matched entities.
    fn og_query_count(&self, query: u32) -> u32;
    /// `ArchetypeView.count`: The number of entities in the archetype.
    fn og_archetype_view_get_count(&self, archetype_view: u32) -> u32;
    /// `ArchetypeView.tick`: The tick this view belongs to.
    fn og_archetype_view_get_tick(&self, archetype_view: u32) -> u32;
    /// `ArchetypeView.entities`: The entities as (index, generation) pairs: entity i is at 2i and 2i + 1.
    fn og_archetype_view_get_entities(&self, archetype_view: u32) -> Self::Array;
    /// `ArchetypeView.entity`: The handle of entity i.
    fn og_archetype_view_entity(&self, archetype_view: u32, index: u32) -> f64;
    /// `ArchetypeView.has`: Whether the archetype carries a component.
    fn og_archetype_view_has(&self, archetype_view: u32, component: String) -> bool;
    /// `ArchetypeView.column`: The typed views of a component's fields: one typed array per field, `count * stride` long.
    fn og_archetype_view_column(&self, archetype_view: u32, component: String) -> Self::Views;
}

/// `system`: Systems: functions the engine calls once per tick with their matched archetypes.
pub trait SystemApi: Wire {
    /// `System.name`: The system's unique name.
    fn og_system_get_name(&self, system: u32) -> String;
    /// `System.reads`: What it reads.
    fn og_system_get_reads(&self, system: u32) -> String;
    /// `System.writes`: What it writes.
    fn og_system_get_writes(&self, system: u32) -> String;
    /// `defineSystem`: Declare a system.
    fn og_system_define_system(&self, name: String, desc: String, run: u32) -> u32;
}

/// `game`: The game entry: what the bundle's default export declares.
pub trait GameApi: Wire {
    /// `Game.tick`: The fixed tick rate in hertz.
    fn og_game_get_tick(&self, game: u32) -> u32;
    /// `Game.systems`: The systems in declaration order.
    fn og_game_get_systems(&self, game: u32) -> String;
    /// `og.game`: Declare the game. The bundle's default export is its result.
    fn og_game_game(&self, desc: Option<String>) -> u32;
}

/// `transform`: Transforms in the three.js vocabulary: position, quaternion and scale, composed into matrixWorld down the parent chain.
pub trait TransformApi: Wire {
    /// `og.transform.compose`: A matrix from position, rotation and scale, as three.js Matrix4.compose.
    fn og_transform_compose(&self, position: String, quaternion: String, scale: String) -> String;
    /// `og.transform.decompose`: Position, rotation and scale from a matrix, as three.js Matrix4.decompose.
    fn og_transform_decompose(&self, matrix: String) -> String;
    /// `og.transform.lookAt`: The rotation that points -Z from an eye towards a target, as three.js Object3D.lookAt for cameras.
    fn og_transform_look_at(&self, eye: String, target: String, up: Option<String>) -> String;
    /// `og.transform.fromAxisAngle`: A rotation about an axis, as three.js Quaternion.setFromAxisAngle.
    fn og_transform_from_axis_angle(&self, axis: String, angle: f64) -> String;
}

/// `rng`: Deterministic randomness. Every random draw comes from an Rng stream; the sequence for a seed is identical on every host and in the Web scaffold's rng.ts.
pub trait RngApi: Wire {
    /// `RngStream.seed`: The stream's derived seed.
    fn og_rng_stream_get_seed(&self, rng_stream: u32) -> u32;
    /// `RngStream.next`: Uniform in [0, 1).
    fn og_rng_stream_next(&self, rng_stream: u32) -> f64;
    /// `RngStream.range`: Uniform in [low, high).
    fn og_rng_stream_range(&self, rng_stream: u32, low: f64, high: f64) -> f64;
    /// `RngStream.integer`: A uniform integer in [low, high].
    fn og_rng_stream_integer(&self, rng_stream: u32, low: i32, high: i32) -> i32;
    /// `RngStream.chance`: True with probability p.
    fn og_rng_stream_chance(&self, rng_stream: u32, p: f64) -> bool;
    /// `RngStream.fork`: A fresh stream on a seed derived from this stream's seed and a label.
    fn og_rng_stream_fork(&self, rng_stream: u32, label: String) -> u32;
    /// `Rng.next`: Uniform in [0, 1).
    fn og_rng_next(&self, world: u32) -> f64;
    /// `Rng.range`: Uniform in [low, high).
    fn og_rng_range(&self, world: u32, low: f64, high: f64) -> f64;
    /// `Rng.integer`: A uniform integer in [low, high].
    fn og_rng_integer(&self, world: u32, low: i32, high: i32) -> i32;
    /// `Rng.chance`: True with probability p.
    fn og_rng_chance(&self, world: u32, p: f64) -> bool;
    /// `Rng.fork`: A stream for a subsystem, derived from the seed and a label, so drawing from it leaves the main sequence alone.
    fn og_rng_fork(&self, world: u32, label: String) -> u32;
}

/// `input`: Input as the game sees it: the Input resource for polling, events for edges.
pub trait InputApi: Wire {
    /// `Input.down`: Whether a key is held.
    fn og_input_down(&self, world: u32, key: String) -> bool;
    /// `Input.pressed`: Whether a key went down this tick.
    fn og_input_pressed(&self, world: u32, key: String) -> bool;
    /// `Input.released`: Whether a key went up this tick.
    fn og_input_released(&self, world: u32, key: String) -> bool;
    /// `Input.axis`: -1, 0 or 1 from two keys, such as `axis("KeyA", "KeyD")`.
    fn og_input_axis(&self, world: u32, negative: String, positive: String) -> f64;
    /// `Input.buttonDown`: Whether a pointer button is held.
    fn og_input_button_down(&self, world: u32, button: String) -> bool;
    /// `Input.buttonPressed`: Whether a pointer button went down this tick.
    fn og_input_button_pressed(&self, world: u32, button: String) -> bool;
}

/// `camera`: The main camera, as a resource with three.js PerspectiveCamera and OrthographicCamera parameter names.
pub trait CameraApi: Wire {
    /// `Camera.lookAt`: Rotate the camera to look at a point, as three.js Camera.lookAt.
    fn og_camera_look_at(&self, world: u32, target: String);
    /// `Camera.worldToScreen`: Where a world point lands on the play surface.
    fn og_camera_world_to_screen(&self, world: u32, point: String) -> String;
    /// `Camera.screenToRay`: The ray through a point of the play surface, for picking.
    fn og_camera_screen_to_ray(&self, world: u32, x: f64, y: f64) -> String;
}

/// `render`: What is drawn: meshes with materials. Draws are instanced by (geometry, material) automatically; there is no per-object draw call.
pub trait RenderApi: Wire {
    /// `og.render.standardMaterial`: Create a standard material.
    fn og_render_standard_material(&self, world: u32, desc: Option<String>) -> String;
    /// `og.render.updateMaterial`: Change some parameters of a standard material.
    fn og_render_update_material(&self, world: u32, material: f64, desc: Option<String>) -> String;
    /// `og.render.capabilities`: The renderer's tier and limits on this device.
    fn og_render_capabilities(&self, world: u32) -> String;
}

/// `load`: Asset loading. Paths are relative to the project and read cooked output; a load returns a handle at once and completes at a tick boundary.
pub trait LoadApi: Wire {
    /// `og.load.gltf`: Load a glTF (.gltf or .glb) through the cook, as three.js GLTFLoader.load.
    fn og_load_gltf(&self, world: u32, path: String, desc: Option<String>) -> String;
    /// `og.load.scene`: Load a scene (.ogt).
    fn og_load_scene(&self, world: u32, path: String) -> String;
    /// `og.load.status`: Where a load is.
    fn og_load_status(&self, world: u32, asset: f64) -> String;
    /// `og.load.gltfInfo`: What a ready glTF holds.
    fn og_load_gltf_info(&self, world: u32, asset: f64) -> String;
}

/// `commands`: The command channel: actions scheduled on simulation time, from `og run --commands "t:action[:args];..."` or the probe's commands().
pub trait CommandsApi: Wire {
    /// `og.commands.register`: Register an action into the command channel. GameDesc.commands does this for a game's own actions.
    fn og_commands_register(&self, name: String, handler: u32, desc: Option<String>) -> String;
    /// `og.commands.parse`: Parse a script without scheduling it.
    fn og_commands_parse(&self, script: String) -> String;
    /// `og.commands.schedule`: Schedule a script on simulation time, from the current tick.
    fn og_commands_schedule(&self, world: u32, script: String) -> String;
}

/// `probe`: The probe: RuntimeProbe protocol 5 (origin/packages/contracts/src/probe.ts) served natively, plus the engine's wire members.
pub trait ProbeApi: Wire {
    /// `RuntimeProbe.version`: PROBE_VERSION.
    fn og_runtime_probe_get_version(&self, runtime_probe: u32) -> u32;
    /// `RuntimeProbe.tick`: Fixed steps the world has taken since load.
    fn og_runtime_probe_tick(&self, runtime_probe: u32) -> u32;
    /// `RuntimeProbe.state`: A JSON snapshot: the fixed timings head, then the game's own facts.
    fn og_runtime_probe_state(&self, runtime_probe: u32) -> String;
    /// `RuntimeProbe.advance`: Take whole fixed steps regardless of the wall clock.
    fn og_runtime_probe_advance(&self, runtime_probe: u32, steps: u32) -> u32;
    /// `RuntimeProbe.pause`: Hold the wall-clock loop; advance still steps.
    fn og_runtime_probe_pause(&self, runtime_probe: u32) -> bool;
    /// `RuntimeProbe.resume`: Release the wall-clock loop.
    fn og_runtime_probe_resume(&self, runtime_probe: u32) -> bool;
    /// `RuntimeProbe.input`: One event through the input layer's own path.
    fn og_runtime_probe_input(&self, runtime_probe: u32, event: String) -> bool;
    /// `RuntimeProbe.accepted`: Page events (keyboard, pointer, wheel) the input layer accepted since load.
    fn og_runtime_probe_accepted(&self, runtime_probe: u32) -> u32;
    /// `RuntimeProbe.pick`: What is under a point of the play surface.
    fn og_runtime_probe_pick(&self, runtime_probe: u32, x: f64, y: f64) -> String;
    /// `RuntimeProbe.entity`: One entity's published parameters: its tunable fields as `Component.field` keys.
    fn og_runtime_probe_entity(&self, runtime_probe: u32, id: String) -> String;
    /// `RuntimeProbe.bounds`: The ranges of one entity's tunable fields, from the schema.
    fn og_runtime_probe_bounds(&self, runtime_probe: u32, id: String) -> String;
    /// `RuntimeProbe.patch`: Move one tunable field for this run only; it applies within one frame. The probe never writes project files.
    fn og_runtime_probe_patch(&self, runtime_probe: u32, id: String, parameter: String, value: String) -> bool;
    /// `RuntimeProbe.run`: Run a program of the members above in order, stopping at the first refusal.
    fn og_runtime_probe_run(&self, runtime_probe: u32, program: String) -> String;
    /// `EngineProbe.hello`: Who is speaking: protocol, wire, engine version, tick rate, renderer tier, build.
    fn og_engine_probe_hello(&self, engine_probe: u32) -> String;
    /// `EngineProbe.capture`: The frame rendered at the current tick, read back offscreen.
    fn og_engine_probe_capture(&self, engine_probe: u32, width: Option<u32>, height: Option<u32>) -> String;
    /// `EngineProbe.sweep`: A filmstrip over one tunable: per value, restore, patch, advance, capture.
    fn og_engine_probe_sweep(&self, engine_probe: u32, desc: Option<String>) -> String;
    /// `EngineProbe.reload`: Hot reload: snapshot, swap the game bundle, re-register components by name, restore.
    fn og_engine_probe_reload(&self, engine_probe: u32) -> String;
    /// `EngineProbe.snapshot`: Snapshot the whole world in the running process.
    fn og_engine_probe_snapshot(&self, engine_probe: u32) -> String;
    /// `EngineProbe.restore`: Restore a snapshot taken by this process.
    fn og_engine_probe_restore(&self, engine_probe: u32, id: String) -> bool;
    /// `EngineProbe.dump`: A world, entity or UI dump, as the command channel's dump actions write.
    fn og_engine_probe_dump(&self, engine_probe: u32, desc: Option<String>) -> String;
    /// `EngineProbe.commands`: Schedule a command script on simulation time, as `og run --commands`.
    fn og_engine_probe_commands(&self, engine_probe: u32, script: String) -> String;
}

/// Every trait: a host is a type that implements all of them.
pub trait Host: Lifecycle + ReflectApi + WorldApi + SystemApi + GameApi + TransformApi + RngApi + InputApi + CameraApi + RenderApi + LoadApi + CommandsApi + ProbeApi {}

impl<T: Lifecycle + ReflectApi + WorldApi + SystemApi + GameApi + TransformApi + RngApi + InputApi + CameraApi + RenderApi + LoadApi + CommandsApi + ProbeApi> Host for T {}

/// Every binding symbol, in binding-table order (`bindings.json` ids).
pub const SYMBOLS: &[&str] = &[
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
    "og_input_button_down",
    "og_input_button_pressed",
    "og_camera_look_at",
    "og_camera_world_to_screen",
    "og_camera_screen_to_ray",
    "og_render_standard_material",
    "og_render_update_material",
    "og_render_capabilities",
    "og_load_gltf",
    "og_load_scene",
    "og_load_status",
    "og_load_gltf_info",
    "og_commands_register",
    "og_commands_parse",
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
];

/// One field of an engine-owned record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldDecl {
    pub name: &'static str,
    /// The column kind (components, resources) or the type (events).
    pub column: &'static str,
    /// The default value as JSON.
    pub default: &'static str,
    pub readonly: bool,
    pub tunable: bool,
}

/// An engine-owned component, resource or event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecordDecl {
    /// `component`, `resource` or `event`.
    pub kind: &'static str,
    /// The IDL name, the TypeScript token's name on `og`.
    pub name: &'static str,
    /// The schema name it registers under.
    pub schema: &'static str,
    pub fields: &'static [FieldDecl],
}

/// Every engine-owned record, in declaration order.
pub const RECORDS: &[RecordDecl] = &[
    RecordDecl {
        kind: "component",
        name: "Transform",
        schema: "og.Transform",
        fields: &[
            FieldDecl {
                name: "position",
                column: "Vector3",
                default: "{\"x\":0,\"y\":0,\"z\":0}",
                readonly: false,
                tunable: false,
            },
            FieldDecl {
                name: "quaternion",
                column: "Quaternion",
                default: "{\"x\":0,\"y\":0,\"z\":0,\"w\":1}",
                readonly: false,
                tunable: false,
            },
            FieldDecl {
                name: "scale",
                column: "Vector3",
                default: "{\"x\":1,\"y\":1,\"z\":1}",
                readonly: false,
                tunable: false,
            },
        ],
    },
    RecordDecl {
        kind: "component",
        name: "Parent",
        schema: "og.Parent",
        fields: &[
            FieldDecl {
                name: "entity",
                column: "Entity",
                default: "0",
                readonly: false,
                tunable: false,
            },
        ],
    },
    RecordDecl {
        kind: "component",
        name: "MatrixWorld",
        schema: "og.MatrixWorld",
        fields: &[
            FieldDecl {
                name: "matrix",
                column: "Matrix4",
                default: "{\"elements\":[1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1]}",
                readonly: true,
                tunable: false,
            },
        ],
    },
    RecordDecl {
        kind: "resource",
        name: "Time",
        schema: "og.Time",
        fields: &[
            FieldDecl {
                name: "tick",
                column: "u32",
                default: "0",
                readonly: true,
                tunable: false,
            },
            FieldDecl {
                name: "dt",
                column: "f32",
                default: "0",
                readonly: true,
                tunable: false,
            },
            FieldDecl {
                name: "elapsed",
                column: "f64",
                default: "0",
                readonly: true,
                tunable: false,
            },
            FieldDecl {
                name: "rate",
                column: "u32",
                default: "0",
                readonly: true,
                tunable: false,
            },
        ],
    },
    RecordDecl {
        kind: "resource",
        name: "Rng",
        schema: "og.Rng",
        fields: &[
            FieldDecl {
                name: "seed",
                column: "u32",
                default: "0",
                readonly: true,
                tunable: false,
            },
            FieldDecl {
                name: "state",
                column: "u32",
                default: "0",
                readonly: true,
                tunable: false,
            },
        ],
    },
    RecordDecl {
        kind: "resource",
        name: "Input",
        schema: "og.Input",
        fields: &[
            FieldDecl {
                name: "pointer",
                column: "Vector2",
                default: "{\"x\":0,\"y\":0}",
                readonly: true,
                tunable: false,
            },
            FieldDecl {
                name: "pointerInside",
                column: "bool",
                default: "false",
                readonly: true,
                tunable: false,
            },
            FieldDecl {
                name: "wheel",
                column: "Vector2",
                default: "{\"x\":0,\"y\":0}",
                readonly: true,
                tunable: false,
            },
        ],
    },
    RecordDecl {
        kind: "event",
        name: "KeyDown",
        schema: "og.KeyDown",
        fields: &[
            FieldDecl {
                name: "key",
                column: "String",
                default: "\"\"",
                readonly: false,
                tunable: false,
            },
            FieldDecl {
                name: "repeat",
                column: "bool",
                default: "false",
                readonly: false,
                tunable: false,
            },
        ],
    },
    RecordDecl {
        kind: "event",
        name: "KeyUp",
        schema: "og.KeyUp",
        fields: &[
            FieldDecl {
                name: "key",
                column: "String",
                default: "\"\"",
                readonly: false,
                tunable: false,
            },
        ],
    },
    RecordDecl {
        kind: "event",
        name: "PointerDown",
        schema: "og.PointerDown",
        fields: &[
            FieldDecl {
                name: "x",
                column: "f32",
                default: "0",
                readonly: false,
                tunable: false,
            },
            FieldDecl {
                name: "y",
                column: "f32",
                default: "0",
                readonly: false,
                tunable: false,
            },
            FieldDecl {
                name: "button",
                column: "PointerButton",
                default: "\"none\"",
                readonly: false,
                tunable: false,
            },
        ],
    },
    RecordDecl {
        kind: "event",
        name: "PointerUp",
        schema: "og.PointerUp",
        fields: &[
            FieldDecl {
                name: "x",
                column: "f32",
                default: "0",
                readonly: false,
                tunable: false,
            },
            FieldDecl {
                name: "y",
                column: "f32",
                default: "0",
                readonly: false,
                tunable: false,
            },
            FieldDecl {
                name: "button",
                column: "PointerButton",
                default: "\"none\"",
                readonly: false,
                tunable: false,
            },
        ],
    },
    RecordDecl {
        kind: "event",
        name: "PointerMove",
        schema: "og.PointerMove",
        fields: &[
            FieldDecl {
                name: "x",
                column: "f32",
                default: "0",
                readonly: false,
                tunable: false,
            },
            FieldDecl {
                name: "y",
                column: "f32",
                default: "0",
                readonly: false,
                tunable: false,
            },
        ],
    },
    RecordDecl {
        kind: "event",
        name: "Wheel",
        schema: "og.Wheel",
        fields: &[
            FieldDecl {
                name: "deltaX",
                column: "f32",
                default: "0",
                readonly: false,
                tunable: false,
            },
            FieldDecl {
                name: "deltaY",
                column: "f32",
                default: "0",
                readonly: false,
                tunable: false,
            },
        ],
    },
    RecordDecl {
        kind: "resource",
        name: "Camera",
        schema: "og.Camera",
        fields: &[
            FieldDecl {
                name: "position",
                column: "Vector3",
                default: "{\"x\":0,\"y\":0,\"z\":0}",
                readonly: false,
                tunable: false,
            },
            FieldDecl {
                name: "quaternion",
                column: "Quaternion",
                default: "{\"x\":0,\"y\":0,\"z\":0,\"w\":1}",
                readonly: false,
                tunable: false,
            },
            FieldDecl {
                name: "fov",
                column: "f32",
                default: "50",
                readonly: false,
                tunable: true,
            },
            FieldDecl {
                name: "near",
                column: "f32",
                default: "0.1",
                readonly: false,
                tunable: false,
            },
            FieldDecl {
                name: "far",
                column: "f32",
                default: "2000",
                readonly: false,
                tunable: false,
            },
            FieldDecl {
                name: "zoom",
                column: "f32",
                default: "1",
                readonly: false,
                tunable: true,
            },
            FieldDecl {
                name: "orthographic",
                column: "bool",
                default: "false",
                readonly: false,
                tunable: false,
            },
            FieldDecl {
                name: "height",
                column: "f32",
                default: "10",
                readonly: false,
                tunable: false,
            },
            FieldDecl {
                name: "background",
                column: "Color",
                default: "{\"r\":0.00699,\"g\":0.00803,\"b\":0.01096}",
                readonly: false,
                tunable: false,
            },
            FieldDecl {
                name: "exposure",
                column: "f32",
                default: "1",
                readonly: false,
                tunable: true,
            },
        ],
    },
    RecordDecl {
        kind: "component",
        name: "Mesh",
        schema: "og.Mesh",
        fields: &[
            FieldDecl {
                name: "geometry",
                column: "Geometry",
                default: "0",
                readonly: false,
                tunable: false,
            },
            FieldDecl {
                name: "material",
                column: "Material",
                default: "0",
                readonly: false,
                tunable: false,
            },
            FieldDecl {
                name: "castShadow",
                column: "bool",
                default: "false",
                readonly: false,
                tunable: false,
            },
            FieldDecl {
                name: "receiveShadow",
                column: "bool",
                default: "false",
                readonly: false,
                tunable: false,
            },
            FieldDecl {
                name: "visible",
                column: "bool",
                default: "true",
                readonly: false,
                tunable: false,
            },
        ],
    },
    RecordDecl {
        kind: "event",
        name: "AssetReady",
        schema: "og.AssetReady",
        fields: &[
            FieldDecl {
                name: "asset",
                column: "Asset",
                default: "0",
                readonly: false,
                tunable: false,
            },
            FieldDecl {
                name: "path",
                column: "String",
                default: "\"\"",
                readonly: false,
                tunable: false,
            },
        ],
    },
    RecordDecl {
        kind: "event",
        name: "AssetFailed",
        schema: "og.AssetFailed",
        fields: &[
            FieldDecl {
                name: "asset",
                column: "Asset",
                default: "0",
                readonly: false,
                tunable: false,
            },
            FieldDecl {
                name: "path",
                column: "String",
                default: "\"\"",
                readonly: false,
                tunable: false,
            },
            FieldDecl {
                name: "reason",
                column: "String",
                default: "\"\"",
                readonly: false,
                tunable: false,
            },
        ],
    },
];
