---
name: building-puzzle-ui
description: Builds puzzle and casual-game UI with the puzzle kit (the saga level map, a level's goal chips) over Origin Engine's common components. Use when a match-3, puzzle or casual game needs its level map or level goals, or when editing, ejecting or reviewing the puzzle kit.
---

# Puzzle kit

A kit written over the engine's public API. It runs unchanged on the web
and native hosts.

## Find what is there

```sh
og ui list                  # every kit and style, each kit's documents
og ui list --json           # the same, for a script
```

The namespace is `puzzle`. Members, with their desc types in `types.ts`
(every field has a doc comment and a default, `{}` is valid):

| Member | What it draws |
|---|---|
| `sagaMap(id, SagaMapDesc)` | Levels along a scrolling path, stars, locks, bosses, the avatar's pin; answers SagaMapResult |
| `goalChips(id, GoalChipsDesc)` | A level's goals as chips that pop when met |

`fixtures/levels.ts` has a world of twelve levels and one level's goals in these shapes: start from it.

## Use it

```ts
import { puzzle } from "@origingame/engine/extensions/puzzle/index.ts";

puzzle.goalChips("Goals", { goals: [{ key: "berry", label: "Berries", have: 18, need: 30 }] });
og.ui.starBar("Stars", { value: score, max: 9000 });
```

Call them from the system that builds the game's UI each tick. Generic
pieces stay in `og.ui`: the score's stars (`og.ui.starBar`), counters, progress bars, celebrations and rewards.

## Change it

1. `og ui add puzzle` copies the kit into `assets/ui-kits/puzzle/`. The copy
   is yours: edit any file. It overrides the bundled kit in `og ui list`.
2. Import the copy instead: `import { puzzle } from "../assets/ui-kits/puzzle/index.ts";`
   (the path from the importing file).
3. A component is ordinary TypeScript in `src/`. Add a field to its desc
   in `types.ts`, then read it in the component. Parts it opens
   (`"Goals.frame"`) are what a style's `dress` block restyles, so a look change
   belongs in the style, not here.
4. Helpers come from `@origingame/engine/ui-parts/hud` and
   `@origingame/engine/ui-parts/screen` only. Never import engine
   files by path.

## Review it

```sh
og ui sheet puzzle           # the installed copy if there is one, else the bundled kit
```

It renders every page in `sheet/project.ogt` at desktop and phone, 100 %
and 150 %, into `og-out/ui-sheet/puzzle/`. Add a page with one more
`screen "<name>" "style:<pack>;page:<page>"` line, and its builder in
`sheet/src/main.ts`.

## Make a new kit

Copy this directory to `assets/ui-kits/<name>/`, rename `name` in
`extension.json` and the namespace in `index.ts`
(`registerUiExtension("<name>", { … })`), then replace the components.
Nothing outside the directory lists kits.
