<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="record-A11yAction"></a>

### `og.A11yAction` (event)

A screen reader's request on a UI control (an engine extension): the web shell's accessibility mirror or a native window sends it; the UI takes it at the start of the next frame. A path longer than 255 bytes is refused.

Token `og.A11yAction`, registered under the schema name `og.A11yAction`; its value type is `A11yAction`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `action` | `A11yActionKind` | `"focus"` | — | — | — | What is asked. |
| `path` | `string` | `""` | — | — | — | The node's id in the UI's accessibility tree, as text. |

```ogt
event A11yAction {
  doc "A screen reader's request on a UI control (an engine extension): the web shell's accessibility mirror or a native window sends it; the UI takes it at the start of the next frame. A path longer than 255 bytes is refused."
  field action A11yActionKind "What is asked."
  field path String "The node's id in the UI's accessibility tree, as text."
}
```

<sub>Module `input` line 1277</sub>
