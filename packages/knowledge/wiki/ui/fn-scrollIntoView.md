<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-scrollIntoView"></a>

### `og.ui.scrollIntoView`

Scroll the scroll container around a control (an interactive or focusable element, by its full path) so the control is in its view, by where both were in the last frame: by the least that shows it whole, or centred in the view (a saga map opening on the current level, a list on its chosen row). The offset stays in the container's range and stops a fling. Nothing for a path the last frame had no control at or that is in no scroll container.

```ts
og.ui.scrollIntoView(path: string, center: boolean): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `path` | `string` | no | The control's full path (og.ui.lastPath, og.ui.focus). |
| `center` | `boolean` | no | Centre it in the view, as far as the range allows; false scrolls by the least that shows it. |

**Returns** `void`.

Binding symbol `og_ui_scroll_into_view` · publish builds: kept

```ogt
fn scrollIntoView {
  doc "Scroll the scroll container around a control (an interactive or focusable element, by its full path) so the control is in its view, by where both were in the last frame: by the least that shows it whole, or centred in the view (a saga map opening on the current level, a list on its chosen row). The offset stays in the container's range and stops a fling. Nothing for a path the last frame had no control at or that is in no scroll container."
  param path String "The control's full path (og.ui.lastPath, og.ui.focus)."
  param center bool "Centre it in the view, as far as the range allows; false scrolls by the least that shows it."
  returns Void
}
```

<sub>Module `ui` line 881</sub>
