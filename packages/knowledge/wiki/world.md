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

<sub>`api.ogt` line 788</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`Entity`](#type-Entity) | handle | An entity: an index and a generation. The value is index + generation * 2^32, with the generation below 2^21, so it stays an exact JavaScript number. |
| [`Entity()`](#column-Entity) | column kind | An Entity field: (index, generation) pairs. |
| [`SpawnDesc`](#type-SpawnDesc) | desc struct | How to spawn an entity. `{}` spawns an unnamed entity at the origin with a Transform. |
| [`RestoreDesc`](#type-RestoreDesc) | desc struct | How world.restore takes a save. |
| [`SnapshotDesc`](#type-SnapshotDesc) | desc struct | What world.snapshot saves. `{}` (or no desc) saves the whole world; naming tags or resources saves a selection: the entities that carry every component of `with` (with all their components and names) and the resources `resources` names, nothing else. world.restore of a selection replaces the live entities that carry those tags by the saved ones (spawned anew, in saved order, a saved entity's Entity fields pointed at its new id) and sets those resources, and leaves the rest of the world (other entities, og.Time, the modules' state, the session) as it is: a game keeps its progress and settings this way and rebuilds the level from its scene. A selection is small (only the types it uses are in its table) and so takes fewer store keys. |
| [`SaveType`](#type-SaveType) | struct | A type a save holds, as it was saved. |
| [`SaveRecord`](#type-SaveRecord) | struct | One saved value of a type: a component of an entity, or (entity null) the type's resource. |
| [`SetRecordsDesc`](#type-SetRecordsDesc) | desc struct | How SaveEdit.setRecords writes a type. |
| [`TransformDesc`](#type-TransformDesc) | desc struct | How SaveEdit.transform writes what it returns. |
| [`SaveSpawnDesc`](#type-SaveSpawnDesc) | desc struct | An entity a migration adds to a save. |
| [`RecordFn`](#type-RecordFn) | callback | A value's new form: called once per saved value of the type, in storage order. |
| [`SplitFn`](#type-SplitFn) | callback | Where one saved value goes when its type splits. |
| [`MigrateFn`](#type-MigrateFn) | callback | One step of a game's save migration: changes the save from SaveMigration.from to the next version. |
| [`SaveMigration`](#type-SaveMigration) | struct | A game's save migration step (GameDesc.migrations): run on a save of version `from` on its way to GameDesc.saveVersion, after the engine's own migrations. |
| [`SaveEdit`](#type-SaveEdit) | interface | A save opened for editing (world.openSave), as a migration sees it: the engine's own migrations already applied, every type with its saved fields. Each edit changes the save, not the world; world.restore takes the result. An edit refused changes nothing. Entity ids are the saved world's. |
| [`og.WorldRestored`](#record-WorldRestored) | event | A world.restore applied: the tick after it (the first whole tick after a save taken mid-tick, whose next step completes the saved tick) delivers this, read through world.events like any event. Everything in the world came back as saved, request numbers and resources the session filled included; the session did not (og-game §8), so a system that reads this resets what belongs to the session: it forgets requests answered long ago, asks the store again for the settings and progress, and closes a menu the player had open. Sent as that tick starts, not when the restore applies, so the world between the two hashes as the save did. |
| [`QueryDesc`](#type-QueryDesc) | desc struct | Which entities a query matches. The change filters (changed, added, removed) narrow an archetype's rows to those that pass every filter (ArchetypeView.rows) and read what happened since the last tick began; a component is tracked once a system filters on it (SystemDesc.changed, added, removed), and filtering on one no system tracks is refused. |
| [`SpatialDesc`](#type-SpatialDesc) | desc struct | Which entities world.spatial places: every live entity with an og.Transform (and every component of `with`, none of `without`), at its og.Transform.position (relative to its parent, for a child). |
| [`Neighbour`](#type-Neighbour) | struct | One entity a spatial query found. |
| [`SpatialIndex`](#type-SpatialIndex) | interface | A spatial hash of entities' positions as they were when world.spatial built it: it does not follow later moves (build it again after they move). Answers are ordered by distance, then by entity, so equal distances come out alike on every run and host. Valid for the system call or the call outside systems it was built in. |
| [`World`](#type-World) | interface | The world a system, a command or a game callback is handed. Game code never holds a world beyond the call it was passed to. |
| [`Query`](#type-Query) | interface | A set of matched archetypes. Iterate archetypes, never entities one by one. |
| [`ArchetypeView`](#type-ArchetypeView) | interface | One archetype's columns for this tick. Views are invalidated by structural changes; using a stale view throws with the tick and the system it belonged to. |

## Components, resources and events

<a id="record-WorldRestored"></a>

### `og.WorldRestored` (event)

A world.restore applied: the tick after it (the first whole tick after a save taken mid-tick, whose next step completes the saved tick) delivers this, read through world.events like any event. Everything in the world came back as saved, request numbers and resources the session filled included; the session did not (og-game §8), so a system that reads this resets what belongs to the session: it forgets requests answered long ago, asks the store again for the settings and progress, and closes a menu the player had open. Sent as that tick starts, not when the restore applies, so the world between the two hashes as the save did.

Token `og.WorldRestored`, registered under the schema name `og.WorldRestored`; its value type is `WorldRestored`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `from` | `number` | `0` | — | `ticks` | — | The tick the world was at when the restore applied. |
| `tick` | `number` | `0` | — | `ticks` | — | The saved tick: og.Time's tick right after the restore. |

```ogt
event WorldRestored {
  doc "A world.restore applied: the tick after it (the first whole tick after a save taken mid-tick, whose next step completes the saved tick) delivers this, read through world.events like any event. Everything in the world came back as saved, request numbers and resources the session filled included; the session did not (og-game §8), so a system that reads this resets what belongs to the session: it forgets requests answered long ago, asks the store again for the settings and progress, and closes a menu the player had open. Sent as that tick starts, not when the restore applies, so the world between the two hashes as the save did."
  field from u32 "The tick the world was at when the restore applied." {
    unit ticks
  }
  field tick u32 "The saved tick: og.Time's tick right after the restore." {
    unit ticks
  }
}
```

<sub>`api.ogt` line 964</sub>

## Interfaces

<a id="type-SaveEdit"></a>

### `SaveEdit` (interface)

A save opened for editing (world.openSave), as a migration sees it: the engine's own migrations already applied, every type with its saved fields. Each edit changes the save, not the world; world.restore takes the result. An edit refused changes nothing. Entity ids are the saved world's.

```ogt
interface SaveEdit {
  doc "A save opened for editing (world.openSave), as a migration sees it: the engine's own migrations already applied, every type with its saved fields. Each edit changes the save, not the world; world.restore takes the result. An edit refused changes nothing. Entity ids are the saved world's."
  # members follow, each quoted below
```

<sub>`api.ogt` line 892</sub>

<a id="member-SaveEdit-version"></a>

#### `SaveEdit.version` (property)

The game's save version the save was written at (1 for a save of a game that never declared one).

```ts
readonly version: number
```

Binding symbol `og_save_edit_get_version` · publish builds: kept

```ogt
prop version u32 "The game's save version the save was written at (1 for a save of a game that never declared one)."
```

<sub>`api.ogt` line 894</sub>

<a id="member-SaveEdit-types"></a>

#### `SaveEdit.types`

The types the save holds, in its table's order.

```ts
SaveEdit.types(): readonly SaveType[]
```

**Returns** `readonly SaveType[]`.

Binding symbol `og_save_edit_types` · publish builds: kept

```ogt
fn types {
  doc "The types the save holds, in its table's order."
  returns List.SaveType
}
```

<sub>`api.ogt` line 895</sub>

<a id="member-SaveEdit-has"></a>

#### `SaveEdit.has`

Whether the save holds a type.

```ts
SaveEdit.has(name: string): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | The type's name. |

**Returns** `boolean`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn has {
  doc "Whether the save holds a type."
  host false
  param name String "The type's name."
  returns bool
}
```

<sub>`api.ogt` line 899</sub>

<a id="member-SaveEdit-records"></a>

#### `SaveEdit.records`

A type's saved values: its components by entity in storage order, then its resource.

```ts
SaveEdit.records(name: string): Result<readonly SaveRecord[]>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | The type's name. |

**Returns** `Result<readonly SaveRecord[]>`: Refused for a type the save does not hold.

Binding symbol `og_save_edit_records` · publish builds: kept

```ogt
fn records {
  doc "A type's saved values: its components by entity in storage order, then its resource."
  param name String "The type's name."
  returns Result.List.SaveRecord "Refused for a type the save does not hold."
}
```

<sub>`api.ogt` line 905</sub>

<a id="member-SaveEdit-setRecords"></a>

#### `SaveEdit.setRecords`

Replace a type's values (its components, its resource) with records written as this build's type `into`: the old type's values and queued events go, an entity keeps its other components and its place. A type the save does not hold is added (a component a new version gives saved entities).

```ts
SaveEdit.setRecords(name: string, desc?: SetRecordsDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | The saved type to replace. |
| `desc` | `SetRecordsDesc` | yes | The type to write and the records. |

**Returns** `Result<void>`: Refused, with nothing changed, for a type this build does not declare, a record that does not fit it, or an entity the save does not hold.

Binding symbol `og_save_edit_set_records` · publish builds: kept

```ogt
fn setRecords {
  doc "Replace a type's values (its components, its resource) with records written as this build's type `into`: the old type's values and queued events go, an entity keeps its other components and its place. A type the save does not hold is added (a component a new version gives saved entities)."
  param name String "The saved type to replace."
  param desc SetRecordsDesc "The type to write and the records."
  returns Result.Void "Refused, with nothing changed, for a type this build does not declare, a record that does not fit it, or an entity the save does not hold."
}
```

<sub>`api.ogt` line 910</sub>

<a id="member-SaveEdit-transform"></a>

#### `SaveEdit.transform`

Rewrite every value of a type through `map`, which sees the old record: records(name), then setRecords with what map returned. With `into`, the type becomes that one (a rename that reshapes).

```ts
SaveEdit.transform(name: string, map: RecordFn, desc?: TransformDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | The saved type. |
| `map` | `RecordFn` | no | The old record to the new one. |
| `desc` | `TransformDesc` | yes | The type the results are written as. |

**Returns** `Result<void>`: Refused as setRecords is, or when map throws.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn transform {
  doc "Rewrite every value of a type through `map`, which sees the old record: records(name), then setRecords with what map returned. With `into`, the type becomes that one (a rename that reshapes)."
  host false
  param name String "The saved type."
  param map RecordFn "The old record to the new one."
  param desc TransformDesc "The type the results are written as."
  returns Result.Void "Refused as setRecords is, or when map throws."
}
```

<sub>`api.ogt` line 916</sub>

<a id="member-SaveEdit-split"></a>

#### `SaveEdit.split`

Split a type into several: each saved value is handed to `map`, whose answer names the new types' records; the old type goes. The migrated world holds the records a world built at the new version holds, and every peer and every restore of the same save migrates to the same world and hash; its hash differs from a built one's only by the archetypes the built one passed through (inserting the new types one at a time leaves an empty archetype of the first alone, which the split never makes), so compare a split's result record by record, not by hash. renameField, renameType, transform, dropType and spawn keep which types sit together: those migrate to a built world's hash exactly.

```ts
SaveEdit.split(name: string, map: SplitFn): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | The saved type. |
| `map` | `SplitFn` | no | The old record to the new types' records. |

**Returns** `Result<void>`: Refused as setRecords is, or when map throws.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn split {
  doc "Split a type into several: each saved value is handed to `map`, whose answer names the new types' records; the old type goes. The migrated world holds the records a world built at the new version holds, and every peer and every restore of the same save migrates to the same world and hash; its hash differs from a built one's only by the archetypes the built one passed through (inserting the new types one at a time leaves an empty archetype of the first alone, which the split never makes), so compare a split's result record by record, not by hash. renameField, renameType, transform, dropType and spawn keep which types sit together: those migrate to a built world's hash exactly."
  host false
  param name String "The saved type."
  param map SplitFn "The old record to the new types' records."
  returns Result.Void "Refused as setRecords is, or when map throws."
}
```

<sub>`api.ogt` line 924</sub>

<a id="member-SaveEdit-renameType"></a>

#### `SaveEdit.renameType`

Give a saved type another name, its fields and values as they are (a later restore maps them field by field against the type of the new name).

```ts
SaveEdit.renameType(from: string, to: string): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `from` | `string` | no | The saved name. |
| `to` | `string` | no | The new name. |

**Returns** `Result<void>`: Refused for a name the save lacks, or one it already has.

Binding symbol `og_save_edit_rename_type` · publish builds: kept

```ogt
fn renameType {
  doc "Give a saved type another name, its fields and values as they are (a later restore maps them field by field against the type of the new name)."
  param from String "The saved name."
  param to String "The new name."
  returns Result.Void "Refused for a name the save lacks, or one it already has."
}
```

<sub>`api.ogt` line 931</sub>

<a id="member-SaveEdit-renameField"></a>

#### `SaveEdit.renameField`

Give a saved type's field another name, its values as they are: `gold` saved, `coins` declared now, and every balance arrives.

```ts
SaveEdit.renameField(name: string, from: string, to: string): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | The saved type. |
| `from` | `string` | no | The saved field. |
| `to` | `string` | no | The new name. |

**Returns** `Result<void>`: Refused for a type or field the save lacks, or a name the type already has.

Binding symbol `og_save_edit_rename_field` · publish builds: kept

```ogt
fn renameField {
  doc "Give a saved type's field another name, its values as they are: `gold` saved, `coins` declared now, and every balance arrives."
  param name String "The saved type."
  param from String "The saved field."
  param to String "The new name."
  returns Result.Void "Refused for a type or field the save lacks, or a name the type already has."
}
```

<sub>`api.ogt` line 937</sub>

<a id="member-SaveEdit-dropType"></a>

#### `SaveEdit.dropType`

Remove a type from the save: its components, resource and queued events.

```ts
SaveEdit.dropType(name: string): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | The saved type. |

**Returns** `Result<void>`: Refused for a type the save does not hold.

Binding symbol `og_save_edit_drop_type` · publish builds: kept

```ogt
fn dropType {
  doc "Remove a type from the save: its components, resource and queued events."
  param name String "The saved type."
  returns Result.Void "Refused for a type the save does not hold."
}
```

<sub>`api.ogt` line 944</sub>

<a id="member-SaveEdit-spawn"></a>

#### `SaveEdit.spawn`

Add an entity to the save, with components of this build's types: a whole save gives it the id the next spawn would take, a selection the next free one.

```ts
SaveEdit.spawn(desc?: SaveSpawnDesc): Result<Entity>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `desc` | `SaveSpawnDesc` | yes | Its name and components. |

**Returns** `Result<Entity>`: Refused for a taken name, a type this build does not declare or a value that does not fit it.

Binding symbol `og_save_edit_spawn` · publish builds: kept

```ogt
fn spawn {
  doc "Add an entity to the save, with components of this build's types: a whole save gives it the id the next spawn would take, a selection the next free one."
  param desc SaveSpawnDesc "Its name and components."
  returns Result.Entity "Refused for a taken name, a type this build does not declare or a value that does not fit it."
}
```

<sub>`api.ogt` line 949</sub>

<a id="member-SaveEdit-text"></a>

#### `SaveEdit.text`

The save as text, at GameDesc.saveVersion: what world.restore takes.

```ts
SaveEdit.text(): Result<string>
```

**Returns** `Result<string>`.

Binding symbol `og_save_edit_text` · publish builds: kept

```ogt
fn text {
  doc "The save as text, at GameDesc.saveVersion: what world.restore takes."
  returns Result.String
}
```

<sub>`api.ogt` line 954</sub>

<a id="member-SaveEdit-close"></a>

#### `SaveEdit.close`

Let go of the open save. world.restore closes the ones it opens.

```ts
SaveEdit.close(): void
```

**Returns** `void`.

Binding symbol `og_save_edit_close` · publish builds: kept

```ogt
fn close {
  doc "Let go of the open save. world.restore closes the ones it opens."
  returns Void
}
```

<sub>`api.ogt` line 958</sub>

<a id="type-SpatialIndex"></a>

### `SpatialIndex` (interface)

A spatial hash of entities' positions as they were when world.spatial built it: it does not follow later moves (build it again after they move). Answers are ordered by distance, then by entity, so equal distances come out alike on every run and host. Valid for the system call or the call outside systems it was built in.

```ogt
interface SpatialIndex {
  doc "A spatial hash of entities' positions as they were when world.spatial built it: it does not follow later moves (build it again after they move). Answers are ordered by distance, then by entity, so equal distances come out alike on every run and host. Valid for the system call or the call outside systems it was built in."
  # members follow, each quoted below
```

<sub>`api.ogt` line 1003</sub>

<a id="member-SpatialIndex-count"></a>

#### `SpatialIndex.count` (property)

The entities it holds.

```ts
readonly count: number
```

Binding symbol `og_spatial_index_get_count` · publish builds: kept

```ogt
prop count u32 "The entities it holds."
```

<sub>`api.ogt` line 1005</sub>

<a id="member-SpatialIndex-within"></a>

#### `SpatialIndex.within`

Every entity at most `radius` from `center`, nearest first.

```ts
SpatialIndex.within(center: Vector3, radius: number): readonly Neighbour[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `center` | `Vector3` | no | The centre. |
| `radius` | `number` | no | The radius in metres; a negative one finds nothing. |

**Returns** `readonly Neighbour[]`.

Binding symbol `og_spatial_index_within` · publish builds: kept

```ogt
fn within {
  doc "Every entity at most `radius` from `center`, nearest first."
  param center Vector3 "The centre."
  param radius f32 "The radius in metres; a negative one finds nothing."
  returns List.Neighbour
}
```

<sub>`api.ogt` line 1006</sub>

<a id="member-SpatialIndex-nearest"></a>

#### `SpatialIndex.nearest`

The `count` entities nearest `center`, nearest first; fewer when it holds fewer.

```ts
SpatialIndex.nearest(center: Vector3, count: number): readonly Neighbour[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `center` | `Vector3` | no | The centre. |
| `count` | `number` | no | How many. |

**Returns** `readonly Neighbour[]`.

Binding symbol `og_spatial_index_nearest` · publish builds: kept

```ogt
fn nearest {
  doc "The `count` entities nearest `center`, nearest first; fewer when it holds fewer."
  param center Vector3 "The centre."
  param count u32 "How many."
  returns List.Neighbour
}
```

<sub>`api.ogt` line 1012</sub>

<a id="type-World"></a>

### `World` (interface)

The world a system, a command or a game callback is handed. Game code never holds a world beyond the call it was passed to.

```ogt
interface World {
  doc "The world a system, a command or a game callback is handed. Game code never holds a world beyond the call it was passed to."
  # members follow, each quoted below
```

<sub>`api.ogt` line 1020</sub>

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

<sub>`api.ogt` line 1023</sub>

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

<sub>`api.ogt` line 1025</sub>

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

<sub>`api.ogt` line 1031</sub>

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

<sub>`api.ogt` line 1037</sub>

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

<sub>`api.ogt` line 1043</sub>

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

<sub>`api.ogt` line 1051</sub>

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

<sub>`api.ogt` line 1062</sub>

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

<sub>`api.ogt` line 1070</sub>

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

<sub>`api.ogt` line 1078</sub>

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

<sub>`api.ogt` line 1087</sub>

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

<sub>`api.ogt` line 1094</sub>

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

<sub>`api.ogt` line 1100</sub>

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

<sub>`api.ogt` line 1107</sub>

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

<sub>`api.ogt` line 1115</sub>

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

<sub>`api.ogt` line 1121</sub>

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

<sub>`api.ogt` line 1128</sub>

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

<sub>`api.ogt` line 1134</sub>

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

<sub>`api.ogt` line 1140</sub>

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

<sub>`api.ogt` line 1146</sub>

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

<sub>`api.ogt` line 1152</sub>

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

<sub>`api.ogt` line 1159</sub>

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

<sub>`api.ogt` line 1167</sub>

<a id="member-World-snapshot"></a>

#### `World.snapshot`

The whole world as text, to save a game's progress in the platform store: every entity with its components and names, every resource (og.Time's clock and the og.Rng streams too), the pending events and the modules' state (physics, animation, flipbooks, tweens, tilemap and terrain edits), each type with its schema hash, and the game's save version (GameDesc.saveVersion), deflated and base64-encoded after `og-save:2:`. Every build has it, publish included, and the same world saves to the same text on both hosts. Save it with og.platform.saveSlot (`{ slot: "slot1", text, meta: { name, chimes } }`), which spreads it over as many store keys as it needs (a key holds 64 KiB) and writes the slot's manifest last, then og.platform.loadSlot and world.restore of the answer's `text` (examples/hollowbell, a 2D platformer of 189 entities on a 96 x 48 tile map, saves to about 25 KiB; examples/kestrel-stair, a 3D world of 93 entities with skinned characters, cloths and minds, to about 78 KB; a world of a few entities to about 11 KiB, most of it the table of every registered type; 5,000 entities, half of them physics bodies, to about 436 KiB, written in about 95 ms and restored in about 60 ms natively). With SnapshotDesc's `with` or `resources`, only the tagged entities and the chosen resources (a selection). Inside a system it is the world in the middle of that tick, as that system sees it: what earlier systems wrote this tick is in it, a structural change the system queued (applied when it returns) is not, and it names the system, so a world restored from it completes that tick: its next step runs the systems after this one and the engine's (physics included) with the tick's own events, then ends the tick, and the world goes on tick for tick as the one that saved did (a replay from it matches; make the snapshot the system's last write of the tick, since what it does after the call is not in the save). Between ticks (GameDesc.start, a command, a UI callback) it is the world between two ticks. Physics carries its mesh, convex-hull and height-field colliders by their geometry's handle and scale, not their triangles (og.physics.setMesh). Asset and material handles in components are saved as their numbers: load what a saved entity uses in the same order on every launch (in GameDesc.start) before restoring.

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
  doc "The whole world as text, to save a game's progress in the platform store: every entity with its components and names, every resource (og.Time's clock and the og.Rng streams too), the pending events and the modules' state (physics, animation, flipbooks, tweens, tilemap and terrain edits), each type with its schema hash, and the game's save version (GameDesc.saveVersion), deflated and base64-encoded after `og-save:2:`. Every build has it, publish included, and the same world saves to the same text on both hosts. Save it with og.platform.saveSlot (`{ slot: \"slot1\", text, meta: { name, chimes } }`), which spreads it over as many store keys as it needs (a key holds 64 KiB) and writes the slot's manifest last, then og.platform.loadSlot and world.restore of the answer's `text` (examples/hollowbell, a 2D platformer of 189 entities on a 96 x 48 tile map, saves to about 25 KiB; examples/kestrel-stair, a 3D world of 93 entities with skinned characters, cloths and minds, to about 78 KB; a world of a few entities to about 11 KiB, most of it the table of every registered type; 5,000 entities, half of them physics bodies, to about 436 KiB, written in about 95 ms and restored in about 60 ms natively). With SnapshotDesc's `with` or `resources`, only the tagged entities and the chosen resources (a selection). Inside a system it is the world in the middle of that tick, as that system sees it: what earlier systems wrote this tick is in it, a structural change the system queued (applied when it returns) is not, and it names the system, so a world restored from it completes that tick: its next step runs the systems after this one and the engine's (physics included) with the tick's own events, then ends the tick, and the world goes on tick for tick as the one that saved did (a replay from it matches; make the snapshot the system's last write of the tick, since what it does after the call is not in the save). Between ticks (GameDesc.start, a command, a UI callback) it is the world between two ticks. Physics carries its mesh, convex-hull and height-field colliders by their geometry's handle and scale, not their triangles (og.physics.setMesh). Asset and material handles in components are saved as their numbers: load what a saved entity uses in the same order on every launch (in GameDesc.start) before restoring."
  param desc SnapshotDesc "The whole world, or a selection: tagged entities and chosen resources." {
    optional true
  }
  returns Result.String "The save, or why not (the world could not be read, or SnapshotDesc names a type this world does not register)."
}
```

<sub>`api.ogt` line 1175</sub>

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

<sub>`api.ogt` line 1183</sub>

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

<sub>`api.ogt` line 1189</sub>

<a id="type-Query"></a>

### `Query` (interface)

A set of matched archetypes. Iterate archetypes, never entities one by one.

```ogt
interface Query {
  doc "A set of matched archetypes. Iterate archetypes, never entities one by one."
  # members follow, each quoted below
```

<sub>`api.ogt` line 1197</sub>

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

<sub>`api.ogt` line 1199</sub>

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

<sub>`api.ogt` line 1203</sub>

<a id="type-ArchetypeView"></a>

### `ArchetypeView` (interface)

One archetype's columns for this tick. Views are invalidated by structural changes; using a stale view throws with the tick and the system it belonged to.

```ogt
interface ArchetypeView {
  doc "One archetype's columns for this tick. Views are invalidated by structural changes; using a stale view throws with the tick and the system it belonged to."
  # members follow, each quoted below
```

<sub>`api.ogt` line 1209</sub>

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

<sub>`api.ogt` line 1211</sub>

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

<sub>`api.ogt` line 1212</sub>

<a id="member-ArchetypeView-entities"></a>

#### `ArchetypeView.entities` (property)

The entities as (index, generation) pairs: entity i is at 2i and 2i + 1. These are not handles: for a handle use entity(i) (world.despawn, get, set, insert and remove refuse a bare index, naming the call).

```ts
readonly entities: Uint32Array
```

Binding symbol `og_archetype_view_get_entities` · publish builds: kept

```ogt
prop entities Uint32Array "The entities as (index, generation) pairs: entity i is at 2i and 2i + 1. These are not handles: for a handle use entity(i) (world.despawn, get, set, insert and remove refuse a bare index, naming the call)."
```

<sub>`api.ogt` line 1213</sub>

<a id="member-ArchetypeView-rows"></a>

#### `ArchetypeView.rows` (property)

The rows the query's change filters pass (SystemDesc.changed, added, removed; QueryDesc's), in row order; every row, 0 to count - 1, for a query without them. Columns still hold every row: loop over rows and index the columns with them.

```ts
readonly rows: Uint32Array
```

Binding symbol `og_archetype_view_get_rows` · publish builds: kept

```ogt
prop rows Uint32Array "The rows the query's change filters pass (SystemDesc.changed, added, removed; QueryDesc's), in row order; every row, 0 to count - 1, for a query without them. Columns still hold every row: loop over rows and index the columns with them."
```

<sub>`api.ogt` line 1214</sub>

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

<sub>`api.ogt` line 1215</sub>

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

<sub>`api.ogt` line 1220</sub>

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

<sub>`api.ogt` line 1226</sub>

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

<sub>`api.ogt` line 793</sub>

<a id="type-SpawnDesc"></a>

### `SpawnDesc` (desc struct)

How to spawn an entity. `{}` spawns an unnamed entity at the origin with a Transform.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | A unique name; the probe and the command channel address the entity by it. Empty leaves it unnamed. |
| `parent` | `Entity \| null` | `null` | — | — | — | The parent entity; the new entity's Transform is then relative to it. It gives the new entity an og.Parent, so inside a system og.Parent must be among the system's writes (as og.Transform must, unless `bare`). |
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | The initial Transform.position. |
| `quaternion` | `Quaternion` | `{ x: 0, y: 0, z: 0, w: 1 }` | — | — | — | The initial Transform.quaternion. |
| `scale` | `Vector3` | `{ x: 1, y: 1, z: 1 }` | — | — | — | The initial Transform.scale. |
| `bare` | `boolean` | `false` | — | — | — | Spawn without a Transform: a pure data entity. |

```ogt
struct SpawnDesc {
  doc "How to spawn an entity. `{}` spawns an unnamed entity at the origin with a Transform."
  desc true
  field name String "A unique name; the probe and the command channel address the entity by it. Empty leaves it unnamed."
  field parent Option.Entity "The parent entity; the new entity's Transform is then relative to it. It gives the new entity an og.Parent, so inside a system og.Parent must be among the system's writes (as og.Transform must, unless `bare`)."
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

<sub>`api.ogt` line 805</sub>

<a id="type-RestoreDesc"></a>

### `RestoreDesc` (desc struct)

How world.restore takes a save.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `migrate` | `boolean` | `false` | — | — | — | Map a type whose layout changed since the save field by field, by name, as a hot reload does: a field added since takes its default, a removed one is dropped, an enum value is mapped by its variant's name, a field of another type starts at its default, and a type this build no longer registers is dropped. Off, such a save is refused naming the type and its fields, except for the host's input the restore keeps (og.Input, og.Pointers, og.Gamepad, og.PointerLock, og.Audio), which is mapped either way: a save from before og.Gamepad's eight pads restores without it. |

```ogt
struct RestoreDesc {
  doc "How world.restore takes a save."
  desc true
  field migrate bool "Map a type whose layout changed since the save field by field, by name, as a hot reload does: a field added since takes its default, a removed one is dropped, an enum value is mapped by its variant's name, a field of another type starts at its default, and a type this build no longer registers is dropped. Off, such a save is refused naming the type and its fields, except for the host's input the restore keeps (og.Input, og.Pointers, og.Gamepad, og.PointerLock, og.Audio), which is mapped either way: a save from before og.Gamepad's eight pads restores without it."
}
```

<sub>`api.ogt` line 820</sub>

<a id="type-SnapshotDesc"></a>

### `SnapshotDesc` (desc struct)

What world.snapshot saves. `{}` (or no desc) saves the whole world; naming tags or resources saves a selection: the entities that carry every component of `with` (with all their components and names) and the resources `resources` names, nothing else. world.restore of a selection replaces the live entities that carry those tags by the saved ones (spawned anew, in saved order, a saved entity's Entity fields pointed at its new id) and sets those resources, and leaves the rest of the world (other entities, og.Time, the modules' state, the session) as it is: a game keeps its progress and settings this way and rebuilds the level from its scene. A selection is small (only the types it uses are in its table) and so takes fewer store keys.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `with` | `readonly AnyComponent[]` | `[]` | — | — | — | The tags: an entity carrying all of them is saved. Empty with `resources` empty: the whole world. |
| `resources` | `readonly AnyResource[]` | `[]` | — | — | — | The resources a selection keeps (settings, progress). |

```ogt
struct SnapshotDesc {
  doc "What world.snapshot saves. `{}` (or no desc) saves the whole world; naming tags or resources saves a selection: the entities that carry every component of `with` (with all their components and names) and the resources `resources` names, nothing else. world.restore of a selection replaces the live entities that carry those tags by the saved ones (spawned anew, in saved order, a saved entity's Entity fields pointed at its new id) and sets those resources, and leaves the rest of the world (other entities, og.Time, the modules' state, the session) as it is: a game keeps its progress and settings this way and rebuilds the level from its scene. A selection is small (only the types it uses are in its table) and so takes fewer store keys."
  desc true
  field with List.AnyComponent "The tags: an entity carrying all of them is saved. Empty with `resources` empty: the whole world."
  field resources List.AnyResource "The resources a selection keeps (settings, progress)."
}
```

<sub>`api.ogt` line 826</sub>

<a id="type-SaveType"></a>

### `SaveType` (struct)

A type a save holds, as it was saved.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | Its reflected name (`Wallet`, `og.Transform`). |
| `version` | `number` | `0` | — | — | — | Its schema version when it was saved. |
| `fields` | `readonly string[]` | `[]` | — | — | — | Its fields, in saved order. |

```ogt
struct SaveType {
  doc "A type a save holds, as it was saved."
  field name String "Its reflected name (`Wallet`, `og.Transform`)."
  field version u32 "Its schema version when it was saved."
  field fields List.String "Its fields, in saved order."
}
```

<sub>`api.ogt` line 833</sub>

<a id="type-SaveRecord"></a>

### `SaveRecord` (struct)

One saved value of a type: a component of an entity, or (entity null) the type's resource.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity \| null` | `null` | — | — | — | The saved entity, or null for the resource. |
| `value` | `unknown` | `null` | — | — | — | The value's fields by name, as saved. |

```ogt
struct SaveRecord {
  doc "One saved value of a type: a component of an entity, or (entity null) the type's resource."
  field entity Option.Entity "The saved entity, or null for the resource."
  field value Json "The value's fields by name, as saved."
}
```

<sub>`api.ogt` line 840</sub>

<a id="type-SetRecordsDesc"></a>

### `SetRecordsDesc` (desc struct)

How SaveEdit.setRecords writes a type.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `into` | `string` | `""` | — | — | — | The type to write them as, one this build declares; empty: the type itself (as this build declares it). |
| `records` | `readonly SaveRecord[]` | `[]` | — | — | — | The new values; a field a record leaves out takes its default, a key the type lacks is refused. |

```ogt
struct SetRecordsDesc {
  doc "How SaveEdit.setRecords writes a type."
  desc true
  field into String "The type to write them as, one this build declares; empty: the type itself (as this build declares it)."
  field records List.SaveRecord "The new values; a field a record leaves out takes its default, a key the type lacks is refused."
}
```

<sub>`api.ogt` line 846</sub>

<a id="type-TransformDesc"></a>

### `TransformDesc` (desc struct)

How SaveEdit.transform writes what it returns.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `into` | `string` | `""` | — | — | — | The type to write the results as, one this build declares; empty: the type itself. |

```ogt
struct TransformDesc {
  doc "How SaveEdit.transform writes what it returns."
  desc true
  field into String "The type to write the results as, one this build declares; empty: the type itself."
}
```

<sub>`api.ogt` line 853</sub>

<a id="type-SaveSpawnDesc"></a>

### `SaveSpawnDesc` (desc struct)

An entity a migration adds to a save.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | A unique name; empty leaves it unnamed. |
| `components` | `unknown` | `null` | — | — | — | Its components by type name, each a value of the type as this build declares it (`{ "og.Transform": {}, Chest: { gold: 5 } }`). |

```ogt
struct SaveSpawnDesc {
  doc "An entity a migration adds to a save."
  desc true
  field name String "A unique name; empty leaves it unnamed."
  field components Json "Its components by type name, each a value of the type as this build declares it (`{ \"og.Transform\": {}, Chest: { gold: 5 } }`)."
}
```

<sub>`api.ogt` line 859</sub>

<a id="type-RecordFn"></a>

### `RecordFn` (callback)

A value's new form: called once per saved value of the type, in storage order.

Game code supplies this function.

```ts
type RecordFn = (value: unknown, entity: Entity | null) => unknown
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `value` | `unknown` | no | The old record: its fields by name, as saved. |
| `entity` | `Entity \| null` | no | Its entity, or null for the type's resource. |

```ogt
callback RecordFn {
  doc "A value's new form: called once per saved value of the type, in storage order."
  param value Json "The old record: its fields by name, as saved."
  param entity Option.Entity "Its entity, or null for the type's resource."
  returns Json "The new record: fields by name of the type it is written as; one left out takes its default."
}
```

<sub>`api.ogt` line 866</sub>

<a id="type-SplitFn"></a>

### `SplitFn` (callback)

Where one saved value goes when its type splits.

Game code supplies this function.

```ts
type SplitFn = (value: unknown, entity: Entity | null) => unknown
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `value` | `unknown` | no | The old record. |
| `entity` | `Entity \| null` | no | Its entity, or null for the resource. |

```ogt
callback SplitFn {
  doc "Where one saved value goes when its type splits."
  param value Json "The old record."
  param entity Option.Entity "Its entity, or null for the resource."
  returns Json "An object of the new types' names to their records: `{ Health: { hp: v.hp }, Mana: { mp: v.mp } }`; a type left out or null, and the entity does not get it."
}
```

<sub>`api.ogt` line 873</sub>

<a id="type-MigrateFn"></a>

### `MigrateFn` (callback)

One step of a game's save migration: changes the save from SaveMigration.from to the next version.

Game code supplies this function.

```ts
type MigrateFn = (save: SaveEdit) => Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `save` | `SaveEdit` | no | The save, open for editing. |

```ogt
callback MigrateFn {
  doc "One step of a game's save migration: changes the save from SaveMigration.from to the next version."
  param save SaveEdit "The save, open for editing."
  returns Result.Void "Refused (or a throw): the restore is refused with the reason, and the world is left as it was."
}
```

<sub>`api.ogt` line 880</sub>

<a id="type-SaveMigration"></a>

### `SaveMigration` (struct)

A game's save migration step (GameDesc.migrations): run on a save of version `from` on its way to GameDesc.saveVersion, after the engine's own migrations.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `from` | `number` | `0` | — | — | — | The save version it migrates from, to from + 1; from 1 to GameDesc.saveVersion - 1, each at most once. |
| `migrate` | `MigrateFn` | — | — | — | — | The step. |

```ogt
struct SaveMigration {
  doc "A game's save migration step (GameDesc.migrations): run on a save of version `from` on its way to GameDesc.saveVersion, after the engine's own migrations."
  field from u32 "The save version it migrates from, to from + 1; from 1 to GameDesc.saveVersion - 1, each at most once."
  field migrate MigrateFn "The step."
}
```

<sub>`api.ogt` line 886</sub>

<a id="type-QueryDesc"></a>

### `QueryDesc` (desc struct)

Which entities a query matches. The change filters (changed, added, removed) narrow an archetype's rows to those that pass every filter (ArchetypeView.rows) and read what happened since the last tick began; a component is tracked once a system filters on it (SystemDesc.changed, added, removed), and filtering on one no system tracks is refused.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `with` | `readonly AnyComponent[]` | `[]` | — | — | — | Components an entity must have. |
| `without` | `readonly AnyComponent[]` | `[]` | — | — | — | Components an entity must not have. |
| `changed` | `readonly AnyComponent[]` | `[]` | — | — | — | Components (required, as `with`) whose value changed: a write that leaves a value as it was is not a change. |
| `added` | `readonly AnyComponent[]` | `[]` | — | — | — | Components (required, as `with`) added: spawned with, or inserted into an entity without them. |
| `removed` | `readonly AnyComponent[]` | `[]` | — | — | — | Components the entity lost (removed; a despawned entity is in no archetype, so it is in world.removed only). |

```ogt
struct QueryDesc {
  doc "Which entities a query matches. The change filters (changed, added, removed) narrow an archetype's rows to those that pass every filter (ArchetypeView.rows) and read what happened since the last tick began; a component is tracked once a system filters on it (SystemDesc.changed, added, removed), and filtering on one no system tracks is refused."
  desc true
  field with List.AnyComponent "Components an entity must have."
  field without List.AnyComponent "Components an entity must not have."
  field changed List.AnyComponent "Components (required, as `with`) whose value changed: a write that leaves a value as it was is not a change."
  field added List.AnyComponent "Components (required, as `with`) added: spawned with, or inserted into an entity without them."
  field removed List.AnyComponent "Components the entity lost (removed; a despawned entity is in no archetype, so it is in world.removed only)."
}
```

<sub>`api.ogt` line 974</sub>

<a id="type-SpatialDesc"></a>

### `SpatialDesc` (desc struct)

Which entities world.spatial places: every live entity with an og.Transform (and every component of `with`, none of `without`), at its og.Transform.position (relative to its parent, for a child).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `with` | `readonly AnyComponent[]` | `[]` | — | — | — | Components an entity must have to be placed. |
| `without` | `readonly AnyComponent[]` | `[]` | — | — | — | Components an entity must not have. |
| `cell` | `number` | `0` | ≥ 0 | `m` | — | The hash's cell size; 0 picks one from the entities' spread. Answers never depend on it, only speed. |

```ogt
struct SpatialDesc {
  doc "Which entities world.spatial places: every live entity with an og.Transform (and every component of `with`, none of `without`), at its og.Transform.position (relative to its parent, for a child)."
  desc true
  field with List.AnyComponent "Components an entity must have to be placed."
  field without List.AnyComponent "Components an entity must not have."
  field cell f32 "The hash's cell size; 0 picks one from the entities' spread. Answers never depend on it, only speed." {
    unit m
    min 0
  }
}
```

<sub>`api.ogt` line 984</sub>

<a id="type-Neighbour"></a>

### `Neighbour` (struct)

One entity a spatial query found.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The entity. |
| `distance` | `number` | `0` | — | `m` | — | Its distance from the query's centre. |

```ogt
struct Neighbour {
  doc "One entity a spatial query found."
  field entity Entity "The entity."
  field distance f64 "Its distance from the query's centre." {
    unit m
  }
}
```

<sub>`api.ogt` line 995</sub>

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

<sub>`api.ogt` line 798</sub>
