# Screen documents

Generic screen documents built from og-ui's public elements and built into
the engine. A game loads one by its id without a copy,
`og.load.document(world, "@ui/screens/pause.svelte")`, and shows it by its
name, `og.ui.push(world, "pause")`; the cook bundles each one the game's
code names, with the components it uses. Copy one with
`og ui eject <screen>` and edit it in `assets/ui/<screen>.svelte`; the
game's copy then replaces the built-in one, and loading the `@ui/` id loads
it. The catalogue and acceptance contract are in `docs/ui.md` §4 and §6.

Each document declares its name, layer, props, actions, and fixtures with
standard `<meta>` elements. Structure belongs in scriptless Svelte markup,
look in scoped CSS and tokens, and data in named sources. Bind sources with
`{$name}`, name actions with `onclick={action}`, and pass named content with
`{#snippet name()}…{/snippet}` and `{@render name?.()}`. TypeScript builds no
UI trees. The generated API describes the mount and store signatures.

Visible words use `t("screen.name.key", { argument: expression })`.
Each document names a companion English string table with
`<meta name="strings" content="./screen.en.ogt">`. Eject copies both files.
Game translations override those base messages. Each screen declares a
`<meta name="fixture:default" content="…">` JSON fixture for previews.

The conversion preserves the core screen frames except the save/load Auto
badge tint: CSS resolves the selected parent's background before the child's
tint. The approved desktop and phone reference changes have RMSE 0.000573
and 0.000607. Phone control sizes preserve their existing pixels.
The game-over entrance now targets its title directly; the old relative
timeline path missed it. Its 300 ms curve and error sound are retained.

The component documents (`card`, `confirm` and the lists, tables and HUD
feedback rows) are the TS builders' replacements: a game mounts one as
`<Card title={…}>` once it is loaded, or ejects and edits it. Lists are keyed
collections; a nested list is flattened, as a retained `{#each}` reads a
named collection (a table's `cells` row by row, a tree's visible `rows`).

The tutorial points at an element with `target`, the element's id in
any document under it (a HUD button): the screen dims around the element,
which stays lit and takes its presses while the dimmed rest takes none,
and the card stands against it. Without a target nothing is dimmed and
the card stands at `x` and `y`. Its input mode is `all`, so the player
can do what a waiting step asks for; a game that wants a step read
before play goes on holds its own input until `next`.

`og ui eject --list` prints these generic screens. Core document tests
cook them independently of style packs.

| Purpose | Documents |
|---|---|
| Boot, splash, title, main menu, mode select | `splash`, `title`, `main-menu`, `mode-select` |
| Settings (graphics, audio, controls and remapping, accessibility, language, HUD layout editor) | `settings`, `hud-editor` |
| Pause and system menu | `pause` |
| Save and load with thumbnails | `save-load` |
| Loading with tips, lore and progress | `loading` |
| Results and post-match (grades, XP cascades, graphs) | `results` |
| Leaderboards (global, friends, leagues) | `leaderboard` |
| Calibration (audio offset, brightness, controller) | `calibration` |
| Tutorial and onboarding overlays | `tutorial` |
| Death, retry, revive offer, game over | `game-over` |
| Creator tools (inspector, hierarchy, catalogues) | `creator-tools` |
| Credits | `credits` |
| Card, confirm dialog | `card`, `confirm` |
| Lists and tables (data list, key-value and stat compare, table, tree, calendar, pagination) | `data-list`, `key-values`, `table`, `tree`, `calendar`, `pagination` |
| HUD feedback (objectives, rating, banner, subtitles, notifications) | `objective-tracker`, `star-rating`, `banner`, `subtitles`, `notifications` |
