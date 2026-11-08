<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-tooltipCard"></a>

### `og.ui.tooltipCard`

A rich tooltip: a card in the tooltip layer beside its trigger, flipped and clamped inside the safe area, with a title in its rarity's colour and pips, an art slot, rich-text body whose keyword links open nested cards (to any depth, each pinnable, as in Crusader Kings III and Baldur's Gate 3), stat rows, a comparison card of what is equipped with up and down deltas (Diablo IV), a footer and action hints. It fades and rises in on the style's enter spring after delayMs; a pinnable card fills a ring at its corner and locks; a locked card stays while the pointer is on it or on a card it opened.

Parts: TooltipCard.card[<rarity>][pinned], TooltipCard.header, TooltipCard.title, TooltipCard.subtitle, TooltipCard.well, TooltipCard.body, TooltipCard.stat, TooltipCard.delta[up|down], TooltipCard.compare (the second card), TooltipCard.footer, TooltipCard.hint, TooltipCard.pin (the ring). Slots: image (art kind item or ability, 1:1). States: group pin (resting, pinning, pinned); group cover (clear, hidden: a banner or level-up is announcing, and the card fades fully out and takes no input until it has gone). Sounds: open when it pins. Accessibility: tooltip named by the title, described by the body; its links are links. Probe ids: <id>, <id>/Title, <id>/Body (its links <id>/Body/<key>), <id>/Stat#i, <id>/Compare, <id>/Pin, a nested card <id>/<key>.

```ts
og.ui.tooltipCard(id: string, desc?: TooltipCardDesc): TooltipCardResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id (one per trigger). |
| `desc` | `TooltipCardDesc` | yes | When it shows, what it says and where. |

**Returns** `TooltipCardResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn tooltipCard {
  doc "A rich tooltip: a card in the tooltip layer beside its trigger, flipped and clamped inside the safe area, with a title in its rarity's colour and pips, an art slot, rich-text body whose keyword links open nested cards (to any depth, each pinnable, as in Crusader Kings III and Baldur's Gate 3), stat rows, a comparison card of what is equipped with up and down deltas (Diablo IV), a footer and action hints. It fades and rises in on the style's enter spring after delayMs; a pinnable card fills a ring at its corner and locks; a locked card stays while the pointer is on it or on a card it opened."
  doc "Parts: TooltipCard.card[<rarity>][pinned], TooltipCard.header, TooltipCard.title, TooltipCard.subtitle, TooltipCard.well, TooltipCard.body, TooltipCard.stat, TooltipCard.delta[up|down], TooltipCard.compare (the second card), TooltipCard.footer, TooltipCard.hint, TooltipCard.pin (the ring). Slots: image (art kind item or ability, 1:1). States: group pin (resting, pinning, pinned); group cover (clear, hidden: a banner or level-up is announcing, and the card fades fully out and takes no input until it has gone). Sounds: open when it pins. Accessibility: tooltip named by the title, described by the body; its links are links. Probe ids: <id>, <id>/Title, <id>/Body (its links <id>/Body/<key>), <id>/Stat#i, <id>/Compare, <id>/Pin, a nested card <id>/<key>."
  host false
  param id String "Its id (one per trigger)."
  param desc TooltipCardDesc "When it shows, what it says and where."
  returns TooltipCardResult
}
```

<sub>Module `ui` line 2894</sub>
