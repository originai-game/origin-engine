<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="type-TextEntry"></a>

### `TextEntry` (struct)

Typed or composed text (an engine extension of protocol 5): what the page's `beforeinput` inserted (a key's character, a paste, an input method's commit), or with `composing` an input method's composition so far. Raises TextInput events; the UI's focused text field takes them.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `text` | `string` | `""` | — | — | — | The text. |
| `composing` | `boolean` | `false` | — | — | optional | An input method's composition, not yet committed: it replaces the composition before it, and empty ends it. Omitted is false: committed text. |

```ogt
struct TextEntry {
  doc "Typed or composed text (an engine extension of protocol 5): what the page's `beforeinput` inserted (a key's character, a paste, an input method's commit), or with `composing` an input method's composition so far. Raises TextInput events; the UI's focused text field takes them."
  field text String "The text."
  field composing bool "An input method's composition, not yet committed: it replaces the composition before it, and empty ends it. Omitted is false: committed text." {
    optional true
  }
}
```

<sub>Module `input` line 729</sub>
