# Screen documents

Generic screen documents built from og-ui's public elements. Copy one with
`og ui eject <screen>` and edit it in `assets/ui/<screen>.ogt`.
Genre kits are being rebuilt and are not bundled yet. Local kits install
with `og ui add <directory>` and their documents can be copied with
`og ui eject <kit>/<screen>`. The rebuild catalogue and acceptance contract
are in `docs/ui.md` §4 and §6.

Each document says at its top what its viewmodel holds (its `props`), the
events it emits to code, and its named slots. Structure belongs in these
documents, look in CSS and tokens, and data in named sources. The retained
authoring API mounts documents once with named inputs and uses keyed
collections for lists. Its signatures are pending the core API landing;
the generated API describes currently implemented calls.

Every word a document shows is a string key with an English fallback
(`key "screen.inventory.title"`, `text "Inventory"`): a game's string
tables (`assets/strings/<locale>.ogt`) translate it, and a missing key
shows the fallback. Each has a `fixture default` viewmodel, so
`ui:push:<screen>:default` previews it with no code. Where an og.ui
pattern fills a slot and draws its own title, tabs or actions, the
document leaves those to it (pass the pattern an empty `title`).

`og ui eject --list` prints these generic screens. Core document tests
cook them independently of genre kits and style packs.

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
