<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`nav`](../nav.md) · [index](../index.md)

<a id="fn-flowAt"></a>

### `og.nav.flowAt`

The direction a flow field gives at a point (unit length, or zero at a target or where nothing is reachable).

```ts
og.nav.flowAt(world: World, field: Navigation, point: Vector3): Vector3
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `field` | `Navigation` | no | The flow field. |
| `point` | `Vector3` | no | A world point. |

**Returns** `Vector3`.

Binding symbol `og_nav_flow_at` · publish builds: kept

```ogt
fn flowAt {
  doc "The direction a flow field gives at a point (unit length, or zero at a target or where nothing is reachable)."
  param world World "The world."
  param field Navigation "The flow field."
  param point Vector3 "A world point."
  returns Vector3
}
```

<sub>`api.ogt` line 14100</sub>
