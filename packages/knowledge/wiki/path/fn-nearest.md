<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`path`](../path.md) · [index](../index.md)

<a id="fn-nearest"></a>

### `og.path.nearest`

The point of the path nearest a point, with its parameter: snapping to a track, projecting a pointer onto a rail.

```ts
og.path.nearest(world: World, path: Path, point: Vector3): Result<PathNearest>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `path` | `Path` | no | The path. |
| `point` | `Vector3` | no | The point. |

**Returns** `Result<PathNearest>`: Refused for a dead path.

Binding symbol `og_path_nearest` · publish builds: kept

```ogt
fn nearest {
  doc "The point of the path nearest a point, with its parameter: snapping to a track, projecting a pointer onto a rail."
  param world World "The world."
  param path Path "The path."
  param point Vector3 "The point."
  returns Result.PathNearest "Refused for a dead path."
}
```

<sub>`api.ogt` line 26006</sub>
