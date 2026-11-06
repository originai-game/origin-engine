<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-PlacementGhostDesc"></a>

### `PlacementGhostDesc` (desc struct)

A building's footprint while it is placed.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `center` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | Its footprint's centre on the ground. |
| `width` | `number` | `2` | ≥ 0 | `m` | — | Its width along x before rotation. |
| `depth` | `number` | `2` | ≥ 0 | `m` | — | Its depth along z. |
| `rotation` | `number` | `0` | — | `rad` | — | Its turn about y, clockwise from above. |
| `cell` | `number` | `0` | ≥ 0 | `m` | — | The grid's cell; the footprint shows its cells; 0 shows none. |
| `valid` | `boolean` | `true` | — | — | — | It can be placed here (success); else danger with hatching. |
| `label` | `string` | `""` | — | — | — | Its cost or why not (Blocked); empty is none. |
| `range` | `number \| null` | `null` | — | `m` | — | A range ring round it (a tower's reach). |
| `ground` | `GroundDesc \| null` | `null` | — | — | — | What it lies on; null lies flat at its points' y. |

```ogt
struct PlacementGhostDesc {
  doc "A building's footprint while it is placed."
  desc true
  field center Vector3 "Its footprint's centre on the ground."
  field width f32 "Its width along x before rotation." {
    default 2
    unit m
    min 0
  }
  field depth f32 "Its depth along z." {
    default 2
    unit m
    min 0
  }
  field rotation f32 "Its turn about y, clockwise from above." {
    unit rad
  }
  field cell f32 "The grid's cell; the footprint shows its cells; 0 shows none." {
    unit m
    min 0
  }
  field valid bool "It can be placed here (success); else danger with hatching." {
    default true
  }
  field label String "Its cost or why not (Blocked); empty is none."
  field range Option.f32 "A range ring round it (a tower's reach)." {
    unit m
  }
  field ground Option.GroundDesc "What it lies on; null lies flat at its points' y."
}
```

<sub>`api.ogt` line 22529</sub>
