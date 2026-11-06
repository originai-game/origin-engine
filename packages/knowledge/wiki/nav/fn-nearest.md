<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`nav`](../nav.md) · [index](../index.md)

<a id="fn-nearest"></a>

### `og.nav.nearest`

The point on a navmesh nearest a world point, within `extent` (half sizes).

```ts
og.nav.nearest(world: World, mesh: Navigation, point: Vector3, extent?: Vector3): Vector3 | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `mesh` | `Navigation` | no | The navmesh. |
| `point` | `Vector3` | no | A world point. |
| `extent` | `Vector3` | yes | The search box's half sizes; zero: 2 2 2. |

**Returns** `Vector3 | null`: Null when no polygon is within the box.

Binding symbol `og_nav_nearest` · publish builds: kept

```ogt
fn nearest {
  doc "The point on a navmesh nearest a world point, within `extent` (half sizes)."
  param world World "The world."
  param mesh Navigation "The navmesh."
  param point Vector3 "A world point."
  param extent Vector3 "The search box's half sizes; zero: 2 2 2." {
    optional true
  }
  returns Option.Vector3 "Null when no polygon is within the box."
}
```

<sub>`api.ogt` line 14111</sub>
