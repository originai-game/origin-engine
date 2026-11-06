<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ai`](../ai.md) · [index](../index.md)

<a id="record-AiChanged"></a>

### `og.AiChanged` (event)

An agent's state or running node changed (systems read it the tick after).

Token `og.AiChanged`, registered under the schema name `og.AiChanged`; its value type is `AiChanged`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The agent. |
| `state` | `number` | `0` | — | — | — | Its state now. |
| `node` | `number` | `0` | — | — | — | The node it runs now. |
| `status` | `number` | `0` | — | — | — | AI_RUNNING, AI_SUCCESS or AI_FAILURE. |

```ogt
event AiChanged {
  doc "An agent's state or running node changed (systems read it the tick after)."
  field entity Entity "The agent."
  field state u32 "Its state now."
  field node u32 "The node it runs now."
  field status u32 "AI_RUNNING, AI_SUCCESS or AI_FAILURE."
}
```

<sub>Module `ai` line 131</sub>
