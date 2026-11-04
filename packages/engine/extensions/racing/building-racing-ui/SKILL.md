---
name: building-racing-ui
description: Builds racing-game UI with the racing kit (speedometer and tach, lap timer, standings tower, race banner, arcade item slot and drift score, the garage) over Origin Engine's common components. Use when a racing or kart game needs its HUD, garage or race results, or when editing, ejecting or reviewing the racing kit.
---

# Racing kit

A kit of racing components written over the engine's public API. It runs
unchanged on the web and native hosts.

## Find what is there

```sh
og ui list                  # every kit and style, each kit's documents
og ui list --json           # the same, for a script
```

The namespace is `racing`. Members, with their desc types in `types.ts`
(every field has a doc comment and a default, `{}` is valid):

| Member | What it draws |
|---|---|
| `speedometer(id, SpeedometerDesc)` | Dial, `tach`, `digital` or `arcade` speed readout with gear, shift lights, boost |
| `lapTimer(id, LapTimerDesc)` | The place and lap with sector pips (`show: "head"`), or the lap times and split (`show: "times"`) |
| `standingsTower(id, StandingsTowerDesc)` | The field's order, absolute or relative to the player |
| `raceBanner(id, RaceBannerDesc)` | Countdown, GO, checkpoint split, final lap, finish, wrong way |
| `rearView(id, RearViewDesc)` | A mirror of a camera's view |
| `itemSlot(id, ItemSlotDesc)`, `driftScore(id, DriftScoreDesc)` | The arcade look's item roulette and drift chain |
| `garage(id, GarageDesc)` | Categories, parts, performance card and livery layers; answers `GarageResult` |

`fixtures/race.ts` has a whole race in these shapes: start from it.

## Use it

```ts
import { racing, type StandingsEntry } from "@origingame/engine/extensions/racing/index.ts";

racing.lapTimer("Head", { show: "head", position: 4, positionTotal: 7, lap: 2, lapTotal: 3 });
og.ui.anchor("Instrument", { anchor: "bottomRight", margin: 16 }, () =>
  racing.speedometer("Speed", { style: "tach", speed: 238, unit: "mph", rpm: 0.8, gear: "5", anchor: null }));
```

Call them from the system that builds the game's UI each tick. Generic
pieces stay in `og.ui`: the minimap (`og.ui.minimapView`, with `track` and
`sectors`), timers (`og.ui.hudTimer`), meters and counters.

## Change it

1. `og ui add racing` copies the kit into `assets/ui-kits/racing/`. The
   copy is yours: edit any file. It overrides the bundled kit in
   `og ui list`.
2. Import the copy instead: `import { racing } from "../assets/ui-kits/racing/index.ts";`
   (the path from the importing file).
3. A component is ordinary TypeScript in `src/`. Add a field to its desc
   in `types.ts`, then read it in the component. Parts it opens
   (`sel("Speedometer", "dial")`) are what a style's `dress` block
   restyles, so a look change belongs in the style, not here.
4. Helpers come from `@origingame/engine/ui-parts/hud` and
   `@origingame/engine/ui-parts/screen` only. Never import engine
   files by path.

## Review it

```sh
og ui sheet racing          # the installed copy if there is one, else the bundled kit
```

It renders every page in `sheet/project.ogt` at desktop and phone, 100 %
and 150 %, into `og-out/ui-sheet/racing/`. Add a page with one more
`screen "<name>" "style:<pack>;page:<page>"` line, and its builder in
`sheet/src/main.ts`.

## Make a new kit

Copy this directory to `assets/ui-kits/<name>/`, rename `name` in
`extension.json` and the namespace in `index.ts`
(`registerUiExtension("<name>", { … })`), then replace the components.
Nothing outside the directory lists kits.
