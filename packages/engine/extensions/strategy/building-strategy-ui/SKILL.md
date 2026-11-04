---
name: building-strategy-ui
description: Builds strategy, RTS and tower-defence UI with the strategy kit (selection panel and control groups, command card and its pages, production queue, the RTS console, drag-select, the resource bar, the wave preview) over Origin Engine's common components. Use when a strategy or TD game needs its console, economy or waves, or when editing, ejecting or reviewing the strategy kit.
---

# Strategy kit

A kit written over the engine's public API. It runs unchanged on the web
and native hosts.

## Find what is there

```sh
og ui list                  # every kit and style, each kit's documents
og ui list --json           # the same, for a script
```

The namespace is `strategy`. Members, with their desc types in `types.ts`
(every field has a doc comment and a default, `{}` is valid):

| Member | What it draws |
|---|---|
| `rtsCommands(id, RtsCommandsDesc)` | The whole console with command pages, targeting and hotkeys; answers RtsCommandsResult |
| `rtsConsole(id, RtsConsoleDesc)` | Minimap, selection and command card in one bar |
| `selectionPanel / commandCard / productionQueue` | The console's parts on their own |
| `dragSelect(id, DragSelectDesc)` | The box-select rectangle and its result |
| `resourceBar(id, ResourceBarDesc)` | Any number of resources, signed rates, caps, the game clock |
| `wavePreview(id, WavePreviewDesc)` | The waves to come, the boss marked |

`fixtures/base.ts` has a selection with its orders and production, an economy and the next waves in these shapes: start from it.

## Use it

```ts
import { strategy } from "@origingame/engine/extensions/strategy/index.ts";

strategy.resourceBar("Economy", { items: [{ key: "gold", icon: "coin", amount: 1240, rate: 14 }], anchor: "top" });
const r = strategy.rtsCommands("Rts", { console, pages, page });
```

Call them from the system that builds the game's UI each tick. Generic
pieces stay in `og.ui`: the minimap, ability slots, tables, graphs, progress and timers.

## Change it

1. `og ui add strategy` copies the kit into `assets/ui-kits/strategy/`. The copy
   is yours: edit any file. It overrides the bundled kit in `og ui list`.
2. Import the copy instead: `import { strategy } from "../assets/ui-kits/strategy/index.ts";`
   (the path from the importing file).
3. A component is ordinary TypeScript in `src/`. Add a field to its desc
   in `types.ts`, then read it in the component. Parts it opens
   (`sel("ResourceBar", "frame")`) are what a style's `dress` block restyles, so a look change
   belongs in the style, not here.
4. Helpers come from `@origingame/engine/ui-parts/hud` and
   `@origingame/engine/ui-parts/screen` only. Never import engine
   files by path.

## Review it

```sh
og ui sheet strategy           # the installed copy if there is one, else the bundled kit
```

It renders every page in `sheet/project.ogt` at desktop and phone, 100 %
and 150 %, into `og-out/ui-sheet/strategy/`. Add a page with one more
`screen "<name>" "style:<pack>;page:<page>"` line, and its builder in
`sheet/src/main.ts`.

## Make a new kit

Copy this directory to `assets/ui-kits/<name>/`, rename `name` in
`extension.json` and the namespace in `index.ts`
(`registerUiExtension("<name>", { … })`), then replace the components.
Nothing outside the directory lists kits.
