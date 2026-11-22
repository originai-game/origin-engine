<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`play`](../play.md) · [index](../index.md)

<a id="type-Candidate"></a>

### `Candidate` (struct)

An entity og.play.candidates found.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The entity. |
| `distance` | `number` | `0` | — | `m` | — | From the point to its world position. |
| `angle` | `number` | `0` | — | `rad` | — | Between the forward direction and the direction to it, 0 to π. |
| `visible` | `boolean` | `false` | — | — | — | No collider of the filter stands in between (true when sight was not asked). |

```ogt
struct Candidate {
  doc "An entity og.play.candidates found."
  field entity Entity "The entity."
  field distance f32 "From the point to its world position." {
    unit m
  }
  field angle f32 "Between the forward direction and the direction to it, 0 to π." {
    unit rad
  }
  field visible bool "No collider of the filter stands in between (true when sight was not asked)."
}
```

<sub>Module `play` line 129</sub>
