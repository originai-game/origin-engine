<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`world`](../world.md) · [index](../index.md)

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

<sub>`api.ogt` line 894</sub>
