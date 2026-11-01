// The world components (docs/ui.md §14, U6c) on the native host with the
// default og.Camera (at the origin, looking down -z), read through dump_ui.
//
// Ways they could fail, each covered below:
// - a nameplate keeps its title and name at every distance, or sheds them
//   for the target too;
// - a waypoint behind the camera leaves the screen instead of holding its
//   edge, or a ping outlives its time;
// - a prompt shows no glyph for an action without a binding, or a hold's
//   ring does not follow the hold;
// - a speech bubble shows its whole text before it types, or never leaves;
// - a loot label beams over a common item;
// - a ground shape is drawn off its centre's projection, or drawn when its
//   centre is behind the camera;
// - damage numbers spawn nothing, or a miss spawns a number.
import { expect, test } from "bun:test";
import { og } from "@origingame/engine";
import type { DamageHitDesc, World } from "@origingame/engine";
import { loadNativeHost } from "../src/node/index.ts";
import { at, dump, engineFor, type El } from "./support/ui.ts";

loadNativeHost();

const point = (x: number, y: number, z: number) => ({ target: { entity: null, point: { x, y, z } } });
const states = (el: El | undefined): Record<string, string> => el?.visualStates ?? {};
const DESKTOP = { width: 1280, height: 720, dpr: 1, safeArea: { left: 0, top: 0, right: 0, bottom: 0 }, coarsePointer: false, breakpoint: "desktop" };
const engine = (name: string, build: (w: World) => void) => engineFor(name, build, DESKTOP, [og.Camera]);

test("og.ui.nameplate: full near, title gone past the first distance, only the bar past the second, the target always full", () => {
  const e = engine("WorldPlates", () => {
    og.ui.nameplate("Near", { ...point(-3, 0, -10), name: "Grull", title: "<Ogres>", level: "12", health: 0.6 });
    og.ui.nameplate("Mid", { ...point(0, 0, -35), name: "Harn", title: "<Ogres>", health: 0.5 });
    og.ui.nameplate("Far", { ...point(3, 0, -60), name: "Ida", title: "<Ogres>", health: 0.4 });
    og.ui.nameplate("Target", { ...point(6, 0, -60), name: "Jory", title: "<Ogres>", health: 0.9, targeted: true });
  });
  e.advance(3);
  const els = dump(e);
  expect([at(els, "Near/Title")?.text, at(els, "Near/Head/Name")?.text, at(els, "Near")?.value]).toEqual(["<Ogres>", "Grull", "Level 12, 60% health"]);
  expect([at(els, "Mid/Title"), at(els, "Mid/Head/Name")?.text, states(at(els, "Mid")).lod]).toEqual([undefined, "Harn", "compact"]);
  expect([at(els, "Far/Head"), at(els, "Far/Bar") !== undefined, states(at(els, "Far")).lod]).toEqual([undefined, true, "bar"]);
  expect([at(els, "Target/Title")?.text, states(at(els, "Target")).selection]).toEqual(["<Ogres>", "targeted"]);
  // Above its point: the near plate's bottom sits over the projection of the point 2 m up.
  const near = at(els, "Near")!;
  expect(near.y + near.height).toBeLessThan(360);
  expect(near.x + near.width / 2).toBeLessThan(640);
});

test("og.ui.waypoint: on screen with its distance and label; behind the camera, held on the edge without its label; a ping leaves", () => {
  const e = engine("WorldWaypoints", () => {
    og.ui.waypoint("Ahead", { ...point(4, 0, -20), kind: "objective", label: "Defend" });
    og.ui.waypoint("Behind", { ...point(0, 0, 30), kind: "enemy", label: "Spotted" });
    og.ui.waypoint("Ping", { ...point(-2, 0, -8), kind: "ping", label: "Loot here", by: "Wraith", at: 0, durationMs: 1000 });
  });
  e.advance(3);
  let els = dump(e);
  expect(at(els, "Ahead/Label")?.text).toBe("Defend");
  expect(at(els, "Ahead/Distance")?.text).toMatch(/^2\dm$/);
  expect(states(at(els, "Ahead")).place).toBe("onScreen");
  const behind = at(els, "Behind")!;
  expect(states(behind).place).toBe("clamped");
  expect(at(els, "Behind/Label")).toBe(undefined);
  expect(behind.x >= 0 && behind.y >= 0 && behind.x + behind.width <= 1280 && behind.y + behind.height <= 720).toBe(true);
  expect(at(els, "Ping/Label")?.text).toBe("Loot here · Wraith");
  e.advance(70);
  els = dump(e);
  expect(at(els, "Ping")).toBe(undefined);
  expect(at(els, "Ahead")).toBeDefined();
});

test("og.ui.interactionPrompt and speechBubble: a key cap for an unbound action, an og.input action's binding, the hold's state; words type out, then leave", () => {
  let hold = 0.4;
  const e = engine("WorldPrompts", (w) => {
    og.ui.interactionPrompt(w, "Chest", { ...point(0, 0, -4), action: "no.such.action", verb: "Open", name: "Chest", hold });
    og.ui.interactionPrompt(w, "Door", { ...point(1.5, 0, -4), verb: "Open", disabled: true, reason: "Locked" });
    // A gameplay action's prompt shows its binding (og.input), not the fallback cap.
    og.input.defineAction(w, { name: "interact", bindings: [{ control: "KeyF" }, { control: "pad.x" }] });
    og.ui.interactionPrompt(w, "Lever", { ...point(-1.5, 0, -4), action: "interact", verb: "Pull" });
    og.ui.speechBubble("Say", { ...point(-1, 0, -6), speaker: "Tom Nook", text: "Hello there, friend!", at: 0, charsPerSecond: 20, durationMs: 500 });
  });
  e.advance(15);
  let els = dump(e);
  expect(at(els, "Chest/Frame/Words/Verb")?.text).toBe("Open");
  expect(at(els, "Chest/Frame/Glyph/Cap/Key")?.text).toBe("E");
  expect([at(els, "Chest")?.role, at(els, "Chest")?.name, states(at(els, "Chest")).use]).toEqual(["button", "Open Chest", "holding"]);
  expect([states(at(els, "Door")).use, at(els, "Door/Frame/Words/Reason")?.text]).toEqual(["disabled", "Locked"]);
  expect(at(els, "Lever")?.value).toBe("F");
  // Fifteen ticks at 20 a second: five characters typed; the screen reader has it all at once.
  const typed = at(els, "Say/Bubble/Text")?.text ?? "";
  expect(typed.length).toBeGreaterThan(2);
  expect(typed.length).toBeLessThan(10);
  expect(at(els, "Say")?.value).toBe("Hello there, friend!");
  expect(states(at(els, "Say")).speech).toBe("typing");
  hold = 1;
  e.advance(60);
  els = dump(e);
  expect(at(els, "Say/Bubble/Text")?.text).toBe("Hello there, friend!");
  expect(states(at(els, "Chest")).use).toBe("ready");
  e.advance(80);
  expect(at(dump(e), "Say")).toBe(undefined);
});

test("og.ui.lootLabel: a chip in its tier with pips, a beam only from rare up, the stack's count", () => {
  const e = engine("WorldLoot", () => {
    og.ui.lootLabel("Sword", { ...point(-1, 0, -6), name: "Doombringer", rarity: "legendary" });
    og.ui.lootLabel("Rags", { ...point(1, 0, -6), name: "Torn Rags", rarity: "common", count: 3 });
  });
  e.advance(3);
  const els = dump(e);
  expect(at(els, "Sword/Chip/Name")?.text).toBe("Doombringer");
  expect(at(els, "Sword/Beam")).toBeDefined();
  expect(els.filter((x) => x.path.startsWith("Sword/Chip/Pips/")).length).toBe(5);
  expect(at(els, "Sword/Chip")?.value).toBe("legendary");
  expect(at(els, "Rags/Beam")).toBe(undefined);
  expect(at(els, "Rags/Chip/Count")?.text).toBe("×3");
});

test("ground shapes sit on their centre's projection, and are not drawn behind the camera", () => {
  const e = engine("WorldGround", (w) => {
    og.ui.selectionRing(w, "Ring", { center: { x: 0, y: -1, z: -8 }, radius: 1 });
    og.ui.selectionRing(w, "Gone", { center: { x: 0, y: -1, z: 8 }, radius: 1 });
    og.ui.telegraph(w, "Cone", { center: { x: 2, y: -1, z: -10 }, shape: "cone", radius: 4, angle: 1.2, direction: Math.PI, progress: 0.5 });
    og.ui.telegraph(w, "Donut", { center: { x: -3, y: -1, z: -12 }, shape: "ring", inner: 2, radius: 4, progress: 1 });
    og.ui.pathLine(w, "Path", { points: [{ x: 0, y: -1, z: -4 }, { x: 2, y: -1, z: -8 }, { x: 2, y: -1, z: -14 }], reach: 6, label: "12 m · 2 AP" });
    og.ui.placementGhost(w, "Ghost", { center: { x: -2, y: -1, z: -9 }, width: 3, depth: 2, cell: 1, valid: false, label: "Blocked", range: 5 });
  });
  e.advance(3);
  const els = dump(e);
  const ring = at(els, "Ring")!;
  // The ring's centre (0, -1, -8) projects to (640, 360 + 1/8 of the focal length): its element is centred there.
  expect(Math.abs(ring.x + ring.width / 2 - 640)).toBeLessThan(1.5);
  expect(ring.y + ring.height / 2).toBeGreaterThan(360);
  expect(ring.width).toBeGreaterThan(ring.height * 2);
  expect(at(els, "Gone")).toBe(undefined);
  expect(at(els, "Cone/Box/Fill#0")).toBeDefined();
  expect(states(at(els, "Cone")).windup).toBe("charging");
  expect(states(at(els, "Donut")).windup).toBe("impact");
  expect(els.filter((x) => /^Donut\/Box\/Area#\d+$/.test(x.path)).length).toBe(32);
  expect(at(els, "Path/Box/Beyond")).toBeDefined();
  expect(at(els, "Path/Box/Label/Text")?.text).toBe("12 m · 2 AP");
  expect(states(at(els, "Ghost")).place).toBe("invalid");
  expect(els.some((x) => x.path.startsWith("Ghost/Box/Hatch#"))).toBe(true);
  expect(at(els, "Ghost/Box/Label")?.name).toBe("Blocked");
});

test("og.ui.damageNumbers and overheadBars: hits spawn mass elements that live out their time; a miss is a word", () => {
  let hits: DamageHitDesc[] = [];
  let counts: number[] = [];
  const e = engine("WorldMass", () => {
    og.ui.damageNumbers("Dmg", { hits });
    og.ui.overheadBars("Bars", { bars: [{ key: "a", position: { x: 0, y: 1, z: -10 }, value: 0.4 }, { key: "b", position: { x: 1, y: 1, z: -10 }, value: 1 }], hideFull: true });
    const n = og.ui.memory("Dmg.number~mass", -1);
    const w = og.ui.memory("Dmg.word~mass", -1);
    counts = [n >= 0 ? og.ui.massCount(n) : -1, w >= 0 ? og.ui.massCount(w) : -1];
    hits = [];
  });
  hits = [
    { position: { x: 0, y: 1, z: -6 }, amount: 120 },
    { position: { x: 0, y: 1, z: -6 }, amount: 48210, kind: "critical", seed: 1 },
    { position: { x: 0, y: 1, z: -6 }, kind: "miss", word: "Dodge" },
  ];
  e.advance(1);
  expect(counts).toEqual([2, 1]);
  e.advance(120);
  expect(counts).toEqual([0, 0]);
});
