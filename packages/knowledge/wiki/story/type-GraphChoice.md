<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="type-GraphChoice"></a>

### `GraphChoice` (struct)

A choice in a dialogue's graph.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `index` | `number` | `0` | — | — | — | Its ChoiceView.index. |
| `id` | `string` | `""` | — | — | — | Its id within the node. |
| `key` | `string` | `""` | — | — | — | Its i18n key. |
| `tags` | `unknown` | `null` | — | — | — | Its hashtags, as ChoiceView.tags. |
| `to` | `readonly string[]` | `[]` | — | — | — | The nodes it may lead to: every node its body may jump to, through a check's #fail too. |

```ogt
struct GraphChoice {
  doc "A choice in a dialogue's graph."
  field index u32 "Its ChoiceView.index."
  field id String "Its id within the node."
  field key String "Its i18n key."
  field tags Json "Its hashtags, as ChoiceView.tags."
  field to List.String "The nodes it may lead to: every node its body may jump to, through a check's #fail too."
}
```

<sub>Module `story` line 181</sub>
