<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="record-DialogueEnded"></a>

### `og.DialogueEnded` (event)

A conversation ended: its dialogue ran out, reached `end`, took a choice without `goto`, or og.story.stop stopped it. Sent the tick after.

Token `og.DialogueEnded`, registered under the schema name `og.DialogueEnded`; its value type is `DialogueEnded`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `conversation` | `number` | `0` | — | — | — | The conversation. |

```ogt
event DialogueEnded {
  doc "A conversation ended: its dialogue ran out, reached `end`, took a choice without `goto`, or og.story.stop stopped it. Sent the tick after."
  field conversation u32 "The conversation."
}
```

<sub>`api.ogt` line 14482</sub>
