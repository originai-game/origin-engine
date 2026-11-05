import { og, defineSystem } from "@origingame/engine";
import { fallbackIcons } from "../../icons.ts";
import manifest from "../../icons.json";

let group = "settings";
const Sheet = defineSystem("FallbackIcons", {}, () => {
  og.ui.column("Page", {
    width: { kind: "grow" }, height: { kind: "grow" },
    background: { token: "backdrop" }, padding: { left: 24, right: 24, top: 48, bottom: 24 }, gap: 12,
  }, () => {
    og.ui.label("Title", `Development fallbacks · ${group}`, { step: "lg" });
    og.ui.label("Notice", "Replace before publishing", { color: { token: "inkMuted" } });
    for (const icon of manifest.icons.filter(i => i.group === group)) {
      og.ui.row(icon.name, { gap: 12 }, () => {
        for (const size of [16, 24, 40]) og.ui.icon(`Size${size}`, icon.name, { size });
        og.ui.richText("Inline", `[icon=${icon.name}] ${icon.name}`);
      });
    }
  });
});

export default og.game({
  tick: 60, seed: 1, systems: [Sheet],
  start(world) {
    const style = og.load.style(world, "@styles/scifi/style.ogt");
    if (!style.ok) throw new Error(style.reason);
    const configured = og.ui.configure(world, { style: style.value, fallbackIcons });
    if (!configured.ok) throw new Error(configured.reason);
  },
  commands: {
    group: (_world, args) => {
      if (!manifest.icons.some(i => i.group === args[0])) throw new Error("Unknown icon group");
      group = args[0]!;
    },
  },
});
