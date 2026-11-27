<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RacingBanner"></a>
### `RacingBanner`

The race's calls at the top centre: a plate of smoked glass with a lit top edge in the HUD's accent, the call in widely set capitals (the count, GO, the final lap, the finish with the player's place, or the game's own words) and a line under it. The game steps the call and its id (at); a new id plays the entrance again. Announced as a status.

`import { RacingBanner } from "@origingame/engine/ui/components"` (kit `racing`, `packages/engine/src/ui/components/racing/banner.tsx`).

| Prop | Type | What |
|---|---|---|
| `kind?` | `string` | What the banner calls: countdown (the count), go, finalLap, finish, call (the game's own words, in text), or none (nothing shown, as for any other word). Default "none". |
| `figure` | `number` | The countdown's figure (3, 2, 1); the game steps it. |
| `text` | `string` | The call's words as the game words them for kind call (Final lap next, Sector best); empty shows nothing. |
| `at` | `number` | The call's id: a new value replays the entrance (each count, the go, the flag). |
| `place` | `number` | On the finish banner, the player's place as an ordinal (1st); 0 hides it. |
| `detail` | `string` | A line under the call as the game words it (New lap record, Lap 3 of 3); empty hides it. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `countdown`, `go`, `finalLap`, `finish`, `overflow`.
