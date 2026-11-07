// The materials kit's review sheet (docs/ui.md §6): one card per preset
// of ../../*.ogt, each a column drawn offscreen and composited through its
// material (og.ui.setDraw's layer). The kit is installed as a game installs
// it, so the cards load `ui-kits/materials/<preset>.ogt`.
//
// Commands: scheme:<light|dark>, burn:<0–1> (the burn card's progress).

import { og, defineSystem, type UiMaterial } from "@origingame/engine";

const PRESETS = ["foil", "holo", "burn", "rarity"] as const;
const state = { burn: 0.45, materials: {} as Record<string, UiMaterial> };

/** Each preset's parameters on the sheet, in its `param` order (16 numbers: four vec4s). */
function params(name: string): number[] {
  switch (name) {
    case "burn": return [state.burn, 0, 0, 0, 6, 0, 0, 0, 40, 0, 0, 0, 1.0, 0.36, 0.05, 0];
    // The band held still across the art, so every capture shows it.
    case "foil": return [0.7, 0, 0, 0, 1.5, 0, 0, 0, 0, -0.12, 0, 0, 0.85, 0.92, 1.0, 0];
    default: return [];
  }
}

function card(id: string, title: string, material: string | null): void {
  og.ui.column(id, { width: { kind: "fixed", px: 168 }, height: { kind: "fixed", px: 236 }, padding: { left: 14, top: 14, right: 14, bottom: 14 }, gap: 10, background: { token: "panelRaised" }, radius: 12, cross: "center" }, () => {
    const handle = material ? state.materials[material] : undefined;
    if (handle) og.ui.setDraw({ layer: { material: "custom", custom: handle, params: params(material!) } });
    og.ui.column(`${id}Art`, { width: { kind: "grow" }, height: { kind: "fixed", px: 132 }, background: { token: "primary" }, radius: 8, align: "center", cross: "center" }, () => {
      og.ui.icon(`${id}Icon`, "crown", { size: 64 });
    });
    og.ui.label(`${id}Title`, title, { step: "lg" });
    og.ui.label(`${id}Note`, material ?? "no material", { step: "caption" });
  });
}

const Sheet = defineSystem("Sheet", { reads: [og.Camera] }, () => {
  const vp = og.ui.frameViewport();
  const s = vp.safeArea;
  og.ui.row("Page", {
    width: { kind: "grow" }, height: { kind: "grow" }, background: { token: "backdrop" },
    padding: { left: s.left + 24, top: s.top + 24, right: s.right + 24, bottom: s.bottom + 24 },
    gap: 16, wrap: true, align: "center", cross: "center",
  }, () => {
    card("Plain", "Common", null);
    card("Foil", "Foil", "foil");
    card("Holo", "Holo", "holo");
    card("Burn", "Burn", "burn");
    card("Rarity", "Legendary", "rarity");
  });
});

export default og.game({
  tick: 60,
  seed: 1,
  systems: [Sheet],
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
    for (const p of PRESETS) {
      const m = og.load.uiMaterial(world, `ui-kits/materials/${p}.ogt`);
      if (!m.ok) throw new Error(m.reason);
      state.materials[p] = m.value;
    }
  },
  commands: {
    scheme: (world, args) => og.ui.configure(world, { scheme: args[0] === "dark" ? "dark" : "light" }),
    burn: (_world, args) => { state.burn = Math.min(1, Math.max(0, Number(args[0] ?? 0.45))); },
  },
  commandHelp: {
    scheme: { usage: "<light|dark>", doc: "The core default's colour scheme." },
    burn: { usage: "<0–1>", doc: "The burn card's progress." },
  },
});
