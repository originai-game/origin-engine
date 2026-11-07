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

<sub>Module `world` line 233</sub>

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

<sub>Module `world` line 236</sub>

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

<sub>Module `world` line 238</sub>

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

<sub>Module `world` line 245</sub>

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

<sub>Module `world` line 252</sub>

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

<sub>Module `world` line 258</sub>

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

<sub>Module `world` line 266</sub>

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

<sub>Module `world` line 277</sub>

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

<sub>Module `world` line 285</sub>

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

<sub>Module `world` line 293</sub>

<a id="member-World-resource"></a>

#### `World.resource`

A resource's live value: the same handle in a world every time, its fields typed reads and writes of the resource's packed record. Ease strings use the shared host timing codec; other fields use packed bytes without JSON. The packed record is fetched again only by the first read after another engine call or a system boundary, and pending writes reach the engine before the next engine call and when the system or command returns, so two handles agree and a read after an engine call sees the engine's value. Inside a system, the resource must be among the system's reads or writes (writes to set a field): an undeclared one throws in a dev or release build, naming the system and the resource, and fails the tick; a publish build refuses it and logs it once per system in the engine log.

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
  doc "A resource's live value: the same handle in a world every time, its fields typed reads and writes of the resource's packed record. Ease strings use the shared host timing codec; other fields use packed bytes without JSON. The packed record is fetched again only by the first read after another engine call or a system boundary, and pending writes reach the engine before the next engine call and when the system or command returns, so two handles agree and a read after an engine call sees the engine's value. Inside a system, the resource must be among the system's reads or writes (writes to set a field): an undeclared one throws in a dev or release build, naming the system and the resource, and fails the tick; a publish build refuses it and logs it once per system in the engine log."
  generic R AnyResource
  param resource R "The resource token."
  returns Result.Value.R "Refused for an unregistered resource (or, in a publish build, one the running system did not declare)."
}
```

<sub>Module `world` line 302</sub>

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

<sub>Module `world` line 309</sub>

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

<sub>Module `world` line 315</sub>

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

<sub>Module `world` line 322</sub>

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

<sub>Module `world` line 330</sub>

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

<sub>Module `world` line 336</sub>

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

<sub>Module `world` line 343</sub>

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

<sub>Module `world` line 349</sub>

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

<sub>Module `world` line 355</sub>

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

<sub>Module `world` line 361</sub>

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

<sub>Module `world` line 367</sub>

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

<sub>Module `world` line 374</sub>

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

<sub>Module `world` line 382</sub>

<a id="member-World-snapshot"></a>

#### `World.snapshot`

The whole world as text, to save a game's progress in the platform store: every entity with its components and names, every resource (og.Time's clock and the og.Rng streams too), the pending events and the modules' state (physics, animation, flipbooks, tweens, tilemap and terrain edits), each type with its schema hash, and the game's save version (GameDesc.saveVersion), deflated and base64-encoded after `og-save:2:`. Every build has it, publish included, and the same world saves to the same text on both hosts. Save it with og.platform.saveSlot (`{ slot: "slot1", text, meta: { name, chimes } }`), which spreads it over as many store keys as it needs (a key holds 64 KiB) and writes the slot's manifest last, then og.platform.loadSlot and world.restore of the answer's `text` (a 2D platformer of 189 entities on a 96 x 48 tile map saves to about 25 KiB; a 3D world of 93 entities with skinned characters, cloths and minds to about 78 KB; a world of a few entities to about 11 KiB, most of it the table of every registered type; 5,000 entities, half of them physics bodies, to about 436 KiB, written in about 95 ms and restored in about 60 ms natively). With SnapshotDesc's `with` or `resources`, only the tagged entities and the chosen resources (a selection). Inside a system it is the world in the middle of that tick, as that system sees it: what earlier systems wrote this tick is in it, a structural change the system queued (applied when it returns) is not, and it names the system, so a world restored from it completes that tick: its next step runs the systems after this one and the engine's (physics included) with the tick's own events, then ends the tick, and the world goes on tick for tick as the one that saved did (a replay from it matches; make the snapshot the system's last write of the tick, since what it does after the call is not in the save). Between ticks (GameDesc.start, a command, a UI callback) it is the world between two ticks. Physics carries its mesh, convex-hull and height-field colliders by their geometry's handle and scale, not their triangles (og.physics.setMesh). Asset and material handles in components are saved as their numbers: load what a saved entity uses in the same order on every launch (in GameDesc.start) before restoring.

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
  doc "The whole world as text, to save a game's progress in the platform store: every entity with its components and names, every resource (og.Time's clock and the og.Rng streams too), the pending events and the modules' state (physics, animation, flipbooks, tweens, tilemap and terrain edits), each type with its schema hash, and the game's save version (GameDesc.saveVersion), deflated and base64-encoded after `og-save:2:`. Every build has it, publish included, and the same world saves to the same text on both hosts. Save it with og.platform.saveSlot (`{ slot: \"slot1\", text, meta: { name, chimes } }`), which spreads it over as many store keys as it needs (a key holds 64 KiB) and writes the slot's manifest last, then og.platform.loadSlot and world.restore of the answer's `text` (a 2D platformer of 189 entities on a 96 x 48 tile map saves to about 25 KiB; a 3D world of 93 entities with skinned characters, cloths and minds to about 78 KB; a world of a few entities to about 11 KiB, most of it the table of every registered type; 5,000 entities, half of them physics bodies, to about 436 KiB, written in about 95 ms and restored in about 60 ms natively). With SnapshotDesc's `with` or `resources`, only the tagged entities and the chosen resources (a selection). Inside a system it is the world in the middle of that tick, as that system sees it: what earlier systems wrote this tick is in it, a structural change the system queued (applied when it returns) is not, and it names the system, so a world restored from it completes that tick: its next step runs the systems after this one and the engine's (physics included) with the tick's own events, then ends the tick, and the world goes on tick for tick as the one that saved did (a replay from it matches; make the snapshot the system's last write of the tick, since what it does after the call is not in the save). Between ticks (GameDesc.start, a command, a UI callback) it is the world between two ticks. Physics carries its mesh, convex-hull and height-field colliders by their geometry's handle and scale, not their triangles (og.physics.setMesh). Asset and material handles in components are saved as their numbers: load what a saved entity uses in the same order on every launch (in GameDesc.start) before restoring."
  aliases "save game, savegame, checkpoint, serialise the world, persist state, quick save"
  param desc SnapshotDesc "The whole world, or a selection: tagged entities and chosen resources." {
    optional true
  }
  returns Result.String "The save, or why not (the world could not be read, or SnapshotDesc names a type this world does not register)."
}
```

<sub>Module `world` line 390</sub>

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

<sub>Module `world` line 399</sub>

<a id="member-World-restore"></a>

#### `World.restore`

Replace the world with a save that world.snapshot wrote (a selection replaces only its tagged entities and its resources: SnapshotDesc). First the engine's own migrations upgrade what an older engine wrote of the engine's state, then GameDesc.migrations run from the save's version to GameDesc.saveVersion, in order, on the save (SaveEdit); a refused or throwing step refuses the restore with its reason and the world is left as it was. A save of a newer saveVersion is refused. Then: entities, components, names, resources, events and the modules' state, so the restored world hashes as the saved one did, on either host. The session stays: input (og.Input, og.Pointers, og.Gamepad, og.PointerLock), the player's bus volumes (og.Audio), the player's action bindings and dead zones (og.input), the UI's focus and hover, and what lives outside the world (the platform's calls and clock, the language, the loaded assets, the command channel's scheduled steps, which keep their remaining delays though og.Time takes the saved tick, so an `og run --commands` script goes on to its last step or `quit` whether the save is of a later tick or an earlier one). Called in a system or a command it applies when the tick ends, after the last system; elsewhere (GameDesc.start, between ticks) at once. Either way the tick after it delivers og.WorldRestored, for the systems that reset what lives in the session. A save a system took mid-tick restores to that moment of its tick (og.Time reads its tick): the next step completes it (the systems after the saver, then the engine's), so the ticks after it are the saved run's. The meshes and height fields its colliders name must be loaded or registered first (a save naming one that is not is refused). A save holds only the world: progress kept in module variables is not in it, so keep it in components and resources.

```ts
World.restore(save: string, desc?: RestoreDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `save` | `string` | no | The text world.snapshot returned. |
| `desc` | `RestoreDesc` | yes | Whether a type whose layout changed since the save is mapped field by field. |

**Returns** `Result<void>`: Refused, with nothing changed, for text world.snapshot did not write, a save from a newer engine or of a newer GameDesc.saveVersion, a migration that refused, or a type whose layout changed since the save (a field added, removed or retyped, an enum's variants) and that the save holds data of (a type it holds none of, no component, resource value or event, takes this build's layout; an engine og.* type that only gained fields takes their declared defaults): the reason names the type and the fields, and the fix (`{ migrate: true }`, a migration in GameDesc.migrations, or keeping a saved type's fields stable between releases).

Binding symbol `og_world_restore` · publish builds: kept

```ogt
fn restore {
  doc "Replace the world with a save that world.snapshot wrote (a selection replaces only its tagged entities and its resources: SnapshotDesc). First the engine's own migrations upgrade what an older engine wrote of the engine's state, then GameDesc.migrations run from the save's version to GameDesc.saveVersion, in order, on the save (SaveEdit); a refused or throwing step refuses the restore with its reason and the world is left as it was. A save of a newer saveVersion is refused. Then: entities, components, names, resources, events and the modules' state, so the restored world hashes as the saved one did, on either host. The session stays: input (og.Input, og.Pointers, og.Gamepad, og.PointerLock), the player's bus volumes (og.Audio), the player's action bindings and dead zones (og.input), the UI's focus and hover, and what lives outside the world (the platform's calls and clock, the language, the loaded assets, the command channel's scheduled steps, which keep their remaining delays though og.Time takes the saved tick, so an `og run --commands` script goes on to its last step or `quit` whether the save is of a later tick or an earlier one). Called in a system or a command it applies when the tick ends, after the last system; elsewhere (GameDesc.start, between ticks) at once. Either way the tick after it delivers og.WorldRestored, for the systems that reset what lives in the session. A save a system took mid-tick restores to that moment of its tick (og.Time reads its tick): the next step completes it (the systems after the saver, then the engine's), so the ticks after it are the saved run's. The meshes and height fields its colliders name must be loaded or registered first (a save naming one that is not is refused). A save holds only the world: progress kept in module variables is not in it, so keep it in components and resources."
  param save String "The text world.snapshot returned."
  param desc RestoreDesc "Whether a type whose layout changed since the save is mapped field by field."
  returns Result.Void "Refused, with nothing changed, for text world.snapshot did not write, a save from a newer engine or of a newer GameDesc.saveVersion, a migration that refused, or a type whose layout changed since the save (a field added, removed or retyped, an enum's variants) and that the save holds data of (a type it holds none of, no component, resource value or event, takes this build's layout; an engine og.* type that only gained fields takes their declared defaults): the reason names the type and the fields, and the fix (`{ migrate: true }`, a migration in GameDesc.migrations, or keeping a saved type's fields stable between releases)."
}
```

<sub>Module `world` line 405</sub>
