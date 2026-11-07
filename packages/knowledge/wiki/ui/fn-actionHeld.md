<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-actionHeld"></a>

### `og.ui.actionHeld`

A hold action is held past its hold, or its toggle is on.

```ts
og.ui.actionHeld(name: string): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | The action. |

**Returns** `boolean`.

Binding symbol `og_ui_action_held` · publish builds: kept

```ogt
fn actionHeld {
  doc "A hold action is held past its hold, or its toggle is on."
  param name String "The action."
  returns bool
}
```

<sub>Module `ui` line 8190</sub>
