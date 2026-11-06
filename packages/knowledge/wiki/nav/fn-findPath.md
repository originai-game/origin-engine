<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`nav`](../nav.md) · [index](../index.md)

<a id="fn-findPath"></a>

### `og.nav.findPath`

A path between two points on a grid or a navmesh, now: corners, length and whether it reaches the end.

Also known as: pathfinding, A*, route, shortest path, Unity NavMesh.CalculatePath, Godot get_simple_path

```ts
og.nav.findPath(world: World, desc?: PathDesc): Result<NavPath>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `PathDesc` | yes | The grid or navmesh and the two points. |

**Returns** `Result<NavPath>`: Refused for an unknown grid or navmesh, or when no path starts (an end off the grid or mesh).

Binding symbol `og_nav_find_path` · publish builds: kept

```ogt
fn findPath {
  doc "A path between two points on a grid or a navmesh, now: corners, length and whether it reaches the end."
  aliases "pathfinding, A*, route, shortest path, Unity NavMesh.CalculatePath, Godot get_simple_path"
  param world World "The world."
  param desc PathDesc "The grid or navmesh and the two points."
  returns Result.NavPath "Refused for an unknown grid or navmesh, or when no path starts (an end off the grid or mesh)."
}
```

<sub>`api.ogt` line 14068</sub>
