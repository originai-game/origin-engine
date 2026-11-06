<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ai`](../ai.md) · [index](../index.md)

<a id="record-AiActionStopped"></a>

### `og.AiActionStopped` (event)

A game action stopped: the game finished it (og.ai.finish), or the behaviour moved on first (stop playing it). Read the tick after.

Token `og.AiActionStopped`, registered under the schema name `og.AiActionStopped`; its value type is `AiActionStopped`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The agent. |
| `node` | `number` | `0` | — | — | — | The action's node. |
| `name` | `string` | `""` | — | — | — | The action's name (at most 63 bytes). |
| `result` | `number` | `0` | — | — | — | AI_SUCCESS or AI_FAILURE, as og.ai.finish said, or AI_INTERRUPTED. |

```ogt
event AiActionStopped {
  doc "A game action stopped: the game finished it (og.ai.finish), or the behaviour moved on first (stop playing it). Read the tick after."
  field entity Entity "The agent."
  field node u32 "The action's node."
  field name String "The action's name (at most 63 bytes)."
  field result u32 "AI_SUCCESS or AI_FAILURE, as og.ai.finish said, or AI_INTERRUPTED."
}
```

<sub>Module `ai` line 147</sub>
