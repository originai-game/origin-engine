<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`sight`](../sight.md) · [index](../index.md)

<a id="fn-forget"></a>

### `og.sight.forget`

Forget what a team explored: its map is unexplored again but for what it sees this tick (a new round, a memory wipe).

Also known as: reset fog of war, clear explored, hide map again, unexplore

```ts
og.sight.forget(world: World, team: number): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `team` | `number` | no | The team (0 to 31). |

**Returns** `Result<void>`: Refused for a world without an og.SightMap or a team over 31.

Binding symbol `og_sight_forget` · publish builds: kept

```ogt
fn forget {
  doc "Forget what a team explored: its map is unexplored again but for what it sees this tick (a new round, a memory wipe)."
  aliases "reset fog of war, clear explored, hide map again, unexplore"
  param world World "The world."
  param team u32 "The team (0 to 31)."
  returns Result.Void "Refused for a world without an og.SightMap or a team over 31."
}
```

<sub>Module `sight` line 98</sub>
