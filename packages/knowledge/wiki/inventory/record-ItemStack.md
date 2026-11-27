<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`inventory`](../inventory.md) · [index](../index.md)

<a id="record-ItemStack"></a>

### `og.ItemStack` (component)

Some of one table row: a slot of an og.Container (a child of it) or, with og.Pickup, a pickup lying in the world. An empty slot has a count of 0 and no row.

Also known as: item stack, slot, item count, stack size, held item, item instance

Token `og.ItemStack`, registered under the schema name `og.ItemStack`; its value type is `ItemStack`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `item` | `TableRow` | `0` | — | — | — | The row of the item table (og.table) it holds; 0 with a count of 0. |
| `count` | `number` | `0` | — | — | — | How many it holds; 0 is an empty slot. |
| `slot` | `number` | `0` | — | — | read only | Written: its slot in its container, from 0. |
| `container` | `Entity` | `0` | — | — | read only | Written: the og.Container it is a slot of (its parent); 0 for a pickup. |

```ogt
component ItemStack {
  aliases "item stack, slot, item count, stack size, held item, item instance"
  doc "Some of one table row: a slot of an og.Container (a child of it) or, with og.Pickup, a pickup lying in the world. An empty slot has a count of 0 and no row."
  field item TableRow "The row of the item table (og.table) it holds; 0 with a count of 0."
  field count u32 "How many it holds; 0 is an empty slot."
  field slot u32 "Written: its slot in its container, from 0." {
    readonly true
  }
  field container Entity "Written: the og.Container it is a slot of (its parent); 0 for a pickup." {
    readonly true
  }
}
```

<sub>Module `inventory` line 15</sub>
