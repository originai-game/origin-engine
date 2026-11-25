<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="type-LoadCompletion"></a>

### `LoadCompletion` (struct)

When a host-timed load finished: an uncooked load put in place or a runtime bundle opened. The journal a replay applies again (og.load.completions).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `string` | `""` | — | — | — | `uncooked` (og.load.imageData, gltfData, environmentData), `bundle` (og.load.bundle of another bundle, og.load.pack) or `data` (og.load.data). A journal without kinds holds uncooked loads. |
| `request` | `number` | `0` | — | — | — | The load's number within its kind: 0 for the world's first, then one more for each. |
| `tick` | `number` | `0` | — | `ticks` | — | The world's tick (world.tick) at the boundary it was put in place: the systems of that tick and every later one see og.load.status ready or failed. |
| `ok` | `boolean` | `false` | — | — | — | Whether it was ready (else failed). |
| `integrity` | `string` | `""` | — | — | optional | For `data`: the integrity of the bytes it read (`sha384-<base64>`); for an og.load.pack `bundle`: its cooked manifest's. A replay requires it again; absent otherwise. |

```ogt
struct LoadCompletion {
  doc "When a host-timed load finished: an uncooked load put in place or a runtime bundle opened. The journal a replay applies again (og.load.completions)."
  field kind String "`uncooked` (og.load.imageData, gltfData, environmentData), `bundle` (og.load.bundle of another bundle, og.load.pack) or `data` (og.load.data). A journal without kinds holds uncooked loads."
  field request u32 "The load's number within its kind: 0 for the world's first, then one more for each."
  field tick f64 "The world's tick (world.tick) at the boundary it was put in place: the systems of that tick and every later one see og.load.status ready or failed." {
    unit ticks
  }
  field ok bool "Whether it was ready (else failed)."
  field integrity String "For `data`: the integrity of the bytes it read (`sha384-<base64>`); for an og.load.pack `bundle`: its cooked manifest's. A replay requires it again; absent otherwise." {
    optional true
  }
}
```

<sub>Module `load` line 643</sub>
