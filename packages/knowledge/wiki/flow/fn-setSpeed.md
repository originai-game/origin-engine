<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`flow`](../flow.md) · [index](../index.md)

<a id="fn-setSpeed"></a>

### `og.flow.setSpeed`

Set playback speed (0.25 to 4.0) for the flow's active segment (fast-forward / slow-motion). Systems may call it.

```ts
og.flow.setSpeed(world: World, flow: Entity, speed: number): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `flow` | `Entity` | no | The flow. |
| `speed` | `number` | no | Playback speed multiplier (0.25 to 4.0). |

**Returns** `Result<void>`: Refused for an entity that is no flow, or a speed outside 0.25 to 4.0.

Binding symbol `og_flow_set_speed` · publish builds: kept

```ogt
fn setSpeed {
  doc "Set playback speed (0.25 to 4.0) for the flow's active segment (fast-forward / slow-motion). Systems may call it."
  param world World "The world."
  param flow Entity "The flow."
  param speed f32 "Playback speed multiplier (0.25 to 4.0)."
  returns Result.Void "Refused for an entity that is no flow, or a speed outside 0.25 to 4.0."
}
```

<sub>Module `flow` line 373</sub>
