<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`inventory`](../inventory.md) · [index](../index.md)

<a id="fn-roll"></a>

### `og.inventory.roll`

Draw from a loot table on og.Rng: each draw picks an entry in proportion to its weight (og.Rng.weighted) and a count from min to max (og.Rng.integer). The same seed draws the same loot; nothing is added anywhere.

Also known as: loot table, drop table, random loot, gacha, chest loot, weighted drop

```ts
og.inventory.roll(world: World, desc: RollDesc): readonly ItemCount[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world; a system that calls this lists og.Rng in its writes. |
| `desc` | `RollDesc` | no | The entries and how many draws. |

**Returns** `readonly ItemCount[]`: One per draw that picked an entry, in draw order.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn roll {
  doc "Draw from a loot table on og.Rng: each draw picks an entry in proportion to its weight (og.Rng.weighted) and a count from min to max (og.Rng.integer). The same seed draws the same loot; nothing is added anywhere."
  aliases "loot table, drop table, random loot, gacha, chest loot, weighted drop"
  host false
  param world World "The world; a system that calls this lists og.Rng in its writes."
  param desc RollDesc "The entries and how many draws."
  returns List.ItemCount "One per draw that picked an entry, in draw order."
}
```

<sub>Module `inventory` line 171</sub>
