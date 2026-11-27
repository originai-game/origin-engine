<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-RacingTelemetry"></a>
### `RacingTelemetry`

The tachometer at the bottom right: the revs as a white arc round a dark glass face with the red zone at its end, the dial's figures inside the arc, and on the face the shift lights, the speed in the game's units and the gear on a lit tag with the gearbox's mode. The game decides every figure, the lights and the shift cue; read only.

`import { RacingTelemetry } from "@origingame/engine/ui/components"` (kit `racing`, `packages/engine/src/ui/components/racing/telemetry.tsx`).

| Prop | Type | What |
|---|---|---|
| `speed` | `number` | The speed as the game measures it, already in its units (a whole number reads best). |
| `units` | `string` | The speed's unit as the game words it (km/h, mph); shown under the figure and read with it. |
| `gear` | `string` | The gear as the game names it: R, N, 1 to 8; on a tag in the HUD's accent; empty hides it. |
| `mode` | `string` | The gearbox's mode as the game words it (Auto, Manual), beside the gear; empty hides it. |
| `rpm` | `number` | Engine speed now: the rev arc's length. |
| `rpmMax?` | `number` | The top of the rev range, the arc's end. Default 1. |
| `redline` | `number` | The engine speed where the red zone starts, marked on the arc (lit, with data-redline on the frame, once the revs reach it); 0 is none. |
| `scale?` | `ReadonlyArray<{ id: string; value: number; label: string }>` | The dial's figures, keyed rows of { id, value, label }: value an engine speed on the arc, label its figure as the game words it (2 for 2,000 rpm); a figure at or past the redline is red. A tick stands by each. Empty draws none. A phone shows the ticks alone. |
| `lights?` | `number` | The share of the shift lights lit, 0 to 1, as the game decides it (seven cells, green to amber to red); negative hides the lights. Default -1. |
| `shift` | `boolean` | The game's shift cue: the Shift word stands beside the gear in place of the mode, and the frame takes data-shift. |

Fixtures: `default`, `redline`, `pit`, `reverse`, `idle`, `overflow`.
