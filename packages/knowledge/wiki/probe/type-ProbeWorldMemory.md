<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-ProbeWorldMemory"></a>

### `ProbeWorldMemory` (struct)

The world's size.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entities` | `number` | `0` | — | — | — | Entities alive. |

```ogt
struct ProbeWorldMemory {
  doc "The world's size."
  field entities u32 "Entities alive."
}
```

<sub>`api.ogt` line 24243</sub>
