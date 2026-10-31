<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `world` module

The world: an archetype ECS. Entities are generational handles; components are columns in engine memory.

Structural changes (spawn, despawn, insert, remove) invalidate typed views; the API refuses a stale view.

- **TypeScript**: functions are none at module level (its members live on the types below); components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Skill**: `og-core`, which teaches the module (how much of it to use, and how to prove it).
- **Index**: [every module and name](index.md).

```ogt
module world {
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 785</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`Entity`](#type-Entity) | handle | An entity: an index and a generation. The value is index + generation * 2^32, with the generation below 2^21, so it stays an exact JavaScript number. |
| [`Entity()`](#column-Entity) | column kind | An Entity field: (index, generation) pairs. |
| [`SpawnDesc`](#type-SpawnDesc) | desc struct | How to spawn an entity. `{}` spawns an unnamed entity at the origin with a Transform. |
| [`RestoreDesc`](#type-RestoreDesc) | desc struct | How world.restore takes a save. |
| [`og.WorldRestored`](#record-WorldRestored) | event | A world.restore applied: the tick after it (the first whole tick after a save taken mid-tick, whose next step completes the saved tick) delivers this, read through world.events like any event. Everything in the world came back as saved, request numbers and resources the session filled included; the session did not (og-game §17), so a system that reads this resets what belongs to the session: it forgets requests answered long ago, asks the store again for the settings and progress, and closes a menu the player had open. Sent as that tick starts, not when the restore applies, so the world between the two hashes as the save did. |
| [`QueryDesc`](#type-QueryDesc) | desc struct | Which entities a query matches. |
| [`World`](#type-World) | interface | The world a system, a command or a game callback is handed. Game code never holds a world beyond the call it was passed to. |
| [`Query`](#type-Query) | interface | A set of matched archetypes. Iterate archetypes, never entities one by one. |
| [`ArchetypeView`](#type-ArchetypeView) | interface | One archetype's columns for this tick. Views are invalidated by structural changes; using a stale view throws with the tick and the system it belonged to. |

## Components, resources and events

<a id="record-WorldRestored"></a>

### `og.WorldRestored` (event)

A world.restore applied: the tick after it (the first whole tick after a save taken mid-tick, whose next step completes the saved tick) delivers this, read through world.events like any event. Everything in the world came back as saved, request numbers and resources the session filled included; the session did not (og-game §17), so a system that reads this resets what belongs to the session: it forgets requests answered long ago, asks the store again for the settings and progress, and closes a menu the player had open. Sent as that tick starts, not when the restore applies, so the world between the two hashes as the save did.

Token `og.WorldRestored`, registered under the schema name `og.WorldRestored`; its value type is `WorldRestored`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `from` | `number` | `0` | — | `ticks` | — | The tick the world was at when the restore applied. |
| `tick` | `number` | `0` | — | `ticks` | — | The saved tick: og.Time's tick right after the restore. |

```ogt
event WorldRestored {
  doc "A world.restore applied: the tick after it (the first whole tick after a save taken mid-tick, whose next step completes the saved tick) delivers this, read through world.events like any event. Everything in the world came back as saved, request numbers and resources the session filled included; the session did not (og-game §17), so a system that reads this resets what belongs to the session: it forgets requests answered long ago, asks the store again for the settings and progress, and closes a menu the player had open. Sent as that tick starts, not when the restore applies, so the world between the two hashes as the save did."
  field from u32 "The tick the world was at when the restore applied." {
    unit ticks
  }
  field tick u32 "The saved tick: og.Time's tick right after the restore." {
    unit ticks
  }
}
```

<sub>`api.ogt` line 823</sub>

## Interfaces

<a id="type-World"></a>

### `World` (interface)

The world a system, a command or a game callback is handed. Game code never holds a world beyond the call it was passed to.

```ogt
interface World {
  doc "The world a system, a command or a game callback is handed. Game code never holds a world beyond the call it was passed to."
  # members follow, each quoted below
```

<sub>`api.ogt` line 840</sub>

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

<sub>`api.ogt` line 843</sub>

<a id="member-World-spawn"></a>

#### `World.spawn`

Spawn an entity.

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
  param desc SpawnDesc "Name, parent and initial transform."
  returns Result.Entity "The new entity, or why it was refused (a duplicate name, a dead parent)."
}
```

<sub>`api.ogt` line 845</sub>

<a id="member-World-despawn"></a>

#### `World.despawn`

Despawn an entity and its descendants.

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
  param entity Entity "The entity."
  returns Result.Void "Refused for a dead entity."
}
```

<sub>`api.ogt` line 851</sub>

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

<sub>`api.ogt` line 857</sub>

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

<sub>`api.ogt` line 863</sub>

<a id="member-World-insert"></a>

#### `World.insert`

Add a component to an entity, or overwrite it when present. A structural change.

```ts
World.insert<C extends AnyComponent>(entity: Entity, component: C, value?: Partial<Value<C>>): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `entity` | `Entity` | no | The entity. |
| `component` | `C` | no | The component token. |
| `value` | `Partial<Value<C>>` | yes | Field values; omitted fields take their defaults. |

**Returns** `Result<void>`: Refused for a dead entity or an unregistered component.

Binding symbol `og_world_insert` · publish builds: kept

```ogt
fn insert {
  doc "Add a component to an entity, or overwrite it when present. A structural change."
  generic C AnyComponent
  param entity Entity "The entity."
  param component C "The component token."
  param value Partial.Value.C "Field values; omitted fields take their defaults." {
    optional true
  }
  returns Result.Void "Refused for a dead entity or an unregistered component."
}
```

<sub>`api.ogt` line 871</sub>

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

<sub>`api.ogt` line 882</sub>

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

<sub>`api.ogt` line 890</sub>

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

<sub>`api.ogt` line 898</sub>

<a id="member-World-resource"></a>

#### `World.resource`

A resource's live value: the same handle in a world every time, its fields typed reads and writes of the resource's packed record (no JSON, whatever its size). The record is fetched again only by the first read after another engine call or a system boundary, and writes reach the engine in one call before the next engine call and when the system or command returns, so two handles agree and a read after an engine call sees the engine's value (og-core §4). Inside a system, the resource must be among the system's reads or writes (writes to set a field): an undeclared one throws in a dev or release build, naming the system and the resource, and fails the tick; a publish build refuses it and logs it once per system in the engine log.

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
  doc "A resource's live value: the same handle in a world every time, its fields typed reads and writes of the resource's packed record (no JSON, whatever its size). The record is fetched again only by the first read after another engine call or a system boundary, and writes reach the engine in one call before the next engine call and when the system or command returns, so two handles agree and a read after an engine call sees the engine's value (og-core §4). Inside a system, the resource must be among the system's reads or writes (writes to set a field): an undeclared one throws in a dev or release build, naming the system and the resource, and fails the tick; a publish build refuses it and logs it once per system in the engine log."
  generic R AnyResource
  param resource R "The resource token."
  returns Result.Value.R "Refused for an unregistered resource (or, in a publish build, one the running system did not declare)."
}
```

<sub>`api.ogt` line 907</sub>

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

<sub>`api.ogt` line 914</sub>

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

<sub>`api.ogt` line 920</sub>

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

<sub>`api.ogt` line 926</sub>

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

<sub>`api.ogt` line 932</sub>

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

<sub>`api.ogt` line 939</sub>

<a id="member-World-instantiate"></a>

#### `World.instantiate`

Spawn a loaded scene or glTF asset into the world: a root entity from the desc, then (for a scene) every scene entity under it with its components and names, or (for a glTF) a Model on the root.

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
  doc "Spawn a loaded scene or glTF asset into the world: a root entity from the desc, then (for a scene) every scene entity under it with its components and names, or (for a glTF) a Model on the root."
  doc "Scene entity names must be free: a name already in use refuses the whole instantiation before anything spawns."
  param asset Asset "A ready asset from og.load."
  param desc SpawnDesc "Name, parent and transform of the root entity."
  returns Result.Entity "The root entity, or why it was refused (an asset still loading, a failed asset)."
}
```

<sub>`api.ogt` line 947</sub>

<a id="member-World-snapshot"></a>

#### `World.snapshot`

The whole world as text, to save a game's progress in the platform store: every entity with its components and names, every resource (og.Time's clock and the og.Rng streams too), the pending events and the modules' state (physics, animation, flipbooks, tweens, tilemap and terrain edits), each type with its schema hash, deflated and base64-encoded after `og-save:1:`. Every build has it, publish included, and the same world saves to the same text on both hosts. Save it with og.platform.saveSlot (`{ slot: "slot1", text, meta: { name, chimes } }`), which spreads it over as many store keys as it needs (a key holds 64 KiB) and writes the slot's manifest last, then og.platform.loadSlot and world.restore of the answer's `text` (examples/hollowbell, a 2D platformer of 189 entities on a 96 x 48 tile map, saves to about 25 KiB; examples/kestrel-stair, a 3D world of 93 entities with skinned characters, cloths and minds, to about 78 KB; a world of a few entities to about 11 KiB, most of it the table of every registered type). Inside a system it is the world in the middle of that tick, as that system sees it: what earlier systems wrote this tick is in it, a structural change the system queued (applied when it returns) is not, and it names the system, so a world restored from it completes that tick: its next step runs the systems after this one and the engine's (physics included) with the tick's own events, then ends the tick, and the world goes on tick for tick as the one that saved did (a replay from it matches; make the snapshot the system's last write of the tick, since what it does after the call is not in the save). Between ticks (GameDesc.start, a command, a UI callback) it is the world between two ticks. Physics carries its mesh, convex-hull and height-field colliders by their geometry's handle and scale, not their triangles (og.physics.setMesh). Asset and material handles in components are saved as their numbers: load what a saved entity uses in the same order on every launch (in GameDesc.start) before restoring.

```ts
World.snapshot(): Result<string>
```

**Returns** `Result<string>`: The save, or why not (the world could not be read).

Binding symbol `og_world_snapshot` · publish builds: kept

```ogt
fn snapshot {
  doc "The whole world as text, to save a game's progress in the platform store: every entity with its components and names, every resource (og.Time's clock and the og.Rng streams too), the pending events and the modules' state (physics, animation, flipbooks, tweens, tilemap and terrain edits), each type with its schema hash, deflated and base64-encoded after `og-save:1:`. Every build has it, publish included, and the same world saves to the same text on both hosts. Save it with og.platform.saveSlot (`{ slot: \"slot1\", text, meta: { name, chimes } }`), which spreads it over as many store keys as it needs (a key holds 64 KiB) and writes the slot's manifest last, then og.platform.loadSlot and world.restore of the answer's `text` (examples/hollowbell, a 2D platformer of 189 entities on a 96 x 48 tile map, saves to about 25 KiB; examples/kestrel-stair, a 3D world of 93 entities with skinned characters, cloths and minds, to about 78 KB; a world of a few entities to about 11 KiB, most of it the table of every registered type). Inside a system it is the world in the middle of that tick, as that system sees it: what earlier systems wrote this tick is in it, a structural change the system queued (applied when it returns) is not, and it names the system, so a world restored from it completes that tick: its next step runs the systems after this one and the engine's (physics included) with the tick's own events, then ends the tick, and the world goes on tick for tick as the one that saved did (a replay from it matches; make the snapshot the system's last write of the tick, since what it does after the call is not in the save). Between ticks (GameDesc.start, a command, a UI callback) it is the world between two ticks. Physics carries its mesh, convex-hull and height-field colliders by their geometry's handle and scale, not their triangles (og.physics.setMesh). Asset and material handles in components are saved as their numbers: load what a saved entity uses in the same order on every launch (in GameDesc.start) before restoring."
  returns Result.String "The save, or why not (the world could not be read)."
}
```

<sub>`api.ogt` line 955</sub>

<a id="member-World-restore"></a>

#### `World.restore`

Replace the world with a save that world.snapshot wrote: entities, components, names, resources, events and the modules' state, so the restored world hashes as the saved one did, on either host. The session stays: input (og.Input, og.Pointers, og.Gamepad, og.PointerLock), the player's bus volumes (og.Audio), the UI's focus and hover, and what lives outside the world (the platform's calls and clock, the language, the loaded assets, the command channel's scheduled steps, which keep their remaining delays though og.Time takes the saved tick, so an `og run --commands` script goes on to its last step or `quit` whether the save is of a later tick or an earlier one). Called in a system or a command it applies when the tick ends, after the last system; elsewhere (GameDesc.start, between ticks) at once. Either way the tick after it delivers og.WorldRestored, for the systems that reset what lives in the session. A save a system took mid-tick restores to that moment of its tick (og.Time reads its tick): the next step completes it (the systems after the saver, then the engine's), so the ticks after it are the saved run's. The meshes and height fields its colliders name must be loaded or registered first (a save naming one that is not is refused). A save holds only the world: progress kept in module variables is not in it, so keep it in components and resources.

```ts
World.restore(save: string, desc?: RestoreDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `save` | `string` | no | The text world.snapshot returned. |
| `desc` | `RestoreDesc` | yes | Whether a type whose layout changed since the save is mapped field by field. |

**Returns** `Result<void>`: Refused, with nothing changed, for text world.snapshot did not write, a save from a newer engine, or a type whose layout changed since the save (a field added, removed or retyped, an enum's variants): the reason names the type and the fields, and the fix (`{ migrate: true }`, or keeping a saved type's fields stable between releases).

Binding symbol `og_world_restore` · publish builds: kept

```ogt
fn restore {
  doc "Replace the world with a save that world.snapshot wrote: entities, components, names, resources, events and the modules' state, so the restored world hashes as the saved one did, on either host. The session stays: input (og.Input, og.Pointers, og.Gamepad, og.PointerLock), the player's bus volumes (og.Audio), the UI's focus and hover, and what lives outside the world (the platform's calls and clock, the language, the loaded assets, the command channel's scheduled steps, which keep their remaining delays though og.Time takes the saved tick, so an `og run --commands` script goes on to its last step or `quit` whether the save is of a later tick or an earlier one). Called in a system or a command it applies when the tick ends, after the last system; elsewhere (GameDesc.start, between ticks) at once. Either way the tick after it delivers og.WorldRestored, for the systems that reset what lives in the session. A save a system took mid-tick restores to that moment of its tick (og.Time reads its tick): the next step completes it (the systems after the saver, then the engine's), so the ticks after it are the saved run's. The meshes and height fields its colliders name must be loaded or registered first (a save naming one that is not is refused). A save holds only the world: progress kept in module variables is not in it, so keep it in components and resources."
  param save String "The text world.snapshot returned."
  param desc RestoreDesc "Whether a type whose layout changed since the save is mapped field by field."
  returns Result.Void "Refused, with nothing changed, for text world.snapshot did not write, a save from a newer engine, or a type whose layout changed since the save (a field added, removed or retyped, an enum's variants): the reason names the type and the fields, and the fix (`{ migrate: true }`, or keeping a saved type's fields stable between releases)."
}
```

<sub>`api.ogt` line 960</sub>

<a id="type-Query"></a>

### `Query` (interface)

A set of matched archetypes. Iterate archetypes, never entities one by one.

```ogt
interface Query {
  doc "A set of matched archetypes. Iterate archetypes, never entities one by one."
  # members follow, each quoted below
```

<sub>`api.ogt` line 968</sub>

<a id="member-Query-archetypes"></a>

#### `Query.archetypes`

The matched archetypes this tick.

```ts
Query.archetypes(): readonly ArchetypeView[]
```

**Returns** `readonly ArchetypeView[]`.

Binding symbol `og_query_archetypes` · publish builds: kept

```ogt
fn archetypes {
  doc "The matched archetypes this tick."
  returns List.ArchetypeView
}
```

<sub>`api.ogt` line 970</sub>

<a id="member-Query-count"></a>

#### `Query.count`

The number of matched entities.

```ts
Query.count(): number
```

**Returns** `number`.

Binding symbol `og_query_count` · publish builds: kept

```ogt
fn count {
  doc "The number of matched entities."
  returns u32
}
```

<sub>`api.ogt` line 974</sub>

<a id="type-ArchetypeView"></a>

### `ArchetypeView` (interface)

One archetype's columns for this tick. Views are invalidated by structural changes; using a stale view throws with the tick and the system it belonged to.

```ogt
interface ArchetypeView {
  doc "One archetype's columns for this tick. Views are invalidated by structural changes; using a stale view throws with the tick and the system it belonged to."
  # members follow, each quoted below
```

<sub>`api.ogt` line 980</sub>

<a id="member-ArchetypeView-count"></a>

#### `ArchetypeView.count` (property)

The number of entities in the archetype.

```ts
readonly count: number
```

Binding symbol `og_archetype_view_get_count` · publish builds: kept

```ogt
prop count u32 "The number of entities in the archetype."
```

<sub>`api.ogt` line 982</sub>

<a id="member-ArchetypeView-tick"></a>

#### `ArchetypeView.tick` (property)

The tick this view belongs to.

```ts
readonly tick: number
```

Binding symbol `og_archetype_view_get_tick` · publish builds: kept

```ogt
prop tick u32 "The tick this view belongs to."
```

<sub>`api.ogt` line 983</sub>

<a id="member-ArchetypeView-entities"></a>

#### `ArchetypeView.entities` (property)

The entities as (index, generation) pairs: entity i is at 2i and 2i + 1.

```ts
readonly entities: Uint32Array
```

Binding symbol `og_archetype_view_get_entities` · publish builds: kept

```ogt
prop entities Uint32Array "The entities as (index, generation) pairs: entity i is at 2i and 2i + 1."
```

<sub>`api.ogt` line 984</sub>

<a id="member-ArchetypeView-entity"></a>

#### `ArchetypeView.entity`

The handle of entity i.

```ts
ArchetypeView.entity(index: number): Entity
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `index` | `number` | no | The row, from 0 to count - 1. |

**Returns** `Entity`.

Binding symbol `og_archetype_view_entity` · publish builds: kept

```ogt
fn entity {
  doc "The handle of entity i."
  param index u32 "The row, from 0 to count - 1."
  returns Entity
}
```

<sub>`api.ogt` line 985</sub>

<a id="member-ArchetypeView-has"></a>

#### `ArchetypeView.has`

Whether the archetype carries a component.

```ts
ArchetypeView.has<C extends AnyComponent>(component: C): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `component` | `C` | no | The component token. |

**Returns** `boolean`.

Binding symbol `og_archetype_view_has` · publish builds: kept

```ogt
fn has {
  doc "Whether the archetype carries a component."
  generic C AnyComponent
  param component C "The component token."
  returns bool
}
```

<sub>`api.ogt` line 990</sub>

<a id="member-ArchetypeView-column"></a>

#### `ArchetypeView.column`

The typed views of a component's fields: one typed array per field, `count * stride` long.

Writing requires the component among the system's writes; reading among its reads or writes.

```ts
ArchetypeView.column<C extends AnyComponent>(component: C): View<C>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `component` | `C` | no | The component token. |

**Returns** `View<C>`.

Binding symbol `og_archetype_view_column` · publish builds: kept

```ogt
fn column {
  doc "The typed views of a component's fields: one typed array per field, `count * stride` long."
  doc "Writing requires the component among the system's writes; reading among its reads or writes."
  generic C AnyComponent
  param component C "The component token."
  returns View.C
}
```

<sub>`api.ogt` line 996</sub>

## Types

<a id="type-Entity"></a>

### `Entity` (handle)

An entity: an index and a generation. The value is index + generation * 2^32, with the generation below 2^21, so it stays an exact JavaScript number.

Generations start at 1, so 0 is never a live entity.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle Entity {
  doc "An entity: an index and a generation. The value is index + generation * 2^32, with the generation below 2^21, so it stays an exact JavaScript number."
  doc "Generations start at 1, so 0 is never a live entity."
}
```

<sub>`api.ogt` line 790</sub>

<a id="type-SpawnDesc"></a>

### `SpawnDesc` (desc struct)

How to spawn an entity. `{}` spawns an unnamed entity at the origin with a Transform.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | A unique name; the probe and the command channel address the entity by it. Empty leaves it unnamed. |
| `parent` | `Entity \| null` | `null` | — | — | — | The parent entity; the new entity's Transform is then relative to it. |
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The initial Transform.position. |
| `quaternion` | `Quaternion` | `{ x: 0, y: 0, z: 0, w: 1 }` | — | — | — | The initial Transform.quaternion. |
| `scale` | `Vector3` | `{ x: 1, y: 1, z: 1 }` | — | — | — | The initial Transform.scale. |
| `bare` | `boolean` | `false` | — | — | — | Spawn without a Transform: a pure data entity. |

```ogt
struct SpawnDesc {
  doc "How to spawn an entity. `{}` spawns an unnamed entity at the origin with a Transform."
  desc true
  field name String "A unique name; the probe and the command channel address the entity by it. Empty leaves it unnamed."
  field parent Option.Entity "The parent entity; the new entity's Transform is then relative to it."
  field position Vector3 "The initial Transform.position." {
    unit m
  }
  field quaternion Quaternion "The initial Transform.quaternion."
  field scale Vector3 "The initial Transform.scale." {
    default 1 1 1
  }
  field bare bool "Spawn without a Transform: a pure data entity."
}
```

<sub>`api.ogt` line 802</sub>

<a id="type-RestoreDesc"></a>

### `RestoreDesc` (desc struct)

How world.restore takes a save.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `migrate` | `boolean` | `false` | — | — | — | Map a type whose layout changed since the save field by field, by name, as a hot reload does: a field added since takes its default, a removed one is dropped, an enum value is mapped by its variant's name, a field of another type starts at its default, and a type this build no longer registers is dropped. Off, such a save is refused naming the type and its fields. |

```ogt
struct RestoreDesc {
  doc "How world.restore takes a save."
  desc true
  field migrate bool "Map a type whose layout changed since the save field by field, by name, as a hot reload does: a field added since takes its default, a removed one is dropped, an enum value is mapped by its variant's name, a field of another type starts at its default, and a type this build no longer registers is dropped. Off, such a save is refused naming the type and its fields."
}
```

<sub>`api.ogt` line 817</sub>

<a id="type-QueryDesc"></a>

### `QueryDesc` (desc struct)

Which entities a query matches.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `with` | `readonly AnyComponent[]` | `[]` | — | — | — | Components an entity must have. |
| `without` | `readonly AnyComponent[]` | `[]` | — | — | — | Components an entity must not have. |

```ogt
struct QueryDesc {
  doc "Which entities a query matches."
  desc true
  field with List.AnyComponent "Components an entity must have."
  field without List.AnyComponent "Components an entity must not have."
}
```

<sub>`api.ogt` line 833</sub>

## Column kinds

Each is a factory, `Entity(value?, options?)`, that makes one field of a component or resource layout.

| Column | Value | Typed array | Stride | Doc |
|---|---|---|---|---|
| <a id="column-Entity"></a>`Entity` | `Entity` | `Uint32Array` | 2 | An Entity field: (index, generation) pairs. |

```ogt
column Entity {
  doc "An Entity field: (index, generation) pairs."
  array Uint32Array
  stride 2
  value Entity
}
```

<sub>`api.ogt` line 795</sub>
