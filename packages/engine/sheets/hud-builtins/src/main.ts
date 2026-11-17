import { og, type World } from "@origingame/engine";

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}
function style(w: World): void {
  const texture = must(og.load.texture(w, "icons/ink-icons.png"));
  const names = ["heart", "star", "coin", "lock", "pause", "gear", "sword", "shield", "potion", "gem", "key", "map", "bolt", "clock", "trophy", "home", "plus", "close", "check", "play", "back", "bag", "bomb", "crown", "flag", "apple", "leaf", "fire", "drop", "skull", "music", "person"];
  og.ui.configure(w, { style: null, font: null, fallbackIcons: { texture, columns: 8, rows: 4, names } });
}
export default og.game({
  tick: 60, seed: 1,
  start(w) { style(w); must(og.load.document(w, "ui/sheet.svelte")); },
  ui(w) {
    const s = og.ui.scope(w, "sheet");
    const lives = s.collection("lives", [{ id: "a", value: 1, label: "First life" }, { id: "b", value: 0.5, label: "Second life" }, { id: "c", value: 0, label: "Third life" }], { key: "id" });
    const full = s.collection("full", [{ id: "a", value: 1 }, { id: "b", value: 1 }, { id: "c", value: 1 }], { key: "id" });
    const empty = s.collection("empty", [], { key: "id" });
    const depleted = s.collection("depleted", [{ id: "last", value: 0 }], { key: "id" });
    const phases = s.collection("phases", [{ id: "shell", label: "Shell", complete: true }, { id: "storm", label: "Storm", complete: false }, { id: "core", label: "Core", complete: false }], { key: "id" });
    const pickups = s.collection("pickups", [{ id: "ore", name: "Iron ore", icon: "gem", quantity: 12, detail: "Crafting material" }, { id: "key", name: "Observatory key", icon: "key", quantity: 1, detail: "Quest item" }, { id: "gold", name: "Gold", icon: "coin", quantity: -40, detail: "Supplies purchased" }], { key: "id" });
    const longPickup = s.collection("longPickup", [{ id: "relic", name: "Fragment of the northern observatory mechanism", icon: "gem", quantity: 123456, detail: "Recovered from the abandoned expedition storehouse" }], { key: "id" });
    must(s.mount("page", "HudBuiltinsSheet", { page: s.writable("page", "resources"), lives, full, empty, depleted, phases, pickups, longPickup }));
  },
  commands: {
    style,
    page(w, args) { og.ui.scope(w, "sheet").writable("page", "resources").set(args[0] ?? "resources"); },
  },
});
