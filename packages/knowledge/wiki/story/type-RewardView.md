<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="type-RewardView"></a>

### `RewardView` (struct)

A quest's reward.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | Its name, resolved in the active locale. |
| `amount` | `number` | `0` | — | — | — | How many. |
| `image` | `string` | `""` | — | — | — | Its image's asset path (empty: none). |
| `rarity` | `string` | `""` | — | — | — | The game's rarity word (empty: none). |

```ogt
struct RewardView {
  doc "A quest's reward."
  field name String "Its name, resolved in the active locale."
  field amount f64 "How many."
  field image String "Its image's asset path (empty: none)."
  field rarity String "The game's rarity word (empty: none)."
}
```

<sub>`api.ogt` line 14676</sub>
