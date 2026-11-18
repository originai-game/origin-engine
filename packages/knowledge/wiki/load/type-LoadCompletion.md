<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="type-LoadCompletion"></a>

### `LoadCompletion` (struct)

When a host-timed load finished: an uncooked load put in place or a runtime bundle opened. The journal a replay applies again (og.load.completions).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `string` | `""` | — | — | — | `uncooked` (og.load.imageData, gltfData, environmentData) or `bundle` (og.load.bundle of another bundle). A journal without kinds holds uncooked loads. |
| `request` | `number` | `0` | — | — | — | The load's number within its kind: 0 for the world's first, then one more for each. |
| `tick` | `number` | `0` | — | `ticks` | — | The world's tick (world.tick) at the boundary it was put in place: the systems of that tick and every later one see og.load.status ready or failed. |
| `ok` | `boolean` | `false` | — | — | — | Whether it was ready (else failed). |

```ogt
struct LoadCompletion {
  doc "When a host-timed load finished: an uncooked load put in place or a runtime bundle opened. The journal a replay applies again (og.load.completions)."
  field kind String "`uncooked` (og.load.imageData, gltfData, environmentData) or `bundle` (og.load.bundle of another bundle). A journal without kinds holds uncooked loads."
  field request u32 "The load's number within its kind: 0 for the world's first, then one more for each."
  field tick f64 "The world's tick (world.tick) at the boundary it was put in place: the systems of that tick and every later one see og.load.status ready or failed." {
    unit ticks
  }
  field ok bool "Whether it was ready (else failed)."
}
```

<sub>Module `load` line 590</sub>
