<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-navigate"></a>

### `og.ui.navigate`

Queue a navigation intent for the next tick: a gamepad og.Input does not read yet, a custom binding, a command. A move queued while the focus is on nothing puts the focus on the first control, whatever its direction, and no further: moves asked again before the focus lands (each tick until og.ui.focus answers a path) land on that same first control.

```ts
og.ui.navigate(world: World, intent: NavIntent): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `intent` | `NavIntent` | no | The intent. |

**Returns** `void`.

Binding symbol `og_ui_navigate` · publish builds: kept

```ogt
fn navigate {
  doc "Queue a navigation intent for the next tick: a gamepad og.Input does not read yet, a custom binding, a command. A move queued while the focus is on nothing puts the focus on the first control, whatever its direction, and no further: moves asked again before the focus lands (each tick until og.ui.focus answers a path) land on that same first control."
  param world World "The world."
  param intent NavIntent "The intent."
  returns Void
}
```

<sub>`api.ogt` line 17329</sub>
