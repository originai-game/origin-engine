<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`inventory`](../inventory.md) · [index](../index.md)

<a id="fn-canFit"></a>

### `og.inventory.canFit`

Whether `count` of a row fit in a container now: onto its stacks of that row up to the row's stack limit, then into its empty slots.

```ts
og.inventory.canFit(world: World, container: Entity, item: TableRow, count: number): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `container` | `Entity` | no | The og.Container. |
| `item` | `TableRow` | no | The row. |
| `count` | `number` | no | How many. |

**Returns** `boolean`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn canFit {
  doc "Whether `count` of a row fit in a container now: onto its stacks of that row up to the row's stack limit, then into its empty slots."
  host false
  param world World "The world."
  param container Entity "The og.Container."
  param item TableRow "The row."
  param count u32 "How many."
  returns bool
}
```

<sub>Module `inventory` line 104</sub>
