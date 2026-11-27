<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`nav`](../nav.md) · [index](../index.md)

<a id="fn-load"></a>

### `og.nav.load`

A cooked file.navmesh TS declaration: agent profiles, voxel sizes, bounds and explicit geometry from a scene's static colliders and models, imported models, terrains or boxes, baked by og cook.

```ts
og.nav.load(world: World, path: string, profile?: string): Result<NavMeshInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `path` | `string` | no | The canonical navmesh content ID, such as `base/levels/arena`. |
| `profile` | `string` | yes | The agent profile's name: one mesh per body size (a rat under the table, an ogre round it). Omitted: the first profile, or the unnamed profile when present. |

**Returns** `Result<NavMeshInfo>`: Refused for a path that is not a cooked navmesh, or a profile it does not have (naming the ones it has).

Binding symbol `og_nav_load` · publish builds: kept

```ogt
fn load {
  doc "A cooked file.navmesh TS declaration: agent profiles, voxel sizes, bounds and explicit geometry from a scene's static colliders and models, imported models, terrains or boxes, baked by og cook."
  param world World "The world."
  param path String "The canonical navmesh content ID, such as `base/levels/arena`."
  param profile String "The agent profile's name: one mesh per body size (a rat under the table, an ogre round it). Omitted: the first profile, or the unnamed profile when present." {
    optional true
  }
  returns Result.NavMeshInfo "Refused for a path that is not a cooked navmesh, or a profile it does not have (naming the ones it has)."
}
```

<sub>Module `nav` line 569</sub>
