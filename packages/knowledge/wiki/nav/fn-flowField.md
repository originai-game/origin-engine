<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`nav`](../nav.md) · [index](../index.md)

<a id="fn-flowField"></a>

### `og.nav.flowField`

A flow field towards targets over a grid: one Dijkstra pass gives every cell a direction, so any number of agents (NavAgent.nav = the field) steer by a lookup. It rebuilds when the grid changes.

Also known as: crowd pathfinding, swarm, many units, RTS movement, vector field, flow map

```ts
og.nav.flowField(world: World, desc?: FlowDesc): Result<Navigation>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `FlowDesc` | yes | The grid and the targets. |

**Returns** `Result<Navigation>`: Refused for an unknown grid.

Binding symbol `og_nav_flow_field` · publish builds: kept

```ogt
fn flowField {
  doc "A flow field towards targets over a grid: one Dijkstra pass gives every cell a direction, so any number of agents (NavAgent.nav = the field) steer by a lookup. It rebuilds when the grid changes."
  aliases "crowd pathfinding, swarm, many units, RTS movement, vector field, flow map"
  param world World "The world."
  param desc FlowDesc "The grid and the targets."
  returns Result.Navigation "Refused for an unknown grid."
}
```

<sub>`api.ogt` line 14076</sub>
