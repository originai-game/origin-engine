import { og, type World } from "@origingame/engine";

function must<T>(result: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!result.ok) throw new Error(result.reason);
  return result.value;
}

function style(world: World): void {
  const font = must(og.load.font(world, "fonts/InstrumentSans-Regular.ttf"));
  const texture = must(og.load.texture(world, "icons/ink-icons.png"));
  const names = [
    "heart", "star", "coin", "lock", "pause", "gear", "sword", "shield",
    "potion", "gem", "key", "map", "bolt", "clock", "trophy", "home",
    "plus", "close", "check", "play", "back", "bag", "bomb", "crown",
    "flag", "apple", "leaf", "fire", "drop", "skull", "music", "person",
  ];
  og.ui.configure(world, { style: null, font, fallbackIcons: { texture, columns: 8, rows: 4, names } });
}

export default og.game({
  tick: 60,
  seed: 1,
  start(world) {
    style(world);
    must(og.load.document(world, "ui/sheet.svelte"));
  },
  ui(world) {
    const scope = og.ui.scope(world, "sheet");
    const actions = scope.collection("actions", [
      { id: "jump", action: "jump", label: "Jump", icon: "", primary: true, charge: -1, disabled: false },
      { id: "dash", action: "dash", label: "Dash", icon: "", primary: false, charge: 0.6, disabled: false },
      { id: "use", action: "use", label: "Use", icon: "", primary: false, charge: -1, disabled: true },
    ], { key: "id" });
    const slots = scope.collection("slots", [
      { id: "arc", name: "Arc", icon: "bolt", action: "", hotkey: "Q", cooldown: 6, cooldownMax: 10, cost: "", locked: false, active: false, unaffordable: false },
      { id: "ward", name: "Ward", icon: "shield", action: "", hotkey: "W", cooldown: 0, cooldownMax: 10, cost: "40", locked: false, active: true, unaffordable: false },
      { id: "nova", name: "Nova", icon: "star", action: "", hotkey: "E", cooldown: 0, cooldownMax: 10, cost: "", locked: true, active: false, unaffordable: false },
    ], { key: "id" });
    const pois = scope.collection("pois", [
      { id: "camp", bearing: 0, icon: "flag", kind: "objective", distance: 240, label: "Camp" },
      { id: "ally", bearing: 0.6, icon: "", kind: "ally", distance: 90, label: "Scout" },
    ], { key: "id" });
    const markers = scope.collection("markers", [
      { id: "ally", kind: "ally", x: 0.45, y: 0.42, label: "Scout", tone: "success", player: false },
      { id: "me", kind: "poi", x: 0.5, y: 0.5, label: "You", tone: "neutral", player: true },
      { id: "quest", kind: "objective", x: 0.58, y: 0.55, label: "Camp", tone: "warn", player: false },
    ], { key: "id" });
    const items = scope.collection("items", [
      { id: "heal", label: "Heal", icon: "heart", at: 0, cost: "", unaffordable: false, disabled: false },
      { id: "ward", label: "Ward", icon: "shield", at: 1 / 6, cost: "150", unaffordable: false, disabled: false },
      { id: "bomb", label: "Bomb", icon: "bomb", at: 2 / 6, cost: "300", unaffordable: true, disabled: false },
      { id: "map", label: "Map", icon: "map", at: 3 / 6, cost: "", unaffordable: false, disabled: false },
      { id: "bag", label: "Bag", icon: "bag", at: 4 / 6, cost: "", unaffordable: false, disabled: true },
      { id: "rally", label: "Rally", icon: "flag", at: 5 / 6, cost: "", unaffordable: false, disabled: false },
    ], { key: "id" });
    const overlays = scope.collection("overlays", [
      { id: "heat", name: "Heat", icon: "fire", image: 0, ramp: "var(--info), var(--warn), var(--danger)", low: "Cold", high: "Hot" },
      { id: "water", name: "Water", icon: "drop", image: 0, ramp: "var(--panel), var(--info)", low: "Dry", high: "Flooded" },
      { id: "value", name: "Land value", icon: "coin", image: 0, ramp: "var(--panel), var(--gold)", low: "Low", high: "High" },
    ], { key: "id" });
    must(scope.mount("sheet", "HudSheet", { page: scope.writable("page", "meters"), actions, slots, pois, markers, items, overlays }));
  },
  commands: {
    style,
    page(world, args) { og.ui.scope(world, "sheet").writable("page", "meters").set(args[0] ?? "meters"); },
  },
});
