<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`inventory`](../inventory.md) · [index](../index.md)

<a id="fn-split"></a>

### `og.inventory.split`

Move `count` of a slot's stack into the first empty slot of its container.

Also known as: split stack, divide stack

```ts
og.inventory.split(world: World, slot: Entity, count: number): Result<Entity>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `slot` | `Entity` | no | The og.ItemStack slot. |
| `count` | `number` | no | How many leave it: at least 1, fewer than it holds. |

**Returns** `Result<Entity>`: The slot they went to; refused, changing nothing, with no empty slot or a count out of range.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn split {
  doc "Move `count` of a slot's stack into the first empty slot of its container."
  aliases "split stack, divide stack"
  host false
  param world World "The world."
  param slot Entity "The og.ItemStack slot."
  param count u32 "How many leave it: at least 1, fewer than it holds."
  returns Result.Entity "The slot they went to; refused, changing nothing, with no empty slot or a count out of range."
}
```

<sub>Module `inventory` line 141</sub>
