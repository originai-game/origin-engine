<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="record-A11yAction"></a>

### `og.A11yAction` (event)

A screen reader's request on a UI control (an engine extension): the web shell sends it from its mirror of og.ui.a11y(); the UI takes it at the start of the next frame. A path longer than 255 bytes is refused.

Token `og.A11yAction`, registered under the schema name `og.A11yAction`; its value type is `A11yAction`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `action` | `A11yActionKind` | `"focus"` | — | — | — | What is asked. |
| `path` | `string` | `""` | — | — | — | The control's path, as og.ui.a11y() names it. |

```ogt
event A11yAction {
  doc "A screen reader's request on a UI control (an engine extension): the web shell sends it from its mirror of og.ui.a11y(); the UI takes it at the start of the next frame. A path longer than 255 bytes is refused."
  field action A11yActionKind "What is asked."
  field path String "The control's path, as og.ui.a11y() names it."
}
```

<sub>Module `input` line 1270</sub>
