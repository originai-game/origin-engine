# Screen documents

The library's screen documents (docs/ui.md §11): one `type screen`
document or more for every screen type of docs/ui.md Appendix B, built
from og-ui's public elements (crates/og-ui/README.md §Documents). A game
does not mount them from here: it copies one into its own project with
`og ui eject <screen>` (into `assets/ui/<screen>.ogt`) and edits it as its
own file, so no screen is limited by a template's fields.

Each document says at its top what its viewmodel holds (its `props`), the
events it emits to code, and which slot takes which og.ui component or
pattern. Code loads and mounts it:

```ts
og.load.document(world, "ui/inventory.ogt");    // once
og.ui.push(world, "inventory", vm);             // onto the layer stack
const r = og.ui.screen("inventory", vm);        // each tick it is up, to fill its slots
if (og.ui.openSlot("inventory/Grid")) {         // <screen>/<slot id>
  og.ui.inventoryGrid(/* … */);
  og.ui.close();
}
for (const e of r.events) if (e.kind === "emit" && e.name === "sort") sort(vm);
```

Every word a document shows is a string key with an English fallback
(`key "screen.inventory.title"`, `text "Inventory"`): a game's string
tables (`assets/strings/<locale>.ogt`) translate it, and a missing key
shows the fallback. Each has a `fixture default` viewmodel, so
`ui:push:<screen>:default` previews it with no code. A slot marked
`PATTERN PENDING` is where a pattern still being written will mount;
until it lands, code fills that slot from the components that exist.

`crates/og-cook/tests/documents.rs` reads the table below: its rows are
Appendix B's, in order, every document cooks, and the third column names
exactly the documents with a pending slot. `og ui eject --list` prints it.

| Appendix B row | Documents | Pattern pending |
|---|---|---|
| Boot, splash, title, main menu, mode select | `splash`, `title`, `main-menu`, `mode-select` | |
| Settings (graphics, audio, controls and remapping, accessibility, language, HUD layout editor) | `settings`, `hud-editor` | the settings tree in `settings`; the HUD layout editor in `hud-editor` |
| Pause and system menu | `pause` | |
| Save and load with thumbnails | `save-load` | |
| Loading with tips, lore and progress | `loading` | |
| Character creation (sliders, presets, turntable) | `character-creation` | |
| Character, loadout, paperdoll and stats | `character` | the loadout in `character` |
| Inventory (grid, list, weighted, nested containers) | `inventory` | |
| Stash, storage, bank | `stash` | |
| Skill, talent, passive and tech trees | `skill-tree` | |
| Crafting and recipe book | `crafting` | |
| Shop and vendor in the world | `vendor` | |
| Store (bundles, rotations) | `store` | |
| Gacha banner and pull results; pack opening | `gacha-banner`, `pull-results` | |
| Battle pass and season track | `battle-pass` | |
| Daily login, missions, events hub | `daily-login`, `events-hub` | |
| Mailbox | `mailbox` | |
| World map (fog, markers, fast travel) | `world-map` | |
| Level and saga map | `saga-map` | |
| Run map (branching nodes) | `run-map` | |
| Reward choice (pick N of M) | `reward-choice` | |
| Quest log, journal, codex, bestiary, encyclopedia | `quest-log`, `codex` | |
| Dialogue (choices, checks, timers) | `dialogue` | |
| Backlog, gallery, route flowchart | `backlog`, `gallery`, `flowchart` | the gallery in `gallery`; the route flowchart in `flowchart` |
| Deck builder and collection | `deck-builder` | the deck builder in `deck-builder` |
| Roster, team builder, character detail and ascension | `roster`, `character-detail` | |
| Character select and versus | `character-select`, `versus` | |
| Draft and ban with timers | `draft` | |
| Lobby, party, matchmaking, ready check | `lobby`, `matchmaking` | |
| Scoreboard | `scoreboard` | |
| Results and post-match (grades, XP cascades, graphs) | `results` | |
| Leaderboards (global, friends, leagues) | `leaderboard` | |
| Song and track select with preview | `song-select` | |
| Garage, tuning, livery editor | `garage` | garage, tuning and livery in `garage` |
| Management dashboards (ledgers, budgets, graphs, overlays) | `management` | the ledgers in `management` |
| Diplomacy and trade | `diplomacy` | |
| Auction house and market | `auction-house` | |
| Guild, friends, chat | `social` | guild, friends and chat in `social` |
| Training mode with frame data | `training` | |
| Calibration (audio offset, brightness, controller) | `calibration` | |
| Photo mode | `photo-mode` | photo mode in `photo-mode` |
| Replay and spectator | `replay` | |
| Prestige and meta-progression hub | `prestige` | |
| Tutorial and onboarding overlays | `tutorial` | |
| Death, retry, revive offer, game over | `game-over` | |
| Creator tools (inspector, hierarchy, catalogues) | `creator-tools` | |
| Credits | `credits` | |
