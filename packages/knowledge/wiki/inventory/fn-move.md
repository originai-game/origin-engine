<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`inventory`](../inventory.md) · [index](../index.md)

<a id="fn-move"></a>

### `og.inventory.move`

Move `count` from a slot into another container (or its own): onto that container's stacks of the row, then into its empty slots. All or nothing: both containers stay as they were when they do not fit.

Also known as: transfer item, move to chest, drag item, deposit, withdraw

```ts
og.inventory.move(world: World, slot: Entity, to: Entity, count: number): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `slot` | `Entity` | no | The og.ItemStack slot taken from. |
| `to` | `Entity` | no | The og.Container put into. |
| `count` | `number` | no | How many; 0 moves nothing. |

**Returns** `Result<void>`: Refused, changing nothing, for more than the slot holds or when they do not fit.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn move {
  doc "Move `count` from a slot into another container (or its own): onto that container's stacks of the row, then into its empty slots. All or nothing: both containers stay as they were when they do not fit."
  aliases "transfer item, move to chest, drag item, deposit, withdraw"
  host false
  param world World "The world."
  param slot Entity "The og.ItemStack slot taken from."
  param to Entity "The og.Container put into."
  param count u32 "How many; 0 moves nothing."
  returns Result.Void "Refused, changing nothing, for more than the slot holds or when they do not fit."
}
```

<sub>Module `inventory` line 130</sub>
