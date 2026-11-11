<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `world` module

The world: an archetype ECS. Entities are generational handles; components are columns in engine memory.

Structural changes (spawn, despawn, insert, remove) invalidate typed views; the API refuses a stale view.

- **TypeScript**: functions are none at module level (its members live on the types below); components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module world {
  doc "The world: an archetype ECS. Entities are generational handles; components are columns in engine memory."
  doc "Structural changes (spawn, despawn, insert, remove) invalidate typed views; the API refuses a stale view."
  # declarations follow, each on its own page
```

<sub>Module `world` line 1</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`Entity`](world/type-Entity.md) | handle |  | An entity: an index and a generation. |
| [`Entity()`](#column-Entity) | column kind |  | An Entity field: (index, generation) pairs. |
| [`SpawnDesc`](world/type-SpawnDesc.md) | desc struct |  | How to spawn an entity. |
| [`RestoreDesc`](world/type-RestoreDesc.md) | desc struct |  | How world.restore takes a save. |
| [`SnapshotDesc`](world/type-SnapshotDesc.md) | desc struct |  | What world.snapshot saves. |
| [`SaveType`](world/type-SaveType.md) | struct |  | A type a save holds, as it was saved. |
| [`SaveRecord`](world/type-SaveRecord.md) | struct |  | One saved value of a type: a component of an entity, or (entity null) the type's resource. |
| [`SetRecordsDesc`](world/type-SetRecordsDesc.md) | desc struct |  | How SaveEdit.setRecords writes a type. |
| [`TransformDesc`](world/type-TransformDesc.md) | desc struct |  | How SaveEdit.transform writes what it returns. |
| [`SaveSpawnDesc`](world/type-SaveSpawnDesc.md) | desc struct |  | An entity a migration adds to a save. |
| [`RecordFn`](world/type-RecordFn.md) | callback |  | A value's new form: called once per saved value of the type, in storage order. |
| [`SplitFn`](world/type-SplitFn.md) | callback |  | Where one saved value goes when its type splits. |
| [`MigrateFn`](world/type-MigrateFn.md) | callback |  | One step of a game's save migration: changes the save from SaveMigration.from to the next version. |
| [`SaveMigration`](world/type-SaveMigration.md) | struct |  | A game's save migration step (GameDesc.migrations): run on a save of version `from` on its way to GameDesc.saveVersion, after the engine's own migrations. |
| [`SaveEdit`](world/type-SaveEdit.md) | interface |  | A save opened for editing (world.openSave), as a migration sees it: the engine's own migrations already applied, every type with its saved fields. |
| [`og.WorldRestored`](world/record-WorldRestored.md) | event |  | A world.restore applied: sent as the tick after it starts (after a mid-tick save, the first whole tick), not when the restore applies, so the world between the two hashes as the save did; read throug… |
| [`QueryDesc`](world/type-QueryDesc.md) | desc struct |  | Which entities a query matches. |
| [`SpatialDesc`](world/type-SpatialDesc.md) | desc struct |  | Which entities world.spatial places: every live entity with an og.Transform (and every component of `with`, none of `without`), at its og.Transform.position (relative to its parent, for a child). |
| [`Neighbour`](world/type-Neighbour.md) | struct |  | One entity a spatial query found. |
| [`SpatialIndex`](world/type-SpatialIndex.md) | interface |  | A spatial hash of entities' positions as they were when world.spatial built it: it does not follow later moves (build it again after they move). |
| [`World`](world/type-World.md) | interface |  | The world a system, a command or a game callback is handed. |
| [`Query`](world/type-Query.md) | interface |  | A set of matched archetypes. |
| [`ArchetypeView`](world/type-ArchetypeView.md) | interface |  | One archetype's columns for this tick. |

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

<sub>Module `world` line 10</sub>
