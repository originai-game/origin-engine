<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-gesture"></a>

### `og.ui.gesture`

What the drag or pinch on an element is doing.

```ts
og.ui.gesture(path: string): GestureInfo
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `path` | `string` | no | The element's full path. |

**Returns** `GestureInfo`.

Binding symbol `og_ui_gesture` · publish builds: kept

```ogt
fn gesture {
  doc "What the drag or pinch on an element is doing."
  param path String "The element's full path."
  returns GestureInfo
}
```

<sub>Module `ui` line 5061</sub>
