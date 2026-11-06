<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`assert`](../assert.md) · [index](../index.md)

<a id="type-AssertBucket"></a>

### `AssertBucket` (struct)

One bucket of an each.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `key` | `string` | `""` | — | — | — | The bucket's key, as text. |
| `fired` | `boolean` | `false` | — | — | — | Whether its condition held once. |
| `firstTick` | `number \| null` | `null` | — | — | — | The first tick it held. |

```ogt
struct AssertBucket {
  doc "One bucket of an each."
  field key String "The bucket's key, as text."
  field fired bool "Whether its condition held once."
  field firstTick Option.u32 "The first tick it held."
}
```

<sub>`api.ogt` line 25874</sub>
