<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`world`](../world.md) · [index](../index.md)

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

<sub>Module `world` line 198</sub>
