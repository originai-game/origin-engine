<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`world`](../world.md) · [index](../index.md)

<a id="type-World"></a>

### `World` (interface)

The world a system, a command or a game callback is handed. Game code never holds a world beyond the call it was passed to.

```ogt
interface World {
  doc "The world a system, a command or a game callback is handed. Game code never holds a world beyond the call it was passed to."
  # members follow, each quoted below
```

<sub>Module `world` line 244</sub>

<a id="member-World-tick"></a>

#### `World.tick` (property)

Fixed steps taken since the world was created.

```ts
readonly tick: number
```

Binding symbol `og_world_get_tick` · publish builds: kept

```ogt
prop tick u32 "Fixed steps taken since the world was created."
```

<sub>Module `world` line 247</sub>

<a id="member-World-spawn"></a>

#### `World.spawn`

Spawn an entity.

Also known as: instantiate, create entity, spawn object, Unity Instantiate, Godot add_child

```ts
World.spawn(desc?: SpawnDesc): Result<Entity>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `desc` | `SpawnDesc` | yes | Name, parent and initial transform. |

**Returns** `Result<Entity>`: The new entity, or why it was refused (a duplicate name, a dead parent).

Binding symbol `og_world_spawn` · publish builds: kept

```ogt
fn spawn {
  doc "Spawn an entity."
  aliases "instantiate, create entity, spawn object, Unity Instantiate, Godot add_child"
  param desc SpawnDesc "Name, parent and initial transform."
  returns Result.Entity "The new entity, or why it was refused (a duplicate name, a dead parent)."
}
```

<sub>Module `world` line 249</sub>

<a id="member-World-despawn"></a>

#### `World.despawn`

Despawn an entity and its descendants.

Also known as: destroy entity, delete object, Unity Destroy, Godot queue_free

```ts
World.despawn(entity: Entity): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `entity` | `Entity` | no | The entity. |

**Returns** `Result<void>`: Refused for a dead entity.

Binding symbol `og_world_despawn` · publish builds: kept

```ogt
fn despawn {
  doc "Despawn an entity and its descendants."
  aliases "destroy entity, delete object, Unity Destroy, Godot queue_free"
  param entity Entity "The entity."
  returns Result.Void "Refused for a dead entity."
}
```

<sub>Module `world` line 256</sub>

<a id="member-World-alive"></a>

#### `World.alive`

Whether an entity handle is live.

```ts
World.alive(entity: Entity): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `entity` | `Entity` | no | The entity. |

**Returns** `boolean`.

Binding symbol `og_world_alive` · publish builds: kept

```ogt
fn alive {
  doc "Whether an entity handle is live."
  param entity Entity "The entity."
  returns bool
}
```

<sub>Module `world` line 263</sub>

<a id="member-World-has"></a>

#### `World.has`

Whether an entity has a component.

```ts
World.has<C extends AnyComponent>(entity: Entity, component: C): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `entity` | `Entity` | no | The entity. |
| `component` | `C` | no | The component token. |

**Returns** `boolean`.

Binding symbol `og_world_has` · publish builds: kept

```ogt
fn has {
  doc "Whether an entity has a component."
  generic C AnyComponent
  param entity Entity "The entity."
  param component C "The component token."
  returns bool
}
```

<sub>Module `world` line 269</sub>

<a id="member-World-insert"></a>

#### `World.insert`

Add a component to an entity, or overwrite it when present. A structural change. Every field is checked against its reflected min/max before the component is changed; an out-of-range value returns an error, never a clamped value.

```ts
World.insert<C extends AnyComponent>(entity: Entity, component: C, value?: Partial<Value<C>>): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `entity` | `Entity` | no | The entity. |
| `component` | `C` | no | The component token. |
| `value` | `Partial<Value<C>>` | yes | Field values; omitted fields take their defaults. |

**Returns** `Result<void>`: Refused for a dead entity, an unregistered component or a field outside its reflected range.

Binding symbol `og_world_insert` · publish builds: kept

```ogt
fn insert {
  doc "Add a component to an entity, or overwrite it when present. A structural change. Every field is checked against its reflected min/max before the component is changed; an out-of-range value returns an error, never a clamped value."
  generic C AnyComponent
  param entity Entity "The entity."
  param component C "The component token."
  param value Partial.Value.C "Field values; omitted fields take their defaults." {
    optional true
  }
  returns Result.Void "Refused for a dead entity, an unregistered component or a field outside its reflected range."
}
```

<sub>Module `world` line 277</sub>

<a id="member-World-remove"></a>

#### `World.remove`

Remove a component from an entity. A structural change.

```ts
World.remove<C extends AnyComponent>(entity: Entity, component: C): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `entity` | `Entity` | no | The entity. |
| `component` | `C` | no | The component token. |

**Returns** `Result<void>`: Refused for a dead entity or a component it does not have.

Binding symbol `og_world_remove` · publish builds: kept

```ogt
fn remove {
  doc "Remove a component from an entity. A structural change."
  generic C AnyComponent
  param entity Entity "The entity."
  param component C "The component token."
  returns Result.Void "Refused for a dead entity or a component it does not have."
}
```

<sub>Module `world` line 288</sub>

<a id="member-World-get"></a>

#### `World.get`

Copy one entity's component value. For bulk work use a system's typed views.

```ts
World.get<C extends AnyComponent>(entity: Entity, component: C): Result<Value<C>>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `entity` | `Entity` | no | The entity. |
| `component` | `C` | no | The component token. |

**Returns** `Result<Value<C>>`.

Binding symbol `og_world_get` · publish builds: kept

```ogt
fn get {
  doc "Copy one entity's component value. For bulk work use a system's typed views."
  generic C AnyComponent
  param entity Entity "The entity."
  param component C "The component token."
  returns Result.Value.C
}
```

<sub>Module `world` line 296</sub>

<a id="member-World-set"></a>

#### `World.set`

Write some fields of one entity's component.

```ts
World.set<C extends AnyComponent>(entity: Entity, component: C, value: Partial<Value<C>>): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `entity` | `Entity` | no | The entity. |
| `component` | `C` | no | The component token. |
| `value` | `Partial<Value<C>>` | no | The fields to write. |

**Returns** `Result<void>`: Refused for a dead entity or a component it does not have.

Binding symbol `og_world_set` · publish builds: kept

```ogt
fn set {
  doc "Write some fields of one entity's component."
  generic C AnyComponent
  param entity Entity "The entity."
  param component C "The component token."
  param value Partial.Value.C "The fields to write."
  returns Result.Void "Refused for a dead entity or a component it does not have."
}
```

<sub>Module `world` line 304</sub>

<a id="member-World-resource"></a>

#### `World.resource`

A resource's live value: the same handle every time, its fields typed reads and writes of the resource's packed record (no JSON; ease strings in the host timing codec). A read after an engine call or a system boundary sees the engine's value; writes reach the engine before the next engine call and when the system or command returns. Inside a system the resource must be among its reads or writes (writes to set a field): undeclared, it throws in a dev or release build naming the system and resource and fails the tick; a publish build refuses it and logs once per system.

```ts
World.resource<R extends AnyResource>(resource: R): Result<Value<R>>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `resource` | `R` | no | The resource token. |

**Returns** `Result<Value<R>>`: Refused for an unregistered resource (or, in a publish build, one the running system did not declare).

Binding symbol `og_world_resource` · publish builds: kept

```ogt
fn resource {
  doc "A resource's live value: the same handle every time, its fields typed reads and writes of the resource's packed record (no JSON; ease strings in the host timing codec). A read after an engine call or a system boundary sees the engine's value; writes reach the engine before the next engine call and when the system or command returns. Inside a system the resource must be among its reads or writes (writes to set a field): undeclared, it throws in a dev or release build naming the system and resource and fails the tick; a publish build refuses it and logs once per system."
  generic R AnyResource
  param resource R "The resource token."
  returns Result.Value.R "Refused for an unregistered resource (or, in a publish build, one the running system did not declare)."
}
```

<sub>Module `world` line 313</sub>

<a id="member-World-children"></a>

#### `World.children`

The entities whose og.Parent is `entity`, in entity order (index, then generation). Any system may ask: it reads only the hierarchy.

```ts
World.children(entity: Entity): readonly Entity[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `entity` | `Entity` | no | The parent. |

**Returns** `readonly Entity[]`: Empty for a dead entity or one without children.

Binding symbol `og_world_children` · publish builds: kept

```ogt
fn children {
  doc "The entities whose og.Parent is `entity`, in entity order (index, then generation). Any system may ask: it reads only the hierarchy."
  param entity Entity "The parent."
  returns List.Entity "Empty for a dead entity or one without children."
}
```

<sub>Module `world` line 320</sub>

<a id="member-World-removed"></a>

#### `World.removed`

Entities that lost a component: removed from them, or despawned with it. Inside a system, since the system last ran (the component among its reads or writes); between ticks, since the last tick began. The world keeps a tick's removals through the next tick. The component must be tracked (a system filters on it: SystemDesc.changed, added or removed).

```ts
World.removed<C extends AnyComponent>(component: C): Result<readonly Entity[]>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `component` | `C` | no | The component token. |

**Returns** `Result<readonly Entity[]>`: In entity order; refused for a component no system tracks.

Binding symbol `og_world_removed` · publish builds: kept

```ogt
fn removed {
  doc "Entities that lost a component: removed from them, or despawned with it. Inside a system, since the system last ran (the component among its reads or writes); between ticks, since the last tick began. The world keeps a tick's removals through the next tick. The component must be tracked (a system filters on it: SystemDesc.changed, added or removed)."
  generic C AnyComponent
  param component C "The component token."
  returns Result.List.Entity "In entity order; refused for a component no system tracks."
}
```

<sub>Module `world` line 326</sub>

<a id="member-World-spatial"></a>

#### `World.spatial`

A spatial hash of the entities SpatialDesc selects, for radius and k-nearest queries on entities without colliders (og.physics answers those with). Inside a system, og.Transform must be among its reads or writes; the index holds the positions as the system sees them when it is built.

```ts
World.spatial(desc?: SpatialDesc): Result<SpatialIndex>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `desc` | `SpatialDesc` | yes | Which entities, and the cell size. |

**Returns** `Result<SpatialIndex>`.

Binding symbol `og_world_spatial` · publish builds: kept

```ogt
fn spatial {
  doc "A spatial hash of the entities SpatialDesc selects, for radius and k-nearest queries on entities without colliders (og.physics answers those with). Inside a system, og.Transform must be among its reads or writes; the index holds the positions as the system sees them when it is built."
  param desc SpatialDesc "Which entities, and the cell size." {
    optional true
  }
  returns Result.SpatialIndex
}
```

<sub>Module `world` line 333</sub>

<a id="member-World-state"></a>

#### `World.state`

A state machine's current value (GameDesc.states).

```ts
World.state(name: string): string | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | The state machine. |

**Returns** `string | null`: Null for a name GameDesc.states does not declare.

Binding symbol `og_world_state` · publish builds: kept

```ogt
fn state {
  doc "A state machine's current value (GameDesc.states)."
  param name String "The state machine."
  returns Option.String "Null for a name GameDesc.states does not declare."
}
```

<sub>Module `world` line 341</sub>

<a id="member-World-setState"></a>

#### `World.setState`

Ask a state machine to move to a value as the next tick begins: that tick runs the systems leaving the old value (SystemDesc.when exit), then those entering the new one (enter), then its systems, those bound to a value (during) only while the machine holds it. The last ask before the tick wins; asking for the value it holds cancels one asked for. Any system may ask.

```ts
World.setState(name: string, value: string): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | The state machine. |
| `value` | `string` | no | One of its values. |

**Returns** `Result<void>`: Refused for an unknown state machine or value.

Binding symbol `og_world_set_state` · publish builds: kept

```ogt
fn setState {
  doc "Ask a state machine to move to a value as the next tick begins: that tick runs the systems leaving the old value (SystemDesc.when exit), then those entering the new one (enter), then its systems, those bound to a value (during) only while the machine holds it. The last ask before the tick wins; asking for the value it holds cancels one asked for. Any system may ask."
  param name String "The state machine."
  param value String "One of its values."
  returns Result.Void "Refused for an unknown state machine or value."
}
```

<sub>Module `world` line 347</sub>

<a id="member-World-find"></a>

#### `World.find`

The entity with a name.

```ts
World.find(name: string): Entity | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | The name given at spawn. |

**Returns** `Entity | null`: Null when no live entity has the name.

Binding symbol `og_world_find` · publish builds: kept

```ogt
fn find {
  doc "The entity with a name."
  param name String "The name given at spawn."
  returns Option.Entity "Null when no live entity has the name."
}
```

<sub>Module `world` line 354</sub>

<a id="member-World-prefabOf"></a>

#### `World.prefabOf`

Where an entity came from when it was placed from a prefab: its og.PrefabLink (the prefab, its node, the instance's root).

```ts
World.prefabOf(entity: Entity): PrefabLink | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `entity` | `Entity` | no | The entity. |

**Returns** `PrefabLink | null`: Null for an entity not placed from a prefab, or a dead one.

Binding symbol `og_world_prefab_of` · publish builds: kept

```ogt
fn prefabOf {
  doc "Where an entity came from when it was placed from a prefab: its og.PrefabLink (the prefab, its node, the instance's root)."
  param entity Entity "The entity."
  returns Option.PrefabLink "Null for an entity not placed from a prefab, or a dead one."
}
```

<sub>Module `world` line 360</sub>

<a id="member-World-nameOf"></a>

#### `World.nameOf`

An entity's name.

```ts
World.nameOf(entity: Entity): string | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `entity` | `Entity` | no | The entity. |

**Returns** `string | null`: Null for an unnamed or dead entity.

Binding symbol `og_world_name_of` · publish builds: kept

```ogt
fn nameOf {
  doc "An entity's name."
  param entity Entity "The entity."
  returns Option.String "Null for an unnamed or dead entity."
}
```

<sub>Module `world` line 366</sub>

<a id="member-World-query"></a>

#### `World.query`

A query over the current archetypes, for commands and callbacks (GameDesc.start, a command, a UI callback). Inside a system it is refused, by design: a system receives its one query (every component of its reads and writes) as its first argument; loop over another archetype in a system of its own.

```ts
World.query(desc?: QueryDesc): Result<Query>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `desc` | `QueryDesc` | yes | The components to match. |

**Returns** `Result<Query>`.

Binding symbol `og_world_query` · publish builds: kept

```ogt
fn query {
  doc "A query over the current archetypes, for commands and callbacks (GameDesc.start, a command, a UI callback). Inside a system it is refused, by design: a system receives its one query (every component of its reads and writes) as its first argument; loop over another archetype in a system of its own."
  param desc QueryDesc "The components to match."
  returns Result.Query
}
```

<sub>Module `world` line 372</sub>

<a id="member-World-events"></a>

#### `World.events`

The events of one type emitted during the previous tick and this one, oldest first. Inside a system the event must be among its reads or writes (undeclared: thrown, or in a publish build refused and logged once).

```ts
World.events<E extends AnyEvent>(event: E): readonly Value<E>[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `event` | `E` | no | The event token. |

**Returns** `readonly Value<E>[]`.

Binding symbol `og_world_events` · publish builds: kept

```ogt
fn events {
  doc "The events of one type emitted during the previous tick and this one, oldest first. Inside a system the event must be among its reads or writes (undeclared: thrown, or in a publish build refused and logged once)."
  generic E AnyEvent
  param event E "The event token."
  returns List.Value.E
}
```

<sub>Module `world` line 378</sub>

<a id="member-World-emit"></a>

#### `World.emit`

Emit an event; readers see it this tick and the next. Inside a system the event must be among its writes (undeclared: thrown, or in a publish build refused and logged once).

```ts
World.emit<E extends AnyEvent>(event: E, payload: Value<E>): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `event` | `E` | no | The event token. |
| `payload` | `Value<E>` | no | The event's fields. |

**Returns** `Result<void>`.

Binding symbol `og_world_emit` · publish builds: kept

```ogt
fn emit {
  doc "Emit an event; readers see it this tick and the next. Inside a system the event must be among its writes (undeclared: thrown, or in a publish build refused and logged once)."
  generic E AnyEvent
  param event E "The event token."
  param payload Value.E "The event's fields."
  returns Result.Void
}
```

<sub>Module `world` line 385</sub>

<a id="member-World-instantiate"></a>

#### `World.instantiate`

Spawn a loaded scene, prefab or glTF asset into the world: a root entity from the desc, then (for a scene or prefab) every entity under it with its components (a scene's with their names, a prefab's unnamed, each with an og.PrefabLink to it), or (for a glTF) a Model on the root.

Scene entity names must be free: a name already in use refuses the whole instantiation before anything spawns.

```ts
World.instantiate(asset: Asset, desc?: SpawnDesc): Result<Entity>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `asset` | `Asset` | no | A ready asset from og.load. |
| `desc` | `SpawnDesc` | yes | Name, parent and transform of the root entity. |

**Returns** `Result<Entity>`: The root entity, or why it was refused (an asset still loading, a failed asset).

Binding symbol `og_world_instantiate` · publish builds: kept

```ogt
fn instantiate {
  doc "Spawn a loaded scene, prefab or glTF asset into the world: a root entity from the desc, then (for a scene or prefab) every entity under it with its components (a scene's with their names, a prefab's unnamed, each with an og.PrefabLink to it), or (for a glTF) a Model on the root."
  doc "Scene entity names must be free: a name already in use refuses the whole instantiation before anything spawns."
  param asset Asset "A ready asset from og.load."
  param desc SpawnDesc "Name, parent and transform of the root entity."
  returns Result.Entity "The root entity, or why it was refused (an asset still loading, a failed asset)."
}
```

<sub>Module `world` line 393</sub>

<a id="member-World-snapshot"></a>

#### `World.snapshot`

The whole world as text: every entity with its components and names, every resource (og.Time and og.Rng too), the pending events and the modules' state, each type with its schema hash, and GameDesc.saveVersion, after `og-save:3:`. Every build has it, publish too; the same world saves to the same text on both hosts. With SnapshotDesc's `with` or `resources`, a selection. Inside a system it is the world mid-tick as that system sees it, without the structural changes it queued, and a restore completes that tick: make it the system's last write. Between ticks it is the world between two ticks.

See `saves`.

Also known as: save game, savegame, checkpoint, serialise the world, persist state, quick save

```ts
World.snapshot(desc?: SnapshotDesc): Result<string>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `desc` | `SnapshotDesc` | yes | The whole world, or a selection: tagged entities and chosen resources. |

**Returns** `Result<string>`: The save, or why not (the world could not be read, or SnapshotDesc names a type this world does not register).

Binding symbol `og_world_snapshot` · publish builds: kept

```ogt
fn snapshot {
  doc "The whole world as text: every entity with its components and names, every resource (og.Time and og.Rng too), the pending events and the modules' state, each type with its schema hash, and GameDesc.saveVersion, after `og-save:3:`. Every build has it, publish too; the same world saves to the same text on both hosts. With SnapshotDesc's `with` or `resources`, a selection. Inside a system it is the world mid-tick as that system sees it, without the structural changes it queued, and a restore completes that tick: make it the system's last write. Between ticks it is the world between two ticks."
  guide "saves"
  aliases "save game, savegame, checkpoint, serialise the world, persist state, quick save"
  param desc SnapshotDesc "The whole world, or a selection: tagged entities and chosen resources." {
    optional true
  }
  returns Result.String "The save, or why not (the world could not be read, or SnapshotDesc names a type this world does not register)."
}
```

<sub>Module `world` line 401</sub>

<a id="member-World-openSave"></a>

#### `World.openSave`

Open a save for editing, as a migration does (SaveEdit): the engine's own migrations applied (an engine upgrade's changes to its components and its modules' state, such as og.ui's state of engine 0.7.7), the game's not yet. world.restore opens and migrates by itself; use this to read a save without restoring it, or to migrate one by hand.

```ts
World.openSave(save: string): Result<SaveEdit>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `save` | `string` | no | The text world.snapshot returned. |

**Returns** `Result<SaveEdit>`: Refused for text world.snapshot did not write, or a save of a newer GameDesc.saveVersion.

Binding symbol `og_world_open_save` · publish builds: kept

```ogt
fn openSave {
  doc "Open a save for editing, as a migration does (SaveEdit): the engine's own migrations applied (an engine upgrade's changes to its components and its modules' state, such as og.ui's state of engine 0.7.7), the game's not yet. world.restore opens and migrates by itself; use this to read a save without restoring it, or to migrate one by hand."
  param save String "The text world.snapshot returned."
  returns Result.SaveEdit "Refused for text world.snapshot did not write, or a save of a newer GameDesc.saveVersion."
}
```

<sub>Module `world` line 411</sub>

<a id="member-World-restore"></a>

#### `World.restore`

Replace the world with a world.snapshot save (a selection replaces only its tagged entities and resources). The engine's migrations, then GameDesc.migrations from the save's version to saveVersion, run on the save; a step that refuses or throws refuses the restore. It then hashes as the saved world did; the session (input, audio volumes, bindings, UI focus, platform, assets) stays. In a system or command it applies when the tick ends, elsewhere at once; the tick after delivers og.WorldRestored. A mid-tick save's next step completes its tick. The meshes its colliders name must be loaded first.

See `saves`.

```ts
World.restore(save: string, desc?: RestoreDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `save` | `string` | no | The text world.snapshot returned. |
| `desc` | `RestoreDesc` | yes | Whether a type whose layout changed since the save is mapped field by field. |

**Returns** `Result<void>`: Refused, with nothing changed, for text world.snapshot did not write, a save from a newer engine or saveVersion, a refused migration, an unloaded collider mesh, or a type whose layout changed that the save holds data of: the reason names the type, its fields and the fix.

Binding symbol `og_world_restore` · publish builds: kept

```ogt
fn restore {
  doc "Replace the world with a world.snapshot save (a selection replaces only its tagged entities and resources). The engine's migrations, then GameDesc.migrations from the save's version to saveVersion, run on the save; a step that refuses or throws refuses the restore. It then hashes as the saved world did; the session (input, audio volumes, bindings, UI focus, platform, assets) stays. In a system or command it applies when the tick ends, elsewhere at once; the tick after delivers og.WorldRestored. A mid-tick save's next step completes its tick. The meshes its colliders name must be loaded first."
  guide "saves"
  param save String "The text world.snapshot returned."
  param desc RestoreDesc "Whether a type whose layout changed since the save is mapped field by field."
  returns Result.Void "Refused, with nothing changed, for text world.snapshot did not write, a save from a newer engine or saveVersion, a refused migration, an unloaded collider mesh, or a type whose layout changed that the save holds data of: the reason names the type, its fields and the fix."
}
```

<sub>Module `world` line 417</sub>
