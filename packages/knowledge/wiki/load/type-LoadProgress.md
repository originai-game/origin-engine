<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="type-LoadProgress"></a>

### `LoadProgress` (struct)

How far a load has arrived (og.load.progress).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `status` | `LoadStatus` | `"pending"` | — | — | — | Where it is. |
| `received` | `number` | `0` | — | `bytes` | — | Bytes received so far of what it fetches (a bundle: its manifest). |
| `bytes` | `number` | `0` | — | `bytes` | — | Bytes it fetches in all, once the host knows (0 before the first response). |
| `fraction` | `number` | `0` | 0 … 1 | — | — | received over bytes, 0 to 1; 1 once ready, 0 when failed or not known yet. |

```ogt
struct LoadProgress {
  doc "How far a load has arrived (og.load.progress)."
  field status LoadStatus "Where it is."
  field received f64 "Bytes received so far of what it fetches (a bundle: its manifest)." {
    unit bytes
  }
  field bytes f64 "Bytes it fetches in all, once the host knows (0 before the first response)." {
    unit bytes
  }
  field fraction f32 "received over bytes, 0 to 1; 1 once ready, 0 when failed or not known yet." {
    min 0
    max 1
  }
}
```

<sub>Module `load` line 229</sub>
