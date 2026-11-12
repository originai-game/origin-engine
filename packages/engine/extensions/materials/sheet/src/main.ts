// The materials pack's review sheet (docs/ui.md §6): one card per preset
// of ../../*.ogt, each composited through its material by the document's
// `-og-material: "materials/materials/<preset>.ogt"`. The pack is installed
// as a game installs it.
//
// Commands: scheme:<light|dark>, burn:<0–1> (the burn card's progress).

import { og } from "@origingame/engine";

export default og.game({
  tick: 60,
  seed: 1,
  start(world) {
    const camera = world.resource(og.Camera);
    if (camera.ok) camera.value.background = { r: 0.08, g: 0.09, b: 0.11 };
    const font = og.load.font(world, "fonts/InstrumentSans-Regular.ttf");
    const icons = og.load.texture(world, "icons/ink-icons.png");
    if (!font.ok) throw new Error(font.reason);
    if (!icons.ok) throw new Error(icons.reason);
    og.ui.configure(world, {
      scheme: "light", font: font.value,
      icons: { texture: icons.value, columns: 8, rows: 4, names: [
        "heart", "star", "coin", "lock", "pause", "gear", "sword", "shield", "potion", "gem", "key",
        "map", "bolt", "clock", "trophy", "home", "plus", "close", "check", "play", "back", "bag",
        "bomb", "crown", "flag", "apple", "leaf", "fire", "drop", "skull", "music", "person",
      ] },
    });
    const loaded = og.load.document(world, "ui/sheet.svelte");
    if (!loaded.ok) throw new Error(loaded.reason);
  },
  ui(world) {
    const scope = og.ui.scope(world, "sheet");
    const burn = scope.writable("burn", 0.45);
    const mounted = scope.mount("sheet", "materialsSheet", { burn });
    if (!mounted.ok) throw new Error(mounted.reason);
  },
  commands: {
    scheme: (world, args) => og.ui.configure(world, { scheme: args[0] === "dark" ? "dark" : "light" }),
    burn: (world, args) => {
      og.ui.scope(world, "sheet").writable("burn", 0.45).set(Math.min(1, Math.max(0, Number(args[0] ?? 0.45))));
    },
  },
  commandHelp: {
    scheme: { usage: "<light|dark>", doc: "The core default's colour scheme." },
    burn: { usage: "<0–1>", doc: "The burn card's progress." },
  },
});
