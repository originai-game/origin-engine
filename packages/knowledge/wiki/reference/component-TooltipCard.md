<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-TooltipCard"></a>
### `TooltipCard`

An item's tooltip card: a dense smoked plate lit along its top in its rarity's colour, its icon in a socket of that colour, the title and the one-to-six rarity pips, its stats (label, figure, a signed delta in the better green or the worse red with its triangle, an added or removed stat a chip), the body, a footer line and the action hints with their key caps; what is equipped stands beside it on a plainer plate, and under it on a narrow screen. A long name or stat wraps between words. Read only.

Dropped against the .svelte: the socket's octagon (no clip-path here; a square well stands in) and the CJK heading sizes (a lang has no class variant here).

`import { TooltipCard } from "@origingame/engine/ui/components"` (kit `panels`, `packages/engine/src/ui/components/panels/tooltip-card.tsx`).

| Prop | Type | What |
|---|---|---|
| `title` | `string` | The title, in its rarity's colour. |
| `subtitle` | `string` | A line under it (Legendary sword, Passive); empty: none. |
| `rarity` | `string` | none, common, uncommon, rare, epic, legendary or mythic: the title's and frame's colour and its pips (one to six, so no tier is told by colour alone). |
| `icon` | `string` | An icon beside the title when there is no image; empty: none. |
| `image` | `Texture \| 0` | An art slot beside the title (an item or ability texture handle, 1:1); 0: the icon. |
| `body` | `string` | The body text; empty: none. |
| `stats` | `ReadonlyArray<{ id: string; label: string; value: string; delta?: number; deltaText?: string; better?: boolean; change?: string }>` | Keyed { id, label, value, delta, deltaText, better, change? } comparison facts supplied by the game. Signed delta and its arrow describe the numeric direction; better true means lower is better, flipping the colours. Optional change is added or removed; absent means both items have the stat. Added shows value without delta; removed shows the compared value muted. Presence is translated and exposed as data-change, never inferred from strings. |
| `compareTitle` | `string` | The heading over a second card of what is equipped (Equipped); empty draws no second card. |
| `compareStats` | `ReadonlyArray<{ id: string; label: string; value: string }>` | The equipped item's stat rows, { id, label, value }. |
| `footer` | `string` | A last line (Sell for 120, a flavour quote) in the muted ink; empty: none. |
| `hints` | `ReadonlyArray<{ id: string; text: string; action: string }>` | Action hints (Right-click to equip), a keyed collection of { id, text, action }: action an input action whose glyph leads the hint; empty: none. |

Pack slots: `Chip.frame`, `Panel.frame`.

Fixtures: `default`, `comparison`.
