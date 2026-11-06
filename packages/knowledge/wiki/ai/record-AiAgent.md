<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ai`](../ai.md) · [index](../index.md)

<a id="record-AiAgent"></a>

### `og.AiAgent` (component)

An entity that thinks with a behaviour. Give it an og.NavAgent to move and an og.Perception to see and hear.

Token `og.AiAgent`, registered under the schema name `og.AiAgent`; its value type is `AiAgent`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `behaviour` | `Behaviour` | `0` | — | — | — | The behaviour it runs (og.ai.load); 0 runs nothing. |
| `active` | `boolean` | `true` | — | — | — | Thinks while true. |
| `thinkEvery` | `number` | `1` | 1 … 600 | `ticks` | — | Thinks every this many ticks (1: every tick); agents are staggered by their order. |
| `status` | `number` | `0` | — | — | read only | AI_RUNNING, AI_SUCCESS or AI_FAILURE after the last think (0 before the first; written by og.ai). |
| `state` | `number` | `0` | — | — | read only | The state machine's state, an index into BehaviourInfo.states (written by og.ai). |
| `node` | `number` | `0` | — | — | read only | The node it ran last, an index into BehaviourInfo.nodes (written by og.ai). |
| `changes` | `number` | `0` | — | — | read only | Times the state or the running node changed (written by og.ai). |

```ogt
component AiAgent {
  doc "An entity that thinks with a behaviour. Give it an og.NavAgent to move and an og.Perception to see and hear."
  field behaviour Behaviour "The behaviour it runs (og.ai.load); 0 runs nothing."
  field active bool "Thinks while true." {
    default true
  }
  field thinkEvery u32 "Thinks every this many ticks (1: every tick); agents are staggered by their order." {
    default 1
    min 1
    max 600
    unit ticks
  }
  field status u32 "AI_RUNNING, AI_SUCCESS or AI_FAILURE after the last think (0 before the first; written by og.ai)." {
    readonly true
  }
  field state u32 "The state machine's state, an index into BehaviourInfo.states (written by og.ai)." {
    readonly true
  }
  field node u32 "The node it ran last, an index into BehaviourInfo.nodes (written by og.ai)." {
    readonly true
  }
  field changes u32 "Times the state or the running node changed (written by og.ai)." {
    readonly true
  }
}
```

<sub>`api.ogt` line 14226</sub>
