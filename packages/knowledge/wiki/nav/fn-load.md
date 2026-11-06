<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`nav`](../nav.md) · [index](../index.md)

<a id="fn-load"></a>

### `og.nav.load`

A cooked navmesh: an `.ogt` of `type navmesh` (the agent, the cell sizes, the bounds and the geometry: a scene's static colliders and models, glTF models, terrains, boxes) baked by og cook.

```ts
og.nav.load(world: World, path: string, profile?: string): Result<NavMeshInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `path` | `string` | no | The navmesh asset, such as `levels/arena.nav.ogt`. |
| `profile` | `string` | yes | The agent profile, an `agent <name>` block's name: one mesh per body size (a rat under the table, an ogre round it). Omitted: the asset's first agent block (its unnamed one, if any). |

**Returns** `Result<NavMeshInfo>`: Refused for a path that is not a cooked navmesh, or a profile it has no agent block for (naming the ones it has).

Binding symbol `og_nav_load` · publish builds: kept

```ogt
fn load {
  doc "A cooked navmesh: an `.ogt` of `type navmesh` (the agent, the cell sizes, the bounds and the geometry: a scene's static colliders and models, glTF models, terrains, boxes) baked by og cook."
  param world World "The world."
  param path String "The navmesh asset, such as `levels/arena.nav.ogt`."
  param profile String "The agent profile, an `agent <name>` block's name: one mesh per body size (a rat under the table, an ogre round it). Omitted: the asset's first agent block (its unnamed one, if any)." {
    optional true
  }
  returns Result.NavMeshInfo "Refused for a path that is not a cooked navmesh, or a profile it has no agent block for (naming the ones it has)."
}
```

<sub>`api.ogt` line 14101</sub>
