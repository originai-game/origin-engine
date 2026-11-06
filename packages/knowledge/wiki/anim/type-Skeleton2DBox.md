<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="type-Skeleton2DBox"></a>

### `Skeleton2DBox` (struct)

A bounding box a 2D skeleton's pose shows: a `box` attachment of its rig (or of a child armature in one of its slots) while its slot shows it.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `slot` | `string` | `""` | — | — | — | The slot showing it (`slot/child-slot` inside a child armature). |
| `name` | `string` | `""` | — | — | — | The box's name in the rig. |
| `points` | `readonly Vector2[]` | `[]` | — | — | — | Its polygon, moved by its bones (IK included), in the frame the skeleton entity's og.Transform is written in (the world for an entity without a parent). |

```ogt
struct Skeleton2DBox {
  doc "A bounding box a 2D skeleton's pose shows: a `box` attachment of its rig (or of a child armature in one of its slots) while its slot shows it."
  field slot String "The slot showing it (`slot/child-slot` inside a child armature)."
  field name String "The box's name in the rig."
  field points List.Vector2 "Its polygon, moved by its bones (IK included), in the frame the skeleton entity's og.Transform is written in (the world for an entity without a parent)."
}
```

<sub>`api.ogt` line 12329</sub>
