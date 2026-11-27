<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RacingGarage"></a>
### `RacingGarage`

Choosing a car: the shown car large (its picture, name and class) with arrows to step the cars, a strip of tiles to jump to one, the shown car's ratings as bars, and one action for it: Drive when owned, Buy with its price when it can be had, a disabled Locked with the reason when it cannot, Driving when it is already the player's car. The game owns prices, unlocking and every rating; the garage reports the car's key and Back.

`import { RacingGarage } from "@origingame/engine/ui/components"` (kit `racing`, `packages/engine/src/ui/components/racing/garage.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | The page's heading; empty takes the kit's (Garage). |
| `vehicles?` | `ReadonlyArray<{ id: string; title: string; body: string; image: Texture \| 0; owned: boolean; locked: boolean; affordable: boolean; price: string }>` | The cars in the game's order, keyed rows of { id, title, body, image, owned, locked, affordable, price }: title the car's name, body its class line, image a texture (og.load.texture, 0 none); owned the player has it; locked it cannot be had yet; affordable the game allows buying it now (false disables Buy and names it Too costly); price what buying costs or why it is locked, as the game words it (12,500 cr, Reach level 12). |
| `shown` | `string` | The car in view (its tile selected, its stats shown, the action for it); empty is none. |
| `index` | `number` | The shown car's place in vehicles, from 0, read out as n of N beside the arrows (the game keeps it with shown). |
| `chosen` | `string` | The car the player drives now (Driving); empty is none. |
| `stats?` | `ReadonlyArray<{ id: string; label: string; value: number; max: number; text: string }>` | The shown car's ratings, keyed rows of { id, label, value, max, text }: label as the game words it (Speed); value over max fills the bar; text the figure shown beside it (8.4). |
| `onTabPrev?` | `(e: UiEvent) => void` | Previous car. |
| `onTabNext?` | `(e: UiEvent) => void` | Next car. |
| `onBack?` | `(e: UiEvent) => void` | Back. |
| `onChoose?` | `(e: UiEvent, id: string) => void` | Choose a car from its tile. |
| `onSelect?` | `(e: UiEvent, id: string) => void` | Drive the shown car. |
| `onBuy?` | `(e: UiEvent, id: string) => void` | Buy the shown car. |

Pack slots: `Button.plate`, `Button.plate.primary`, `Chip.frame`, `Panel.frame`, `Plate.frame`.

Fixtures: `default`, `selected`, `buy`, `short`, `locked`, `empty`, `overflow`.
