<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-ExpectReport"></a>

### `ExpectReport` (struct)

The expectations evaluated since the last call.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `passed` | `number` | `0` | — | — | — | Those that held. |
| `failed` | `number` | `0` | — | — | — | Those that did not. |
| `unchecked` | `number` | `0` | — | — | — | Those that needed a renderer. |
| `results` | `readonly ExpectResult[]` | `[]` | — | — | — | Each, in order. |

```ogt
struct ExpectReport {
  doc "The expectations evaluated since the last call."
  field passed u32 "Those that held."
  field failed u32 "Those that did not."
  field unchecked u32 "Those that needed a renderer."
  field results List.ExpectResult "Each, in order."
}
```

<sub>`api.ogt` line 24571</sub>
