---
name: building-gacha-ui
description: Builds gacha UI with the gacha kit (the banner with its featured units, rates, pity and wish buttons; the summon sequence from charge to reveal and summary) over Origin Engine's common components. Use when a gacha or collection game needs its banners or pull results, or when editing, ejecting or reviewing the gacha kit.
---

# Gacha kit

A kit written over the engine's public API. It runs unchanged on the web
and native hosts.

## Find what is there

```sh
og ui list                  # every kit and style, each kit's documents
og ui list --json           # the same, for a script
```

The namespace is `gacha`. Members, with their desc types in `types.ts`
(every field has a doc comment and a default, `{}` is valid):

| Member | What it draws |
|---|---|
| `gachaBanner(id, GachaBannerDesc, art)` | Banner tabs, the hero art (drawn by the game's callback), featured units, pity, single and multi wish; answers GachaBannerResult |
| `pullSequence(id, PullSequenceDesc)` | Charge, one reveal per item by its stars, the summary with wish again; answers PullSequenceResult |

`fixtures/banner.ts` has three banners and a ten-pull in these shapes: start from it.

## Use it

```ts
import { gacha } from "@origingame/engine/extensions/gacha/index.ts";

const r = gacha.gachaBanner("Banner", { banners, current, single: 160, multi: 1600, balance }, () => og.ui.artSlot("Hero", { texture: heroArt }));
if (r.pull) token = startPull(r.pull);
```

Call them from the system that builds the game's UI each tick. Generic
pieces stay in `og.ui`: the store (`liveops.storeFront`), reward choice, cards, celebrations, counters and timers.

Tiers are the game's. A unit or pull item takes the engine's `rarity`
(common … mythic), or its own `tier: { label: "SSR", rank: 6, color:
"#ffcc33" }` over it: the label is the word on the reveal, the colour
tints it, and the rank orders the summary (5 and up shine and burst).

## Change it

1. `og ui add gacha` copies the kit into `assets/ui-kits/gacha/`. The copy
   is yours: edit any file. It overrides the bundled kit in `og ui list`.
2. Import the copy instead: `import { gacha } from "../assets/ui-kits/gacha/index.ts";`
   (the path from the importing file).
3. A component is ordinary TypeScript in `src/`. Add a field to its desc
   in `types.ts`, then read it in the component. Parts it opens
   (`sel("Gacha", "tab")`) are what a style's `dress` block restyles, so a look change
   belongs in the style, not here.
4. Helpers come from `@origingame/engine/ui-parts/hud` and
   `@origingame/engine/ui-parts/screen` only. Never import engine
   files by path.

## Review it

```sh
og ui sheet gacha           # the installed copy if there is one, else the bundled kit
```

It renders every page in `sheet/project.ogt` at desktop and phone, 100 %
and 150 %, into `og-out/ui-sheet/gacha/`. Add a page with one more
`screen "<name>" "style:<pack>;page:<page>"` line, and its builder in
`sheet/src/main.ts`.

## Make a new kit

Copy this directory to `assets/ui-kits/<name>/`, rename `name` in
`extension.json` and the namespace in `index.ts`
(`registerUiExtension("<name>", { … })`), then replace the components.
Nothing outside the directory lists kits.
