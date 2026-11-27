<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`inventory`](../inventory.md) · [index](../index.md)

<a id="record-Container"></a>

### `og.Container` (component)

Holds items in `slots` slot entities: og.systems.inventory places them as its children (og.Parent), each an og.ItemStack numbered by its slot, on the tick it appears; a move outside a system (GameDesc.start, a command) places them itself. Its stacks are og.inventory.slots in slot order.

Also known as: inventory, backpack, bag, chest, stash, item container, slots, Unity inventory, storage box

Token `og.Container`, registered under the schema name `og.Container`; its value type is `Container`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `slots` | `number` | `0` | 0 … 1024 | — | tunable | How many stacks it holds; 0 holds nothing. Lowering it keeps the slots past it: they hold and give what they hold but take nothing new. |
| `stack` | `string` | `""` | — | — | `str(32)`: at most 32 bytes of UTF-8 | The field of the item table's component that holds a row's stack limit here (a u32 field, as `maxStack`), read with og.table.get; empty, or a row whose field is 0, fits none. |

```ogt
component Container {
  aliases "inventory, backpack, bag, chest, stash, item container, slots, Unity inventory, storage box"
  doc "Holds items in `slots` slot entities: og.systems.inventory places them as its children (og.Parent), each an og.ItemStack numbered by its slot, on the tick it appears; a move outside a system (GameDesc.start, a command) places them itself. Its stacks are og.inventory.slots in slot order."
  field slots u32 "How many stacks it holds; 0 holds nothing. Lowering it keeps the slots past it: they hold and give what they hold but take nothing new." {
    min 0
    max 1024
    tunable true
  }
  field stack str.32 "The field of the item table's component that holds a row's stack limit here (a u32 field, as `maxStack`), read with og.table.get; empty, or a row whose field is 0, fits none."
}
```

<sub>Module `inventory` line 4</sub>
