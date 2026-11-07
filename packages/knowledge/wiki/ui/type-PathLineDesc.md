<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-PathLineDesc"></a>

### `PathLineDesc` (desc struct)

A path on the ground.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `points` | `readonly Vector3[]` | `[]` | — | — | — | The path's world points, from the mover. |
| `reach` | `number` | `0` | ≥ 0 | `m` | — | How far along it the mover can go this turn: past it the line turns danger; 0 is all of it. |
| `label` | `string` | `""` | — | — | — | A label at its end (12 m · 2 AP); empty is none. |
| `invalid` | `boolean` | `false` | — | — | — | It cannot be taken: danger, with a cross at its end. |
| `color` | `Paint \| null` | `null` | — | — | — | Its colour; null is the highlight. |
| `width` | `number` | `0` | ≥ 0 | `cssPx` | — | Its width; 0 is 4. |
| `arrow` | `boolean` | `true` | — | — | — | An arrow head at its end (the default). |
| `ground` | `GroundDesc \| null` | `null` | — | — | — | What it lies on; null lies flat at its points' y. |

```ogt
struct PathLineDesc {
  doc "A path on the ground."
  desc true
  field points List.Vector3 "The path's world points, from the mover."
  field reach f32 "How far along it the mover can go this turn: past it the line turns danger; 0 is all of it." {
    unit m
    min 0
  }
  field label String "A label at its end (12 m · 2 AP); empty is none."
  field invalid bool "It cannot be taken: danger, with a cross at its end."
  field color Option.Paint "Its colour; null is the highlight."
  field width f32 "Its width; 0 is 4." {
    min 0
    unit cssPx
  }
  field arrow bool "An arrow head at its end (the default)." {
    default true
  }
  field ground Option.GroundDesc "What it lies on; null lies flat at its points' y."
}
```

<sub>Module `ui` line 6742</sub>
