<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ai`](../ai.md) · [index](../index.md)

<a id="type-BehaviourInfo"></a>

### `BehaviourInfo` (struct)

A loaded behaviour.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `behaviour` | `Behaviour` | `0` | — | — | — | What AiAgent.behaviour takes. |
| `name` | `string` | `""` | — | — | — | Its name. |
| `kind` | `string` | `""` | — | — | — | tree or machine. |
| `states` | `readonly string[]` | `[]` | — | — | — | A machine's states, by index (AiAgent.state). |
| `nodes` | `readonly string[]` | `[]` | — | — | — | Its nodes' names, by index (AiAgent.node). |
| `keys` | `readonly string[]` | `[]` | — | — | — | Its declared blackboard keys (its subtrees' too). |
| `subtrees` | `readonly string[]` | `[]` | — | — | — | The behaviours its `use` nodes inline, every one reached, in the order first met. |

```ogt
struct BehaviourInfo {
  doc "A loaded behaviour."
  field behaviour Behaviour "What AiAgent.behaviour takes."
  field name String "Its name."
  field kind String "tree or machine."
  field states List.String "A machine's states, by index (AiAgent.state)."
  field nodes List.String "Its nodes' names, by index (AiAgent.node)."
  field keys List.String "Its declared blackboard keys (its subtrees' too)."
  field subtrees List.String "The behaviours its `use` nodes inline, every one reached, in the order first met."
}
```

<sub>Module `ai` line 155</sub>
