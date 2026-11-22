<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`flow`](../flow.md) · [index](../index.md)

<a id="fn-play"></a>

### `og.flow.play`

Play a paused flow from where it is. Systems may call it.

```ts
og.flow.play(world: World, flow: Entity): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `flow` | `Entity` | no | The flow. |

**Returns** `Result<void>`: Refused for an entity that is no flow.

Binding symbol `og_flow_play` · publish builds: kept

```ogt
fn play {
  doc "Play a paused flow from where it is. Systems may call it."
  param world World "The world."
  param flow Entity "The flow."
  returns Result.Void "Refused for an entity that is no flow."
}
```

<sub>Module `flow` line 245</sub>
