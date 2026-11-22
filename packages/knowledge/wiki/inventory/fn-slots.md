<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`inventory`](../inventory.md) · [index](../index.md)

<a id="fn-slots"></a>

### `og.inventory.slots`

A container's slots in slot order (og.ItemStack entities, empty ones too): what a backpack screen lists. A system that calls this or another og.inventory function lists og.Container and og.ItemStack in its writes.

```ts
og.inventory.slots(world: World, container: Entity): readonly Entity[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `container` | `Entity` | no | The og.Container. |

**Returns** `readonly Entity[]`: Empty for an entity without og.Container, or before its slots are placed.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn slots {
  doc "A container's slots in slot order (og.ItemStack entities, empty ones too): what a backpack screen lists. A system that calls this or another og.inventory function lists og.Container and og.ItemStack in its writes."
  host false
  param world World "The world."
  param container Entity "The og.Container."
  returns List.Entity "Empty for an entity without og.Container, or before its slots are placed."
}
```

<sub>Module `inventory` line 87</sub>
