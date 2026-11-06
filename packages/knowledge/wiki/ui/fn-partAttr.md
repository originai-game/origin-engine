<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-partAttr"></a>

### `og.ui.partAttr`

Gives the element open now (a grid's cell, a panel, a card) a selector attribute its style dresses by: partAttr("rarity", "epic") in a bag's cell makes it `Slot.frame[rarity=epic]`, so a style that dresses the tier recolours the cell's own frame (one stroke, never a second). Call it right after the element opens, inside it.

```ts
og.ui.partAttr(key: string, value: string): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `key` | `string` | no | The attribute's name (rarity, tone, team). |
| `value` | `string` | no | Its value (epic, success). |

**Returns** `boolean`: Whether the style dresses this part by the attribute (for any of its values, `Slot.frame[rarity=rare]` answers for every tier): false, its values draw alike, and a caller that wants a mark draws its own.

Binding symbol `og_ui_part_attr` · publish builds: kept

```ogt
fn partAttr {
  doc "Gives the element open now (a grid's cell, a panel, a card) a selector attribute its style dresses by: partAttr(\"rarity\", \"epic\") in a bag's cell makes it `Slot.frame[rarity=epic]`, so a style that dresses the tier recolours the cell's own frame (one stroke, never a second). Call it right after the element opens, inside it."
  param key String "The attribute's name (rarity, tone, team)."
  param value String "Its value (epic, success)."
  returns bool "Whether the style dresses this part by the attribute (for any of its values, `Slot.frame[rarity=rare]` answers for every tier): false, its values draw alike, and a caller that wants a mark draws its own."
}
```

<sub>`api.ogt` line 23132</sub>
