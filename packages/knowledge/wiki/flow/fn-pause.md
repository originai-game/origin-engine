<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`flow`](../flow.md) · [index](../index.md)

<a id="fn-pause"></a>

### `og.flow.pause`

Hold a flow on its frame (a pause menu); its window's time holds too. Systems may call it.

```ts
og.flow.pause(world: World, flow: Entity): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `flow` | `Entity` | no | The flow. |

**Returns** `Result<void>`: Refused for an entity that is no flow.

Binding symbol `og_flow_pause` · publish builds: kept

```ogt
fn pause {
  doc "Hold a flow on its frame (a pause menu); its window's time holds too. Systems may call it."
  param world World "The world."
  param flow Entity "The flow."
  returns Result.Void "Refused for an entity that is no flow."
}
```

<sub>Module `flow` line 252</sub>
