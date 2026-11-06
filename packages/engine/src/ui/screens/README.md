# Screen documents

Generic screen documents built from og-ui's public elements. Copy one with
`og ui eject <screen>` and edit it in `assets/ui/<screen>.ogt`.
Genre screens belong to `extensions/<kit>`: install with `og ui add <kit>`
and copy with `og ui eject <kit>/<screen>` (for example `rpg/inventory`).
The full Appendix B catalogue is `extensions/tests/screen-catalogue.md`.

Each document says at its top what its viewmodel holds (its `props`), the
events it emits to code, and which slot takes which og.ui component or
pattern. Code loads and mounts it:

```ts
import { rpg } from "@origingame/engine/extensions/rpg/index.ts";

og.load.document(world, "ui/rpg/inventory.ogt");    // once
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

`og ui eject --list` prints these generic screens. The kits lane cooks
them together with the genre screens against the full Appendix B catalogue.

| Appendix B row | Documents |
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
