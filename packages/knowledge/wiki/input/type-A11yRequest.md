<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="type-A11yRequest"></a>

### `A11yRequest` (struct)

A screen reader's request on a UI control (an engine extension of protocol 5): the web shell's accessibility mirror sends it when a reader focuses, activates or adjusts a mirrored node, and a native window sends it for the platform's reader. Raises an A11yAction event; the UI acts on it at the start of the next frame.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `action` | `A11yActionKind` | `"focus"` | — | — | — | What is asked. |
| `path` | `string` | `""` | — | — | — | The node's id in the UI's accessibility tree, as text. |

```ogt
struct A11yRequest {
  doc "A screen reader's request on a UI control (an engine extension of protocol 5): the web shell's accessibility mirror sends it when a reader focuses, activates or adjusts a mirrored node, and a native window sends it for the platform's reader. Raises an A11yAction event; the UI acts on it at the start of the next frame."
  field action A11yActionKind "What is asked."
  field path String "The node's id in the UI's accessibility tree, as text."
}
```

<sub>Module `input` line 751</sub>
