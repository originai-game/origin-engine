<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`inventory`](../inventory.md) · [index](../index.md)

<a id="fn-craft"></a>

### `og.inventory.craft`

Take the inputs from a container and put the outputs in it, as one move: refused, changing nothing, when an input is short or the outputs do not fit once the inputs are taken.

Also known as: craft item, recipe, combine items, cook, smelt, forge

```ts
og.inventory.craft(world: World, container: Entity, desc: CraftDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `container` | `Entity` | no | The og.Container. |
| `desc` | `CraftDesc` | no | The inputs and outputs. |

**Returns** `Result<void>`: Refused, naming the first input short or the output that does not fit.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn craft {
  doc "Take the inputs from a container and put the outputs in it, as one move: refused, changing nothing, when an input is short or the outputs do not fit once the inputs are taken."
  aliases "craft item, recipe, combine items, cook, smelt, forge"
  host false
  param world World "The world."
  param container Entity "The og.Container."
  param desc CraftDesc "The inputs and outputs."
  returns Result.Void "Refused, naming the first input short or the output that does not fit."
}
```

<sub>Module `inventory` line 167</sub>
