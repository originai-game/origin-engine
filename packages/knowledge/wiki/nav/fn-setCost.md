<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`nav`](../nav.md) · [index](../index.md)

<a id="fn-setCost"></a>

### `og.nav.setCost`

Change a grid cell's cost (0 blocks it): agents on the grid re-plan, flow fields over it rebuild.

```ts
og.nav.setCost(world: World, grid: Navigation, x: number, y: number, cost: number): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `grid` | `Navigation` | no | The grid. |
| `x` | `number` | no | The column. |
| `y` | `number` | no | The row (0 is the bottom). |
| `cost` | `number` | no | 0 blocked, 1 normal, up to 255. |

**Returns** `Result<void>`: Refused for an unknown grid or a cell outside it.

Binding symbol `og_nav_set_cost` · publish builds: kept

```ogt
fn setCost {
  doc "Change a grid cell's cost (0 blocks it): agents on the grid re-plan, flow fields over it rebuild."
  param world World "The world."
  param grid Navigation "The grid."
  param x u32 "The column."
  param y u32 "The row (0 is the bottom)."
  param cost u32 "0 blocked, 1 normal, up to 255."
  returns Result.Void "Refused for an unknown grid or a cell outside it."
}
```

<sub>`api.ogt` line 13986</sub>
