<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tween`](../tween.md) · [index](../index.md)

<a id="type-StaggerDesc"></a>

### `StaggerDesc` (desc struct)

GSAP's stagger object: each target's start offset by its distance from `from`.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `each` | `number \| null` | `null` | — | `s` | — | Seconds between neighbours. |
| `amount` | `number \| null` | `null` | — | `s` | — | Seconds over all of them (wins over each). |
| `from` | `string \| number \| null` | `null` | — | — | — | `start` (null), `center`, `end`, `edges` (both ends first), `random` (a seeded shuffle), or a target's index. |
| `grid` | `readonly number[] \| null` | `null` | — | — | — | `[rows, columns]`: distances measured on that grid. |
| `ease` | `Ease` | `"none"` | — | — | — | Distributes the offsets through a curve. |

```ogt
struct StaggerDesc {
  doc "GSAP's stagger object: each target's start offset by its distance from `from`."
  desc true
  field each Option.f32 "Seconds between neighbours." {
    unit s
  }
  field amount Option.f32 "Seconds over all of them (wins over each)." {
    unit s
  }
  field from Option.OneOf.String.u32 "`start` (null), `center`, `end`, `edges` (both ends first), `random` (a seeded shuffle), or a target's index."
  field grid Option.List.u32 "`[rows, columns]`: distances measured on that grid."
  field ease Ease "Distributes the offsets through a curve." {
    default none
  }
}
```

<sub>Module `tween` line 168</sub>
