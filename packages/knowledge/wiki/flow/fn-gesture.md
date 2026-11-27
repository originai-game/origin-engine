<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`flow`](../flow.md) · [index](../index.md)

<a id="fn-gesture"></a>

### `og.flow.gesture`

Feed a deterministic swipe or drag gesture into active QTEs (mobile touch support). Systems may call it.

```ts
og.flow.gesture(world: World, flow: Entity, desc?: FlowGestureDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `flow` | `Entity` | no | The flow. |
| `desc` | `FlowGestureDesc` | yes | The gesture kind (`swipe` or `drag`), direction (`up`, `down`, `left`, `right`) and drag progress. |

**Returns** `Result<void>`: Refused for an entity that is no flow or an invalid gesture.

Binding symbol `og_flow_gesture` · publish builds: kept

```ogt
fn gesture {
  doc "Feed a deterministic swipe or drag gesture into active QTEs (mobile touch support). Systems may call it."
  param world World "The world."
  param flow Entity "The flow."
  param desc FlowGestureDesc "The gesture kind (`swipe` or `drag`), direction (`up`, `down`, `left`, `right`) and drag progress."
  returns Result.Void "Refused for an entity that is no flow or an invalid gesture."
}
```

<sub>Module `flow` line 397</sub>
