<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-WorldTarget"></a>

### `WorldTarget` (desc struct)

What an anchored element or a world panel is placed on: an entity (its world pose, down og.Parent) or a world point.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity \| null` | `null` | — | — | — | The entity; null places it on point. |
| `point` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | The world point, when entity is null. |

```ogt
struct WorldTarget {
  doc "What an anchored element or a world panel is placed on: an entity (its world pose, down og.Parent) or a world point."
  desc true
  field entity Option.Entity "The entity; null places it on point."
  field point Vector3 "The world point, when entity is null."
}
```

<sub>Module `ui` line 7248</sub>
