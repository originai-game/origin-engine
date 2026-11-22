<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`inventory`](../inventory.md) · [index](../index.md)

<a id="fn-add"></a>

### `og.inventory.add`

Put `count` of a row in a container: onto its stacks of that row in slot order up to the stack limit, then into its empty slots in order. All or nothing.

Also known as: give item, add to inventory, grant item

```ts
og.inventory.add(world: World, container: Entity, item: TableRow, count: number): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `container` | `Entity` | no | The og.Container. |
| `item` | `TableRow` | no | The row. |
| `count` | `number` | no | How many; 0 adds nothing. |

**Returns** `Result<void>`: Refused, changing nothing, when they do not fit, for a row no loaded table has, or inside a system for a container whose slots are not placed yet (the tick it appeared), naming why.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn add {
  doc "Put `count` of a row in a container: onto its stacks of that row in slot order up to the stack limit, then into its empty slots in order. All or nothing."
  aliases "give item, add to inventory, grant item"
  host false
  param world World "The world."
  param container Entity "The og.Container."
  param item TableRow "The row."
  param count u32 "How many; 0 adds nothing."
  returns Result.Void "Refused, changing nothing, when they do not fit, for a row no loaded table has, or inside a system for a container whose slots are not placed yet (the tick it appeared), naming why."
}
```

<sub>Module `inventory` line 114</sub>
