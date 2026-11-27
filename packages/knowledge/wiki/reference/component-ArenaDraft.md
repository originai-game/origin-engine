<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-ArenaDraft"></a>
### `ArenaDraft`

A ban-and-pick draft: both teams' ban and pick slots in columns, the team choosing now marked current, the phase named in a band, the turn's timer and Lock in. The game owns the order, the turn and when the local player may lock in; it reports onConfirm.

`import { ArenaDraft } from "@origingame/engine/ui/components"` (kit `arena`, `packages/engine/src/ui/components/arena/draft.tsx`).

| Prop | Type | What |
|---|---|---|
| `title?` | `string` | The draft's heading; empty takes the kit's (Draft). |
| `phase?` | `string` | The phase the game is in: ban, pick or done (each named in words in the band). |
| `active?` | `string` | The team choosing now: a, b, or empty for neither (its column is marked current and named in the band). |
| `nameA?` | `string` | The first team's name; empty takes the kit's (Blue team). |
| `nameB?` | `string` | The second team's name; empty takes the kit's (Red team). |
| `slots?` | `ReadonlyArray<DraftSlot>` | Both teams' ban and pick slots in the game's order. |
| `timeLeft?` | `number` | Seconds left in the turn as the game counts them (a HudTimer); negative hides it. Default -1. |
| `timeTotal?` | `number` | The turn's whole time, for the timer's ring (0: no ring). |
| `canConfirm?` | `boolean` | Lock in is offered (false: disabled; the game decides when the local player may lock in). |
| `onConfirm?` | `(e: UiEvent) => void` | Lock in. |

Each `DraftSlot`:

| Field | Type | What |
|---|---|---|
| `id` | `string` | The slot's key. |
| `team?` | `string` | a or b. |
| `kind?` | `string` | ban or pick. |
| `name?` | `string` | The character placed (empty: the slot is open). |
| `portrait?` | `Texture \| 0` | The character's portrait texture (0 shows the icon). |
| `icon?` | `string` | An icon name, shown while no portrait is set. |
| `player?` | `string` | Who the pick belongs to (empty hides). |
| `active?` | `boolean` | The slot being filled now (Banning, Picking). |

Pack slots: `Button.plate.primary`, `Chip.frame`, `Panel.frame`, `Row.frame`, `Row.frame.selected`.

Fixtures: `default`, `ban`, `done`, `overflow`.
