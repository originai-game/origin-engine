<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-actionButton"></a>

### `og.ui.actionButton`

A square touch action button, cornered by the style's control radius: pressed while held (jump, fire), clicked on a tap; at least 44 CSS px.

```ts
og.ui.actionButton(id: string, text: string, desc?: ActionButtonDesc): Response
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `text` | `string` | no | Its label (and what the probe reads). |
| `desc` | `ActionButtonDesc` | yes | Size, primary and icon. |

**Returns** `Response`.

Binding symbol `og_ui_action_button` · publish builds: kept

```ogt
fn actionButton {
  doc "A square touch action button, cornered by the style's control radius: pressed while held (jump, fire), clicked on a tap; at least 44 CSS px."
  param id String "Its id."
  param text String "Its label (and what the probe reads)."
  param desc ActionButtonDesc "Size, primary and icon."
  returns Response
}
```

<sub>Module `ui` line 1463</sub>
