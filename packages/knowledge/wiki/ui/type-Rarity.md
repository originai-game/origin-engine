<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-Rarity"></a>

### `Rarity` (enum)

A loot tier (the style's rarity colours, each paired with its pips so no tier is told by colour alone).

A string: one of the variants below. The zero value is `"none"`.

| Variant | Doc |
|---|---|
| `"none"` | No tier. |
| `"common"` | Common: one pip. |
| `"uncommon"` | Uncommon: two pips. |
| `"rare"` | Rare: three pips. |
| `"epic"` | Epic: four pips. |
| `"legendary"` | Legendary: five pips. |
| `"mythic"` | Mythic: six pips. |

```ogt
enum Rarity {
  doc "A loot tier (the style's rarity colours, each paired with its pips so no tier is told by colour alone)."
  variant none "No tier."
  variant common "Common: one pip."
  variant uncommon "Uncommon: two pips."
  variant rare "Rare: three pips."
  variant epic "Epic: four pips."
  variant legendary "Legendary: five pips."
  variant mythic "Mythic: six pips."
}
```

<sub>Module `ui` line 3007</sub>
