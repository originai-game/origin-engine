import { og, type World } from "@origingame/engine";
import { FIXTURES } from "./fixtures.ts";

function show(world: World) {
  const scope = og.ui.scope(world, "sheet");
  const key = scope.get(scope.writable("state", "default"));
  const vm = { ...FIXTURES.default, ...FIXTURES[key] };
  const result = scope.mount("page", "ProfileSheet", {
    vm: scope.writable(`${key}-vm`, vm),
    stats: scope.collection(`${key}-stats`, vm.stats as { id: string }[], { key: "id" }),
    actions: scope.collection(`${key}-actions`, vm.actions as { id: string }[], { key: "id" }),
  });
  if (!result.ok) throw new Error(result.reason);
}
export default og.game({
  start(world) {
    const icons = og.load.texture(world, "icons/ink-icons.png");
    if (!icons.ok) throw new Error(icons.reason);
    og.ui.configure(world, { style: null, font: null, fallbackIcons: { texture: icons.value, columns: 8, rows: 4, names: ["heart", "star", "coin", "lock", "pause", "gear", "sword", "shield", "potion", "gem", "key", "map", "bolt", "clock", "trophy", "home", "plus", "close", "check", "play", "back", "bag", "bomb", "crown", "flag", "apple", "leaf", "fire", "drop", "skull", "music", "person"] } });
    const page = og.load.document(world, "ui/sheet.svelte");
    if (!page.ok) throw new Error(page.reason);
  },
  ui: show,
  commands: {
    state(world, args) {
      const scope = og.ui.scope(world, "sheet");
      scope.unmount("page");
      scope.writable("state", "default").set(FIXTURES[args[0]!] ? args[0]! : "default");
      show(world);
    },
  },
});
