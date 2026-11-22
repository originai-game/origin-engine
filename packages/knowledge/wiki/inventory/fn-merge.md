<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`inventory`](../inventory.md) · [index](../index.md)

<a id="fn-merge"></a>

### `og.inventory.merge`

Move as much of one slot into another as fits: into an empty slot, or onto a stack of the same row up to its limit; the rest stays. The two may be in different containers.

Also known as: stack items, combine stacks, drop onto slot

```ts
og.inventory.merge(world: World, from: Entity, into: Entity): Result<number>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `from` | `Entity` | no | The og.ItemStack slot taken from. |
| `into` | `Entity` | no | The og.ItemStack slot put into. |

**Returns** `Result<number>`: How many moved (0 when the other slot is full or holds another row); refused for an entity that is not a slot.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn merge {
  doc "Move as much of one slot into another as fits: into an empty slot, or onto a stack of the same row up to its limit; the rest stays. The two may be in different containers."
  aliases "stack items, combine stacks, drop onto slot"
  host false
  param world World "The world."
  param from Entity "The og.ItemStack slot taken from."
  param into Entity "The og.ItemStack slot put into."
  returns Result.u32 "How many moved (0 when the other slot is full or holds another row); refused for an entity that is not a slot."
}
```

<sub>Module `inventory` line 157</sub>
