<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`world`](../world.md) · [index](../index.md)

<a id="type-Neighbour"></a>

### `Neighbour` (struct)

One entity a spatial query found.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The entity. |
| `distance` | `number` | `0` | — | `m` | — | Its distance from the query's centre. |

```ogt
struct Neighbour {
  doc "One entity a spatial query found."
  field entity Entity "The entity."
  field distance f64 "Its distance from the query's centre." {
    unit m
  }
}
```

<sub>Module `world` line 208</sub>
