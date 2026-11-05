// Fixtures for components the existing kit pages do not call directly.
// Only the fixture data is authored here; names, documentation and signatures
// in the catalogue continue to come from reflection and extension sources.
import { og, defineSystem, type World, type Entity } from "@origingame/engine";
import { racing } from "@origingame/engine/extensions/racing/index.ts";
import { strategy } from "@origingame/engine/extensions/strategy/index.ts";

let page = "abilitySlot";
let shownAt = 0;
let speaker: Entity | undefined;
const point = { x: 0, y: 0, z: -6 };
const center = { x: 0, y: -1, z: -6 };
const commands = [{ key: "move", name: "Move", hotkey: "M" }, { key: "hold", name: "Hold position", hotkey: "H" }];
const selection = { units: [{ key: "scout", name: "Scout", health: 72, healthMax: 100 }] };

export const FIXTURES: Record<string, (world: World) => void> = {
  abilitySlot: () => og.ui.abilitySlot("Sample", { name: "Dash", key: "dash", hotkey: "Q", cooldown: 3, cooldownMax: 8, charges: 1, maxCharges: 2 }),
  banner: () => og.ui.banner("Sample", { title: "Wave complete", subtitle: "Prepare for the next wave", shownAt, durationMs: 10000 }),
  celebration: () => og.ui.celebration("Sample", { kind: "levelUp", title: "LEVEL UP", value: "12", at: shownAt, durationMs: 10000 }),
  countBadge: () => og.ui.countBadge("Sample", { count: 12, corner: "center" }),
  flightHud: () => og.ui.flightHud("Sample", { speed: 240, altitude: 3200, heading: 125, pitch: 4, roll: -8 }),
  interactionPrompt: world => og.ui.interactionPrompt(world, "Sample", { target: { point }, verb: "Open", name: "Chest" }),
  keyBindingRow: world => og.ui.keyBindingRow(world, "Sample", { action: "interact", label: "Interact" }),
  lootLabel: () => og.ui.lootLabel("Sample", { target: { point }, name: "Ancient key", rarity: "rare", count: 2 }),
  notifications: () => og.ui.notifications("Sample", { items: [{ key: "quest", title: "Quest complete", body: "The road is open", sticky: true, points: "+120 XP" }] }),
  overheadBars: () => og.ui.overheadBars("Sample", { bars: [{ key: "scout", position: point, value: 0.72 }], reference: 0, width: 160 }),
  pathLine: world => og.ui.pathLine(world, "Sample", { points: [{ x: -2, y: -1, z: -8 }, center, { x: 2, y: -1, z: -5 }], label: "Route", arrow: true }),
  placementGhost: world => og.ui.placementGhost(world, "Sample", { center, width: 3, depth: 2, cell: 1, valid: true, label: "Place tower" }),
  progressBar: () => og.ui.progressBar("Sample", { value: 0.65, buffer: 0.82, label: "Loading", showValue: true }),
  progressRing: () => og.ui.progressRing("Sample", { value: 0.65, size: 96, label: "65%" }),
  "racing.rearView": () => racing.rearView("Sample", { width: 480 }),
  ribbon: () => og.ui.ribbon("Sample", { text: "NEW", corner: "top" }),
  selectionRing: world => og.ui.selectionRing(world, "Sample", { center, radius: 1.5 }),
  skeletonGroup: () => og.ui.skeletonGroup("Sample", { shape: "card", lines: 3, height: 180 }),
  speechBubble: () => og.ui.speechBubble("Sample", { target: { point }, speaker: "Scout", text: "The bridge is clear.", at: shownAt, durationMs: 10000 }),
  starRating: () => og.ui.starRating("Sample", { earned: 2, max: 3, size: 48 }),
  "strategy.commandCard": () => strategy.commandCard("Sample", { commands }),
  "strategy.dragSelect": () => strategy.dragSelect("Sample", { from: { x: 420, y: 240 }, to: { x: 780, y: 470 } }),
  "strategy.productionQueue": () => strategy.productionQueue("Sample", { items: [{ key: "scout", name: "Scout", remaining: 7, duration: 20 }], slots: 5 }),
  "strategy.rtsConsole": () => strategy.rtsConsole("Sample", { selection, commands: { commands } }),
  "strategy.selectionPanel": () => strategy.selectionPanel("Sample", selection),
  subtitles: () => og.ui.subtitles("Sample", { lines: [{ key: "line", speaker: "Scout", text: "Follow the river to the old bridge." }] }),
  telegraph: world => og.ui.telegraph(world, "Sample", { center, radius: 2, progress: 0.65, hostile: true }),
  tooltipCard: () => {
    og.ui.pushButton("Target", "Ancient key");
    og.ui.tooltipCard("Sample", { target: "Fixture/Target", open: true, title: "Ancient key", body: "Opens the northern gate.", rarity: "rare", delayMs: 0 });
  },
  barks: () => {
    if (speaker === undefined) return;
    og.ui.barks("Sample", { barks: [{ entity: speaker, dialogue: "sample", bark: "clear", seq: 1, key: "", args: {}, text: "The bridge is clear!", speaker: { id: "scout", name: "Scout", entity: speaker, portrait: "", expression: "", side: "left" }, voice: "", voiceLeft: 0, ticksLeft: 600, priority: 1 }] });
  },
  idleGenerator: () => og.ui.idleGenerator("Sample", { name: "Workshop", level: 3, ownedText: "12", rateText: "+25 / s", cycle: 0.65, costText: "120 coins", affordable: true }),
  settingsTree: world => og.ui.settingsTree(world, "Sample", { title: "Settings", page: "audio", pages: [{ key: "audio", label: "Audio", entries: [{ key: "music", kind: "slider", label: "Music", value: 0.7, max: 1, unit: "%" }, { key: "captions", label: "Captions", on: true }] }] }, () => {}),
};

const Samples = defineSystem("CatalogueSamples", { reads: [og.Camera] }, (_q, _t, world) => {
  og.ui.column("Ground", { width: { kind: "grow" }, height: { kind: "grow" }, background: { token: "backdrop" } }, () => {});
  og.ui.anchor("Fixture", { anchor: "center", width: { kind: "fixed", px: 560 }, height: { kind: "fixed", px: 320 }, gap: 12 }, () => FIXTURES[page]!(world));
});

export default og.game({
  systems: [Samples],
  start(world) {
    const style = og.load.style(world, "@styles/scifi/style.ogt");
    if (!style.ok) throw new Error(style.reason);
    og.ui.configure(world, { style: style.value });
    const entity = world.spawn({ name: "scout" });
    if (!entity.ok) throw new Error(entity.reason);
    speaker = entity.value;
    world.insert(speaker, og.Transform, { position: point });
  },
  commands: {
    page: (_world, args) => {
      if (!args[0] || !FIXTURES[args[0]]) throw new Error(`Unknown fixture ${args[0]}`);
      page = args[0];
      shownAt = og.ui.tick();
    },
  },
});
