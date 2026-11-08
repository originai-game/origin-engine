<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-focus"></a>

### `og.ui.focus`

The path of the control pad pad's focus is on in the last frame (pad 0: the shared focus of the pointer, keyboard and pad 0), as touch_move_to resolves it; empty when it is on none.

```ts
og.ui.focus(pad: number): string
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `pad` | `number` | no | The pad, 0 to 3; past 3 answers empty. |

**Returns** `string`.

Binding symbol `og_ui_focus` · publish builds: kept

```ogt
fn focus {
  doc "The path of the control pad pad's focus is on in the last frame (pad 0: the shared focus of the pointer, keyboard and pad 0), as touch_move_to resolves it; empty when it is on none."
  param pad u32 "The pad, 0 to 3; past 3 answers empty."
  returns String
}
```

<sub>Module `ui` line 1214</sub>
