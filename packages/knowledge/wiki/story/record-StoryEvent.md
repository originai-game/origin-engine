<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="record-StoryEvent"></a>

### `og.StoryEvent` (event)

A Yarn command such as `<<give_sword {$amount}>>`: open the shop, give the sword, play the cutscene. Sent the tick after.

Token `og.StoryEvent`, registered under the schema name `og.StoryEvent`; its value type is `StoryEvent`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `conversation` | `number` | `0` | — | — | — | The conversation that emitted it. |
| `name` | `string` | `""` | — | — | — | The command name. |
| `arg` | `string` | `""` | — | — | — | Its argument, evaluated and written as text (empty without one). |

```ogt
event StoryEvent {
  doc "A Yarn command such as `<<give_sword {$amount}>>`: open the shop, give the sword, play the cutscene. Sent the tick after."
  field conversation u32 "The conversation that emitted it."
  field name String "The command name."
  field arg String "Its argument, evaluated and written as text (empty without one)."
}
```

<sub>Module `story` line 54</sub>
