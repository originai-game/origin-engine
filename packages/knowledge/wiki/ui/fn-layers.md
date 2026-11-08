<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-layers"></a>

### `og.ui.layers`

The layer stack, bottom first.

```ts
og.ui.layers(world: World): readonly UiLayer[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `readonly UiLayer[]`.

Binding symbol `og_ui_layers` · publish builds: kept

```ogt
fn layers {
  doc "The layer stack, bottom first."
  param world World "The world."
  returns List.UiLayer
}
```

<sub>Module `ui` line 3693</sub>
