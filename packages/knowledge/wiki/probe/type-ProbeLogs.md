<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-ProbeLogs"></a>

### `ProbeLogs` (struct)

Recent probe logs and warnings (up to 512 entries).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entries` | `readonly ProbeLogEntry[]` | `[]` | — | — | — | Entries in chronological order. |
| `dropped` | `number` | `0` | — | — | — | Older entries evicted before this read. |

```ogt
struct ProbeLogs {
  doc "Recent probe logs and warnings (up to 512 entries)."
  field entries List.ProbeLogEntry "Entries in chronological order."
  field dropped u32 "Older entries evicted before this read."
}
```

<sub>Module `probe` line 798</sub>
