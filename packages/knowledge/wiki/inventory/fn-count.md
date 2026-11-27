<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`inventory`](../inventory.md) · [index](../index.md)

<a id="fn-count"></a>

### `og.inventory.count`

How many of a row a container holds, over all its stacks.

```ts
og.inventory.count(world: World, container: Entity, item: TableRow): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `container` | `Entity` | no | The og.Container. |
| `item` | `TableRow` | no | The row. |

**Returns** `number`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn count {
  doc "How many of a row a container holds, over all its stacks."
  host false
  param world World "The world."
  param container Entity "The og.Container."
  param item TableRow "The row."
  returns u32
}
```

<sub>Module `inventory` line 89</sub>
