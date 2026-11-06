<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-SnapshotInfo"></a>

### `SnapshotInfo` (struct)

A snapshot of the whole world (components, resources, Rng states, pending events) kept in the running process.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `id` | `string` | `""` | — | — | — | The snapshot's id, for restore. |
| `tick` | `number` | `0` | — | — | — | The tick it was taken at. |
| `hash` | `string` | `""` | — | — | — | The world hash at that tick, hex: equal hashes mean equal worlds. |
| `bytes` | `number` | `0` | — | `bytes` | — | Its size. |

```ogt
struct SnapshotInfo {
  doc "A snapshot of the whole world (components, resources, Rng states, pending events) kept in the running process."
  field id String "The snapshot's id, for restore."
  field tick u32 "The tick it was taken at."
  field hash String "The world hash at that tick, hex: equal hashes mean equal worlds."
  field bytes u32 "Its size." {
    unit bytes
  }
}
```

<sub>`api.ogt` line 24313</sub>
