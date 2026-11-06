<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`nav`](../nav.md) · [index](../index.md)

<a id="fn-setLinkOpen"></a>

### `og.nav.setLinkOpen`

Open or close an off-mesh link (a door): a closed one is never planned across, and agents on the mesh re-plan. World state.

Also known as: door, gate, open or close a passage, off-mesh link, Unity NavMeshLink, Godot NavigationLink

```ts
og.nav.setLinkOpen(world: World, mesh: Navigation, id: number, open: boolean): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `mesh` | `Navigation` | no | The navmesh. |
| `id` | `number` | no | The link's id. |
| `open` | `boolean` | no | Open. |

**Returns** `Result<void>`: Refused for an unknown navmesh or link.

Binding symbol `og_nav_set_link_open` · publish builds: kept

```ogt
fn setLinkOpen {
  doc "Open or close an off-mesh link (a door): a closed one is never planned across, and agents on the mesh re-plan. World state."
  aliases "door, gate, open or close a passage, off-mesh link, Unity NavMeshLink, Godot NavigationLink"
  param world World "The world."
  param mesh Navigation "The navmesh."
  param id u32 "The link's id."
  param open bool "Open."
  returns Result.Void "Refused for an unknown navmesh or link."
}
```

<sub>`api.ogt` line 14147</sub>
