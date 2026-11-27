<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-PerfReadout"></a>
### `PerfReadout`

A HUD plate of how the game runs: the connection state in its tone's colour, then four metrics — frames, frame time, ping and packet loss — each its caption over its figure, Unavailable where the game has no reading. The game supplies every value and threshold; the document samples no clocks or network. Read only.

`import { PerfReadout } from "@origingame/engine/ui/components"` (kit `hud`, `packages/engine/src/ui/components/hud/perf-readout.tsx`).

| Prop | Type | What |
|---|---|---|
| `fps?` | `number` | Frames per second; negative means unavailable, zero is a real reading. |
| `frameMs?` | `number` | Frame time in milliseconds; negative means unavailable. |
| `ping?` | `number` | Round-trip milliseconds; negative means unavailable. |
| `loss?` | `number` | Packet loss as a fraction, 0 to 1; negative means unavailable. |
| `status?` | `string` | offline, connecting, online or degraded. The game supplies connection state and thresholds; the document samples no clocks or network. |

Pack slots: `Plate.frame`.

Fixtures: `default`, `degraded`, `offline`, `connecting`, `unknown`, `zero`.
