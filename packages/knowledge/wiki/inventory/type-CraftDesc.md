<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`inventory`](../inventory.md) · [index](../index.md)

<a id="type-CraftDesc"></a>

### `CraftDesc` (struct)

What og.inventory.craft takes and gives.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `inputs` | `readonly ItemCount[]` | `[]` | — | — | — | What it takes from the container. |
| `outputs` | `readonly ItemCount[]` | `[]` | — | — | — | What it puts in the container once the inputs are taken. |

```ogt
struct CraftDesc {
  doc "What og.inventory.craft takes and gives."
  field inputs List.ItemCount "What it takes from the container."
  field outputs List.ItemCount "What it puts in the container once the inputs are taken."
}
```

<sub>Module `inventory` line 75</sub>
