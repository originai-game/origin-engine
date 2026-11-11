<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="type-MarkerView"></a>

### `MarkerView` (struct)

Where a quest points: the marker og.story.mark set, else the current stage's first undone objective's `marker`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | An entity (0: none). |
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | `m` | — | A point. |

```ogt
struct MarkerView {
  doc "Where a quest points: the marker og.story.mark set, else the current stage's first undone objective's `marker`."
  field entity Entity "An entity (0: none)."
  field position Vector3 "A point." {
    unit m
  }
}
```

<sub>Module `story` line 234</sub>
