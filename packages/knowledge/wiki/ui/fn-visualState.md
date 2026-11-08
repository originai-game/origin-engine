<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-visualState"></a>

### `og.ui.visualState`

The state a group of an element is in (last tick); empty for none.

```ts
og.ui.visualState(path: string, group: string): string
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `path` | `string` | no | The element's full path. |
| `group` | `string` | no | The group. |

**Returns** `string`.

Binding symbol `og_ui_visual_state` · publish builds: kept

```ogt
fn visualState {
  doc "The state a group of an element is in (last tick); empty for none."
  param path String "The element's full path."
  param group String "The group."
  returns String
}
```

<sub>Module `ui` line 2633</sub>
