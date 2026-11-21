# racing: built-in components

The race HUD (the place and lap times, the position tower, the tachometer,
the track map, the boost meter, the pedals, the car's tyres and fuel, the
calls and the points earned), garage selection and the results screen.
They draw what the game tells them and report intents with stable row
keys. They never compute speed, gear, lap times, gaps, positions, points,
prices or unlocks: every figure and time arrives formatted from the game.
The track map takes the racing line as `Line` points.

A game imports one by name from `@origingame/engine/ui/components`, with
its props as typed in its `.tsx` (`<Name>Props`, each prop documented
there); `fixtures` in the same file are its cases. How components are built
and dressed is `../README.md`.

## Members

| Component | File | What it is |
| --- | --- | --- |
| `RacingBanner` | `racing/banner` | The race's calls at the top centre. |
| `RacingBonus` | `racing/bonus` | Feedback above the road at the top centre. |
| `RacingBoost` | `racing/boost` | A boost meter (nitro, ERS) beside the tachometer. |
| `RacingCar` | `racing/car` | The car's condition at the bottom left. |
| `RacingGarage` | `racing/garage` | Choosing a car. |
| `RacingLapHud` | `racing/lap-hud` | The race's place and clock at the top left. |
| `RacingPedals` | `racing/pedals` | The pedals at the bottom left. |
| `RacingResults` | `racing/results` | The classification after the flag. |
| `RacingStandings` | `racing/standings` | The position tower at the left edge. |
| `RacingTelemetry` | `racing/telemetry` | The tachometer at the bottom right. |
| `RacingTrackMap` | `racing/track-map` | The track map at the top right. |

## Gallery

`bun packages/engine/src/node/gallery.ts --only racing/` draws every member's fixtures under every pack.
