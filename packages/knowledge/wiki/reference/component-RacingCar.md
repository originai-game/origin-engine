<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RacingCar"></a>
### `RacingCar`

The car's condition at the bottom left: the tyres as tags two to a line, each worded and inked by its wear, the fuel as a thin bar with the game's words after it (danger and Fuel low when it runs low), and Box this lap when the team calls the car in. Read only.

`import { RacingCar } from "@origingame/engine/ui/components"` (kit `racing`, `packages/engine/src/ui/components/racing/car.tsx`).

| Prop | Type | What |
|---|---|---|
| `tyres?` | `ReadonlyArray<{ id: string; label: string; text: string; state: string }>` | The tyres in the game's order (front left, front right, rear left, rear right), keyed rows of { id, label, text, state }: label and text as the game words them (FL, 92); state good, worn or bad, a green, gold or red ink on the tag. Two to a line. Empty hides the tyres. |
| `fuel?` | `number` | The fuel left, 0 to 1; negative hides the fuel. Default -1. |
| `fuelText` | `string` | The fuel as the game words it (18 L · 4 laps), after the bar; empty for none. |
| `fuelLow` | `boolean` | The fuel is low: the bar turns to danger with a Fuel low word (data-low). |
| `pit` | `boolean` | The team calls the car in: a Box this lap tag stands under the readout (data-pit). |

Pack slots: `Chip.frame`, `Plate.frame`.

Fixtures: `default`, `fuel`, `fuelLow`, `overflow`.
