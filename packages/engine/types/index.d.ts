// Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate.
// @origingame/engine API v1.

// The Origin Engine API. Game code declares components and systems, and reads the world through typed views over engine memory.
// Every failure is a returned Result; desc objects may omit any field; nothing reads the wall clock inside a tick.

declare const valueType: unique symbol;
declare const handleType: unique symbol;

/**
 * Every failure in the API is a value, never a throw: check `ok` before
 * reading `value`. `reason` is a plain sentence that names the fix.
 */
export type Result<T> =
  | { readonly ok: true; readonly value: T }
  | { readonly ok: false; readonly reason: string };

/** A column kind: what one field of a component or resource stores. */
export type ColumnKind = keyof ColumnKinds;

/**
 * One field of a reflected layout, made by a column factory
 * (`f32(100, { min: 0 })`, `Vector3()`). `default` is the value a new
 * component or resource starts with.
 */
export interface FieldSpec<K extends ColumnKind = ColumnKind> {
  readonly kind: K;
  readonly default: ColumnKinds[K]["value"];
  readonly options: FieldOptions;
}

/** A reflected layout: field names to field specs. */
export type Fields = { readonly [field: string]: FieldSpec };

/** The value of a layout: each field as its column's value type. */
export type FieldValues<F extends Fields> = {
  -readonly [K in keyof F]: ColumnKinds[F[K]["kind"]]["value"];
};

/** The typed views of a layout: each field as its column's typed array. */
export type ColumnViews<F extends Fields> = {
  readonly [K in keyof F]: ColumnKinds[F[K]["kind"]]["array"];
};

/** A registered component: the token systems, queries and the world take. */
export interface ComponentType<F extends Fields = Fields, V = FieldValues<F>> {
  readonly kind: "component";
  readonly name: string;
  readonly fields: F;
  readonly [valueType]: V;
}

/** A registered resource: a reflected singleton. */
export interface ResourceType<F extends Fields = Fields, V = FieldValues<F>> {
  readonly kind: "resource";
  readonly name: string;
  readonly fields: F;
  readonly [valueType]: V;
}

/** A registered event type. */
export interface EventType<V = unknown> {
  readonly kind: "event";
  readonly name: string;
  readonly [valueType]: V;
}

/** Any component token. */
export type AnyComponent = ComponentType<any, any>;
/** Any resource token. */
export type AnyResource = ResourceType<any, any>;
/** Any event token. */
export type AnyEvent = EventType<any>;
/** Any reflected token a system may declare: a component or a resource. */
export type AnyReflected = AnyComponent | AnyResource;

/** The value a token stands for: a component's fields, a resource object, an event payload. */
export type Value<T> = T extends { readonly [valueType]: infer V } ? V : never;

/** The typed column views of a component token, as `ArchetypeView.column` returns them. */
export type View<T> = T extends ComponentType<infer F, any> ? ColumnViews<F> : never;

/** Every column kind, from the IDL's `column` declarations. */
export interface ColumnKinds {
  /** A 32-bit float, the default numeric field. */
  f32: { readonly array: Float32Array; readonly stride: 1; readonly value: number };
  /** A 64-bit float, for accumulated values that must not drift, such as elapsed time. */
  f64: { readonly array: Float64Array; readonly stride: 1; readonly value: number };
  /** A 32-bit unsigned integer: counters, flags, indices. */
  u32: { readonly array: Uint32Array; readonly stride: 1; readonly value: number };
  /** A 32-bit signed integer. */
  i32: { readonly array: Int32Array; readonly stride: 1; readonly value: number };
  /** A boolean, stored as one byte: 0 or 1. */
  bool: { readonly array: Uint8Array; readonly stride: 1; readonly value: boolean };
  /** A Vector2 field: x, y interleaved. */
  Vector2: { readonly array: Float32Array; readonly stride: 2; readonly value: Vector2 };
  /** A Vector3 field: x, y, z interleaved. */
  Vector3: { readonly array: Float32Array; readonly stride: 3; readonly value: Vector3 };
  /** A Quaternion field: x, y, z, w interleaved. */
  Quaternion: { readonly array: Float32Array; readonly stride: 4; readonly value: Quaternion };
  /** A Matrix4 field: sixteen column-major elements per entity. */
  Matrix4: { readonly array: Float32Array; readonly stride: 16; readonly value: Matrix4 };
  /** A Color field: r, g, b interleaved. */
  Color: { readonly array: Float32Array; readonly stride: 3; readonly value: Color };
  /** An Entity field: (index, generation) pairs. */
  Entity: { readonly array: Uint32Array; readonly stride: 2; readonly value: Entity };
  /** A Geometry handle field. */
  Geometry: { readonly array: Uint32Array; readonly stride: 2; readonly value: Geometry };
  /** A Material handle field. */
  Material: { readonly array: Uint32Array; readonly stride: 2; readonly value: Material };
  /** An Asset handle field. */
  Asset: { readonly array: Uint32Array; readonly stride: 2; readonly value: Asset };
  /** A Sound handle field. */
  Sound: { readonly array: Uint32Array; readonly stride: 2; readonly value: Sound };
  /** A SkeletonAsset handle field. */
  SkeletonAsset: { readonly array: Uint32Array; readonly stride: 2; readonly value: SkeletonAsset };
  /** A ClipAsset handle field. */
  ClipAsset: { readonly array: Uint32Array; readonly stride: 2; readonly value: ClipAsset };
  /** A GraphAsset handle field. */
  GraphAsset: { readonly array: Uint32Array; readonly stride: 2; readonly value: GraphAsset };
  /** A RootMotion field: the variant's index. */
  RootMotion: { readonly array: Uint32Array; readonly stride: 1; readonly value: RootMotion };
  /** A BlendshapeSource field: the variant's index. */
  BlendshapeSource: { readonly array: Uint32Array; readonly stride: 1; readonly value: BlendshapeSource };
  /** An IkSolver field: the variant's index. */
  IkSolver: { readonly array: Uint32Array; readonly stride: 1; readonly value: IkSolver };
  /** An IkSpace field: the variant's index. */
  IkSpace: { readonly array: Uint32Array; readonly stride: 1; readonly value: IkSpace };
  /** An AudioBus field: the variant's index. */
  AudioBus: { readonly array: Uint32Array; readonly stride: 1; readonly value: AudioBus };
}

// ── math ────────────────────────────────────────────────────────
// Plain math values in the three.js vocabulary. They are data: the engine owns the operations, so results are identical on every host.

/** A 2D vector, as three.js Vector2. */
export interface Vector2 {
  /** The x component. */
  x: number;
  /** The y component. */
  y: number;
}

/** A 3D vector, as three.js Vector3. The engine is right-handed with +Y up, as three.js. */
export interface Vector3 {
  /** The x component. */
  x: number;
  /** The y component. */
  y: number;
  /** The z component. */
  z: number;
}

/** A unit quaternion rotation, as three.js Quaternion. The zero value is the identity. */
export interface Quaternion {
  /** The x component. */
  x: number;
  /** The y component. */
  y: number;
  /** The z component. */
  z: number;
  /**
   * The w (scalar) component.
   *
   * @default 1
   */
  w: number;
}

/**
 * A 4x4 matrix, as three.js Matrix4: sixteen elements in column-major order. The zero value is the identity.
 */
export interface Matrix4 {
  /**
   * The elements, column-major, as three.js Matrix4.elements.
   *
   * @default [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1]
   */
  elements: [number, number, number, number, number, number, number, number, number, number, number, number, number, number, number, number];
}

/**
 * A linear RGB colour, as three.js Color with colour management on. Components are usually between 0 and 1; above 1 is HDR.
 */
export interface Color {
  /** The red component. */
  r: number;
  /** The green component. */
  g: number;
  /** The blue component. */
  b: number;
}

/** A ray, as three.js Ray. */
export interface Ray {
  /** Where the ray starts. */
  readonly origin: Vector3;
  /** The unit direction the ray points in. */
  readonly direction: Vector3;
}

// ── reflect ─────────────────────────────────────────────────────
// Reflection: components and resources are declared once with their field layout and metadata, and registered by name.
// The inspector, Tune, the probe, scene text and the wiki read this one schema. Each column factory below makes one field.

/**
 * Metadata of one field. Ranges and units are what Tune and the probe's bounds show; nothing clamps writes.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface FieldOptions {
  /**
   * The smallest meaningful value.
   *
   * @default null
   */
  readonly min?: number | null;
  /**
   * The largest meaningful value.
   *
   * @default null
   */
  readonly max?: number | null;
  /**
   * The increment Tune uses.
   *
   * @default null
   */
  readonly step?: number | null;
  /**
   * A unit name from the API's unit table, or empty.
   *
   * @default ""
   */
  readonly unit?: string;
  /**
   * The person-facing name; empty uses the field name.
   *
   * @default ""
   */
  readonly label?: string;
  /**
   * One sentence on what the field means.
   *
   * @default ""
   */
  readonly doc?: string;
  /**
   * Publish the field to Tune and to the probe's bounds and patch.
   *
   * @default false
   */
  readonly tunable?: boolean;
}

/**
 * A 32-bit float, the default numeric field.
 *
 * A `f32` field: stored in a `Float32Array` with stride 1.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function f32(value?: number, options?: FieldOptions): FieldSpec<"f32">;

/**
 * A 64-bit float, for accumulated values that must not drift, such as elapsed time.
 *
 * A `f64` field: stored in a `Float64Array` with stride 1.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function f64(value?: number, options?: FieldOptions): FieldSpec<"f64">;

/**
 * A 32-bit unsigned integer: counters, flags, indices.
 *
 * A `u32` field: stored in a `Uint32Array` with stride 1.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function u32(value?: number, options?: FieldOptions): FieldSpec<"u32">;

/**
 * A 32-bit signed integer.
 *
 * A `i32` field: stored in a `Int32Array` with stride 1.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function i32(value?: number, options?: FieldOptions): FieldSpec<"i32">;

/**
 * A boolean, stored as one byte: 0 or 1.
 *
 * A `bool` field: stored in a `Uint8Array` with stride 1.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function bool(value?: boolean, options?: FieldOptions): FieldSpec<"bool">;

/**
 * A Vector2 field: x, y interleaved.
 *
 * A `Vector2` field: stored in a `Float32Array` with stride 2.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function Vector2(value?: Vector2, options?: FieldOptions): FieldSpec<"Vector2">;

/**
 * A Vector3 field: x, y, z interleaved.
 *
 * A `Vector3` field: stored in a `Float32Array` with stride 3.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function Vector3(value?: Vector3, options?: FieldOptions): FieldSpec<"Vector3">;

/**
 * A Quaternion field: x, y, z, w interleaved.
 *
 * A `Quaternion` field: stored in a `Float32Array` with stride 4.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function Quaternion(value?: Quaternion, options?: FieldOptions): FieldSpec<"Quaternion">;

/**
 * A Matrix4 field: sixteen column-major elements per entity.
 *
 * A `Matrix4` field: stored in a `Float32Array` with stride 16.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function Matrix4(value?: Matrix4, options?: FieldOptions): FieldSpec<"Matrix4">;

/**
 * A Color field: r, g, b interleaved.
 *
 * A `Color` field: stored in a `Float32Array` with stride 3.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function Color(value?: Color, options?: FieldOptions): FieldSpec<"Color">;

/**
 * Declare a component: plain data stored per entity in engine memory, one column per field.
 * Registration is by name, so a hot reload re-registers the same component and migrates its fields by name.
 *
 * @param name The component's unique name, PascalCase, as it appears in scenes and dumps.
 * @param fields The layout: field names to column factories, as `{ hp: f32(100) }`.
 * @returns The token systems, queries and the world take.
 */
export declare function defineComponent<F extends Fields>(name: string, fields: F): ComponentType<F>;

/**
 * Declare a resource: a reflected singleton, such as a tuning table. Its fields start at their defaults.
 *
 * @param name The resource's unique name, PascalCase.
 * @param fields The layout, as for defineComponent.
 * @returns The token `world.resource` and system declarations take.
 */
export declare function defineResource<F extends Fields>(name: string, fields: F): ResourceType<F>;

// ── world ───────────────────────────────────────────────────────
// The world: an archetype ECS. Entities are generational handles; components are columns in engine memory.
// Structural changes (spawn, despawn, insert, remove) invalidate typed views; the API refuses a stale view.

/**
 * An entity: an index and a generation. The value is index + generation * 2^32, with the generation below 2^21, so it stays an exact JavaScript number.
 * Generations start at 1, so 0 is never a live entity.
 *
 * A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.
 */
export type Entity = number & { readonly [handleType]: "Entity" };

/**
 * An Entity field: (index, generation) pairs.
 *
 * A `Entity` field: stored in a `Uint32Array` with stride 2.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function Entity(value?: Entity, options?: FieldOptions): FieldSpec<"Entity">;

/**
 * How to spawn an entity. `{}` spawns an unnamed entity at the origin with a Transform.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface SpawnDesc {
  /**
   * A unique name; the probe and the command channel address the entity by it. Empty leaves it unnamed.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * The parent entity; the new entity's Transform is then relative to it.
   *
   * @default null
   */
  readonly parent?: Entity | null;
  /**
   * The initial Transform.position.
   *
   * @default { x: 0, y: 0, z: 0 }
   * @unit m (metres)
   */
  readonly position?: Vector3;
  /**
   * The initial Transform.quaternion.
   *
   * @default { x: 0, y: 0, z: 0, w: 1 }
   */
  readonly quaternion?: Quaternion;
  /**
   * The initial Transform.scale.
   *
   * @default { x: 1, y: 1, z: 1 }
   */
  readonly scale?: Vector3;
  /**
   * Spawn without a Transform: a pure data entity.
   *
   * @default false
   */
  readonly bare?: boolean;
}

/**
 * Which entities a query matches.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface QueryDesc {
  /**
   * Components an entity must have.
   *
   * @default []
   */
  readonly with?: readonly AnyComponent[];
  /**
   * Components an entity must not have.
   *
   * @default []
   */
  readonly without?: readonly AnyComponent[];
}

/**
 * The world a system, a command or a game callback is handed. Game code never holds a world beyond the call it was passed to.
 */
export interface World {
  /** Fixed steps taken since the world was created. */
  readonly tick: number;
  /**
   * Spawn an entity.
   *
   * @param desc Name, parent and initial transform.
   * @returns The new entity, or why it was refused (a duplicate name, a dead parent).
   */
  spawn(desc?: SpawnDesc): Result<Entity>;
  /**
   * Despawn an entity and its descendants.
   *
   * @param entity The entity.
   * @returns Refused for a dead entity.
   */
  despawn(entity: Entity): Result<void>;
  /**
   * Whether an entity handle is live.
   *
   * @param entity The entity.
   */
  alive(entity: Entity): boolean;
  /**
   * Whether an entity has a component.
   *
   * @param entity The entity.
   * @param component The component token.
   */
  has<C extends AnyComponent>(entity: Entity, component: C): boolean;
  /**
   * Add a component to an entity, or overwrite it when present. A structural change.
   *
   * @param entity The entity.
   * @param component The component token.
   * @param value Field values; omitted fields take their defaults.
   * @returns Refused for a dead entity or an unregistered component.
   */
  insert<C extends AnyComponent>(entity: Entity, component: C, value?: Partial<Value<C>>): Result<void>;
  /**
   * Remove a component from an entity. A structural change.
   *
   * @param entity The entity.
   * @param component The component token.
   * @returns Refused for a dead entity or a component it does not have.
   */
  remove<C extends AnyComponent>(entity: Entity, component: C): Result<void>;
  /**
   * Copy one entity's component value. For bulk work use a system's typed views.
   *
   * @param entity The entity.
   * @param component The component token.
   */
  get<C extends AnyComponent>(entity: Entity, component: C): Result<Value<C>>;
  /**
   * Write some fields of one entity's component.
   *
   * @param entity The entity.
   * @param component The component token.
   * @param value The fields to write.
   * @returns Refused for a dead entity or a component it does not have.
   */
  set<C extends AnyComponent>(entity: Entity, component: C, value: Partial<Value<C>>): Result<void>;
  /**
   * A resource's live value. Inside a system, the resource must be among the system's reads or writes.
   *
   * @param resource The resource token.
   * @returns Refused for an unregistered resource or one the running system did not declare.
   */
  resource<R extends AnyResource>(resource: R): Result<Value<R>>;
  /**
   * The entity with a name.
   *
   * @param name The name given at spawn.
   * @returns Null when no live entity has the name.
   */
  find(name: string): Entity | null;
  /**
   * An entity's name.
   *
   * @param entity The entity.
   * @returns Null for an unnamed or dead entity.
   */
  nameOf(entity: Entity): string | null;
  /**
   * A query over the current archetypes, for commands and callbacks; systems receive theirs.
   *
   * @param desc The components to match.
   */
  query(desc?: QueryDesc): Result<Query>;
  /**
   * The events of one type emitted during the previous tick and this one, oldest first.
   *
   * @param event The event token.
   */
  events<E extends AnyEvent>(event: E): readonly Value<E>[];
  /**
   * Emit an event; readers see it this tick and the next.
   *
   * @param event The event token.
   * @param payload The event's fields.
   */
  emit<E extends AnyEvent>(event: E, payload: Value<E>): Result<void>;
  /**
   * Spawn a loaded scene or glTF asset into the world: a root entity from the desc, then (for a scene) every scene entity under it with its components and names, or (for a glTF) a Model on the root.
   * Scene entity names must be free: a name already in use refuses the whole instantiation before anything spawns.
   *
   * @param asset A ready asset from og.load.
   * @param desc Name, parent and transform of the root entity.
   * @returns The root entity, or why it was refused (an asset still loading, a failed asset).
   */
  instantiate(asset: Asset, desc?: SpawnDesc): Result<Entity>;
}

/** A set of matched archetypes. Iterate archetypes, never entities one by one. */
export interface Query {
  /** The matched archetypes this tick. */
  archetypes(): readonly ArchetypeView[];
  /** The number of matched entities. */
  count(): number;
}

/**
 * One archetype's columns for this tick. Views are invalidated by structural changes; using a stale view throws with the tick and the system it belonged to.
 */
export interface ArchetypeView {
  /** The number of entities in the archetype. */
  readonly count: number;
  /** The tick this view belongs to. */
  readonly tick: number;
  /** The entities as (index, generation) pairs: entity i is at 2i and 2i + 1. */
  readonly entities: Uint32Array;
  /**
   * The handle of entity i.
   *
   * @param index The row, from 0 to count - 1.
   */
  entity(index: number): Entity;
  /**
   * Whether the archetype carries a component.
   *
   * @param component The component token.
   */
  has<C extends AnyComponent>(component: C): boolean;
  /**
   * The typed views of a component's fields: one typed array per field, `count * stride` long.
   * Writing requires the component among the system's writes; reading among its reads or writes.
   *
   * @param component The component token.
   */
  column<C extends AnyComponent>(component: C): View<C>;
}

// ── system ──────────────────────────────────────────────────────
// Systems: functions the engine calls once per tick with their matched archetypes.
// A system declares what it reads and writes; a write to an undeclared component is refused at registration with the field name.

/**
 * A system's declared access and order.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface SystemDesc {
  /**
   * Components, resources and events the system reads (world.events). Components here also shape its query.
   *
   * @default []
   */
  readonly reads?: readonly (AnyReflected | AnyEvent)[];
  /**
   * Components, resources and events the system writes (world.emit). Components here also shape its query.
   *
   * @default []
   */
  readonly writes?: readonly (AnyReflected | AnyEvent)[];
  /**
   * Components a matched entity must not have.
   *
   * @default []
   */
  readonly without?: readonly AnyComponent[];
  /**
   * Names of systems that must run before this one.
   *
   * @default []
   */
  readonly after?: readonly string[];
  /**
   * Names of systems that must run after this one.
   *
   * @default []
   */
  readonly before?: readonly string[];
}

/**
 * A system's body. It is called once per tick; loop over `q.archetypes()`, then over rows.
 *
 * @param q The archetypes matching the declared components.
 * @param time The Time resource for this tick.
 * @param world The world, for resources, events and structural changes.
 */
export type SystemRun = (q: Query, time: Time, world: World) => void;

/** A registered system. */
export interface System {
  /** The system's unique name. */
  readonly name: string;
  /** What it reads. */
  readonly reads: readonly AnyReflected[];
  /** What it writes. */
  readonly writes: readonly AnyReflected[];
}

/**
 * Declare a system.
 *
 * @param name The system's unique name, PascalCase.
 * @param desc Reads, writes and order.
 * @param run The body.
 */
export declare function defineSystem(name: string, desc: SystemDesc, run: SystemRun): System;

// ── game ────────────────────────────────────────────────────────
// The game entry: what the bundle's default export declares.

/**
 * The game's own facts for the probe's state(): a small JSON object, computed from the world without changing it.
 *
 * @param world The world, read only.
 */
export type FactsFn = (world: World) => unknown;

/**
 * The game's setup: called once before the first tick, after GameDesc.scene has spawned. Spawn, insert, load and set resources here; systems then only read and write their views.
 *
 * @param world The world.
 */
export type StartFn = (world: World) => void;

/**
 * A game: its tick, scenes, systems and commands.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface GameDesc {
  /**
   * The fixed tick rate.
   *
   * @default 60
   * @range 1..240
   * @unit hz (ticks or cycles per second)
   */
  readonly tick?: number;
  /**
   * The project seed every Rng stream derives from. A replay overrides it.
   *
   * @default 0
   */
  readonly seed?: number;
  /**
   * Scene names to `.ogt` paths, as `{ main: "scenes/main.ogt" }`.
   *
   * @default {}
   */
  readonly scenes?: Readonly<Record<string, string>>;
  /**
   * The scene loaded first.
   *
   * @default "main"
   */
  readonly scene?: string;
  /**
   * The systems, in declaration order before `after`/`before` constraints apply.
   *
   * @default []
   */
  readonly systems?: readonly System[];
  /**
   * The game's own resources, created at their defaults.
   *
   * @default []
   */
  readonly resources?: readonly AnyResource[];
  /**
   * Game actions registered into the command channel, by action name.
   *
   * @default {}
   */
  readonly commands?: Readonly<Record<string, CommandHandler>>;
  /**
   * The most ticks one frame may run to catch up with the clock; the accumulator drops the rest.
   *
   * @default 4
   * @range 1..60
   * @unit ticks (fixed simulation steps)
   */
  readonly maxCatchUp?: number;
  /**
   * The game's own facts for the probe's state().
   *
   * @default null
   */
  readonly facts?: FactsFn | null;
  /**
   * Called once before the first tick, after the first scene spawned.
   *
   * @default null
   */
  readonly start?: StartFn | null;
  /**
   * How the renderer is created: antialiasing and shadow map size. Read once, when the host attaches the renderer.
   *
   * @default {}
   */
  readonly render?: RenderOptions;
}

/** A declared game, the bundle's default export. */
export interface Game {
  /** The fixed tick rate in hertz. */
  readonly tick: number;
  /** The systems in declaration order. */
  readonly systems: readonly System[];
}


// ── transform ───────────────────────────────────────────────────
// Transforms in the three.js vocabulary: position, quaternion and scale, composed into matrixWorld down the parent chain.

/**
 * Where an entity is, relative to its parent (or the world).
 *
 * The value of the `Transform` component (token `og.Transform`).
 */
export interface Transform {
  /**
   * The position, as three.js Object3D.position.
   *
   * @unit m (metres)
   */
  position: Vector3;
  /** The rotation, as three.js Object3D.quaternion. */
  quaternion: Quaternion;
  /**
   * The scale, as three.js Object3D.scale.
   *
   * @default { x: 1, y: 1, z: 1 }
   */
  scale: Vector3;
}

/**
 * The entity this one's Transform is relative to. Set by SpawnDesc.parent.
 *
 * The value of the `Parent` component (token `og.Parent`).
 */
export interface Parent {
  /** The parent. */
  entity: Entity;
}

/**
 * The world matrix the engine composes after the systems run, as three.js Object3D.matrixWorld.
 *
 * The value of the `MatrixWorld` component (token `og.MatrixWorld`).
 */
export interface MatrixWorld {
  /**
   * The composed world matrix.
   *
   * @readonly written by the engine
   */
  readonly matrix: Matrix4;
}





// ── time ────────────────────────────────────────────────────────
// Simulation time. Game code never reads the wall clock; it reads Time.

/**
 * The fixed tick's clock. Every value is a function of the tick count, so a replay sees the same numbers.
 *
 * The `Time` resource (token `og.Time`).
 */
export interface Time {
  /**
   * Fixed steps taken since load.
   *
   * @unit ticks (fixed simulation steps)
   * @readonly written by the engine
   */
  readonly tick: number;
  /**
   * The length of one tick: 1 / rate.
   *
   * @unit s (seconds of simulation time)
   * @readonly written by the engine
   */
  readonly dt: number;
  /**
   * Simulation time since load: tick * dt.
   *
   * @unit s (seconds of simulation time)
   * @readonly written by the engine
   */
  readonly elapsed: number;
  /**
   * The tick rate, from GameDesc.tick.
   *
   * @unit hz (ticks or cycles per second)
   * @readonly written by the engine
   */
  readonly rate: number;
}

// ── rng ─────────────────────────────────────────────────────────
// Deterministic randomness. Every random draw comes from an Rng stream; the sequence for a seed is identical on every host and in the Web scaffold's rng.ts.

/**
 * One generator stream, forked from the Rng resource for a subsystem that must not perturb the main sequence.
 */
export interface RngStream {
  /** The stream's derived seed. */
  readonly seed: number;
  /** Uniform in [0, 1). */
  next(): number;
  /**
   * Uniform in [low, high).
   *
   * @param low The inclusive lower bound.
   * @param high The exclusive upper bound.
   */
  range(low: number, high: number): number;
  /**
   * A uniform integer in [low, high].
   *
   * @param low The inclusive lower bound.
   * @param high The inclusive upper bound.
   */
  integer(low: number, high: number): number;
  /**
   * True with probability p.
   *
   * @param p The probability, between 0 and 1.
   */
  chance(p: number): boolean;
  /**
   * A fresh stream on a seed derived from this stream's seed and a label.
   *
   * @param label The subsystem's name; the same label always gives the same stream.
   */
  fork(label: string): RngStream;
}

/**
 * The world's main random stream, seeded from GameDesc.seed (or the replay seed). Its state is part of every snapshot.
 *
 * The `Rng` resource (token `og.Rng`).
 */
export interface Rng {
  /**
   * The seed the stream started from.
   *
   * @readonly written by the engine
   */
  readonly seed: number;
  /**
   * The generator state; snapshot and restore carry it.
   *
   * @readonly written by the engine
   */
  readonly state: number;
  /** Uniform in [0, 1). */
  next(): number;
  /**
   * Uniform in [low, high).
   *
   * @param low The inclusive lower bound.
   * @param high The exclusive upper bound.
   */
  range(low: number, high: number): number;
  /**
   * A uniform integer in [low, high].
   *
   * @param low The inclusive lower bound.
   * @param high The inclusive upper bound.
   */
  integer(low: number, high: number): number;
  /**
   * True with probability p.
   *
   * @param p The probability, between 0 and 1.
   */
  chance(p: number): boolean;
  /**
   * A stream for a subsystem, derived from the seed and a label, so drawing from it leaves the main sequence alone.
   *
   * @param label The subsystem's name, such as `spawner`.
   */
  fork(label: string): RngStream;
}

// ── input ───────────────────────────────────────────────────────
// Input as the game sees it: the Input resource for polling, events for edges.
// The shell collects pointer, keyboard and wheel input on the main thread; the probe and the command channel inject through the same path.
// Several pointers at once (multi-touch) are og.Pointers; the first four gamepads, in the W3C standard mapping, are og.Gamepad. Both are world state, as og.Input is, so they replay.

/** The most pointers og.Pointers tracks at once: ids 0 (the primary pointer) to 4. */
export declare const MAX_POINTERS: 5;

/** The most gamepads og.Gamepad reads: pads 0 to 3. */
export declare const MAX_GAMEPADS: 4;

/**
 * A gamepad button in the W3C standard mapping, in its index order (a is button 0).
 * - `"a"`: The bottom face button (A, Cross).
 * - `"b"`: The right face button (B, Circle).
 * - `"x"`: The left face button (X, Square).
 * - `"y"`: The top face button (Y, Triangle).
 * - `"leftBumper"`: The left shoulder button.
 * - `"rightBumper"`: The right shoulder button.
 * - `"leftTrigger"`: The left trigger, as a button (its analog value is the leftTrigger axis).
 * - `"rightTrigger"`: The right trigger, as a button.
 * - `"back"`: Back, Select or View.
 * - `"start"`: Start, Options or Menu.
 * - `"leftStick"`: The left stick pressed in.
 * - `"rightStick"`: The right stick pressed in.
 * - `"up"`: The d-pad's up.
 * - `"down"`: The d-pad's down.
 * - `"left"`: The d-pad's left.
 * - `"right"`: The d-pad's right.
 * - `"home"`: The home or guide button.
 */
export type GamepadButton = "a" | "b" | "x" | "y" | "leftBumper" | "rightBumper" | "leftTrigger" | "rightTrigger" | "back" | "start" | "leftStick" | "rightStick" | "up" | "down" | "left" | "right" | "home";

/**
 * An analog input of a gamepad.
 * - `"leftX"`: The left stick's x, -1 (left) to 1 (right).
 * - `"leftY"`: The left stick's y, -1 (up) to 1 (down), as the Gamepad API reports it.
 * - `"rightX"`: The right stick's x.
 * - `"rightY"`: The right stick's y, down positive.
 * - `"leftTrigger"`: The left trigger, 0 to 1.
 * - `"rightTrigger"`: The right trigger, 0 to 1.
 */
export type GamepadAxis = "leftX" | "leftY" | "rightX" | "rightY" | "leftTrigger" | "rightTrigger";

/**
 * What an injected key event does.
 * - `"press"`: Down, then up on the next tick.
 * - `"down"`: Held down until an up.
 * - `"up"`: Released.
 */
export type KeyAction = "press" | "down" | "up";

/**
 * What an injected pointer event does.
 * - `"click"`: Down, then up on the next tick, at the point.
 * - `"move"`: A move to the point.
 * - `"down"`: A press at the point.
 * - `"up"`: A release at the point.
 */
export type PointerAction = "click" | "move" | "down" | "up";

/**
 * A physical pointer button; `none` is a hover move.
 * - `"none"`: No button.
 * - `"left"`: The primary button, or a touch.
 * - `"middle"`: The middle button.
 * - `"right"`: The secondary button.
 */
export type PointerButton = "none" | "left" | "middle" | "right";

/** Modifier keys held during a key event. */
export interface Modifiers {
  /** Alt or Option. */
  readonly alt: boolean;
  /** Control. */
  readonly control: boolean;
  /** Command on macOS, the Windows key on Windows. */
  readonly platform: boolean;
  /** Shift. */
  readonly shift: boolean;
}

/**
 * A key event. `key` names the physical key as KeyboardEvent.code does: ArrowRight, Space, KeyA.
 */
export interface KeyInput {
  /** The physical key code. */
  readonly key: string;
  /** Press, down or up. */
  readonly action: KeyAction;
  /** Held modifiers; omitted means none. */
  readonly modifiers?: Modifiers;
}

/** A pointer event at a point in CSS pixels of the play surface. */
export interface PointerInput {
  /** Click, move, down or up. */
  readonly action: PointerAction;
  /**
   * Horizontal position.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly x: number;
  /**
   * Vertical position.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly y: number;
  /** The button; `none` for a hover move. */
  readonly button: PointerButton;
  /**
   * Which pointer: 0 the primary one (the mouse, the first touch; what protocol 5 means), 1 to 4 further touches held at the same time (an engine extension). Omitted is 0.
   *
   * @range ..4
   */
  readonly id?: number;
}

/**
 * A gamepad's whole state (an engine extension of protocol 5): the shell sends one whenever a pad changes, and a playtest can send its own.
 */
export interface GamepadInput {
  /**
   * Which pad, 0 to 3; omitted is 0.
   *
   * @range ..3
   */
  readonly pad?: number;
  /** Whether the pad is connected (omitted is true); a disconnected pad holds nothing. */
  readonly connected?: boolean;
  /** The buttons held (omitted is none). */
  readonly buttons?: readonly GamepadButton[];
  /** Left stick x, y, right stick x, y; each -1 to 1, y down. Missing values are 0. */
  readonly axes?: readonly number[];
  /** The left and right analog triggers, 0 to 1. Missing values are 0. */
  readonly triggers?: readonly number[];
}

/** A wheel or trackpad scroll at a point. */
export interface WheelInput {
  /**
   * Horizontal position.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly x: number;
  /**
   * Vertical position.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly y: number;
  /**
   * Horizontal scroll amount.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly deltaX: number;
  /**
   * Vertical scroll amount.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly deltaY: number;
}

/**
 * One input event in the game's own vocabulary, as the probe's input() and playtest scripts carry it.
 */
export type InjectedInput =
  /** A key event. */
  | ({ readonly kind: "key" } & KeyInput)
  /** A pointer event. */
  | ({ readonly kind: "pointer" } & PointerInput)
  /** A wheel event. */
  | ({ readonly kind: "wheel" } & WheelInput)
  /** A gamepad's state. */
  | ({ readonly kind: "gamepad" } & GamepadInput);

/**
 * The input state at the start of this tick (og-input's og.Input, version 2). Edges (pressed, released) last exactly one tick; read the state through the methods.
 * Key sets are 128 bits over og-input's KEY_CODES (KeyboardEvent.code names): code i is bit i % 32 of word i / 32. Button masks: left 1, middle 2, right 4.
 *
 * The `Input` resource (token `og.Input`).
 */
export interface Input {
  /**
   * The pointer position.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   * @readonly written by the engine
   */
  readonly pointer: Vector2;
  /**
   * Whether a pointer event has placed the pointer over the play surface.
   *
   * @readonly written by the engine
   */
  readonly pointerInside: boolean;
  /**
   * Scroll accumulated during this tick.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   * @readonly written by the engine
   */
  readonly wheel: Vector2;
  /**
   * Held pointer buttons, as a mask.
   *
   * @readonly written by the engine
   */
  readonly buttons: number;
  /**
   * Buttons that went down this tick.
   *
   * @readonly written by the engine
   */
  readonly buttonsPressed: number;
  /**
   * Buttons that went up this tick.
   *
   * @readonly written by the engine
   */
  readonly buttonsReleased: number;
  /**
   * Buttons a click releases at the end of the next tick.
   *
   * @readonly written by the engine
   */
  readonly buttonsReleaseNext: number;
  /**
   * Held keys, word 0: codes 0 to 31 of KEY_CODES.
   *
   * @readonly written by the engine
   */
  readonly keys0: number;
  /**
   * Held keys, word 1: codes 32 to 63 of KEY_CODES.
   *
   * @readonly written by the engine
   */
  readonly keys1: number;
  /**
   * Held keys, word 2: codes 64 to 95 of KEY_CODES.
   *
   * @readonly written by the engine
   */
  readonly keys2: number;
  /**
   * Held keys, word 3: codes 96 to 127 of KEY_CODES.
   *
   * @readonly written by the engine
   */
  readonly keys3: number;
  /**
   * Keys that went down this tick, word 0: codes 0 to 31 of KEY_CODES.
   *
   * @readonly written by the engine
   */
  readonly pressed0: number;
  /**
   * Keys that went down this tick, word 1: codes 32 to 63 of KEY_CODES.
   *
   * @readonly written by the engine
   */
  readonly pressed1: number;
  /**
   * Keys that went down this tick, word 2: codes 64 to 95 of KEY_CODES.
   *
   * @readonly written by the engine
   */
  readonly pressed2: number;
  /**
   * Keys that went down this tick, word 3: codes 96 to 127 of KEY_CODES.
   *
   * @readonly written by the engine
   */
  readonly pressed3: number;
  /**
   * Keys that went up this tick, word 0: codes 0 to 31 of KEY_CODES.
   *
   * @readonly written by the engine
   */
  readonly released0: number;
  /**
   * Keys that went up this tick, word 1: codes 32 to 63 of KEY_CODES.
   *
   * @readonly written by the engine
   */
  readonly released1: number;
  /**
   * Keys that went up this tick, word 2: codes 64 to 95 of KEY_CODES.
   *
   * @readonly written by the engine
   */
  readonly released2: number;
  /**
   * Keys that went up this tick, word 3: codes 96 to 127 of KEY_CODES.
   *
   * @readonly written by the engine
   */
  readonly released3: number;
  /**
   * Keys a press releases at the end of the next tick, word 0: codes 0 to 31 of KEY_CODES.
   *
   * @readonly written by the engine
   */
  readonly releaseNext0: number;
  /**
   * Keys a press releases at the end of the next tick, word 1: codes 32 to 63 of KEY_CODES.
   *
   * @readonly written by the engine
   */
  readonly releaseNext1: number;
  /**
   * Keys a press releases at the end of the next tick, word 2: codes 64 to 95 of KEY_CODES.
   *
   * @readonly written by the engine
   */
  readonly releaseNext2: number;
  /**
   * Keys a press releases at the end of the next tick, word 3: codes 96 to 127 of KEY_CODES.
   *
   * @readonly written by the engine
   */
  readonly releaseNext3: number;
  /**
   * Whether a key is held.
   *
   * @param key A KeyboardEvent.code name, such as KeyW or Space.
   */
  down(key: string): boolean;
  /**
   * Whether a key went down this tick.
   *
   * @param key A KeyboardEvent.code name.
   */
  pressed(key: string): boolean;
  /**
   * Whether a key went up this tick.
   *
   * @param key A KeyboardEvent.code name.
   */
  released(key: string): boolean;
  /**
   * -1, 0 or 1 from two keys, such as `axis("KeyA", "KeyD")`.
   *
   * @param negative The key for -1.
   * @param positive The key for +1.
   */
  axis(negative: string, positive: string): number;
  /**
   * A 2D axis from four keys, such as `axis2("KeyA", "KeyD", "KeyS", "KeyW")`: each component -1, 0 or 1.
   *
   * @param left The key for x = -1.
   * @param right The key for x = +1.
   * @param down The key for y = -1.
   * @param up The key for y = +1.
   */
  axis2(left: string, right: string, down: string, up: string): Vector2;
  /**
   * Whether a pointer button is held.
   *
   * @param button The button.
   */
  buttonDown(button: PointerButton): boolean;
  /**
   * Whether a pointer button went down this tick.
   *
   * @param button The button.
   */
  buttonPressed(button: PointerButton): boolean;
  /**
   * Whether a pointer button went up this tick.
   *
   * @param button The button.
   */
  buttonReleased(button: PointerButton): boolean;
}

/**
 * Every pointer on the play surface at once, at the start of this tick (og-input's og.Pointers): a touch stick and a touch button can be held together. Pointer 0 is og.Input's pointer; ids 1 to 4 are further touches, in the order they landed (the lowest free id).
 * Edges last exactly one tick, as og.Input's do. Masks are bit `id`; read them through the methods.
 *
 * The `Pointers` resource (token `og.Pointers`).
 */
export interface Pointers {
  /**
   * Pointers held down, as a mask.
   *
   * @readonly written by the engine
   */
  readonly heldMask: number;
  /**
   * Pointers that went down this tick.
   *
   * @readonly written by the engine
   */
  readonly pressedMask: number;
  /**
   * Pointers that went up this tick.
   *
   * @readonly written by the engine
   */
  readonly releasedMask: number;
  /**
   * Pointers a click releases at the end of the next tick.
   *
   * @readonly written by the engine
   */
  readonly releaseNextMask: number;
  /**
   * Where pointer 0 is.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   * @readonly written by the engine
   */
  readonly position0: Vector2;
  /**
   * Where pointer 1 is.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   * @readonly written by the engine
   */
  readonly position1: Vector2;
  /**
   * Where pointer 2 is.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   * @readonly written by the engine
   */
  readonly position2: Vector2;
  /**
   * Where pointer 3 is.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   * @readonly written by the engine
   */
  readonly position3: Vector2;
  /**
   * Where pointer 4 is.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   * @readonly written by the engine
   */
  readonly position4: Vector2;
  /**
   * Where pointer 0 went down.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   * @readonly written by the engine
   */
  readonly start0: Vector2;
  /**
   * Where pointer 1 went down.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   * @readonly written by the engine
   */
  readonly start1: Vector2;
  /**
   * Where pointer 2 went down.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   * @readonly written by the engine
   */
  readonly start2: Vector2;
  /**
   * Where pointer 3 went down.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   * @readonly written by the engine
   */
  readonly start3: Vector2;
  /**
   * Where pointer 4 went down.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   * @readonly written by the engine
   */
  readonly start4: Vector2;
  /**
   * Whether a pointer is held.
   *
   * @param id The pointer, 0 to 4.
   */
  down(id: number): boolean;
  /**
   * Whether a pointer went down this tick.
   *
   * @param id The pointer, 0 to 4.
   */
  pressed(id: number): boolean;
  /**
   * Whether a pointer went up this tick.
   *
   * @param id The pointer, 0 to 4.
   */
  released(id: number): boolean;
  /**
   * Where a pointer is, in CSS pixels.
   *
   * @param id The pointer, 0 to 4.
   */
  position(id: number): Vector2;
  /**
   * Where a pointer went down, in CSS pixels: a touch control claims the pointers that started on it.
   *
   * @param id The pointer, 0 to 4.
   */
  start(id: number): Vector2;
  /** How many pointers are held. */
  count(): number;
}

/**
 * The first four gamepads in the W3C standard mapping, at the start of this tick (og-input's og.Gamepad). On the web the shell polls the pads each display frame and sends their state through the input ring, so a pad is world state and replays; it also maps pads to keys for games that only read keys.
 * Button masks are bit `GamepadButton` index per pad; edges last exactly one tick, and a press and a release between two ticks leave both edges set. Read them through the methods.
 *
 * The `Gamepad` resource (token `og.Gamepad`).
 */
export interface Gamepad {
  /**
   * Connected pads: bit `pad`.
   *
   * @readonly written by the engine
   */
  readonly connectedMask: number;
  /**
   * Held buttons of pad 0.
   *
   * @readonly written by the engine
   */
  readonly buttons0: number;
  /**
   * Held buttons of pad 1.
   *
   * @readonly written by the engine
   */
  readonly buttons1: number;
  /**
   * Held buttons of pad 2.
   *
   * @readonly written by the engine
   */
  readonly buttons2: number;
  /**
   * Held buttons of pad 3.
   *
   * @readonly written by the engine
   */
  readonly buttons3: number;
  /**
   * Buttons of pad 0 that went down this tick.
   *
   * @readonly written by the engine
   */
  readonly pressed0: number;
  /**
   * Buttons of pad 1 that went down this tick.
   *
   * @readonly written by the engine
   */
  readonly pressed1: number;
  /**
   * Buttons of pad 2 that went down this tick.
   *
   * @readonly written by the engine
   */
  readonly pressed2: number;
  /**
   * Buttons of pad 3 that went down this tick.
   *
   * @readonly written by the engine
   */
  readonly pressed3: number;
  /**
   * Buttons of pad 0 that went up this tick.
   *
   * @readonly written by the engine
   */
  readonly released0: number;
  /**
   * Buttons of pad 1 that went up this tick.
   *
   * @readonly written by the engine
   */
  readonly released1: number;
  /**
   * Buttons of pad 2 that went up this tick.
   *
   * @readonly written by the engine
   */
  readonly released2: number;
  /**
   * Buttons of pad 3 that went up this tick.
   *
   * @readonly written by the engine
   */
  readonly released3: number;
  /**
   * The left stick of pad 0, -1 to 1, y down.
   *
   * @readonly written by the engine
   */
  readonly leftStick0: Vector2;
  /**
   * The left stick of pad 1, -1 to 1, y down.
   *
   * @readonly written by the engine
   */
  readonly leftStick1: Vector2;
  /**
   * The left stick of pad 2, -1 to 1, y down.
   *
   * @readonly written by the engine
   */
  readonly leftStick2: Vector2;
  /**
   * The left stick of pad 3, -1 to 1, y down.
   *
   * @readonly written by the engine
   */
  readonly leftStick3: Vector2;
  /**
   * The right stick of pad 0, -1 to 1, y down.
   *
   * @readonly written by the engine
   */
  readonly rightStick0: Vector2;
  /**
   * The right stick of pad 1, -1 to 1, y down.
   *
   * @readonly written by the engine
   */
  readonly rightStick1: Vector2;
  /**
   * The right stick of pad 2, -1 to 1, y down.
   *
   * @readonly written by the engine
   */
  readonly rightStick2: Vector2;
  /**
   * The right stick of pad 3, -1 to 1, y down.
   *
   * @readonly written by the engine
   */
  readonly rightStick3: Vector2;
  /**
   * The left (x) and right (y) triggers of pad 0, 0 to 1.
   *
   * @readonly written by the engine
   */
  readonly triggers0: Vector2;
  /**
   * The left (x) and right (y) triggers of pad 1, 0 to 1.
   *
   * @readonly written by the engine
   */
  readonly triggers1: Vector2;
  /**
   * The left (x) and right (y) triggers of pad 2, 0 to 1.
   *
   * @readonly written by the engine
   */
  readonly triggers2: Vector2;
  /**
   * The left (x) and right (y) triggers of pad 3, 0 to 1.
   *
   * @readonly written by the engine
   */
  readonly triggers3: Vector2;
  /**
   * Whether a pad is connected.
   *
   * @param pad The pad, 0 to 3; omitted is 0.
   */
  connected(pad?: number): boolean;
  /**
   * Whether a button is held.
   *
   * @param button The button.
   * @param pad The pad, 0 to 3; omitted is 0.
   */
  down(button: GamepadButton, pad?: number): boolean;
  /**
   * Whether a button went down this tick.
   *
   * @param button The button.
   * @param pad The pad, 0 to 3; omitted is 0.
   */
  pressed(button: GamepadButton, pad?: number): boolean;
  /**
   * Whether a button went up this tick.
   *
   * @param button The button.
   * @param pad The pad, 0 to 3; omitted is 0.
   */
  released(button: GamepadButton, pad?: number): boolean;
  /**
   * An analog input: a stick's x or y (-1 to 1, y down) or a trigger (0 to 1). Dead zones are the game's.
   *
   * @param axis Which input.
   * @param pad The pad, 0 to 3; omitted is 0.
   */
  axis(axis: GamepadAxis, pad?: number): number;
}

/**
 * A key went down (a down for a held key is a repeat and sets no edge). Input events are sent between ticks and read by the next tick's systems.
 *
 * The payload of the `KeyDown` event (token `og.KeyDown`).
 */
export interface KeyDown {
  /** The physical key code. */
  readonly key: string;
  /** Whether this is an auto-repeat. */
  readonly repeat: boolean;
}

/**
 * A key went up.
 *
 * The payload of the `KeyUp` event (token `og.KeyUp`).
 */
export interface KeyUp {
  /** The physical key code. */
  readonly key: string;
}

/**
 * A pointer button went down (or a touch landed).
 *
 * The payload of the `PointerDown` event (token `og.PointerDown`).
 */
export interface PointerDown {
  /** Horizontal position in CSS pixels. */
  readonly x: number;
  /** Vertical position in CSS pixels. */
  readonly y: number;
  /** The button. */
  readonly button: PointerButton;
  /** Which pointer: 0 the primary one, 1 to 4 further touches (og.Pointers). */
  readonly id: number;
}

/**
 * A pointer button went up (or a touch lifted).
 *
 * The payload of the `PointerUp` event (token `og.PointerUp`).
 */
export interface PointerUp {
  /** Horizontal position in CSS pixels. */
  readonly x: number;
  /** Vertical position in CSS pixels. */
  readonly y: number;
  /** The button. */
  readonly button: PointerButton;
  /** Which pointer: 0 the primary one, 1 to 4 further touches. */
  readonly id: number;
}

/**
 * A pointer moved.
 *
 * The payload of the `PointerMove` event (token `og.PointerMove`).
 */
export interface PointerMove {
  /** Horizontal position in CSS pixels. */
  readonly x: number;
  /** Vertical position in CSS pixels. */
  readonly y: number;
  /** Which pointer: 0 the primary one, 1 to 4 further touches. */
  readonly id: number;
}

/**
 * A wheel or trackpad scroll.
 *
 * The payload of the `Wheel` event (token `og.Wheel`).
 */
export interface Wheel {
  /** Horizontal scroll in CSS pixels. */
  readonly deltaX: number;
  /** Vertical scroll in CSS pixels. */
  readonly deltaY: number;
}

// ── camera ──────────────────────────────────────────────────────
// The main camera, as a resource with three.js PerspectiveCamera and OrthographicCamera parameter names.
// Camera controllers are components (OrbitCamera, FollowCamera, Follow2D) on the entity the camera orbits or follows; the systems that drive them ship in TypeScript as og.systems.cameras.

/**
 * The camera the frame is rendered from. Rendering reads it after the tick; it never feeds back into the simulation.
 *
 * The `Camera` resource (token `og.Camera`).
 */
export interface Camera {
  /**
   * Where the camera is.
   *
   * @unit m (metres)
   */
  position: Vector3;
  /** Its rotation; the camera looks down its local -Z. */
  quaternion: Quaternion;
  /**
   * The vertical field of view of the perspective projection.
   *
   * @default 50
   * @range 1..179
   * @unit deg (degrees)
   * @tunable
   */
  fov: number;
  /**
   * The near clipping plane.
   *
   * @default 0.1
   * @range 0.001..
   * @unit m (metres)
   */
  near: number;
  /**
   * The far clipping plane.
   *
   * @default 2000
   * @range 0.01..
   * @unit m (metres)
   */
  far: number;
  /**
   * The zoom factor, as three.js Camera.zoom.
   *
   * @default 1
   * @range 0.01..100
   * @tunable
   */
  zoom: number;
  /** Use an orthographic projection (the 2D profile) instead of a perspective one. */
  orthographic: boolean;
  /**
   * The visible world height of the orthographic projection.
   *
   * @default 10
   * @range 0.01..
   * @unit m (metres)
   */
  height: number;
  /**
   * The clear colour, as three.js Scene.background. The default is the brand ink #14161b in linear RGB.
   *
   * @default { r: 0.00699, g: 0.00803, b: 0.01096 }
   */
  background: Color;
  /**
   * Exposure before tone mapping, as three.js toneMappingExposure.
   *
   * @default 1
   * @range 0..16
   * @tunable
   */
  exposure: number;
  /**
   * Rotate the camera to look at a point, as three.js Camera.lookAt.
   *
   * @param target The point to look at.
   */
  lookAt(target: Vector3): void;
  /**
   * Where a world point lands on the play surface.
   *
   * @param point A world point.
   * @returns CSS pixels, or null behind the camera.
   */
  worldToScreen(point: Vector3): Vector2 | null;
  /**
   * The ray through a point of the play surface, for picking.
   *
   * @param x Horizontal position in CSS pixels.
   * @param y Vertical position in CSS pixels.
   */
  screenToRay(x: number, y: number): Ray;
}

/**
 * Orbits og.Camera around the entity carrying it, as three.js OrbitControls: a left-button drag on the play surface turns it, the wheel zooms, autoRotate turns it on its own.
 * Camera controllers are game-level systems the engine ships in TypeScript (og.systems.cameras); list them in GameDesc.systems: `systems: [...og.systems.cameras, ...yours]`. The first entity carrying one, in query order, drives the camera.
 * The camera sits `distance` from the orbited point (the entity's position plus `target`) in the direction given by `azimuth` about +Y and `polar` from +Y, and looks at the point. Every angle is computed with deterministic arithmetic, so a replay moves the camera identically on every host.
 *
 * The value of the `OrbitCamera` component (token `og.OrbitCamera`).
 */
export interface OrbitCamera {
  /**
   * The orbited point, relative to the entity's position.
   *
   * @unit m (metres)
   */
  target: Vector3;
  /**
   * How far the camera is from the orbited point, as three.js OrbitControls' distance.
   *
   * @default 10
   * @range 0.01..
   * @unit m (metres)
   * @tunable
   */
  distance: number;
  /**
   * The closest the wheel zooms in, as three.js OrbitControls.minDistance.
   *
   * @default 1
   * @range 0.01..
   * @unit m (metres)
   */
  minDistance: number;
  /**
   * The farthest the wheel zooms out, as three.js OrbitControls.maxDistance.
   *
   * @default 100
   * @range 0.01..
   * @unit m (metres)
   */
  maxDistance: number;
  /**
   * The horizontal angle about +Y; 0 puts the camera on the +Z side of the point.
   *
   * @unit rad (radians)
   * @tunable
   */
  azimuth: number;
  /**
   * The angle from +Y: near 0 looks straight down, half pi is level with the point.
   *
   * @default 1.1
   * @range 0..3.1416
   * @unit rad (radians)
   * @tunable
   */
  polar: number;
  /**
   * The smallest polar angle a drag reaches, as three.js OrbitControls.minPolarAngle.
   *
   * @default 0.05
   * @range 0..3.1416
   * @unit rad (radians)
   */
  minPolarAngle: number;
  /**
   * The largest polar angle a drag reaches, as three.js OrbitControls.maxPolarAngle.
   *
   * @default 3.0916
   * @range 0..3.1416
   * @unit rad (radians)
   */
  maxPolarAngle: number;
  /**
   * Drag sensitivity, as three.js OrbitControls.rotateSpeed: at 1, dragging the play surface's height turns a full circle.
   *
   * @default 1
   * @range 0..
   * @tunable
   */
  rotateSpeed: number;
  /**
   * Wheel sensitivity, as three.js OrbitControls.zoomSpeed: at 1, 100 CSS pixels of wheel change the distance by a tenth.
   *
   * @default 1
   * @range 0..
   * @tunable
   */
  zoomSpeed: number;
  /**
   * Turn about the point on its own while not dragged, as three.js OrbitControls.autoRotate.
   */
  autoRotate: boolean;
  /**
   * How fast autoRotate turns.
   *
   * @default 0.5
   * @unit radps (radians per second)
   * @tunable
   */
  autoRotateSpeed: number;
  /**
   * Read the pointer and the wheel; false keeps only autoRotate and the fields' own values.
   *
   * @default true
   */
  enabled: boolean;
  /**
   * Controller state: a left-button drag that started on the play surface is in progress.
   *
   * @readonly written by the engine
   */
  readonly dragging: boolean;
  /**
   * Controller state: where the pointer was on the previous tick of a drag.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   * @readonly written by the engine
   */
  readonly pointer: Vector2;
}

/**
 * Chases the entity carrying it with og.Camera: a third-person 3D view that eases towards a point offset from the entity and looks at it. Driven by og.systems.cameras; the first entity carrying one, in query order, drives the camera.
 *
 * The value of the `FollowCamera` component (token `og.FollowCamera`).
 */
export interface FollowCamera {
  /**
   * Where the camera sits relative to the entity: in the entity's own axes with followRotation, else in world axes.
   *
   * @default { x: 0, y: 2, z: 6 }
   * @unit m (metres)
   */
  offset: Vector3;
  /**
   * The point the camera looks at, relative to the entity's position.
   *
   * @default { x: 0, y: 1, z: 0 }
   * @unit m (metres)
   */
  lookOffset: Vector3;
  /** Turn the offset with the entity's rotation, so the camera stays behind it. */
  followRotation: boolean;
  /**
   * Roughly how long the camera takes to catch up with a moved entity; 0 snaps it there every tick.
   *
   * @default 0.25
   * @range 0..
   * @unit s (seconds of simulation time)
   * @tunable
   */
  smoothTime: number;
}

/**
 * Keeps og.Camera on the entity carrying it in the xy plane, for 2D games: the camera moves only when the entity leaves the dead zone, eases by smoothTime and, with clamp, stays inside the bounds. The camera keeps its z, rotation and projection. Driven by og.systems.cameras.
 *
 * The value of the `Follow2D` component (token `og.Follow2D`).
 */
export interface Follow2D {
  /**
   * Where the camera centre sits relative to the entity.
   *
   * @unit m (metres)
   */
  offset: Vector2;
  /**
   * Half the width and height of the box around the camera centre inside which the entity moves without moving the camera.
   *
   * @unit m (metres)
   */
  deadZone: Vector2;
  /**
   * Roughly how long the camera takes to catch up; 0 snaps it every tick.
   *
   * @default 0.15
   * @range 0..
   * @unit s (seconds of simulation time)
   * @tunable
   */
  smoothTime: number;
  /** Keep the camera centre between boundsMin and boundsMax. */
  clamp: boolean;
  /**
   * The smallest x and y the camera centre takes with clamp.
   *
   * @unit m (metres)
   */
  boundsMin: Vector2;
  /**
   * The largest x and y the camera centre takes with clamp.
   *
   * @unit m (metres)
   */
  boundsMax: Vector2;
}

// ── render ──────────────────────────────────────────────────────
// What is drawn: meshes with materials. Draws are instanced by (geometry, material) automatically; there is no per-object draw call.

/**
 * Vertex and index data, as three.js BufferGeometry. Comes from a loaded glTF.
 *
 * A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.
 */
export type Geometry = number & { readonly [handleType]: "Geometry" };

/**
 * A material instance.
 *
 * A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.
 */
export type Material = number & { readonly [handleType]: "Material" };

/**
 * A cooked texture.
 *
 * A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.
 */
export type Texture = number & { readonly [handleType]: "Texture" };

/**
 * A Geometry handle field.
 *
 * A `Geometry` field: stored in a `Uint32Array` with stride 2.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function Geometry(value?: Geometry, options?: FieldOptions): FieldSpec<"Geometry">;

/**
 * A Material handle field.
 *
 * A `Material` field: stored in a `Uint32Array` with stride 2.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function Material(value?: Material, options?: FieldOptions): FieldSpec<"Material">;

/**
 * Which faces are drawn, as three.js FrontSide, BackSide and DoubleSide.
 * - `"front"`: Front faces only.
 * - `"back"`: Back faces only.
 * - `"double"`: Both.
 */
export type Side = "front" | "back" | "double";

/**
 * The backend the renderer runs on; the probe's hello reports it.
 * - `"webgpu"`: WebGPU in a browser: the full feature set.
 * - `"webgl2"`: The WebGL2 fallback: no compute, CPU culling and particles, tiled lights.
 * - `"vulkan"`: Native Vulkan, on lavapipe or a GPU.
 * - `"metal"`: Native Metal.
 * - `"none"`: No renderer: frames are not produced and capture refuses.
 */
export type RenderTier = "webgpu" | "webgl2" | "vulkan" | "metal" | "none";

/**
 * Antialiasing, as og-render's Antialias. RenderCapabilities.antialias reports what the device resolved.
 * - `"tier"`: The tier's method: TAA on the full tier (WebGPU, Metal, Vulkan), 4x MSAA on WebGL2.
 * - `"off"`: One sample per pixel.
 * - `"msaa4"`: 4x MSAA on any tier: sharp sprites and text, no temporal blur; off where the HDR format cannot be multisampled.
 * - `"taa"`: Temporal antialiasing on any tier: a sub-pixel jitter per frame and a reprojected history; converges over about eight frames.
 */
export type Antialias = "tier" | "off" | "msaa4" | "taa";

/**
 * How the renderer is created for a game (GameDesc.render). The host reads it once, when it attaches the renderer.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface RenderOptions {
  /**
   * The antialiasing method.
   *
   * @default "tier"
   */
  readonly antialias?: Antialias;
  /**
   * The side of each of the directional light's three cascade maps, in texels; smaller saves memory on phones (three 2048 maps are 64 MB).
   *
   * @default 2048
   * @range 256..4096
   */
  readonly shadowMapSize?: number;
}

/**
 * A physically based metallic-roughness material with three.js MeshStandardMaterial parameter names.
 * Mapping to the renderer (og-render Material): color and opacity are the base colour and its alpha; map the base colour texture; metalness and roughness the factors; metalnessMap and roughnessMap are one glTF metallic-roughness texture (blue metal, green rough), so give both the same texture; normalMap the normal texture; emissive times emissiveIntensity the emitted colour, emissiveMap its texture; alphaTest above 0 is alpha masking with that cutoff.
 * Accepted and kept but not drawn yet: aoMap, transparent (blending), side (meshes draw front faces), flatShading and envMapIntensity; the renderer's own phase-2 list owns them.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface StandardMaterialDesc {
  /**
   * The base colour.
   *
   * @default { r: 1, g: 1, b: 1 }
   */
  readonly color?: Color;
  /**
   * The base colour texture.
   *
   * @default null
   */
  readonly map?: Texture | null;
  /**
   * How metallic the surface is.
   *
   * @default 0
   * @range 0..1
   */
  readonly metalness?: number;
  /**
   * Metalness from the texture's blue channel.
   *
   * @default null
   */
  readonly metalnessMap?: Texture | null;
  /**
   * How rough the surface is.
   *
   * @default 1
   * @range 0..1
   */
  readonly roughness?: number;
  /**
   * Roughness from the texture's green channel.
   *
   * @default null
   */
  readonly roughnessMap?: Texture | null;
  /**
   * A tangent-space normal map.
   *
   * @default null
   */
  readonly normalMap?: Texture | null;
  /**
   * Ambient occlusion from the texture's red channel.
   *
   * @default null
   */
  readonly aoMap?: Texture | null;
  /**
   * The emitted colour.
   *
   * @default { r: 0, g: 0, b: 0 }
   */
  readonly emissive?: Color;
  /**
   * A multiplier on the emitted colour.
   *
   * @default 1
   * @range 0..
   */
  readonly emissiveIntensity?: number;
  /**
   * The emitted colour texture.
   *
   * @default null
   */
  readonly emissiveMap?: Texture | null;
  /**
   * The opacity; below 1 only with transparent.
   *
   * @default 1
   * @range 0..1
   */
  readonly opacity?: number;
  /**
   * Blend instead of writing opaque pixels.
   *
   * @default false
   */
  readonly transparent?: boolean;
  /**
   * Discard pixels whose alpha is below this.
   *
   * @default 0
   * @range 0..1
   */
  readonly alphaTest?: number;
  /**
   * Which faces are drawn.
   *
   * @default "front"
   */
  readonly side?: Side;
  /**
   * Shade with face normals.
   *
   * @default false
   */
  readonly flatShading?: boolean;
  /**
   * A multiplier on image-based lighting.
   *
   * @default 1
   * @range 0..
   */
  readonly envMapIntensity?: number;
}

/**
 * Draws a geometry with a material at the entity's MatrixWorld, as a three.js Mesh.
 * The material is a handle from og.render.standardMaterial or from a loaded glTF (GltfInfo.materials); its StandardMaterialDesc maps to the renderer as that struct's docs say.
 *
 * The value of the `Mesh` component (token `og.Mesh`).
 */
export interface Mesh {
  /** The geometry. */
  geometry: Geometry;
  /** The material. */
  material: Material;
  /** Cast shadows, as three.js Object3D.castShadow. */
  castShadow: boolean;
  /** Receive shadows, as three.js Object3D.receiveShadow. */
  receiveShadow: boolean;
  /**
   * Whether it is drawn.
   *
   * @default true
   */
  visible: boolean;
}

/**
 * The sun, as three.js DirectionalLight: parallel light along the entity's local -Z (aim it with transform.lookAt, as a camera). The first one found is drawn, with three shadow cascades.
 * Without one, the renderer's default sun (from above and to the side, intensity 3) lights the scene.
 *
 * The value of the `DirectionalLight` component (token `og.DirectionalLight`).
 */
export interface DirectionalLight {
  /**
   * The light's linear colour.
   *
   * @default { r: 1, g: 1, b: 1 }
   */
  color: Color;
  /**
   * The radiance multiplier, as three.js DirectionalLight.intensity.
   *
   * @default 3
   * @range 0..
   * @tunable
   */
  intensity: number;
  /**
   * Cast shadows through three cascades, as three.js Light.castShadow.
   *
   * @default true
   */
  castShadow: boolean;
  /**
   * How far from the camera shadows reach; they fade over the last tenth. Smaller is sharper.
   *
   * @default 200
   * @range 1..
   * @unit m (metres)
   */
  shadowDistance: number;
}

/**
 * A light shining in every direction from the entity's position, as three.js PointLight. Its radiance at distance d is color * intensity / d^2, faded smoothly to zero at distance.
 * Up to MAX_LIGHTS point and spot lights are drawn per frame (the rest are counted, not drawn); the first two with castShadow get cube shadows.
 *
 * The value of the `PointLight` component (token `og.PointLight`).
 */
export interface PointLight {
  /**
   * The light's linear colour.
   *
   * @default { r: 1, g: 1, b: 1 }
   */
  color: Color;
  /**
   * The radiance at one metre.
   *
   * @default 10
   * @range 0..
   * @tunable
   */
  intensity: number;
  /**
   * Where the light reaches zero, as three.js PointLight.distance (never infinite here).
   *
   * @default 10
   * @range 0.01..
   * @unit m (metres)
   * @tunable
   */
  distance: number;
  /** Cast cube shadows. */
  castShadow: boolean;
}

/**
 * A cone of light along the entity's local -Z, as three.js SpotLight.
 *
 * The value of the `SpotLight` component (token `og.SpotLight`).
 */
export interface SpotLight {
  /**
   * The light's linear colour.
   *
   * @default { r: 1, g: 1, b: 1 }
   */
  color: Color;
  /**
   * The radiance at one metre on the axis.
   *
   * @default 10
   * @range 0..
   * @tunable
   */
  intensity: number;
  /**
   * Where the light reaches zero.
   *
   * @default 10
   * @range 0.01..
   * @unit m (metres)
   * @tunable
   */
  distance: number;
  /**
   * The half-angle of the outer cone, as three.js SpotLight.angle.
   *
   * @default 0.5236
   * @range 0..1.5708
   * @unit rad (radians)
   * @tunable
   */
  angle: number;
  /**
   * The fraction of the cone over which the light fades out, as three.js SpotLight.penumbra.
   *
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   */
  penumbra: number;
  /** Cast a shadow map. */
  castShadow: boolean;
}

/**
 * The frame's look after lighting: environment light, sky, bloom, exposure metering and tone mapping. Read after each tick; it never feeds back into the simulation. Camera.exposure is the manual exposure.
 *
 * The `RenderSettings` resource (token `og.RenderSettings`).
 */
export interface RenderSettings {
  /**
   * The strength of the image-based ambient light from the procedural sky.
   *
   * @default 0.55
   * @range 0..
   * @tunable
   */
  environmentIntensity: number;
  /** Draw the procedural sky behind the scene instead of Camera.background. */
  sky: boolean;
  /** Light above the threshold bleeds into its surroundings. */
  bloom: boolean;
  /**
   * Brightness after exposure where bloom starts; 1 is display white.
   *
   * @default 1
   * @range 0..
   * @tunable
   */
  bloomThreshold: number;
  /**
   * Softens the threshold over threshold plus or minus knee; 0 is a hard cut.
   *
   * @default 0.5
   * @range 0..
   */
  bloomKnee: number;
  /**
   * How much of the blurred light is added back.
   *
   * @default 0.6
   * @range 0..
   * @tunable
   */
  bloomIntensity: number;
  /**
   * Meter every frame to middle grey (deterministic: the current frame only), then apply Camera.exposure as compensation.
   */
  autoExposure: boolean;
  /**
   * ACES tone mapping; false clamps, so 2D colours come out exactly as authored at exposure 1.
   *
   * @default true
   */
  toneMapped: boolean;
}

/**
 * A line or block of MSDF text in the world's xy plane facing +z, drawn this frame only (immediate mode: submit it every tick it should show).
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface TextDesc {
  /**
   * The font from og.load.font; null draws nothing.
   *
   * @default null
   */
  readonly font?: Font | null;
  /**
   * The text; `\n` starts a new line.
   *
   * @default ""
   */
  readonly text?: string;
  /**
   * The baseline origin of the first line.
   *
   * @default { x: 0, y: 0, z: 0 }
   * @unit m (metres)
   */
  readonly position?: Vector3;
  /**
   * The em size in world units.
   *
   * @default 1
   * @range 0..
   */
  readonly size?: number;
  /**
   * Rotation about +z.
   *
   * @default 0
   * @unit rad (radians)
   */
  readonly rotation?: number;
  /**
   * The linear colour.
   *
   * @default { r: 1, g: 1, b: 1 }
   */
  readonly color?: Color;
  /**
   * The alpha.
   *
   * @default 1
   * @range 0..1
   */
  readonly opacity?: number;
  /**
   * Draw order among sprites and text; higher draws on top.
   *
   * @default 0
   */
  readonly layer?: number;
  /**
   * How each line sits on `position`: start puts its left end there, center its middle, end its right end.
   *
   * @default "start"
   */
  readonly align?: TextAlign;
}

/**
 * Text to measure with og.render.measureText, laid out as og.render.text lays it out.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface TextMeasureDesc {
  /**
   * The font from og.load.font.
   *
   * @default null
   */
  readonly font?: Font | null;
  /**
   * The text; `\n` starts a new line.
   *
   * @default ""
   */
  readonly text?: string;
  /**
   * The em size in world units.
   *
   * @default 1
   * @range 0..
   */
  readonly size?: number;
}

/** The extent of a laid-out text, in world units. */
export interface TextMetrics {
  /** The widest line's advance. */
  readonly width: number;
  /** Lines times the font's line height. */
  readonly height: number;
  /** How many lines. */
  readonly lines: number;
}

/**
 * A textured or solid quad in the world's xy plane facing +z, drawn this frame only (immediate mode, as og.render.text): submit it every tick it should show. Sprites and text are sorted by layer and drawn over the 3D scene.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface SpriteDesc {
  /**
   * The texture from og.load.texture; null draws a solid quad of the colour.
   *
   * @default null
   */
  readonly texture?: Texture | null;
  /**
   * Where the anchor point is.
   *
   * @default { x: 0, y: 0, z: 0 }
   * @unit m (metres)
   */
  readonly position?: Vector3;
  /**
   * Its width in world units.
   *
   * @default 1
   * @range 0..
   * @unit m (metres)
   */
  readonly width?: number;
  /**
   * Its height in world units.
   *
   * @default 1
   * @range 0..
   * @unit m (metres)
   */
  readonly height?: number;
  /**
   * The point of the quad at `position`, as a fraction of its size from the bottom-left corner: (0.5, 0.5) is the centre.
   *
   * @default { x: 0.5, y: 0.5 }
   */
  readonly anchor?: Vector2;
  /**
   * Counter-clockwise rotation about the anchor.
   *
   * @default 0
   * @unit rad (radians)
   */
  readonly rotation?: number;
  /**
   * A linear colour multiplied with the texture.
   *
   * @default { r: 1, g: 1, b: 1 }
   */
  readonly color?: Color;
  /**
   * The alpha.
   *
   * @default 1
   * @range 0..1
   */
  readonly opacity?: number;
  /**
   * The texture region drawn, [u0, v0, u1, v1] with v down; null is the whole texture (a sheet's frame for animation).
   *
   * @default null
   */
  readonly uv?: [number, number, number, number] | null;
  /**
   * Draw order among sprites and text; higher draws on top.
   *
   * @default 0
   */
  readonly layer?: number;
}



/**
 * A box centred at the origin, as three.js BoxGeometry(width, height, depth).
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface BoxGeometryDesc {
  /**
   * Its extent along x.
   *
   * @default 1
   * @range 0..
   * @unit m (metres)
   */
  readonly width?: number;
  /**
   * Its extent along y.
   *
   * @default 1
   * @range 0..
   * @unit m (metres)
   */
  readonly height?: number;
  /**
   * Its extent along z.
   *
   * @default 1
   * @range 0..
   * @unit m (metres)
   */
  readonly depth?: number;
}

/**
 * A rectangle in the xy plane facing +z, centred at the origin, as three.js PlaneGeometry(width, height); turn it -pi/2 about x for a floor.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface PlaneGeometryDesc {
  /**
   * Its extent along x.
   *
   * @default 1
   * @range 0..
   * @unit m (metres)
   */
  readonly width?: number;
  /**
   * Its extent along y.
   *
   * @default 1
   * @range 0..
   * @unit m (metres)
   */
  readonly height?: number;
}

/**
 * A UV sphere centred at the origin, as three.js SphereGeometry(radius, widthSegments, heightSegments).
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface SphereGeometryDesc {
  /**
   * Its radius.
   *
   * @default 1
   * @range 0..
   * @unit m (metres)
   */
  readonly radius?: number;
  /**
   * Segments around the equator.
   *
   * @default 32
   * @range 3..
   */
  readonly widthSegments?: number;
  /**
   * Rings from pole to pole.
   *
   * @default 16
   * @range 2..
   */
  readonly heightSegments?: number;
}




/** The most point and spot lights one frame draws. */
export declare const MAX_LIGHTS: 8;

/**
 * What the renderer can do on this device. Backend limits are reported here, never by missing members.
 */
export interface RenderCapabilities {
  /** The backend. */
  readonly tier: RenderTier;
  /** Whether compute passes run (GPU culling, GPU particles). */
  readonly compute: boolean;
  /** The antialiasing the device resolved RenderOptions.antialias to. */
  readonly antialias: Antialias;
  /** How many point and spot lights one frame draws. */
  readonly maxLights: number;
  /** How many point and spot lights may cast shadows. */
  readonly maxShadowedLights: number;
  /** The render target width in device pixels. */
  readonly width: number;
  /** The render target height in device pixels. */
  readonly height: number;
}





// ── load ────────────────────────────────────────────────────────
// Asset loading from cooked output: `og cook` turns assets/ into cooked/ (.ogb blobs and the manifest cooked/.cook.ogt), and the runtime reads only cooked/.
// Paths are asset ids: the path under assets/ (`models/ship.glb`, `scenes/main.ogt`); an `assets/` prefix is accepted. Every blob is checked against the manifest's type, schema version and cook stamp.
// The game's bundle (`cooked`) is opened before the first tick and GameDesc.scene is spawned then. Loads from an open bundle complete at once on both hosts (the web host fetches the bundle's files before the engine starts); only og.load.bundle of another bundle can be pending, and it completes at a tick boundary.

/**
 * A loaded bundle, scene or glTF asset.
 *
 * A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.
 */
export type Asset = number & { readonly [handleType]: "Asset" };

/**
 * A cooked MSDF font: its glyph table and one or more atlas pages.
 *
 * A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.
 */
export type Font = number & { readonly [handleType]: "Font" };

/**
 * A cooked sound: QOA-encoded PCM the audio module plays.
 *
 * A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.
 */
export type Sound = number & { readonly [handleType]: "Sound" };

/**
 * An Asset handle field.
 *
 * A `Asset` field: stored in a `Uint32Array` with stride 2.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function Asset(value?: Asset, options?: FieldOptions): FieldSpec<"Asset">;

/**
 * A Sound handle field.
 *
 * A `Sound` field: stored in a `Uint32Array` with stride 2.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function Sound(value?: Sound, options?: FieldOptions): FieldSpec<"Sound">;

/**
 * Where a load is.
 * - `"pending"`: Still loading.
 * - `"ready"`: Loaded; it can be instantiated.
 * - `"failed"`: Failed; og.load.error gives the reason.
 */
export type LoadStatus = "pending" | "ready" | "failed";

/**
 * How to load a glTF, as three.js GLTFLoader options where they apply.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface GltfDesc {
  /**
   * Which glTF scene to instantiate.
   *
   * @default 0
   */
  readonly scene?: number;
  /**
   * Set Mesh.castShadow on every mesh instantiated from it.
   *
   * @default false
   */
  readonly castShadow?: boolean;
  /**
   * Set Mesh.receiveShadow on every mesh instantiated from it.
   *
   * @default false
   */
  readonly receiveShadow?: boolean;
}

/** What a loaded glTF holds. */
export interface GltfInfo {
  /** How many scenes it has. */
  readonly scenes: number;
  /** Its geometries, in glTF mesh-primitive order. */
  readonly geometries: readonly Geometry[];
  /** Its materials, in glTF order, then the default material. */
  readonly materials: readonly Material[];
  /**
   * Its animation clip names (the cook counts clips and does not cook them yet, so this is empty).
   */
  readonly animations: readonly string[];
}

/** An open cooked bundle. */
export interface BundleInfo {
  /** The cooked directory it was opened from. */
  readonly path: string;
  /** Source assets it holds. */
  readonly assets: number;
  /** Cooked outputs (blobs) it holds. */
  readonly outputs: number;
}

/** A loaded font's metrics, in em units unless stated. */
export interface FontInfo {
  /** Glyphs cooked into it. */
  readonly glyphs: number;
  /** Atlas pages; the renderer draws one page per pass with one shared layout. */
  readonly pages: number;
  /** Atlas pixels per em. */
  readonly emPx: number;
  /** The distance range in atlas pixels. */
  readonly pxRange: number;
  /** Ascender above the baseline. */
  readonly ascender: number;
  /** Descender below the baseline (negative). */
  readonly descender: number;
  /** Ascender minus descender plus the line gap. */
  readonly lineHeight: number;
}

/** A loaded sound. */
export interface SoundInfo {
  /**
   * Samples per second.
   *
   * @unit hz (ticks or cycles per second)
   */
  readonly rate: number;
  /** Interleaved channels. */
  readonly channels: number;
  /** Samples per channel. */
  readonly frames: number;
  /**
   * Its length.
   *
   * @unit s (seconds of simulation time)
   */
  readonly seconds: number;
}

/**
 * Draws a loaded glTF's default scene at the entity's world transform: every mesh node with its glTF material. Scene entities with a `model` line get one; world.instantiate of a glTF asset spawns one.
 *
 * The value of the `Model` component (token `og.Model`).
 */
export interface Model {
  /** The glTF asset. */
  asset: Asset;
}















/**
 * A load completed.
 *
 * The payload of the `AssetReady` event (token `og.AssetReady`).
 */
export interface AssetReady {
  /** The handle. */
  readonly asset: Asset;
  /** The path it was loaded from. */
  readonly path: string;
}

/**
 * A load failed.
 *
 * The payload of the `AssetFailed` event (token `og.AssetFailed`).
 */
export interface AssetFailed {
  /** The handle. */
  readonly asset: Asset;
  /** The path it was loaded from. */
  readonly path: string;
  /** Why, naming the fix. */
  readonly reason: string;
}

// ── physics ─────────────────────────────────────────────────────
// Rigid bodies over rapier (rapier3d in the 3D profile, rapier2d in 2D, both with enhanced determinism): game code owns the components, the engine's og.physics system steps them after the game's systems and writes the dynamic bodies back.
// The profile is the project's (`profile "2d"` or `"3d"` in project.ogt). In 2D, bodies move in the xy plane and rotate about z; a transform keeps its z. A collider without a RigidBody is static at its transform.
// The rapier world persists between ticks and is world state: snapshots, restore and the world hash cover the whole simulation. Events sent in tick N are read in tick N + 1.

/**
 * RigidBody.kind: moved by the solver; its Transform and Velocity are written back every tick.
 */
export declare const BODY_DYNAMIC: 0;

/** RigidBody.kind: never moves; collides with dynamic bodies. */
export declare const BODY_STATIC: 1;

/**
 * RigidBody.kind: moved by game code through its Transform; its Velocity, if any, is how fast it pushes.
 */
export declare const BODY_KINEMATIC: 2;

/** RigidBody.lock bit: no translation along x. */
export declare const LOCK_TRANSLATION_X: 1;

/** RigidBody.lock bit: no translation along y. */
export declare const LOCK_TRANSLATION_Y: 2;

/** RigidBody.lock bit: no translation along z. */
export declare const LOCK_TRANSLATION_Z: 4;

/** RigidBody.lock bit: no rotation about x. */
export declare const LOCK_ROTATION_X: 8;

/** RigidBody.lock bit: no rotation about y. */
export declare const LOCK_ROTATION_Y: 16;

/** RigidBody.lock bit: no rotation about z (the 2D profile's only rotation). */
export declare const LOCK_ROTATION_Z: 32;

/** Collider.shape: a box of `size` (both profiles). */
export declare const SHAPE_BOX: 0;

/** Collider.shape: a sphere of `radius` (a circle in 2D). */
export declare const SHAPE_SPHERE: 1;

/**
 * Collider.shape: a capsule along y of `radius`, with `height` between the cap centres (both profiles).
 */
export declare const SHAPE_CAPSULE: 2;

/** Collider.shape: a cylinder along y of `radius` and `height` (3D only). */
export declare const SHAPE_CYLINDER: 3;

/**
 * Collider.shape: the convex hull of the vertices of `mesh` (3D), or of their xy (a convex polygon, 2D).
 */
export declare const SHAPE_CONVEX: 4;

/** Collider.shape: the triangles of `mesh` (3D only; for static and kinematic bodies). */
export declare const SHAPE_TRIMESH: 5;

/** Physics.profile: rapier2d, bodies in the xy plane rotating about z. */
export declare const PROFILE_2D: 2;

/** Physics.profile: rapier3d. */
export declare const PROFILE_3D: 3;

/**
 * A body the physics step simulates. It needs an og.Transform and usually has an og.Collider; a dynamic body without an og.Velocity gets one.
 * Changing a RigidBody, a Collider or Transform.scale rebuilds the body (it loses its contacts' warm starts); a Transform or Velocity that game code changed moves it (a teleport).
 *
 * The value of the `RigidBody` component (token `og.RigidBody`).
 */
export interface RigidBody {
  /**
   * BODY_DYNAMIC (0, simulated and written back), BODY_STATIC (1, never moves) or BODY_KINEMATIC (2, moved by game code through its Transform).
   *
   * @range 0..2
   * @label Kind
   */
  kind: number;
  /**
   * Mass; 0 derives it from the collider's density and volume.
   *
   * @range 0..
   * @unit kg (kilograms)
   * @tunable
   */
  mass: number;
  /**
   * How fast linear velocity decays (0: none).
   *
   * @range 0..100
   * @unit perSecond (a rate per second: the fraction of a value that decays in one second)
   * @tunable
   */
  linearDamping: number;
  /**
   * How fast angular velocity decays (0: none).
   *
   * @range 0..100
   * @unit perSecond (a rate per second: the fraction of a value that decays in one second)
   * @tunable
   */
  angularDamping: number;
  /**
   * Multiplies Physics.gravity for this body: 0 floats, 1 is normal.
   *
   * @default 1
   * @range -10..10
   * @tunable
   */
  gravityScale: number;
  /**
   * Locked axes, a sum of the LOCK_* bits: 1, 2, 4 no translation along x, y, z; 8, 16, 32 no rotation about x, y, z (2D uses 1, 2 and 32).
   *
   * @range 0..63
   */
  lock: number;
  /**
   * 1 enables continuous collision detection, for fast small bodies that must not tunnel; 0 off.
   *
   * @range 0..1
   */
  ccd: number;
}

/**
 * The shape an entity collides with, attached to its RigidBody (or, without one, a static collider at its Transform). Sizes are scaled by og.Transform.scale.
 *
 * The value of the `Collider` component (token `og.Collider`).
 */
export interface Collider {
  /**
   * One of the SHAPE_* codes: 0 box, 1 sphere (a circle in 2D), 2 capsule, 3 cylinder (3D), 4 convex hull of `mesh` (a polygon in 2D), 5 triangle mesh of `mesh` (3D).
   *
   * @range 0..5
   * @label Shape
   */
  shape: number;
  /**
   * Full extents of a box, as three.js BoxGeometry(width, height, depth).
   *
   * @default { x: 1, y: 1, z: 1 }
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  size: Vector3;
  /**
   * Radius of a sphere, circle, capsule or cylinder.
   *
   * @default 0.5
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  radius: number;
  /**
   * A capsule's distance between its cap centres (three.js CapsuleGeometry length); a cylinder's height.
   *
   * @default 1
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  height: number;
  /**
   * The geometry whose vertices a convex or triangle-mesh shape uses: a loaded glTF's geometry (registered automatically) or one given to og.physics.setMesh.
   */
  mesh: Geometry;
  /**
   * Position of the shape relative to the entity.
   *
   * @unit m (metres)
   */
  offset: Vector3;
  /** Rotation of the shape relative to the entity. */
  rotation: Quaternion;
  /**
   * Coulomb friction (0 is ice, 1 rubber); a pair uses the mean of both colliders.
   *
   * @default 0.5
   * @range 0..2
   * @tunable
   */
  friction: number;
  /**
   * Bounciness (0 no bounce, 1 elastic); a pair uses the mean of both colliders.
   *
   * @range 0..1
   * @tunable
   */
  restitution: number;
  /**
   * Density, used when RigidBody.mass is 0.
   *
   * @default 1
   * @range 0..
   * @unit kgPerM3 (kilograms per cubic metre (per square metre in the 2D profile))
   * @tunable
   */
  density: number;
  /**
   * 1 makes this a sensor: it reports SensorEntered and SensorExited and pushes nothing; 0 a solid.
   *
   * @range 0..1
   */
  sensor: number;
  /**
   * The collision layers this collider is on, as bits.
   *
   * @default 1
   */
  layers: number;
  /**
   * The layers it collides with, as bits. Two colliders interact when each one's layers meet the other's mask.
   *
   * @default 4294967295
   */
  mask: number;
}

/**
 * Linear and angular velocity. Written for dynamic bodies every tick (added to them by the step when missing), read for kinematic ones; set it to launch or stop a body.
 *
 * The value of the `Velocity` component (token `og.Velocity`).
 */
export interface Velocity {
  /**
   * The linear velocity (2D uses x and y).
   *
   * @unit mps (metres per second)
   * @tunable
   */
  linear: Vector3;
  /**
   * The angular velocity about each axis (2D uses z).
   *
   * @unit radps (radians per second)
   * @tunable
   */
  angular: Vector3;
}

/**
 * The physics settings of the world. The profile is the project's; the same components serve both profiles.
 *
 * The `Physics` resource (token `og.Physics`).
 */
export interface Physics {
  /**
   * PROFILE_2D (2: rapier2d, the xy plane, rotation about z) or PROFILE_3D (3: rapier3d), from project.ogt.
   *
   * @default 3
   * @range 2..3
   * @label Profile
   * @readonly written by the engine
   */
  readonly profile: number;
  /**
   * The acceleration of every dynamic body, times its gravityScale; 2D uses x and y.
   *
   * @default { x: 0, y: -9.81, z: 0 }
   * @unit mps2 (metres per second squared)
   * @tunable
   */
  gravity: Vector3;
  /**
   * Physics steps per tick, each of dt / substeps: more is stiffer stacks and less tunnelling, at more cost.
   *
   * @default 1
   * @range 1..16
   * @tunable
   */
  substeps: number;
  /**
   * Solver iterations per step.
   *
   * @default 4
   * @range 1..32
   * @tunable
   */
  solverIterations: number;
  /**
   * A dynamic body touching something, supported with at most 2 mm of overlap and slower than this before and after a step, keeps its pose and loses its residual velocity (the same number in m/s and rad/s); 0 disables it.
   *
   * @default 0.02
   * @range 0..1
   * @unit mps (metres per second)
   * @tunable
   */
  restSpeed: number;
}

/**
 * Two solid colliders started touching during the tick.
 *
 * The payload of the `CollisionStarted` event (token `og.CollisionStarted`).
 */
export interface CollisionStarted {
  /** The entity with the lower id. */
  readonly a: Entity;
  /** The entity with the higher id. */
  readonly b: Entity;
  /** The deepest contact point in world space (z is 0 in 2D). */
  readonly point: Vector3;
  /** The contact normal, from a towards b. */
  readonly normal: Vector3;
}

/**
 * Two solid colliders stopped touching, or one of them went away.
 *
 * The payload of the `CollisionEnded` event (token `og.CollisionEnded`).
 */
export interface CollisionEnded {
  /** The entity with the lower id. */
  readonly a: Entity;
  /** The entity with the higher id. */
  readonly b: Entity;
}

/**
 * A collider started overlapping a sensor.
 *
 * The payload of the `SensorEntered` event (token `og.SensorEntered`).
 */
export interface SensorEntered {
  /** The entity whose collider is the sensor. */
  readonly sensor: Entity;
  /** The entity that entered it. */
  readonly other: Entity;
}

/**
 * A collider stopped overlapping a sensor, or one of them went away.
 *
 * The payload of the `SensorExited` event (token `og.SensorExited`).
 */
export interface SensorExited {
  /** The entity whose collider is the sensor. */
  readonly sensor: Entity;
  /** The entity that left it. */
  readonly other: Entity;
}

/**
 * One pair in contact at the end of a step, sent every tick for every touching pair: how a game asks what is touching what right now.
 *
 * The payload of the `Contact` event (token `og.Contact`).
 */
export interface Contact {
  /** The entity with the lower id. */
  readonly a: Entity;
  /** The entity with the higher id. */
  readonly b: Entity;
  /**
   * 0 for two solids; an overlap with a sensor otherwise: 1 a is the sensor, 2 b is, 3 both are.
   */
  readonly sensor: number;
}

/**
 * Which colliders a scene query considers. The zero value considers every solid collider on every layer.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface QueryFilter {
  /**
   * Layers to consider: a collider counts when its layers meet this; 0 means every layer.
   *
   * @default 0
   */
  readonly mask?: number;
  /**
   * Also report sensors.
   *
   * @default false
   */
  readonly includeSensors?: boolean;
  /**
   * An entity to ignore, such as the caster itself.
   *
   * @default null
   */
  readonly exclude?: Entity | null;
}

/**
 * A ray for og.physics.raycast.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface RaycastDesc {
  /**
   * Where the ray starts.
   *
   * @default { x: 0, y: 0, z: 0 }
   * @unit m (metres)
   */
  readonly origin?: Vector3;
  /**
   * Where it points; it need not be normalised and must not be zero (in 2D, not zero in xy).
   *
   * @default { x: 0, y: -1, z: 0 }
   */
  readonly direction?: Vector3;
  /**
   * How far it reaches; 0 is unbounded.
   *
   * @default 0
   * @range 0..
   * @unit m (metres)
   */
  readonly maxDistance?: number;
  /**
   * Which colliders count.
   *
   * @default {}
   */
  readonly filter?: QueryFilter;
}

/** The nearest collider a ray meets. */
export interface RaycastHit {
  /** The collider's entity. */
  readonly entity: Entity;
  /**
   * From the origin along the normalised direction.
   *
   * @unit m (metres)
   */
  readonly distance: number;
  /** Where the ray meets the collider. */
  readonly point: Vector3;
  /** The surface normal there (z is 0 in 2D). */
  readonly normal: Vector3;
}

/**
 * The shape of a shape cast or an overlap test: the shape fields of a Collider. The zero value is a 1 m box.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ShapeDesc {
  /**
   * One of the SHAPE_* codes.
   *
   * @default 0
   * @range 0..5
   */
  readonly shape?: number;
  /**
   * Full extents of a box.
   *
   * @default { x: 1, y: 1, z: 1 }
   * @unit m (metres)
   */
  readonly size?: Vector3;
  /**
   * Radius of a sphere, circle, capsule or cylinder.
   *
   * @default 0.5
   * @unit m (metres)
   */
  readonly radius?: number;
  /**
   * Capsule or cylinder height.
   *
   * @default 1
   * @unit m (metres)
   */
  readonly height?: number;
  /**
   * The geometry of a convex or triangle-mesh shape.
   *
   * @default 0
   */
  readonly mesh?: Geometry;
}

/**
 * A shape swept along a direction for og.physics.shapeCast.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ShapeCastDesc {
  /**
   * The swept shape.
   *
   * @default {}
   */
  readonly shape?: ShapeDesc;
  /**
   * Where the sweep starts.
   *
   * @default { x: 0, y: 0, z: 0 }
   * @unit m (metres)
   */
  readonly position?: Vector3;
  /**
   * The shape's rotation; a zero quaternion is no rotation.
   *
   * @default { x: 0, y: 0, z: 0, w: 1 }
   */
  readonly rotation?: Quaternion;
  /**
   * Where it sweeps; it need not be normalised and must not be zero.
   *
   * @default { x: 0, y: -1, z: 0 }
   */
  readonly direction?: Vector3;
  /**
   * How far it sweeps; 0 is unbounded.
   *
   * @default 0
   * @range 0..
   * @unit m (metres)
   */
  readonly maxDistance?: number;
  /**
   * Which colliders count.
   *
   * @default {}
   */
  readonly filter?: QueryFilter;
}

/** The first collider a swept shape meets. */
export interface ShapeCastHit {
  /** The collider's entity. */
  readonly entity: Entity;
  /**
   * How far the shape travelled before touching.
   *
   * @unit m (metres)
   */
  readonly distance: number;
  /** The contact point on the hit collider, in world space. */
  readonly point: Vector3;
  /** The hit collider's surface normal there, in world space. */
  readonly normal: Vector3;
}

/**
 * A shape placed in the world for og.physics.overlap.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface OverlapDesc {
  /**
   * The shape.
   *
   * @default {}
   */
  readonly shape?: ShapeDesc;
  /**
   * Where it is.
   *
   * @default { x: 0, y: 0, z: 0 }
   * @unit m (metres)
   */
  readonly position?: Vector3;
  /**
   * Its rotation; a zero quaternion is no rotation.
   *
   * @default { x: 0, y: 0, z: 0, w: 1 }
   */
  readonly rotation?: Quaternion;
  /**
   * Which colliders count.
   *
   * @default {}
   */
  readonly filter?: QueryFilter;
}

/**
 * Geometry for convex and triangle-mesh colliders, in the mesh's own space.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface MeshDesc {
  /**
   * Vertex positions (2D polygons use x and y).
   *
   * @default []
   */
  readonly positions?: readonly Vector3[];
  /**
   * Triangles as triples of indices into positions; a convex hull ignores them.
   *
   * @default []
   */
  readonly indices?: readonly number[];
}





// ── anim ────────────────────────────────────────────────────────
// Skeletal animation over og-anim: glTF skins and clips (cooked into the `<model>#anim` blob, loaded with og.load.animation), blend-tree graphs with layered state machines, root motion, two-bone and FABRIK IK and blendshapes, evaluated by the engine's og.anim system after the game's systems and before physics.
// Game code owns the components and sets players, graph parameters and IK targets; og.anim writes the pose (kept as world state in the `og.anim` state block, so snapshots, restore and the world hash cover it), the joint palette, root motion into og.Transform and the fields marked written. Asset handles are what og.load.animation and og.anim.graph answer; 0 is none.
// Clips bind to a skeleton's nodes by name, so a clip of one rig plays on another with the same node names. Assets are not world state: a hot reload registers them again.

/**
 * A rig: a glTF's nodes in topological order with their rest transforms, and its skins (joints and inverse bind matrices).
 *
 * A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.
 */
export type SkeletonAsset = number & { readonly [handleType]: "SkeletonAsset" };

/**
 * An animation clip, resampled at the cook's rate, with its markers.
 *
 * A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.
 */
export type ClipAsset = number & { readonly [handleType]: "ClipAsset" };

/**
 * A blend-tree graph (`type animgraph`) whose clip references were resolved to clip handles.
 *
 * A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.
 */
export type GraphAsset = number & { readonly [handleType]: "GraphAsset" };

/**
 * A SkeletonAsset handle field.
 *
 * A `SkeletonAsset` field: stored in a `Uint32Array` with stride 2.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function SkeletonAsset(value?: SkeletonAsset, options?: FieldOptions): FieldSpec<"SkeletonAsset">;

/**
 * A ClipAsset handle field.
 *
 * A `ClipAsset` field: stored in a `Uint32Array` with stride 2.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function ClipAsset(value?: ClipAsset, options?: FieldOptions): FieldSpec<"ClipAsset">;

/**
 * A GraphAsset handle field.
 *
 * A `GraphAsset` field: stored in a `Uint32Array` with stride 2.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function GraphAsset(value?: GraphAsset, options?: FieldOptions): FieldSpec<"GraphAsset">;

/**
 * What the root joint's motion does to the entity.
 * - `"off"`: The root animates in place; the entity does not move.
 * - `"planar"`: The root's horizontal (xz) travel and yaw move og.Transform.
 * - `"full"`: The root's whole translation and rotation move og.Transform.
 */
export type RootMotion = "off" | "planar" | "full";

/**
 * A RootMotion field: the variant's index.
 *
 * A `RootMotion` field: stored in a `Uint32Array` with stride 1.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function RootMotion(value?: RootMotion, options?: FieldOptions): FieldSpec<"RootMotion">;

/**
 * Who decides blendshape weights.
 * - `"component"`: Game code writes w0...w7.
 * - `"clips"`: The playing clips' weight channels write them.
 */
export type BlendshapeSource = "component" | "clips";

/**
 * A BlendshapeSource field: the variant's index.
 *
 * A `BlendshapeSource` field: stored in a `Uint32Array` with stride 1.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function BlendshapeSource(value?: BlendshapeSource, options?: FieldOptions): FieldSpec<"BlendshapeSource">;

/**
 * How an IK chain is solved.
 * - `"twoBone"`: Analytic two-bone (thigh, shin, foot), bending towards the pole.
 * - `"fabrik"`: FABRIK over `bones` bones, iteratively.
 */
export type IkSolver = "twoBone" | "fabrik";

/**
 * An IkSolver field: the variant's index.
 *
 * A `IkSolver` field: stored in a `Uint32Array` with stride 1.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function IkSolver(value?: IkSolver, options?: FieldOptions): FieldSpec<"IkSolver">;

/**
 * What an IK chain's target and pole are relative to.
 * - `"world"`: World positions: the skeleton entity's og.Transform applies.
 * - `"model"`: The skeleton's model space.
 */
export type IkSpace = "world" | "model";

/**
 * An IkSpace field: the variant's index.
 *
 * A `IkSpace` field: stored in a `Uint32Array` with stride 1.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function IkSpace(value?: IkSpace, options?: FieldOptions): FieldSpec<"IkSpace">;

/**
 * A posed skeleton: the entity is drawn skinned by the joint palette og.anim writes each tick. The pose itself (every node's local transform) is world state kept by the `og.anim` state block.
 *
 * The value of the `Skeleton` component (token `og.Skeleton`).
 */
export interface Skeleton {
  /** The skeleton asset (og.load.animation's `skeleton`); 0 is none. */
  asset: SkeletonAsset;
  /**
   * Which of the asset's skins the joint palette is for (glTF skin index).
   *
   * @range 0..
   */
  skin: number;
  /**
   * Root motion extraction: off (animate in place), planar (xz travel and yaw move og.Transform) or full.
   */
  rootMotion: RootMotion;
  /**
   * The joint whose motion is extracted (an index into the skin's joints; 0 is its first joint, usually the hips).
   *
   * @range 0..
   */
  rootJoint: number;
  /**
   * Written: the number of joints in the palette (0 until the first tick).
   *
   * @readonly written by the engine
   */
  readonly jointCount: number;
}

/**
 * Plays one clip on a skeleton. Alone it poses the skeleton from its rest pose; with an og.AnimationGraph it plays over the graph's result with `weight`. Changing `clip` cross-fades from the previous one over `fade`; a clip change starts the new clip at 0 (or at `time` if the game wrote it in the same tick).
 *
 * The value of the `AnimationPlayer` component (token `og.AnimationPlayer`).
 */
export interface AnimationPlayer {
  /** The clip asset; 0 plays nothing. */
  clip: ClipAsset;
  /**
   * The playback position; written every tick, set it to seek (no markers fire across a seek).
   *
   * @range 0..
   * @unit s (seconds of simulation time)
   */
  time: number;
  /**
   * Playback rate (1 = as authored, negative plays backwards).
   *
   * @default 1
   * @range -10..10
   * @tunable
   */
  speed: number;
  /**
   * Wrap at the end of the clip; otherwise hold the last frame and set `finished`.
   *
   * @default true
   */
  loop: boolean;
  /**
   * How much of the pose this player decides (0 = none, 1 = all).
   *
   * @default 1
   * @range 0..1
   * @tunable
   */
  weight: number;
  /**
   * Cross-fade duration when `clip` changes.
   *
   * @default 0.2
   * @range 0..10
   * @unit s (seconds of simulation time)
   * @tunable
   */
  fade: number;
  /**
   * Written: a non-looping clip reached its end.
   *
   * @readonly written by the engine
   */
  readonly finished: boolean;
}

/**
 * Runs a blend-tree graph (og.anim.graph) on a skeleton: its layers' state machines pick motions by the parameters p0...p7, which the graph names in `param` order (`param speed` is p0, the next `param` is p1, ...).
 *
 * The value of the `AnimationGraph` component (token `og.AnimationGraph`).
 */
export interface AnimationGraph {
  /** The graph asset; 0 is none. */
  asset: GraphAsset;
  /**
   * How much of the pose the graph decides over the rest pose.
   *
   * @default 1
   * @range 0..1
   * @tunable
   */
  weight: number;
  /**
   * A playback-rate multiplier for every layer.
   *
   * @default 1
   * @range 0..10
   * @tunable
   */
  speed: number;
  /**
   * Parameter 0 (the graph's first `param`).
   *
   * @tunable
   */
  p0: number;
  /**
   * Parameter 1.
   *
   * @tunable
   */
  p1: number;
  /**
   * Parameter 2.
   *
   * @tunable
   */
  p2: number;
  /**
   * Parameter 3.
   *
   * @tunable
   */
  p3: number;
  /**
   * Parameter 4.
   *
   * @tunable
   */
  p4: number;
  /**
   * Parameter 5.
   *
   * @tunable
   */
  p5: number;
  /**
   * Parameter 6.
   *
   * @tunable
   */
  p6: number;
  /**
   * Parameter 7.
   *
   * @tunable
   */
  p7: number;
  /**
   * Written: the base layer's current state (its index in the graph file).
   *
   * @readonly written by the engine
   */
  readonly state: number;
  /**
   * Written: seconds since the base layer entered `state`.
   *
   * @unit s (seconds of simulation time)
   * @readonly written by the engine
   */
  readonly stateTime: number;
}

/**
 * Morph-target weights of the skeleton entity's mesh, in the order of the glTF mesh's targets. With `source` clips the playing clips' weight channels write them; with `component` game code does.
 *
 * The value of the `Blendshapes` component (token `og.Blendshapes`).
 */
export interface Blendshapes {
  /**
   * Who decides the weights: component (game code writes w0...w7) or clips (the animation writes them).
   */
  source: BlendshapeSource;
  /**
   * The glTF mesh index whose targets the weights drive (0 is the first mesh).
   *
   * @range 0..
   */
  mesh: number;
  /**
   * Weight of target 0.
   *
   * @range -1..2
   * @tunable
   */
  w0: number;
  /**
   * Weight of target 1.
   *
   * @range -1..2
   * @tunable
   */
  w1: number;
  /**
   * Weight of target 2.
   *
   * @range -1..2
   * @tunable
   */
  w2: number;
  /**
   * Weight of target 3.
   *
   * @range -1..2
   * @tunable
   */
  w3: number;
  /**
   * Weight of target 4.
   *
   * @range -1..2
   * @tunable
   */
  w4: number;
  /**
   * Weight of target 5.
   *
   * @range -1..2
   * @tunable
   */
  w5: number;
  /**
   * Weight of target 6.
   *
   * @range -1..2
   * @tunable
   */
  w6: number;
  /**
   * Weight of target 7.
   *
   * @range -1..2
   * @tunable
   */
  w7: number;
}

/**
 * An inverse-kinematics constraint on a skeleton, applied after its clips and graph: the chain of `bones` bones ending at joint `tip` turns so the tip reaches `target`. Put it on its own entity (a skeleton can have many chains); chains apply in query order.
 *
 * The value of the `IkChain` component (token `og.IkChain`).
 */
export interface IkChain {
  /** The entity with the og.Skeleton this chain bends. */
  skeleton: Entity;
  /**
   * The end joint (an index into the skin's joints): the foot or the hand.
   *
   * @range 0..
   */
  tip: number;
  /**
   * How many bones above the tip turn (two-bone always uses 2).
   *
   * @default 2
   * @range 1..32
   */
  bones: number;
  /** twoBone (analytic, with a pole) or fabrik (iterative, any length). */
  solver: IkSolver;
  /**
   * Where the tip should be.
   *
   * @unit m (metres)
   */
  target: Vector3;
  /**
   * A point the middle joint bends towards (two-bone); the zero vector keeps the animated bend.
   *
   * @unit m (metres)
   */
  pole: Vector3;
  /**
   * What `target` and `pole` are relative to: world (the skeleton entity's og.Transform applies) or model.
   */
  space: IkSpace;
  /**
   * How much of the solved rotation is applied (0 = the animated pose, 1 = fully solved).
   *
   * @default 1
   * @range 0..1
   * @tunable
   */
  weight: number;
  /**
   * FABRIK iterations at most.
   *
   * @default 10
   * @range 1..64
   */
  iterations: number;
  /**
   * The tip distance (model units) at which the target counts as reached.
   *
   * @default 0.001
   * @range 0..
   * @unit m (metres)
   */
  tolerance: number;
  /**
   * Written: the tip is within `tolerance` of the target after solving.
   *
   * @readonly written by the engine
   */
  readonly reached: boolean;
  /**
   * Written: the tip's distance from the target after solving, in model units.
   *
   * @unit m (metres)
   * @readonly written by the engine
   */
  readonly error: number;
}

/**
 * A named marker of a clip that playback passed in the last tick (read in the tick after it fired, like every event). Cross-fades and blends fire every contributing clip's markers.
 *
 * The payload of the `AnimationEvent` event (token `og.AnimationEvent`).
 */
export interface AnimationEvent {
  /** The skeleton entity. */
  readonly entity: Entity;
  /** The clip asset the marker belongs to. */
  readonly clip: ClipAsset;
  /** The marker's name (at most 63 bytes). */
  readonly name: string;
  /**
   * The marker's time in the clip.
   *
   * @unit s (seconds of simulation time)
   */
  readonly time: number;
  /** The clip's blend weight when it fired. */
  readonly weight: number;
}

/** One clip of a loaded animation. */
export interface ClipInfo {
  /** Its handle, what AnimationPlayer.clip takes. */
  readonly clip: ClipAsset;
  /** The glTF animation's name. */
  readonly name: string;
  /**
   * Its length.
   *
   * @unit s (seconds of simulation time)
   */
  readonly duration: number;
  /** Its markers' names, in time order. */
  readonly markers: readonly string[];
}

/** What og.load.animation registered from a model's `#anim` blob. */
export interface Animation {
  /** The rig, what Skeleton.asset takes. */
  readonly skeleton: SkeletonAsset;
  /** Its clips, in glTF order. */
  readonly clips: readonly ClipInfo[];
  /** The rig's node names, in topological order. */
  readonly nodes: readonly string[];
  /** Each skin's joint count, in glTF order. */
  readonly skins: readonly number[];
  /** Morph targets across the model's meshes. */
  readonly morphTargets: number;
}

/**
 * A blend-tree graph to register.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface GraphDesc {
  /**
   * The graph's `.ogt` text (`type animgraph`, `version 1`; the grammar is og-anim's README).
   *
   * @default ""
   */
  readonly text?: string;
  /**
   * The clips a `clip <motion> "<name>"` line may name, by ClipInfo.name: usually og.load.animation's `clips`.
   *
   * @default []
   */
  readonly clips?: readonly ClipInfo[];
}


// ── audio ───────────────────────────────────────────────────────
// Sound: inside the tick, the engine's og.audio system (after the game's systems and physics) turns AudioSource changes into a deterministic, time-stamped command stream; outside the tick the host's mixer renders it (an AudioWorklet on the web, the output device, a capture or a null sink natively).
// The stream, not the PCM, is world state: snapshots, restore and the world hash cover it, and nothing a sink does reaches the world. Load sounds with og.load.sound; inserting a source plays it, setting `playing` false stops it, and a one-shot sets `playing` back to false when it ends.
// Spatial gain and pan are computed in the tick from the source's og.Transform relative to the first AudioListener (three.js convention: the listener looks down -Z, +X is right; with none, the origin looking down -Z).

/** SoundEnded.reason: a one-shot played to its end. */
export declare const SOUND_FINISHED: 0;

/** SoundEnded.reason: `playing` was set to false. */
export declare const SOUND_STOPPED: 1;

/** SoundEnded.reason: the AudioSource was removed from a living entity. */
export declare const SOUND_REMOVED: 2;

/** SoundEnded.reason: `sound` changed while it played. */
export declare const SOUND_REPLACED: 3;

/**
 * The bus a source plays on; every bus passes through master.
 * - `"master"`: The master bus: what every bus passes through.
 * - `"music"`: Music.
 * - `"sfx"`: Sound effects.
 * - `"ui"`: Interface sounds.
 */
export type AudioBus = "master" | "music" | "sfx" | "ui";

/**
 * An AudioBus field: the variant's index.
 *
 * A `AudioBus` field: stored in a `Uint32Array` with stride 1.
 * @param value The field's default; omitted, the column's zero value.
 * @param options Range, unit, label, docs and the tunable flag.
 */
export declare function AudioBus(value?: AudioBus, options?: FieldOptions): FieldSpec<"AudioBus">;

/**
 * A sound an entity plays. Level-triggered: while `playing` is true it plays (inserting a source plays it), setting it false stops it, and a one-shot that reaches its end sets it back to false and sends og.SoundEnded.
 * A one-shot of F frames at rate R and pitch p started in tick S ends in tick S + ceil(F * tickRate / (R * p)) - 1. A despawned entity's voice is silenced without an event.
 *
 * The value of the `AudioSource` component (token `og.AudioSource`).
 */
export interface AudioSource {
  /**
   * The sound, from og.load.sound; 0 plays nothing. A handle not yet loaded waits and starts on the first tick it is there.
   */
  sound: Sound;
  /**
   * The bus it plays on.
   *
   * @default "sfx"
   */
  bus: AudioBus;
  /**
   * Linear gain.
   *
   * @default 1
   * @range 0..4
   * @tunable
   */
  volume: number;
  /**
   * Playback rate: 2 is an octave up and half as long.
   *
   * @default 1
   * @range 0.0625..16
   * @tunable
   */
  pitch: number;
  /**
   * Stereo position of a non-spatial source, -1 left to 1 right.
   *
   * @range -1..1
   * @tunable
   */
  pan: number;
  /** Loop between loopStart and loopEnd, seamlessly. */
  loop: boolean;
  /**
   * Where a loop restarts.
   *
   * @range 0..
   * @unit s (seconds of simulation time)
   */
  loopStart: number;
  /**
   * Where a loop wraps; 0 is the sound's end.
   *
   * @range 0..
   * @unit s (seconds of simulation time)
   */
  loopEnd: number;
  /** Attenuate and pan by the entity's og.Transform relative to the listener. */
  spatial: boolean;
  /**
   * Full volume up to this distance, as Web Audio's PannerNode.refDistance.
   *
   * @default 1
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  refDistance: number;
  /**
   * No quieter past this distance, as Web Audio's PannerNode.maxDistance.
   *
   * @default 100
   * @range 0..
   * @unit m (metres)
   * @tunable
   */
  maxDistance: number;
  /**
   * How fast volume falls with distance, in Web Audio's inverse model: ref / (ref + rolloff * (clamp(d, ref, max) - ref)).
   *
   * @default 1
   * @range 0..
   * @tunable
   */
  rolloffFactor: number;
  /**
   * Plays while true; the engine sets it false when a one-shot ends.
   *
   * @default true
   */
  playing: boolean;
}

/**
 * Where spatial sources are heard from: the entity's og.Transform, looking down its -Z with +X to the right. The first listener in query order is used.
 *
 * The value of the `AudioListener` component (token `og.AudioListener`).
 */
export interface AudioListener {
  /**
   * A gain on every spatial source this listener hears.
   *
   * @default 1
   * @range 0..4
   * @tunable
   */
  volume: number;
}

/**
 * Bus volumes; master scales every bus. A change ramps over 256 frames in the mixer.
 *
 * The `Audio` resource (token `og.Audio`).
 */
export interface Audio {
  /**
   * Gain of the master bus, applied to every sound.
   *
   * @default 1
   * @range 0..2
   * @tunable
   */
  master: number;
  /**
   * Gain of the music bus.
   *
   * @default 1
   * @range 0..2
   * @tunable
   */
  music: number;
  /**
   * Gain of the sfx bus.
   *
   * @default 1
   * @range 0..2
   * @tunable
   */
  sfx: number;
  /**
   * Gain of the ui bus.
   *
   * @default 1
   * @range 0..2
   * @tunable
   */
  ui: number;
}

/**
 * A source started, sent in the tick it happens (systems read it the tick after).
 *
 * The payload of the `SoundStarted` event (token `og.SoundStarted`).
 */
export interface SoundStarted {
  /** The entity whose AudioSource started. */
  readonly entity: Entity;
  /** The sound. */
  readonly sound: Sound;
  /** The voice id in the command stream. */
  readonly voice: number;
}

/**
 * A sound ended, sent in the tick it happens (systems read it the tick after).
 *
 * The payload of the `SoundEnded` event (token `og.SoundEnded`).
 */
export interface SoundEnded {
  /** The entity whose AudioSource ended. */
  readonly entity: Entity;
  /** The sound. */
  readonly sound: Sound;
  /** The voice id in the command stream. */
  readonly voice: number;
  /** Why: SOUND_FINISHED (0), SOUND_STOPPED (1), SOUND_REMOVED (2) or SOUND_REPLACED (3). */
  readonly reason: number;
}

// ── ui ──────────────────────────────────────────────────────────
// The engine's UI: an immediate-mode toolkit. Systems describe the tree every tick with these calls; the engine lays it out Clay-style, themes it with the platform's design tokens, draws it over the frame and answers the probe (dump_ui, touch_move_to, move_to_text) from it.
// The engine opens the frame before the game's systems (from og.Input and the play surface) and closes it after them, so calls belong inside systems; outside one they do nothing and log a warning. Interaction is decided against the previous tick's rectangles, which are also what the probe resolves, so a scripted touch_move_to and a click agree; response edges (clicked, changed, submitted) last one tick.
// Lengths are CSS pixels of the play surface (origin top left, y down), the unit of og.Input and the probe. Ids: an element's id is unique among its siblings (`idi("Slot", 3)` is `Slot#3`); a path `Inventory/Slot#3` names it after its nearest ancestors. The UI state (hover, press, focus, caret, scroll) is world state: snapshots and the world hash carry it.

/**
 * Which way a container lays out its children.
 * - `"row"`: Left to right.
 * - `"column"`: Top to bottom.
 */
export type Direction = "row" | "column";

/**
 * Where children sit along the main axis.
 * - `"start"`: At the start.
 * - `"center"`: Centred.
 * - `"end"`: At the end.
 * - `"spaceBetween"`: The free space split evenly between children.
 */
export type Align = "start" | "center" | "end" | "spaceBetween";

/**
 * Where children sit across the main axis.
 * - `"start"`: At the start.
 * - `"center"`: Centred.
 * - `"end"`: At the end.
 * - `"stretch"`: Stretched to the container's inner size.
 */
export type Cross = "start" | "center" | "end" | "stretch";

/**
 * How the lines of a label align.
 * - `"start"`: To the start (left).
 * - `"center"`: Centred.
 * - `"end"`: To the end (right).
 */
export type TextAlign = "start" | "center" | "end";

/**
 * Which configured font text uses; display and mono fall back to body.
 * - `"body"`: The body font.
 * - `"display"`: The display font.
 * - `"mono"`: The monospaced font.
 */
export type FontRole = "body" | "display" | "mono";

/**
 * Button looks, as the design language's buttons.
 * - `"secondary"`: A neutral fill (the default).
 * - `"primary"`: The amber action: one per screen.
 * - `"quiet"`: Text until hovered.
 * - `"danger"`: A destructive action: danger ink on a neutral fill.
 */
export type ButtonVariant = "secondary" | "primary" | "quiet" | "danger";

/**
 * Control heights from the density (h-control-sm, -md, -lg).
 * - `"medium"`: The medium control height.
 * - `"small"`: The small control height.
 * - `"large"`: The large control height.
 */
export type ControlSize = "medium" | "small" | "large";

/**
 * The palette: games run on a stage, so dark is the default.
 * - `"dark"`: The dark palette.
 * - `"light"`: The light palette.
 */
export type Scheme = "dark" | "light";

/**
 * Control sizes: auto picks touch sizes on a coarse pointer or a play surface under 768 px, compact otherwise.
 * - `"auto"`: Touch sizes on a coarse pointer or a narrow surface, compact otherwise.
 * - `"compact"`: Compact controls.
 * - `"comfortable"`: Comfortable controls.
 * - `"touch"`: Touch-sized controls.
 */
export type DensityMode = "auto" | "compact" | "comfortable" | "touch";

/**
 * A semantic colour of the design tokens (tokens.json `color`, in the current scheme).
 * - `"accent"`: accent: the amber action colour.
 * - `"accentActive"`: accent-active.
 * - `"accentFg"`: accent-fg: ink on the accent.
 * - `"accentHover"`: accent-hover.
 * - `"accentInk"`: accent-ink: the accent as text.
 * - `"accentSoft"`: accent-soft: a translucent accent wash.
 * - `"bg"`: bg: the page.
 * - `"bgSubtle"`: bg-subtle.
 * - `"border"`: border.
 * - `"borderStrong"`: border-strong.
 * - `"danger"`: danger.
 * - `"fg"`: fg: the primary ink.
 * - `"fgInverse"`: fg-inverse.
 * - `"fgMuted"`: fg-muted.
 * - `"fgSubtle"`: fg-subtle: placeholders.
 * - `"fill"`: fill: neutral control fills.
 * - `"fillStrong"`: fill-strong.
 * - `"focus"`: focus: the focus ring.
 * - `"info"`: info.
 * - `"materialControl"`: material-control.
 * - `"materialControlHover"`: material-control-hover.
 * - `"materialEdge"`: material-edge.
 * - `"materialGlow"`: material-glow.
 * - `"materialRegular"`: material-regular.
 * - `"materialRimSoft"`: material-rim-soft.
 * - `"materialSheen"`: material-sheen.
 * - `"materialShine"`: material-shine.
 * - `"materialThick"`: material-thick.
 * - `"materialThin"`: material-thin.
 * - `"scrim"`: scrim.
 * - `"shadow"`: shadow.
 * - `"sheen"`: sheen.
 * - `"stage"`: stage: the game stage.
 * - `"stageBorder"`: stage-border.
 * - `"stageDanger"`: stage-danger.
 * - `"stageFg"`: stage-fg.
 * - `"stageMuted"`: stage-muted.
 * - `"stageRaised"`: stage-raised.
 * - `"stageSurface"`: stage-surface.
 * - `"success"`: success.
 * - `"surface"`: surface: cards and panels.
 * - `"surfaceHover"`: surface-hover.
 * - `"surfaceRaised"`: surface-raised: tooltips.
 * - `"thumb"`: thumb: slider and switch thumbs.
 * - `"warn"`: warn.
 */
export type ThemeColor = "accent" | "accentActive" | "accentFg" | "accentHover" | "accentInk" | "accentSoft" | "bg" | "bgSubtle" | "border" | "borderStrong" | "danger" | "fg" | "fgInverse" | "fgMuted" | "fgSubtle" | "fill" | "fillStrong" | "focus" | "info" | "materialControl" | "materialControlHover" | "materialEdge" | "materialGlow" | "materialRegular" | "materialRimSoft" | "materialSheen" | "materialShine" | "materialThick" | "materialThin" | "scrim" | "shadow" | "sheen" | "stage" | "stageBorder" | "stageDanger" | "stageFg" | "stageMuted" | "stageRaised" | "stageSurface" | "success" | "surface" | "surfaceHover" | "surfaceRaised" | "thumb" | "warn";

/**
 * A step of the type ramp (tokens.json `font.scale`).
 * - `"body"`: text-body, the default.
 * - `"xs"`: text-xs.
 * - `"meta"`: text-meta.
 * - `"caption"`: text-caption.
 * - `"sm"`: text-sm.
 * - `"reading"`: text-reading.
 * - `"base"`: text-base.
 * - `"lg"`: text-lg.
 * - `"xl"`: text-xl.
 * - `"xl2"`: text-2xl.
 * - `"xl3"`: text-3xl.
 * - `"xl4"`: text-4xl.
 * - `"xl5"`: text-5xl.
 * - `"xl6"`: text-6xl.
 * - `"xl7"`: text-7xl.
 * - `"xl8"`: text-8xl.
 * - `"displaySm"`: text-display-sm.
 * - `"displayMd"`: text-display-md.
 * - `"displayLg"`: text-display-lg.
 * - `"displayXl"`: text-display-xl.
 * - `"displayXl2"`: text-display-2xl.
 */
export type TypeStepName = "body" | "xs" | "meta" | "caption" | "sm" | "reading" | "base" | "lg" | "xl" | "xl2" | "xl3" | "xl4" | "xl5" | "xl6" | "xl7" | "xl8" | "displaySm" | "displayMd" | "displayLg" | "displayXl" | "displayXl2";

/** The bounds of a fit or grow size. */
export interface SizeRange {
  /**
   * The smallest size.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly min?: number;
  /**
   * The largest size; absent is unbounded.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly max?: number;
}

/** A fixed size. */
export interface FixedSize {
  /**
   * The size.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly px: number;
}

/** A size relative to the parent. */
export interface PercentSize {
  /**
   * The fraction of the parent's inner size.
   *
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly fraction: number;
}

/** A size along one axis. Absent (null) is fit: the content's size. */
export type Size =
  /** The content's size, within the range. */
  | ({ readonly kind: "fit" } & SizeRange)
  /** The content's size, then a share of the parent's free space, within the range. */
  | ({ readonly kind: "grow" } & SizeRange)
  /** Exactly this size. */
  | ({ readonly kind: "fixed" } & FixedSize)
  /** A fraction of the parent's inner size. */
  | ({ readonly kind: "percent" } & PercentSize);

/** Padding or margins on four sides. */
export interface Insets {
  /**
   * The left side.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly left: number;
  /**
   * The top side.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly top: number;
  /**
   * The right side.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly right: number;
  /**
   * The bottom side.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly bottom: number;
}

/**
 * A colour: a theme token, or an sRGB colour as authored (components 0 to 1), with an alpha. Neither is white.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface Paint {
  /**
   * A semantic colour of the theme.
   *
   * @default null
   */
  readonly token?: ThemeColor | null;
  /**
   * An sRGB colour as authored, when no token is given.
   *
   * @default null
   */
  readonly color?: Color | null;
  /**
   * The opacity.
   *
   * @default 1
   * @range 0..1
   */
  readonly alpha?: number;
}

/**
 * A container: its layout and its look.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ElementDesc {
  /**
   * Which way it lays out its children.
   *
   * @default "row"
   */
  readonly direction?: Direction;
  /**
   * Its width; null fits the content.
   *
   * @default null
   */
  readonly width?: Size | null;
  /**
   * Its height; null fits the content.
   *
   * @default null
   */
  readonly height?: Size | null;
  /**
   * Space inside its edges.
   *
   * @default { left: 0, top: 0, right: 0, bottom: 0 }
   */
  readonly padding?: Insets;
  /**
   * Space between children.
   *
   * @default 0
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly gap?: number;
  /**
   * Where children sit along the main axis.
   *
   * @default "start"
   */
  readonly align?: Align;
  /**
   * Where children sit across it.
   *
   * @default "start"
   */
  readonly cross?: Cross;
  /**
   * Break a row into lines at its inner width.
   *
   * @default false
   */
  readonly wrap?: boolean;
  /**
   * Scroll its content vertically and clip it.
   *
   * @default false
   */
  readonly scroll?: boolean;
  /**
   * Report a Response from close(), as a button does.
   *
   * @default false
   */
  readonly interactive?: boolean;
  /**
   * Its fill; null draws none.
   *
   * @default null
   */
  readonly background?: Paint | null;
  /**
   * Corner radius.
   *
   * @default 0
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly radius?: number;
}

/**
 * How a label draws its text.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface LabelDesc {
  /**
   * The type step; null is body.
   *
   * @default null
   */
  readonly step?: TypeStepName | null;
  /**
   * The ink; null is fg.
   *
   * @default null
   */
  readonly color?: Paint | null;
  /**
   * How the lines align.
   *
   * @default "start"
   */
  readonly align?: TextAlign;
  /**
   * Keep to one line (breaking only at `\n`).
   *
   * @default false
   */
  readonly nowrap?: boolean;
  /**
   * Its width; null fits the text.
   *
   * @default null
   */
  readonly width?: Size | null;
  /**
   * Which font.
   *
   * @default "body"
   */
  readonly font?: FontRole;
}

/**
 * A button's look.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ButtonDesc {
  /**
   * The look.
   *
   * @default "secondary"
   */
  readonly variant?: ButtonVariant;
  /**
   * The control height.
   *
   * @default "medium"
   */
  readonly size?: ControlSize;
  /**
   * Neutral and inert.
   *
   * @default false
   */
  readonly disabled?: boolean;
  /**
   * Its width; null fits the label.
   *
   * @default null
   */
  readonly width?: Size | null;
}

/**
 * A card: surface, the card radius, padded and spaced by the tokens.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface PanelDesc {
  /**
   * Lay children left to right (the default is top to bottom).
   *
   * @default false
   */
  readonly row?: boolean;
  /**
   * Its width; null fits the content.
   *
   * @default null
   */
  readonly width?: Size | null;
  /**
   * Its height; null fits the content.
   *
   * @default null
   */
  readonly height?: Size | null;
  /**
   * Space inside; null is space.lg on every side.
   *
   * @default null
   */
  readonly padding?: Insets | null;
  /**
   * Space between children; null is space.md.
   *
   * @default null
   */
  readonly gap?: number | null;
  /**
   * Where children sit along the main axis.
   *
   * @default "start"
   */
  readonly align?: Align;
  /**
   * Where children sit across it.
   *
   * @default "start"
   */
  readonly cross?: Cross;
  /**
   * Its fill; null is surface.
   *
   * @default null
   */
  readonly background?: Paint | null;
}

/**
 * A vertical scroll container: the wheel or a drag scrolls it, it clips its content, a capsule indicator shows the position.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ScrollDesc {
  /**
   * Its width; null fits the content.
   *
   * @default null
   */
  readonly width?: Size | null;
  /**
   * Its height; null fits the content.
   *
   * @default null
   */
  readonly height?: Size | null;
  /**
   * Space inside.
   *
   * @default { left: 0, top: 0, right: 0, bottom: 0 }
   */
  readonly padding?: Insets;
  /**
   * Space between children.
   *
   * @default 0
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly gap?: number;
}

/**
 * A texture region, or a solid rectangle without a texture.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ImageDesc {
  /**
   * The texture; null draws a solid rectangle of the tint.
   *
   * @default null
   */
  readonly texture?: Texture | null;
  /**
   * The region drawn, [u0, v0, u1, v1] with v down; null is the whole texture.
   *
   * @default null
   */
  readonly uv?: [number, number, number, number] | null;
  /**
   * Multiplies the texture; null is white.
   *
   * @default null
   */
  readonly tint?: Paint | null;
  /**
   * Its width; null fits.
   *
   * @default null
   */
  readonly width?: Size | null;
  /**
   * Its height; null fits.
   *
   * @default null
   */
  readonly height?: Size | null;
}

/**
 * A progress bar.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ProgressDesc {
  /**
   * Its width; null grows into the free space.
   *
   * @default null
   */
  readonly width?: Size | null;
}

/**
 * A slider's range and width.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface SliderDesc {
  /**
   * The smallest value; min equal to max (the zero value) means 0 to 1.
   *
   * @default 0
   */
  readonly min?: number;
  /**
   * The largest value.
   *
   * @default 0
   */
  readonly max?: number;
  /**
   * Snap to multiples of step from min; 0 is continuous.
   *
   * @default 0
   * @range 0..
   */
  readonly step?: number;
  /**
   * Its width; null grows.
   *
   * @default null
   */
  readonly width?: Size | null;
}

/**
 * A one-line text field.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface FieldDesc {
  /**
   * Shown, dimmed, while the field is empty.
   *
   * @default ""
   */
  readonly placeholder?: string;
  /**
   * Its width; null grows.
   *
   * @default null
   */
  readonly width?: Size | null;
  /**
   * The most characters it takes; 0 is no limit.
   *
   * @default 0
   */
  readonly maxChars?: number;
}

/**
 * What happened to an element this tick. Edges (clicked, changed, submitted) are true for exactly one tick.
 */
export interface Response {
  /** The pointer is over it and no other element holds the press. */
  readonly hovered: boolean;
  /** The press that started on it is still held. */
  readonly pressed: boolean;
  /** Pressed and released inside it. */
  readonly clicked: boolean;
  /** Its value changed (slider, toggle, text field). */
  readonly changed: boolean;
  /**
   * It has focus: a text field being edited, or the control keyboard and gamepad navigation landed on.
   */
  readonly focused: boolean;
  /** Enter was pressed in it (text field). */
  readonly submitted: boolean;
  /**
   * A list row or grid cell a press started on was released over this one: that row's or cell's index (drag and drop).
   */
  readonly dropped: number | null;
}

/** A slider's response and its value after this tick's input. */
export interface SliderResult {
  /** What happened to it. */
  readonly response: Response;
  /** The value, to keep in game state and pass back next tick. */
  readonly value: number;
}

/** A toggle's response and its state after this tick's input. */
export interface ToggleResult {
  /** What happened to it. */
  readonly response: Response;
  /** The state, to keep in game state and pass back next tick. */
  readonly on: boolean;
}

/** A text field's response and its text after this tick's input. */
export interface FieldResult {
  /** What happened to it. */
  readonly response: Response;
  /** The text, to keep in game state and pass back next tick. */
  readonly value: string;
}

/** The play surface the UI lays out in. */
export interface Viewport {
  /**
   * Its width.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly width: number;
  /**
   * Its height.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly height: number;
  /**
   * Device pixels per CSS pixel; rectangles snap to device pixels.
   *
   * @default 1
   */
  readonly dpr: number;
  /** The device's unsafe margins (notch, home indicator); the root pads by them. */
  readonly safeArea: Insets;
  /** A touch screen or another coarse pointer. */
  readonly coarsePointer: boolean;
  /** Its size class: phone portrait or landscape, tablet, desktop. */
  readonly breakpoint: Breakpoint;
}

/**
 * How the UI looks: the scheme, the density, its fonts, the contrast, the game's accent, motion and the icon atlas.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface UiDesc {
  /**
   * The palette.
   *
   * @default "dark"
   */
  readonly scheme?: Scheme;
  /**
   * The control sizes.
   *
   * @default "auto"
   */
  readonly density?: DensityMode;
  /**
   * The body font, from og.load.font; null measures text at half an em per character and draws none.
   *
   * @default null
   */
  readonly font?: Font | null;
  /**
   * The display font; null uses the body font.
   *
   * @default null
   */
  readonly displayFont?: Font | null;
  /**
   * The monospaced font; null uses the body font.
   *
   * @default null
   */
  readonly monoFont?: Font | null;
  /**
   * Standard or high contrast.
   *
   * @default "standard"
   */
  readonly contrast?: Contrast;
  /**
   * A token hue as the game's accent (its hover, active, ink and focus tones derived as the tokens derive amber's); null keeps amber.
   *
   * @default null
   */
  readonly accentHue?: HueName | null;
  /**
   * An sRGB colour as the game's accent, when no hue is given.
   *
   * @default null
   */
  readonly accentColor?: Color | null;
  /**
   * Transitions only fade; sheens stop (prefers-reduced-motion).
   *
   * @default false
   */
  readonly reducedMotion?: boolean;
  /**
   * The icon atlas og.ui.icon draws from.
   *
   * @default null
   */
  readonly icons?: IconAtlasDesc | null;
  /**
   * Where toasts stack; null is the bottom centre.
   *
   * @default null
   */
  readonly toastAnchor?: Anchor | null;
}

/** One step of the type ramp. */
export interface TypeStep {
  /**
   * The font size.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly size: number;
  /**
   * The leading of Latin text.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly lineHeight: number;
  /** Latin tracking, in em. */
  readonly trackingEm: number;
  /**
   * The leading text with Chinese, Japanese or Korean characters takes.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly cjkLineHeight: number;
}

/** The control, row and header heights and the gutter of the density in effect. */
export interface ControlSizes {
  /**
   * h-control-sm.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly controlSm: number;
  /**
   * h-control-md.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly controlMd: number;
  /**
   * h-control-lg.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly controlLg: number;
  /**
   * The gutter.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly gutter: number;
  /**
   * The header height.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly header: number;
  /**
   * The row height.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly row: number;
  /**
   * The section spacing.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly section: number;
}

/** The design tokens the UI draws with, read-only. */
export interface UiTheme {
  /** The palette in use. */
  readonly scheme: Scheme;
  /** Each ThemeColor as `#rrggbb` or `#rrggbbaa` (sRGB as authored). */
  readonly palette: Readonly<Record<string, string>>;
  /** The spacing scale by name (xxs, xs, sm, md, lg, xl, xxl, xl3), in CSS pixels. */
  readonly space: Readonly<Record<string, number>>;
  /**
   * The radius scale by name (control, field, block, card, panel, sheet, button), in CSS pixels.
   */
  readonly radius: Readonly<Record<string, number>>;
  /** Each TypeStepName's step. */
  readonly typeRamp: Readonly<Record<string, TypeStep>>;
  /** The control sizes of the density in effect. */
  readonly sizes: ControlSizes;
  /**
   * The motion durations by name (instant, fast, base, slow, spring, bounce), in milliseconds.
   */
  readonly durationsMs: Readonly<Record<string, number>>;
}

/** Builds a container's children: ui calls made inside it nest under the container. */
export type UiBuild = () => void;





















/**
 * A point of the safe area an anchored group sits at.
 * - `"topLeft"`: The top left corner.
 * - `"top"`: The middle of the top edge.
 * - `"topRight"`: The top right corner.
 * - `"left"`: The middle of the left edge.
 * - `"center"`: The centre.
 * - `"right"`: The middle of the right edge.
 * - `"bottomLeft"`: The bottom left corner.
 * - `"bottom"`: The middle of the bottom edge.
 * - `"bottomRight"`: The bottom right corner.
 */
export type Anchor = "topLeft" | "top" | "topRight" | "left" | "center" | "right" | "bottomLeft" | "bottom" | "bottomRight";

/**
 * How a transition moves its content in and out. Under reduced motion every kind only fades.
 * - `"fade"`: Opacity only.
 * - `"slideUp"`: Rises into place from below.
 * - `"slideDown"`: Drops into place from above.
 * - `"slideLeft"`: Enters from the right.
 * - `"slideRight"`: Enters from the left.
 * - `"scale"`: Grows from slightly smaller around its centre.
 */
export type TransitionKind = "fade" | "slideUp" | "slideDown" | "slideLeft" | "slideRight" | "scale";

/**
 * A status tone, as the design language's badges and toasts.
 * - `"neutral"`: Neutral: fill and muted ink.
 * - `"success"`: Success.
 * - `"warn"`: Warning (orange).
 * - `"danger"`: Danger.
 * - `"info"`: Information.
 * - `"accent"`: Amber identity: the soft accent and amber ink.
 * - `"inverse"`: Neutral emphasis: fg fill, inverse ink.
 */
export type Tone = "neutral" | "success" | "warn" | "danger" | "info" | "accent" | "inverse";

/**
 * What a meter measures, which sets its colour.
 * - `"health"`: Green, warning orange under 30 %, danger red under 15 %.
 * - `"mana"`: The blue hue.
 * - `"xp"`: The purple hue.
 * - `"accent"`: The accent.
 */
export type MeterKind = "health" | "mana" | "xp" | "accent";

/**
 * The play surface's size class.
 * - `"desktop"`: 1024 CSS px wide or more, the shorter side 600 or more.
 * - `"tablet"`: The shorter side 600 or more, under 1024 wide.
 * - `"phoneLandscape"`: The shorter side under 600, wider than tall.
 * - `"phonePortrait"`: The shorter side under 600, taller than wide.
 */
export type Breakpoint = "desktop" | "tablet" | "phoneLandscape" | "phonePortrait";

/**
 * The theme variant: high contrast remaps muted inks, fills and borders to stronger tokens and outlines every control (no new colour).
 * - `"standard"`: The token palette as it is.
 * - `"high"`: Stronger inks, fills and edges.
 */
export type Contrast = "standard" | "high";

/**
 * One of the twelve category hues of the tokens, for a game's accent.
 * - `"red"`: hue-red.
 * - `"orange"`: hue-orange.
 * - `"yellow"`: hue-yellow.
 * - `"green"`: hue-green.
 * - `"mint"`: hue-mint.
 * - `"teal"`: hue-teal.
 * - `"cyan"`: hue-cyan.
 * - `"blue"`: hue-blue.
 * - `"indigo"`: hue-indigo.
 * - `"purple"`: hue-purple.
 * - `"pink"`: hue-pink.
 * - `"brown"`: hue-brown.
 */
export type HueName = "red" | "orange" | "yellow" | "green" | "mint" | "teal" | "cyan" | "blue" | "indigo" | "purple" | "pink" | "brown";

/**
 * A focus-navigation intent, as the arrow keys, Tab, Enter, Space, Escape and a gamepad's d-pad, stick, A and B give.
 * - `"up"`: Up.
 * - `"down"`: Down.
 * - `"left"`: Left.
 * - `"right"`: Right.
 * - `"next"`: The next control in reading order (Tab).
 * - `"prev"`: The previous one (Shift+Tab).
 * - `"activate"`: Press the focused control (Enter, Space, A).
 * - `"back"`: Leave: the topmost dialog or menu closes, else og.ui.back() answers true (Escape, B).
 */
export type NavIntent = "up" | "down" | "left" | "right" | "next" | "prev" | "activate" | "back";

/** A rectangle of the play surface. */
export interface UiRect {
  /**
   * Its left edge.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly x: number;
  /**
   * Its top edge.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly y: number;
  /**
   * Its width.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly width: number;
  /**
   * Its height.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly height: number;
}

/**
 * The icon atlas: a texture of equal cells in a grid, named in reading order.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface IconAtlasDesc {
  /**
   * The atlas texture, from og.load.texture.
   *
   * @default null
   */
  readonly texture?: Texture | null;
  /**
   * Cells across.
   *
   * @default 1
   * @range 1..
   */
  readonly columns?: number;
  /**
   * Cells down.
   *
   * @default 1
   * @range 1..
   */
  readonly rows?: number;
  /**
   * Each cell's name, in reading order: og.ui.icon draws a cell by its name.
   *
   * @default []
   */
  readonly names?: readonly string[];
}

/**
 * An anchored group: a layer at a point of the safe area, over the tree (HUD corners, a centred title, a bar of touch controls).
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface AnchorDesc {
  /**
   * Where it sits.
   *
   * @default "topLeft"
   */
  readonly anchor?: Anchor;
  /**
   * The inset from the safe area's edges; null is the density's gutter.
   *
   * @default null
   */
  readonly margin?: number | null;
  /**
   * Moves it right after placing.
   *
   * @default 0
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly offsetX?: number;
  /**
   * Moves it down after placing.
   *
   * @default 0
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly offsetY?: number;
  /**
   * Which way it lays out its children.
   *
   * @default "row"
   */
  readonly direction?: Direction;
  /**
   * Space between children.
   *
   * @default 0
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly gap?: number;
  /**
   * Space inside its edges.
   *
   * @default { left: 0, top: 0, right: 0, bottom: 0 }
   */
  readonly padding?: Insets;
  /**
   * Where children sit along the main axis.
   *
   * @default "start"
   */
  readonly align?: Align;
  /**
   * Where children sit across it.
   *
   * @default "start"
   */
  readonly cross?: Cross;
  /**
   * Its fill; null draws none.
   *
   * @default null
   */
  readonly background?: Paint | null;
  /**
   * Corner radius.
   *
   * @default 0
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly radius?: number;
  /**
   * Its width; null fits the content.
   *
   * @default null
   */
  readonly width?: Size | null;
  /**
   * Its height; null fits the content.
   *
   * @default null
   */
  readonly height?: Size | null;
  /**
   * Draw over dialogs and spotlights (a tutorial callout, a dragged item); the default draws under them.
   *
   * @default false
   */
  readonly over?: boolean;
}

/**
 * How a transition moves, timed in ticks from the motion tokens.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface TransitionDesc {
  /**
   * The movement.
   *
   * @default "fade"
   */
  readonly kind?: TransitionKind;
  /**
   * How long entering and leaving take; 0 is motion.duration.base (240 ms).
   *
   * @default 0
   * @range 0..
   * @unit ms (milliseconds)
   */
  readonly durationMs?: number;
  /**
   * How far slides travel; 0 is space.xl.
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly distance?: number;
  /**
   * Shown at once the first time it is built (no entrance).
   *
   * @default false
   */
  readonly instant?: boolean;
}

/** Where a transition or dialog is this tick. */
export interface Presence {
  /** Build its content: shown, entering or leaving. */
  readonly visible: boolean;
  /**
   * 0 gone, 1 settled.
   *
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly progress: number;
  /** Moving in. */
  readonly entering: boolean;
  /** Moving out. */
  readonly leaving: boolean;
}

/**
 * A dialog: a scrim over everything and a centred panel. Focus moves in and stays while it is the topmost; Back and the backdrop dismiss it unless locked.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ModalDesc {
  /**
   * A title over the content; empty is none.
   *
   * @default ""
   */
  readonly title?: string;
  /**
   * The panel's width; null is 420 CSS px, at most the surface less its gutters.
   *
   * @default null
   */
  readonly width?: Size | null;
  /**
   * Back and the backdrop do not dismiss it (a choice must be made).
   *
   * @default false
   */
  readonly locked?: boolean;
  /**
   * A press on the backdrop does not dismiss it.
   *
   * @default false
   */
  readonly ignoreBackdrop?: boolean;
  /**
   * How it enters and leaves; null is scale.
   *
   * @default null
   */
  readonly transition?: TransitionKind | null;
  /**
   * The backdrop's fill; null is scrim.
   *
   * @default null
   */
  readonly backdrop?: Paint | null;
  /**
   * No panel: the content sits centred on the backdrop (full-screen menus, loading).
   *
   * @default false
   */
  readonly bare?: boolean;
}

/** A dialog's answer this tick. */
export interface ModalResult {
  /** Build its content (open, or leaving). */
  readonly visible: boolean;
  /** Back or a press on the backdrop asked to close it: set your open flag false. */
  readonly dismissed: boolean;
}

/**
 * A spotlight: the surface dimmed except a hole around a target.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface SpotlightDesc {
  /**
   * Room around the target; 0 is space.sm.
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly padding?: number;
}

/**
 * A virtualised list: only the rows in view (and one more each side) are built, fillers keep the scroll range of every row.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ListDesc {
  /**
   * How many rows there are.
   *
   * @default 0
   */
  readonly count?: number;
  /**
   * Every row's height; 0 is the density's row height.
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly rowHeight?: number;
  /**
   * Space between rows.
   *
   * @default 0
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly gap?: number;
  /**
   * Its width; null fits.
   *
   * @default null
   */
  readonly width?: Size | null;
  /**
   * Its height (it scrolls inside); null grows.
   *
   * @default null
   */
  readonly height?: Size | null;
  /**
   * Space inside.
   *
   * @default { left: 0, top: 0, right: 0, bottom: 0 }
   */
  readonly padding?: Insets;
  /**
   * The selected row (fill-strong), for og.ui.list.
   *
   * @default null
   */
  readonly selected?: number | null;
}

/** The rows (or cells) to build this tick: first up to, not including, last. */
export interface ListRange {
  /** The first index. */
  readonly first: number;
  /** One past the last index. */
  readonly last: number;
}

/** What happened in a list or grid this tick. */
export interface ListResult {
  /** The row or cell pressed (or activated) this tick. */
  readonly clicked: number | null;
  /** A drag and drop ended: the index it started on. */
  readonly dropFrom: number | null;
  /** And the index it was released over. */
  readonly dropTo: number | null;
}

/**
 * A virtualised grid of equal cells (inventories, level select), scrolling vertically.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface GridDesc {
  /**
   * How many cells there are.
   *
   * @default 0
   */
  readonly count?: number;
  /**
   * A cell's width; 0 is 64 (never under the 44 px touch target).
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly cellWidth?: number;
  /**
   * A cell's height; 0 is its width.
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly cellHeight?: number;
  /**
   * Cells across; 0 is as many as fit.
   *
   * @default 0
   */
  readonly columns?: number;
  /**
   * Space between cells; 0 is space.sm.
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly gap?: number;
  /**
   * Its width; null fits (exactly the columns when columns is set).
   *
   * @default null
   */
  readonly width?: Size | null;
  /**
   * Its height; null grows.
   *
   * @default null
   */
  readonly height?: Size | null;
  /**
   * Space inside.
   *
   * @default { left: 0, top: 0, right: 0, bottom: 0 }
   */
  readonly padding?: Insets;
  /**
   * The selected cell, for og.ui.grid.
   *
   * @default null
   */
  readonly selected?: number | null;
}

/**
 * A segmented control of tabs.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface TabsDesc {
  /**
   * Equal segments across the width.
   *
   * @default false
   */
  readonly fill?: boolean;
  /**
   * Its width; null fits (grows with fill).
   *
   * @default null
   */
  readonly width?: Size | null;
}

/** A choice control's response and the chosen index after this tick's input. */
export interface ChoiceResult {
  /** What happened to it; changed when the choice moved. */
  readonly response: Response;
  /** The chosen index, to keep in game state and pass back next tick. */
  readonly selected: number;
}

/**
 * A select: a trigger that opens a menu of options.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface SelectDesc {
  /**
   * Shown while nothing is chosen.
   *
   * @default ""
   */
  readonly placeholder?: string;
  /**
   * Its width; null grows from 160.
   *
   * @default null
   */
  readonly width?: Size | null;
  /**
   * Neutral and inert.
   *
   * @default false
   */
  readonly disabled?: boolean;
}

/**
 * A number stepper's range, step and digits.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface StepperDesc {
  /**
   * The smallest value; min equal to max (the zero value) is unbounded.
   *
   * @default 0
   */
  readonly min?: number;
  /**
   * The largest value.
   *
   * @default 0
   */
  readonly max?: number;
  /**
   * How much a press changes it; 0 is 1.
   *
   * @default 0
   * @range 0..
   */
  readonly step?: number;
  /**
   * Digits after the point.
   *
   * @default 0
   * @range ..6
   */
  readonly decimals?: number;
}

/** A stepper's response and value after this tick's input. */
export interface StepperResult {
  /** What happened to it. */
  readonly response: Response;
  /** The value, to keep in game state and pass back next tick. */
  readonly value: number;
}

/**
 * How a radio group lays out.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface RadioDesc {
  /**
   * Options in a row (the default is a column).
   *
   * @default false
   */
  readonly horizontal?: boolean;
}

/** A checkbox list's response and values after this tick's input. */
export interface CheckListResult {
  /** What happened to it; changed when any box flipped. */
  readonly response: Response;
  /** The values, to keep in game state and pass back next tick. */
  readonly values: readonly boolean[];
}

/**
 * A toast's tone and time.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ToastDesc {
  /**
   * Its tone: a dot of the tone's colour before the text.
   *
   * @default "neutral"
   */
  readonly tone?: Tone;
  /**
   * How long it stays; 0 is 3200 ms (6000 for danger).
   *
   * @default 0
   * @range 0..
   * @unit ms (milliseconds)
   */
  readonly durationMs?: number;
}

/**
 * A chip's state.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ChipDesc {
  /**
   * Selected: inverted to fg.
   *
   * @default false
   */
  readonly selected?: boolean;
  /**
   * Neutral and inert.
   *
   * @default false
   */
  readonly disabled?: boolean;
}

/**
 * A meter: a capsule track with a fill that animates its changes by the tick.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface MeterDesc {
  /**
   * What it measures, which sets its colour.
   *
   * @default "health"
   */
  readonly kind?: MeterKind;
  /**
   * A colour of your own instead of the kind's.
   *
   * @default null
   */
  readonly color?: Paint | null;
  /**
   * Text inside the bar (such as `75 / 100`); empty is none.
   *
   * @default ""
   */
  readonly label?: string;
  /**
   * Its width; null grows.
   *
   * @default null
   */
  readonly width?: Size | null;
  /**
   * Its height; 0 is space.md, or the caption's line with a label.
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly height?: number;
  /**
   * Notches dividing it into this many parts; 0 or 1 is none.
   *
   * @default 0
   */
  readonly segments?: number;
}

/**
 * A cooldown ring's size and centre.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface CooldownDesc {
  /**
   * Its diameter; 0 is 56 (never under 44).
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly size?: number;
  /**
   * Text in the middle (a key, seconds left); empty is none.
   *
   * @default ""
   */
  readonly label?: string;
  /**
   * An icon of the atlas in the middle, by name; empty is none.
   *
   * @default ""
   */
  readonly icon?: string;
}

/**
 * An icon's size and ink.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface IconDesc {
  /**
   * Its size; 0 is 24.
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly size?: number;
  /**
   * Its ink; null is fg.
   *
   * @default null
   */
  readonly tint?: Paint | null;
}

/**
 * A marker on a mini-map.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface MapMarker {
  /**
   * Across the map, 0 to 1.
   *
   * @default 0
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly x?: number;
  /**
   * Down the map, 0 to 1.
   *
   * @default 0
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly y?: number;
  /**
   * Its colour; neutral is the stage ink.
   *
   * @default "neutral"
   */
  readonly tone?: Tone;
  /**
   * The player: the accent, larger, ringed.
   *
   * @default false
   */
  readonly player?: boolean;
}

/**
 * A mini-map frame.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface MinimapDesc {
  /**
   * The map; null is the stage colour.
   *
   * @default null
   */
  readonly texture?: Texture | null;
  /**
   * The region of the map shown, [u0, v0, u1, v1]; null is the whole texture.
   *
   * @default null
   */
  readonly uv?: [number, number, number, number] | null;
  /**
   * Its side; 0 is 160.
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly size?: number;
  /**
   * Markers over the map.
   *
   * @default []
   */
  readonly markers?: readonly MapMarker[];
  /**
   * A zone name under it; empty is none.
   *
   * @default ""
   */
  readonly label?: string;
}

/**
 * A dialogue box: a portrait slot, the speaker and the line typed out by the tick.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface DialogueDesc {
  /**
   * Who speaks (amber ink).
   *
   * @default ""
   */
  readonly speaker?: string;
  /**
   * The line. A new text starts typing again.
   *
   * @default ""
   */
  readonly text?: string;
  /**
   * A portrait texture; null is the speaker's initial.
   *
   * @default null
   */
  readonly portrait?: Texture | null;
  /**
   * The portrait's region of the texture; null is the whole texture.
   *
   * @default null
   */
  readonly portraitUv?: [number, number, number, number] | null;
  /**
   * A portrait from the icon atlas by name, when no texture is given; empty is the speaker's initial.
   *
   * @default ""
   */
  readonly portraitIcon?: string;
  /**
   * Typing speed; 0 is 40.
   *
   * @default 0
   * @range 0..
   */
  readonly charsPerSecond?: number;
  /**
   * The portrait on the right.
   *
   * @default false
   */
  readonly portraitRight?: boolean;
}

/** A dialogue box's answer this tick. */
export interface DialogueResult {
  /** Characters shown. */
  readonly shown: number;
  /** The whole line is shown. */
  readonly done: boolean;
  /** Pressed (or activated) while done: show the next line. */
  readonly advanced: boolean;
}

/**
 * A virtual joystick.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface JoystickDesc {
  /**
   * The base's diameter; 0 is 128 (never under three touch targets).
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly size?: number;
  /**
   * The ignored radius; 0 is 0.15.
   *
   * @default 0
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly deadZone?: number;
}

/** A joystick's vector this tick. */
export interface JoystickResult {
  /** -1 left to 1 right; 0 inside the dead zone. */
  readonly x: number;
  /** -1 up to 1 down; 0 inside the dead zone. */
  readonly y: number;
  /** A press that started on it is held. */
  readonly active: boolean;
}

/**
 * A d-pad.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface DpadDesc {
  /**
   * The pad's size; 0 is 132 (each arm at least 44).
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly size?: number;
}

/** Which arms of a d-pad are held this tick. */
export interface DpadResult {
  /** Up is held. */
  readonly up: boolean;
  /** Down is held. */
  readonly down: boolean;
  /** Left is held. */
  readonly left: boolean;
  /** Right is held. */
  readonly right: boolean;
}

/**
 * A round touch action button (jump, fire).
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ActionButtonDesc {
  /**
   * Its diameter; 0 is 64 (never under 44).
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly size?: number;
  /**
   * The amber button (one per screen).
   *
   * @default false
   */
  readonly primary?: boolean;
  /**
   * An icon of the atlas by name, instead of the label; empty is the label.
   *
   * @default ""
   */
  readonly icon?: string;
}

/**
 * A skeleton: the neutral fill standing in for content that is loading.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface SkeletonDesc {
  /**
   * Its width; null grows.
   *
   * @default null
   */
  readonly width?: Size | null;
  /**
   * A line's height; 0 is the body size.
   *
   * @default 0
   * @range 0..
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly height?: number;
  /**
   * Stacked lines; 0 or 1 is one, the last of several is shorter.
   *
   * @default 0
   */
  readonly lines?: number;
}

/**
 * Builds one row or cell of a virtualised list or grid: ui calls made inside nest under it.
 *
 * @param index Its index.
 */
export type UiItemBuild = (index: number) => void;





















































/**
 * A graphics quality preset.
 * - `"medium"`: Medium.
 * - `"low"`: Low: the fewest effects.
 * - `"high"`: High: every effect.
 */
export type GraphicsQuality = "medium" | "low" | "high";

/**
 * A template's action, by its id (a menu item, `buy:<item>`, `share`).
 *
 * @param action The action's id.
 */
export type UiAction = (action: string) => void;

/**
 * The settings after a change: persist them (cloud save, local storage through the shell) and apply them.
 *
 * @param settings The new settings.
 */
export type SettingsChanged = (settings: GameSettings) => void;

/**
 * An item of a menu: an id the game acts on and the label a player reads.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface MenuItem {
  /**
   * The id answered when it is chosen; also its probe id.
   *
   * @default ""
   */
  readonly id?: string;
  /**
   * What it says.
   *
   * @default ""
   */
  readonly label?: string;
  /**
   * Shown but inert.
   *
   * @default false
   */
  readonly disabled?: boolean;
  /**
   * The amber action (one per screen).
   *
   * @default false
   */
  readonly primary?: boolean;
}

/** What a menu-like template answered this tick. */
export interface MenuResult {
  /** The id of the item chosen this tick. */
  readonly selected: string | null;
}

/**
 * A title screen: the title, a subtitle, a column of items (the first primary unless one says so) and a footer. Centred on desktop and tablet, bottom-weighted on a phone.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface MainMenuDesc {
  /**
   * The game's title, in display type.
   *
   * @default ""
   */
  readonly title?: string;
  /**
   * A line under it; empty is none.
   *
   * @default ""
   */
  readonly subtitle?: string;
  /**
   * The items, such as Play, Continue, Settings, Quit.
   *
   * @default []
   */
  readonly items?: readonly MenuItem[];
  /**
   * A line at the bottom (a version, credits); empty is none.
   *
   * @default ""
   */
  readonly footer?: string;
  /**
   * Called with the chosen item's id.
   *
   * @default null
   */
  readonly onSelect?: UiAction | null;
}

/**
 * A pause screen: a dialog over the frozen game with Resume first. Back resumes.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface PauseMenuDesc {
  /**
   * Whether the game is paused.
   *
   * @default false
   */
  readonly open?: boolean;
  /**
   * Its title.
   *
   * @default "Paused"
   */
  readonly title?: string;
  /**
   * The items; empty is Resume, Settings, Quit to menu (`resume`, `settings`, `quit`).
   *
   * @default []
   */
  readonly items?: readonly MenuItem[];
  /**
   * Called with the chosen item's id; Back answers `resume`.
   *
   * @default null
   */
  readonly onSelect?: UiAction | null;
}

/**
 * A control the settings screen lists: an action and the key bound to it.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface KeyBinding {
  /**
   * The action's id.
   *
   * @default ""
   */
  readonly action?: string;
  /**
   * What a player reads.
   *
   * @default ""
   */
  readonly label?: string;
  /**
   * The bound key, as shown (`Space`, `A`, `←`).
   *
   * @default ""
   */
  readonly key?: string;
}

/**
 * A language the settings screen offers, in its own name.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface LanguageOption {
  /**
   * Its code (`en`, `zh-CN`).
   *
   * @default ""
   */
  readonly code?: string;
  /**
   * Its name in itself (`English`, `简体中文`).
   *
   * @default ""
   */
  readonly label?: string;
}

/**
 * The settings most games share. The game keeps them (a resource, a save) and passes them in every tick.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface GameSettings {
  /**
   * The master volume.
   *
   * @default 1
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly masterVolume?: number;
  /**
   * The music bus.
   *
   * @default 0.8
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly musicVolume?: number;
  /**
   * The effects bus.
   *
   * @default 1
   * @range 0..1
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly sfxVolume?: number;
  /**
   * Everything silent.
   *
   * @default false
   */
  readonly muted?: boolean;
  /**
   * The graphics preset.
   *
   * @default "medium"
   */
  readonly quality?: GraphicsQuality;
  /**
   * Show the frame rate.
   *
   * @default false
   */
  readonly showFps?: boolean;
  /**
   * Keep state changes, drop movement (the UI's transitions only fade).
   *
   * @default false
   */
  readonly reducedMotion?: boolean;
  /**
   * The high-contrast theme.
   *
   * @default false
   */
  readonly highContrast?: boolean;
  /**
   * Vibrate on hits (where the device can).
   *
   * @default true
   */
  readonly vibration?: boolean;
  /**
   * Invert the vertical look axis.
   *
   * @default false
   */
  readonly invertY?: boolean;
  /**
   * Look sensitivity.
   *
   * @default 1
   * @range 0.1..3
   */
  readonly sensitivity?: number;
  /**
   * The language code.
   *
   * @default "en"
   */
  readonly language?: string;
  /**
   * The controls, as shown on the Controls tab.
   *
   * @default []
   */
  readonly bindings?: readonly KeyBinding[];
}

/**
 * A settings screen: Audio, Graphics, Controls and Language tabs (a select on a phone) over the game's settings, with persistence hooks.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface SettingsDesc {
  /**
   * Whether it is shown.
   *
   * @default false
   */
  readonly open?: boolean;
  /**
   * The settings from game state.
   *
   * @default {}
   */
  readonly settings?: GameSettings;
  /**
   * The languages offered; empty hides the Language tab.
   *
   * @default []
   */
  readonly languages?: readonly LanguageOption[];
  /**
   * Called on every change (apply it at once: volume, contrast).
   *
   * @default null
   */
  readonly onChange?: SettingsChanged | null;
  /**
   * Called when the screen closes after changes (persist here).
   *
   * @default null
   */
  readonly onSave?: SettingsChanged | null;
}

/** The settings screen's answer this tick. */
export interface SettingsResult {
  /** The settings after this tick's input: keep them. */
  readonly settings: GameSettings;
  /** Something changed this tick. */
  readonly changed: boolean;
  /** The player closed the screen (Back, Done): set open false. */
  readonly closed: boolean;
}

/**
 * An ability on a HUD: a cooldown ring with its key.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface Ability {
  /**
   * Its id (and probe id).
   *
   * @default ""
   */
  readonly id?: string;
  /**
   * The key or button shown in the middle.
   *
   * @default ""
   */
  readonly key?: string;
  /**
   * An icon of the atlas instead of the key; empty is the key.
   *
   * @default ""
   */
  readonly icon?: string;
  /**
   * How much of its cooldown is left: 1 just used, 0 ready.
   *
   * @default 0
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly remaining?: number;
}

/**
 * A platformer HUD: lives as hearts and a level name top left, coins and score top right, a timer, a pause button.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface PlatformerHudDesc {
  /**
   * Lives left.
   *
   * @default 0
   */
  readonly lives?: number;
  /**
   * Lives at most (hearts drawn).
   *
   * @default 3
   */
  readonly maxLives?: number;
  /**
   * Coins collected.
   *
   * @default 0
   */
  readonly coins?: number;
  /**
   * The score.
   *
   * @default 0
   */
  readonly score?: number;
  /**
   * The level's name; empty is none.
   *
   * @default ""
   */
  readonly level?: string;
  /**
   * Seconds left; null hides the timer.
   *
   * @default null
   */
  readonly timeLeft?: number | null;
  /**
   * A health bar instead of hearts, 0 to 1; null shows hearts.
   *
   * @default null
   */
  readonly health?: number | null;
  /**
   * Show touch controls (a d-pad and Jump) on phones and tablets.
   *
   * @default false
   */
  readonly touch?: boolean;
}

/** What a HUD answered this tick. */
export interface HudResult {
  /** The pause button was pressed (or Back). */
  readonly paused: boolean;
  /**
   * A HUD action pressed this tick (an ability's id, `jump`, a generator's `buy:<id>`, `tap`).
   */
  readonly action: string | null;
  /** The touch controls' horizontal input, -1 to 1. */
  readonly moveX: number;
  /** The touch controls' vertical input, -1 (up) to 1. */
  readonly moveY: number;
  /** The touch action buttons held this tick. */
  readonly held: readonly string[];
}

/**
 * A shooter HUD: health and armor bottom left, ammo bottom right, abilities as cooldown rings, kills top right, a crosshair, an optional mini-map.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ShooterHudDesc {
  /**
   * Health.
   *
   * @default 0
   */
  readonly health?: number;
  /**
   * Full health.
   *
   * @default 100
   */
  readonly maxHealth?: number;
  /**
   * Armor; 0 with maxArmor 0 hides it.
   *
   * @default 0
   */
  readonly armor?: number;
  /**
   * Full armor.
   *
   * @default 0
   */
  readonly maxArmor?: number;
  /**
   * Rounds in the magazine.
   *
   * @default 0
   */
  readonly ammo?: number;
  /**
   * Rounds a magazine holds.
   *
   * @default 0
   */
  readonly magazine?: number;
  /**
   * Rounds in reserve.
   *
   * @default 0
   */
  readonly reserve?: number;
  /**
   * The weapon's name.
   *
   * @default ""
   */
  readonly weapon?: string;
  /**
   * Kills.
   *
   * @default 0
   */
  readonly kills?: number;
  /**
   * Abilities with cooldowns.
   *
   * @default []
   */
  readonly abilities?: readonly Ability[];
  /**
   * Draw a crosshair at the centre.
   *
   * @default true
   */
  readonly crosshair?: boolean;
  /**
   * A mini-map top left; null is none.
   *
   * @default null
   */
  readonly map?: MinimapDesc | null;
  /**
   * Show touch controls (a stick and Fire) on phones and tablets.
   *
   * @default false
   */
  readonly touch?: boolean;
}

/**
 * A puzzle HUD: level and moves, the score towards a target with three star marks, an optional timer.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface PuzzleHudDesc {
  /**
   * The level's name.
   *
   * @default ""
   */
  readonly level?: string;
  /**
   * The score.
   *
   * @default 0
   */
  readonly score?: number;
  /**
   * The score that wins.
   *
   * @default 1000
   */
  readonly target?: number;
  /**
   * The scores for one, two and three stars; empty is a third, two thirds and all of the target.
   *
   * @default []
   */
  readonly stars?: readonly number[];
  /**
   * Moves left; null hides it.
   *
   * @default null
   */
  readonly moves?: number | null;
  /**
   * Seconds left; null hides it.
   *
   * @default null
   */
  readonly timeLeft?: number | null;
}

/**
 * A currency an idle game counts.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface Currency {
  /**
   * Its id.
   *
   * @default ""
   */
  readonly id?: string;
  /**
   * Its name.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * How much there is.
   *
   * @default 0
   */
  readonly amount?: number;
  /**
   * How much it earns per second.
   *
   * @default 0
   */
  readonly perSecond?: number;
  /**
   * An icon of the atlas; empty is none.
   *
   * @default ""
   */
  readonly icon?: string;
}

/**
 * A generator an idle game lists: level, cost, progress.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface Generator {
  /**
   * Its id: buying answers `buy:<id>`.
   *
   * @default ""
   */
  readonly id?: string;
  /**
   * Its name.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * Its level.
   *
   * @default 0
   */
  readonly level?: number;
  /**
   * The next level's price.
   *
   * @default 0
   */
  readonly cost?: number;
  /**
   * The player can pay for it.
   *
   * @default false
   */
  readonly affordable?: boolean;
  /**
   * Its cycle, 0 to 1.
   *
   * @default 0
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly progress?: number;
  /**
   * An icon of the atlas; empty is none.
   *
   * @default ""
   */
  readonly icon?: string;
}

/**
 * An idle or incremental HUD: currencies with their rates on top, a big tap target, the generators in a list.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface IdleHudDesc {
  /**
   * The currencies, the first the main one.
   *
   * @default []
   */
  readonly currencies?: readonly Currency[];
  /**
   * The generators.
   *
   * @default []
   */
  readonly generators?: readonly Generator[];
  /**
   * The tap target's label; empty hides it.
   *
   * @default "Tap"
   */
  readonly tapLabel?: string;
}

/**
 * A racing HUD: position and lap top left, the times top right, the speedometer and gear bottom right, the boost meter, an optional mini-map.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface RacingHudDesc {
  /**
   * The speed, in `unit`.
   *
   * @default 0
   */
  readonly speed?: number;
  /**
   * The dial's top.
   *
   * @default 300
   */
  readonly maxSpeed?: number;
  /**
   * The speed's unit.
   *
   * @default "km/h"
   */
  readonly unit?: string;
  /**
   * The gear (`3`, `R`, `N`).
   *
   * @default ""
   */
  readonly gear?: string;
  /**
   * The lap.
   *
   * @default 1
   */
  readonly lap?: number;
  /**
   * Laps in the race.
   *
   * @default 3
   */
  readonly laps?: number;
  /**
   * The player's place.
   *
   * @default 1
   */
  readonly position?: number;
  /**
   * Cars in the race.
   *
   * @default 8
   */
  readonly racers?: number;
  /**
   * The race time in seconds.
   *
   * @default 0
   */
  readonly time?: number;
  /**
   * The best lap in seconds; null hides it.
   *
   * @default null
   */
  readonly best?: number | null;
  /**
   * The boost meter.
   *
   * @default 0
   * @unit ratio (a fraction between 0 and 1)
   */
  readonly boost?: number;
  /**
   * A mini-map; null is none.
   *
   * @default null
   */
  readonly map?: MinimapDesc | null;
  /**
   * Show touch controls (steering arrows, Brake, Boost) on phones and tablets.
   *
   * @default false
   */
  readonly touch?: boolean;
}

/**
 * One line of a cutscene.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface DialogueLine {
  /**
   * Who speaks.
   *
   * @default ""
   */
  readonly speaker?: string;
  /**
   * What they say.
   *
   * @default ""
   */
  readonly text?: string;
  /**
   * An icon of the atlas as the portrait; empty is the speaker's initial.
   *
   * @default ""
   */
  readonly portrait?: string;
}

/**
 * A dialogue scene: letterbox bars, the dialogue box at the bottom, Skip at the top right.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface CutsceneDesc {
  /**
   * Whether it plays.
   *
   * @default false
   */
  readonly open?: boolean;
  /**
   * The lines.
   *
   * @default []
   */
  readonly lines?: readonly DialogueLine[];
  /**
   * The line shown, from game state.
   *
   * @default 0
   */
  readonly index?: number;
  /**
   * Cinematic bars.
   *
   * @default true
   */
  readonly letterbox?: boolean;
  /**
   * Show Skip.
   *
   * @default true
   */
  readonly skippable?: boolean;
}

/** A cutscene's answer this tick. */
export interface CutsceneResult {
  /** The line to show next tick: keep it. */
  readonly index: number;
  /** The last line was advanced past. */
  readonly finished: boolean;
  /** Skip was pressed (or Back). */
  readonly skipped: boolean;
}

/**
 * An item a shop sells.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ShopItem {
  /**
   * Its id: buying answers it.
   *
   * @default ""
   */
  readonly id?: string;
  /**
   * Its name.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * Its price in the shop's currency.
   *
   * @default 0
   */
  readonly price?: number;
  /**
   * A line about it.
   *
   * @default ""
   */
  readonly description?: string;
  /**
   * An icon of the atlas; empty is none.
   *
   * @default ""
   */
  readonly icon?: string;
  /**
   * Its category tab; empty is the first.
   *
   * @default ""
   */
  readonly category?: string;
  /**
   * Already owned (one-off items): shown, not for sale.
   *
   * @default false
   */
  readonly owned?: boolean;
}

/**
 * A shop: the balance, category tabs, a grid of items with prices, a detail panel with Buy (disabled when the balance is short).
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ShopDesc {
  /**
   * Its title.
   *
   * @default "Shop"
   */
  readonly title?: string;
  /**
   * The currency's name (`coins`).
   *
   * @default "coins"
   */
  readonly currency?: string;
  /**
   * The currency's icon; empty is none.
   *
   * @default ""
   */
  readonly currencyIcon?: string;
  /**
   * What the player has.
   *
   * @default 0
   */
  readonly balance?: number;
  /**
   * What is for sale.
   *
   * @default []
   */
  readonly items?: readonly ShopItem[];
  /**
   * Category tabs; empty is one list.
   *
   * @default []
   */
  readonly categories?: readonly string[];
  /**
   * Called with the bought item's id.
   *
   * @default null
   */
  readonly onBuy?: UiAction | null;
}

/** A shop's answer this tick. */
export interface ShopResult {
  /** The item bought this tick: take the price and grant it. */
  readonly bought: string | null;
  /** Close was pressed (or Back). */
  readonly closed: boolean;
}

/**
 * What an inventory slot holds.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface InventoryItem {
  /**
   * The slot index.
   *
   * @default 0
   */
  readonly slot?: number;
  /**
   * The item's id.
   *
   * @default ""
   */
  readonly id?: string;
  /**
   * Its name.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * How many (stacks); 0 or 1 hides the count.
   *
   * @default 0
   */
  readonly count?: number;
  /**
   * An icon of the atlas; empty is its initial.
   *
   * @default ""
   */
  readonly icon?: string;
  /**
   * A line about it.
   *
   * @default ""
   */
  readonly description?: string;
}

/**
 * An inventory: a grid of slots, items dragged between them, a detail panel with Use.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface InventoryDesc {
  /**
   * Its title.
   *
   * @default "Inventory"
   */
  readonly title?: string;
  /**
   * How many slots.
   *
   * @default 24
   */
  readonly slots?: number;
  /**
   * Slots across; 0 fits the width.
   *
   * @default 0
   */
  readonly columns?: number;
  /**
   * The items, by slot.
   *
   * @default []
   */
  readonly items?: readonly InventoryItem[];
  /**
   * The detail panel's action; empty hides it.
   *
   * @default "Use"
   */
  readonly useLabel?: string;
}

/** An item dragged from one slot to another. */
export interface SlotMove {
  /** The slot it left. */
  readonly from: number;
  /** The slot it was dropped on (swap when that slot holds an item). */
  readonly to: number;
}

/** An inventory's answer this tick. */
export interface InventoryResult {
  /** The selected slot. */
  readonly selected: number | null;
  /** A drag and drop this tick: move (or swap) the items. */
  readonly moved: SlotMove | null;
  /** The id of the item Use was pressed for. */
  readonly used: string | null;
  /** Close was pressed (or Back). */
  readonly closed: boolean;
}

/**
 * A level in a level select.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface LevelInfo {
  /**
   * Its name or number.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * Stars earned, 0 to 3.
   *
   * @default 0
   */
  readonly stars?: number;
  /**
   * Not yet reachable: a lock, inert.
   *
   * @default false
   */
  readonly locked?: boolean;
  /**
   * The best result as shown; empty is none.
   *
   * @default ""
   */
  readonly best?: string;
}

/**
 * A level select: a grid of level cards with stars and locks, the total stars.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface LevelSelectDesc {
  /**
   * Its title.
   *
   * @default "Levels"
   */
  readonly title?: string;
  /**
   * The levels.
   *
   * @default []
   */
  readonly levels?: readonly LevelInfo[];
  /**
   * Cards across; 0 fits the width.
   *
   * @default 0
   */
  readonly columns?: number;
}

/** A level select's answer this tick. */
export interface LevelSelectResult {
  /** The level picked this tick. */
  readonly chosen: number | null;
  /** Back was pressed. */
  readonly closed: boolean;
}

/**
 * A labelled value on a results screen.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface StatRow {
  /**
   * The label.
   *
   * @default ""
   */
  readonly label?: string;
  /**
   * The value, as shown.
   *
   * @default ""
   */
  readonly value?: string;
}

/**
 * A results screen: the title, stars that land one by one, the score counting up, the best (a badge for a new best), stats, actions and a share hook.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ResultsDesc {
  /**
   * Whether it is shown.
   *
   * @default false
   */
  readonly open?: boolean;
  /**
   * Its title.
   *
   * @default "Level complete"
   */
  readonly title?: string;
  /**
   * The score.
   *
   * @default 0
   */
  readonly score?: number;
  /**
   * The best score.
   *
   * @default 0
   */
  readonly best?: number;
  /**
   * This run set the best.
   *
   * @default false
   */
  readonly newBest?: boolean;
  /**
   * Stars earned.
   *
   * @default 0
   */
  readonly stars?: number;
  /**
   * Stars possible; 0 hides them.
   *
   * @default 3
   */
  readonly maxStars?: number;
  /**
   * Stats under the score.
   *
   * @default []
   */
  readonly stats?: readonly StatRow[];
  /**
   * The actions; empty is Retry, Next and Menu (`retry`, `next`, `menu`).
   *
   * @default []
   */
  readonly actions?: readonly MenuItem[];
  /**
   * Text a share would post; empty hides Share.
   *
   * @default ""
   */
  readonly shareText?: string;
  /**
   * Called with the share text when Share is pressed (the shell posts it).
   *
   * @default null
   */
  readonly onShare?: UiAction | null;
}

/**
 * A row of a leaderboard.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface LeaderboardEntry {
  /**
   * The rank.
   *
   * @default 0
   */
  readonly rank?: number;
  /**
   * The player's name.
   *
   * @default ""
   */
  readonly name?: string;
  /**
   * The score, as shown.
   *
   * @default ""
   */
  readonly score?: string;
  /**
   * This is the player: highlighted.
   *
   * @default false
   */
  readonly player?: boolean;
}

/**
 * A leaderboard: scope tabs, a virtualised list of ranks (medal colours for 1 to 3, the player highlighted), and honest loading, error and empty states.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface LeaderboardDesc {
  /**
   * Its title.
   *
   * @default "Leaderboard"
   */
  readonly title?: string;
  /**
   * Scope tabs (Today, This week, All time); empty is none.
   *
   * @default []
   */
  readonly scopes?: readonly string[];
  /**
   * The rows.
   *
   * @default []
   */
  readonly entries?: readonly LeaderboardEntry[];
  /**
   * The rows are loading: skeletons.
   *
   * @default false
   */
  readonly loading?: boolean;
  /**
   * Loading failed: this message and Retry.
   *
   * @default ""
   */
  readonly error?: string;
  /**
   * Shown when there are no rows.
   *
   * @default "No scores yet"
   */
  readonly empty?: string;
}

/** A leaderboard's answer this tick. */
export interface LeaderboardResult {
  /** The scope tab chosen: keep it and load its rows. */
  readonly scope: number;
  /** The scope changed this tick. */
  readonly changed: boolean;
  /** Retry was pressed. */
  readonly retry: boolean;
  /** Close was pressed (or Back). */
  readonly closed: boolean;
}

/**
 * A step of a tutorial: the element to point at, a title and a line.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface TutorialStep {
  /**
   * The element's id or path (as touch_move_to takes); empty points at nothing.
   *
   * @default ""
   */
  readonly target?: string;
  /**
   * The step's title.
   *
   * @default ""
   */
  readonly title?: string;
  /**
   * What to do.
   *
   * @default ""
   */
  readonly text?: string;
}

/**
 * An onboarding overlay: a spotlight on each step's target, a callout beside it with Next and Skip, and the step count.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface TutorialDesc {
  /**
   * Whether it is shown.
   *
   * @default false
   */
  readonly open?: boolean;
  /**
   * The steps.
   *
   * @default []
   */
  readonly steps?: readonly TutorialStep[];
  /**
   * The step shown, from game state.
   *
   * @default 0
   */
  readonly index?: number;
}

/** A tutorial's answer this tick. */
export interface TutorialResult {
  /** The step to show next tick: keep it. */
  readonly index: number;
  /** The last step's Done was pressed. */
  readonly finished: boolean;
  /** Skip was pressed (or Back). */
  readonly skipped: boolean;
}

/**
 * A loading screen: the title, a real progress bar (or a spinner while unknown) and tips that change by the tick.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface LoadingDesc {
  /**
   * What is loading.
   *
   * @default "Loading"
   */
  readonly title?: string;
  /**
   * How far, 0 to 1; null is unknown (a spinner, never a fake bar).
   *
   * @default null
   */
  readonly progress?: number | null;
  /**
   * Tips shown in turn.
   *
   * @default []
   */
  readonly tips?: readonly string[];
  /**
   * Seconds per tip.
   *
   * @default 4
   * @range 1..
   */
  readonly tipSeconds?: number;
}

/**
 * A confirm dialog: a title, a consequence sentence, Cancel and the action.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface ConfirmDesc {
  /**
   * Whether it is open.
   *
   * @default false
   */
  readonly open?: boolean;
  /**
   * The question.
   *
   * @default ""
   */
  readonly title?: string;
  /**
   * What will happen.
   *
   * @default ""
   */
  readonly message?: string;
  /**
   * The action's label.
   *
   * @default "Confirm"
   */
  readonly confirmLabel?: string;
  /**
   * Cancel's label.
   *
   * @default "Cancel"
   */
  readonly cancelLabel?: string;
  /**
   * The action destroys something: the danger button.
   *
   * @default false
   */
  readonly danger?: boolean;
}

/** A confirm dialog's answer this tick. */
export interface ConfirmResult {
  /** The action was chosen. */
  readonly confirmed: boolean;
  /** Cancel, Back or the backdrop. */
  readonly cancelled: boolean;
}

/**
 * An alert: a title, a message and one button.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface AlertDesc {
  /**
   * Whether it is open.
   *
   * @default false
   */
  readonly open?: boolean;
  /**
   * The title.
   *
   * @default ""
   */
  readonly title?: string;
  /**
   * The message.
   *
   * @default ""
   */
  readonly message?: string;
  /**
   * The button's label.
   *
   * @default "OK"
   */
  readonly okLabel?: string;
}



















// ── systems ─────────────────────────────────────────────────────
// Systems the engine ships as TypeScript: game-level code, listed explicitly in GameDesc.systems like the game's own, so the schedule (and dump_world) shows them and nothing runs that the game did not ask for.

// ── commands ────────────────────────────────────────────────────
// The command channel: actions scheduled on simulation time, from `og run --commands "t:action[:args];..."` or the probe's commands().
// Core actions ship with the engine; a module or a game registers more. Unknown actions warn and continue. Not in publish builds.

/**
 * The actions the engine ships. Arguments follow the action, separated by `:`.
 * - `"screenshot"`: `screenshot[:path][:full|:N]`: capture the next frame.
 * - `"touch_move_to"`: `touch_move_to:<ui id>`: move the pointer to a UI element.
 * - `"touch_move"`: `touch_move:x,y`: move the pointer to CSS pixel coordinates.
 * - `"touch_start"`: `touch_start`: press the pointer where it is.
 * - `"touch_end"`: `touch_end`: release the pointer.
 * - `"scroll"`: `scroll:dx,dy`: scroll at the pointer.
 * - `"touch"`: `touch:<id>:<down|move|up>:<x,y | ui id>`: a further touch (pointer 1 to 4) held while the pointer is busy elsewhere.
 * - `"gamepad"`: `gamepad:<a+b+...|->[:lx,ly[,rx,ry]][:lt,rt]`: set pad 0's whole state; `gamepad:off` disconnects it.
 * - `"key_down"`: `key_down:<Key>`: hold a key by its code.
 * - `"key_up"`: `key_up:<Key>`: release a key.
 * - `"type"`: `type:<text>`: type text, one key press per character.
 * - `"move_to_text"`: `move_to_text:[=]text[@N]`: move the pointer to the UI element showing the text (exactly with `=`, the Nth with `@N`).
 * - `"teleport"`: `teleport:<entity>:x,y,z`: set an entity's position.
 * - `"frame_entity"`: `frame_entity:<entity>`: point the camera at an entity.
 * - `"dump_world"`: `dump_world[:path]`: tables, systems and their matched counts.
 * - `"dump_entity"`: `dump_entity:<name>[:path]`: one entity's components and transform.
 * - `"dump_ui"`: `dump_ui[:path]`: the UI tree with bounds.
 * - `"set"`: `set:<entity>.<component>.<field>=<value>`: a Tune write through the schema.
 * - `"seed"`: `seed:<n>`: reseed the Rng resource.
 * - `"pause"`: `pause`: hold the clock; advance still steps.
 * - `"resume"`: `resume`: release the clock.
 * - `"advance"`: `advance:<ticks>`: take fixed steps.
 * - `"quit"`: `quit`: exit.
 * - `"help"`: `help`: list every action the channel knows, core and registered.
 */
export type CoreAction = "screenshot" | "touch_move_to" | "touch_move" | "touch_start" | "touch_end" | "scroll" | "touch" | "gamepad" | "key_down" | "key_up" | "type" | "move_to_text" | "teleport" | "frame_entity" | "dump_world" | "dump_entity" | "dump_ui" | "set" | "seed" | "pause" | "resume" | "advance" | "quit" | "help";

/**
 * A registered action. It runs at the start of the tick it was scheduled for.
 *
 * @param world The world.
 * @param args The `:`-separated arguments after the action name.
 */
export type CommandHandler = (world: World, args: readonly string[]) => void;

/**
 * How a registered action describes itself in `og run --help` and the wiki.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface CommandDesc {
  /**
   * The argument form, such as `<count>`.
   *
   * @default ""
   */
  readonly usage?: string;
  /**
   * One sentence on what it does.
   *
   * @default ""
   */
  readonly doc?: string;
}

/** One parsed step of a command script. */
export interface CommandStep {
  /**
   * When it runs, in simulation seconds from the start of the script.
   *
   * @unit s (seconds of simulation time)
   */
  readonly at: number;
  /** The action name. */
  readonly action: string;
  /** The arguments. */
  readonly args: readonly string[];
}

/** One action as help lists it. */
export interface CommandInfo {
  /** The action name. */
  readonly name: string;
  /** Its argument form, such as `screenshot[:path][:full|:N]`. */
  readonly usage: string;
  /** One sentence on what it does. */
  readonly doc: string;
  /** Shipped with the engine (true) or registered by a module or a game. */
  readonly core: boolean;
}

/** What scheduling a script did. */
export interface CommandsReport {
  /** Steps scheduled. */
  readonly scheduled: number;
  /** The tick of the last step. */
  readonly untilTick: number;
  /** Unknown actions and other skipped steps, each naming the fix. */
  readonly warnings: readonly string[];
}





// ── probe ───────────────────────────────────────────────────────
// The probe: RuntimeProbe protocol 5 (origin/packages/contracts/src/probe.ts) served natively, plus the engine's wire members.
// In a page it is published on globalThis.__runtime_probe__; on native, `og` serves it as JSON-RPC 2.0 over a loopback WebSocket at /probe.
// A refusal is false or null exactly as protocol 5; a JSON-RPC error is only for a malformed call or an unknown method.
// Probe entity ids are entity names; an unnamed entity is `#<index>.<generation>`.

/** The global the page publishes the probe on. */
export declare const PROBE_GLOBAL: "__runtime_probe__";

/** The RuntimeProbe contract revision both sides speak. */
export declare const PROBE_VERSION: 5;

/** The wire revision hello reports. */
export declare const PROBE_WIRE_VERSION: 1;

/** The page query parameter that starts a deterministic replay from a uint32 seed. */
export declare const REPLAY_SEED_PARAM: "replay-seed";

/** The longest playtest one request may advance, in fixed steps. */
export declare const PLAYTEST_MAX_TICKS: 3600;

/** The most operations one probe program may carry. */
export declare const PROBE_PROGRAM_MAX_OPERATIONS: 256;

/** What a screen point resolved to. */
export interface PickResult {
  /** The probe id of the entity under the point. */
  readonly entity: string;
  /**
   * Where the pick landed in world space: two components in the 2D profile, three in 3D; nothing pads a plane with an unmeasured zero.
   */
  readonly world: [number, number] | [number, number, number];
  /** Where a person would change it, as `player.ts::jump`. */
  readonly source: string | null;
}

/** One published tunable parameter and the range it means something over. */
export interface ProbeTunable {
  /** Where the value lives, as `Health.hp`. */
  readonly path: string;
  /** The person-facing name. */
  readonly label?: string;
  /** The smallest meaningful value. */
  readonly min?: number;
  /** The largest meaningful value. */
  readonly max?: number;
  /** Tune's increment. */
  readonly step?: number;
}

/** One entity the probe publishes in state(). */
export interface ProbeEntityRef {
  /** Its probe id. */
  readonly id: string;
  /** A person-facing name. */
  readonly label?: string;
}

/** The fixed timings head of state(). */
export interface ProbeTimings {
  /**
   * Time the last tick took.
   *
   * @unit ms (milliseconds)
   */
  readonly step: number;
  /**
   * Time the last frame's render took; 0 without a renderer.
   *
   * @unit ms (milliseconds)
   */
  readonly render: number;
  /**
   * step + render.
   *
   * @unit ms (milliseconds)
   */
  readonly frameMs: number;
}

/** The probe's state(): a fixed head, then the game's own facts. */
export interface ProbeState {
  /** Fixed steps since load. */
  readonly tick: number;
  /** Whether the clock is held. */
  readonly paused: boolean;
  /** The seed the world was assembled from. */
  readonly seed: number;
  /** The last tick's timings. */
  readonly timings: ProbeTimings;
  /** The named entities. */
  readonly entities: readonly ProbeEntityRef[];
  /** What GameDesc.facts returned, or null. */
  readonly facts: unknown;
}

/** The advance operation's payload. */
export interface AdvanceOperation {
  /** Fixed steps to take. */
  readonly steps: number;
}

/** The input operation's payload. */
export interface InputOperation {
  /** The event. */
  readonly event: InjectedInput;
}

/** The pick operation's payload. */
export interface PickOperation {
  /** Horizontal position in CSS pixels. */
  readonly x: number;
  /** Vertical position in CSS pixels. */
  readonly y: number;
}

/** The entity operation's payload. */
export interface EntityOperation {
  /** The probe id. */
  readonly id: string;
}

/**
 * One operation of a probe program, which runs in one evaluation so a scripted segment costs one round trip.
 */
export type ProbeOperation =
  /** Take fixed steps; answers the tick. */
  | ({ readonly op: "advance" } & AdvanceOperation)
  /** Inject an event; answers true. */
  | ({ readonly op: "input" } & InputOperation)
  /** Hold the clock; answers true. */
  | { readonly op: "pause" }
  /** Release the clock; answers true. */
  | { readonly op: "resume" }
  /** Read the state; answers `{ tick, state }`. */
  | { readonly op: "read" }
  /** Pick a point; answers its result or null. */
  | ({ readonly op: "pick" } & PickOperation)
  /** Read an entity's parameters; null is a refusal. */
  | ({ readonly op: "entity" } & EntityOperation);

/**
 * What a program observed: one answer per operation that ran. A false, or entity's null, stops the program and `refused` names its index.
 */
export interface ProbeProgramResult {
  /** The answers, in order. */
  readonly results: readonly unknown[];
  /** The index of the refusing operation, or null. */
  readonly refused: number | null;
  /** The tick after the last operation that ran. */
  readonly tick: number;
}

/** The wire's hello: who is speaking. */
export interface Hello {
  /** PROBE_VERSION. */
  readonly protocol: number;
  /** PROBE_WIRE_VERSION. */
  readonly wire: number;
  /** Always `origin-engine`. */
  readonly substrate: string;
  /** `origin-engine <version>`. */
  readonly engine: string;
  /**
   * The fixed tick rate.
   *
   * @unit hz (ticks or cycles per second)
   */
  readonly ticksPerSecond: number;
  /** The renderer backend. */
  readonly tier: RenderTier;
  /** The build configuration. */
  readonly build: BuildProfile;
  /** The probe members this build serves. */
  readonly members: readonly string[];
  /**
   * The engine modules this build carries beyond the core (`physics`, `audio`, `ui`, `cook`), so a client knows which members do something.
   */
  readonly modules: readonly string[];
}

/**
 * The build configuration; publish builds keep only the player-facing probe members.
 * - `"dev"`: A development build: every member, the command channel, dumps.
 * - `"release"`: An optimised build with every member.
 * - `"publish"`: A published game: hello, tick, state, pick, entity, input and capture only.
 */
export type BuildProfile = "dev" | "release" | "publish";

/** A captured frame. */
export interface Capture {
  /** The tick the frame shows. */
  readonly tick: number;
  /** Width in pixels. */
  readonly width: number;
  /** Height in pixels. */
  readonly height: number;
  /** The PNG, base64. */
  readonly png: string;
}

/**
 * A parameter sweep: for each value, restore the snapshot taken at the call, patch, advance and capture.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface SweepDesc {
  /**
   * The probe id of the entity.
   *
   * @default ""
   */
  readonly id?: string;
  /**
   * The parameter, as `Health.hp`.
   *
   * @default ""
   */
  readonly parameter?: string;
  /**
   * The values to try, in order.
   *
   * @default []
   */
  readonly values?: readonly unknown[];
  /**
   * Fixed steps to advance after each patch.
   *
   * @default 0
   */
  readonly steps?: number;
  /**
   * Capture width; 0 is the render target's.
   *
   * @default 0
   */
  readonly width?: number;
  /**
   * Capture height; 0 is the render target's.
   *
   * @default 0
   */
  readonly height?: number;
}

/** One frame of a sweep. */
export interface SweepFrame {
  /** The value patched in. */
  readonly value: unknown;
  /** The tick captured. */
  readonly tick: number;
  /** The PNG, base64. */
  readonly png: string;
}

/** A sweep's frames, one per value. */
export interface SweepResult {
  /** The frames. */
  readonly frames: readonly SweepFrame[];
}

/**
 * What a hot reload did. Components are re-registered by name and migrated field by field through the schema.
 */
export interface ReloadReport {
  /** The tick the world was restored at. */
  readonly tick: number;
  /**
   * From bundle swap to restored world.
   *
   * @unit ms (milliseconds)
   */
  readonly ms: number;
  /** Fields carried across a layout change, as `Health.hp`. */
  readonly migrated: readonly string[];
  /** Components or fields that no longer exist, dropped with their data. */
  readonly dropped: readonly string[];
  /** Anything else worth reading, each naming the fix. */
  readonly warnings: readonly string[];
}

/**
 * A snapshot of the whole world (components, resources, Rng states, pending events) kept in the running process.
 */
export interface SnapshotInfo {
  /** The snapshot's id, for restore. */
  readonly id: string;
  /** The tick it was taken at. */
  readonly tick: number;
  /** The world hash at that tick, hex: equal hashes mean equal worlds. */
  readonly hash: string;
  /**
   * Its size.
   *
   * @unit bytes (bytes)
   */
  readonly bytes: number;
}

/**
 * What dump() reports.
 * - `"world"`: Archetype tables, systems and their matched counts.
 * - `"entity"`: One entity's components and transform.
 * - `"ui"`: The UI tree with bounds.
 */
export type DumpKind = "world" | "entity" | "ui";

/**
 * What to dump.
 *
 * A desc: every field may be omitted and then takes its documented default; `{}` is valid.
 */
export interface DumpDesc {
  /**
   * The kind of dump.
   *
   * @default "world"
   */
  readonly what?: DumpKind;
  /**
   * The probe id, for an entity dump.
   *
   * @default ""
   */
  readonly id?: string;
}

/** One archetype table. */
export interface ArchetypeDump {
  /** Its components, by name. */
  readonly components: readonly string[];
  /** Entities in it. */
  readonly count: number;
}

/**
 * One system. The engine fills reads, writes and matched for every system the game declared; ms is null because the probe reads no clock (state().timings has the tick's time).
 */
export interface SystemDump {
  /** Its name. */
  readonly name: string;
  /**
   * Entities its queries match now; null for a system the host did not declare to the probe.
   */
  readonly matched: number | null;
  /** What it declared it reads; null when undeclared. */
  readonly reads: readonly string[] | null;
  /** What it declared it writes; null when undeclared. */
  readonly writes: readonly string[] | null;
  /**
   * Time it took on the last tick; null when not measured.
   *
   * @unit ms (milliseconds)
   */
  readonly ms: number | null;
}

/** The world: tables, systems, resources. */
export interface WorldDump {
  /** The tick. */
  readonly tick: number;
  /** The world hash, hex. */
  readonly hash: string;
  /** Live entities. */
  readonly entities: number;
  /** The archetype tables. */
  readonly archetypes: readonly ArchetypeDump[];
  /** The systems in run order. */
  readonly systems: readonly SystemDump[];
  /** The registered resources. */
  readonly resources: readonly string[];
}

/** One entity. */
export interface EntityDump {
  /** Its probe id. */
  readonly id: string;
  /** Its name. */
  readonly name: string | null;
  /** Each component's field values, by component name. */
  readonly components: Readonly<Record<string, unknown>>;
}

/** One UI element with its bounds. */
export interface UiElement {
  /** Its stable id, as the command channel targets it. */
  readonly id: string;
  /** Its parent's id; empty at the root. */
  readonly parent: string;
  /** The text it shows, if any. */
  readonly text: string;
  /**
   * Left edge.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly x: number;
  /**
   * Top edge.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly y: number;
  /**
   * Width.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly width: number;
  /**
   * Height.
   *
   * @unit cssPx (CSS pixels of the play surface, origin at the top left)
   */
  readonly height: number;
}

/** The UI tree, flattened in draw order. */
export interface UiDump {
  /** The tick. */
  readonly tick: number;
  /** The elements. */
  readonly elements: readonly UiElement[];
}

/** A dump, tagged by its kind. */
export type DumpResult =
  /** The world dump. */
  | ({ readonly kind: "world" } & WorldDump)
  /** An entity dump. */
  | ({ readonly kind: "entity" } & EntityDump)
  /** The UI dump. */
  | ({ readonly kind: "ui" } & UiDump);

/**
 * RuntimeProbe protocol 5, member for member. Every member answers synchronously with JSON-serialisable values.
 */
export interface RuntimeProbe {
  /** PROBE_VERSION. */
  readonly version: number;
  /** Fixed steps the world has taken since load. */
  tick(): number;
  /** A JSON snapshot: the fixed timings head, then the game's own facts. */
  state(): ProbeState;
  /**
   * Take whole fixed steps regardless of the wall clock.
   *
   * @param steps Fixed steps, at most PLAYTEST_MAX_TICKS.
   * @returns The tick after the steps.
   */
  advance(steps: number): number;
  /** Hold the wall-clock loop; advance still steps. */
  pause(): boolean;
  /** Release the wall-clock loop. */
  resume(): boolean;
  /**
   * One event through the input layer's own path.
   *
   * @param event The event.
   * @returns False is a refusal.
   */
  input(event: InjectedInput): boolean;
  /** Page events (keyboard, pointer, wheel) the input layer accepted since load. */
  accepted(): number;
  /**
   * What is under a point of the play surface.
   *
   * @param x Horizontal position in CSS pixels.
   * @param y Vertical position in CSS pixels.
   * @returns Null when nothing is under the point.
   */
  pick(x: number, y: number): PickResult | null;
  /**
   * One entity's published parameters: its tunable fields as `Component.field` keys.
   *
   * @param id The probe id.
   * @returns Null for an unknown entity.
   */
  entity(id: string): Readonly<Record<string, unknown>> | null;
  /**
   * The ranges of one entity's tunable fields, from the schema.
   *
   * @param id The probe id.
   * @returns Null for an unknown entity.
   */
  bounds(id: string): Readonly<Record<string, ProbeTunable>> | null;
  /**
   * Move one tunable field for this run only; it applies within one frame. The probe never writes project files.
   *
   * @param id The probe id.
   * @param parameter The field, as `Health.hp`.
   * @param value The new value.
   * @returns False is a refusal.
   */
  patch(id: string, parameter: string, value: unknown): boolean;
  /**
   * Run a program of the members above in order, stopping at the first refusal.
   *
   * @param program At most PROBE_PROGRAM_MAX_OPERATIONS operations.
   */
  run(program: readonly ProbeOperation[]): ProbeProgramResult;
}

/**
 * The engine's probe: protocol 5 plus the wire members of the loopback WebSocket and the in-page request ring.
 */
export interface EngineProbe extends RuntimeProbe {
  /** Who is speaking: protocol, wire, engine version, tick rate, renderer tier, build. */
  hello(): Hello;
  /**
   * The frame of the current tick, rendered offscreen and read back as a PNG.
   * Size rule: width and height are both 0 (or absent), which is the render target's size, or both 1 to 4096, which renders that size with the same camera (the aspect follows the requested size); one zero and one not is a malformed call.
   * On the web the readback is asynchronous: through the request ring the answer arrives when the copy completes; a synchronous in-page call answers null on the first tick and the frame from then on (a capture is requested on tick N and answered on N + 1).
   *
   * @param width Width in pixels; 0 is the render target's. Default `0`.
   * @param height Height in pixels; 0 is the render target's. Default `0`.
   * @returns Null without a renderer.
   */
  capture(width?: number, height?: number): Capture | null;
  /**
   * A filmstrip over one tunable: per value, restore, patch, advance, capture.
   *
   * @param desc The parameter, values, steps and size.
   * @returns Null for an unknown entity or parameter, or without a renderer.
   */
  sweep(desc?: SweepDesc): SweepResult | null;
  /**
   * Hot reload: snapshot, swap the game bundle, re-register components by name, restore.
   *
   * @returns Null when the new bundle fails to load; the old one keeps running.
   */
  reload(): ReloadReport | null;
  /** Snapshot the whole world in the running process. */
  snapshot(): SnapshotInfo | null;
  /**
   * Restore a snapshot taken by this process.
   *
   * @param id The snapshot id.
   * @returns False for an unknown id.
   */
  restore(id: string): boolean;
  /**
   * A world, entity or UI dump, as the command channel's dump actions write.
   *
   * @param desc What to dump.
   * @returns Null for an unknown entity.
   */
  dump(desc?: DumpDesc): DumpResult | null;
  /**
   * Schedule a command script on simulation time, as `og run --commands`.
   *
   * @param script `t:action[:args];...`.
   * @returns Null for a malformed script.
   */
  commands(script: string): CommandsReport | null;
}

// ── og ──────────────────────────────────────────────────────────

/**
 * The engine's root object: root functions (`og.game`), every engine
 * component, resource and event token (`og.Transform`, `og.Time`), and one
 * namespace per module (`og.load`, `og.commands`).
 */
export declare namespace og {
  /**
   * Declare the game. The bundle's default export is its result.
   *
   * @param desc Tick, scenes, systems and commands.
   */
  function game(desc?: GameDesc): Game;
  /** Where an entity is, relative to its parent (or the world). */
  const Transform: ComponentType<{ readonly position: FieldSpec<"Vector3">; readonly quaternion: FieldSpec<"Quaternion">; readonly scale: FieldSpec<"Vector3"> }, Transform>;
  /** The entity this one's Transform is relative to. Set by SpawnDesc.parent. */
  const Parent: ComponentType<{ readonly entity: FieldSpec<"Entity"> }, Parent>;
  /**
   * The world matrix the engine composes after the systems run, as three.js Object3D.matrixWorld.
   */
  const MatrixWorld: ComponentType<{ readonly matrix: FieldSpec<"Matrix4"> }, MatrixWorld>;
  /**
   * Transforms in the three.js vocabulary: position, quaternion and scale, composed into matrixWorld down the parent chain.
   */
  namespace transform {
    /**
     * A matrix from position, rotation and scale, as three.js Matrix4.compose.
     *
     * @param position The translation.
     * @param quaternion The rotation.
     * @param scale The scale.
     */
    function compose(position: Vector3, quaternion: Quaternion, scale: Vector3): Matrix4;
    /**
     * Position, rotation and scale from a matrix, as three.js Matrix4.decompose.
     *
     * @param matrix An affine matrix.
     */
    function decompose(matrix: Matrix4): Transform;
    /**
     * The rotation that points -Z from an eye towards a target, as three.js Object3D.lookAt for cameras.
     *
     * @param eye Where the viewer is.
     * @param target What it looks at.
     * @param up The up direction; the zero vector means +Y.
     */
    function lookAt(eye: Vector3, target: Vector3, up?: Vector3): Quaternion;
    /**
     * A rotation about an axis, as three.js Quaternion.setFromAxisAngle.
     *
     * @param axis A unit axis.
     * @param angle The angle in radians.
     */
    function fromAxisAngle(axis: Vector3, angle: number): Quaternion;
  }
  /**
   * The fixed tick's clock. Every value is a function of the tick count, so a replay sees the same numbers.
   */
  const Time: ResourceType<{ readonly tick: FieldSpec<"u32">; readonly dt: FieldSpec<"f32">; readonly elapsed: FieldSpec<"f64">; readonly rate: FieldSpec<"u32"> }, Time>;
  /**
   * The world's main random stream, seeded from GameDesc.seed (or the replay seed). Its state is part of every snapshot.
   */
  const Rng: ResourceType<{ readonly seed: FieldSpec<"u32">; readonly state: FieldSpec<"u32"> }, Rng>;
  /**
   * The input state at the start of this tick (og-input's og.Input, version 2). Edges (pressed, released) last exactly one tick; read the state through the methods.
   * Key sets are 128 bits over og-input's KEY_CODES (KeyboardEvent.code names): code i is bit i % 32 of word i / 32. Button masks: left 1, middle 2, right 4.
   */
  const Input: ResourceType<{ readonly pointer: FieldSpec<"Vector2">; readonly pointerInside: FieldSpec<"bool">; readonly wheel: FieldSpec<"Vector2">; readonly buttons: FieldSpec<"u32">; readonly buttonsPressed: FieldSpec<"u32">; readonly buttonsReleased: FieldSpec<"u32">; readonly buttonsReleaseNext: FieldSpec<"u32">; readonly keys0: FieldSpec<"u32">; readonly keys1: FieldSpec<"u32">; readonly keys2: FieldSpec<"u32">; readonly keys3: FieldSpec<"u32">; readonly pressed0: FieldSpec<"u32">; readonly pressed1: FieldSpec<"u32">; readonly pressed2: FieldSpec<"u32">; readonly pressed3: FieldSpec<"u32">; readonly released0: FieldSpec<"u32">; readonly released1: FieldSpec<"u32">; readonly released2: FieldSpec<"u32">; readonly released3: FieldSpec<"u32">; readonly releaseNext0: FieldSpec<"u32">; readonly releaseNext1: FieldSpec<"u32">; readonly releaseNext2: FieldSpec<"u32">; readonly releaseNext3: FieldSpec<"u32"> }, Input>;
  /**
   * Every pointer on the play surface at once, at the start of this tick (og-input's og.Pointers): a touch stick and a touch button can be held together. Pointer 0 is og.Input's pointer; ids 1 to 4 are further touches, in the order they landed (the lowest free id).
   * Edges last exactly one tick, as og.Input's do. Masks are bit `id`; read them through the methods.
   */
  const Pointers: ResourceType<{ readonly heldMask: FieldSpec<"u32">; readonly pressedMask: FieldSpec<"u32">; readonly releasedMask: FieldSpec<"u32">; readonly releaseNextMask: FieldSpec<"u32">; readonly position0: FieldSpec<"Vector2">; readonly position1: FieldSpec<"Vector2">; readonly position2: FieldSpec<"Vector2">; readonly position3: FieldSpec<"Vector2">; readonly position4: FieldSpec<"Vector2">; readonly start0: FieldSpec<"Vector2">; readonly start1: FieldSpec<"Vector2">; readonly start2: FieldSpec<"Vector2">; readonly start3: FieldSpec<"Vector2">; readonly start4: FieldSpec<"Vector2"> }, Pointers>;
  /**
   * The first four gamepads in the W3C standard mapping, at the start of this tick (og-input's og.Gamepad). On the web the shell polls the pads each display frame and sends their state through the input ring, so a pad is world state and replays; it also maps pads to keys for games that only read keys.
   * Button masks are bit `GamepadButton` index per pad; edges last exactly one tick, and a press and a release between two ticks leave both edges set. Read them through the methods.
   */
  const Gamepad: ResourceType<{ readonly connectedMask: FieldSpec<"u32">; readonly buttons0: FieldSpec<"u32">; readonly buttons1: FieldSpec<"u32">; readonly buttons2: FieldSpec<"u32">; readonly buttons3: FieldSpec<"u32">; readonly pressed0: FieldSpec<"u32">; readonly pressed1: FieldSpec<"u32">; readonly pressed2: FieldSpec<"u32">; readonly pressed3: FieldSpec<"u32">; readonly released0: FieldSpec<"u32">; readonly released1: FieldSpec<"u32">; readonly released2: FieldSpec<"u32">; readonly released3: FieldSpec<"u32">; readonly leftStick0: FieldSpec<"Vector2">; readonly leftStick1: FieldSpec<"Vector2">; readonly leftStick2: FieldSpec<"Vector2">; readonly leftStick3: FieldSpec<"Vector2">; readonly rightStick0: FieldSpec<"Vector2">; readonly rightStick1: FieldSpec<"Vector2">; readonly rightStick2: FieldSpec<"Vector2">; readonly rightStick3: FieldSpec<"Vector2">; readonly triggers0: FieldSpec<"Vector2">; readonly triggers1: FieldSpec<"Vector2">; readonly triggers2: FieldSpec<"Vector2">; readonly triggers3: FieldSpec<"Vector2"> }, Gamepad>;
  /**
   * A key went down (a down for a held key is a repeat and sets no edge). Input events are sent between ticks and read by the next tick's systems.
   */
  const KeyDown: EventType<KeyDown>;
  /** A key went up. */
  const KeyUp: EventType<KeyUp>;
  /** A pointer button went down (or a touch landed). */
  const PointerDown: EventType<PointerDown>;
  /** A pointer button went up (or a touch lifted). */
  const PointerUp: EventType<PointerUp>;
  /** A pointer moved. */
  const PointerMove: EventType<PointerMove>;
  /** A wheel or trackpad scroll. */
  const Wheel: EventType<Wheel>;
  /**
   * The camera the frame is rendered from. Rendering reads it after the tick; it never feeds back into the simulation.
   */
  const Camera: ResourceType<{ readonly position: FieldSpec<"Vector3">; readonly quaternion: FieldSpec<"Quaternion">; readonly fov: FieldSpec<"f32">; readonly near: FieldSpec<"f32">; readonly far: FieldSpec<"f32">; readonly zoom: FieldSpec<"f32">; readonly orthographic: FieldSpec<"bool">; readonly height: FieldSpec<"f32">; readonly background: FieldSpec<"Color">; readonly exposure: FieldSpec<"f32"> }, Camera>;
  /**
   * Orbits og.Camera around the entity carrying it, as three.js OrbitControls: a left-button drag on the play surface turns it, the wheel zooms, autoRotate turns it on its own.
   * Camera controllers are game-level systems the engine ships in TypeScript (og.systems.cameras); list them in GameDesc.systems: `systems: [...og.systems.cameras, ...yours]`. The first entity carrying one, in query order, drives the camera.
   * The camera sits `distance` from the orbited point (the entity's position plus `target`) in the direction given by `azimuth` about +Y and `polar` from +Y, and looks at the point. Every angle is computed with deterministic arithmetic, so a replay moves the camera identically on every host.
   */
  const OrbitCamera: ComponentType<{ readonly target: FieldSpec<"Vector3">; readonly distance: FieldSpec<"f32">; readonly minDistance: FieldSpec<"f32">; readonly maxDistance: FieldSpec<"f32">; readonly azimuth: FieldSpec<"f32">; readonly polar: FieldSpec<"f32">; readonly minPolarAngle: FieldSpec<"f32">; readonly maxPolarAngle: FieldSpec<"f32">; readonly rotateSpeed: FieldSpec<"f32">; readonly zoomSpeed: FieldSpec<"f32">; readonly autoRotate: FieldSpec<"bool">; readonly autoRotateSpeed: FieldSpec<"f32">; readonly enabled: FieldSpec<"bool">; readonly dragging: FieldSpec<"bool">; readonly pointer: FieldSpec<"Vector2"> }, OrbitCamera>;
  /**
   * Chases the entity carrying it with og.Camera: a third-person 3D view that eases towards a point offset from the entity and looks at it. Driven by og.systems.cameras; the first entity carrying one, in query order, drives the camera.
   */
  const FollowCamera: ComponentType<{ readonly offset: FieldSpec<"Vector3">; readonly lookOffset: FieldSpec<"Vector3">; readonly followRotation: FieldSpec<"bool">; readonly smoothTime: FieldSpec<"f32"> }, FollowCamera>;
  /**
   * Keeps og.Camera on the entity carrying it in the xy plane, for 2D games: the camera moves only when the entity leaves the dead zone, eases by smoothTime and, with clamp, stays inside the bounds. The camera keeps its z, rotation and projection. Driven by og.systems.cameras.
   */
  const Follow2D: ComponentType<{ readonly offset: FieldSpec<"Vector2">; readonly deadZone: FieldSpec<"Vector2">; readonly smoothTime: FieldSpec<"f32">; readonly clamp: FieldSpec<"bool">; readonly boundsMin: FieldSpec<"Vector2">; readonly boundsMax: FieldSpec<"Vector2"> }, Follow2D>;
  /**
   * Draws a geometry with a material at the entity's MatrixWorld, as a three.js Mesh.
   * The material is a handle from og.render.standardMaterial or from a loaded glTF (GltfInfo.materials); its StandardMaterialDesc maps to the renderer as that struct's docs say.
   */
  const Mesh: ComponentType<{ readonly geometry: FieldSpec<"Geometry">; readonly material: FieldSpec<"Material">; readonly castShadow: FieldSpec<"bool">; readonly receiveShadow: FieldSpec<"bool">; readonly visible: FieldSpec<"bool"> }, Mesh>;
  /**
   * The sun, as three.js DirectionalLight: parallel light along the entity's local -Z (aim it with transform.lookAt, as a camera). The first one found is drawn, with three shadow cascades.
   * Without one, the renderer's default sun (from above and to the side, intensity 3) lights the scene.
   */
  const DirectionalLight: ComponentType<{ readonly color: FieldSpec<"Color">; readonly intensity: FieldSpec<"f32">; readonly castShadow: FieldSpec<"bool">; readonly shadowDistance: FieldSpec<"f32"> }, DirectionalLight>;
  /**
   * A light shining in every direction from the entity's position, as three.js PointLight. Its radiance at distance d is color * intensity / d^2, faded smoothly to zero at distance.
   * Up to MAX_LIGHTS point and spot lights are drawn per frame (the rest are counted, not drawn); the first two with castShadow get cube shadows.
   */
  const PointLight: ComponentType<{ readonly color: FieldSpec<"Color">; readonly intensity: FieldSpec<"f32">; readonly distance: FieldSpec<"f32">; readonly castShadow: FieldSpec<"bool"> }, PointLight>;
  /** A cone of light along the entity's local -Z, as three.js SpotLight. */
  const SpotLight: ComponentType<{ readonly color: FieldSpec<"Color">; readonly intensity: FieldSpec<"f32">; readonly distance: FieldSpec<"f32">; readonly angle: FieldSpec<"f32">; readonly penumbra: FieldSpec<"f32">; readonly castShadow: FieldSpec<"bool"> }, SpotLight>;
  /**
   * The frame's look after lighting: environment light, sky, bloom, exposure metering and tone mapping. Read after each tick; it never feeds back into the simulation. Camera.exposure is the manual exposure.
   */
  const RenderSettings: ResourceType<{ readonly environmentIntensity: FieldSpec<"f32">; readonly sky: FieldSpec<"bool">; readonly bloom: FieldSpec<"bool">; readonly bloomThreshold: FieldSpec<"f32">; readonly bloomKnee: FieldSpec<"f32">; readonly bloomIntensity: FieldSpec<"f32">; readonly autoExposure: FieldSpec<"bool">; readonly toneMapped: FieldSpec<"bool"> }, RenderSettings>;
  /**
   * What is drawn: meshes with materials. Draws are instanced by (geometry, material) automatically; there is no per-object draw call.
   */
  namespace render {
    /**
     * Draw a sprite this frame. Systems and commands may call it; the list is cleared at every tick, so submit it every tick the sprite should show.
     *
     * @param world The world whose renderer draws it.
     * @param desc The texture, placement, region and colour.
     * @returns Refused for a dead texture.
     */
    function sprite(world: World, desc?: SpriteDesc): Result<void>;
    /**
     * The size text takes when og.render.text draws it, for centring, wrapping and layout.
     *
     * @param world The world whose renderer owns the font.
     * @param desc The font, text and size.
     * @returns Refused for a dead font.
     */
    function measureText(world: World, desc?: TextMeasureDesc): Result<TextMetrics>;
    /**
     * Make a box geometry for og.Mesh. Physics can use it as a convex or triangle-mesh Collider.mesh too.
     *
     * @param world The world whose renderer owns the geometry.
     * @param desc Its extents.
     */
    function boxGeometry(world: World, desc?: BoxGeometryDesc): Result<Geometry>;
    /**
     * Make a plane geometry for og.Mesh. Physics can use it as a triangle-mesh Collider.mesh too.
     *
     * @param world The world whose renderer owns the geometry.
     * @param desc Its extents.
     */
    function planeGeometry(world: World, desc?: PlaneGeometryDesc): Result<Geometry>;
    /**
     * Make a sphere geometry for og.Mesh (give its body a sphere Collider, not this mesh).
     *
     * @param world The world whose renderer owns the geometry.
     * @param desc Its radius and tessellation.
     */
    function sphereGeometry(world: World, desc?: SphereGeometryDesc): Result<Geometry>;
    /**
     * Create a standard material.
     *
     * @param world The world whose renderer owns the material.
     * @param desc The parameters; omitted ones take three.js defaults.
     */
    function standardMaterial(world: World, desc?: StandardMaterialDesc): Result<Material>;
    /**
     * Change some parameters of a standard material.
     *
     * @param world The world whose renderer owns the material.
     * @param material The material.
     * @param desc The parameters to change; omitted ones keep their values.
     * @returns Refused for a dead material.
     */
    function updateMaterial(world: World, material: Material, desc?: StandardMaterialDesc): Result<void>;
    /**
     * The renderer's tier and limits on this device.
     *
     * @param world The world whose renderer is asked.
     */
    function capabilities(world: World): RenderCapabilities;
    /**
     * Draw text this frame. Systems and commands may call it; the list is cleared at every tick, so submit it every tick the text should show. Multi-page fonts are drawn page by page with one layout.
     *
     * @param world The world whose renderer draws it.
     * @param desc The font, text, placement and colour.
     * @returns Refused for a dead font.
     */
    function text(world: World, desc?: TextDesc): Result<void>;
  }
  /**
   * Draws a loaded glTF's default scene at the entity's world transform: every mesh node with its glTF material. Scene entities with a `model` line get one; world.instantiate of a glTF asset spawns one.
   */
  const Model: ComponentType<{ readonly asset: FieldSpec<"Asset"> }, Model>;
  /** A load completed. */
  const AssetReady: EventType<AssetReady>;
  /** A load failed. */
  const AssetFailed: EventType<AssetFailed>;
  /**
   * Asset loading from cooked output: `og cook` turns assets/ into cooked/ (.ogb blobs and the manifest cooked/.cook.ogt), and the runtime reads only cooked/.
   * Paths are asset ids: the path under assets/ (`models/ship.glb`, `scenes/main.ogt`); an `assets/` prefix is accepted. Every blob is checked against the manifest's type, schema version and cook stamp.
   * The game's bundle (`cooked`) is opened before the first tick and GameDesc.scene is spawned then. Loads from an open bundle complete at once on both hosts (the web host fetches the bundle's files before the engine starts); only og.load.bundle of another bundle can be pending, and it completes at a tick boundary.
   */
  namespace load {
    /**
     * Open another cooked bundle (the game's own `cooked` is open before the first tick). Its loads read through it once it is ready.
     *
     * @param world The world that owns the asset.
     * @param path The cooked directory, relative to the project (or the page), such as `dlc/cooked`.
     * @returns The bundle's handle: pending until its manifest is read, then ready or failed.
     */
    function bundle(world: World, path: string): Result<Asset>;
    /**
     * What an open bundle holds.
     *
     * @param world The world that owns the asset.
     * @param asset A bundle handle; the game's own bundle is og.load.bundle's first answer, also given by `defaultBundle`.
     * @returns Refused for a pending, failed or non-bundle asset.
     */
    function bundleInfo(world: World, asset: Asset): Result<BundleInfo>;
    /**
     * The game's own bundle, opened before the first tick from `cooked/`.
     *
     * @param world The world that owns the asset.
     * @returns Refused when cooked/ was missing or unreadable (run og cook); the reason names the path.
     */
    function defaultBundle(world: World): Result<Asset>;
    /**
     * Load a cooked glTF (.gltf or .glb), as three.js GLTFLoader.load: its meshes, materials and textures join the renderer once.
     *
     * @param world The world that owns the asset.
     * @param path The asset id, such as `models/ship.glb`.
     * @param desc Scene and shadow options.
     * @returns The handle, or why it was refused (an uncooked path, a cook stamp mismatch, no open bundle).
     */
    function gltf(world: World, path: string, desc?: GltfDesc): Result<Asset>;
    /**
     * Load a cooked scene (.ogt). world.instantiate spawns its entities: depth first, parents before children, each component table matched to a registered component by its reflected name (`Transform` is og.Transform) and migrated field by field; a field the scene lacks keeps its default, one the component lacks is dropped with a warning.
     *
     * @param world The world that owns the asset.
     * @param path The scene's asset id, such as `scenes/main.ogt`.
     */
    function scene(world: World, path: string): Result<Asset>;
    /**
     * Load a cooked MSDF font for og.render.text.
     *
     * @param world The world that owns the font.
     * @param path The font's asset id, such as `fonts/NotoSansSC-Regular-subset.ttf`.
     */
    function font(world: World, path: string): Result<Font>;
    /**
     * A loaded font's metrics.
     *
     * @param world The world that owns the font.
     * @param font The font.
     * @returns Refused for a dead font.
     */
    function fontInfo(world: World, font: Font): Result<FontInfo>;
    /**
     * Load a cooked sound for the audio module.
     *
     * @param world The world that owns the sound.
     * @param path The sound's asset id, such as `audio/blip.wav`.
     */
    function sound(world: World, path: string): Result<Sound>;
    /**
     * Load a cooked texture for og.render.sprite and og.ui.image, as three.js TextureLoader.load.
     *
     * @param world The world whose renderer owns the texture.
     * @param path The texture's asset id, such as `textures/stone.png`.
     */
    function texture(world: World, path: string): Result<Texture>;
    /**
     * A loaded sound's format and length.
     *
     * @param world The world that owns the sound.
     * @param sound The sound.
     * @returns Refused for a dead sound.
     */
    function soundInfo(world: World, sound: Sound): Result<SoundInfo>;
    /**
     * Where a load is.
     *
     * @param world The world that owns the asset.
     * @param asset The handle.
     */
    function status(world: World, asset: Asset): LoadStatus;
    /**
     * Why a load failed.
     *
     * @param world The world that owns the asset.
     * @param asset The handle.
     * @returns The reason, naming the fix; null unless the load failed.
     */
    function error(world: World, asset: Asset): string | null;
    /**
     * What a ready glTF holds.
     *
     * @param world The world that owns the asset.
     * @param asset A ready glTF handle.
     * @returns Refused for a pending, failed or non-glTF asset.
     */
    function gltfInfo(world: World, asset: Asset): Result<GltfInfo>;
    /**
     * Load a cooked glTF's animation (its `<model>#anim` blob: the rig, the clips, the skin and morph streams) and register its skeleton and clips with og.anim. Loading the same model again answers the same handles.
     *
     * @param world The world whose animation owns the assets.
     * @param path The model's asset id, such as `models/walker.glb` (a `#anim` suffix is accepted).
     * @returns Refused for an uncooked path, a model the cook wrote no `#anim` blob for (no skins, clips or morph targets), a cook stamp mismatch or an invalid rig.
     */
    function animation(world: World, path: string): Result<Animation>;
  }
  /**
   * A body the physics step simulates. It needs an og.Transform and usually has an og.Collider; a dynamic body without an og.Velocity gets one.
   * Changing a RigidBody, a Collider or Transform.scale rebuilds the body (it loses its contacts' warm starts); a Transform or Velocity that game code changed moves it (a teleport).
   */
  const RigidBody: ComponentType<{ readonly kind: FieldSpec<"u32">; readonly mass: FieldSpec<"f32">; readonly linearDamping: FieldSpec<"f32">; readonly angularDamping: FieldSpec<"f32">; readonly gravityScale: FieldSpec<"f32">; readonly lock: FieldSpec<"u32">; readonly ccd: FieldSpec<"u32"> }, RigidBody>;
  /**
   * The shape an entity collides with, attached to its RigidBody (or, without one, a static collider at its Transform). Sizes are scaled by og.Transform.scale.
   */
  const Collider: ComponentType<{ readonly shape: FieldSpec<"u32">; readonly size: FieldSpec<"Vector3">; readonly radius: FieldSpec<"f32">; readonly height: FieldSpec<"f32">; readonly mesh: FieldSpec<"Geometry">; readonly offset: FieldSpec<"Vector3">; readonly rotation: FieldSpec<"Quaternion">; readonly friction: FieldSpec<"f32">; readonly restitution: FieldSpec<"f32">; readonly density: FieldSpec<"f32">; readonly sensor: FieldSpec<"u32">; readonly layers: FieldSpec<"u32">; readonly mask: FieldSpec<"u32"> }, Collider>;
  /**
   * Linear and angular velocity. Written for dynamic bodies every tick (added to them by the step when missing), read for kinematic ones; set it to launch or stop a body.
   */
  const Velocity: ComponentType<{ readonly linear: FieldSpec<"Vector3">; readonly angular: FieldSpec<"Vector3"> }, Velocity>;
  /**
   * The physics settings of the world. The profile is the project's; the same components serve both profiles.
   */
  const Physics: ResourceType<{ readonly profile: FieldSpec<"u32">; readonly gravity: FieldSpec<"Vector3">; readonly substeps: FieldSpec<"u32">; readonly solverIterations: FieldSpec<"u32">; readonly restSpeed: FieldSpec<"f32"> }, Physics>;
  /** Two solid colliders started touching during the tick. */
  const CollisionStarted: EventType<CollisionStarted>;
  /** Two solid colliders stopped touching, or one of them went away. */
  const CollisionEnded: EventType<CollisionEnded>;
  /** A collider started overlapping a sensor. */
  const SensorEntered: EventType<SensorEntered>;
  /** A collider stopped overlapping a sensor, or one of them went away. */
  const SensorExited: EventType<SensorExited>;
  /**
   * One pair in contact at the end of a step, sent every tick for every touching pair: how a game asks what is touching what right now.
   */
  const Contact: EventType<Contact>;
  /**
   * Rigid bodies over rapier (rapier3d in the 3D profile, rapier2d in 2D, both with enhanced determinism): game code owns the components, the engine's og.physics system steps them after the game's systems and writes the dynamic bodies back.
   * The profile is the project's (`profile "2d"` or `"3d"` in project.ogt). In 2D, bodies move in the xy plane and rotate about z; a transform keeps its z. A collider without a RigidBody is static at its transform.
   * The rapier world persists between ticks and is world state: snapshots, restore and the world hash cover the whole simulation. Events sent in tick N are read in tick N + 1.
   */
  namespace physics {
    /**
     * The nearest collider along a ray. Inside a system it sees the colliders as they were at the start of the tick (og.physics.queries records them before the game's systems); between ticks, as the world holds them.
     *
     * @param world The world.
     * @param desc The ray and the filter.
     * @returns The hit, or null when nothing is hit; refused for a zero direction.
     */
    function raycast(world: World, desc?: RaycastDesc): Result<RaycastHit | null>;
    /**
     * The first collider a shape meets when swept along a direction; the same view of the colliders as raycast.
     *
     * @param world The world.
     * @param desc The shape, its sweep and the filter.
     * @returns The hit, or null; refused for a zero direction or an unknown mesh.
     */
    function shapeCast(world: World, desc?: ShapeCastDesc): Result<ShapeCastHit | null>;
    /**
     * Every collider overlapping a shape, sorted by entity id; the same view of the colliders as raycast.
     *
     * @param world The world.
     * @param desc The shape, its placement and the filter.
     */
    function overlap(world: World, desc?: OverlapDesc): Result<readonly Entity[]>;
    /**
     * Register (or replace) the geometry a convex or triangle-mesh Collider names by its mesh handle. Loaded glTF geometries are registered under their own handles already; meshes are assets, not world state.
     *
     * @param world The world.
     * @param mesh The handle colliders name in Collider.mesh.
     * @param desc Positions and triangles.
     * @returns Refused for an empty, non-finite or out-of-range mesh.
     */
    function setMesh(world: World, mesh: Geometry, desc?: MeshDesc): Result<void>;
  }
  /**
   * A posed skeleton: the entity is drawn skinned by the joint palette og.anim writes each tick. The pose itself (every node's local transform) is world state kept by the `og.anim` state block.
   */
  const Skeleton: ComponentType<{ readonly asset: FieldSpec<"SkeletonAsset">; readonly skin: FieldSpec<"u32">; readonly rootMotion: FieldSpec<"RootMotion">; readonly rootJoint: FieldSpec<"u32">; readonly jointCount: FieldSpec<"u32"> }, Skeleton>;
  /**
   * Plays one clip on a skeleton. Alone it poses the skeleton from its rest pose; with an og.AnimationGraph it plays over the graph's result with `weight`. Changing `clip` cross-fades from the previous one over `fade`; a clip change starts the new clip at 0 (or at `time` if the game wrote it in the same tick).
   */
  const AnimationPlayer: ComponentType<{ readonly clip: FieldSpec<"ClipAsset">; readonly time: FieldSpec<"f32">; readonly speed: FieldSpec<"f32">; readonly loop: FieldSpec<"bool">; readonly weight: FieldSpec<"f32">; readonly fade: FieldSpec<"f32">; readonly finished: FieldSpec<"bool"> }, AnimationPlayer>;
  /**
   * Runs a blend-tree graph (og.anim.graph) on a skeleton: its layers' state machines pick motions by the parameters p0...p7, which the graph names in `param` order (`param speed` is p0, the next `param` is p1, ...).
   */
  const AnimationGraph: ComponentType<{ readonly asset: FieldSpec<"GraphAsset">; readonly weight: FieldSpec<"f32">; readonly speed: FieldSpec<"f32">; readonly p0: FieldSpec<"f32">; readonly p1: FieldSpec<"f32">; readonly p2: FieldSpec<"f32">; readonly p3: FieldSpec<"f32">; readonly p4: FieldSpec<"f32">; readonly p5: FieldSpec<"f32">; readonly p6: FieldSpec<"f32">; readonly p7: FieldSpec<"f32">; readonly state: FieldSpec<"u32">; readonly stateTime: FieldSpec<"f32"> }, AnimationGraph>;
  /**
   * Morph-target weights of the skeleton entity's mesh, in the order of the glTF mesh's targets. With `source` clips the playing clips' weight channels write them; with `component` game code does.
   */
  const Blendshapes: ComponentType<{ readonly source: FieldSpec<"BlendshapeSource">; readonly mesh: FieldSpec<"u32">; readonly w0: FieldSpec<"f32">; readonly w1: FieldSpec<"f32">; readonly w2: FieldSpec<"f32">; readonly w3: FieldSpec<"f32">; readonly w4: FieldSpec<"f32">; readonly w5: FieldSpec<"f32">; readonly w6: FieldSpec<"f32">; readonly w7: FieldSpec<"f32"> }, Blendshapes>;
  /**
   * An inverse-kinematics constraint on a skeleton, applied after its clips and graph: the chain of `bones` bones ending at joint `tip` turns so the tip reaches `target`. Put it on its own entity (a skeleton can have many chains); chains apply in query order.
   */
  const IkChain: ComponentType<{ readonly skeleton: FieldSpec<"Entity">; readonly tip: FieldSpec<"u32">; readonly bones: FieldSpec<"u32">; readonly solver: FieldSpec<"IkSolver">; readonly target: FieldSpec<"Vector3">; readonly pole: FieldSpec<"Vector3">; readonly space: FieldSpec<"IkSpace">; readonly weight: FieldSpec<"f32">; readonly iterations: FieldSpec<"u32">; readonly tolerance: FieldSpec<"f32">; readonly reached: FieldSpec<"bool">; readonly error: FieldSpec<"f32"> }, IkChain>;
  /**
   * A named marker of a clip that playback passed in the last tick (read in the tick after it fired, like every event). Cross-fades and blends fire every contributing clip's markers.
   */
  const AnimationEvent: EventType<AnimationEvent>;
  /**
   * Skeletal animation over og-anim: glTF skins and clips (cooked into the `<model>#anim` blob, loaded with og.load.animation), blend-tree graphs with layered state machines, root motion, two-bone and FABRIK IK and blendshapes, evaluated by the engine's og.anim system after the game's systems and before physics.
   * Game code owns the components and sets players, graph parameters and IK targets; og.anim writes the pose (kept as world state in the `og.anim` state block, so snapshots, restore and the world hash cover it), the joint palette, root motion into og.Transform and the fields marked written. Asset handles are what og.load.animation and og.anim.graph answer; 0 is none.
   * Clips bind to a skeleton's nodes by name, so a clip of one rig plays on another with the same node names. Assets are not world state: a hot reload registers them again.
   */
  namespace anim {
    /**
     * Register a blend-tree graph for AnimationGraph.asset. Its `clip` lines are resolved by name against `clips`.
     *
     * @param world The world whose animation owns the graph.
     * @param desc The text and the clips it may name.
     * @returns Refused for a parse error (the reason names the line), a clip name `clips` lacks, or an invalid graph.
     */
    function graph(world: World, desc?: GraphDesc): Result<GraphAsset>;
  }
  /**
   * A sound an entity plays. Level-triggered: while `playing` is true it plays (inserting a source plays it), setting it false stops it, and a one-shot that reaches its end sets it back to false and sends og.SoundEnded.
   * A one-shot of F frames at rate R and pitch p started in tick S ends in tick S + ceil(F * tickRate / (R * p)) - 1. A despawned entity's voice is silenced without an event.
   */
  const AudioSource: ComponentType<{ readonly sound: FieldSpec<"Sound">; readonly bus: FieldSpec<"AudioBus">; readonly volume: FieldSpec<"f32">; readonly pitch: FieldSpec<"f32">; readonly pan: FieldSpec<"f32">; readonly loop: FieldSpec<"bool">; readonly loopStart: FieldSpec<"f32">; readonly loopEnd: FieldSpec<"f32">; readonly spatial: FieldSpec<"bool">; readonly refDistance: FieldSpec<"f32">; readonly maxDistance: FieldSpec<"f32">; readonly rolloffFactor: FieldSpec<"f32">; readonly playing: FieldSpec<"bool"> }, AudioSource>;
  /**
   * Where spatial sources are heard from: the entity's og.Transform, looking down its -Z with +X to the right. The first listener in query order is used.
   */
  const AudioListener: ComponentType<{ readonly volume: FieldSpec<"f32"> }, AudioListener>;
  /** Bus volumes; master scales every bus. A change ramps over 256 frames in the mixer. */
  const Audio: ResourceType<{ readonly master: FieldSpec<"f32">; readonly music: FieldSpec<"f32">; readonly sfx: FieldSpec<"f32">; readonly ui: FieldSpec<"f32"> }, Audio>;
  /** A source started, sent in the tick it happens (systems read it the tick after). */
  const SoundStarted: EventType<SoundStarted>;
  /** A sound ended, sent in the tick it happens (systems read it the tick after). */
  const SoundEnded: EventType<SoundEnded>;
  /**
   * The engine's UI: an immediate-mode toolkit. Systems describe the tree every tick with these calls; the engine lays it out Clay-style, themes it with the platform's design tokens, draws it over the frame and answers the probe (dump_ui, touch_move_to, move_to_text) from it.
   * The engine opens the frame before the game's systems (from og.Input and the play surface) and closes it after them, so calls belong inside systems; outside one they do nothing and log a warning. Interaction is decided against the previous tick's rectangles, which are also what the probe resolves, so a scripted touch_move_to and a click agree; response edges (clicked, changed, submitted) last one tick.
   * Lengths are CSS pixels of the play surface (origin top left, y down), the unit of og.Input and the probe. Ids: an element's id is unique among its siblings (`idi("Slot", 3)` is `Slot#3`); a path `Inventory/Slot#3` names it after its nearest ancestors. The UI state (hover, press, focus, caret, scroll) is world state: snapshots and the world hash carry it.
   */
  namespace ui {
    /**
     * Set the scheme, the density and the fonts, typically in GameDesc.start. Takes effect from the next tick.
     *
     * @param world The world whose UI is configured.
     * @param desc The scheme, density and fonts.
     * @returns Refused for a dead font.
     */
    function configure(world: World, desc?: UiDesc): Result<void>;
    /**
     * This tick's play surface: the render target's size in CSS pixels, its DPR and safe area, as the host reports them.
     *
     * @param world The world.
     */
    function viewport(world: World): Viewport;
    /**
     * The design tokens the UI draws with.
     *
     * @param world The world.
     */
    function theme(world: World): UiTheme;
    /**
     * Open a container; the calls until its close() build its children.
     *
     * @param id Its id, unique among its siblings.
     * @param desc Its layout and look.
     */
    function open(id: string, desc?: ElementDesc): void;
    /**
     * Open a card, closed by close(): og.ui.panel without the callback.
     *
     * @param id Its id.
     * @param desc Its layout and look.
     */
    function openPanel(id: string, desc?: PanelDesc): void;
    /**
     * Open a vertical scroll container, closed by close(): og.ui.scroll without the callback.
     *
     * @param id Its id.
     * @param desc Its size and spacing.
     */
    function openScroll(id: string, desc?: ScrollDesc): void;
    /**
     * Close the open container.
     *
     * @returns Its response, for an interactive container; the zero response otherwise.
     */
    function close(): Response;
    /**
     * A row built by `build`: open(id, {...desc, direction: row}), build(), close().
     *
     * @param id Its id.
     * @param desc Its layout and look; direction is row.
     * @param build Makes its children.
     * @returns Its response, for an interactive row.
     */
    function row(id: string, desc: ElementDesc, build: UiBuild): Response;
    /**
     * A column built by `build`: open(id, {...desc, direction: column}), build(), close().
     *
     * @param id Its id.
     * @param desc Its layout and look; direction is column.
     * @param build Makes its children.
     * @returns Its response, for an interactive column.
     */
    function column(id: string, desc: ElementDesc, build: UiBuild): Response;
    /**
     * A card built by `build`: openPanel(id, desc), build(), close().
     *
     * @param id Its id.
     * @param desc Its layout and look.
     * @param build Makes its children.
     */
    function panel(id: string, desc: PanelDesc, build: UiBuild): void;
    /**
     * A vertical scroll container built by `build`: openScroll(id, desc), build(), close().
     *
     * @param id Its id.
     * @param desc Its size and spacing.
     * @param build Makes its children.
     */
    function scroll(id: string, desc: ScrollDesc, build: UiBuild): void;
    /**
     * MSDF text that wraps at its width (word breaks, any CJK character, long words between characters) unless nowrap. The probe reads its text.
     *
     * @param id Its id.
     * @param text The text.
     * @param desc Type step, ink, alignment, wrapping, width and font.
     */
    function label(id: string, text: string, desc?: LabelDesc): void;
    /**
     * A capsule button with a label; `clicked` is a press and a release inside it. The probe reads its label.
     *
     * @param id Its id.
     * @param text The label.
     * @param desc Variant, size, disabled and width.
     */
    function button(id: string, text: string, desc?: ButtonDesc): Response;
    /**
     * A texture region, or a solid rectangle.
     *
     * @param id Its id.
     * @param desc Texture, region, tint and size.
     */
    function image(id: string, desc?: ImageDesc): void;
    /**
     * A capsule track filled to a fraction.
     *
     * @param id Its id.
     * @param value How full it is, from 0 to 1.
     * @param desc Its width.
     */
    function progress(id: string, value: number, desc?: ProgressDesc): void;
    /**
     * A track with a fill and a thumb: a press anywhere on it sets the value and dragging follows. Pass back the value it answers next tick.
     *
     * @param id Its id.
     * @param value The value from game state.
     * @param desc Range, step and width.
     */
    function slider(id: string, value: number, desc?: SliderDesc): SliderResult;
    /**
     * A switch with a label; a click flips it. Pass back the state it answers next tick. The probe reads its label.
     *
     * @param id Its id.
     * @param on The state from game state.
     * @param label The label.
     */
    function toggle(id: string, on: boolean, label: string): ToggleResult;
    /**
     * One line of text: a press focuses it with the caret at the press, typed text inserts at the caret, Backspace, Delete, arrows, Home and End edit, Enter submits and leaves, Escape leaves. Pass back the text it answers next tick. The probe reads its text, or the placeholder while empty.
     *
     * @param id Its id.
     * @param value The text from game state.
     * @param desc Placeholder, width and length limit.
     */
    function textField(id: string, value: string, desc?: FieldDesc): FieldResult;
    /**
     * A tooltip for the element just built: shown below it (above without room) once the pointer has rested on it for the slow motion duration, drawn over everything. The probe reads its text.
     *
     * @param text The text.
     */
    function tooltip(text: string): void;
    /**
     * The id of the index-th of several siblings of one name: `idi("Slot", 3)` is `Slot#3`.
     *
     * @param name The name.
     * @param index The index.
     */
    function idi(name: string, index: number): string;
    /**
     * The size class of the play surface of the frame being built (desktop outside a frame).
     */
    function breakpoint(): Breakpoint;
    /**
     * The play surface of the frame being built: og.ui.viewport without the world, for code that only builds UI.
     */
    function frameViewport(): Viewport;
    /**
     * Whether the icon atlas has an icon of this name (templates draw a fallback otherwise).
     *
     * @param name The icon's name.
     */
    function hasIcon(name: string): boolean;
    /**
     * The UI's clock in seconds: frames built over the tick rate (simulation time, never the wall clock).
     */
    function seconds(): number;
    /**
     * The design tokens of the frame being built: og.ui.theme without the world, for code that only builds UI (the templates).
     */
    function frameTheme(): UiTheme;
    /**
     * Open an anchored group, closed by close(): og.ui.anchor without the callback.
     *
     * @param id Its id; the probe path is the id alone.
     * @param desc Where it sits and how it lays out.
     */
    function openAnchor(id: string, desc?: AnchorDesc): void;
    /**
     * An anchored group built by `build`: a layer at a point of the safe area, over the tree, for HUD corners and touch controls.
     *
     * @param id Its id.
     * @param desc Where it sits and how it lays out.
     * @param build Makes its children.
     */
    function anchor(id: string, desc: AnchorDesc, build: UiBuild): void;
    /**
     * Open a transition container (a column), closed by close(); build its children only while the answer is visible: it stays visible while it leaves. Its content takes no input while it moves.
     *
     * @param id Its id.
     * @param show Whether it should be shown.
     * @param desc How it moves.
     */
    function openTransition(id: string, show: boolean, desc?: TransitionDesc): Presence;
    /**
     * Content that enters and leaves: fade, slide or scale by the tick from the motion tokens. `build` runs while it is visible.
     *
     * @param id Its id.
     * @param show Whether it should be shown.
     * @param desc How it moves.
     * @param build Makes its content.
     */
    function transition(id: string, show: boolean, desc: TransitionDesc, build: UiBuild): Presence;
    /**
     * Open a dialog, closed by closeModal(); build its content only while the answer is visible. The probe sees the backdrop as `<id>` and the panel as `<id>/Panel`.
     *
     * @param id Its id.
     * @param open Whether it should be open.
     * @param desc Title, width and dismissal rules.
     */
    function openModal(id: string, open: boolean, desc?: ModalDesc): ModalResult;
    /** Close a dialog opened with openModal (its panel and backdrop). */
    function closeModal(): void;
    /**
     * A dialog built by `build` while visible: scrim, centred panel, focus kept inside, Back and the backdrop dismiss it unless locked. Set your open flag false on `dismissed`.
     *
     * @param id Its id.
     * @param open Whether it should be open.
     * @param desc Title, width and dismissal rules.
     * @param build Makes its content.
     */
    function modal(id: string, open: boolean, desc: ModalDesc, build: UiBuild): ModalResult;
    /**
     * A spotlight over everything (a tutorial step): the scrim on four sides of the hole, an amber edge around it. Presses on the dimmed parts stop there (clicked); presses in the hole reach what is under it. Shades are `<id>/Top`, `Left`, `Right`, `Bottom`; the hole is `<id>/Hole`.
     *
     * @param id Its id.
     * @param hole The rectangle to leave lit, as og.ui.bounds answers; null dims everything.
     * @param desc Room around the hole.
     */
    function spotlight(id: string, hole: UiRect | null, desc?: SpotlightDesc): Response;
    /**
     * Where an element was in the last frame, by id or `Parent/Child` path (topmost first), as touch_move_to resolves it.
     *
     * @param path The id or path.
     */
    function bounds(path: string): UiRect | null;
    /**
     * Open a virtualised list (a scroll column) and learn which rows to build: listItem(i) and close() for each, then closeList().
     *
     * @param id Its id.
     * @param desc Row count and size.
     */
    function openList(id: string, desc?: ListDesc): ListRange;
    /**
     * Open a virtualised grid and learn which cells to build: gridItem(i) and close() for each, then closeList().
     *
     * @param id Its id.
     * @param desc Cell count, size and columns.
     */
    function openGrid(id: string, desc?: GridDesc): ListRange;
    /**
     * Open row `index` of the open list (`Row#index`): selectable and focusable; its content follows until close(). A press that started on one row and is released on this one answers `dropped`.
     *
     * @param index Its index.
     * @param selected Drawn selected (fill-strong).
     */
    function listItem(index: number, selected: boolean): Response;
    /**
     * Open cell `index` of the open grid (`Cell#index`): a slot on fill, focusable; its content follows until close(). Drops answer `dropped` as listItem's.
     *
     * @param index Its index.
     * @param selected Drawn selected (fill-strong with an amber bar).
     */
    function gridItem(index: number, selected: boolean): Response;
    /** Close a list or grid opened with openList or openGrid. */
    function closeList(): Response;
    /**
     * The row or cell a held press started on in the list or grid of this id (built under the same parent): a drag in progress, to draw what is dragged under the pointer.
     *
     * @param id The list's or grid's id.
     */
    function dragging(id: string): number | null;
    /**
     * A virtualised list built by `build(index)` for the rows in view.
     *
     * @param id Its id.
     * @param desc Row count, size and the selected row.
     * @param build Makes one row's content.
     */
    function list(id: string, desc: ListDesc, build: UiItemBuild): ListResult;
    /**
     * A virtualised grid built by `build(index)` for the cells in view.
     *
     * @param id Its id.
     * @param desc Cell count, size, columns and the selected cell.
     * @param build Makes one cell's content.
     */
    function grid(id: string, desc: GridDesc, build: UiItemBuild): ListResult;
    /**
     * A segmented control: a neutral track, the chosen tab on the thumb. Tabs are `Tab#i` and the probe reads their labels.
     *
     * @param id Its id.
     * @param items The tab labels.
     * @param selected The chosen index from game state.
     * @param desc Fill and width.
     */
    function tabs(id: string, items: readonly string[], selected: number, desc?: TabsDesc): ChoiceResult;
    /**
     * A select: a field-like trigger with the chosen label and a chevron; a press or Activate opens its menu `<id>.menu` (options `Option#i`) under it, a focus scope; choosing closes it with changed. Back or a press outside closes it.
     *
     * @param id Its id.
     * @param options The option labels.
     * @param selected The chosen index from game state.
     * @param desc Placeholder, width, disabled.
     */
    function select(id: string, options: readonly string[], selected: number, desc?: SelectDesc): ChoiceResult;
    /**
     * A number stepper: `Dec`, `Value`, `Inc`. The buttons step within the range; Left and Right step it while a button is focused.
     *
     * @param id Its id.
     * @param value The value from game state.
     * @param desc Range, step and digits.
     */
    function stepper(id: string, value: number, desc?: StepperDesc): StepperResult;
    /**
     * A radio group of `Option#i` rows: a ring, filled amber when chosen, and a label.
     *
     * @param id Its id.
     * @param options The option labels.
     * @param selected The chosen index from game state.
     * @param desc Row or column.
     */
    function radio(id: string, options: readonly string[], selected: number, desc?: RadioDesc): ChoiceResult;
    /**
     * A checkbox with a label; a press flips it.
     *
     * @param id Its id.
     * @param on The state from game state.
     * @param label The label.
     */
    function checkbox(id: string, on: boolean, label: string): ToggleResult;
    /**
     * Checkboxes in a column (`Option#i`).
     *
     * @param id Its id.
     * @param labels The labels.
     * @param values The states from game state.
     */
    function checkboxList(id: string, labels: readonly string[], values: readonly boolean[]): CheckListResult;
    /**
     * A chip: a capsule on fill; selected inverts to fg. For filters and tags.
     *
     * @param id Its id.
     * @param text The label.
     * @param desc Selected and disabled.
     */
    function chip(id: string, text: string, desc?: ChipDesc): Response;
    /**
     * A badge: a small status capsule in its tone.
     *
     * @param id Its id.
     * @param text The label (a count, a word).
     * @param tone Its tone.
     */
    function badge(id: string, text: string, tone: Tone): void;
    /**
     * A hairline in the border colour across its parent.
     *
     * @param id Its id.
     * @param vertical Stand it up (in a row).
     */
    function divider(id: string, vertical: boolean): void;
    /**
     * Empty space.
     *
     * @param id Its id.
     * @param size Its size along both axes; null grows into the free space.
     */
    function spacer(id: string, size: Size | null): void;
    /**
     * A key (fg-muted) and its value (fg, at the end) on one row: stats, settings summaries, receipts.
     *
     * @param id Its id.
     * @param key The key.
     * @param value The value.
     * @param mono The value in the monospaced font (prices, ids).
     */
    function keyValue(id: string, key: string, value: string, mono: boolean): void;
    /**
     * A skeleton: the neutral fill standing in for content that is loading, with a sheen that passes every 1.6 s (still under reduced motion).
     *
     * @param id Its id.
     * @param desc Width, line height and lines.
     */
    function skeleton(id: string, desc?: SkeletonDesc): void;
    /**
     * A loading spinner: eight dots around a circle, the bright one turning by the tick.
     *
     * @param id Its id.
     * @param size Its diameter in CSS px; 0 is the small control height.
     */
    function spinner(id: string, size: number): void;
    /**
     * An icon of the atlas (UiDesc.icons) by name; an unknown name draws an empty slot and warns. The probe reads the name.
     *
     * @param id Its id.
     * @param name The icon's name.
     * @param desc Size and ink.
     */
    function icon(id: string, name: string, desc?: IconDesc): void;
    /**
     * A health, mana or xp bar. A drop shows at once and leaves a pale trail that follows after a beat; a rise grows into place with a ghost of the target; both by the tick.
     *
     * @param id Its id.
     * @param value The current value.
     * @param max The full value.
     * @param desc Kind, label, size and notches.
     */
    function meter(id: string, value: number, max: number, desc?: MeterDesc): void;
    /**
     * A cooldown: a disc whose amber ring fills clockwise from the top as it runs out, bright and full when ready, a label or icon in the middle. A press reports clicked (ready or not).
     *
     * @param id Its id.
     * @param remaining How much of the cooldown is left, 0 to 1: 1 just used, 0 ready.
     * @param desc Size, label and icon.
     */
    function cooldown(id: string, remaining: number, desc?: CooldownDesc): Response;
    /**
     * A mini-map frame: the map in a border-strong frame, markers as dots (the player amber and ringed), an N mark and a zone label.
     *
     * @param id Its id.
     * @param desc Map, markers and label.
     */
    function minimap(id: string, desc?: MinimapDesc): void;
    /**
     * A dialogue box: a portrait slot, the speaker in amber ink and the line typed out by the tick. A press or Activate while typing shows the whole line; once shown it answers advanced. The line is `<id>/Body/Line` and the probe reads what is shown.
     *
     * @param id Its id.
     * @param desc Speaker, text, portrait and speed.
     */
    function dialogue(id: string, desc?: DialogueDesc): DialogueResult;
    /**
     * A virtual joystick: a base and a knob that follows a press that started on it. og.Input carries one pointer, so a stick and a button are not held at once yet.
     *
     * @param id Its id.
     * @param desc Size and dead zone.
     */
    function joystick(id: string, desc?: JoystickDesc): JoystickResult;
    /**
     * A d-pad of four arms (`Up`, `Down`, `Left`, `Right`), each at least 44 CSS px.
     *
     * @param id Its id.
     * @param desc Its size.
     */
    function dpad(id: string, desc?: DpadDesc): DpadResult;
    /**
     * A round touch action button: pressed while held (jump, fire), clicked on a tap; at least 44 CSS px.
     *
     * @param id Its id.
     * @param text Its label (and what the probe reads).
     * @param desc Size, primary and icon.
     */
    function actionButton(id: string, text: string, desc?: ActionButtonDesc): Response;
    /**
     * Queue a toast: it stacks at UiDesc.toastAnchor, enters, stays its time and leaves; at most three are up (older ones leave first); a press dismisses it. Callable anywhere (systems, commands, start); it is world state, so snapshots carry it.
     *
     * @param world The world whose UI shows it.
     * @param text The message: a few words.
     * @param desc Tone and time.
     * @returns Its id, for dismissToast.
     */
    function toast(world: World, text: string, desc?: ToastDesc): number;
    /**
     * Start a toast's exit.
     *
     * @param world The world.
     * @param toastId The id toast() answered.
     */
    function dismissToast(world: World, toastId: number): void;
    /**
     * Queue a navigation intent for the next tick: a gamepad og.Input does not read yet, a custom binding, a command.
     *
     * @param world The world.
     * @param intent The intent.
     */
    function navigate(world: World, intent: NavIntent): void;
    /**
     * Escape or gamepad B this tick, not taken by a dialog or menu: true once, then taken. A pause screen closes on it.
     */
    function back(): boolean;
    /**
     * Whether keyboard and gamepad navigation moves focus on this screen (on by default). A gameplay HUD whose keys move the player turns it off; an open dialog or menu turns it back on while it is open. Takes effect from the next tick.
     *
     * @param on On or off.
     */
    function setNavigation(on: boolean): void;
    /**
     * A number the UI keeps for an id under the open element (a template's tab, a count-up's start tick), in snapshots like the rest of the UI state. Read it every tick it is used: an unused memory is dropped.
     *
     * @param id The id, unique among its siblings' memories.
     * @param initial Its value the first time.
     */
    function memory(id: string, initial: number): number;
    /**
     * Set a number memory (see memory).
     *
     * @param id The id.
     * @param value The value.
     */
    function remember(id: string, value: number): void;
    /** Frames the UI has built: its clock, for animations by the tick. */
    function tick(): number;
    /**
     * Lay the UI out for another play surface (a phone in portrait at 3x, a notch) until cleared with null: for headless previews and playtests of responsive layouts. Render at width x dpr by height x dpr (`og run --size`).
     *
     * @param world The world.
     * @param viewport The surface to lay out for; null returns to the host's.
     */
    function previewViewport(world: World, viewport: Viewport | null): void;
    /**
     * The title screen template. Returns the item chosen this tick.
     *
     * @param id Its id.
     * @param desc Title, items and footer.
     */
    function mainMenu(id: string, desc?: MainMenuDesc): MenuResult;
    /**
     * The pause screen template: a dialog with Resume first; Back answers `resume`.
     *
     * @param id Its id.
     * @param desc Open and items.
     */
    function pauseMenu(id: string, desc?: PauseMenuDesc): MenuResult;
    /**
     * The settings template: Audio (volumes, mute), Graphics (quality, frame rate, reduced motion, contrast), Controls (sensitivity, invert, vibration, bindings) and Language. Pass the settings in every tick and keep what it answers.
     *
     * @param id Its id.
     * @param desc Open, the settings, languages and hooks.
     */
    function settingsScreen(id: string, desc?: SettingsDesc): SettingsResult;
    /**
     * The platformer HUD template.
     *
     * @param id Its id.
     * @param desc Lives, coins, score, level and timer.
     */
    function platformerHud(id: string, desc?: PlatformerHudDesc): HudResult;
    /**
     * The shooter HUD template.
     *
     * @param id Its id.
     * @param desc Health, armor, ammo, abilities and map.
     */
    function shooterHud(id: string, desc?: ShooterHudDesc): HudResult;
    /**
     * The puzzle HUD template.
     *
     * @param id Its id.
     * @param desc Level, score, target, stars, moves and timer.
     */
    function puzzleHud(id: string, desc?: PuzzleHudDesc): HudResult;
    /**
     * The idle and incremental HUD template: answers `tap` and `buy:<id>` actions.
     *
     * @param id Its id.
     * @param desc Currencies and generators.
     */
    function idleHud(id: string, desc?: IdleHudDesc): HudResult;
    /**
     * The racing HUD template.
     *
     * @param id Its id.
     * @param desc Speed, gear, laps, place, times and boost.
     */
    function racingHud(id: string, desc?: RacingHudDesc): HudResult;
    /**
     * The dialogue and cutscene template: letterbox, the dialogue box, Skip.
     *
     * @param id Its id.
     * @param desc Lines and the current index.
     */
    function cutscene(id: string, desc?: CutsceneDesc): CutsceneResult;
    /**
     * The shop template.
     *
     * @param id Its id.
     * @param desc Balance, items and categories.
     */
    function shopScreen(id: string, desc?: ShopDesc): ShopResult;
    /**
     * The inventory template: a grid of slots with drag and drop.
     *
     * @param id Its id.
     * @param desc Slots and items.
     */
    function inventoryScreen(id: string, desc?: InventoryDesc): InventoryResult;
    /**
     * The level select template.
     *
     * @param id Its id.
     * @param desc The levels.
     */
    function levelSelect(id: string, desc?: LevelSelectDesc): LevelSelectResult;
    /**
     * The results template: stars, the score counting up, the best, stats, actions and Share.
     *
     * @param id Its id.
     * @param desc Score, stars, stats and actions.
     */
    function resultsScreen(id: string, desc?: ResultsDesc): MenuResult;
    /**
     * The leaderboard template.
     *
     * @param id Its id.
     * @param desc Scopes, rows and states.
     */
    function leaderboardScreen(id: string, desc?: LeaderboardDesc): LeaderboardResult;
    /**
     * The onboarding template: a spotlight and a callout per step.
     *
     * @param id Its id.
     * @param desc Steps and the current index.
     */
    function tutorialOverlay(id: string, desc?: TutorialDesc): TutorialResult;
    /**
     * The loading template.
     *
     * @param id Its id.
     * @param desc Title, progress and tips.
     */
    function loadingScreen(id: string, desc?: LoadingDesc): void;
    /**
     * The confirm dialog template.
     *
     * @param id Its id.
     * @param desc Question, consequence and labels.
     */
    function confirmDialog(id: string, desc?: ConfirmDesc): ConfirmResult;
    /**
     * The alert template. Returns true the tick it is dismissed.
     *
     * @param id Its id.
     * @param desc Title, message and label.
     */
    function alertDialog(id: string, desc?: AlertDesc): boolean;
  }
  /**
   * Systems the engine ships as TypeScript: game-level code, listed explicitly in GameDesc.systems like the game's own, so the schedule (and dump_world) shows them and nothing runs that the game did not ask for.
   */
  namespace systems {
    /**
     * The camera controllers, in this order: og.orbitCamera (og.OrbitCamera), og.followCamera (og.FollowCamera) and og.follow2D (og.Follow2D). Each moves og.Camera only while an entity carries its component. List them before the game's systems that read og.Camera: `systems: [...og.systems.cameras, ...yours]`.
     */
    const cameras: readonly System[];
  }
  /**
   * The command channel: actions scheduled on simulation time, from `og run --commands "t:action[:args];..."` or the probe's commands().
   * Core actions ship with the engine; a module or a game registers more. Unknown actions warn and continue. Not in publish builds.
   */
  namespace commands {
    /**
     * Register an action into the command channel. GameDesc.commands does this for a game's own actions.
     *
     * @param name The action name, snake_case, such as `spawn_wave`.
     * @param handler What it does.
     * @param desc Its usage line and doc.
     * @returns Refused for a name a core action or another registration already uses.
     */
    function register(name: string, handler: CommandHandler, desc?: CommandDesc): Result<void>;
    /**
     * Parse a script without scheduling it.
     *
     * @param script `t:action[:args];...` with t in simulation seconds.
     * @returns The steps, or the first malformed step.
     */
    function parse(script: string): Result<readonly CommandStep[]>;
    /**
     * Every action the channel knows: core actions in CoreAction order, then registered ones by name.
     *
     * @param world The world whose channel is asked.
     */
    function help(world: World): readonly CommandInfo[];
    /**
     * Schedule a script on simulation time, from the current tick.
     *
     * @param world The world.
     * @param script `t:action[:args];...` with t in simulation seconds.
     * @returns The report, or the first malformed step.
     */
    function schedule(world: World, script: string): Result<CommandsReport>;
  }
}
