<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="record-TextInput"></a>

### `og.TextInput` (event)

Text typed or composed (an engine extension): the shell sends it from the page's `beforeinput` and composition events, the probe's `type:`, `compose:` and `commit:` from scripts. A text longer than 255 bytes arrives as several events in order. Key events still arrive for the keys; a text field types from these once a world has had one.

Token `og.TextInput`, registered under the schema name `og.TextInput`; its value type is `TextInput`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `text` | `string` | `""` | — | — | — | The text (whole characters). |
| `composing` | `boolean` | `false` | — | — | — | An input method's composition so far, not committed: it replaces the composition before it (empty ends it). False: committed text to insert. |
| `more` | `boolean` | `false` | — | — | — | Continues the composition of the event before it (a composition longer than one event). |

```ogt
event TextInput {
  doc "Text typed or composed (an engine extension): the shell sends it from the page's `beforeinput` and composition events, the probe's `type:`, `compose:` and `commit:` from scripts. A text longer than 255 bytes arrives as several events in order. Key events still arrive for the keys; a text field types from these once a world has had one."
  field text String "The text (whole characters)."
  field composing bool "An input method's composition so far, not committed: it replaces the composition before it (empty ends it). False: committed text to insert."
  field more bool "Continues the composition of the event before it (a composition longer than one event)."
}
```

<sub>Module `input` line 1213</sub>
