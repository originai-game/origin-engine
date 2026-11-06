<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ai`](../ai.md) · [index](../index.md)

<a id="record-AiActionStarted"></a>

### `og.AiActionStarted` (event)

A game action (`action <name> args` in a behaviour) started: play it (a reload, a door, a line of dialogue) and call og.ai.finish when it is over; until then the node runs. Read the tick after. Joins the world's types only once a behaviour with a game action is loaded.

Token `og.AiActionStarted`, registered under the schema name `og.AiActionStarted`; its value type is `AiActionStarted`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The agent. |
| `node` | `number` | `0` | — | — | — | The action's node, an index into BehaviourInfo.nodes. |
| `name` | `string` | `""` | — | — | — | The action's name (at most 63 bytes). |
| `args` | `string` | `""` | — | — | — | Its arguments as written, separated by spaces (at most 255 bytes; longer ones are refused when the behaviour is parsed). |

```ogt
event AiActionStarted {
  doc "A game action (`action <name> args` in a behaviour) started: play it (a reload, a door, a line of dialogue) and call og.ai.finish when it is over; until then the node runs. Read the tick after. Joins the world's types only once a behaviour with a game action is loaded."
  field entity Entity "The agent."
  field node u32 "The action's node, an index into BehaviourInfo.nodes."
  field name String "The action's name (at most 63 bytes)."
  field args String "Its arguments as written, separated by spaces (at most 255 bytes; longer ones are refused when the behaviour is parsed)."
}
```

<sub>Module `ai` line 139</sub>
