<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-TelegraphDesc"></a>

### `TelegraphDesc` (desc struct)

An area an attack will hit.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `center` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | Its origin on the ground. |
| `shape` | `TelegraphShape` | `"circle"` | — | — | — | Its area. |
| `radius` | `number` | `4` | ≥ 0 | `m` | — | A disc's or ring's radius, a cone's or line's length. |
| `inner` | `number` | `0` | ≥ 0 | `m` | — | A ring's inner radius. |
| `angle` | `number` | `1.0471976` | — | `rad` | — | A cone's width. |
| `width` | `number` | `2` | ≥ 0 | `m` | — | A line's width. |
| `direction` | `number` | `0` | — | `rad` | — | Which way a cone or line faces, clockwise from +z seen from above (0 is +z, π/2 is +x). |
| `progress` | `number` | `0` | — | `ratio` | — | The wind-up 0 to 1: the fill grows to the edge as the hit nears. |
| `hostile` | `boolean` | `true` | — | — | — | An enemy's (danger, the default); else an ally's (info). |
| `color` | `Paint \| null` | `null` | — | — | — | A colour of its own. |
| `ground` | `GroundDesc \| null` | `null` | — | — | — | What it lies on; null lies flat at its points' y. |

```ogt
struct TelegraphDesc {
  doc "An area an attack will hit."
  desc true
  field center Vector3 "Its origin on the ground."
  field shape TelegraphShape "Its area."
  field radius f32 "A disc's or ring's radius, a cone's or line's length." {
    default 4
    unit m
    min 0
  }
  field inner f32 "A ring's inner radius." {
    unit m
    min 0
  }
  field angle f32 "A cone's width." {
    default 1.0471976
    unit rad
  }
  field width f32 "A line's width." {
    default 2
    unit m
    min 0
  }
  field direction f32 "Which way a cone or line faces, clockwise from +z seen from above (0 is +z, π/2 is +x)." {
    unit rad
  }
  field progress f32 "The wind-up 0 to 1: the fill grows to the edge as the hit nears." {
    unit ratio
  }
  field hostile bool "An enemy's (danger, the default); else an ally's (info)." {
    default true
  }
  field color Option.Paint "A colour of its own."
  field ground Option.GroundDesc "What it lies on; null lies flat at its points' y."
}
```

<sub>`api.ogt` line 22474</sub>
