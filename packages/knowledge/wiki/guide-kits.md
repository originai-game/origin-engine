<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `kits` guide: The UI kits

The genre kits bundled with the engine: what each one draws, how a game installs, changes and reviews one, and how to make a kit of its own. Every kit is TypeScript over the public API and runs unchanged on the web and native hosts.

`og ui list` (`--json` for a script) names every bundled and installed kit
with its documents. Each kit's members take a desc whose type is in its
`types.ts`: every field has a doc comment and a default, so `{}` is valid.
Its `fixtures/` hold a whole example in those shapes: start from them.

<a id="guide-kits-1"></a>
### `kits §1` What each kit draws

| Kit | Members |
|---|---|
| `rpg` | `inventoryGrid` (multi-cell items, stacks, rarity, drag and drop), `paperdoll` (slots round the game's figure, stats with deltas), `questLog`, `questTracker` (og.story quests), `conversation` (an og.story line with choices, checks and rolls), `unitFrame`, `partyFrames`, `raidFrames`, `castBar`, `turnOrder`, `worldMap` |
| `shooter` | `crosshair` (spread, hit and headshot markers), `ammoCounter`, `killfeed`, `damageDirection`, `lowHealthVignette`, `scoreboard`, `loadout` |
| `racing` | `speedometer` (dial, tach, digital, arcade), `lapTimer`, `standingsTower`, `raceBanner`, `rearView`, `itemSlot`, `driftScore`, `garage`; the `race-results` screen document |
| `strategy` | `rtsCommands` (the whole console), `rtsConsole`, `selectionPanel`, `commandCard`, `productionQueue`, `dragSelect`, `resourceBar`, `wavePreview`, `ledger` |
| `puzzle` | `sagaMap` (levels on a scrolling path, stars, locks, the avatar's pin), `goalChips` |
| `gacha` | `gachaBanner` (tabs, hero art drawn by the game's callback, pity, single and multi wish), `pullSequence` (charge, one reveal per item, the summary) |
| `card` | `cardHand`, `rewardChoice`, `deckBuilder` |
| `idle` | `idleGenerator`, `prestigePanel` |
| `liveops` | `storeFront`, `battlePass`; store, daily-login, events and mailbox screens |
| `story` | `backlog`, `barks`, `cgGallery`, `routeChart` |
| `versus` | `lobby`, `characterSelect`, `versus`, `draft`, `chat` |
| `rhythm` | `songSelect`, `laneKeys`, `comboCounter`, `judgement` |
| `sports` | `scorebug`, `matchStats`, `goalTimeline`, `playerPlate`, `substitution` |
| `photo` | `photoMode` |
| `flight` | `flightHud` |
| `materials` | per-element shader finishes, not a genre kit (ui-materials §1) |

Generic pieces stay in og.ui: minimaps, compasses, timers, meters,
counters, ability slots, tooltips, celebrations. A kit builds on them and
never adds members to `og.ui`.

<a id="guide-kits-2"></a>
### `kits §2` Use one

Import the kit's namespace and call its members from the system that
builds the game's UI each tick. The game owns the state and applies the
results a member answers (`GarageResult`, `LoadoutResult`, `SagaMapResult`).

```ts
import { racing } from "@origingame/engine/extensions/racing/index.ts";

racing.lapTimer("Head", { show: "head", position: 4, positionTotal: 7, lap: 2, lapTotal: 3 });
og.ui.anchor("Instrument", { anchor: "bottomRight", margin: 16 }, () =>
  racing.speedometer("Speed", { style: "tach", speed: 238, unit: "mph", rpm: 0.8, gear: "5", anchor: null }));
```

- A screen document (`race-results.ogt`) is loaded with og.load.document
  from the kit's path (`ui-kits/racing/race-results.ogt`) and mounted with
  og.ui.screen and a viewmodel. Its `props` line is the viewmodel, its
  `emit` events are what the game answers, its slots take the game's own
  content through og.ui.openSlot. A new field goes into the rows the game
  passes, an element bound to it and the document's `fixture default`.
- `raceBanner`'s `below` names the HUD element its split and final-lap band
  stand under; without it they sit a ninth and a fifth of the way down.
- Gacha tiers are the game's: a unit takes the engine's `rarity`, or its own
  `tier: { label, rank, color }`; rank 5 and up shine and burst.

<a id="guide-kits-3"></a>
### `kits §3` Change one

1. `og ui add <kit>` copies it into `assets/ui-kits/<kit>/`. The copy is the
   game's: edit any file. It overrides the bundled kit in `og ui list`.
2. Import the copy (`../assets/ui-kits/<kit>/index.ts`, from the importing
   file) instead of `@origingame/engine/extensions/<kit>/index.ts`.
3. A new option is a field in the desc in `types.ts`, read in the
   component in `src/`.
4. The parts a component opens (`sel("Speedometer", "dial")`) are what a
   style's selector blocks restyle (`Speedometer::dial:hover { … }`): a look change belongs in the style
   (screens §3), not in the kit.
5. Helpers come from `@origingame/engine/ui-parts/hud` and
   `@origingame/engine/ui-parts/screen` only; never import engine files by
   path.
6. `og ui eject <kit>/<document>` copies one screen with its companions
   into `assets/ui/<kit>/` without installing the kit.

<a id="guide-kits-4"></a>
### `kits §4` Review one

`og ui sheet <kit>` renders every page of the kit's `sheet/project.ogt` at
desktop and phone, 100 % and 150 % text, into `og-out/ui-sheet/<kit>/`
(the installed copy when there is one). A new page is one more
`screen "<name>" "style:<pack>;page:<page>"` line and its builder in
`sheet/src/main.ts`. Judge it by screens §5: keyboard, pad, touch and the
larger text, in the game's own style.

<a id="guide-kits-5"></a>
### `kits §5` A kit of the game's own

Copy a kit to `assets/ui-kits/<name>/`, rename `name` in `extension.json`
and the namespace in `index.ts` (`registerUiExtension("<name>", { … })`),
then replace the components. Nothing outside the directory lists kits.
