import { og, defineSystem } from "@origingame/engine";
import { fallbackIcons } from "../../icons.ts";
import manifest from "../../icons.json";

let group = "settings";
// Names the sheet's scifi atlas paints and the fallback set also has: the
// atlas keeps them (UiDesc.fallbackIcons), which the precedence row shows
// beside the fallback's own drawing, aliased as `fallback:<name>`.
const PAINTED = ["heart", "coin", "home", "trophy"];
const TINTS = ["primary", "danger", "gold", "success"] as const;
const sheetIcons = {
  ...fallbackIcons,
  vectors: [
    ...(fallbackIcons.vectors ?? []),
    ...PAINTED.map(name => ({ ...fallbackIcons.vectors!.find(v => v.name === name)!, name: `fallback:${name}` })),
  ],
};
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
    const first = manifest.icons.find(i => i.group === group)!.name;
    og.ui.row("Tinted", { gap: 12 }, () => {
      for (const token of TINTS) og.ui.icon(`Tint-${token}`, first, { size: 40, tint: { token } } as never);
      og.ui.label("Caption", `${first}, tinted primary, danger, gold, success`, { color: { token: "inkMuted" } });
    });
    og.ui.row("AtlasFirst", { gap: 12 }, () => {
      for (const name of PAINTED) {
        og.ui.icon(`Atlas-${name}`, name, { size: 40 });
        og.ui.icon(`Fallback-${name}`, `fallback:${name}`, { size: 40 });
      }
      og.ui.label("Caption", "the style's painted icon, then the fallback of the same name", { color: { token: "inkMuted" } });
    });
  });
});

export default og.game({
  tick: 60, seed: 1, systems: [Sheet],
  start(world) {
    const style = og.load.style(world, "@styles/scifi/style.ogt");
    if (!style.ok) throw new Error(style.reason);
    const configured = og.ui.configure(world, { style: style.value, fallbackIcons: sheetIcons });
    if (!configured.ok) throw new Error(configured.reason);
  },
  commands: {
    group: (_world, args) => {
      if (!manifest.icons.some(i => i.group === args[0])) throw new Error("Unknown icon group");
      group = args[0]!;
    },
  },
});
