import { og } from "@origingame/engine";
import fixture from "./inbox-repro.json";
export default og.game({
  start(world) {
    const font = og.load.font(world, "fonts/InstrumentSans-Regular.ttf");
    if (!font.ok) throw new Error(font.reason);
    const icons = og.load.texture(world, "icons/ink-icons.png");
    if (!icons.ok) throw new Error(icons.reason);
    og.ui.configure(world, { style: null, font: font.value, fallbackIcons: { texture: icons.value, columns: 8, rows: 4, names: ["heart", "star", "coin", "lock", "pause", "gear", "sword", "shield", "potion", "gem", "key", "map", "bolt", "clock", "trophy", "home", "plus", "close", "check", "play", "back", "bag", "bomb", "crown", "flag", "apple", "leaf", "fire", "drop", "skull", "music", "person"] } });
    const page = og.load.document(world, "ui/inbox-repro.svelte");
    if (!page.ok) throw new Error(page.reason);
  },
  ui(world) {
    const scope = og.ui.scope(world, "repro");
    const mounted = scope.mount("page", "InboxRepro", {
      selected: scope.writable("selected", "welcome"),
      summary: scope.writable("summary", "1 unread · 2 / 50"),
      messages: scope.collection("messages", fixture.messages, { key: "id" }),
      attachments: scope.collection("attachments", fixture.attachments, { key: "id" }),
    });
    if (!mounted.ok) throw new Error(mounted.reason);
  },
});
