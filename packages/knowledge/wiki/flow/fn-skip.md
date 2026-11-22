<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`flow`](../flow.md) · [index](../index.md)

<a id="fn-skip"></a>

### `og.flow.skip`

Skip ahead in the segment playing: to its window's open (flow.open) when that is ahead, else to its cut, which takes the pending choice, else the default. For a segment the player has seen; what was seen is the game's (guide flow §5). Systems may call it.

```ts
og.flow.skip(world: World, flow: Entity): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `flow` | `Entity` | no | The flow. |

**Returns** `Result<void>`: Refused for an entity that is no flow, or one that has ended.

Binding symbol `og_flow_skip` · publish builds: kept

```ogt
fn skip {
  doc "Skip ahead in the segment playing: to its window's open (flow.open) when that is ahead, else to its cut, which takes the pending choice, else the default. For a segment the player has seen; what was seen is the game's (guide flow §5). Systems may call it."
  param world World "The world."
  param flow Entity "The flow."
  returns Result.Void "Refused for an entity that is no flow, or one that has ended."
}
```

<sub>Module `flow` line 259</sub>
