<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-confirm"></a>

### `og.ui.confirm`

A confirmation dialog: the message, Cancel and the action (danger when it cannot be undone).

Also known as: confirmation dialog, are you sure, yes or no prompt, alert, quit prompt

```ts
og.ui.confirm(id: string, open: boolean, desc?: ConfirmationDesc): ConfirmResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `open` | `boolean` | no | Whether it is open. |
| `desc` | `ConfirmationDesc` | yes | Message and labels. |

**Returns** `ConfirmResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn confirm {
  doc "A confirmation dialog: the message, Cancel and the action (danger when it cannot be undone)."
  aliases "confirmation dialog, are you sure, yes or no prompt, alert, quit prompt"
  host false
  param id String "Its id."
  param open bool "Whether it is open."
  param desc ConfirmationDesc "Message and labels."
  returns ConfirmResult
}
```

<sub>Module `ui` line 2422</sub>
