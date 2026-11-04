---
name: building-shooter-ui
description: Builds shooter and battle-royale UI with the shooter kit (crosshair and hit markers, ammunition, killfeed, damage direction, low-health vignette, the tab scoreboard, the loadout) over Origin Engine's common components. Use when a shooter needs its HUD, scoreboard or loadout screen, or when editing, ejecting or reviewing the shooter kit.
---

# Shooter kit

A kit written over the engine's public API. It runs unchanged on the web
and native hosts.

## Find what is there

```sh
og ui list                  # every kit and style, each kit's documents
og ui list --json           # the same, for a script
```

The namespace is `shooter`. Members, with their desc types in `types.ts`
(every field has a doc comment and a default, `{}` is valid):

| Member | What it draws |
|---|---|
| `crosshair(id, CrosshairDesc)` | Cross, dot, circle or chevron reticle with spread, hit and headshot markers |
| `ammoCounter(id, AmmoDesc)` | Magazine and reserve, fire mode, reload progress |
| `killfeed(id, KillfeedDesc)` | Recent eliminations, newest first, the player's lit |
| `damageDirection(id, DamageDirectionDesc)` | Arcs round the centre toward recent hits |
| `lowHealthVignette(id, LowHealthDesc)` | The screen's edge reddening under a health threshold |
| `scoreboard(id, ScoreboardDesc)` | Teams, any columns the game names, ping, MVP, the dead dimmed |
| `loadout(id, LoadoutDesc)` | Slots, the selected item with attachment points, stats; answers LoadoutResult |

`fixtures/match.ts` has a match's killfeed, ammunition, scoreboard and loadout in these shapes: start from it.

## Use it

```ts
import { shooter } from "@origingame/engine/extensions/shooter/index.ts";

shooter.crosshair("Aim", { style: "cross", spread: 0.2, dot: true });
shooter.ammoCounter("Ammo", { magazine: 7, magazineMax: 30, reserve: 120, anchor: "bottomRight" });
```

Call them from the system that builds the game's UI each tick. Generic
pieces stay in `og.ui`: the compass (`og.ui.compass`), minimap, waypoints and pings, stat bars, the ability bar and timers.

## Change it

1. `og ui add shooter` copies the kit into `assets/ui-kits/shooter/`. The copy
   is yours: edit any file. It overrides the bundled kit in `og ui list`.
2. Import the copy instead: `import { shooter } from "../assets/ui-kits/shooter/index.ts";`
   (the path from the importing file).
3. A component is ordinary TypeScript in `src/`. Add a field to its desc
   in `types.ts`, then read it in the component. Parts it opens
   (`sel("Crosshair", "line")`) are what a style's `dress` block restyles, so a look change
   belongs in the style, not here.
4. Helpers come from `@origingame/engine/ui-parts/hud` and
   `@origingame/engine/ui-parts/screen` only. Never import engine
   files by path.

## Review it

```sh
og ui sheet shooter           # the installed copy if there is one, else the bundled kit
```

It renders every page in `sheet/project.ogt` at desktop and phone, 100 %
and 150 %, into `og-out/ui-sheet/shooter/`. Add a page with one more
`screen "<name>" "style:<pack>;page:<page>"` line, and its builder in
`sheet/src/main.ts`.

## Make a new kit

Copy this directory to `assets/ui-kits/<name>/`, rename `name` in
`extension.json` and the namespace in `index.ts`
(`registerUiExtension("<name>", { … })`), then replace the components.
Nothing outside the directory lists kits.
