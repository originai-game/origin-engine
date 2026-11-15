<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-actionFired"></a>

### `og.ui.actionFired`

The action fired this tick: on its press, its release or once its hold is reached, as its trigger says (a document's `on action` fires on the top layer that takes input). A bool, where og.input.action answers an action's whole state.

```ts
og.ui.actionFired(name: string): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | The action. |

**Returns** `boolean`.

Binding symbol `og_ui_action_fired` · publish builds: kept

```ogt
fn actionFired {
  doc "The action fired this tick: on its press, its release or once its hold is reached, as its trigger says (a document's `on action` fires on the top layer that takes input). A bool, where og.input.action answers an action's whole state."
  param name String "The action."
  returns bool
}
```

<sub>Module `ui` line 1786</sub>
