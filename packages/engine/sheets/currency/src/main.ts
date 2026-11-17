import { og } from "@origingame/engine";

export default og.game({
  start(world) {
    const icons = og.load.texture(world, "icons/ink-icons.png");
    if (!icons.ok) throw new Error(icons.reason);
    og.ui.configure(world, { style: null, font: null, fallbackIcons: {
      texture: icons.value, columns: 8, rows: 4,
      names: ["heart", "star", "coin", "lock", "pause", "gear", "sword", "shield", "potion", "gem", "key", "map", "bolt", "clock", "trophy", "home", "plus", "close", "check", "play", "back", "bag", "bomb", "crown", "flag", "apple", "leaf", "fire", "drop", "skull", "music", "person"],
    } });
    const page = og.load.document(world, "ui/states.svelte");
    if (!page.ok) throw new Error(page.reason);
  },
  ui(world) {
    const mounted = og.ui.scope(world, "sheet").mount("page", "CurrencyStates", {});
    if (!mounted.ok) throw new Error(mounted.reason);
  },
  commands: { style() {}, page() {} },
});
