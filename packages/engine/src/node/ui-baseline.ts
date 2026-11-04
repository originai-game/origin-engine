// Fixed review pages, salvaged from ui-kit/tests/baseline.ts (aa85693ff).
// Each cell starts a fresh engine, so each page establishes all its state.
export const BASELINE: ReadonlyArray<{ name: string; commands: string; root: string }> = [
  ...["widgets", "foundations", "controls", "fields", "ranges", "navigation", "containers", "data"].map((name) => ({ name, commands: `show:${name}`, root: "Back" })),
  { name: "overlays-menu", commands: "show:overlays;demo:overlay:menu", root: "Back" },
  { name: "overlays-dialog", commands: "show:overlays;demo:overlay:dialog", root: "Back" },
  ...["gear", "store", "gacha", "rewards", "dialogue", "quests", "lobby"].map((name) => ({ name, commands: `show:${name};pat:reset`, root: "Back" })),
  { name: "settings", commands: "show:settingsTree", root: "Back" },
  { name: "pause", commands: "show:actionPause", root: "pause" },
  { name: "results", commands: "show:actionResults", root: "results" },
  ...["actionHud", "shooterHud", "rpgHud", "mmoHud", "survivalHud", "raceHud"].map((name) => ({ name, commands: `show:${name};sc:reset`, root: "Back" })),
].map((page) => ({ ...page, commands: `demo:ground:pack;${page.commands}` }));
