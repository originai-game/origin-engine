---
name: building-rpg-ui
description: Builds RPG and MMO UI with the RPG kit (grid inventory, paperdoll equipment, quest log and tracker, dialogue with choices and checks, unit, party and raid frames, cast bar, turn order) over Origin Engine's common components. Use when an RPG needs its inventory, character, journal, conversation or combat frames, or when editing, ejecting or reviewing the RPG kit.
---

# RPG kit

A kit written over the engine's public API. It runs unchanged on the web
and native hosts.

## Find what is there

```sh
og ui list                  # every kit and style, each kit's documents
og ui list --json           # the same, for a script
```

The namespace is `rpg`. Members, with their desc types in `types.ts`
(every field has a doc comment and a default, `{}` is valid):

| Member | What it draws |
|---|---|
| `inventoryGrid(id, InventoryGridDesc)` | Multi-cell items with stacks, rarity, drag and drop; answers InventoryGridResult |
| `paperdoll(id, PaperdollDesc, figure)` | Equipment slots round the game's figure, stats with deltas |
| `questLog(id, QuestLogDesc)` | og.story quests with tabs, objectives and rewards |
| `questTracker(id, QuestTrackerDesc)` | The tracked quests' objectives over the game |
| `conversation(id, ConversationDesc)` | An og.story line with its speaker, choices, checks and rolls |
| `unitFrame / partyFrames / raidFrames` | Portraits, health, resource, effects; party and raid grids |
| `castBar(id, CastBarDesc)` | A cast or channel with its interrupt states |
| `turnOrder(id, TurnOrderDesc)` | Who acts next; any number of teams |

`fixtures/party.ts` has a party's bag, gear, quests, one conversation and its frames in these shapes: start from it.

## Use it

```ts
import { rpg } from "@origingame/engine/extensions/rpg/index.ts";

const r = rpg.inventoryGrid("Bag", { columns: 6, rows: 4, items });
rpg.conversation("Talk", { view: og.story.view(world, talk) });
```

Call them from the system that builds the game's UI each tick. Generic
pieces stay in `og.ui`: lists, slots, stat bars, tooltips, stat comparison, world tags and speech bubbles, story.backlog.

## Change it

1. `og ui add rpg` copies the kit into `assets/ui-kits/rpg/`. The copy
   is yours: edit any file. It overrides the bundled kit in `og ui list`.
2. Import the copy instead: `import { rpg } from "../assets/ui-kits/rpg/index.ts";`
   (the path from the importing file).
3. A component is ordinary TypeScript in `src/`. Add a field to its desc
   in `types.ts`, then read it in the component. Parts it opens
   (`sel("UnitFrame", "frame")`) are what a style's `dress` block restyles, so a look change
   belongs in the style, not here.
4. Helpers come from `@origingame/engine/ui-parts/hud` and
   `@origingame/engine/ui-parts/screen` only. Never import engine
   files by path.

## Review it

```sh
og ui sheet rpg           # the installed copy if there is one, else the bundled kit
```

It renders every page in `sheet/project.ogt` at desktop and phone, 100 %
and 150 %, into `og-out/ui-sheet/rpg/`. Add a page with one more
`screen "<name>" "style:<pack>;page:<page>"` line, and its builder in
`sheet/src/main.ts`.

## Make a new kit

Copy this directory to `assets/ui-kits/<name>/`, rename `name` in
`extension.json` and the namespace in `index.ts`
(`registerUiExtension("<name>", { … })`), then replace the components.
Nothing outside the directory lists kits.
