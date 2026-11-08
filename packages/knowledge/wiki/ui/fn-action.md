<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-action"></a>

### `og.ui.action`

The action fired this tick (a document's `on action` fires on the top layer that takes input).

```ts
og.ui.action(name: string): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | The action. |

**Returns** `boolean`.

Binding symbol `og_ui_action` · publish builds: kept

```ogt
fn action {
  doc "The action fired this tick (a document's `on action` fires on the top layer that takes input)."
  param name String "The action."
  returns bool
}
```

<sub>Module `ui` line 8194</sub>
