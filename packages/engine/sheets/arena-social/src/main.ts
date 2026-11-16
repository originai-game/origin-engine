import { og, type World } from "@origingame/engine";
import { FIXTURES } from "./fixtures.ts";

function show(world: World) {
  const scope = og.ui.scope(world, "sheet");
  const page = scope.writable("page", "ready");
  const state = scope.writable("state", "default");
  const fixtures = FIXTURES[scope.get(page)] ?? FIXTURES.ready!;
  const vm = { ...fixtures.default, ...fixtures[scope.get(state)] };
  const rows = (vm.players ?? vm.options ?? vm.targets ?? []) as { id: string }[];
  const key = `${scope.get(page)}-${scope.get(state)}`;
  const result = scope.mount("page", "ArenaSocialSheet", {
    page, vm: scope.writable(`${key}-vm`, vm), rows: scope.collection(`${key}-rows`, rows, { key: "id" }),
  });
  if (!result.ok) throw new Error(result.reason);
}
export default og.game({
  start(world) {
    const font = og.load.font(world, "fonts/InstrumentSans-Regular.ttf");
    if (!font.ok) throw new Error(font.reason);
    const icons = og.load.texture(world, "icons/ink-icons.png");
    if (!icons.ok) throw new Error(icons.reason);
    og.ui.configure(world, { style: null, font: font.value, fallbackIcons: { texture: icons.value, columns: 8, rows: 4, names: ["heart", "star", "coin", "lock", "pause", "gear", "sword", "shield", "potion", "gem", "key", "map", "bolt", "clock", "trophy", "home", "plus", "close", "check", "play", "back", "bag", "bomb", "crown", "flag", "apple", "leaf", "fire", "drop", "skull", "music", "person"] } });
    const page = og.load.document(world, "ui/sheet.svelte");
    if (!page.ok) throw new Error(page.reason);
  },
  ui: show,
  commands: {
    page(world, args) {
      const scope = og.ui.scope(world, "sheet");
      scope.unmount("page");
      scope.writable("page", "ready").set(FIXTURES[args[0]!] ? args[0]! : "ready");
      show(world);
    },
    state(world, args) {
      const scope = og.ui.scope(world, "sheet");
      scope.unmount("page");
      scope.writable("state", "default").set(args[0] || "default");
      show(world);
    },
  },
});
