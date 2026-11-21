<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="record-DialogueShown"></a>

### `og.DialogueShown` (event)

A conversation shows a new line (start the voice, reset the reveal). Sent before the game's systems, the tick after the call that moved it.

Token `og.DialogueShown`, registered under the schema name `og.DialogueShown`; its value type is `DialogueShown`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `conversation` | `number` | `0` | — | — | — | The conversation. |
| `seq` | `number` | `0` | — | — | — | The line's DialogueView.seq. |

```ogt
event DialogueShown {
  doc "A conversation shows a new line (start the voice, reset the reveal). Sent before the game's systems, the tick after the call that moved it."
  field conversation u32 "The conversation."
  field seq u32 "The line's DialogueView.seq."
}
```

<sub>Module `story` line 36</sub>
