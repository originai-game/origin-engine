<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-CompassPoi"></a>

### `CompassPoi` (desc struct)

A point on a compass strip.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `key` | `string` | `""` | — | — | — | Its key (its probe id). |
| `bearing` | `number` | `0` | — | `rad` | — | Its bearing from the player, clockwise from north. |
| `distance` | `number \| null` | `null` | — | `m` | — | How far (shown under it); null shows none. |
| `icon` | `string` | `""` | — | — | — | Its icon. |
| `kind` | `MarkerKind` | `"player"` | — | — | — | What it is (its colour, and objectives and pings stay on the strip's ends when behind). |
| `color` | `Paint \| null` | `null` | — | — | — | Its colour; null is the kind's. |
| `label` | `string` | `""` | — | — | — | What a screen reader says of it. |

```ogt
struct CompassPoi {
  doc "A point on a compass strip."
  desc true
  field key String "Its key (its probe id)."
  field bearing f32 "Its bearing from the player, clockwise from north." {
    unit rad
  }
  field distance Option.f32 "How far (shown under it); null shows none." {
    unit m
  }
  field icon String "Its icon."
  field kind MarkerKind "What it is (its colour, and objectives and pings stay on the strip's ends when behind)."
  field color Option.Paint "Its colour; null is the kind's."
  field label String "What a screen reader says of it."
}
```

<sub>`api.ogt` line 21929</sub>
