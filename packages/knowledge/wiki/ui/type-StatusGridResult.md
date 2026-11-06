<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-StatusGridResult"></a>

### `StatusGridResult` (struct)

What a status grid answered.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `hovered` | `string \| null` | `null` | — | — | — | The effect under the pointer or focus (show its tooltip). |
| `clicked` | `string \| null` | `null` | — | — | — | The effect pressed (cancel a buff). |

```ogt
struct StatusGridResult {
  doc "What a status grid answered."
  field hovered Option.String "The effect under the pointer or focus (show its tooltip)."
  field clicked Option.String "The effect pressed (cancel a buff)."
}
```

<sub>`api.ogt` line 21906</sub>
