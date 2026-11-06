import { og, defineSystem } from "@origingame/engine";

const Sheet = defineSystem("StyleSheet", {}, () => {
  og.ui.column("StyleSheet", { width: { kind: "grow" }, height: { kind: "grow" }, padding: { left: 24, right: 24, top: 24, bottom: 24 }, gap: 16, background: { token: "backdrop" } }, () => {
    og.ui.label("Title", "Style pack", { step: "xl2" });
    og.ui.label("Body", "Text, surfaces and controls", {});
    og.ui.button("Primary", "Primary", { variant: "primary" });
    og.ui.button("Secondary", "Secondary", { variant: "secondary" });
    og.ui.button("Disabled", "Disabled", { disabled: true });
  });
});

export default og.game({
  systems: [Sheet],
  commands: {
    style(world, args) {
      const loaded = og.load.style(world, `styles/${args[0]}/style.ogt`);
      if (!loaded.ok) throw new Error(loaded.reason);
      og.ui.configure(world, { style: loaded.value });
    },
  },
});
