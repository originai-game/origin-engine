<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-DialogAction"></a>

### `DialogAction` (desc struct)

A button of a dialog.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `id` | `string` | `""` | — | — | — | The id answered when it is pressed; also its probe id. |
| `label` | `string` | `""` | — | — | — | What it says; empty is its id. |
| `variant` | `ButtonVariant` | `"secondary"` | — | — | — | Its weight: the primary sits last in a row and first in a phone's stack. |
| `disabled` | `boolean` | `false` | — | — | — | Shown but inert. |
| `autofocus` | `boolean` | `false` | — | — | — | It takes the focus when the dialog opens. |

```ogt
struct DialogAction {
  doc "A button of a dialog."
  desc true
  field id String "The id answered when it is pressed; also its probe id."
  field label String "What it says; empty is its id."
  field variant ButtonVariant "Its weight: the primary sits last in a row and first in a phone's stack."
  field disabled bool "Shown but inert."
  field autofocus bool "It takes the focus when the dialog opens."
}
```

<sub>Module `ui` line 1855</sub>
