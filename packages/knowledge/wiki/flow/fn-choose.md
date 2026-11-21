<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`flow`](../flow.md) · [index](../index.md)

<a id="fn-choose"></a>

### `og.flow.choose`

Take a choice while the window is open: recorded now (FlowView.window.pending; a later call replaces it) and taken at the cut, so the footage plays on to it. Systems may call it; og.flow applies it this tick.

```ts
og.flow.choose(world: World, flow: Entity, index: number): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `flow` | `Entity` | no | The flow. |
| `index` | `number` | no | ChoiceView.index. |

**Returns** `Result<void>`: Refused outside an open window, or for a choice that is hidden or disabled.

Binding symbol `og_flow_choose` · publish builds: kept

```ogt
fn choose {
  doc "Take a choice while the window is open: recorded now (FlowView.window.pending; a later call replaces it) and taken at the cut, so the footage plays on to it. Systems may call it; og.flow applies it this tick."
  param world World "The world."
  param flow Entity "The flow."
  param index u32 "ChoiceView.index."
  returns Result.Void "Refused outside an open window, or for a choice that is hidden or disabled."
}
```

<sub>Module `flow` line 221</sub>
