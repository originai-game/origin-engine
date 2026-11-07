<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-setFocus"></a>

### `og.ui.setFocus`

Move the focus (the pointer's, the keys' and pad 0's) to the element at a full path when this frame ends, if a focusable control is there then: a component moving the focus with its choice (a radio group's arrow keys, a list opening at its chosen option, a keyboard's first key). The ring shows as navigation's does. The last call in a frame wins; a path with no focusable control leaves the focus where it is.

```ts
og.ui.setFocus(path: string): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `path` | `string` | no | The element's full path (og.ui.lastPath, og.ui.focus). |

**Returns** `void`.

Binding symbol `og_ui_set_focus` · publish builds: kept

```ogt
fn setFocus {
  doc "Move the focus (the pointer's, the keys' and pad 0's) to the element at a full path when this frame ends, if a focusable control is there then: a component moving the focus with its choice (a radio group's arrow keys, a list opening at its chosen option, a keyboard's first key). The ring shows as navigation's does. The last call in a frame wins; a path with no focusable control leaves the focus where it is."
  param path String "The element's full path (og.ui.lastPath, og.ui.focus)."
  returns Void
}
```

<sub>Module `ui` line 1232</sub>
