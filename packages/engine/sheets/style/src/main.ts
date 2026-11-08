import { og } from "@origingame/engine";

export default og.game({
  start(world) {
    const loaded = og.load.document(world, "ui/sheet.svelte");
    if (!loaded.ok) throw new Error(loaded.reason);
  },
  ui(world) {
    const mounted = og.ui.scope(world, "sheet").mount("sheet", "styleSheet", {});
    if (!mounted.ok) throw new Error(mounted.reason);
  },
  commands: {
    style(world, args) {
      const loaded = og.load.style(world, `styles/${args[0]}/style.ogt`);
      if (!loaded.ok) throw new Error(loaded.reason);
      og.ui.configure(world, { style: loaded.value });
    },
  },
});
