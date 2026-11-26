<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`procgen`](../procgen.md) · [index](../index.md)

<a id="type-PlantLevelDesc"></a>

### `PlantLevelDesc` (desc struct)

Growth parameters for one branch level (level 0 is the trunk).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `length` | `number` | `1` | ≥ 0 | `m` | — | Branch length. |
| `radius` | `number` | `0.6` | ≥ 0 | — | — | Trunk base radius in metres, or child radius as a share of parent radius. |
| `sections` | `number` | `4` | 1 … 32 | — | — | Segments along the branch length. |
| `segments` | `number` | `4` | 3 … 32 | — | — | Radial sides around the branch. |
| `taper` | `number` | `0.6` | 0 … 1 | `ratio` | — | Radius loss towards branch tip (0..1). |
| `gnarliness` | `number` | `0.1` | 0 … 2 | — | — | Random angular wander per section (radians). |
| `twist` | `number` | `0` | -3.1416 … 3.1416 | — | — | Axial twist per section (radians). |
| `children` | `number` | `3` | ≤ 64 | — | — | Child branches spawned per parent branch. |
| `angle` | `number` | `50` | 0 … 175 | `deg` | — | Branching angle from parent. |
| `start` | `number` | `0.3` | 0 … 1 | `ratio` | — | Fractional distance along parent where children begin (0..1). |

```ogt
struct PlantLevelDesc {
  doc "Growth parameters for one branch level (level 0 is the trunk)."
  desc true
  field length f32 "Branch length." {
    default 1
    min 0
    unit m
  }
  field radius f32 "Trunk base radius in metres, or child radius as a share of parent radius." {
    default 0.6
    min 0
  }
  field sections u32 "Segments along the branch length." {
    default 4
    min 1
    max 32
  }
  field segments u32 "Radial sides around the branch." {
    default 4
    min 3
    max 32
  }
  field taper f32 "Radius loss towards branch tip (0..1)." {
    default 0.6
    min 0
    max 1
    unit ratio
  }
  field gnarliness f32 "Random angular wander per section (radians)." {
    default 0.1
    min 0
    max 2
  }
  field twist f32 "Axial twist per section (radians)." {
    min -3.1416
    max 3.1416
  }
  field children u32 "Child branches spawned per parent branch." {
    default 3
    max 64
  }
  field angle f32 "Branching angle from parent." {
    default 50
    min 0
    max 175
    unit deg
  }
  field start f32 "Fractional distance along parent where children begin (0..1)." {
    default 0.3
    min 0
    max 1
    unit ratio
  }
}
```

<sub>Module `procgen` line 595</sub>
