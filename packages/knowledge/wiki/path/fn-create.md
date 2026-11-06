<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`path`](../path.md) · [index](../index.md)

<a id="fn-create"></a>

### `og.path.create`

Make a path from points.

```ts
og.path.create(world: World, desc?: CurveDesc): Result<Path>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `CurveDesc` | yes | The points, kind, closed and tension. |

**Returns** `Result<Path>`: Refused for too few points for the kind (bezier needs 3n + 1 open, 3n closed) or a point that is not finite.

Binding symbol `og_path_create` · publish builds: kept

```ogt
fn create {
  doc "Make a path from points."
  param world World "The world."
  param desc CurveDesc "The points, kind, closed and tension."
  returns Result.Path "Refused for too few points for the kind (bezier needs 3n + 1 open, 3n closed) or a point that is not finite."
}
```

<sub>`api.ogt` line 26090</sub>
