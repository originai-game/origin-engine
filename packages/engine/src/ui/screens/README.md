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
look in scoped CSS and tokens, and data in named sources. A screen reads
its declared props bare (`{title}`, `bind:open={open}`), from a pushed
screen's viewmodel or a mount's stores of those names; `$toasts` and
`$captions` are the engine's stores. Name actions with `onclick={action}`,
and pass named content with
`{#snippet name()}…{/snippet}` and `{@render name?.()}`. TypeScript builds no
UI trees. The generated API describes the mount and store signatures.

Visible words use `t("screen.name.key", { argument: expression })`.
Each document names a companion English string table with
`<meta name="strings" content="./screen.en.ogt">`. Eject copies both files.
Game translations override those base messages. Each screen declares a
`<meta name="fixture:default" content="…">` JSON fixture for previews.

The component documents here (`card`, `confirm` and the lists, tables and
HUD feedback rows) use the explicit component form with the exact name
they declare: `<ui:card>…</ui:card>`, `<ui:star-rating stars={$stars}
label={$label}/>`, `<ui:key-values rows={$rows}/>` and `<ui:banner
title={$title} subtitle={$subtitle}/>`; the prefix is not part of the name
or manifest key. This keeps `<ui:table>` separate from native `<table>`.
Capitalised component names also keep their unprefixed form (`<StatBar/>`).
They can also be ejected and edited. Lists are keyed
collections; a nested list is flattened, as a retained `{#each}` reads a
named collection (a table's `cells` row by row, a tree's visible `rows`).

The tutorial points at an element with `target`, the element's id in
any document under it (a HUD button): the screen dims around the element,
which stays lit and takes its presses while the dimmed rest takes none,
and the card stands against it. Without a target nothing is dimmed and
the card stands at `x` and `y`. Its input mode is `all`, so the player
can do what a waiting step asks for; a game that wants a step read
before play goes on holds its own input until `next`.

`og ui eject --list` prints these generic screens. og-ui's
`tests/component_documents.rs` builds each from its fixtures, and
og-tests-ui's `tests/document_frames.rs` draws its `fixture:default` in
the default style against its reference frames (`references/vulkan/`); the fantasy, scifi and cartoon packs dress them through the role parts and
the core controls.

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
| Interactive film (og.film: the window's choices with the time left, hotspots on the picture and quick-time progress; the story-so-far flowchart; chapter select; the ending gallery) | `film-choices`, `film-flowchart`, `film-chapters`, `film-gallery` |
| Card, confirm dialog | `card`, `confirm` |
| Lists and tables (data list, key-value and stat compare, table, tree, calendar, pagination) | `data-list`, `key-values`, `table`, `tree`, `calendar`, `pagination` |
| HUD feedback (objectives, rating, banner, subtitles, notifications) | `objective-tracker`, `star-rating`, `banner`, `subtitles`, `notifications` |

## Parts

Each document's parts as its `<meta name="parts">` declares them
(`pause.frame` in a selector). A part's role name (`frame`, `band`,
`title`, `row`, `cell`…) takes the default style's role dressing
(docs/ui.md §3.1). Each part's state qualifiers are listed after it: a
class (`.selection-on`), an attribute (`[aria-selected]`) or `:disabled`.
A part that is a button also has the control's own hover, pressed,
focus-visible and disabled states (docs/ui.md §3.2).

| Document | Parts |
| --- | --- |
| `banner` (`banner.svelte`) | `frame`, `title`, `subtitle` |
| `calendar` (`calendar.svelte`) | `frame`, `band`, `title`, `heading`, `cell` (a button: `.marked` `.outside` `[aria-current]`) |
| `calibration` (`calibration.svelte`) | `frame`, `label` |
| `card` (`card.svelte`) | `frame`, `image`, `title`, `body` |
| `confirm` (`confirm.svelte`) | `title`, `body`, `actions` |
| `creator-tools` (`creator-tools.svelte`) | `frame` (`.chosen`), `label`, `title` |
| `credits` (`credits.svelte`) | `frame`, `heading`, `name`, `shade`, `rule` (`.end`), `role` |
| `data-list` (`data-list.svelte`) | `frame`, `row` (a button), `label`, `figure`, `note` |
| `film-chapters` (`film-chapters.svelte`) | `frame`, `band`, `title`, `card` (`.locked` `.selected` `[aria-disabled]` `:disabled`), `thumb`, `name` |
| `film-choices` (`film-choices.svelte`) | `frame`, `timer`, `choice` (a button: `.chosen` `[aria-current]` `:disabled`), `hotspot` (a button: `:disabled`), `qte` |
| `film-flowchart` (`film-flowchart.svelte`) | `frame`, `band`, `title`, `detail` |
| `film-gallery` (`film-gallery.svelte`) | `frame`, `band`, `title`, `card` (`.locked` `.selected` `[aria-disabled]` `:disabled`), `thumb`, `name` |
| `game-over` (`game-over.svelte`) | `frame`, `title`, `figure`, `label`, `offer` |
| `hud-editor` (`hud-editor.svelte`) | `hint`, `dock` |
| `key-values` (`key-values.svelte`) | `frame`, `row`, `label`, `figure`, `delta` (`.up`) |
| `leaderboard` (`leaderboard.svelte`) | `frame`, `band`, `title`, `league`, `row` (`.mine-other` `.mine-you`), `rank`, `figure`, `me` |
| `loading` (`loading.svelte`) | `scrim`, `frame` (the tip plate), `title`, `eyebrow`, `label`, `next` (a button), `bar` (`[data-ready]`), `fill`, `figure` (`[data-ready]`), `status` |
| `main-menu` (`main-menu.svelte`) | `frame`, `logo`, `version`, `shade`, `entry` (a button: `.minor`), `news`, `name` |
| `mode-select` (`mode-select.svelte`) | `frame`, `band`, `title`, `cell` (`.lock-locked` `.lock-open` `.selection-off` `.selection-on`), `plate`, `name`, `shade`, `foot`, `tag` (`.fresh`), `lock` |
| `notifications` (`notifications.svelte`) | `frame` (`.danger` `.leaving` `.success` `.warn`), `body` |
| `objective-tracker` (`objective-tracker.svelte`) | `frame`, `title`, `row` (`.done`), `well`, `fill` |
| `pagination` (`pagination.svelte`) | `frame`, `gap` |
| `pause` (`pause.svelte`) | `frame`, `title`, `objective` |
| `results` (`results.svelte`) | `frame`, `title` (`.outcome-lost` `.outcome-won`), `grade`, `row`, `label`, `figure`, `cell` |
| `save-load` (`save-load.svelte`) | `frame`, `band`, `title`, `row` (`.selection-off` `.selection-on`), `thumb`, `name` |
| `settings` (`settings.svelte`) | `frame`, `band`, `title`, `row` |
| `splash` (`splash.svelte`) | `frame`, `mark`, `dots`, `dot` (`.on`), `boot`, `track`, `fill`, `skip` |
| `star-rating` (`star-rating.svelte`) | `frame` |
| `subtitles` (`subtitles.svelte`) | `frame`, `line`, `speaker` |
| `table` (`table.svelte`) | `frame`, `band`, `heading`, `entry` (a button) |
| `title` (`title.svelte`) | `frame`, `logo`, `prompt`, `shade`, `rule` (`.end`), `name` |
| `tree` (`tree.svelte`) | `row` (`[aria-expanded]` `[aria-selected]`) |
| `tutorial` (`tutorial.svelte`) | `frame`, `title`, `text`, `step` |
