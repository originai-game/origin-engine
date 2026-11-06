<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ai`](../ai.md) · [index](../index.md)

<a id="type-AiDescription"></a>

### `AiDescription` (struct)

What an agent is doing.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `behaviour` | `string` | `""` | — | — | — | Its behaviour's name. |
| `kind` | `string` | `""` | — | — | — | tree or machine. |
| `state` | `string` | `""` | — | — | — | A machine's state (empty for a tree). |
| `node` | `string` | `""` | — | — | — | The node it ran last, such as `chase`. |
| `path` | `string` | `""` | — | — | — | The path to it from the root, such as `selector > sequence chase > chase`. |
| `status` | `string` | `""` | — | — | — | running, success or failure. |
| `blackboard` | `unknown` | `null` | — | — | — | Its blackboard, keys to numbers, booleans, points and entity handles. |
| `sees` | `boolean` | `false` | — | — | — | It sees its target. |
| `target` | `Entity` | `0` | — | — | — | Its target (0: none). |

```ogt
struct AiDescription {
  doc "What an agent is doing."
  field behaviour String "Its behaviour's name."
  field kind String "tree or machine."
  field state String "A machine's state (empty for a tree)."
  field node String "The node it ran last, such as `chase`."
  field path String "The path to it from the root, such as `selector > sequence chase > chase`."
  field status String "running, success or failure."
  field blackboard Json "Its blackboard, keys to numbers, booleans, points and entity handles."
  field sees bool "It sees its target."
  field target Entity "Its target (0: none)."
}
```

<sub>Module `ai` line 180</sub>
