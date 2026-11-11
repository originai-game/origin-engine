<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`platform`](../platform.md) · [index](../index.md)

<a id="type-PurchaseDesc"></a>

### `PurchaseDesc` (desc struct)

A purchase to ask the player for.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `product` | `string` | `""` | — | — | — | The product's id. |
| `quantity` | `number` | `1` | 1 … 99 | — | — | How many, for a consumable. |

```ogt
struct PurchaseDesc {
  doc "A purchase to ask the player for."
  desc true
  field product String "The product's id."
  field quantity u32 "How many, for a consumable." {
    default 1
    min 1
    max 99
  }
}
```

<sub>Module `platform` line 262</sub>
