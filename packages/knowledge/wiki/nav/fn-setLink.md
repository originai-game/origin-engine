<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`nav`](../nav.md) · [index](../index.md)

<a id="fn-setLink"></a>

### `og.nav.setLink`

Set, move or (with no desc) remove an off-mesh link on a navmesh. Paths cross it where it is cheaper than walking; an agent that reaches its start crosses it (NAV_ON_LINK, og.NavLinkReached) and lands at its end. Links baked in the navmesh asset (`link` lines) have the ids they were given. World state.

```ts
og.nav.setLink(world: World, mesh: Navigation, id: number, desc?: NavLinkDesc): Result<boolean>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `mesh` | `Navigation` | no | The navmesh. |
| `id` | `number` | no | The link's id, the game's own. |
| `desc` | `NavLinkDesc` | yes | The link; omitted removes it. |

**Returns** `Result<boolean>`: Whether both ends are on the mesh (a link with an end off it is kept but never crossed). Refused for an unknown navmesh.

Binding symbol `og_nav_set_link` · publish builds: kept

```ogt
fn setLink {
  doc "Set, move or (with no desc) remove an off-mesh link on a navmesh. Paths cross it where it is cheaper than walking; an agent that reaches its start crosses it (NAV_ON_LINK, og.NavLinkReached) and lands at its end. Links baked in the navmesh asset (`link` lines) have the ids they were given. World state."
  param world World "The world."
  param mesh Navigation "The navmesh."
  param id u32 "The link's id, the game's own."
  param desc NavLinkDesc "The link; omitted removes it." {
    optional true
  }
  returns Result.bool "Whether both ends are on the mesh (a link with an end off it is kept but never crossed). Refused for an unknown navmesh."
}
```

<sub>`api.ogt` line 14150</sub>
