<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-RadialDesc"></a>

### `RadialDesc` (desc struct)

A radial wheel: slices around a centre, chosen by where the pointer or a stick points.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `items` | `readonly RadialItem[]` | `[]` | — | — | — | The slices, clockwise from the top. |
| `aim` | `Vector2 \| null` | `null` | — | — | — | A stick's direction (x right, y down, -1 to 1); null reads the pointer. |
| `deadZone` | `number` | `0.35` | 0 … 1 | `ratio` | — | How far the stick must point before a slice is chosen. |
| `release` | `boolean` | `false` | — | — | — | The press or stick that opened it was let go: pick the slice it points at. |
| `radius` | `number \| null` | `null` | — | `cssPx` | — | Its radius; null is up to 170, at most 36 % of the short side. |
| `hole` | `number` | `0.42` | 0.2 … 0.8 | `ratio` | — | The centre's radius as a fraction of the wheel's. |
| `title` | `string` | `""` | — | — | — | What the centre shows when nothing is chosen. |
| `labels` | `boolean` | `true` | — | — | — | Each slice's label under its icon. |
| `dim` | `boolean` | `true` | — | — | — | Dim what is behind it. |

```ogt
struct RadialDesc {
  doc "A radial wheel: slices around a centre, chosen by where the pointer or a stick points."
  desc true
  field items List.RadialItem "The slices, clockwise from the top."
  field aim Option.Vector2 "A stick's direction (x right, y down, -1 to 1); null reads the pointer."
  field deadZone f32 "How far the stick must point before a slice is chosen." {
    default 0.35
    min 0
    max 1
    unit ratio
  }
  field release bool "The press or stick that opened it was let go: pick the slice it points at."
  field radius Option.f32 "Its radius; null is up to 170, at most 36 % of the short side." {
    unit cssPx
  }
  field hole f32 "The centre's radius as a fraction of the wheel's." {
    default 0.42
    min 0.2
    max 0.8
    unit ratio
  }
  field title String "What the centre shows when nothing is chosen."
  field labels bool "Each slice's label under its icon." {
    default true
  }
  field dim bool "Dim what is behind it." {
    default true
  }
}
```

<sub>Module `ui` line 2202</sub>
