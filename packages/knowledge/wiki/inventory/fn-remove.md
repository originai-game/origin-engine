<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`inventory`](../inventory.md) · [index](../index.md)

<a id="fn-remove"></a>

### `og.inventory.remove`

Take `count` of a row from a container: from its stacks of that row from the last slot back, so the first slots stay full. All or nothing.

Also known as: consume item, take item, spend item

```ts
og.inventory.remove(world: World, container: Entity, item: TableRow, count: number): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `container` | `Entity` | no | The og.Container. |
| `item` | `TableRow` | no | The row. |
| `count` | `number` | no | How many; 0 takes nothing. |

**Returns** `Result<void>`: Refused, changing nothing, when the container holds fewer.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn remove {
  doc "Take `count` of a row from a container: from its stacks of that row from the last slot back, so the first slots stay full. All or nothing."
  aliases "consume item, take item, spend item"
  host false
  param world World "The world."
  param container Entity "The og.Container."
  param item TableRow "The row."
  param count u32 "How many; 0 takes nothing."
  returns Result.Void "Refused, changing nothing, when the container holds fewer."
}
```

<sub>Module `inventory` line 125</sub>
