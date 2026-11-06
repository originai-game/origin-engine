<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`nav`](../nav.md) · [index](../index.md)

<a id="fn-setArea"></a>

### `og.nav.setArea`

Set, move or (with no desc) remove a run-time area on a navmesh (water, mud, a road): its tiles rebuild so no polygon mixes areas, and agents re-plan, weighing it by their og.NavFilter. World state.

```ts
og.nav.setArea(world: World, mesh: Navigation, id: number, desc?: NavAreaDesc): Result<NavRebuild>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `mesh` | `Navigation` | no | The navmesh. |
| `id` | `number` | no | The area's id, the game's own. |
| `desc` | `NavAreaDesc` | yes | The shape and the area; omitted removes it. |

**Returns** `Result<NavRebuild>`: Refused for an unknown navmesh or an area past 63.

Binding symbol `og_nav_set_area` · publish builds: kept

```ogt
fn setArea {
  doc "Set, move or (with no desc) remove a run-time area on a navmesh (water, mud, a road): its tiles rebuild so no polygon mixes areas, and agents re-plan, weighing it by their og.NavFilter. World state."
  param world World "The world."
  param mesh Navigation "The navmesh."
  param id u32 "The area's id, the game's own."
  param desc NavAreaDesc "The shape and the area; omitted removes it." {
    optional true
  }
  returns Result.NavRebuild "Refused for an unknown navmesh or an area past 63."
}
```

<sub>`api.ogt` line 14073</sub>
