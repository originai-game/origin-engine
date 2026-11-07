<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-SelectionRingDesc"></a>

### `SelectionRingDesc` (desc struct)

A selection ring on the ground.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `center` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | Its centre on the ground (a unit's feet). |
| `radius` | `number` | `0.6` | ≥ 0 | `m` | — | Its radius. |
| `reaction` | `Reaction` | `"self"` | — | — | — | Its colour (friendly green, hostile red, as RTS rings). |
| `color` | `Paint \| null` | `null` | — | — | — | A colour of its own. |
| `hover` | `boolean` | `false` | — | — | — | Hovered, not selected: thin and dim. |
| `selectedAt` | `number` | `0` | — | — | — | og.ui.tick() when it was selected: it springs in from larger. |
| `segments` | `number` | `0` | — | — | — | Broken into this many arcs turning slowly (a target ring); 0 is a whole ring. |
| `ground` | `GroundDesc \| null` | `null` | — | — | — | What it lies on; null lies flat at its points' y. |

```ogt
struct SelectionRingDesc {
  doc "A selection ring on the ground."
  desc true
  field center Vector3 "Its centre on the ground (a unit's feet)."
  field radius f32 "Its radius." {
    default 0.6
    unit m
    min 0
  }
  field reaction Reaction "Its colour (friendly green, hostile red, as RTS rings)." {
    default self
  }
  field color Option.Paint "A colour of its own."
  field hover bool "Hovered, not selected: thin and dim."
  field selectedAt u32 "og.ui.tick() when it was selected: it springs in from larger."
  field segments u32 "Broken into this many arcs turning slowly (a target ring); 0 is a whole ring."
  field ground Option.GroundDesc "What it lies on; null lies flat at its points' y."
}
```

<sub>Module `ui` line 6659</sub>
