<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`nav`](../nav.md) · [index](../index.md)

<a id="fn-build"></a>

### `og.nav.build`

A navmesh over a flat floor with boxes carved out, built now: for procedural levels (a level with slopes and stairs is baked at cook time with og.nav.load).

```ts
og.nav.build(world: World, desc?: NavMeshDesc): Result<NavMeshInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `NavMeshDesc` | yes | The floor, the boxes and the agent. |

**Returns** `Result<NavMeshInfo>`: Refused for an empty floor.

Binding symbol `og_nav_build` · publish builds: kept

```ogt
fn build {
  doc "A navmesh over a flat floor with boxes carved out, built now: for procedural levels (a level with slopes and stairs is baked at cook time with og.nav.load)."
  param world World "The world."
  param desc NavMeshDesc "The floor, the boxes and the agent."
  returns Result.NavMeshInfo "Refused for an empty floor."
}
```

<sub>Module `nav` line 578</sub>
