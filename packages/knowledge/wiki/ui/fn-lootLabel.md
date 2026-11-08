<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-lootLabel"></a>

### `og.ui.lootLabel`

A loot label to the bar of Diablo IV, Path of Exile and Borderlands: the name on a dark chip edged in its rarity's colour with pips so tiers do not rest on colour alone, the stack's count, a beam of light rising over rare and better items, labels stacking instead of overlapping (higher rarity on top), pressable to pick up.

Parts: LootLabel.chip[rarity=<r>], LootLabel.name, LootLabel.pip, LootLabel.count, LootLabel.beam. Accessibility: button named the name, value its rarity. Probe ids: <id>, <id>/Chip/Name.

```ts
og.ui.lootLabel(id: string, desc?: LootLabelDesc): LootLabelResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `LootLabelDesc` | yes | The item. |

**Returns** `LootLabelResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn lootLabel {
  doc "A loot label to the bar of Diablo IV, Path of Exile and Borderlands: the name on a dark chip edged in its rarity's colour with pips so tiers do not rest on colour alone, the stack's count, a beam of light rising over rare and better items, labels stacking instead of overlapping (higher rarity on top), pressable to pick up."
  doc "Parts: LootLabel.chip[rarity=<r>], LootLabel.name, LootLabel.pip, LootLabel.count, LootLabel.beam. Accessibility: button named the name, value its rarity. Probe ids: <id>, <id>/Chip/Name."
  host false
  param id String "Its id."
  param desc LootLabelDesc "The item."
  returns LootLabelResult
}
```

<sub>Module `ui` line 6713</sub>
