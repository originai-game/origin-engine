<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`flow`](../flow.md) · [index](../index.md)

<a id="type-FlowInfo"></a>

### `FlowInfo` (struct)

What og.flow.load prepared.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `dialogue` | `string` | `""` | — | — | — | The dialogue. |
| `segments` | `readonly string[]` | `[]` | — | — | — | Its flow nodes (those with `og-flow`), in file order. |
| `chapters` | `readonly string[]` | `[]` | — | — | — | Its chapters' i18n keys (`og-flow-chapter`), in file order. |
| `endings` | `readonly string[]` | `[]` | — | — | — | Its endings (`og-flow-ending`), in file order. |

```ogt
struct FlowInfo {
  doc "What og.flow.load prepared."
  field dialogue String "The dialogue."
  field segments List.String "Its flow nodes (those with `og-flow`), in file order."
  field chapters List.String "Its chapters' i18n keys (`og-flow-chapter`), in file order."
  field endings List.String "Its endings (`og-flow-ending`), in file order."
}
```

<sub>Module `flow` line 18</sub>
