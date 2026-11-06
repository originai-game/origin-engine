<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`nav`](../nav.md) · [index](../index.md)

<a id="fn-cellAt"></a>

### `og.nav.cellAt`

The grid cell under a world point.

```ts
og.nav.cellAt(world: World, grid: Navigation, point: Vector3): NavCell
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `grid` | `Navigation` | no | The grid. |
| `point` | `Vector3` | no | A world point. |

**Returns** `NavCell`.

Binding symbol `og_nav_cell_at` · publish builds: kept

```ogt
fn cellAt {
  doc "The grid cell under a world point."
  param world World "The world."
  param grid Navigation "The grid."
  param point Vector3 "A world point."
  returns NavCell
}
```

<sub>`api.ogt` line 14068</sub>
