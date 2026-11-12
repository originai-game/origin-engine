<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`terrain`](../terrain.md) · [index](../index.md)

<a id="type-TerrainEdit"></a>

### `TerrainEdit` (struct)

What an edit changed.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `samples` | `number` | `0` | — | — | — | Height or control-map samples it touched. |
| `minHeight` | `number` | `0` | — | `m` | — | The terrain's lowest point now. |
| `maxHeight` | `number` | `0` | — | `m` | — | Its highest now. |
| `stamp` | `number` | `0` | — | — | — | og.terrain.stamp: the stamp's id, for og.terrain.unstamp (from 1). 0 for every other edit. |

```ogt
struct TerrainEdit {
  doc "What an edit changed."
  field samples u32 "Height or control-map samples it touched."
  field minHeight f32 "The terrain's lowest point now." {
    unit m
  }
  field maxHeight f32 "Its highest now." {
    unit m
  }
  field stamp u32 "og.terrain.stamp: the stamp's id, for og.terrain.unstamp (from 1). 0 for every other edit."
}
```

<sub>Module `terrain` line 518</sub>
