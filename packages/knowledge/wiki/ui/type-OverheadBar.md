<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-OverheadBar"></a>

### `OverheadBar` (desc struct)

One overhead bar.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `key` | `string` | `""` | — | — | — | The unit's key (its ghost is tracked by it). |
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | The world point it sits on (over the unit's head). |
| `value` | `number` | `0` | — | `ratio` | — | Health 0 to 1. |
| `reaction` | `Reaction` | `"hostile"` | — | — | — | Its colour. |
| `color` | `Paint \| null` | `null` | — | — | — | A colour of its own. |

```ogt
struct OverheadBar {
  doc "One overhead bar."
  desc true
  field key String "The unit's key (its ghost is tracked by it)."
  field position Vector3 "The world point it sits on (over the unit's head)."
  field value f32 "Health 0 to 1." {
    unit ratio
  }
  field reaction Reaction "Its colour."
  field color Option.Paint "A colour of its own."
}
```

<sub>Module `ui` line 6464</sub>
