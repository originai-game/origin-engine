<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="type-GraphNode"></a>

### `GraphNode` (struct)

A node in a dialogue's graph.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `title` | `string` | `""` | — | — | — | Its title. |
| `headers` | `unknown` | `null` | — | — | — | Its header lines, `{ title: "kitchen", "og-flow": "segments/kitchen.ogt" }`. |
| `choices` | `readonly GraphChoice[]` | `[]` | — | — | — | Its choices, in order (hidden ones too). |
| `to` | `readonly string[]` | `[]` | — | — | — | The nodes its own steps may jump to, every branch of its conditions counted (a choice's leads are its own). |

```ogt
struct GraphNode {
  doc "A node in a dialogue's graph."
  field title String "Its title."
  field headers Json "Its header lines, `{ title: \"kitchen\", \"og-flow\": \"segments/kitchen.ogt\" }`."
  field choices List.GraphChoice "Its choices, in order (hidden ones too)."
  field to List.String "The nodes its own steps may jump to, every branch of its conditions counted (a choice's leads are its own)."
}
```

<sub>Module `story` line 191</sub>
