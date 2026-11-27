<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RpgTurnOrder"></a>
### `RpgTurnOrder`

The initiative: round portraits in acting order, the one acting first and ringed, between two end flourishes, and a round plate on a rule under them. All portraits share the width, so five fit a phone. Reports nothing.

`import { RpgTurnOrder } from "@origingame/engine/ui/components"` (kit `rpg`, `packages/engine/src/ui/components/rpg/turn-order.tsx`).

| Prop | Type | What |
|---|---|---|
| `round` | `number` | The round, on the plate under the portraits. |
| `turns?` | `ReadonlyArray<{ id: string; name: string; portrait: Texture \| 0; side: string; acting: boolean; down: boolean }>` | Keyed rows of { id, name, portrait, side, acting, down }, in the order they act: side ally, enemy or neutral; acting the one whose turn it is; portrait a texture, 0 none. |

Pack slots: `Chip.frame`.

Fixtures: `default`.
