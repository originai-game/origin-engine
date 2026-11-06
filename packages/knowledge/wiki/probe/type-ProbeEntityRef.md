<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-ProbeEntityRef"></a>

### `ProbeEntityRef` (struct)

One entity the probe publishes in state().

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `id` | `string` | `""` | — | — | — | Its probe id. |
| `label` | `string` | `""` | — | — | optional | A person-facing name. |

```ogt
struct ProbeEntityRef {
  doc "One entity the probe publishes in state()."
  field id String "Its probe id."
  field label String "A person-facing name." {
    optional true
  }
}
```

<sub>`api.ogt` line 24100</sub>
