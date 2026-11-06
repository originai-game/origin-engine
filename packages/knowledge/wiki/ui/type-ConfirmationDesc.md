<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ConfirmationDesc"></a>

### `ConfirmationDesc` (desc struct)

A confirmation: a message, Cancel and the action.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `title` | `string` | `""` | — | — | — | Its title. |
| `message` | `string` | `""` | — | — | — | What will happen, as rich text markup. |
| `confirmLabel` | `string` | `""` | — | — | — | The action's label; empty is Confirm. |
| `cancelLabel` | `string` | `""` | — | — | — | Cancel's label; empty is Cancel. |
| `danger` | `boolean` | `false` | — | — | — | The action cannot be undone: the danger variant, and Cancel takes the focus. |
| `tone` | `DialogTone` | `"neutral"` | — | — | — | Its tone when it is not danger. |
| `locked` | `boolean` | `false` | — | — | — | Back and the backdrop do not cancel. |

```ogt
struct ConfirmationDesc {
  doc "A confirmation: a message, Cancel and the action."
  desc true
  field title String "Its title."
  field message String "What will happen, as rich text markup."
  field confirmLabel String "The action's label; empty is Confirm."
  field cancelLabel String "Cancel's label; empty is Cancel."
  field danger bool "The action cannot be undone: the danger variant, and Cancel takes the focus."
  field tone DialogTone "Its tone when it is not danger."
  field locked bool "Back and the backdrop do not cancel."
}
```

<sub>Module `ui` line 2232</sub>
