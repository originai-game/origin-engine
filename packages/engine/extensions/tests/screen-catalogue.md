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
  rpg.inventoryGrid(/* … */);
  og.ui.close();
}
for (const e of r.events) if (e.kind === "emit" && e.name === "sort") sort(vm);
```

Every word a document shows is a string key with an English fallback
(`key "screen.inventory.title"`, `text "Inventory"`): a game's string
tables (`assets/strings/<locale>.ogt`) translate it, and a missing key
shows the fallback. Each has a `fixture default` viewmodel, so
`ui:push:<screen>:default` previews it with no code. Where an og.ui
pattern fills a slot and draws its own title, tabs or actions, the
document leaves those to it (pass the pattern an empty `title`).

`crates/og-cook/tests/documents.rs` reads the table below: its rows are
Appendix B's, in order, and every document it names cooks. `og ui eject
--list` prints it.
A row whose cell is — has no document since its kit was removed; 0.8.0's
UI work rebuilds those screens.

| Appendix B row | Documents |
|---|---|
| Boot, splash, title, main menu, mode select | `splash`, `title`, `main-menu`, `mode-select` |
| Settings (graphics, audio, controls and remapping, accessibility, language, HUD layout editor) | `settings`, `hud-editor` |
| Pause and system menu | `pause` |
| Save and load with thumbnails | `save-load` |
| Loading with tips, lore and progress | `loading` |
| Character creation (sliders, presets, turntable) | `rpg/character-creation` |
| Character, loadout, paperdoll and stats | `rpg/character` |
| Inventory (grid, list, weighted, nested containers) | `rpg/inventory` |
| Stash, storage, bank | `rpg/stash` |
| Skill, talent, passive and tech trees | `rpg/skill-tree` |
| Crafting and recipe book | `rpg/crafting` |
| Shop and vendor in the world | `rpg/vendor` |
| Store (bundles, rotations) | — |
| Gacha banner and pull results; pack opening | `gacha/gacha-banner`, `gacha/pull-results` |
| Battle pass and season track | — |
| Daily login, missions, events hub | — |
| Mailbox | — |
| World map (fog, markers, fast travel) | `rpg/world-map` |
| Level and saga map | `puzzle/saga-map` |
| Run map (branching nodes) | — |
| Reward choice (pick N of M) | — |
| Quest log, journal, codex, bestiary, encyclopedia | `rpg/quest-log`, `rpg/codex` |
| Dialogue (choices, checks, timers) | `rpg/dialogue` |
| Backlog, gallery, route flowchart | — |
| Deck builder and collection | — |
| Roster, team builder, character detail and ascension | `gacha/roster`, `gacha/character-detail` |
| Character select and versus | — |
| Draft and ban with timers | — |
| Lobby, party, matchmaking, ready check | — |
| Scoreboard | `shooter/scoreboard` |
| Results and post-match (grades, XP cascades, graphs) | `results`, `racing/race-results` |
| Leaderboards (global, friends, leagues) | `leaderboard` |
| Song and track select with preview | — |
| Garage, tuning, livery editor | `racing/garage` |
| Management dashboards (ledgers, budgets, graphs, overlays) | `strategy/management` |
| Diplomacy and trade | `strategy/diplomacy` |
| Auction house and market | `rpg/auction-house` |
| Guild, friends, chat | — |
| Training mode with frame data | — |
| Calibration (audio offset, brightness, controller) | `calibration` |
| Photo mode | — |
| Replay and spectator | — |
| Prestige and meta-progression hub | — |
| Tutorial and onboarding overlays | `tutorial` |
| Death, retry, revive offer, game over | `game-over` |
| Creator tools (inspector, hierarchy, catalogues) | `creator-tools` |
| Credits | `credits` |
