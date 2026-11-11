<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`world`](../world.md) · [index](../index.md)

<a id="type-SnapshotDesc"></a>

### `SnapshotDesc` (desc struct)

What world.snapshot saves. `{}` (or no desc) saves the whole world; naming tags or resources saves a selection: the entities that carry every component of `with` (with all their components and names) and the resources `resources` names, nothing else. world.restore of a selection replaces the live entities that carry those tags by the saved ones (spawned anew in saved order, their Entity fields pointed at the new ids), sets those resources and leaves the rest of the world as it is.

See `saves §2`.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `with` | `readonly AnyComponent[]` | `[]` | — | — | — | The tags: an entity carrying all of them is saved. Empty with `resources` empty: the whole world. |
| `resources` | `readonly AnyResource[]` | `[]` | — | — | — | The resources a selection keeps (settings, progress). |

```ogt
struct SnapshotDesc {
  doc "What world.snapshot saves. `{}` (or no desc) saves the whole world; naming tags or resources saves a selection: the entities that carry every component of `with` (with all their components and names) and the resources `resources` names, nothing else. world.restore of a selection replaces the live entities that carry those tags by the saved ones (spawned anew in saved order, their Entity fields pointed at the new ids), sets those resources and leaves the rest of the world as it is."
  desc true
  guide "saves §2"
  field with List.AnyComponent "The tags: an entity carrying all of them is saved. Empty with `resources` empty: the whole world."
  field resources List.AnyResource "The resources a selection keeps (settings, progress)."
}
```

<sub>Module `world` line 45</sub>
